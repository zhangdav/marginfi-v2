use super::price::OraclePriceFeedAdapter;
use crate::constants::{
    ASSET_TAG_DEFAULT, ASSET_TAG_SOL, ASSET_TAG_STAKED, BANKRUPT_THRESHOLD,
    EMISSIONS_FLAG_BORROW_ACTIVE, EMISSIONS_FLAG_LENDING_ACTIVE, EMPTY_BALANCE_THRESHOLD,
    EXP_10_I80F48, MIN_EMISSIONS_START_TIME, SECONDS_PER_YEAR, ZERO_AMOUNT_THRESHOLD,
};
use crate::errors::MarginfiError;
use crate::prelude::MarginfiResult;
use crate::state::emode::{reconcile_emode_configs, EmodeConfig};
use crate::state::health_cache::HealthCache;
use crate::state::marginfi_group::{Bank, RiskTier, WrappedI80F48};
use crate::state::price::PriceAdapter;
use crate::state::price::{OraclePriceType, PriceBias};
use crate::utils::NumTraitsWithTolerance;
use crate::{assert_struct_align, assert_struct_size};
use crate::{check, check_eq, debug, math_error};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use std::cmp::{max, min};
use type_layout::TypeLayout;

pub const ACCOUNT_IN_FLASHLOAN: u64 = 1 << 1;
pub const ACCOUNT_DISABLED: u64 = 1 << 0;

fn get_remaining_accounts_per_balance(balance: &Balance) -> MarginfiResult<usize> {
    get_remaining_accounts_per_balance_with_tag(balance.bank_asset_tag)
}

fn get_remaining_accounts_per_balance_with_tag(asset_tag: u8) -> MarginfiResult<usize> {
    match asset_tag {
        ASSET_TAG_DEFAULT | ASSET_TAG_SOL => Ok(2),
        ASSET_TAG_STAKED => Ok(4),
        _ => err!(MarginfiError::AssetTagMismatch),
    }
}

#[derive(Debug)]
pub enum BalanceIncreaseType {
    Any,
    RepayOnly,
    DepositOnly,
    BypassDepositLimit,
}

#[derive(Debug)]
pub enum BalanceDecreaseType {
    Any,
    WithdrawOnly,
    BorrowOnly,
    BypassBorrowLimit,
}

pub enum RiskRequirementType {
    Initial,
    Maintenance,
    Equity,
}

impl RiskRequirementType {
    pub fn to_weight_type(&self) -> RequirementType {
        match self {
            RiskRequirementType::Initial => RequirementType::Initial,
            RiskRequirementType::Maintenance => RequirementType::Maintenance,
            RiskRequirementType::Equity => RequirementType::Equity,
        }
    }
}

pub struct BankAccountWithPriceFeed<'a, 'info> {
    bank: AccountLoader<'info, Bank>,
    price_feed: Box<MarginfiResult<OraclePriceFeedAdapter>>,
    balance: &'a Balance,
}

