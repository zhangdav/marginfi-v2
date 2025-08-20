use std::collections::BTreeMap;

use crate::errors::MarginfiError;
use crate::prelude::MarginfiResult;
use crate::state::marginfi_group::WrappedI80F48;
use crate::{assert_struct_align, assert_struct_size, check};
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use fixed::types::I80F48;
use type_layout::TypeLayout;

// Enable eMode flag
pub const EMODE_ON: u64 = 1;

// Limit each config to 10 entries
pub const MAX_EMODE_ENTRIES: usize = 10;
// Represents an invalid tag, used as a sentinel value
pub const EMODE_TAG_EMPTY: u16 = 0;

assert_struct_size!(EmodeSettings, 424);
assert_struct_align!(EmodeSettings, 8);
#[repr(C)]
#[derive(
    AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Eq, Pod, Zeroable, Copy, Clone, TypeLayout,
)]
pub struct EmodeSettings {
    // The eMode type of the current bank (for example, all stablecoins may be tag=1)
    pub emode_tag: u16,
    pub pad0: [u8; 6],
    pub timestamp: i64,
    pub flags: u64,
    // A collection of eMode policies defined for this bank (maximum 10 entries)
    pub emode_config: EmodeConfig,
}

// Returns an all-zero structure to facilitate initialization of on-chain accounts
impl Default for EmodeSettings {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl EmodeSettings {
    pub fn validate_entries(&self) -> MarginfiResult {
        for entry in self.emode_config.entries {
            if entry.is_empty() {
                continue;
            }
            let asset_init_w: I80F48 = I80F48::from(entry.asset_weight_init);
            let asset_maint_w: I80F48 = I80F48::from(entry.asset_weight_maint);

            // The initial mortgage rate must be between 0 (equivalent to 0%) and 1 (equivalent to 100%)
            check!(
                asset_init_w >= I80F48::ZERO && asset_init_w <= I80F48::ONE,
                MarginfiError::BadEmodeConfig
            );
            // The liquidation mortgage ratio cannot be higher than 2 (200%) - a reasonable upper limit is reserved
            check!(
                asset_maint_w <= (I80F48::ONE + I80F48::ONE),
                MarginfiError::BadEmodeConfig
            );
            // The maintenance mortgage rate must be ≥ the initial mortgage rate (otherwise the user will be liquidated as soon as the loan is completed)
            check!(asset_maint_w >= asset_init_w, MarginfiError::BadEmodeConfig);
        }

        // Check if there are duplicate tags in all entries
        self.check_dupes()?;

        Ok(())
    }

    fn check_dupes(&self) -> MarginfiResult {
        let non_empty_tags: Vec<u16> = self
            .emode_config
            .entries
            .iter()
            .filter(|e| !e.is_empty())
            .map(|e| e.collateral_bank_emode_tag)
            .collect();

        if non_empty_tags.windows(2).any(|w| w[0] == w[1]) {
            err!(MarginfiError::BadEmodeConfig)
        } else {
            Ok(())
        }
    }

    // Check whether the EMODE_ON flag is set, that is, whether the current emode is enabled
    pub fn is_enabled(&self) -> bool {
        self.flags & EMODE_ON != 0
    }

    /// Sets EMODE on flag if configuration has any entries, removes the flag if it has no entries.
    pub fn update_emode_enabled(&mut self) {
        if self.emode_config.has_entries() {
            self.flags |= EMODE_ON;
        } else {
            self.flags &= !EMODE_ON;
        }
    }
}

assert_struct_size!(EmodeConfig, 400);
assert_struct_align!(EmodeConfig, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Debug, PartialEq, Eq, Pod, Zeroable, Copy, Clone, TypeLayout,
)]
pub struct EmodeConfig {
    pub entries: [EmodeEntry; MAX_EMODE_ENTRIES],
}

impl EmodeConfig {
    pub fn from_entries(entries: &[EmodeEntry]) -> Self {
        let count = entries.len();
        if count > MAX_EMODE_ENTRIES {
            panic!(
                "Too many EmodeEntry items {:?}, maximum allowed {:?}",
                count, MAX_EMODE_ENTRIES
            );
        }

        let mut config = Self::zeroed();
        for (i, entry) in entries.iter().enumerate() {
            config.entries[i] = *entry;
        }
        config.entries[..count].sort_by_key(|e| e.collateral_bank_emode_tag);

        config
    }

    pub fn find_with_tag(&self, tag: u16) -> Option<&EmodeEntry> {
        if tag == EMODE_TAG_EMPTY {
            return None;
        }
        self.entries.iter().find(|e| e.tag_equals(tag))
    }

    pub fn has_entries(&self) -> bool {
        self.entries.iter().any(|e| !e.is_empty())
    }
}

