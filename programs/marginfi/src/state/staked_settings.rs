use crate::{
    assert_struct_align, assert_struct_size, check,
    errors::MarginfiError,
    state::marginfi_group::{RiskTier, WrappedI80F48},
    MarginfiResult,
};
use anchor_lang::prelude::*;
use fixed::types::I80F48;

assert_struct_size!(StakedSettings, 256);
assert_struct_align!(StakedSettings, 8);
#[account(zero_copy)]
#[repr(C)]
pub struct StakedSettings {
    pub key: Pubkey,
    pub marginfi_group: Pubkey,
    pub oracle: Pubkey,

    pub asset_weight_init: WrappedI80F48,
    pub asset_weight_maint: WrappedI80F48,

    pub deposit_limit: u64,
    pub total_asset_value_init_limit: u64,

    pub oracle_max_age: u16,
    pub risk_tier: RiskTier,
    _pad0: [u8; 5],

    _reserved0: [u8; 8],
    _reserved1: [u8; 32],
    _reserved2: [u8; 64],
}

impl StakedSettings {
    pub const LEN: usize = std::mem::size_of::<StakedSettings>();

    pub fn validate(&self) -> MarginfiResult {
        let asset_init_w = I80F48::from(self.asset_weight_init);
        let asset_maint_w = I80F48::from(self.asset_weight_maint);

        check!(
            asset_init_w >= I80F48::ZERO && asset_init_w <= I80F48::ONE,
            MarginfiError::InvalidConfig
        );
        check!(asset_maint_w >= asset_init_w, MarginfiError::InvalidConfig);
        check!(
            asset_maint_w <= (I80F48::ONE + I80F48::ONE),
            MarginfiError::InvalidConfig
        );
        if self.risk_tier == RiskTier::Isolated {
            check!(asset_init_w == I80F48::ZERO, MarginfiError::InvalidConfig);
            check!(asset_maint_w == I80F48::ZERO, MarginfiError::InvalidConfig);
        }

        Ok(())
    }
}
