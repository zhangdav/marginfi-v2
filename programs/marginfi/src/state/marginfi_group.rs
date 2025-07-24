use anchor_lang::prelude::*;
use crate::assert_struct_size;
use type_layout::TypeLayout;
use bytemuck::{Pod, Zeroable};
use crate::{borsh::{BorshDeserialize, BorshSerialize}};
use std::fmt::{Debug, Formatter};
use fixed::types::I80F48;
use crate::prelude::MarginfiResult;
use crate::state::emode::EmodeSettings;

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Zeroable, Pod, Debug, PartialEq, Eq,)]
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
assert_struct_size!(Bank, 8);
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
    
    pub _pad1: [u8; 4],

    pub collected_insurance_fess_outstanding: WrappedI80F48,

    pub fee_vault: Pubkey,
    pub fee_vault_bump: u8,
    pub fee_vault_authority_bump: u8,

    pub _pad2: [u8; 6],

    pub collected_group_fees_outstanding: WrappedI80F48,
    
    pub total_liability_shares: WrappedI80F48,
    pub total_asset_shares: WrappedI80F48,

    pub last_update: i64,

    pub config: BankConfig,

    pub flags: u64,
    pub emissions_rate: u64,
    pub emissions_reaming: WrappedI80F48,
    pub emissions_mint: Pubkey,

    pub collected_program_fees_outstanding: WrappedI80F48,

    pub emode: EmodeSettings,

    pub fees_festination_account: Pubkey,

    pub _padding_0: [u8; 8],
    pub _padding_1: [[u64; 2]; 30],
}
