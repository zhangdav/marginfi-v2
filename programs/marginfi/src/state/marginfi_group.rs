use crate::borsh::{BorshDeserialize, BorshSerialize};
use crate::constants::{ASSET_TAG_DEFAULT, MAX_ORACLE_KEYS, TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE};
use crate::errors::MarginfiError;
use crate::math_error;
use crate::prelude::MarginfiResult;
use crate::state::emode::EmodeSettings;
use crate::state::price::OracleSetup;
use crate::{assert_struct_align, assert_struct_size};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use std::fmt::{Debug, Formatter};
use type_layout::TypeLayout;
use crate::state::marginfi_account::calc_value;

assert_struct_size!(MarginfiGroup, 1056);
#[account(zero_copy)]
#[derive(Default, Debug, PartialEq, Eq, TypeLayout)]
pub struct MarginfiGroup {
    // Protocol administrator address (super authority of the platform)
    // Allows updating configuration, clearing settings, upgrading permissions, etc.
    pub admin: Pubkey,
    // Indicates the current market status (such as whether to suspend lending or enable certain functions)
    pub group_flags: u64,
    pub fee_state_cache: FeeStateCache,
    // Number of banks/markets currently enabled (and possibly number of token pairs)
    pub banks: u16,
    // Together with banks: u16, it makes up 8-byte alignment, which is convenient for zero-copy and #[repr(C)]
    pub pad0: [u8; 6],
    // Administrators who specifically control eMode (efficient mode, such as relaxing collateral factors when borrowing similar assets)
    pub emode_admin: Pubkey,

    // TODO:
    pub _padding_0: [[u64; 2]; 24],
    pub _padding_1: [[u64; 2]; 32],
    pub _padding_4: u64,
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Zeroable, Pod, Debug, PartialEq, Eq,
)]
#[repr(C)]
pub struct FeeStateCache {
    // Meaning: The destination address (wallet) for collecting platform fees.
    // Type: `Pubkey` on Solana, fixed at 32 bytes.
    // Purpose: Whenever a user pays any platform fee on Marginfi (e.g., borrowing interest, liquidation penalties), the funds will be transferred to this wallet.
    pub global_fee_wallet: Pubkey,
    // Meaning: The fixed fee component, e.g.,
    // A flat fee of 0.01 USDC is charged for each borrow operation.
    //
    // Type: `WrappedI80F48`, which wraps a high-precision fixed-point number (`I80F48`)
    //
    // Purpose:
    // This fixed fee is charged on every operation,
    // ensuring a minimum cost even for very small transactions.
    pub program_fee_fixed: WrappedI80F48,
    // Meaning: A fee rate charged based on the operation amount,
    // e.g., a 0.05% borrowing fee.
    //
    // Type: `WrappedI80F48`, a fixed-point number wrapper.
    //
    // Purpose:
    // The dynamic fee is calculated as: operation_amount * program_fee_rate.
    pub program_fee_rate: WrappedI80F48,
    // The block timestamp (in seconds) of the last update of this set of fee data
    pub last_update: i64,
}

#[zero_copy]
#[repr(C, align(8))]
#[derive(Default, BorshDeserialize, BorshSerialize, TypeLayout)]
pub struct WrappedI80F48 {
    pub value: [u8; 16],
}

impl Debug for WrappedI80F48 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", I80F48::from_le_bytes(self.value))
    }
}

impl From<I80F48> for WrappedI80F48 {
    fn from(i: I80F48) -> Self {
        Self {
            value: i.to_le_bytes(),
        }
    }
}

impl From<WrappedI80F48> for I80F48 {
    fn from(w: WrappedI80F48) -> Self {
        Self::from_le_bytes(w.value)
    }
}

impl PartialEq for WrappedI80F48 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for WrappedI80F48 {}

#[derive(Clone, Debug)]
pub struct GroupBankConfig {
    pub program_fees: bool,
}

assert_struct_size!(Bank, 1856);
assert_struct_align!(Bank, 8);
#[account(zero_copy)]
#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, TypeLayout)]
pub struct Bank {
    pub mint: Pubkey,
    pub mint_decimals: u8,

    pub group: Pubkey,

    pub _pad0: [u8; 7],

    pub asset_share_value: WrappedI80F48,
    pub liability_share_value: WrappedI80F48,

    pub liquidity_vault: Pubkey,
    pub liquidity_vault_bump: u8,
    pub liquidity_vault_authority_bump: u8,

    // For deposit: a portion of the assets collected by the protocol from interest,
    // penalties or other sources as risk buffer funds
    pub insurance_vault: Pubkey,
    pub insurance_vault_bump: u8,
    pub insurance_vault_authority_bump: u8,

    pub _pad1: [u8; 4],

    // Insurance fee that has not yet been withdrawn
    pub collected_insurance_fees_outstanding: WrappedI80F48,

    pub fee_vault: Pubkey,
    pub fee_vault_bump: u8,
    pub fee_vault_authority_bump: u8,

    pub _pad2: [u8; 6],

    pub collected_group_fees_outstanding: WrappedI80F48,

