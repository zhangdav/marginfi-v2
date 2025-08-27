use anchor_spl::token_2022::spl_token_2022::extension::{
    transfer_fee::TransferFeeConfig, BaseStateWithExtensions,
};
use fixed::types::I80F48;
use fixed_macro::types::I80F48;
use fixtures::{assert_custom_error, assert_eq_noise, native, prelude::*};
use marginfi::{
    prelude::MarginfiError,
    state::marginfi_group::{BankConfig, BankVaultType},
};
use pretty_assertions::assert_eq;
use solana_program_test::*;
use test_case::test_case;

#[test_case(BankMint::Usdc, BankMint::Sol)]
#[test_case(BankMint::Sol, BankMint::Usdc)]
#[test_case(BankMint::PyUSD, BankMint::T22WithFee)]
#[test_case(BankMint::T22WithFee, BankMint::Sol)]
#[tokio::test]
async fn marginfi_group_handle_bankruptcy_failure_not_bankrupt(
    collateral_mint: BankMint,
    debt_mint: BankMint,
) -> anyhow::Result<()> {
    let mut test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;
    let borrow_amount = 10_000.;

    let lp_deposit_amount = 2. * borrow_amount;
    let lp_wallet_balance = get_max_deposit_amount_pre_fee(lp_deposit_amount);
    let lp_mfi_account_f = test_f.create_marginfi_account().await;
    let lp_token_account_f_sol = test_f
        .get_bank(&debt_mint)
        .mint
        .create_token_account_and_mint_to(lp_wallet_balance)
        .await;
    lp_mfi_account_f
        .try_bank_deposit(
            lp_token_account_f_sol.key,
            test_f.get_bank(&debt_mint),
            lp_deposit_amount,
            None,
        )
        .await?;

    let user_mfi_account_f = test_f.create_marginfi_account().await;
    let sufficient_collateral_amount = test_f
        .get_sufficient_collateral_for_outflow(borrow_amount, &collateral_mint, &debt_mint)
        .await;
    let user_wallet_balance = get_max_deposit_amount_pre_fee(sufficient_collateral_amount);
    let user_collateral_token_account_f = test_f
        .get_bank_mut(&collateral_mint)
        .mint
        .create_token_account_and_mint_to(user_wallet_balance)
        .await;
    let user_debt_token_account_f = test_f
        .get_bank_mut(&debt_mint)
        .mint
        .create_empty_token_account()
        .await;
    user_mfi_account_f
        .try_bank_deposit(
            user_collateral_token_account_f.key,
            test_f.get_bank(&collateral_mint),
            sufficient_collateral_amount,
            None,
        )
        .await?;
    user_mfi_account_f
        .try_bank_borrow(
            user_debt_token_account_f.key,
            test_f.get_bank(&debt_mint),
            borrow_amount,
        )
        .await?;

    let debt_bank_f = test_f.get_bank(&debt_mint);

    let res = test_f
        .marginfi_group
        .try_handle_bankruptcy(debt_bank_f, &user_mfi_account_f)
        .await;

    assert!(res.is_err());
    assert_custom_error!(res.unwrap_err(), MarginfiError::AccountNotBankrupt);

    Ok(())
}