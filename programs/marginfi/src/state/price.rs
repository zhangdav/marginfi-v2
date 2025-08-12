use crate::check_eq;
use crate::constants::{NATIVE_STAKE_ID, PYTH_ID, SPL_SINGLE_POOL_ID, SWITCHBOARD_PULL_ID};
use crate::errors::MarginfiError;
use crate::prelude::MarginfiResult;
use crate::state::marginfi_group::BankConfig;
use pyth_solana_receiver_sdk::PYTH_PUSH_ORACLE_ID;
use crate::{check, live};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use enum_dispatch::enum_dispatch;
use pyth_solana_receiver_sdk::price_update::{FeedId, PriceUpdateV2};
use std::cell::Ref;
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

#[enum_dispatch(PriceAdapter)]
#[cfg_attr(feature = "client", derive(Clone))]
pub enum OraclePriceFeedAdapter {
    PythPushOracle(PythPushOraclePriceFeed),
    SwitchboardPull(SwitchboardPullPriceFeed),
}

impl OraclePriceFeedAdapter {
    // pub fn try_from_bank_config<'info>(
    //     bank_config: &BankConfig,
    //     ais: &'info [AccountInfo<'info>],
    //     clock: &Clock,
    // ) -> MarginfiResult<Self> {
    //     Self::try_from_bank_config_with_max_age(
    //         bank_config,
    //         ais,
    //         clock,
    //         bank_config.get_oracle_max_age(),
    //     )
    // }

    // TODO: try_from_bank_config_with_max_age

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
    // TODO: load_checked
    // TODO: load_unchecked
    // TODO: peek_feed_id
    // TODO: get_confidence_interval
    // TODO: get_ema_price
    // TODO: get_unweighted_price

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
}

// TODO: impl PriceAdapter for PythPushOraclePriceFeed

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct SwitchboardPullPriceFeed {
    pub feed: Box<LitePullFeedAccountData>,
}

impl SwitchboardPullPriceFeed {
    // TODO: load_checked

    fn check_ais(ai: &AccountInfo) -> MarginfiResult {
        let ai_data = ai.data.borrow();

        check!(
            ai.owner.eq(&SWITCHBOARD_PULL_ID),
            MarginfiError::SwitchboardWrongAccountOwner
        );

        let _feed = parse_swb_ignore_alignment(ai_data)?;

        Ok(())
    }

    // TODO: get_price
    // TODO: get_confidence_interval
}

// TODO: impl PriceAdapter for SwitchboardPullPriceFeed

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct LitePullFeedAccountData {
    pub result: CurrentResult,
    #[cfg(feature = "client")]
    pub feed_hash: [u8; 32],
    #[cfg(feature = "client")]
    pub last_update_timestamp: i64,
}

// TODO: impl From<&PullFeedAccountData> for LitePullFeedAccountData
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
