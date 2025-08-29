use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

use anchor_lang::solana_program::pubkey;
use anchor_lang::AccountDeserialize;
use anyhow::bail;
use base64::{prelude::BASE64_STANDARD, Engine};
use bytemuck::Zeroable;
use fixed::types::I80F48;
use marginfi::{
    constants::ASSET_TAG_DEFAULT,
    state::{
        bank_cache::BankCache,
        health_cache::HealthCache,
        marginfi_account::MarginfiAccount,
        marginfi_group::{Bank, BankOperationalState, RiskTier},
        price::OracleSetup,
    },
};
use solana_account_decoder::UiAccountData;
use solana_cli_output::CliAccount;
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;

#[tokio::test]
async fn account_field_values_reg() -> anyhow::Result<()> {
    let account_fixtures_path = "tests/fixtures/marginfi_account";

    // Sample 1

    let mut path = PathBuf::from_str(account_fixtures_path).unwrap();
    path.push("marginfi_account_sample_1.json");
    let mut file = File::open(&path).unwrap();
    let mut account_info_raw = String::new();
    file.read_to_string(&mut account_info_raw).unwrap();

    let account: CliAccount = serde_json::from_str(&account_info_raw).unwrap();
    let UiAccountData::Binary(data, _) = account.keyed_account.account.data else {
        bail!("Expected Binary format for fixtures")
    };
    let account = MarginfiAccount::try_deserialize(&mut BASE64_STANDARD.decode(data)?.as_slice())?;

    assert_eq!(
        account.group,
        pubkey!("4qp6Fx6tnZkY5Wropq9wUYgtFxXKwE6viZxFHg3rdAG8")
    );
    assert_eq!(
        account.authority,
        pubkey!("Dq7wypbedtaqQK9QqEFvfrxc4ppfRGXCeTVd7ee7n2jw")
    );
    assert_eq!(account.account_flags, 0);
    // health cache doesn't exist on these old accounts, but it also doesn't matter since it's read-only
    assert_eq!(account.health_cache, HealthCache::zeroed());
    assert_eq!(account._padding0, [0; 13]);

    let balance_1 = account.lending_account.balances[0];
    assert!(balance_1.is_active());
    assert_eq!(
        balance_1.bank_pk,
        pubkey!("2s37akK2eyBbp8DZgCm7RtsaEz8eJP3Nxd4urLHQv7yB")
    );
    assert_eq!(balance_1.bank_asset_tag, ASSET_TAG_DEFAULT);
    assert_eq!(balance_1._pad0, [0; 6]);
    assert_eq!(
        I80F48::from(balance_1.asset_shares),
        I80F48::from_str("1650216221.466876226897366").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.liability_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.emissions_outstanding),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.last_update),
        I80F48::from_str("1711158766").unwrap()
    );
    assert_eq!(balance_1._padding, [0; 1]);

    let balance_2 = account.lending_account.balances[1];
    assert!(balance_2.is_active());
    assert_eq!(
        balance_2.bank_pk,
        pubkey!("CCKtUs6Cgwo4aaQUmBPmyoApH2gUDErxNZCAntD6LYGh")
    );
    assert_eq!(balance_2.bank_asset_tag, ASSET_TAG_DEFAULT);
    assert_eq!(balance_2._pad0, [0; 6]);
    assert_eq!(
        I80F48::from(balance_2.asset_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.liability_shares),
        I80F48::from_str("3806372611.588862122556122").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.emissions_outstanding),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.last_update),
        I80F48::from_str("1711158793").unwrap()
    );
    assert_eq!(balance_2._padding, [0; 1]);

    // Sample 2

    let mut path = PathBuf::from_str(account_fixtures_path).unwrap();
    path.push("marginfi_account_sample_2.json");
    let mut file = File::open(&path).unwrap();
    let mut account_info_raw = String::new();
    file.read_to_string(&mut account_info_raw).unwrap();

    let account: CliAccount = serde_json::from_str(&account_info_raw).unwrap();
    let UiAccountData::Binary(data, _) = account.keyed_account.account.data else {
        bail!("Expecting Binary format for fixtures")
    };
    let account = MarginfiAccount::try_deserialize(&mut BASE64_STANDARD.decode(data)?.as_slice())?;

    assert_eq!(
        account.group,
        pubkey!("4qp6Fx6tnZkY5Wropq9wUYgtFxXKwE6viZxFHg3rdAG8")
    );
    assert_eq!(
        account.authority,
        pubkey!("3T1kGHp7CrdeW9Qj1t8NMc2Ks233RyvzVhoaUPWoBEFK")
    );
    assert_eq!(account.account_flags, 0);
    assert_eq!(account._padding0, [0; 13]);

    let balance_1 = account.lending_account.balances[0];
    assert!(balance_1.is_active());
    assert_eq!(
        balance_1.bank_pk,
        pubkey!("6hS9i46WyTq1KXcoa2Chas2Txh9TJAVr6n1t3tnrE23K")
    );
    assert_eq!(balance_1.bank_asset_tag, ASSET_TAG_DEFAULT);
    assert_eq!(balance_1._pad0, [0; 6]);
    assert_eq!(
        I80F48::from(balance_1.asset_shares),
        I80F48::from_str("470.952530958931234").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.liability_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.emissions_outstanding),
        I80F48::from_str("26891413.388324654086347").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.last_update),
        I80F48::from_str("1705760628").unwrap()
    );
    assert_eq!(balance_1._padding, [0; 1]);
    
    let balance_2 = account.lending_account.balances[1];
    assert!(!balance_2.is_active());
    assert_eq!(
        balance_2.bank_pk,
        pubkey!("11111111111111111111111111111111")
    );
    assert_eq!(balance_2.bank_asset_tag, ASSET_TAG_DEFAULT);
    assert_eq!(balance_2._pad0, [0; 6]);
    assert_eq!(
        I80F48::from(balance_2.asset_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.liability_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.emissions_outstanding),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_2.last_update),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(balance_2._padding, [0; 1]);

    // Sample 3

    let mut path = PathBuf::from_str(account_fixtures_path).unwrap();
    path.push("marginfi_account_sample_3.json");
    let mut file = File::open(&path).unwrap();
    let mut account_info_raw = String::new();
    file.read_to_string(&mut account_info_raw).unwrap();

    let account: CliAccount = serde_json::from_str(&account_info_raw).unwrap();
    let UiAccountData::Binary(data, _) = account.keyed_account.account.data else {
        bail!("Expecting Binary format for fixtures")
    };
    let account = MarginfiAccount::try_deserialize(&mut BASE64_STANDARD.decode(data)?.as_slice())?;

    assert_eq!(
        account.group,
        pubkey!("4qp6Fx6tnZkY5Wropq9wUYgtFxXKwE6viZxFHg3rdAG8")
    );

    assert_eq!(
        account.authority,
        pubkey!("7hmfVTuXc7HeX3YQjpiCXGVQuTeXonzjp795jorZukVR")
    );
    assert_eq!(account.account_flags, 0);
    assert_eq!(account._padding0, [0; 13]);

    let balance_1 = account.lending_account.balances[0];
    assert!(!balance_1.is_active());
    assert_eq!(
        balance_1.bank_pk,
        pubkey!("11111111111111111111111111111111")
    );
    assert_eq!(balance_1.bank_asset_tag, ASSET_TAG_DEFAULT);
    assert_eq!(balance_1._pad0, [0; 6]);
    assert_eq!(
        I80F48::from(balance_1.asset_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.liability_shares),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.emissions_outstanding),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(
        I80F48::from(balance_1.last_update),
        I80F48::from_str("0").unwrap()
    );
    assert_eq!(balance_1._padding, [0; 1]);

    Ok(())
}
