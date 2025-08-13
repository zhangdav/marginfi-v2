use crate::check_eq;
use crate::constants::{
    CONF_INTERVAL_MULTIPLE, EXP_10_I80F48, MAX_CONF_INTERVAL, MIN_PYTH_PUSH_VERIFICATION_LEVEL,
    NATIVE_STAKE_ID, PYTH_ID, SPL_SINGLE_POOL_ID, STD_DEV_MULTIPLE, SWITCHBOARD_PULL_ID, U32_MAX,
    U32_MAX_DIV_10,
};
use crate::errors::MarginfiError;
use crate::prelude::MarginfiResult;
use crate::state::marginfi_group::BankConfig;
use crate::{check, debug, live, math_error, msg, require_keys_eq};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{borsh1::try_from_slice_unchecked, stake::state::StakeStateV2};
use anchor_spl::token::Mint;
use bytemuck::{Pod, Zeroable};
use enum_dispatch::enum_dispatch;
use fixed::types::I80F48;
use pyth_solana_receiver_sdk::price_update::{self, FeedId, PriceUpdateV2};
use pyth_solana_receiver_sdk::PYTH_PUSH_ORACLE_ID;
use std::{cell::Ref, cmp::min};
use switchboard_on_demand::{
    CurrentResult, Discriminator, PullFeedAccountData, SPL_TOKEN_PROGRAM_ID,
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq, Eq)]
pub enum OracleSetup {
    None,
    PythLegacy,
    SwitchboardV2,
    PythPushOracle,
    SwitchboardPull,
    StakedWithPythPush,
}
unsafe impl Zeroable for OracleSetup {}
unsafe impl Pod for OracleSetup {}

impl OracleSetup {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::PythLegacy),
            2 => Some(Self::SwitchboardV2),
            3 => Some(Self::PythPushOracle),
            4 => Some(Self::SwitchboardPull),
            5 => Some(Self::StakedWithPythPush),
            _ => None,
        }
    }
}

// TODO: PriceBias

#[enum_dispatch]
pub trait PriceAdapter {
    fn get_price_of_type(
        &self,
        oracle_price_type: OraclePriceType,
        bias: Option<PriceBias>,
        oracle_max_confidence: u32,
    ) -> MarginfiResult<I80F48>;
}

#[enum_dispatch(PriceAdapter)]
#[cfg_attr(feature = "client", derive(Clone))]
pub enum OraclePriceFeedAdapter {
    PythPushOracle(PythPushOraclePriceFeed),
    SwitchboardPull(SwitchboardPullPriceFeed),
}

