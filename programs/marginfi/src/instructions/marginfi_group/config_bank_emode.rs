use crate::errors::MarginfiError;
use crate::{
    state::emode::{EmodeEntry, MAX_EMODE_ENTRIES},
    state::marginfi_group::{Bank, MarginfiGroup},
    MarginfiResult,
};
use anchor_lang::prelude::*;

pub fn lending_pool_configure_bank_emode(
    ctx: Context<LendingPoolConfigureBankEmode>,
    emode_tag: u16,
    entries: [EmodeEntry; MAX_EMODE_ENTRIES],
) -> MarginfiResult {
    let mut bank = ctx.accounts.bank.load_mut()?;

    let mut sorted_entries = entries;
    sorted_entries.sort_by_key(|e| e.collateral_bank_emode_tag);

    for entry in sorted_entries.iter_mut() {
        entry.pad0 = [0; 5];
    }

    bank.emode.emode_tag = emode_tag;
    bank.emode.emode_config.entries = sorted_entries;
    bank.emode.timestamp = Clock::get()?.unix_timestamp;
    bank.emode.validate_entries()?;

    if bank.emode.emode_config.has_entries() {
        msg!("emode entries detected and activated");
        bank.emode.update_emode_enabled();
    } else {
        msg!("no emode entries detected");
        bank.emode.update_emode_enabled();
    }

    msg!(
        "emode tag set to {:?} entries set to: {:?}",
        emode_tag,
        sorted_entries
    );

    Ok(())
}

#[derive(Accounts)]
pub struct LendingPoolConfigureBankEmode<'info> {
    #[account(
        has_one = emode_admin @ MarginfiError::Unauthorized
    )]
    pub group: AccountLoader<'info, MarginfiGroup>,

    pub emode_admin: Signer<'info>,

    #[account(
        mut,
        has_one = group,
    )]
    pub bank: AccountLoader<'info, Bank>,
}
