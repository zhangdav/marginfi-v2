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