impl OraclePriceFeedAdapter {
    pub fn try_from_bank_config<'info>(
        bank_config: &BankConfig,
        ais: &'info [AccountInfo<'info>],
        clock: &Clock,
    ) -> MarginfiResult<Self> {
        Self::try_from_bank_config_with_max_age(
            bank_config,
            ais,
            clock,
            bank_config.get_oracle_max_age(),
        )
    }

    pub fn try_from_bank_config_with_max_age<'info>(
        bank_config: &BankConfig,
        ais: &'info [AccountInfo<'info>],
        clock: &Clock,
        max_age: u64,
    ) -> MarginfiResult<Self> {
        match bank_config.oracle_setup {
            OracleSetup::None => Err(MarginfiError::OracleNotSetup.into()),
            OracleSetup::PythLegacy => {
                panic!("pyth legacy is deprecated");
            }
            OracleSetup::SwitchboardV2 => {
                panic!("swb v2 is deprecated");
            }
            OracleSetup::PythPushOracle => {
                check!(ais.len() == 1, MarginfiError::WrongNumberOfOracleAccounts);

                let account_info = &ais[0];

                if live!() {
                    check_eq!(
                        *account_info.owner,
                        pyth_solana_receiver_sdk::id(),
                        MarginfiError::PythPushWrongAccountOwner
                    );
                } else {
                    // On localnet, allow the mock program ID -OR- the real one
                    let owner_ok = account_info.owner.eq(&PYTH_ID)
                        || account_info.owner.eq(&pyth_solana_receiver_sdk::id());
                    check!(owner_ok, MarginfiError::PythPushWrongAccountOwner);
                }

                // 0.1.4 or later
                if bank_config.is_pyth_push_migrated() {
                    require_keys_eq!(
                        *account_info.key,
                        bank_config.oracle_keys[0],
                        MarginfiError::WrongOracleAccountKeys
                    );

                    Ok(OraclePriceFeedAdapter::PythPushOracle(
                        PythPushOraclePriceFeed::load_checked(account_info, None, clock, max_age)?,
                    ))
                } else {
                    // 0.1.3 or earlier, migrate with `migrate_pyth_push_oracle`
                    // TODO remove in 0.1.5
                    let price_feed_id = bank_config.get_pyth_push_oracle_feed_id().unwrap();

                    PythPushOraclePriceFeed::check_ai_and_feed_id(account_info, price_feed_id)?;

                    Ok(OraclePriceFeedAdapter::PythPushOracle(
                        PythPushOraclePriceFeed::load_checked(
                            account_info,
                            Some(price_feed_id),
                            clock,
                            max_age,
                        )?,
                    ))
                }
            }
            OracleSetup::SwitchboardPull => {
                check!(ais.len() == 1, MarginfiError::WrongNumberOfOracleAccounts);
                if ais[0].key != &bank_config.oracle_keys[0] {
                    msg!(
                        "Expected oracle key: {:?}, got: {:?}",
                        bank_config.oracle_keys[0],
                        ais[0].key
                    );
                    return Err(error!(MarginfiError::WrongOracleAccountKeys));
                }

                Ok(OraclePriceFeedAdapter::SwitchboardPull(
                    SwitchboardPullPriceFeed::load_checked(&ais[0], clock.unix_timestamp, max_age)?,
                ))
            }
            OracleSetup::StakedWithPythPush => {
                check!(ais.len() == 3, MarginfiError::WrongNumberOfOracleAccounts);

                if ais[1].key != &bank_config.oracle_keys[1]
                    || ais[2].key != &bank_config.oracle_keys[2]
                {
                    msg!(
                        "Expected oracle keys: [1] {:?}, [2] {:?}, got: [1] {:?}, [2] {:?}",
                        bank_config.oracle_keys[1],
                        bank_config.oracle_keys[2],
                        ais[1].key,
                        ais[2].key
                    );
                    return Err(error!(MarginfiError::WrongOracleAccountKeys));
                }

                let lst_mint = Account::<'info, Mint>::try_from(&ais[1]).unwrap();
                let lst_supply = lst_mint.supply;
                let stake_state = try_from_slice_unchecked::<StakeStateV2>(&ais[2].data.borrow())?;
                let (_, stake) = match stake_state {
                    StakeStateV2::Stake(meta, stake, _) => (meta, stake),
                    _ => panic!("unsupported stake state"), // TODO emit more specific error
                };
                let sol_pool_balance = stake.delegation.stake;
                // Note: When the pool is fresh, it has 1 SOL in it (an initial and non-refundable
                // balance that will stay in the pool forever). We don't want to include that
                // balance when reading the quantity of SOL that has been staked from actual
                // depositors (i.e. the amount that can actually be redeemed again).
                let lamports_per_sol: u64 = 1_000_000_000;
                let sol_pool_adjusted_balance = sol_pool_balance
                    .checked_sub(lamports_per_sol)
                    .ok_or_else(math_error!())?;
                // Note: exchange rate is `sol_pool_balance / lst_supply`, but we will do the
                // division last to avoid precision loss. Division does not need to be
                // decimal-adjusted because both SOL and stake positions use 9 decimals

                let account_info = &ais[0];

                if live!() {
                    check_eq!(
                        account_info.owner,
                        &pyth_solana_receiver_sdk::id(),
                        MarginfiError::StakedPythPushWrongAccountOwner
                    );
                } else {
                    // On localnet, allow the mock program ID OR the real one (for regression tests against
                    // actual mainnet accounts).
                    // * Note: Typically price updates are owned by `pyth_solana_receiver_sdk` and the oracle
                    // feed account itself is owned by PYTH ID. On localnet, the mock program may own both for
                    // simplicity.
                    let owner_ok = account_info.owner.eq(&PYTH_ID)
                        || account_info.owner.eq(&pyth_solana_receiver_sdk::id());
                    check!(owner_ok, MarginfiError::StakedPythPushWrongAccountOwner);
                }

                let mut feed;
                // 0.1.4 or later
                if bank_config.is_pyth_push_migrated() {
                    require_keys_eq!(
                        *account_info.key,
                        bank_config.oracle_keys[0],
                        MarginfiError::WrongOracleAccountKeys
                    );

                    feed =
                        PythPushOraclePriceFeed::load_checked(account_info, None, clock, max_age)?;
                } else {
                    // 0.1.3 or earlier, migrate with `propagate_staked_settings`
                    // TODO remove in 0.1.5
                    let price_feed_id = bank_config.get_pyth_push_oracle_feed_id().unwrap();

                    PythPushOraclePriceFeed::check_ai_and_feed_id(account_info, price_feed_id)?;

                    feed = PythPushOraclePriceFeed::load_checked(
                        account_info,
                        Some(price_feed_id),
                        clock,
                        max_age,
                    )?;
                }

                let adjusted_price = (feed.price.price as i128)
                    .checked_mul(sol_pool_adjusted_balance as i128)
                    .ok_or_else(math_error!())?
                    .checked_div(lst_supply as i128)
                    .ok_or_else(math_error!())?;
                feed.price.price = adjusted_price.try_into().unwrap();

                let adjusted_ema_price = (feed.ema_price.price as i128)
                    .checked_mul(sol_pool_adjusted_balance as i128)
                    .ok_or_else(math_error!())?
                    .checked_div(lst_supply as i128)
                    .ok_or_else(math_error!())?;
                feed.ema_price.price = adjusted_ema_price.try_into().unwrap();

                let price = OraclePriceFeedAdapter::PythPushOracle(feed);
                Ok(price)
            }
        }
    }

    /// * lst_mint, stake_pool, sol_pool - required only if configuring
    ///   `OracleSetup::StakedWithPythPush` initially. (subsequent validations of staked banks can
    ///   omit these)
    pub fn validate_bank_config(
        bank_config: &BankConfig,
        oracle_ais: &[AccountInfo],
        lst_mint: Option<Pubkey>,
        stake_pool: Option<Pubkey>,
        sol_pool: Option<Pubkey>,
    ) -> MarginfiResult {
        match bank_config.oracle_setup {
            OracleSetup::None => Err(MarginfiError::OracleNotSetup.into()),
            OracleSetup::PythLegacy => {
                panic!("pyth legacy is deprecated");
            }
            OracleSetup::SwitchboardV2 => {
                panic!("swb v2 is deprecated");
            }
            OracleSetup::PythPushOracle => {
                check!(
                    oracle_ais.len() == 1,
                    MarginfiError::WrongNumberOfOracleAccounts
                );

                // 0.1.4 or later
                if bank_config.is_pyth_push_migrated() {
                    require_keys_eq!(
                        oracle_ais[0].key(),
                        bank_config.oracle_keys[0],
                        MarginfiError::WrongOracleAccountKeys
                    );
                    load_price_update_v2_checked(&oracle_ais[0])?;
                    Ok(())
                } else {
                    // 0.1.3 or earlier, run migrate_pyth_push_oracle to convert.
                    // TODO remove in 0.1.5
                    PythPushOraclePriceFeed::check_ai_and_feed_id(
                        &oracle_ais[0],
                        bank_config.get_pyth_push_oracle_feed_id().unwrap(),
                    )?;

                    Ok(())
                }
            }
            OracleSetup::SwitchboardPull => {
                check!(
                    oracle_ais.len() == 1,
                    MarginfiError::WrongNumberOfOracleAccounts
                );
                if oracle_ais[0].key != &bank_config.oracle_keys[0] {
                    msg!(
                        "Expected oracle key: {:?}, got: {:?}",
                        bank_config.oracle_keys[0],
                        oracle_ais[0].key
                    );
                    return Err(error!(MarginfiError::WrongOracleAccountKeys));
                }

                SwitchboardPullPriceFeed::check_ais(&oracle_ais[0])?;

                Ok(())
            }
            OracleSetup::StakedWithPythPush => {
                if lst_mint.is_some() && stake_pool.is_some() && sol_pool.is_some() {
                    check!(
                        oracle_ais.len() == 3,
                        MarginfiError::WrongNumberOfOracleAccounts
                    );

                    // 0.1.4 or later
                    if bank_config.is_pyth_push_migrated() {
                        require_keys_eq!(
                            oracle_ais[0].key(),
                            bank_config.oracle_keys[0],
                            MarginfiError::WrongOracleAccountKeys
                        );
                        load_price_update_v2_checked(&oracle_ais[0])?;
                    } else {
                        // TODO remove in 0.1.5
                        // 0.1.3 or earlier, run propagate_staked_settings to convert.
                        PythPushOraclePriceFeed::check_ai_and_feed_id(
                            &oracle_ais[0],
                            bank_config.get_pyth_push_oracle_feed_id().unwrap(),
                        )?;
                    }

                    let lst_mint = lst_mint.unwrap();
                    let stake_pool = stake_pool.unwrap();
                    let sol_pool = sol_pool.unwrap();

                    let program_id = &SPL_SINGLE_POOL_ID;
                    let stake_pool_bytes = &stake_pool.to_bytes();
                    // Validate the given stake_pool derives the same lst_mint, proving stake_pool is correct
                    let (exp_mint, _) =
                        Pubkey::find_program_address(&[b"mint", stake_pool_bytes], program_id);
                    check_eq!(exp_mint, lst_mint, MarginfiError::StakePoolValidationFailed);
                    // Validate the now-proven stake_pool derives the given sol_pool
                    let (exp_pool, _) =
                        Pubkey::find_program_address(&[b"stake", stake_pool_bytes], program_id);
                    check_eq!(exp_pool, sol_pool, MarginfiError::StakePoolValidationFailed);

                    // Sanity check the mint. Note: spl-single-pool uses a classic Token, never Token22
                    check!(
                        oracle_ais[1].owner == &SPL_TOKEN_PROGRAM_ID,
                        MarginfiError::StakePoolValidationFailed
                    );
                    check_eq!(
                        oracle_ais[1].key(),
                        lst_mint,
                        MarginfiError::StakePoolValidationFailed
                    );
                    // Sanity check the pool is a native stake pool. Note: the native staking program is
                    // written in vanilla Solana and has no Anchor discriminator.
                    check!(
                        oracle_ais[2].owner == &NATIVE_STAKE_ID,
                        MarginfiError::StakePoolValidationFailed
                    );
                    check_eq!(
                        oracle_ais[2].key(),
                        sol_pool,
                        MarginfiError::StakePoolValidationFailed
                    );

                    Ok(())
                } else {
                    // light validation (after initial setup, only the Pyth oracle needs to be validated)
                    check!(
                        oracle_ais.len() == 1,
                        MarginfiError::WrongNumberOfOracleAccounts
                    );

                    // 0.1.4 or later
                    if bank_config.is_pyth_push_migrated() {
                        require_keys_eq!(
                            oracle_ais[0].key(),
                            bank_config.oracle_keys[0],
                            MarginfiError::WrongOracleAccountKeys
                        );
                        load_price_update_v2_checked(&oracle_ais[0])?;
                    } else {
                        // TODO remove in 0.1.5
                        // 0.1.3 or earlier, run propagate_staked_settings to convert.
                        PythPushOraclePriceFeed::check_ai_and_feed_id(
                            &oracle_ais[0],
                            bank_config.get_pyth_push_oracle_feed_id().unwrap(),
                        )?;
                    }

                    Ok(())
                }
            }
        }
    }
}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct PythPushOraclePriceFeed {
    ema_price: Box<pyth_solana_receiver_sdk::price_update::Price>,
    price: Box<pyth_solana_receiver_sdk::price_update::Price>,
}

