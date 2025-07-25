use anchor_lang::prelude::*;

#[error_code]
pub enum MarginfiError {
    #[msg("Bank deposit capacity exceeded")] // 6003
    BankAssetCapacityExceeded,
    #[msg("Invalid group config")] // 6015
    InvalidConfig,
    #[msg("Math error")] // 6062
    MathError,
    #[msg("The Emode config was invalid")] // 6075
    BadEmodeConfig,
}