impl<'info> BankAccountWithPriceFeed<'_, 'info> {
    pub fn load<'a>(
        lending_account: &'a LendingAccount,
        remaining_ais: &'info [AccountInfo<'info>],
    ) -> MarginfiResult<Vec<BankAccountWithPriceFeed<'a, 'info>>> {
        let clock = Clock::get()?;
        let mut account_index = 0;

        lending_account
            .balances
            .iter()
            .filter(|balance| balance.is_active())
            .map(|balance| {
                let bank_ai: Option<&AccountInfo<'info>> = remaining_ais.get(account_index);
                if bank_ai.is_none() {
                    msg!("Ran out of remaining accoubnts at {:?}", account_index);
                    return err!(MarginfiError::InvalidBankAccount);
                }
                let bank_ai = bank_ai.unwrap();
                let bank_al = AccountLoader::<Bank>::try_from(bank_ai)?;

                let num_accounts = get_remaining_accounts_per_balance(balance)?;
                check_eq!(
                    balance.bank_pk,
                    *bank_ai.key,
                    MarginfiError::InvalidBankAccount
                );
                let bank = bank_al.load()?;

                let oracle_ai_idx = account_index + 1;
                let oracle_ais = &remaining_ais[oracle_ai_idx..oracle_ai_idx + num_accounts - 1];

                let price_adapter = Box::new(OraclePriceFeedAdapter::try_from_bank_config(
                    &bank.config,
                    oracle_ais,
                    &clock,
                ));

                account_index += num_accounts;

                Ok(BankAccountWithPriceFeed {
                    bank: bank_al.clone(),
                    price_feed: price_adapter,
                    balance,
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    #[inline(always)]
    /// Calculate the value of the balance, which is either an asset or a liability. If it is an
    /// asset, returns (asset_value, 0, price, 0), and if it is a liability, returns (0, liabilty
    /// value, price, 0), where price is the actual oracle price used to determine the value after
    /// bias adjustments, etc.
    ///
    /// err_code is an internal oracle error code in the event the oracle did not load. This applies
    /// only to assets and the return type will always be (0, 0, 0, err_code).
    ///
    /// Nuances:
    /// 1. Maintenance requirement is calculated using the real time price feed.
    /// 2. Initial requirement is calculated using the time weighted price feed, if available.
    /// 3. Initial requirement is discounted by the initial discount, if enabled and the usd limit
    ///    is exceeded.
    /// 4. Assets are only calculated for collateral risk tier.
    /// 5. Oracle errors are ignored for deposits in isolated risk tier.
    fn calc_weighted_value(
        &self,
        requirement_type: RequirementType,
        emode_config: &EmodeConfig,
    ) -> MarginfiResult<(I80F48, I80F48, I80F48, u32)> {
        match self.balance.get_side() {
            Some(side) => {
                let bank = &self.bank.load()?;

                match side {
                    BalanceSide::Assets => {
                        let (value, price, err_code) =
                            self.calc_weighted_asset_value(requirement_type, bank, emode_config)?;
                        Ok((value, I80F48::ZERO, price, err_code))
                    }
                    BalanceSide::Liabilities => {
                        let (value, price) =
                            self.calc_weighted_liab_value(requirement_type, bank)?;
                        Ok((I80F48::ZERO, value, price, 0))
                    }
                }
            }
            None => Ok((I80F48::ZERO, I80F48::ZERO, I80F48::ZERO, 0)),
        }
    }

    /// Returns value, the net asset value in $, and the price used to determine that value. In most
    /// cases, returns (value, price, 0). If there was an error loading the price feed, treats the
    /// price as zero, and passes the u32 argument that contains the error code, i.e. the return
    /// type is (0, 0, err_code). Other types of errors (e.g. math) will still throw.
    #[inline(always)]
    fn calc_weighted_asset_value(
        &self,
        requirement_type: RequirementType,
        bank: &Bank,
        emode_config: &EmodeConfig,
    ) -> MarginfiResult<(I80F48, I80F48, u32)> {
        match bank.config.risk_tier {
            RiskTier::Collateral => {
                let (price_feed, err_code) = self.try_get_price_feed();

                if matches!(
                    (&price_feed, requirement_type),
                    (&Err(_), RequirementType::Initial)
                ) {
                    debug!("Skipping stale oracle");
                    return Ok((I80F48::ZERO, I80F48::ZERO, err_code));
                }

                let price_feed = price_feed?;

                // If an emode entry exists for this bank's emode tag in the reconciled config of
                // all borrowing banks, use its weight, otherwise use the weight designated on the
                // collateral bank itself. If the bank's weight is higher, always use that weight.
                let mut asset_weight =
                    if let Some(emode_entry) = emode_config.find_with_tag(bank.emode.emode_tag) {
                        let bank_weight = bank
                            .config
                            .get_weight(requirement_type, BalanceSide::Assets);
                        let emode_weight = match requirement_type {
                            RequirementType::Initial => I80F48::from(emode_entry.asset_weight_init),
                            RequirementType::Maintenance => {
                                I80F48::from(emode_entry.asset_weight_maint)
                            }
                            // Note: For equity (which is only used for bankruptcies) emode does not
                            // apply, as the asset weight is always 1
                            RequirementType::Equity => I80F48::ONE,
                        };
                        max(bank_weight, emode_weight)
                    } else {
                        bank.config
                            .get_weight(requirement_type, BalanceSide::Assets)
                    };

                let lower_price = price_feed.get_price_of_type(
                    requirement_type.get_oracle_price_type(),
                    Some(PriceBias::Low),
                    bank.config.oracle_max_confidence,
                )?;

                if matches!(requirement_type, RequirementType::Initial) {
                    if let Some(discount) =
                        bank.maybe_get_asset_weight_init_discount(lower_price)?
                    {
                        asset_weight = asset_weight
                            .checked_mul(discount)
                            .ok_or_else(math_error!())?;
                    }
                }

                let value = calc_value(
                    bank.get_asset_amount(self.balance.asset_shares.into())?,
                    lower_price,
                    bank.mint_decimals,
                    Some(asset_weight),
                )?;

                Ok((value, lower_price, 0))
            }
            RiskTier::Isolated => Ok((I80F48::ZERO, I80F48::ZERO, 0)),
        }
    }

    #[inline(always)]
    fn calc_weighted_liab_value(
        &self,
        requirement_type: RequirementType,
        bank: &Bank,
    ) -> MarginfiResult<(I80F48, I80F48)> {
        let (price_feed, _) = self.try_get_price_feed();
        let price_feed = price_feed?;
        let liability_weight = bank
            .config
            .get_weight(requirement_type, BalanceSide::Liabilities);

        let higher_price = price_feed.get_price_of_type(
            requirement_type.get_oracle_price_type(),
            Some(PriceBias::High),
            bank.config.oracle_max_confidence,
        )?;

        let value = calc_value(
            bank.get_liability_amount(self.balance.liability_shares.into())?,
            higher_price,
            bank.mint_decimals,
            Some(liability_weight),
        )?;

        Ok((value, higher_price))
    }

    fn try_get_price_feed(&self) -> (MarginfiResult<&OraclePriceFeedAdapter>, u32) {
        match self.price_feed.as_ref() {
            Ok(a) => (Ok(a), 0),
            #[allow(unused_variables)]
            Err(e) => match e {
                anchor_lang::error::Error::AnchorError(inner) => {
                    let error_code = inner.as_ref().error_code_number;
                    let custom_error = MarginfiError::from(error_code);
                    (Err(error!(custom_error)), error_code)
                }
                anchor_lang::error::Error::ProgramError(inner) => {
                    match inner.as_ref().program_error {
                        ProgramError::Custom(error_code) => {
                            let custom_error = MarginfiError::from(error_code);
                            (Err(error!(custom_error)), error_code)
                        }
                        _ => (
                            Err(error!(MarginfiError::InternalLogicError)),
                            MarginfiError::InternalLogicError as u32,
                        ),
                    }
                }
            },
        }
    }

    #[inline]
    pub fn is_empty(&self, side: BalanceSide) -> bool {
        self.balance.is_empty(side)
    }
}

assert_struct_size!(MarginfiAccount, 2304);
assert_struct_align!(MarginfiAccount, 8);
#[account(zero_copy)]
#[repr(C)]
#[derive(PartialEq, Eq, TypeLayout)]
pub struct MarginfiAccount {
    pub group: Pubkey,
    pub authority: Pubkey,
    pub lending_account: LendingAccount,
    pub account_flags: u64,
    pub emissions_destination_account: Pubkey,
    pub health_cache: HealthCache,
    pub migrated_from: Pubkey,
    pub migrated_to: Pubkey,
    pub _padding: [u64; 13],
}

impl MarginfiAccount {
    /// Set the initial data for the marginfi account.
    pub fn initialize(&mut self, group: Pubkey, authority: Pubkey) {
        self.authority = authority;
        self.group = group;
        self.emissions_destination_account = Pubkey::default();
        self.migrated_from = Pubkey::default();
        self.migrated_to = Pubkey::default();
    }

    pub fn get_flag(&self, flag: u64) -> bool {
        self.account_flags & flag != 0
    }

    pub fn set_flag(&mut self, flag: u64) {
        msg!("Setting account flag {:b}", flag);
        self.account_flags |= flag;
    }
}

pub const MAX_LENDING_ACCOUNT_BALANCES: usize = 16;

assert_struct_size!(LendingAccount, 1728);
assert_struct_align!(LendingAccount, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Copy, Clone, Zeroable, Pod, PartialEq, Eq, TypeLayout,
)]
pub struct LendingAccount {
    pub balances: [Balance; MAX_LENDING_ACCOUNT_BALANCES],
    pub _padding: [u64; 8],
}

/// TODO: LendingAccount impl

assert_struct_size!(Balance, 104);
assert_struct_align!(Balance, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Copy, Clone, Zeroable, Pod, PartialEq, Eq, TypeLayout,
)]
pub struct Balance {
    pub active: u8,
    pub bank_pk: Pubkey,
    pub bank_asset_tag: u8,
    pub _pad0: [u8; 6],
    pub asset_shares: WrappedI80F48,
    pub liability_shares: WrappedI80F48,
    pub emissions_outstanding: WrappedI80F48,
    pub last_update: u64,
    pub _padding: [u64; 1],
}

impl Balance {
    pub fn is_active(&self) -> bool {
        self.active != 0
    }

    pub fn get_side(&self) -> Option<BalanceSide> {
        let asset_shares = I80F48::from(self.asset_shares);
        let liability_shares = I80F48::from(self.liability_shares);

        assert!(
            asset_shares < EMPTY_BALANCE_THRESHOLD || liability_shares < EMPTY_BALANCE_THRESHOLD
        );

        if I80F48::from(self.liability_shares) >= EMPTY_BALANCE_THRESHOLD {
            Some(BalanceSide::Liabilities)
        } else if I80F48::from(self.asset_shares) >= EMPTY_BALANCE_THRESHOLD {
            Some(BalanceSide::Assets)
        } else {
            None
        }
    }

    pub fn change_liability_shares(&mut self, delta: I80F48) -> MarginfiResult {
        let liability_shares: I80F48 = self.liability_shares.into();
        self.liability_shares = liability_shares
            .checked_add(delta)
            .ok_or_else(math_error!())?
            .into();
        Ok(())
    }

    pub fn change_asset_shares(&mut self, delta: I80F48) -> MarginfiResult {
        let asset_shares: I80F48 = self.asset_shares.into();
        self.asset_shares = asset_shares
            .checked_add(delta)
            .ok_or_else(math_error!())?
            .into();
        Ok(())
    }

    #[inline]
    pub fn is_empty(&self, side: BalanceSide) -> bool {
        let shares: I80F48 = match side {
            BalanceSide::Assets => self.asset_shares,
            BalanceSide::Liabilities => self.liability_shares,
        }
        .into();

        shares < EMPTY_BALANCE_THRESHOLD
    }
}

// Convert a token quantity to USD value with 10⁻⁸ precision (I80F48 fixed-point format) at the current price
#[inline]
pub fn calc_value(
    amount: I80F48,
    price: I80F48,
    mint_decimals: u8,
    weight: Option<I80F48>,
) -> MarginfiResult<I80F48> {
    if amount == I80F48::ZERO {
        return Ok(I80F48::ZERO);
    }

    let scaling_factor = EXP_10_I80F48[mint_decimals as usize];

    let weighted_asset_amount = if let Some(weight) = weight {
        amount.checked_mul(weight).unwrap()
    } else {
        amount
    };

    #[cfg(target_os = "solana")]
    crate::debug!(
        "weighted_asset_qt: {}, price: {}, expo: {}",
        weight_asset_amount,
        price,
        mint_decimals
    );
    let value = weighted_asset_amount
        .checked_mul(price)
        .ok_or_else(math_error!())?
        .checked_div(scaling_factor)
        .ok_or_else(math_error!())?;

    Ok(value)
}

#[derive(Copy, Clone)]
pub enum RequirementType {
    Initial,
    Maintenance,
    Equity,
}

impl RequirementType {
    /// Get oracle price type for the requirement type.
    ///
    /// Initial and equity requirements use the time weighted price feed.
    /// Maintenance requirement uses the real time price feed, as its more accurate for triggering liquidations.
    /// Choosing the right oracle price type for different uses (stable vs. accurate)
    pub fn get_oracle_price_type(&self) -> OraclePriceType {
        match self {
            RequirementType::Initial | RequirementType::Equity => OraclePriceType::TimeWeighted,
            RequirementType::Maintenance => OraclePriceType::RealTime,
        }
    }
}

pub enum BalanceSide {
    Assets,
    Liabilities,
}

pub struct RiskEngine<'a, 'info> {
    marginfi_account: &'a MarginfiAccount,
    bank_accounts_with_price: Vec<BankAccountWithPriceFeed<'a, 'info>>,
    emode_config: EmodeConfig,
}

impl<'info> RiskEngine<'_, 'info> {
    pub fn new<'a>(
        marginfi_account: &'a MarginfiAccount,
        remaining_ais: &'info [AccountInfo<'info>],
    ) -> MarginfiResult<RiskEngine<'a, 'info>> {
        check!(
            !marginfi_account.get_flag(ACCOUNT_IN_FLASHLOAN),
            MarginfiError::AccountInFlashloan
        );

        Self::new_no_flashloan_check(marginfi_account, remaining_ais)
    }

    fn new_no_flashloan_check<'a>(
        marginfi_account: &'a MarginfiAccount,
        remaining_ais: &'info [AccountInfo<'info>],
    ) -> MarginfiResult<RiskEngine<'a, 'info>> {
        let bank_accounts_with_price =
            BankAccountWithPriceFeed::load(&marginfi_account.lending_account, remaining_ais)?;

        let reconciled_emode_config = reconcile_emode_configs(
            bank_accounts_with_price
                .iter()
                .filter(|b| !b.balance.is_empty(BalanceSide::Liabilities))
                .map(|b| b.bank.load().unwrap().emode.emode_config),
        );

        Ok(RiskEngine {
            marginfi_account,
            bank_accounts_with_price,
            emode_config: reconciled_emode_config,
        })
    }

    pub fn check_account_bankrupt(
        &self,
        health_cache: &mut Option<&mut HealthCache>,
    ) -> MarginfiResult {
        let (total_assets, total_liabilities) =
            self.get_account_health_components(RiskRequirementType::Equity, health_cache)?;

        check!(
            !self.marginfi_account.get_flag(ACCOUNT_IN_FLASHLOAN),
            MarginfiError::AccountInFlashloan
        );

        msg!(
            "check_bankrupt: assets {} - liabs: {}",
            total_assets,
            total_liabilities
        );

        check!(
            total_assets < total_liabilities,
            MarginfiError::AccountNotBankrupt
        );
        check!(
            total_assets < BANKRUPT_THRESHOLD && total_liabilities > ZERO_AMOUNT_THRESHOLD,
            MarginfiError::AccountNotBankrupt
        );

        Ok(())
    }

    /// Returns the total assets and liabilities of the account in the form of (assets, liabilities)
    pub fn get_account_health_components(
        &self,
        requirement_type: RiskRequirementType,
        health_cache: &mut Option<&mut HealthCache>,
    ) -> MarginfiResult<(I80F48, I80F48)> {
        let mut total_assets: I80F48 = I80F48::ZERO;
        let mut total_liabilities: I80F48 = I80F48::ZERO;
        const NO_INDEX_FOUND: usize = 255;
        let mut first_err_index = NO_INDEX_FOUND;

        for (i, bank_account) in self.bank_accounts_with_price.iter().enumerate() {
            let requirement_type = requirement_type.to_weight_type();
            let (asset_val, liab_val, price, err_code) =
                bank_account.calc_weighted_value(requirement_type, &self.emode_config)?;
            if err_code != 0 && first_err_index == NO_INDEX_FOUND {
                first_err_index = i;
                if let Some(cache) = health_cache {
                    cache.err_index = i as u8;
                    cache.internal_err = err_code;
                };
            }

            if let Some(health_cache) = health_cache {
                // Note: We only record the Initial weighted price in cache, at some point we may
                // record others.
                if let RequirementType::Initial = requirement_type {
                    health_cache.prices[i] = price.to_num::<f64>().to_le_bytes();
                }
            }

            debug!(
                "Balance {}, assets: {}, liabilities: {}",
                bank_account.balance.bank_pk, asset_val, liab_val
            );

            total_assets = total_assets
                .checked_add(asset_val)
                .ok_or_else(math_error!())?;
            total_liabilities = total_liabilities
                .checked_add(liab_val)
                .ok_or_else(math_error!())?;
        }

        if let Some(health_cache) = health_cache {
            match requirement_type {
                RiskRequirementType::Initial => {
                    health_cache.asset_value = total_assets.into();
                    health_cache.liability_value = total_liabilities.into();
                }
                RiskRequirementType::Maintenance => {
                    health_cache.asset_value_maint = total_assets.into();
                    health_cache.liability_value_maint = total_liabilities.into();
                }
                RiskRequirementType::Equity => {
                    health_cache.asset_value_equity = total_assets.into();
                    health_cache.liability_value_equity = total_liabilities.into();
                }
            }
        }

        Ok((total_assets, total_liabilities))
    }
}