impl PythPushOraclePriceFeed {
    pub fn load_checked(
        ai: &AccountInfo,
        feed_id: Option<&FeedId>, // TODO remove in 0.1.5
        clock: &Clock,
        max_age: u64,
    ) -> MarginfiResult<Self> {
        let price_feed_account = load_price_update_v2_checked(ai)?;
        let feed_id = if let Some(id) = feed_id {
            id
        } else {
            &price_feed_account.price_message.feed_id
        };

        let price = price_feed_account
            .get_price_no_older_than_with_custom_verification_level(
                clock,
                max_age,
                feed_id,
                MIN_PYTH_PUSH_VERIFICATION_LEVEL,
            )
            .map_err(|e| {
                debug!("Pyth push oracle error: {:?}", e);
                let error: MarginfiError = e.into();
                error
            })?;

        let ema_price = {
            let price_update::PriceFeedMessage {
                exponent,
                publish_time,
                ema_price,
                ema_conf,
                ..
            } = price_feed_account.price_message;

            pyth_solana_receiver_sdk::price_update::Price {
                price: ema_price,
                conf: ema_conf,
                exponent,
                publish_time,
            }
        };

        Ok(Self {
            price: Box::new(price),
            ema_price: Box::new(ema_price),
        })
    }

    // TODO: load_unchecked
    // TODO: peek_feed_id