assert_struct_size!(EmodeEntry, 40);
assert_struct_align!(EmodeEntry, 8);
#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Debug, PartialEq, Eq, Pod, Zeroable, Copy, Clone, TypeLayout,
)]
pub struct EmodeEntry {
    // Which type of collateral object is applicable to this strategy (e.g. tag=1 is a stablecoin)
    pub collateral_bank_emode_tag: u16,
    pub flags: u8,
    pub pad0: [u8; 5],
    // Initial asset weight for lending (affects the maximum loan amount)
    pub asset_weight_init: WrappedI80F48,
    // Liquidation asset weight (affects when liquidation occurs)
    pub asset_weight_maint: WrappedI80F48,
}

impl EmodeEntry {
    pub fn is_empty(&self) -> bool {
        self.collateral_bank_emode_tag == EMODE_TAG_EMPTY
    }
    pub fn tag_equals(&self, tag: u16) -> bool {
        self.collateral_bank_emode_tag == tag
    }
}

pub fn reconcile_emode_configs<I>(configs: I) -> EmodeConfig
where
    I: IntoIterator<Item = EmodeConfig>,
{
    // TODO benchmark this in the mock program
    let mut iter = configs.into_iter();
    // Pull off the first config (if any)
    let first = match iter.next() {
        None => return EmodeConfig::zeroed(),
        Some(cfg) => cfg,
    };

    let mut merged_entries: BTreeMap<u16, (EmodeEntry, usize)> = BTreeMap::new();
    let mut num_configs = 1;

    // A helper to merge an EmodeConfig into the map
    let mut merge_cfg = |cfg: EmodeConfig| {
        for entry in cfg.entries.iter() {
            if entry.is_empty() {
                continue;
            }
            let tag = entry.collateral_bank_emode_tag;
            merged_entries
                .entry(tag)
                .and_modify(|(merged, cnt)| {
                    merged.flags = merged.flags.min(entry.flags);
                    let cur_i: I80F48 = merged.asset_weight_init.into();
                    let new_i: I80F48 = entry.asset_weight_init.into();
                    if new_i < cur_i {
                        merged.asset_weight_init = entry.asset_weight_init;
                    }
                    let cur_m: I80F48 = merged.asset_weight_maint.into();
                    let new_m: I80F48 = entry.asset_weight_maint.into();
                    if new_m < cur_m {
                        merged.asset_weight_maint = entry.asset_weight_maint;
                    }
                    *cnt += 1;
                })
                .or_insert((*entry, 1));
        }
    };

    // First config
    merge_cfg(first);

    // All following configs
    for cfg in iter {
        num_configs += 1;
        merge_cfg(cfg);
    }

    // Cllect only those tags seen in *every* config:
    let mut buf: [EmodeEntry; MAX_EMODE_ENTRIES] = [EmodeEntry::zeroed(); MAX_EMODE_ENTRIES];
    let mut buf_len = 0;

    for (_tag, (entry, cnt)) in merged_entries {
        // if cnt of appearances = num of configs, then it was in every config.
        if cnt == num_configs {
            buf[buf_len] = entry;
            buf_len += 1;
        }
    }

    // Sort what we have and pad the rest with zeroed space
    EmodeConfig::from_entries(&buf[..buf_len])
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixed_macro::types::I80F48;

    fn create_entry(tag: u16, flags: u8, init: f32, maint: f32) -> EmodeEntry {
        EmodeEntry {
            collateral_bank_emode_tag: tag,
            flags,
            pad0: [0u8; 5],
            asset_weight_init: I80F48::from_num(init).into(),
            asset_weight_maint: I80F48::from_num(maint).into(),
        }
    }

    /// "Standard" entry with flags=0, init=0.7, maint=0.8.
    fn generic_entry(tag: u16) -> EmodeEntry {
        create_entry(tag, 0, 0.7, 0.8)
    }

    #[test]
    fn test_emode_valid_entries() {
        let mut settings = EmodeSettings::zeroed();
        settings.emode_config.entries[0] = generic_entry(1);
        settings.emode_config.entries[1] = generic_entry(2);
        settings.emode_config.entries[2] = generic_entry(3);
        // Note: The remaining entries stay zeroed (and are skipped during validation).
        assert!(settings.validate_entries().is_ok());
    }

    #[test]
    fn test_emode_invalid_duplicate_tags() {
        let mut settings = EmodeSettings::zeroed();
        settings.emode_config.entries[0] = generic_entry(1);
        settings.emode_config.entries[1] = generic_entry(1);
        settings.emode_config.entries[2] = generic_entry(2);
        let result = settings.validate_entries();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), MarginfiError::BadEmodeConfig.into());
    }

    #[test]
    fn test_emode_invalid_weight_too_high() {
        let mut settings = EmodeSettings::zeroed();
        // Using a weight greater than 1.0 is invalid.
        let entry = EmodeEntry {
            collateral_bank_emode_tag: 1,
            flags: 0,
            pad0: [0u8; 5],
            asset_weight_init: I80F48!(1.2).into(),
            asset_weight_maint: I80F48!(1.3).into(),
        };
        settings.emode_config.entries[0] = entry;
        let result = settings.validate_entries();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), MarginfiError::BadEmodeConfig.into());
    }

    #[test]
    fn test_emode_invalid_weight_main_le_init() {
        let mut settings = EmodeSettings::zeroed();
        // Maint must be greater than init
        let entry = EmodeEntry {
            collateral_bank_emode_tag: 1,
            flags: 0,
            pad0: [0u8; 5],
            asset_weight_init: I80F48!(0.8).into(),
            asset_weight_maint: I80F48!(0.7).into(),
        };
        settings.emode_config.entries[0] = entry;
        let result = settings.validate_entries();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), MarginfiError::BadEmodeConfig.into());
    }

    #[test]
    fn test_reconcile_emode_single_common_tag() {
        // Example 1:
        // * Config1 has an entry with tag 101, flags 1, init 0.7, maint 0.75.
        // * Config2 has an entry with tag 101, flags 0, init 0.6, maint 0.8.
        let entry1 = create_entry(101, 1, 0.7, 0.75);
        let entry2 = create_entry(101, 0, 0.6, 0.8);
        let config1 = EmodeConfig::from_entries(&[entry1]);
        let config2 = EmodeConfig::from_entries(&[entry2]);

        let reconciled = reconcile_emode_configs(vec![config1, config2]);

        // Expected: For tag 101, flags = min(1,0)=0, init = min(0.7,0.6)=0.6, maint = min(0.75,0.8)=0.75.
        let expected_entry = create_entry(101, 0, 0.6, 0.75);

        assert_eq!(reconciled.entries[0], expected_entry);
        // The rest of the entries should be zeroed.
        for entry in reconciled.entries.iter().skip(1) {
            assert!(entry.is_empty());
        }
    }

    #[test]
    fn test_reconcile_emode_no_common_tags() {
        // Example 2:
        // * Config1 has an entry with tag 99.
        // * Config2 has an entry with tag 101.
        // * Since there is no common tag across both, the result should be an empty (zeroed) config.
        let config1 = EmodeConfig::from_entries(&[generic_entry(99)]);
        let config2 = EmodeConfig::from_entries(&[generic_entry(101)]);

        let reconciled = reconcile_emode_configs(vec![config1, config2]);

        // Verify that all entries are empty.
        assert!(reconciled.entries.iter().all(|entry| entry.is_empty()));
    }

    #[test]
    fn test_reconcile_emode_multiple_configs() {
        // Example 3:
        // * Config1 has entries with tags 101 and 99.
        // * Config2 has an entry with tag 101.
        // * Config3 has an entry with tag 101.
        // * Only tag 101 is common to all configs.
        // * For tag 101:
        //   - Config1: flags 1, init 0.7, maint 0.75
        //   - Config2: flags 0, init 0.6, maint 0.8
        //   - Config3: flags 0, init 0.65, maint 0.8
        // * The reconciled entry should have:
        //   - flags = min(1,0,0) = 0,
        //   - init = min(0.7,0.6,0.65)=0.6
        //   - maint = min(0.75,0.8,0.8)=0.75
        let entry1 = create_entry(101, 1, 0.7, 0.75);
        let entry2 = create_entry(101, 0, 0.6, 0.8);
        let entry3 = create_entry(101, 0, 0.65, 0.8);

        let config1 = EmodeConfig::from_entries(&[entry1, generic_entry(99)]);
        let config2 = EmodeConfig::from_entries(&[entry2]);
        let config3 = EmodeConfig::from_entries(&[entry3]);

        let reconciled = reconcile_emode_configs(vec![config1, config2, config3]);

        let expected_entry = create_entry(101, 0, 0.6, 0.75);

        assert_eq!(reconciled.entries[0], expected_entry);
        // All other entries should be zeroed.
        for entry in reconciled.entries.iter().skip(1) {
            assert!(entry.is_empty());
        }
    }

    #[test]
    #[should_panic(expected = "Too many EmodeEntry items")]
    fn test_emode_from_entries_panics_on_too_many_entries() {
        // Generate more entries than allowed.
        let mut entries = Vec::new();
        for i in 0..(MAX_EMODE_ENTRIES as u16 + 1) {
            entries.push(generic_entry(i));
        } 
        // This call should panic.
        let _ = EmodeConfig::from_entries(&entries);
    }
}
