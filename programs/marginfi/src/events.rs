use crate::state::marginfi_group::BankConfigOpt;
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct GroupEventHeader {
    pub signer: Option<Pubkey>,
    pub marginfi_group: Pubkey,
}

#[event]
pub struct LendingPoolBankAccrueInterestEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub delta: u64,
    pub fees_collected: f64,
    pub insurance_collected: f64,
}

#[event]
pub struct MarginfiGroupCreateEvent {
    pub header: GroupEventHeader,
}

#[event]
pub struct MarginfiGroupConfigureEvent {
    pub header: GroupEventHeader,
    pub admin: Pubkey,
    pub flags: u64,
}

#[event]
pub struct LendingPoolBankConfigureFrozenEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub deposit_limit: u64,
    pub borrow_limit: u64,
}

#[event]
pub struct LendingPoolBankConfigureEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub config: BankConfigOpt,
}

#[event]
pub struct LendingPoolBankConfigureOracleEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub oracle_setup: u8,
    pub oracle: Pubkey,
}
