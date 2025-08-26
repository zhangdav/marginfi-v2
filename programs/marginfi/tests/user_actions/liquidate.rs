use anchor_spl::token_2022::spl_token_2022::extension::{
    transfer_fee::TransferFeeConfig, BaseStateWithExtensions,
};
use fixed::types::I80F48;
use fixed_macro::types::I80F48;
use fixtures::{assert_custom_error, assert_eq_noise, native, prelude::*};
use marginfi::{
    prelude::*,
    state::{
        emode::EmodeEntry,
        marginfi_group::{Bank, BankConfig, BankConfigOpt, BankVaultType},
    },
};
use pretty_assertions::assert_eq;
use solana_program_test::*;
use test_case::test_case;

#[test_case(100., 9.9, 1., BankMint::Usdc, BankMint::Sol)]
#[test_case(123., 122., 10., BankMint::SolEquivalent, BankMint::SolEqIsolated)]
#[test_case(1_000., 999., 10., BankMint::Usdc, BankMint::T22WithFee)]
#[test_case(2_000., 99., 1_000., BankMint::T22WithFee, BankMint::SolEquivalent)]
#[test_case(2_000., 1_999., 2_000., BankMint::Usdc, BankMint::PyUSD)]
#[tokio::test]
async fn marginfi_account_liquidation_success(
    deposit_amount: f64,
    borrow_amount: f64,
    liquidate_amount: f64,
    collateral_mint: BankMint,
    debt_mint: BankMint,
) -> anyhow::Result<()> {
    let mut test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    {
        let lp_deposit_amount = 2. * borrow_amount;
        let lp_wallet_balance = get_max_deposit_amount_pre_fee(lp_deposit_amount);
        let lp_mfi_account_f = test_f.create_marginfi_account().await;
        let lp_collateral_token_account = test_f
            .get_bank(&debt_mint)
            .mint
            .create_token_account_and_mint_to(lp_wallet_balance)
            .await;
        lp_mfi_account_f
            .try_bank_deposit(
                lp_collateral_token_account.key,
                test_f.get_bank(&debt_mint),
                lp_deposit_amount,
                None,
            )
            .await?;
    }

    let (liquidatee_mfi_account_f, borrow_amount_actual, collateral_index, debt_index) = {
        let liquidatee_mfi_account_f = test_f.create_marginfi_account().await;
        let liquidatee_wallet_balance = get_max_deposit_amount_pre_fee(deposit_amount);
        let liquidatee_collateral_token_account_f = test_f
            .get_bank_mut(&collateral_mint)
            .mint
            .create_token_account_and_mint_to(liquidatee_wallet_balance)
            .await;
        let liquidatee_debt_token_account_f = test_f
            .get_bank_mut(&debt_mint)
            .mint
            .create_empty_token_account()
            .await;
        let collateral_bank = test_f.get_bank(&collateral_mint);
        liquidatee_mfi_account_f
            .try_bank_deposit(
                liquidatee_collateral_token_account_f.key,
                collateral_bank,
                deposit_amount,
                None,
            )
            .await?;
        let debt_bank = test_f.get_bank(&debt_mint);
        liquidatee_mfi_account_f
            .try_bank_borrow(
                liquidatee_debt_token_account_f.key,
                debt_bank,
                borrow_amount,
            )
            .await?;

        let liquidatee_mfi_ma = liquidatee_mfi_account_f.load().await;
        let collateral_index = liquidatee_mfi_ma
            .lending_account
            .balances
            .iter()
            .position(|b| b.is_active() && b.bank_pk == collateral_bank.key)
            .unwrap();
        let debt_index = liquidatee_mfi_ma
            .lending_account
            .balances
            .iter()
            .position(|b| b.is_active() && b.bank_pk == debt_bank.key)
            .unwrap();

        let debt_bank = test_f.get_bank(&debt_mint).load().await;
        let borrow_amount_actual_native = debt_bank.get_liability_amount(
            liquidatee_mfi_ma.lending_account.balances[debt_index]
                .liability_shares
                .into(),
        )?;
        let borrow_amount_actual = borrow_amount_actual_native.to_num::<f64>()
            / 10_f64.powf(debt_bank.mint_decimals as f64);
        (
            liquidatee_mfi_account_f,
            borrow_amount_actual,
            collateral_index,
            debt_index,
        )
    };

    let liquidator_mfi_account_f = {
        let liquidator_mfi_account_f = test_f.create_marginfi_account().await;
        let liquidator_wallet_balance = get_max_deposit_amount_pre_fee(borrow_amount_actual);
        let liquidator_collateral_token_account_f = test_f
            .get_bank_mut(&debt_mint)
            .mint
            .create_token_account_and_mint_to(liquidator_wallet_balance)
            .await;
        liquidator_mfi_account_f
            .try_bank_deposit(
                liquidator_collateral_token_account_f.key,
                test_f.get_bank(&debt_mint),
                borrow_amount_actual,
                None,
            )
            .await?;

        liquidator_mfi_account_f
    };

    test_f
        .get_bank_mut(&collateral_mint)
        .update_config(
            BankConfigOpt {
                asset_weight_init: Some(I80F48!(0.25).into()),
                asset_weight_maint: Some(I80F48!(0.5).into()),
                ..Default::default()
            },
            None,
        )
        .await?;

    let collateral_bank_f = test_f.get_bank(&collateral_mint);
    let debt_bank_f = test_f.get_bank(&debt_mint);

    liquidator_mfi_account_f
        .try_liquidate(
            &liquidatee_mfi_account_f,
            collateral_bank_f,
            liquidate_amount,
            debt_bank_f,
        )
        .await?;

    let collateral_bank = collateral_bank_f.load().await;
    let debt_bank = debt_bank_f.load().await;

    let liquidator_mfi_ma = liquidator_mfi_account_f.load().await;
    let liquidatee_mfi_ma = liquidatee_mfi_account_f.load().await;

    let collateral_mint_liquidator_balance = collateral_bank.get_asset_amount(
        liquidator_mfi_ma.lending_account.balances[collateral_index]
            .asset_shares
            .into(),
    )?;
    let expected_collateral_mint_liquidator_balance =
        native!(liquidate_amount, collateral_bank_f.mint.mint.decimals, f64);
    assert_eq!(
        expected_collateral_mint_liquidator_balance,
        collateral_mint_liquidator_balance,
    );

    let debt_paid_out = liquidate_amount * 0.975 * collateral_bank_f.get_price().await
        / debt_bank_f.get_price().await;

    let expected_debt_mint_liquidator_balance = I80F48::from(native!(
        borrow_amount_actual - debt_paid_out,
        debt_bank_f.mint.mint.decimals,
        f64
    ));
    let debt_mint_liquidator_balance = debt_bank.get_asset_amount(
        liquidator_mfi_ma.lending_account.balances[debt_index]
            .asset_shares
            .into(),
    )?;
    assert_eq_noise!(
        expected_debt_mint_liquidator_balance,
        debt_mint_liquidator_balance,
        1.
    );

    let debt_covered = liquidate_amount * 0.95 * collateral_bank_f.get_price().await
        / debt_bank_f.get_price().await;
    let expected_debt_mint_liquidatee_balance = I80F48::from(native!(
        borrow_amount_actual - debt_covered,
        debt_bank_f.mint.mint.decimals,
        f64
    ));
    let debt_mint_liquidatee_balance = debt_bank.get_liability_amount(
        liquidatee_mfi_ma.lending_account.balances[debt_index]
            .liability_shares
            .into(),
    )?;
    assert_eq_noise!(
        expected_debt_mint_liquidatee_balance,
        debt_mint_liquidatee_balance,
        1.
    );

    let expected_collateral_mint_liquidatee_balance = I80F48::from(native!(
        deposit_amount - liquidate_amount,
        collateral_bank_f.mint.mint.decimals,
        f64
    ));
    let collateral_mint_liquidatee_balance = collateral_bank
        .get_liability_amount(
            liquidatee_mfi_ma.lending_account.balances[collateral_index]
                .asset_shares
                .into(),
        )
        .unwrap();
    assert_eq_noise!(
        collateral_mint_liquidatee_balance,
        expected_collateral_mint_liquidatee_balance,
        1.
    );

    let insurance_fund_fee = liquidate_amount * 0.025 * collateral_bank_f.get_price().await
        / debt_bank_f.get_price().await;
    let expected_insurance_fund_usdc_pre_fee =
        native!(insurance_fund_fee, debt_bank_f.mint.mint.decimals, f64);
    let if_transfer_fee = debt_bank_f
        .mint
        .load_state()
        .await
        .get_extension::<TransferFeeConfig>()
        .map(|tf| {
            tf.calculate_epoch_fee(0, expected_insurance_fund_usdc_pre_fee)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let expected_insurance_fund_usdc =
        (expected_insurance_fund_usdc_pre_fee - if_transfer_fee) as i64;

    let insurance_fund_usdc = debt_bank_f
        .get_vault_token_account(BankVaultType::Insurance)
        .await
        .balance()
        .await as i64;
    assert_eq_noise!(expected_insurance_fund_usdc, insurance_fund_usdc, 1);

    Ok(())
}
