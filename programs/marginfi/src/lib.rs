use anchor_lang::prelude::*;
use instructions::*;
use prelude::*;

use state::emode::{EmodeEntry, MAX_EMODE_ENTRIES};
use state::marginfi_group::WrappedI80F48;
use state::marginfi_group::{BankConfigCompact, BankConfigOpt, InterestRateConfigOpt};

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod macros;
pub mod prelude;
pub mod state;
pub mod utils;

declare_id!("5tZcX5B6QBaYVykWFCB4HzEiodfY4hy4WDYGE43Wo3G9");

#[program]
pub mod marginfi {
    use crate::state::marginfi_group::WrappedI80F48;

    use super::*;

    /// Any user can call
    pub fn marginfi_group_initialize(
        ctx: Context<MarginfiGroupInitialize>,
        is_arena_group: bool,
    ) -> MarginfiResult {
        marginfi_group::initialize_group(ctx, is_arena_group)
    }

    /// The creator of a group can call
    pub fn marginfi_group_configure(
        ctx: Context<MarginfiGroupConfigure>,
        new_admin: Pubkey,
        new_emode_admin: Pubkey,
        new_curve_admin: Pubkey,
        new_limit_admin: Pubkey,
        new_emissions_admin: Pubkey,
        is_arena_group: bool,
    ) -> MarginfiResult {
        marginfi_group::configure(
            ctx,
            new_admin,
            new_emode_admin,
            new_curve_admin,
            new_limit_admin,
            new_emissions_admin,
            is_arena_group,
        )
    }

    /// (Runs once per program) Configures the fee state account, where the global admin sets fees
    /// that are assessed to the protocol
    pub fn init_global_fee_state(
        ctx: Context<InitFeeState>,
        admin: Pubkey,
        fee_wallet: Pubkey,
        bank_init_flat_sol_fee: u32,
        program_fee_fixed: WrappedI80F48,
        program_fee_rate: WrappedI80F48,
    ) -> MarginfiResult {
        marginfi_group::initialize_fee_state(
            ctx,
            admin,
            fee_wallet,
            bank_init_flat_sol_fee,
            program_fee_fixed,
            program_fee_rate,
        )
    }

    /// (global fee admin only) Adjust fees, admin, or the destination wallet
    pub fn edit_global_fee_state(
        ctx: Context<EditFeeState>,
        admin: Pubkey,
        fee_wallet: Pubkey,
        bank_init_flat_sol_fee: u32,
        program_fee_fixed: WrappedI80F48,
        program_fee_rate: WrappedI80F48,
    ) -> MarginfiResult {
        marginfi_group::edit_fee_state(
            ctx,
            admin,
            fee_wallet,
            bank_init_flat_sol_fee,
            program_fee_fixed,
            program_fee_rate,
        )
    }

    /// (admin only)
    pub fn lending_pool_configure_bank(
        ctx: Context<LendingPoolConfigureBank>,
        bank_config_opt: BankConfigOpt,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_configure_bank(ctx, bank_config_opt)
    }

    /// (delegate_emissions_admin only)
    pub fn lending_pool_setup_emissions(
        ctx: Context<LendingPoolSetupEmissions>,
        flags: u64,
        rate: u64,
        total_emissions: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_setup_emissions(ctx, flags, rate, total_emissions)
    }

    /// (delegate_emissions_admin only)
    pub fn lending_pool_update_emissions_parameters(
        ctx: Context<LendingPoolUpdateEmissionsParameters>,
        emissions_flags: Option<u64>,
        emissions_rate: Option<u64>,
        additional_emissions: Option<u64>,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_update_emissions_parameters(
            ctx,
            emissions_flags,
            emissions_rate,
            additional_emissions,
        )
    }

    /// (delegate_curve_admin only)
    pub fn lending_pool_configure_bank_interest_only(
        ctx: Context<LendingPoolConfigureBankInterestOnly>,
        interest_rate_config: InterestRateConfigOpt,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_configure_bank_interest_only(ctx, interest_rate_config)
    }

    /// (delegate_limits_admin only)
    pub fn lending_pool_configure_bank_limits_only(
        ctx: Context<LendingPoolConfigureBankLimitsOnly>,
        deposit_limit: Option<u64>,
        borrow_limit: Option<u64>,
        total_asset_value_init_limit: Option<u64>,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_configure_bank_limits_only(
            ctx,
            deposit_limit,
            borrow_limit,
            total_asset_value_init_limit,
        )
    }

    // Operational instructions
    pub fn lending_pool_accrue_bank_interest(
        ctx: Context<LendingPoolAccrueBankInterest>,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_accrue_bank_interest(ctx)
    }