    // The total number of shares currently lent/deposited by all users
    pub total_liability_shares: WrappedI80F48,
    pub total_asset_shares: WrappedI80F48,

    pub last_update: i64,

    pub config: BankConfig,

    pub flags: u64,
    pub emissions_rate: u64,
    pub emissions_remaining: WrappedI80F48,
    pub emissions_mint: Pubkey,

    pub collected_program_fees_outstanding: WrappedI80F48,

    pub emode: EmodeSettings,

    pub fees_destination_account: Pubkey,

    pub _padding_0: [u8; 8],
    pub _padding_1: [[u64; 2]; 30],
}

// Initialize a Bank instance
impl Bank {
    pub const LEN: usize = std::mem::size_of::<Bank>();

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        marginfi_group_pk: Pubkey,
        config: BankConfig,
        mint: Pubkey,
        mint_decimals: u8,
        liquidity_vault: Pubkey,
        insurance_vault: Pubkey,
        fee_vault: Pubkey,
        current_timestamp: i64,
        liquidity_vault_bump: u8,
        liquidity_vault_authority_bump: u8,
        insurance_vault_bump: u8,
        insurance_vault_authority_bump: u8,
        fee_vault_bump: u8,
        fee_vault_authority_bump: u8,
    ) -> Bank {
        Bank {
            mint,
            mint_decimals,
            group: marginfi_group_pk,
            asset_share_value: I80F48::ONE.into(),
            liability_share_value: I80F48::ONE.into(),
            liquidity_vault,
            liquidity_vault_bump,
            liquidity_vault_authority_bump,
            insurance_vault,
            insurance_vault_bump,
            insurance_vault_authority_bump,
            collected_insurance_fees_outstanding: I80F48::ZERO.into(),
            fee_vault,
            fee_vault_bump,
            fee_vault_authority_bump,
            collected_group_fees_outstanding: I80F48::ZERO.into(),
            total_liability_shares: I80F48::ZERO.into(),
            total_asset_shares: I80F48::ZERO.into(),
            last_update: current_timestamp,
            config,
            flags: 0,
            emissions_rate: 0,
            emissions_remaining: I80F48::ZERO.into(),
            emissions_mint: Pubkey::default(),
            collected_program_fees_outstanding: I80F48::ZERO.into(),
            emode: EmodeSettings::zeroed(),
            fees_destination_account: Pubkey::default(),
            ..Default::default()
        }
    }

    // Convert the user's liability shares to the actual loan amount (token quantity)
    pub fn get_liability_amount(&self, shares: I80F48) -> MarginfiResult<I80F48> {
        Ok(shares
            .checked_mul(self.liability_share_value.into())
            .ok_or_else(math_error!())?)
    }

    // Convert the user's asset shares into the current actual withdrawable amount
    pub fn get_asset_amount(&self, shares: I80F48) -> MarginfiResult<I80F48> {
        Ok(shares
            .checked_mul(self.asset_share_value.into())
            .ok_or_else(math_error!())?)
    }

    // Convert an amount (such as the new loan amount) into the current loan share (liability shares) that should be obtained
    pub fn get_liability_shares(&self, value: I80F48) -> MarginfiResult<I80F48> {
        Ok(value
            .checked_div(self.liability_share_value.into())
            .ok_or_else(math_error!())?)
    }

    // Convert a deposit amount into the current deposit share (asset shares) that should be obtained
    pub fn get_asset_shares(&self, value: I80F48) -> MarginfiResult<I80F48> {
        Ok(value
            .checked_div(self.asset_share_value.into())
            .ok_or_else(math_error!())?)
    }

    // updating the total_asset_shares of a Bank, check whether the deposit limit has been exceeded.
    pub fn change_asset_shares(
        &mut self,
        shares: I80F48,
        // Whether to skip the deposit limit check
        bypass_deposit_limit: bool,
    ) -> MarginfiResult {
        let total_asset_shares: I80F48 = self.total_asset_shares.into();
        self.total_asset_shares = total_asset_shares
            .checked_add(shares)
            .ok_or_else(math_error!())?
            .into();

        // If all of the above are met, check the deposit limit
        if shares.is_positive() && self.config.is_deposit_limit_active() && !bypass_deposit_limit {
            let total_deposits_amount = self.get_asset_amount(self.total_asset_shares.into())?;
            let deposit_limit = I80F48::from_num(self.config.deposit_limit);

            if total_deposits_amount >= deposit_limit {
                let deposits_num: f64 = total_deposits_amount.to_num();
                let limit_num: f64 = deposit_limit.to_num();
                msg!("deposits: {:?} deposit lim: {:?}", deposits_num, limit_num);
                return err!(MarginfiError::BankAssetCapacityExceeded);
            }
        }

        Ok(())
    }

    pub fn maybe_get_asset_weight_init_discount(
        &self,
        price: I80F48,
    ) -> MarginfiResult<Option<I80F48>> {
        if self.config.usd_init_limit_active() {
            let bank_total_assets_value = calc_value(
                self.get_asset_amount(self.total_asset_shares.into())?,
                price,
                self.mint_decimals,
                None,
            )?;

            let total_asset_value_init_limit =
            I80F48::from_num(self.config.total_asset_value_init_limit);

            #[cfg(target_os = "solana")]
            debug!(
                "Init limit active, limit: {}, total_assets: {}",
                total_asset_value_init_limit, bank_total_assets_value
            );

            if bank_total_assets_value > total_asset_value_init_limit {
                let discount = total_asset_value_init_limit
                    .checked_div(bank_total_assets_value)
                    .ok_or_else(math_error!())?;

                #[cfg(target_os = "solana")]
                debug!(
                    "Discounting assets by {:.2} because of total deposits {} over {} use cap",
                    discount, bank_total_assets_value, total_asset_value_init_limit
                );

                Ok(Some(discount))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
 }

assert_struct_size!(BankConfig, 544);
assert_struct_align!(BankConfig, 8);
#[repr(C)]
#[derive(
    Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, Zeroable, Pod, PartialEq, Eq, TypeLayout,
)]
pub struct BankConfig {
    // Initial mortgage rate
    pub asset_weight_init: WrappedI80F48,
    // Maintaining collateral ratio
    pub asset_weight_maint: WrappedI80F48,

