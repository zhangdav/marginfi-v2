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
    #[msg("Missing Oracle, Bank, LST mint, or Sol Pool")] // 6005
    MissingPythOrBankAccount,
    #[msg("Missing Pyth account")] // 6006
    MissingPythAccount,
    #[msg("Missing Bank account")] // 6007
    MissingBankAccount,
    #[msg("Invalid bank account")] // 6008
    InvalidBankAccount,
    #[msg("RiskEngine rejected due to either bad health or stale oracles")] // 6009
    RiskEngineInitRejected,
    #[msg("Lending account balance slots are full")] // 6010
    LendingAccountBalanceSlotsFull,
    #[msg("Amount to liquidate must be positive")] // 6012
    ZeroLiquidationAmount,
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
    #[msg("Operation is withdraw-only")] // 6020
    OperationWithdrawOnly,
    #[msg("Operation is borrow-only")] // 6021
    OperationBorrowOnly,
    #[msg("Operation is repay-only")] // 6022
    OperationRepayOnly,
    #[msg("No asset found")] // 6023
    NoAssetFound,
    #[msg("No liability found")] // 6024
    NoLiabilityFound,
    #[msg("Invalid oracle setup")] // 6025
    InvalidOracleSetup,
    #[msg("Invalid bank utilization ratio")] // 6026
    IllegalUtilizationRatio,
    #[msg("Bank borrow cap exceeded")] // 6027
    BankLiabilityCapacityExceeded,
    #[msg("Account can have only one liability when account is under isolated risk")] // 6029
    IsolatedAccountIllegalState,
    #[msg("Emissions already setup")] // 6030
    EmissionsAlreadySetup,
    #[msg("Oracle is not set")] // 6031
    OracleNotSetup,
    #[msg("Cannot close balance because of outstanding emissions")] // 6033
    CannotCloseOutstandingEmissions,
    #[msg("Update emissions error")] //6034
    EmissionsUpdateError,
    #[msg("Account disabled")] // 6035
    AccountDisabled,
    #[msg("Illegal action during flashloan")] // 6037
    AccountInFlashloan,
    #[msg("Illegal flashloan")] // 6038
    IllegalFlashloan,
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
    #[msg("Invalid emissions destination account")] // 6063
    InvalidEmissionsDestinationAccount,
    #[msg("Asset and liability bank cannot be the same")] // 6064
    SameAssetAndLiabilityBanks,
    #[msg("Trying to withdraw more assets than available")] // 6065
    OverliquidationAttempt,
    #[msg("Liability bank has no liabilities")] // 6066
    NoLiabilitiesInLiabilityBank,
    #[msg("Liability bank has assets")] // 6067
    AssetsInLiabilityBank,
    #[msg("Account is healthy and cannot be liquidated")] // 6068
    HealthyAccount,
    #[msg("Liability payoff too severe, exhausted liability")] // 6069
    ExhaustedLiability,
    #[msg("Liability payoff too severe, liability balance has assets")] // 6070
    TooSeverePayoff,
    #[msg("Liquidation too severe, account above maintenance requirement")] // 6071
    TooSevereLiquidation,
    #[msg("Liquidation would worsen account health")] // 6072
    WorseHealthPostLiquidation,
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
    #[msg("Zero asset price")] // 6078
    ZeroAssetPrice,
    #[msg("Zero liability price")] // 6079
    ZeroLiabilityPrice,
    #[msg("Oracle max confidence exceeded: try again later")] // 6080
    OracleMaxConfidenceExceeded,
    #[msg("Banks cannot close when they have open positions or emissions outstanding")] // 6081
    BankCannotClose,
    #[msg("Account already migrated")] // 6082
    AccountAlreadyMigrated,
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

impl MarginfiError {
    pub fn is_oracle_error(&self) -> bool {
        matches!(
            self,
            MarginfiError::WrongNumberOfOracleAccounts
                | MarginfiError::SwitchboardInvalidAccount
                | MarginfiError::PythPushInvalidAccount
                | MarginfiError::SwitchboardWrongAccountOwner
                | MarginfiError::PythPushFeedIdNonHexCharacter
                | MarginfiError::PythPushFeedIdMustBe32Bytes
                | MarginfiError::PythPushInsufficientVerificationLevel
                | MarginfiError::PythPushMismatchedFeedId
                | MarginfiError::StakedPythPushWrongAccountOwner
                | MarginfiError::PythPushWrongAccountOwner
                | MarginfiError::WrongOracleAccountKeys
                | MarginfiError::PythPushStalePrice
                | MarginfiError::SwitchboardStalePrice
                | MarginfiError::StakePoolValidationFailed
                | MarginfiError::InvalidBankAccount
                | MarginfiError::MissingBankAccount
                | MarginfiError::MissingPythAccount
                | MarginfiError::MissingPythOrBankAccount
                | MarginfiError::PythPushInvalidWindowSize
                | MarginfiError::OracleMaxConfidenceExceeded
        )
    }
    pub fn is_risk_engine_rejection(&self) -> bool {
        matches!(self, MarginfiError::RiskEngineInitRejected)
    }
}
