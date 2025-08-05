use crate::{assert_struct_align, assert_struct_size, state::marginfi_group::WrappedI80F48};
use anchor_lang::prelude::*;

assert_struct_size!(FeeState, 256);
assert_struct_align!(FeeState, 8);

#[account(zero_copy)]
#[repr(C)]
pub struct FeeState {
    pub key: Pubkey,
    // A super administrator with permission to change fee settings
    pub global_fee_admin: Pubkey,
    // Receiving wallet for all protocol fees:
    // SOL fees are transferred directly to this address
    // Non-SOL tokens are transferred to the associated token account at this address
    pub global_fee_wallet: Pubkey,
    pub placeholder0: u64,
    // Fixed fee charged when initializing Bank
    pub bank_init_flat_sol_fee: u32,
    pub bump_seed: u8,
    _padding0: [u8; 4],
    _padding1: [u8; 15],
    // All transactions within the Marginfi Group are subject to this fixed fee
    pub program_fee_fixed: WrappedI80F48,
    // All transactions are charged a fee based on this ratio
    pub program_fee_rate: WrappedI80F48,
    _reserved0: [u8; 32],
    _reserved1: [u8; 64],
}

impl FeeState {
    pub const LEN: usize = std::mem::size_of::<FeeState>();
}