pub struct BankAccountWrapper<'a> {
    pub balance: &'a mut Balance,
    pub bank: &'a mut Bank,
}

impl<'a> BankAccountWrapper<'a> {
    // Find existing user lending account balance by bank address.
    pub fn find(
        bank_pk: &Pubkey,
        bank: &'a mut Bank,
        lending_account: &'a mut LendingAccount,
    ) -> MarginfiResult<BankAccountWrapper<'a>> {
        let balance = lending_account
            .balances
            .iter_mut()
            .find(|balance| balance.is_active() && balance.bank_pk.eq(bank_pk))
            .ok_or_else(|| error!(MarginfiError::BankAccountNotFound))?;

        Ok(Self { balance, bank })
    }

    /// Repay a liability, will error if there is not enough liability - depositing is not allowed.
    pub fn repay(&mut self, amount: I80F48) -> MarginfiResult {
        self.increase_balance_internal(amount, BalanceIncreaseType::RepayOnly)
    }

    fn increase_balance_internal(
        &mut self,
        balance_delta: I80F48,
        operation_type: BalanceIncreaseType,
    ) -> MarginfiResult {
        debug!(
            "Balance increase: {} {type: {:?}",
            balance_delta, operation_type
        );

        self.claim_emissions(Clock::get()?.unix_timestamp as u64)?;

        let balance = &mut self.balance;
        let bank = &mut self.bank;
        // Record if the balance was an asset/liability beforehand
        let had_assets =
            I80F48::from(balance.asset_shares).is_positive_with_tolerance(ZERO_AMOUNT_THRESHOLD);
        let had_liabs = I80F48::from(balance.liability_shares)
            .is_positive_with_tolerance(ZERO_AMOUNT_THRESHOLD);

        let current_liability_shares: I80F48 = balance.liability_shares.into();
        let current_liability_amount = bank.get_liability_amount(current_liability_shares)?;

        let (liability_amount_decrease, asset_amount_increase) = (
            min(current_liability_amount, balance_delta),
            max(
                balance_delta
                    .checked_sub(current_liability_amount)
                    .ok_or_else(math_error!())?,
                I80F48::ZERO,
            ),
        );

        match operation_type {
            BalanceIncreaseType::RepayOnly => {
                check!(
                    asset_amount_increase.is_zero_with_tolerance(ZERO_AMOUNT_THRESHOLD),
                    MarginfiError::OperationRepayOnly
                );
            }
            BalanceIncreaseType::DepositOnly => {
                check!(
                    liability_amount_decrease.is_zero_with_tolerance(ZERO_AMOUNT_THRESHOLD),
                    MarginfiError::OperationDepositOnly
                );
            }
            BalanceIncreaseType::Any | BalanceIncreaseType::BypassDepositLimit => {}
        }

        {
            let is_asset_amount_increasing =
                asset_amount_increase.is_positive_with_tolerance(ZERO_AMOUNT_THRESHOLD);
            bank.assert_operational_mode(Some(is_asset_amount_increasing))?;
        }

        let asset_shares_increase = bank.get_asset_shares(asset_amount_increase)?;
        balance.change_asset_shares(asset_shares_increase)?;
        bank.change_asset_shares(
            asset_shares_increase,
            matches!(operation_type, BalanceIncreaseType::BypassDepositLimit),
        )?;

        let liability_shares_decrease = bank.get_liability_shares(liability_amount_decrease)?;
        balance.change_liability_shares(-liability_shares_decrease)?;
        bank.change_liability_shares(-liability_shares_decrease, true)?;

        let has_assets =
            I80F48::from(balance.asset_shares).is_positive_with_tolerance(ZERO_AMOUNT_THRESHOLD);
        let has_liabs = I80F48::from(balance.liability_shares)
            .is_positive_with_tolerance(ZERO_AMOUNT_THRESHOLD);

        if !had_assets && has_assets {
            bank.increment_lending_position_count();
        }

        if had_assets && !has_assets {
            bank.decrement_lending_position_count();
        }

        if !had_liabs && has_liabs {
            bank.increment_borrowing_position_count();
        }

        if had_liabs && !has_liabs {
            bank.decrement_borrowing_position_count();
        }

        Ok(())
    }

    /// Claim any unclaimed emissions and add them to the outstanding emissions amount.
    pub fn claim_emissions(&mut self, current_timestamp: u64) -> MarginfiResult {
        if let Some(balance_amount) = match (
            self.balance.get_side(),
            self.bank.get_flag(EMISSIONS_FLAG_LENDING_ACTIVE),
            self.bank.get_flag(EMISSIONS_FLAG_BORROW_ACTIVE),
        ) {
            (Some(BalanceSide::Assets), true, _) => Some(
                self.bank
                    .get_asset_amount(self.balance.asset_shares.into())?,
            ),
            _ => None,
        } {
            let last_update = if self.balance.last_update < MIN_EMISSIONS_START_TIME {
                current_timestamp
            } else {
                self.balance.last_update
            };
            let period = I80F48::from_num(
                current_timestamp
                    .checked_sub(last_update)
                    .ok_or_else(math_error!())?,
            );
            let emissions_rate = I80F48::from_num(self.bank.emissions_rate);
            let emissions = calc_emissions(
                period,
                balance_amount,
                self.bank.mint_decimals as usize,
                emissions_rate,
            )?;

            let emissions_real = min(emissions, I80F48::from(self.bank.emissions_remaining));

            if emissions != emissions_real {
                msg!(
                    "Emissions capped: {} ({} calculated) for period {}s",
                    emissions_real,
                    emissions,
                    period
                );
            }

            debug!(
                "Outstanding emissions: {}",
                I80F48::from(self.balance.emissions_outstanding)
            );

            self.balance.emissions_outstanding = {
                I80F48::from(self.balance.emissions_outstanding)
                    .checked_add(emissions_real)
                    .ok_or_else(math_error!())?
            }
            .into();
            self.bank.emissions_remaining = {
                I80F48::from(self.bank.emissions_remaining)
                    .checked_sub(emissions_real)
                    .ok_or_else(math_error!())?
            }
            .into();
        }

        self.balance.last_update = current_timestamp;

        Ok(())
    }
}

fn calc_emissions(
    period: I80F48,
    balance_amount: I80F48,
    mint_decimals: usize,
    emissions_rate: I80F48,
) -> MarginfiResult<I80F48> {
    let exponent = EXP_10_I80F48[mint_decimals];
    let balance_amount_ui = balance_amount
        .checked_div(exponent)
        .ok_or_else(math_error!())?;

    let emissions = period
        .checked_mul(balance_amount_ui)
        .ok_or_else(math_error!())?
        .checked_div(SECONDS_PER_YEAR)
        .ok_or_else(math_error!())?
        .checked_mul(emissions_rate)
        .ok_or_else(math_error!())?;

    Ok(emissions)
}