    fn get_confidence_interval(
        &self,
        use_ema: bool,
        oracle_max_confidence: u32,
    ) -> MarginfiResult<I80F48> {
        let price = if use_ema {
            &self.ema_price
        } else {
            &self.price
        };

        let conf_interval =
            pyth_price_components_to_i80f48(I80F48::from_num(price.conf), price.exponent)?
                .checked_mul(CONF_INTERVAL_MULTIPLE)
                .ok_or_else(math_error!())?;

        let price = pyth_price_components_to_i80f48(I80F48::from_num(price.price), price.exponent)?;

        // Fail the price fetch if confidence > price * oracle_max_confidence
        let oracle_max_confidence = if oracle_max_confidence > 0 {
            I80F48::from_num(oracle_max_confidence)
        } else {
            // The default max confidence is 10%
            U32_MAX_DIV_10
        };
        let max_conf = price
            .checked_mul(oracle_max_confidence)
            .ok_or_else(math_error!())?
            .checked_div(U32_MAX)
            .ok_or_else(math_error!())?;
        if conf_interval > max_conf {
            let conf_interval = conf_interval.to_num::<f64>();
            let max_conf = max_conf.to_num::<f64>();
            msg!("conf was {:?}, but max is {:?}", conf_interval, max_conf);
            return err!(MarginfiError::OracleMaxConfidenceExceeded);
        }

        // Cap confidence interval to 5% of price regardless
        let capped_conf_interval = price
            .checked_mul(MAX_CONF_INTERVAL)
            .ok_or_else(math_error!())?;

        assert!(
            capped_conf_interval >= I80F48::ZERO,
            "Negative max confidence interval"
        );

        assert!(
            conf_interval >= I80F48::ZERO,
            "Negative confidence interval"
        );

        Ok(min(conf_interval, capped_conf_interval))
    }

