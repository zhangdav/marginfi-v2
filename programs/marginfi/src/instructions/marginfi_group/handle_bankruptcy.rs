use crate::{
    bank_signer, check,
    constants::{
        INSURANCE_VAULT_AUTHORITY_SEED, INSURANCE_VAULT_SEED, LIQUIDITY_VAULT_SEED,
        PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG, PROGRAM_VERSION, ZERO_AMOUNT_THRESHOLD,
    },
    debug,
    errors::MarginfiError,
    math_error,
    state::{
        health_cache::HealthCache,
        marginfi_account::{MarginfiAccount, RiskEngine},
        marginfi_group::{Bank, BankVaultType, MarginfiGroup},
    },
    utils, MarginfiResult,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use bytemuck::Zeroable;
use fixed::types::I80F48;
use std::cmp::{max, min};

/// Handle a bankrupt marginfi account.
/// 1. Verify account is bankrupt, and lending account belonging to account contains bad debt.
/// 2. Determine the amount of bad debt covered by the insurance fund and the amount socialized between depositors.
/// 3. Cover the bad debt of the bankrupt account.
/// 4. Transfer the insured amount from the insurance fund.
/// 5. Socialize the loss between lenders if any.
pub fn lending_pool_handle_bankruptcy<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, LendingPoolHandleBankruptcy<'info>>,
) -> MarginfiResult {
    let LendingPoolHandleBankruptcy {
        marginfi_account: marginfi_account_loader,
        insurance_vault,
        token_program,
        bank: bank_loader,
        group: marginfi_group_loader,
        ..
    } = ctx.accounts;
    let bank = bank_loader.load()?;
    let maybe_bank_mint =
        utils::maybe_take_bank_mint(&mut ctx.remaining_accounts, &bank, token_program.key)?;

    let clock = Clock::get()?;

    if !bank.get_flag(PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG) {
        check!(
            ctx.accounts.signer.key() == marginfi_group_loader.load()?.admin,
            MarginfiError::Unauthorized
        );
    }

    drop(bank);

    let mut marginfi_account = marginfi_account_loader.load_mut()?;

    let mut health_cache = HealthCache::zeroed();
    health_cache.timestamp = clock.unix_timestamp;
    health_cache.program_version = PROGRAM_VERSION;
    RiskEngine::new(&marginfi_account, ctx.remaining_accounts)?
        .check_account_bankrupt(&mut Some(&mut health_cache))?;
    health_cache.set_engine_ok(true);
    marginfi_account.health_cache = health_cache;
}

#[derive(Accounts)]
pub struct LendingPoolHandleBankruptcy<'info> {
    pub group: AccountLoader<'info, MarginfiGroup>,

    /// CHECK: The admin signer constraint is only validated (in handler) if bank
    /// PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG is not set
    pub signer: Signer<'info>,

    #[account(
        mut,
        has_one = group,
    )]
    pub bank: AccountLoader<'info, Bank>,

    #[account(
        mut,
        has_one = group,
    )]
    pub marginfi_account: AccountLoader<'info, MarginfiAccount>,

    /// CHECK: Seed constraint
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_bump
    )]
    pub liquidity_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            INSURANCE_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.insurance_vault_bump
    )]
    pub insurance_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Seed constraint
    #[account(
        seeds = [
            INSURANCE_VAULT_AUTHORITY_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.insurance_vault_authority_bump
    )]
    pub insurance_vault_authority: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}
