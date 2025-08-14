use anchor_lang::prelude::*;

#[error_code]
pub enum MarginfiError {
    #[msg("Internal Marginfi logic error")] // 6000
    InternalLogicError,
    #[msg("Invalid bank index")] // 6001
    BankNotFound,
    #[msg("Lending account balance not found")] // 6002
    LendingAccountBalanceNotFound,
    #[msg("Bank deposit capacity exceeded")] // 6003
    BankAssetCapacityExceeded,
    #[msg("Invalid transfer")] // 6004
    InvalidTransfer,
    #[msg("Invalid bank account")] // 6008
    InvalidBankAccount,
    #[msg("Account is not bankrupt")] // 6013
    AccountNotBankrupt,
    #[msg("Account balance is not bad debt")] // 6014
    BalanceNotBadDebt,
    #[msg("Invalid group config")] // 6015
    InvalidConfig,
    #[msg("Bank paused")] // 6016
    BankPaused,
    #[msg("Bank is ReduceOnly mode")] // 6017
    BankReduceOnly,
    #[msg("Bank is missing")] // 6018
    BankAccountNotFound,
    #[msg("Operation is deposit-only")] // 6019
    OperationDepositOnly,
    #[msg("Operation is repay-only")] // 6022
    OperationRepayOnly,
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
    #[msg("Illegal action during flashloan")] // 6037
    AccountInFlashloan,
    #[msg("Unauthorized")] // 6042
    Unauthorized,
    #[msg("Token22 Banks require mint account as first remaining account")] // 6044
    T22MintRequired,
    #[msg("Invalid ATA for global fee account")] // 6045
    InvalidFeeAta,
    #[msg("Use add pool permissionless instead")] // 6046
    AddedStakedPoolManually,
    #[msg("Staked SOL accounts can only deposit staked assets and borrow SOL")] // 6047
    AssetTagMismatch,
    #[msg("Stake pool validation failed: check the stake pool, mint, or sol pool")] // 6048
    StakePoolValidationFailed,
    #[msg("Switchboard oracle: stale price")] // 6049
    SwitchboardStalePrice,
    #[msg("Pyth Push oracle: stale price")] // 6050
    PythPushStalePrice,
    #[msg("Oracle error: wrong number of accounts")] // 6051
    WrongNumberOfOracleAccounts,
    #[msg("Oracle error: wrong account keys")] // 6052
    WrongOracleAccountKeys,
    #[msg("Pyth Push oracle: wrong account owner")] // 6053
    PythPushWrongAccountOwner,
    #[msg("Staked Pyth Push oracle: wrong account owner")] // 6054
    StakedPythPushWrongAccountOwner,
    #[msg("Pyth Push oracle: mismatched feed id")] // 6055
    PythPushMismatchedFeedId,
    #[msg("Pyth Push oracle: insufficient verification level")] // 6056
    PythPushInsufficientVerificationLevel,
    #[msg("Pyth Push oracle: feed id must be 32 Bytes")] // 6057
    PythPushFeedIdMustBe32Bytes,
    #[msg("Pyth Push oracle: feed id contains non-hex characters")] // 6058
    PythPushFeedIdNonHexCharacter,
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
    #[msg("TWAP window size does not match expected duration")] // 6076
    PythPushInvalidWindowSize,
    #[msg("Invalid fees destination account")] // 6077
    InvalidFeesDestinationAccount,
    #[msg("Oracle max confidence exceeded: try again later")] // 6080
    OracleMaxConfidenceExceeded,
    #[msg("Banks cannot close when they have open positions or emissions outstanding")] // 6081
    BankCannotClose,
}

impl From<MarginfiError> for ProgramError {
    fn from(e: MarginfiError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<pyth_solana_receiver_sdk::error::GetPriceError> for MarginfiError {
    fn from(e: pyth_solana_receiver_sdk::error::GetPriceError) -> Self {
        match e {
            pyth_solana_receiver_sdk::error::GetPriceError::PriceTooOld => {
                MarginfiError::PythPushStalePrice
            }
            pyth_solana_receiver_sdk::error::GetPriceError::MismatchedFeedId => {
                MarginfiError::PythPushMismatchedFeedId
            }
            pyth_solana_receiver_sdk::error::GetPriceError::InsufficientVerificationLevel => {
                MarginfiError::PythPushInsufficientVerificationLevel
            }
            pyth_solana_receiver_sdk::error::GetPriceError::FeedIdMustBe32Bytes => {
                MarginfiError::PythPushFeedIdMustBe32Bytes
            }
            pyth_solana_receiver_sdk::error::GetPriceError::FeedIdNonHexCharacter => {
                MarginfiError::PythPushFeedIdNonHexCharacter
            }
            pyth_solana_receiver_sdk::error::GetPriceError::InvalidWindowSize => {
                MarginfiError::PythPushInvalidWindowSize
            }
        }
    }
}
impl From<u32> for MarginfiError {
    fn from(value: u32) -> Self {
        match value {
            6001 => MarginfiError::BankNotFound,
            _ => MarginfiError::InternalLogicError,
        }
    }
}
