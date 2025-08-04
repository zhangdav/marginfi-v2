use super::price::OraclePriceFeedAdapter;
use crate::constants::EXP_10_I80F48;
use crate::math_error;
use crate::prelude::MarginfiResult;
use crate::state::marginfi_group::{Bank, WrappedI80F48};
use crate::state::price::OraclePriceType;
use crate::{assert_struct_align, assert_struct_size};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use type_layout::TypeLayout;
use crate::state::health_cache::HealthCache;

pub struct BankAccountWithPriceFeed<'a, 'info> {
    bank: AccountLoader<'info, Bank>,
    price_feed: Box<MarginfiResult<OraclePriceFeedAdapter>>,
    balance: &'a Balance,
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

/// TODO: MarginfiAccount impl

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
    pub emission_shares: WrappedI80F48,
    pub last_update: u64,
    pub _padding: [u64; 1],
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
    debug!(
        "weighted_asset_qt: {}, price: {}, expo: {}",
        weight_asset_amount, price, mint_decimals
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
