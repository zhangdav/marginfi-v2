use crate::{assert_struct_align, assert_struct_size};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use type_layout::TypeLayout;

use super::marginfi_group::WrappedI80F48;

assert_struct_size!(BankCache, 160);
assert_struct_align!(BankCache, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Copy, Clone, Zeroable, Pod, PartialEq, Eq, TypeLayout, Debug,
)]
pub struct BankCache {
    pub base_rate: u32,
    pub lending_rate: u32,
    pub borrowing_rate: u32,
    pub interest_accumulated_for: u32,
    pub accumulated_since_last_update: WrappedI80F48,
    // Space reserved for future fields
    _reserved0: [u8; 128],
}

impl Default for BankCache {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl BankCache {
    pub fn update_interest_rates(&mut self, interest_rates: &ComputedInterestRates) {
        self.base_rate = apr_to_u32(interest_rates.base_rate_apr);
        self.lending_rate = apr_to_u32(interest_rates.lending_rate_apr);
        self.borrowing_rate = apr_to_u32(interest_rates.borrowing_rate_apr);
    }
}

/// Useful when converting an I80F48 apr into a BankCache u32 from 0-1000. Clamps to 1000% if
/// exceeding that amount. Invalid for negative inputs.
pub fn apr_to_u32(value: I80F48) -> u32 {
    let max_percent = I80F48::from_num(10.0); // 1000%
    let clamped = value.min(max_percent);
    let ratio = clamped / max_percent;
    (ratio * I80F48::from_num(u32::MAX)).to_num::<u32>()
}

#[derive(Debug, Clone)]
pub struct ComputedInterestRates {
    pub base_rate_apr: I80F48,
    pub lending_rate_apr: I80F48,
    pub borrowing_rate_apr: I80F48,
    pub group_fee_apr: I80F48,
    pub insurance_fee_apr: I80F48,
    pub protocol_fee_apr: I80F48,
}
