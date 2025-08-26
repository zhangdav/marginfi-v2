use anchor_lang::solana_program::{instruction::Instruction, pubkey::Pubkey};
use anchor_lang::{InstructionData, ToAccountMetas};
use fixtures::{assert_custom_error, prelude::*};
use marginfi::prelude::*;
use pretty_assertions::assert_eq;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::system_program;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, signer::Signer, transaction::Transaction,
};

// Flashloan tests
// 1. Flashloan success (1 action)
// 2. Flashloan success (3 actions)
// 3. Flashloan fails because of bad account health
// 4. Flashloan fails because of non whitelisted account
// 5. Flashloan fails because of missing `end_flashloan` ix
// 6. Flashloan fails because of invalid instructions sysvar
// 7. Flashloan fails because of invalid `end_flashloan` ix order
// 8. Flashloan fails because `end_flashloan` ix is for another account
// 9. Flashloan fails because account is already in a flashloan
// 10. Flashloan fails because account transfer during flashloan

#[tokio::test]
async fn flashloan_success_1op() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let sol_bank = test_f.get_bank(&BankMint::Sol);

    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_f_sol = test_f
        .sol_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_f_sol.key, sol_bank, 1_000, None)
        .await?;

    let borrower_mfi_account_f = test_f.create_marginfi_account().await;

    let borrower_token_account_f_sol = test_f.sol_mint.create_empty_token_account().await;

    let borrow_ix = borrower_mfi_account_f
        .make_bank_borrow_ix(borrower_token_account_f_sol.key, sol_bank, 1_000)
        .await;

    let repay_ix = borrower_mfi_account_f
        .make_bank_repay_ix(
            borrower_token_account_f_sol.key,
            sol_bank,
            1_000,
            Some(true),
        )
        .await;

    let flash_loan_result = borrower_mfi_account_f
        .try_flashloan(vec![borrow_ix, repay_ix], vec![], vec![], None)
        .await;

    assert!(flash_loan_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn flashloan_success_3op() -> anyhow::Result<()> {
    let test_f = TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let sol_bank = test_f.get_bank(&BankMint::Sol);

    let lender_mfi_account_f = test_f.create_marginfi_account().await;
    let lender_token_account_f_sol = test_f
        .sol_mint
        .create_token_account_and_mint_to(1_000)
        .await;
    lender_mfi_account_f
        .try_bank_deposit(lender_token_account_f_sol.key, sol_bank, 1_000, None)
        .await?;

    let borrower_mfi_account_f = test_f.create_marginfi_account().await;

    let borrower_token_account_f_sol = test_f.sol_mint.create_empty_token_account().await;

    // Create borrow and repay instructions
    let mut ixs = Vec::new();
    for _ in 0..3 {
        let borrow_ix = borrower_mfi_account_f
            .make_bank_borrow_ix(borrower_token_account_f_sol.key, sol_bank, 1_000)
            .await;
        ixs.push(borrow_ix);

        let repay_ix = borrower_mfi_account_f
            .make_bank_repay_ix(
                borrower_token_account_f_sol.key,
                sol_bank,
                1_000,
                Some(true),
            )
            .await;
        ixs.push(repay_ix);
    }

    ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(1_400_000));

    let flash_loan_result = borrower_mfi_account_f
        .try_flashloan(ixs, vec![], vec![], None)
        .await;

    assert!(flash_loan_result.is_ok());

    Ok(())
}