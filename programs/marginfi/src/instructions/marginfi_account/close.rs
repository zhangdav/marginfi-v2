use crate::{
    check, errors::MarginfiError, prelude::*, state::marginfi_account::MarginfiAccount,
    MarginfiResult,
};
use anchor_lang::prelude::*;

pub fn close_account(ctx: Context<MarginfiAccountClose>) -> MarginfiResult {
    let marginfi_account = &ctx.accounts.marginfi_account.load()?;

    check!(
        marginfi_account.can_be_closed(),
        MarginfiError::IllegalAction,
        "Account cannot be closed"
    );

    Ok(())
}

#[derive(Accounts)]
pub struct MarginfiAccountClose<'info> {
    #[account(
        mut,
        has_one = authority,
        close = fee_payer
    )]
    pub marginfi_account: AccountLoader<'info, MarginfiAccount>,

    pub authority: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,
}