    /// Find PDA address of a pyth push oracle given a shard_id and feed_id
    ///
    /// Pyth sponsored feed id
    /// `constants::PYTH_PUSH_PYTH_SPONSORED_SHARD_ID = 0`
    ///
    /// Marginfi sponsored feed id
    /// `constants::PYTH_PUSH_MARGINFI_SPONSORED_SHARD_ID = 3301`
    pub fn find_oracle_address(shard_id: u16, feed_id: &FeedId) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&shard_id.to_le_bytes(), feed_id], &PYTH_PUSH_ORACLE_ID)
    }

    pub fn check_ai_and_feed_id(ai: &AccountInfo, feed_id: &FeedId) -> MarginfiResult {
        let price_feed_account = load_price_update_v2_checked(ai)?;

        check_eq!(
            &price_feed_account.price_message.feed_id,
            feed_id,
            MarginfiError::PythPushMismatchedFeedId
        );

        Ok(())
    }

    #[inline(always)]
    fn get_ema_price(&self) -> MarginfiResult<I80F48> {
        pyth_price_components_to_i80f48(
            I80F48::from_num(self.ema_price.price),
            self.ema_price.exponent,
        )
    }

    #[inline(always)]
    fn get_unweighted_price(&self) -> MarginfiResult<I80F48> {
        pyth_price_components_to_i80f48(I80F48::from_num(self.price.price), self.price.exponent)
    }
}

