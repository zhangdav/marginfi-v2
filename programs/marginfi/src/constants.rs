use anchor_lang::prelude::*;
use fixed::types::I80F48;
use fixed_macro::types::I80F48;

pub const MAX_ORACLE_KEYS: usize = 5;
pub const ASSET_TAG_DEFAULT: u8 = 0;
pub const TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE: u64 = 0;

// Anyone can try to settle bad debts in an account without permission or administrator status
pub const PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG: u64 = 1 << 2;

pub const PYTH_PUSH_MIGRATED: u8 = 1 << 0;

// Some of the Bank's configurations are frozen and cannot be changed.
pub const FREEZE_SETTINGS: u64 = 1 << 3;

pub const EMISSION_FLAG_BORROW_ACTIVE: u64 = 1 << 0;
pub const EMISSION_FLAG_LENDING_ACTIVE: u64 = 1 << 1;
pub(crate) const EMISSION_FLAGS: u64 = EMISSION_FLAG_BORROW_ACTIVE | EMISSION_FLAG_LENDING_ACTIVE;
pub(crate) const GROUP_FLAGS: u64 = PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG | FREEZE_SETTINGS;

pub const SECONDS_PER_YEAR: I80F48 = I80F48!(31_536_000);

pub const MAX_EXP_10_I80F48: usize = 24;
pub const EXP_10_I80F48: [I80F48; MAX_EXP_10_I80F48] = [
    I80F48!(1),                        // 10^0
    I80F48!(10),                       // 10^1
    I80F48!(100),                      // 10^2
    I80F48!(1000),                     // 10^3
    I80F48!(10000),                    // 10^4
    I80F48!(100000),                   // 10^5
    I80F48!(1000000),                  // 10^6
    I80F48!(10000000),                 // 10^7
    I80F48!(100000000),                // 10^8
    I80F48!(1000000000),               // 10^9
    I80F48!(10000000000),              // 10^10
    I80F48!(100000000000),             // 10^11
    I80F48!(1000000000000),            // 10^12
    I80F48!(10000000000000),           // 10^13
    I80F48!(100000000000000),          // 10^14
    I80F48!(1000000000000000),         // 10^15
    I80F48!(10000000000000000),        // 10^16
    I80F48!(100000000000000000),       // 10^17
    I80F48!(1000000000000000000),      // 10^18
    I80F48!(10000000000000000000),     // 10^19
    I80F48!(100000000000000000000),    // 10^20
    I80F48!(1000000000000000000000),   // 10^21
    I80F48!(10000000000000000000000),  // 10^22
    I80F48!(100000000000000000000000), // 10^23
];

cfg_if::cfg_if! {
    if #[cfg(feature = "devnet")] {
        pub const PYTH_ID: Pubkey = pubkey!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
    } else if #[cfg(any(feature = "mainnet-beta", feature = "staging"))] {
        pub const PYTH_ID: Pubkey = pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    } else {
        // The key of the mock program on localnet (see its declared id)
        pub const PYTH_ID: Pubkey = pubkey!("5XaaR94jBubdbrRrNW7DtRvZeWvLhSHkEGU3jHTEXV3C");
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "devnet")] {
        pub const SWITCHBOARD_PULL_ID: Pubkey = pubkey!("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
    } else {
        pub const SWITCHBOARD_PULL_ID: Pubkey = pubkey!("SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv");
    }
}

pub const NATIVE_STAKE_ID: Pubkey = pubkey!("Stake11111111111111111111111111111111111111");

cfg_if::cfg_if! {
    if #[cfg(feature = "devnet")] {
        pub const SPL_SINGLE_POOL_ID: Pubkey = pubkey!("SVSPxpvHdN29nkVg9rPapPNDddN5DipNLRUFhyjFThE");
    } else if #[cfg(any(feature = "mainnet-beta", feature = "staging"))] {
        pub const SPL_SINGLE_POOL_ID: Pubkey = pubkey!("SVSPxpvHdN29nkVg9rPapPNDddN5DipNLRUFhyjFThE");
    } else {
        pub const SPL_SINGLE_POOL_ID: Pubkey = pubkey!("SVSPxpvHdN29nkVg9rPapPNDddN5DipNLRUFhyjFThE");
    }
}
