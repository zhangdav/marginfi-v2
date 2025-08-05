use anchor_lang::prelude::*;
use instructions::*;
use prelude::*;

use state::marginfi_group::WrappedI80F48;

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
}
