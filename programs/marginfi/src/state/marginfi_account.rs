use crate::prelude::MarginfiResult;
use crate::state::marginfi_group::{Bank, WrappedI80F48};
use super::price::OraclePriceFeedAdapter;
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use type_layout::TypeLayout;
use crate::{assert_struct_align, assert_struct_size};
use crate::constants::EXP_10_I80F48;
use crate::math_error;

pub struct BankAccountWithPriceFeed<'a, 'info> {
    bank: AccountLoader<'info, Bank>,
    price_feed: Box<MarginfiResult<OraclePriceFeedAdapter>>,
    balance: &'a Balance,
}

assert_struct_size!(Balance, 104);
assert_struct_align!(Balance, 8);
#[repr(C)]
#[derive(AnchorDeserialize, AnchorSerialize, Copy, Clone, Zeroable, Pod, PartialEq, Eq, TypeLayout)]
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