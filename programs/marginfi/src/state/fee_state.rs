use anchor_lang::prelude::*;
use crate::{
    assert_struct_align, assert_struct_size, state::marginfi_group::WrappedI80F48,
};

assert_struct_size!(FeeState, 256);
assert_struct_align!(FeeState, 8);

#[account(zero_copy)]
#[repr(C)]
pub struct FeeState {
    pub key: Pubkey,
    pub global_fee_admin: Pubkey,
    pub global_fee_wallet: Pubkey,
    pub placeholder0: u64,
    pub bank_init_flat_sol_fee: u32,
    pub bump_seed: u8,
    _padding0: [u8; 4],
    _padding1: [u8; 15],
    pub program_fee_fixed: WrappedI80F48,
    pub program_fee_rate: WrappedI80F48,
    _reserved0: [u8; 32],
    _reserved1: [u8; 64],
}

impl FeeState {
    pub const LEN: usize = std::mem::size_of::<FeeState>();
}