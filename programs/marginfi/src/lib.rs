use anchor_lang::prelude::*;

pub mod state;
pub mod macros;
pub mod prelude;
pub mod errors;

declare_id!("5tZcX5B6QBaYVykWFCB4HzEiodfY4hy4WDYGE43Wo3G9");

#[program]
pub mod marginfi {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