impl PriceAdapter for PythPushOraclePriceFeed {
    fn get_price_of_type(
        &self,
        price_type: OraclePriceType,
        bias: Option<PriceBias>,
        oracle_max_confidence: u32,
    ) -> MarginfiResult<I80F48> {
        let price = match price_type {
            OraclePriceType::TimeWeighted => self.get_ema_price()?,
            OraclePriceType::RealTime => self.get_unweighted_price()?,
        };

        match bias {
            None => Ok(price),
            Some(price_bias) => {
                let confidence_interval = self.get_confidence_interval(
                    matches!(price_type, OraclePriceType::TimeWeighted),
                    oracle_max_confidence,
                )?;

                match price_bias {
                    PriceBias::Low => Ok(price
                        .checked_sub(confidence_interval)
                        .ok_or_else(math_error!())?),
                    PriceBias::High => Ok(price
                        .checked_add(confidence_interval)
                        .ok_or_else(math_error!())?),
                }
            }
        }
    }
}

impl PriceAdapter for SwitchboardPullPriceFeed {
    fn get_price_of_type(
        &self,
        _price_type: OraclePriceType,
        bias: Option<PriceBias>,
        oracle_max_confidence: u32,
    ) -> MarginfiResult<I80F48> {
        let price = self.get_price()?;

        match bias {
            Some(price_bias) => {
                let confidence_interval = self.get_confidence_interval(oracle_max_confidence)?;

                match price_bias {
                    PriceBias::Low => Ok(price
                        .checked_sub(confidence_interval)
                        .ok_or_else(math_error!())?),
                    PriceBias::High => Ok(price
                        .checked_add(confidence_interval)
                        .ok_or_else(math_error!())?),
                }
            }
            None => Ok(price),
        }
    }
}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct SwitchboardPullPriceFeed {
    pub feed: Box<LitePullFeedAccountData>,
}

impl SwitchboardPullPriceFeed {
    pub fn load_checked(
        ai: &AccountInfo,
        current_timestamp: i64,
        max_age: u64,
    ) -> MarginfiResult<Self> {
        let ai_data = ai.data.borrow();

        check!(
            ai.owner.eq(&SWITCHBOARD_PULL_ID),
            MarginfiError::SwitchboardWrongAccountOwner
        );

        let feed: PullFeedAccountData = parse_swb_ignore_alignment(ai_data)?;
        let lite_feed = LitePullFeedAccountData::from(&feed);
        // TODO restore when swb fixes alignment issue in crate.
        // let feed = PullFeedAccountData::parse(ai_data)
        //     .map_err(|_| MarginfiError::SwitchboardInvalidAccount)?;

        // Check staleness
        let last_updated = feed.last_update_timestamp;
        if current_timestamp.saturating_sub(last_updated) > max_age as i64 {
            return err!(MarginfiError::SwitchboardStalePrice);
        }

        Ok(Self {
            feed: Box::new(lite_feed),
        })
    }

    fn check_ais(ai: &AccountInfo) -> MarginfiResult {
        let ai_data = ai.data.borrow();

        check!(
            ai.owner.eq(&SWITCHBOARD_PULL_ID),
            MarginfiError::SwitchboardWrongAccountOwner
        );

        let _feed = parse_swb_ignore_alignment(ai_data)?;

        Ok(())
    }

    fn get_price(&self) -> MarginfiResult<I80F48> {
        let sw_result = self.feed.result;

        let price = I80F48::from_num(sw_result.value)
            .checked_div(EXP_10_I80F48[switchboard_on_demand::PRECISION as usize])
            .ok_or_else(math_error!())?;

        Ok(price)
    }

    fn get_confidence_interval(&self, oracle_max_confidence: u32) -> MarginfiResult<I80F48> {}
}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct LitePullFeedAccountData {
    pub result: CurrentResult,
    #[cfg(feature = "client")]
    pub feed_hash: [u8; 32],
    #[cfg(feature = "client")]
    pub last_update_timestamp: i64,
}

