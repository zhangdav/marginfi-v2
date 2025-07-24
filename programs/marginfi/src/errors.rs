use anchor_lang::prelude::*;

#[error_code]
pub enum MarginfiError {
    #[msg("Invalid group config")] // 6015
    InvalidConfig,
    #[msg("The Emode config was invalid")] // 6075
    BadEmodeConfig,
}