    /// (emode_admin only)
    pub fn lending_pool_configure_bank_emode(
        ctx: Context<LendingPoolConfigureBankEmode>,
        emode_tag: u16,
        entries: [EmodeEntry; MAX_EMODE_ENTRIES],
    ) -> MarginfiResult {
        marginfi_group::lending_pool_configure_bank_emode(ctx, emode_tag, entries)
    }

    /// (admin only)
    pub fn lending_pool_configure_bank_oracle(
        ctx: Context<LendingPoolConfigureBankOracle>,
        setup: u8,
        oracle: Pubkey,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_configure_bank_oracle(ctx, setup, oracle)
    }

    pub fn lending_pool_close_bank(ctx: Context<LendingPoolCloseBank>) -> MarginfiResult {
        marginfi_group::lending_pool_close_bank(ctx)
    }

    pub fn lending_pool_collect_bank_fees<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolCollectBankFees<'info>>,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_collect_bank_fees(ctx)
    }

    pub fn lending_pool_withdraw_fees<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolWithdrawFees<'info>>,
        amount: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_withdraw_fees(ctx, amount)
    }

    pub fn lending_pool_withdraw_insurance<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolWithdrawInsurance<'info>>,
        amount: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_withdraw_insurance(ctx, amount)
    }

    pub fn lending_pool_withdraw_fees_permissionless<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolWithdrawFeesPermissionless<'info>>,
        amount: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_withdraw_fees_permissionless(ctx, amount)
    }

    pub fn lending_pool_add_bank(
        ctx: Context<LendingPoolAddBank>,
        bank_config: BankConfigCompact,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_add_bank(ctx, bank_config)
    }

    pub fn lending_pool_add_bank_permissionless(
        ctx: Context<LendingPoolAddBankPermissionless>,
        bank_seed: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_add_bank_permissionless(ctx, bank_seed)
    }

    pub fn lending_pool_add_bank_with_seed(
        ctx: Context<LendingPoolAddBankWithSeed>,
        bank_config: BankConfigCompact,
        bank_seed: u64,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_add_bank_with_seed(ctx, bank_config, bank_seed)
    }

    pub fn propagate_fee_state(ctx: Context<PropagateFee>) -> MarginfiResult {
        marginfi_group::propagate_fee(ctx)
    }

    pub fn edit_staked_settings(
        ctx: Context<EditStakedSettings>,
        settings: StakedSettingsEditConfig,
    ) -> MarginfiResult {
        marginfi_group::edit_staked_settings(ctx, settings)
    }

    pub fn init_staked_settings(
        ctx: Context<InitStakedSettings>,
        settings: StakedSettingsConfig,
    ) -> MarginfiResult {
        marginfi_group::initialize_staked_settings(ctx, settings)
    }

    pub fn propagate_staked_settings(ctx: Context<PropagateStakedSettings>) -> MarginfiResult {
        marginfi_group::propagate_staked_settings(ctx)
    }

    pub fn migrate_pyth_push_oracle(ctx: Context<MigratePythPushOracle>) -> MarginfiResult {
        marginfi_group::migrate_pyth_push_oracle(ctx)
    }

    pub fn lending_pool_handle_bankruptcy<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolHandleBankruptcy<'info>>,
    ) -> MarginfiResult {
        marginfi_group::lending_pool_handle_bankruptcy(ctx)
    }

    /// Initialize a marginfi account for a given group.
    pub fn marginfi_account_initialize(ctx: Context<MarginfiAccountInitialize>) -> MarginfiResult {
        marginfi_account::initialize_account(ctx)
    }

    pub fn lending_account_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountDeposit<'info>>,
        amount: u64,
        deposit_up_to_limit: Option<bool>,
    ) -> MarginfiResult {
        marginfi_account::lending_account_deposit(ctx, amount, deposit_up_to_limit)
    }

    pub fn lending_account_borrow<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountBorrow<'info>>,
        amount: u64,
    ) -> MarginfiResult {
        marginfi_account::lending_account_borrow(ctx, amount)
    }

    pub fn lending_account_repay<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountRepay<'info>>,
        amount: u64,
        repay_all: Option<bool>,
    ) -> MarginfiResult {
        marginfi_account::lending_account_repay(ctx, amount, repay_all)
    }

    pub fn lending_account_start_flashloan(
        ctx: Context<LendingAccountStartFlashloan>,
        end_index: u64,
    ) -> MarginfiResult {
        marginfi_account::lending_account_start_flashloan(ctx, end_index)
    }

    pub fn lending_account_end_flashloan<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountEndFlashloan<'info>>,
    ) -> MarginfiResult {
        marginfi_account::lending_account_end_flashloan(ctx)
    }

    pub fn lending_account_withdraw<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountWithdraw<'info>>,
        amount: u64,
        withdraw_all: Option<bool>,
    ) -> MarginfiResult {
        marginfi_account::lending_account_withdraw(ctx, amount, withdraw_all)
    }
}
