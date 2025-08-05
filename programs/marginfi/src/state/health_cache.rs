use crate::{
    assert_struct_align, assert_struct_size,
    state::marginfi_account::MAX_LENDING_ACCOUNT_BALANCES, state::marginfi_group::WrappedI80F48,
};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use type_layout::TypeLayout;

assert_struct_size!(HealthCache, 304);
assert_struct_align!(HealthCache, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Copy, Clone, Zeroable, Pod, PartialEq, Eq, TypeLayout, Debug,
)]
pub struct HealthCache {
    pub asset_value: WrappedI80F48,
    pub liability_value: WrappedI80F48,
    pub asset_value_maint: WrappedI80F48,
    pub liability_maint: WrappedI80F48,
    pub asset_value_equity: WrappedI80F48,
    pub liability_value_equity: WrappedI80F48,
    pub timestamp: i64,
    pub flags: u32,
    pub mrgn_err: u32,
    pub prices: [[u8; 8]; MAX_LENDING_ACCOUNT_BALANCES],
    pub internal_err: u32,
    pub err_index: u8,
    pub prgram_version: u8,
    pub pad0: [u8; 2],
    pub internal_liq_err: u32,
    pub internal_bankruptcy_err: u32,
    pub reserved0: [u8; 32],
    pub reserved1: [u8; 16],
}

// TODO: HealthCache impl