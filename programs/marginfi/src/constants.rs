use fixed::types::I80F48;
use fixed_macro::types::I80F48;

pub const MAX_ORACLE_KEYS: usize = 5;
pub const ASSET_TAG_DEFAULT: u8 = 0;
pub const TOTAL_ASSET_VALUE_INIT_LIMIT_INACTIVE: u64 = 0;

// Anyone can try to settle bad debts in an account without permission or administrator status
pub const PERMISSIONLESS_BAD_DEBT_SETTLEMENT_FLAG: u64 = 1 << 2;

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
