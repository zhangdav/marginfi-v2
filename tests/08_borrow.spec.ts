import { workspace, Program, getProvider, AnchorProvider, BN } from "@coral-xyz/anchor";
import { Marginfi } from "../target/types/marginfi";
import { ecosystem, users, bankKeypairUsdc, bankKeypairA, oracles, bankKeypairSol, verbose } from "./rootHooks";
import { USER_ACCOUNT } from "./utils/mocks";
import { borrowIx, composeRemainingAccounts, repayIx } from "./utils/user-instructions";
import { assertI80F48Equal, expectFailedTxWithError, getTokenBalance, assertI80F48Approx, assertBNApproximately } from "./utils/genericTests";
import { Transaction } from "@solana/web3.js";
import { wrappedI80F48toBigNumber } from "@mrgnlabs/mrgn-common";
import { assert } from "chai";

describe("Borrow funds", () => {
    const program = workspace.Marginfi as Program<Marginfi>;
    const provider = getProvider() as AnchorProvider;

    // Bank has 100 USDC available to borrow
    // User has 2 Token A (worth $20) deposited
    const borrowAmountUsdc = 5;
    const borrowAmountUsdc_native = new BN(
        borrowAmountUsdc * 10 ** ecosystem.usdcDecimals
    );

    // Used fir isolated tier logic checks
    const borrowAmountSol = borrowAmountUsdc * 0.01;
    const borrowAmountSol_native = new BN(
        borrowAmountSol * 10 ** ecosystem.wsolDecimals
    );

    it("(user 0) tries to borrow usdc with a bad oracle - should fail", async () => {
        const user = users[0];
        const user0Account = user.accounts.get(USER_ACCOUNT);
        const bank = bankKeypairUsdc.publicKey;
        await expectFailedTxWithError(
            async () => {
                await user.mrgnProgram.provider.sendAndConfirm(
                    new Transaction().add(
                        await borrowIx(user.mrgnProgram, {
                            marginfiAccount: user0Account,
                            bank: bank,
                            tokenAccount: user.usdcAccount,
                            remaining: [
                                bankKeypairA.publicKey,
                                oracles.tokenAOracle.publicKey,
                                bank,
                                oracles.fakeUsdc,
                            ],
                            amount: borrowAmountUsdc_native,
                        })
                    )
                );
            },
            "WrongOracleAccountKeys",
            6055
        );
    });

    it("(user 0) borrow SOL (isolated tier) against their token A position - happy path", async () => {
        const user = users[0];
        const user0Account = user.accounts.get(USER_ACCOUNT);

        await user.mrgnProgram.provider.sendAndConfirm(
            new Transaction().add(
                await borrowIx(user.mrgnProgram, {
                    marginfiAccount: user0Account,
                    bank: bankKeypairSol.publicKey,
                    tokenAccount: user.wsolAccount,
                    remaining: composeRemainingAccounts([
                        [bankKeypairSol.publicKey, oracles.wsolOracle.publicKey],
                        [bankKeypairA.publicKey, oracles.tokenAOracle.publicKey],
                    ]),
                    amount: borrowAmountSol_native,
                })
            )
        );
    });

    it("(user 0) repays their SOL (isolated tier) debt and borrows USDC against their token A position - happy path", async () => {
        const user = users[0];
        const userAccKey = user.accounts.get(USER_ACCOUNT);
    
        await user.mrgnProgram.provider.sendAndConfirm(
          new Transaction().add(
            await repayIx(user.mrgnProgram, {
              marginfiAccount: userAccKey,
              bank: bankKeypairSol.publicKey,
              tokenAccount: user.wsolAccount,
              remaining: composeRemainingAccounts([
                [bankKeypairSol.publicKey, oracles.wsolOracle.publicKey],
                [bankKeypairA.publicKey, oracles.tokenAOracle.publicKey],
              ]),
              amount: new BN(0),
              repayAll: true,
            })
          )
        );
    
        const bank = bankKeypairUsdc.publicKey;
        const userUsdcBefore = await getTokenBalance(provider, user.usdcAccount);
        const bankBefore = await program.account.bank.fetch(bank);
        if (verbose) {
          console.log("user 0 USDC before: " + userUsdcBefore.toLocaleString());
          console.log(
            "usdc fees owed to bank: " +
              wrappedI80F48toBigNumber(
                bankBefore.collectedGroupFeesOutstanding
              ).toString()
          );
          console.log(
            "usdc fees owed to program: " +
              wrappedI80F48toBigNumber(
                bankBefore.collectedProgramFeesOutstanding
              ).toString()
          );
        }
    
        const user0Account = user.accounts.get(USER_ACCOUNT);
    
        await user.mrgnProgram.provider.sendAndConfirm(
          new Transaction().add(
            await borrowIx(user.mrgnProgram, {
              marginfiAccount: user0Account,
              bank: bank,
              tokenAccount: user.usdcAccount,
              remaining: [
                bankKeypairA.publicKey,
                oracles.tokenAOracle.publicKey,
                bank,
                oracles.usdcOracle.publicKey,
              ],
              amount: borrowAmountUsdc_native,
            })
          )
        );
    
        const userAcc = await program.account.marginfiAccount.fetch(user0Account);
        const bankAfter = await program.account.bank.fetch(bank);
        const balances = userAcc.lendingAccount.balances;
        assert.equal(bankAfter.borrowingPositionCount, 1);
        const userUsdcAfter = await getTokenBalance(provider, user.usdcAccount);
        if (verbose) {
          console.log("user 0 USDC after: " + userUsdcAfter.toLocaleString());
          console.log(
            "usdc fees owed to bank: " +
              wrappedI80F48toBigNumber(
                bankAfter.collectedGroupFeesOutstanding
              ).toString()
          );
          console.log(
            "usdc fees owed to program: " +
              wrappedI80F48toBigNumber(
                bankAfter.collectedProgramFeesOutstanding
              ).toString()
          );
        }
    
        assert.equal(balances[1].active, 1);
    
        // Note: the newly added balance may NOT be the last one in the list, due to sorting, so we have to find its position first
        const borrowIndex = balances.findIndex((balance) =>
          balance.bankPk.equals(bank)
        );
    
        assertI80F48Equal(balances[borrowIndex].assetShares, 0);
        // Note: The first borrow issues shares 1:1 and the shares use the same decimals
        // Note: An origination fee of 0.01 is also incurred here (configured during addBank)
        const originationFee_native = borrowAmountUsdc_native.toNumber() * 0.01;
        const amtUsdcWithFee_native = new BN(
          borrowAmountUsdc_native.toNumber() + originationFee_native
        );
        assertI80F48Approx(
          balances[borrowIndex].liabilityShares,
          amtUsdcWithFee_native
        );
        assertI80F48Equal(balances[borrowIndex].emissionsOutstanding, 0);
    
        let now = Math.floor(Date.now() / 1000);
        assertBNApproximately(balances[borrowIndex].lastUpdate, now, 2);
    
        assert.equal(
          userUsdcAfter - borrowAmountUsdc_native.toNumber(),
          userUsdcBefore
        );
    
        // The origination fee is recorded on the bank. The group gets 98%, the program gets the
        // remaining 2% (see PROGRAM_FEE_RATE)
        const origination_fee_group = originationFee_native * 0.98;
        const origination_fee_program = originationFee_native * 0.02;
        assertI80F48Approx(
          bankAfter.collectedGroupFeesOutstanding,
          origination_fee_group
        );
        assertI80F48Approx(
          bankAfter.collectedProgramFeesOutstanding,
          origination_fee_program
        );
    });

    it("(user 0) tries to borrow SOL (isolated tier) against their token A position with active debt in USDC bank - should fail", async () => {
        const user = users[0];
        const user0Account = user.accounts.get(USER_ACCOUNT);

        const borrowAmountSol = borrowAmountUsdc * 0.01;
        const borrowAmountSol_native = new BN(
            borrowAmountSol * 10 ** ecosystem.wsolDecimals
        );

        await expectFailedTxWithError(
            async () => {
                await user.mrgnProgram.provider.sendAndConfirm(
                    new Transaction().add(
                        await borrowIx(user.mrgnProgram, {
                            marginfiAccount: user0Account,
                            bank: bankKeypairSol.publicKey,
                            tokenAccount: user.wsolAccount,
                            remaining: composeRemainingAccounts([
                                [bankKeypairA.publicKey, oracles.tokenAOracle.publicKey],
                                [bankKeypairUsdc.publicKey, oracles.usdcOracle.publicKey],
                                [bankKeypairSol.publicKey, oracles.wsolOracle.publicKey],
                            ]),
                            amount: borrowAmountSol_native,
                        })
                    )
                );
            },
            "IsolatedAccountIllegalState",
            6029
        );
    });
});