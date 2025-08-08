use anchor_lang::prelude::*;

#[error_code]
pub enum MarginfiError {
    #[msg("Bank deposit capacity exceeded")] // 6003
    BankAssetCapacityExceeded,
    #[msg("Invalid transfer")] // 6004
    InvalidTransfer,
    #[msg("Invalid group config")] // 6015
    InvalidConfig,
    #[msg("Bank paused")] // 6016
    BankPaused,
    #[msg("Bank is ReduceOnly mode")] // 6017
    BankReduceOnly,
    #[msg("Invalid oracle setup")] // 6025
    InvalidOracleSetup,
    #[msg("Invalid bank utilization ratio")] // 6026
    IllegalUtilizationRatio,
    #[msg("Bank borrow cap exceeded")] // 6027
    BankLiabilityCapacityExceeded,
    #[msg("Emissions already setup")] // 6030
    EmissionsAlreadySetup,
    #[msg("Oracle is not set")] // 6031
    OracleNotSetup,
    #[msg("Update emissions error")] //6034
    EmissionsUpdateError,
    #[msg("Unauthorized")] // 6042
    Unauthorized,
    #[msg("Token22 Banks require mint account as first remaining account")] // 6044
    T22MintRequired,
    #[msg("Invalid ATA for global fee account")] // 6045
    InvalidFeeAta,
    #[msg("Stake pool validation failed: check the stake pool, mint, or sol pool")] // 6048
    StakePoolValidationFailed,
    #[msg("Oracle error: wrong number of accounts")] // 6051
    WrongNumberOfOracleAccounts,
    #[msg("Oracle error: wrong account keys")] // 6052
    WrongOracleAccountKeys,
    #[msg("Pyth Push oracle: wrong account owner")] // 6053
    PythPushWrongAccountOwner,
    #[msg("Pyth Push oracle: mismatched feed id")] // 6055
    PythPushMismatchedFeedId,
    #[msg("Switchboard oracle: wrong account owner")] // 6059
    SwitchboardWrongAccountOwner,
    #[msg("Pyth Push oracle: invalid account")] // 6060
    PythPushInvalidAccount,
    #[msg("Switchboard oracle: invalid account")] // 6061
    SwitchboardInvalidAccount,
    #[msg("Math error")] // 6062
    MathError,
    #[msg("Arena groups can only support two banks")] // 6073
    ArenaBankLimit,
    #[msg("Arena groups cannot return to non-arena status")] // 6074
    ArenaSettingCannotChange,
    #[msg("The Emode config was invalid")] // 6075
    BadEmodeConfig,
    #[msg("Invalid fees destination account")] // 6077
    InvalidFeesDestinationAccount,
    #[msg("Banks cannot close when they have open positions or emissions outstanding")] // 6081
    BankCannotClose,
}
