use fixed_macro::types::I80F48;
use fixtures::{
    assert_custom_error,
    test::{
        BankMint, TestBankSetting, TestFixture, TestSettings, DEFAULT_SOL_TEST_BANK_CONFIG,
        PYTH_SOL_EQUIVALENT_FEED, PYTH_SOL_FEED, PYTH_USDC_FEED,
    },
};
use marginfi::{
    prelude::MarginfiError,
    state::marginfi_group::{BankConfig, BankConfigOpt, BankVaultType},
};
use solana_program_test::tokio;

#[tokio::test]
/// Usdc deposits $5000 SOLE and $500 USDC, borrowing $990 SOL should fail due to stale oracle
async fn re_one_oracle_stale_failure() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let usdc_bank = test_f.get_bank(&BankMint::Usdc);
    let sol_bank = test_f.get_bank(&BankMint::Sol);
    let sol_eq_bank = test_f.get_bank(&BankMint::SolEquivalent);

    // Make SOLE feed stale
    test_f.set_time(0);
    test_f.set_pyth_oracle_timestamp(PYTH_SOL_FEED, 120).await;
    test_f.set_pyth_oracle_timestamp(PYTH_USDC_FEED, 120).await;
    test_f
        .set_pyth_oracle_timestamp(PYTH_SOL_EQUIVALENT_FEED, 0)
        .await;
    test_f.advance_time(120).await;

    // Fund SOL lender
    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_sol = test_f
        .sol_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_sol.key, sol_bank, 1_000, None)
        .await?;

    // Fund SOL borrower
    let borrower_mfi_account_f = test_f.create_marginfi_account().await;
    let borrower_token_account_f_usdc = test_f
        .usdc_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    let borrower_token_account_f_sol_eq = test_f
        .sol_equivalent_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    let borrower_token_account_f_sol = test_f.sol_mint.create_empty_token_account().await;

    borrower_mfi_account_f
        .try_bank_deposit(borrower_token_account_f_usdc.key, usdc_bank, 500, None)
        .await?;

    borrower_mfi_account_f
        .try_bank_deposit(borrower_token_account_f_sol_eq.key, sol_eq_bank, 500, None)
        .await?;

    // Borrow SOL
    let res = borrower_mfi_account_f
        .try_bank_borrow_with_nonce(borrower_token_account_f_sol.key, sol_bank, 99, 1)
        .await;

    assert!(res.is_err());
    // Note that the error is RiskEngineInitRejected, and not PythPushStalePrice because
    // we're ignoring the stale oracle errors for the collateral banks. This is because
    // the most important thing is to have enough collateral (in non-stale banks) in total.
    assert_custom_error!(res.unwrap_err(), MarginfiError::RiskEngineInitRejected);

    // Make SOLE feed not stale
    sol_eq_bank
        .update_config(
            BankConfigOpt {
                oracle_max_age: Some(200),
                ..Default::default()
            },
            None,
        )
        .await?;

    // Borrow SOL
    let res = borrower_mfi_account_f
        .try_bank_borrow_with_nonce(borrower_token_account_f_sol.key, sol_bank, 99, 2)
        .await;

    assert!(res.is_ok());

    Ok(())
}

#[tokio::test]
/// Usdc deposits $500 of SOLE and $500 of USDC, but SOLE oracle is stale
/// -> borrowing 51 sol should not succeed ($500 USDC collateral < $510 SOL borrow), but borrowing 40 SOL should go through despite the stale SOL oracle ($500 USDC collateral > $400 SOL borrow)
async fn re_one_oracle_stale_success() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let usdc_bank = test_f.get_bank(&BankMint::Usdc);
    let sol_bank = test_f.get_bank(&BankMint::Sol);
    let sol_eq_bank = test_f.get_bank(&BankMint::SolEquivalent);

    // Make SOLE feed stale
    test_f.set_time(0);
    test_f.set_pyth_oracle_timestamp(PYTH_SOL_FEED, 120).await;
    test_f.set_pyth_oracle_timestamp(PYTH_USDC_FEED, 120).await;
    test_f
        .set_pyth_oracle_timestamp(PYTH_SOL_EQUIVALENT_FEED, 0)
        .await;
    test_f.advance_time(120).await;

    // Fund SOL lender
    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_sol = test_f
        .sol_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_sol.key, sol_bank, 1_000, None)
        .await?;

    // Fund SOL borrower
    let borrower_mfi_account_f = test_f.create_marginfi_account().await;
    let borrower_token_account_f_usdc = test_f
        .usdc_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    let borrower_token_account_f_sol_eq = test_f
        .sol_equivalent_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    let borrower_token_account_f_sol = test_f.sol_mint.create_empty_token_account().await;

    borrower_mfi_account_f
        .try_bank_deposit(borrower_token_account_f_usdc.key, usdc_bank, 500, None)
        .await?;

    borrower_mfi_account_f
        .try_bank_deposit(borrower_token_account_f_sol_eq.key, sol_eq_bank, 500, None)
        .await?;

    // Borrow SOL
    let res = borrower_mfi_account_f
        .try_bank_borrow(borrower_token_account_f_sol.key, sol_bank, 51)
        .await;

    assert!(res.is_err());

    assert_custom_error!(res.unwrap_err(), MarginfiError::RiskEngineInitRejected);

    let res = borrower_mfi_account_f
        .try_bank_borrow(borrower_token_account_f_sol.key, sol_bank, 40)
        .await;

    assert!(res.is_ok());

    Ok(())
}

#[tokio::test]
/// Borrowing from a bank with a stale oracle should fail
async fn re_one_oracle_stale_failure_2() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let usdc_bank = test_f.get_bank(&BankMint::Usdc);
    let sol_bank = test_f.get_bank(&BankMint::Sol);

    test_f.set_time(0);

    // Fund SOL lender
    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_sol = test_f
        .sol_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_sol.key, sol_bank, 1_000, None)
        .await?;

    // Fund SOL borrower
    let borrower_mfi_account_f = test_f.create_marginfi_account().await;
    let borrower_token_account_f_usdc =
        test_f.usdc_mint.create_token_account_and_mint_to(500).await;
    let borrower_token_account_f_sol = test_f.sol_mint.create_empty_token_account().await;

    borrower_mfi_account_f
        .try_bank_deposit(borrower_token_account_f_usdc.key, usdc_bank, 500, None)
        .await?;

    // Make SOL oracle stale
    test_f.set_time(0);
    test_f.set_pyth_oracle_timestamp(PYTH_USDC_FEED, 120).await;
    test_f.set_pyth_oracle_timestamp(PYTH_SOL_FEED, 0).await;
    test_f.advance_time(120).await;

    let res = borrower_mfi_account_f
        .try_bank_borrow_with_nonce(borrower_token_account_f_sol.key, sol_bank, 40, 1)
        .await;

    assert!(res.is_err());
    assert_custom_error!(res.unwrap_err(), MarginfiError::PythPushStalePrice);

    // Make SOL oracle not stale
    test_f.set_pyth_oracle_timestamp(PYTH_SOL_FEED, 120).await;

    let res = borrower_mfi_account_f
        .try_bank_borrow_with_nonce(borrower_token_account_f_sol.key, sol_bank, 40, 2)
        .await;
    assert!(res.is_ok());

    Ok(())
}
