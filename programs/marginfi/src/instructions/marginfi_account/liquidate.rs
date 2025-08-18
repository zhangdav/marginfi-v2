use crate::constants::{
    INSURANCE_VAULT_SEED, LIQUIDATION_INSURANCE_FEE, LIQUIDATION_LIQUIDATOR_FEE,
};
use crate::events::{AccountEventHeader, LendingAccountLiquidateEvent, LiquidationBalances};
use crate::state::marginfi_account::{
    calc_amount, calc_value, get_remaining_accounts_per_bank, RiskEngine,
};
use crate::state::marginfi_group::{Bank, BankVaultType, MarginfiGroup};
use crate::state::price::{OraclePriceFeedAdapter, OraclePriceType, PriceAdapter, PriceBias};
use crate::utils::{validate_asset_tags, validate_bank_asset_tags};
use crate::{
    bank_signer,
    constants::{LIQUIDITY_VAULT_AUTHORITY_SEED, LIQUIDITY_VAULT_SEED},
    state::marginfi_account::{BankAccountWrapper, MarginfiAccount},
};
use crate::{check, debug, prelude::*, utils};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::sysvar::Sysvar;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use fixed::types::I80F48;

#[derive(Accounts)]
pub struct LendingAccountLiquidate<'info> {
    pub group: AccountLoader<'info, MarginfiGroup>,

    #[account(
        mut,
        has_one = group
    )]
    pub asset_bank: AccountLoader<'info, Bank>,

    #[account(
        mut,
        has_one = group
    )]
    pub liab_bank: AccountLoader<'info, Bank>,

    #[account(
        mut,
        has_one = group,
        has_one = authority
    )]
    pub liquidator_marginfi_account: AccountLoader<'info, MarginfiAccount>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = group
    )]
    pub liquidatee_marginfi_account: AccountLoader<'info, MarginfiAccount>,

    /// CHECK: Seed constraint
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(),
            liab_bank.key().as_ref(),
        ],
        bump = liab_bank.load()?.liquidity_vault_authority_bump
    )]
    pub bank_liquidity_vault_authority: AccountInfo<'info>,

    /// CHECK: Seed constraint
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            liab_bank.key().as_ref(),
        ],
        bump = liab_bank.load()?.liquidity_vault_bump
    )]
    pub bank_liquidity_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Seed constraint
    #[account(
        mut,
        seeds = [
            INSURANCE_VAULT_SEED.as_bytes(),
            liab_bank.key().as_ref(),
        ],
        bump = liab_bank.load()?.insurance_vault_bump
    )]
    pub bank_insurance_vault: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}