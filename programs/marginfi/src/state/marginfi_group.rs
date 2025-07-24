use crate::borsh::{BorshDeserialize, BorshSerialize};
use crate::constants::{ASSET_TAG_DEFAULT, MAX_ORACLE_KEYS, TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE};
use crate::prelude::MarginfiResult;
use crate::state::emode::EmodeSettings;
use crate::state::price::OracleSetup;
use crate::{assert_struct_align, assert_struct_size};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use std::fmt::{Debug, Formatter};
use type_layout::TypeLayout;

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

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Zeroable, Pod, Debug, PartialEq, Eq,
)]
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

    pub insurance_vault: Pubkey,
    pub insurance_vault_bump: u8,
    pub insurance_vault_authority_bump: u8,

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

assert_struct_size!(BankConfig, 544);
assert_struct_align!(BankConfig, 8);
#[repr(C)]
#[derive(
    Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, Zeroable, Pod, PartialEq, Eq, TypeLayout,
)]
pub struct BankConfig {
    // Initial mortgage rate
    pub asset_weight_init: WrappedI80F48,
    // Maintaining collateral ratio
    pub asset_weight_maint: WrappedI80F48,

    // Initial calculation weight of loan
    pub liability_weight_init: WrappedI80F48,
    // Borrowing to maintain weight
    pub liability_weight_maint: WrappedI80F48,

    // The current total deposit limit of the asset market
    pub deposit_limit: u64,

    pub interest_rate_config: InterestRateConfig,
    // Market status (e.g. open/closed/withdrawal only)
    pub operational_state: BankOperationalState,

    pub oracle_setup: OracleSetup,
    pub oracle_keys: [Pubkey; MAX_ORACLE_KEYS],

    pub _pad0: [u8; 6],

    // Total loan limit
    pub borrow_limit: u64,

    // Indicates whether the asset can be used across portfolios
    pub risk_tier: RiskTier,

    // Asset Type Tags
    pub asset_tag: u8,

    pub _pad1: [u8; 6],

    // Limit the maximum value of the asset used for collateral
    pub total_asset_value_init_limit: u64,

    pub oracle_max_age: u16,

    pub _padding0: [u8; 6],
    pub _padding1: [u8; 32],
}

// Used to provide a default initialization value
impl Default for BankConfig {
    fn default() -> Self {
        Self {
            asset_weight_init: I80F48::ZERO.into(),
            asset_weight_maint: I80F48::ZERO.into(),
            liability_weight_init: I80F48::ONE.into(),
            liability_weight_maint: I80F48::ONE.into(),
            deposit_limit: 0,
            borrow_limit: 0,
            interest_rate_config: Default::default(),
            operational_state: BankOperationalState::Paused,
            oracle_setup: OracleSetup::None,
            oracle_keys: [Pubkey::default(); MAX_ORACLE_KEYS],
            _pad0: [0; 6],
            risk_tier: RiskTier::Isolated,
            asset_tag: ASSET_TAG_DEFAULT,
            _pad1: [0; 6],
            total_asset_value_init_limit: TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE,
            oracle_max_age: 0,
            _padding0: [0; 6],
            _padding1: [0; 32],
        }
    }
}

assert_struct_size!(InterestRateConfig, 240);
#[repr(C)]
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    AnchorDeserialize,
    AnchorSerialize,
    Zeroable,
    Pod,
    PartialEq,
    Eq,
    TypeLayout,
)]
pub struct InterestRateConfig {
    pub optimal_utilization_rate: WrappedI80F48,
    // APR, which represents the interest rate when utilization = optimal_utilization_rate
    pub plateau_interest_rate: WrappedI80F48,
    // Maximum interest rate when utilization = 100%
    pub max_interest_rate: WrappedI80F48,

    // Fixed APR share allocated to insurance fund
    pub insurance_fee_fixed_apr: WrappedI80F48,
    // Dynamic fee sharing related to interest rates
    pub insurance_ir_fee: WrappedI80F48,
    // A fixed agreement fee (e.g. 0.3% annual interest rate) is deducted directly from the borrower's interest
    pub protocol_fixed_fee_apr: WrappedI80F48,
    pub protocol_ir_fee: WrappedI80F48,
    // A one-time fee (not annualized) when a loan is initiated, similar to a startup fee
    pub protocol_origination_fee: WrappedI80F48,

    pub _padding0: [u8; 16],
    pub _padding1: [[u8; 32]; 3],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq, Eq)]
pub enum BankOperationalState {
    Paused,
    Operational,
    ReduceOnly,
}
unsafe impl Zeroable for BankOperationalState {}
unsafe impl Pod for BankOperationalState {}

#[repr(u8)]
#[derive(Copy, Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq, Eq, Default)]
pub enum RiskTier {
    #[default]
    Collateral = 0,
    Isolated = 1,
}
unsafe impl Zeroable for RiskTier {}
unsafe impl Pod for RiskTier {}