    // Initial calculation weight of loan
    pub liability_weight_init: WrappedI80F48,
    // Borrowing to maintain weight
    pub liability_weight_maint: WrappedI80F48,

    // The current total deposit limit of the asset market
    pub deposit_limit: u64,

    pub interest_rate_config: InterestRateConfig,
    // Market status (e.g. open/closed/withdrawal only)
    pub operational_state: BankOperationalState,

    pub oracle_setup: OracleSetup,
    pub oracle_keys: [Pubkey; MAX_ORACLE_KEYS],

    pub _pad0: [u8; 6],

    // Total loan limit
    pub borrow_limit: u64,

    // Indicates whether the asset can be used across portfolios
    pub risk_tier: RiskTier,

    // Asset Type Tags
    pub asset_tag: u8,

    pub _pad1: [u8; 6],

    // Limit the maximum value of the asset used for collateral
    pub total_asset_value_init_limit: u64,

    pub oracle_max_age: u16,

    pub _padding0: [u8; 6],
    pub _padding1: [u8; 32],
}

// Used to provide a default initialization value
impl Default for BankConfig {
    fn default() -> Self {
        Self {
            asset_weight_init: I80F48::ZERO.into(),
            asset_weight_maint: I80F48::ZERO.into(),
            liability_weight_init: I80F48::ONE.into(),
            liability_weight_maint: I80F48::ONE.into(),
            deposit_limit: 0,
            borrow_limit: 0,
            interest_rate_config: Default::default(),
            operational_state: BankOperationalState::Paused,
            oracle_setup: OracleSetup::None,
            oracle_keys: [Pubkey::default(); MAX_ORACLE_KEYS],
            _pad0: [0; 6],
            risk_tier: RiskTier::Isolated,
            asset_tag: ASSET_TAG_DEFAULT,
            _pad1: [0; 6],
            total_asset_value_init_limit: TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE,
            oracle_max_age: 0,
            _padding0: [0; 6],
            _padding1: [0; 32],
        }
    }
}

impl BankConfig {
    pub fn usd_init_limit_active(&self) -> bool {
        self.total_asset_value_init_limit != TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE
    }

    #[inline]
    pub fn is_deposit_limit_active(&self) -> bool {
        self.deposit_limit != u64::MAX
    }
}

assert_struct_size!(InterestRateConfig, 240);
#[repr(C)]
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    AnchorDeserialize,
    AnchorSerialize,
    Zeroable,
    Pod,
    PartialEq,
    Eq,
    TypeLayout,
)]
pub struct InterestRateConfig {
    pub optimal_utilization_rate: WrappedI80F48,
    // APR, which represents the interest rate when utilization = optimal_utilization_rate
    pub plateau_interest_rate: WrappedI80F48,
    // Maximum interest rate when utilization = 100%
    pub max_interest_rate: WrappedI80F48,

    // Fixed APR share allocated to insurance fund
    pub insurance_fee_fixed_apr: WrappedI80F48,
    // Dynamic fee sharing related to interest rates
    pub insurance_ir_fee: WrappedI80F48,
    // A fixed agreement fee (e.g. 0.3% annual interest rate) is deducted directly from the borrower's interest
    pub protocol_fixed_fee_apr: WrappedI80F48,
    pub protocol_ir_fee: WrappedI80F48,
    // A one-time fee (not annualized) when a loan is initiated, similar to a startup fee
    pub protocol_origination_fee: WrappedI80F48,

    pub _padding0: [u8; 16],
    pub _padding1: [[u8; 32]; 3],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq, Eq)]
pub enum BankOperationalState {
    Paused,
    Operational,
    ReduceOnly,
}
unsafe impl Zeroable for BankOperationalState {}
unsafe impl Pod for BankOperationalState {}

#[repr(u8)]
#[derive(Copy, Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq, Eq, Default)]
pub enum RiskTier {
    #[default]
    Collateral = 0,
    Isolated = 1,
}
unsafe impl Zeroable for RiskTier {}
unsafe impl Pod for RiskTier {}
