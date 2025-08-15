use crate::{
    instructions::marginfi_group::StakedSettingsEditConfig, state::marginfi_group::BankConfigOpt,
};
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct GroupEventHeader {
    pub signer: Option<Pubkey>,
    pub marginfi_group: Pubkey,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct AccountEventHeader {
    pub signer: Option<Pubkey>,
    pub marginfi_account: Pubkey,
    pub marginfi_account_authority: Pubkey,
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

#[event]
pub struct LendingPoolBankCollectFeesEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub group_fees_collected: f64,
    pub group_fees_outstanding: f64,
    pub insurance_fees_collected: f64,
    pub insurance_fees_outstanding: f64,
}

#[event]
pub struct LendingPoolBankCreateEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
}

#[event]
pub struct EditStakedSettingsEvent {
    pub group: Pubkey,
    pub settings: StakedSettingsEditConfig,
}

#[event]
pub struct LendingPoolBankHandleBankruptcyEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub bad_debt: f64,
    pub covered_amount: f64,
    pub socialized_amount: f64,
}

// marginfi account events

#[event]
pub struct MarginfiAccountCreateEvent {
    pub header: AccountEventHeader,
}

#[event]
pub struct LendingAccountDepositEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}
