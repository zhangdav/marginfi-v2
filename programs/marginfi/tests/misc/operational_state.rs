use fixed_macro::types::I80F48;
use fixtures::{assert_custom_error, prelude::*};
use marginfi::{
    prelude::MarginfiError,
    state::marginfi_group::{BankConfig, BankConfigOpt, BankOperationalState},
};
use pretty_assertions::assert_eq;
use solana_program_test::*;

#[tokio::test]
async fn marginfi_group_bank_paused_should_error() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings {
        banks: vec![TestBankSetting {
            mint: BankMint::Usdc,
            config: None,
        }],
        protocol_fees: false,
    }))
    .await;

    let usdc_bank_f = test_f.get_bank(&BankMint::Usdc);

    test_f
        .marginfi_group
        .try_lending_pool_configure_bank(
            usdc_bank_f,
            BankConfigOpt {
                operational_state: Some(BankOperationalState::Paused),
                ..BankConfigOpt::default()
            },
        )
        .await?;

    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_usdc = test_f
        .usdc_mint
        .create_token_account_and_mint_to(100_000)
        .await;
    let res = lender_mfi_account_f
        .try_bank_deposit(lender_token_account_usdc.key, usdc_bank_f, 100_000, None)
        .await;

    assert!(res.is_err());
    assert_custom_error!(res.unwrap_err(), MarginfiError::BankPaused);

    Ok(())
}

#[tokio::test]
async fn marginfi_group_bank_reduce_only_withdraw_success() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings {
        banks: vec![TestBankSetting {
            mint: BankMint::Usdc,
            config: None,
        }],
        protocol_fees: false,
    }))
    .await;

    let usdc_bank_f = test_f.get_bank(&BankMint::Usdc);

    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_usdc = test_f
        .usdc_mint
        .create_token_account_and_mint_to(100_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_usdc.key, usdc_bank_f, 100_000, None)
        .await?;

    usdc_bank_f
        .update_config(
            BankConfigOpt {
                operational_state: Some(BankOperationalState::ReduceOnly),
                ..Default::default()
            },
            None,
        )
        .await?;

    let res = lender_mfi_account_f
        .try_bank_withdraw(lender_token_account_usdc.key, usdc_bank_f, 0, Some(true))
        .await;

    assert!(res.is_ok());

    Ok(())
}