impl From<&PullFeedAccountData> for LitePullFeedAccountData {
    fn from(feed: &PullFeedAccountData) -> Self {
        Self {
            result: feed.result,
            #[cfg(feature = "client")]
            feed_hash: feed.feed_hash,
            #[cfg(feature = "client")]
            last_update_timestamp: feed.last_update_timestamp,
        }
    }
}
// TODO: impl From<Ref<'_, PullFeedAccountData>> for LitePullFeedAccountData

#[derive(Copy, Clone, Debug)]
pub enum OraclePriceType {
    /// Time weighted price
    /// EMA for PythEma
    TimeWeighted,
    /// Real time price
    RealTime,
}

pub fn check_ai_and_feed_id(ai: &AccountInfo, feed_id: &FeedId) -> MarginfiResult {
    let price_feed_account = load_price_update_v2_checked(ai)?;

    check!(
        &price_feed_account.price_message.feed_id.eq(feed_id),
        MarginfiError::PythPushMismatchedFeedId
    );

    Ok(())
}

#[inline(always)]
fn pyth_price_components_to_i80f48(price: I80F48, exponent: i32) -> MarginfiResult<I80F48> {
    let scaling_factor = EXP_10_I80F48[exponent.unsigned_abs() as usize];

    let price = if exponent == 0 {
        price
    } else if exponent < 0 {
        price
            .checked_div(scaling_factor)
            .ok_or_else(math_error!())?
    } else {
        price
            .checked_mul(scaling_factor)
            .ok_or_else(math_error!())?
    };

    Ok(price)
}

// TODO remove when swb fixes the alignment issue in their crate
// (TargetAlignmentGreaterAndInputNotAligned) when bytemuck::from_bytes executes on any local system
// (including bpf next-test) where the struct is "properly" aligned 16
/// The same as PullFeedAccountData::parse but completely ignores input alignment.
pub fn parse_swb_ignore_alignment(data: Ref<&mut [u8]>) -> MarginfiResult<PullFeedAccountData> {
    if data.len() < 8 {
        return err!(MarginfiError::SwitchboardInvalidAccount);
    }

    if data[..8] != PullFeedAccountData::DISCRIMINATOR {
        return err!(MarginfiError::SwitchboardInvalidAccount);
    }

    let feed = bytemuck::try_pod_read_unaligned::<PullFeedAccountData>(
        &data[8..8 + std::mem::size_of::<PullFeedAccountData>()],
    )
    .map_err(|_| MarginfiError::SwitchboardInvalidAccount)?;

    Ok(feed)
}

/// Prevent non-oracle accounts from falsifying price data.
/// Ensure the account format is correct (matching the correct type).
/// Extract price data for risk control (e.g., valuation, liquidation, etc.).
pub fn load_price_update_v2_checked(ai: &AccountInfo) -> MarginfiResult<PriceUpdateV2> {
    if live!() {
        check_eq!(
            *ai.owner,
            pyth_solana_receiver_sdk::id(),
            MarginfiError::PythPushWrongAccountOwner
        );
    } else {
        // On localnet, allow the mock program ID OR the real one (for regression tests against
        // actual mainnet accounts).
        // * Note: Typically price updates are owned by `pyth_solana_receiver_sdk` and the oracle
        // feed account itself is owned by PYTH ID. On localnet, the mock program may own both for
        // simplicity.
        let owner_ok = ai.owner.eq(&PYTH_ID) || ai.owner.eq(&pyth_solana_receiver_sdk::id());
        check!(owner_ok, MarginfiError::PythPushWrongAccountOwner);
    }

    let price_feed_data = ai.try_borrow_data()?;
    let discriminator = &price_feed_data[0..8];
    let expected_discrim = <PriceUpdateV2 as anchor_lang::Discriminator>::DISCRIMINATOR;

    check_eq!(
        discriminator,
        expected_discrim,
        MarginfiError::PythPushInvalidAccount
    );

    Ok(PriceUpdateV2::deserialize(
        &mut &price_feed_data.as_ref()[8..],
    )?)
}

// TODO: pyth_price_components_to_i80f48

#[derive(Copy, Clone, Debug)]
pub enum PriceBias {
    Low,
    High,
}
