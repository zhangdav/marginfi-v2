import { BN, Program, workspace, getProvider } from "@coral-xyz/anchor";
import { Marginfi } from "../target/types/marginfi";
import { AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Transaction, AccountMeta } from "@solana/web3.js";
import { defaultBankConfig, ORACLE_SETUP_PYTH_PUSH } from "./utils/types";
import { deriveBankWithSeed } from "./utils/pdas";
import { marginfiGroup, ecosystem, groupAdmin, oracles, users, bankKeypairUsdc, bankKeypairA } from "./rootHooks";
import { addBankWithSeed, closeBank } from "./utils/group-instructions";
import { USER_ACCOUNT } from "./utils/mocks";
import { depositIx, withdrawIx, composeRemainingAccounts } from "./utils/user-instructions";
import { assert } from "chai";
import { expectFailedTxWithError } from "./utils/genericTests";
import { dumpAccBalances } from "./utils/tools";

describe("Close bank", () => {
    const program = workspace.Marginfi as Program<Marginfi>;
    const provider = getProvider() as AnchorProvider;
    let bankKey: PublicKey;
    const seed = new BN(987613);

    before(async () => {
        const config = defaultBankConfig();
        [bankKey] = deriveBankWithSeed(
            program.programId,
            marginfiGroup.publicKey,
            ecosystem.tokenAMint.publicKey,
            seed
        );
        await groupAdmin.mrgnProgram.provider.sendAndConfirm(
            new Transaction().add(
                await addBankWithSeed(groupAdmin.mrgnProgram, {
                    marginfiGroup: marginfiGroup.publicKey,
                    feePayer: groupAdmin.wallet.publicKey,
                    bankMint: ecosystem.tokenAMint.publicKey,
                    config: config,
                    seed: seed,
                }),
                await program.methods
                    .lendingPoolConfigureBankOracle(
                        ORACLE_SETUP_PYTH_PUSH,
                        oracles.tokenAOracle.publicKey
                    )
                    .accountsPartial({
                        group: marginfiGroup.publicKey,
                        bank: bankKey,
                        admin: groupAdmin.wallet.publicKey,
                    })
                    .remainingAccounts([
                        {
                            pubkey: oracles.tokenAOracle.publicKey,
                            isSigner: false,
                            isWritable: false,
                        } as AccountMeta,
                    ])
                    .instruction()
            )
        );
    });

    it("bank cannot close with open positions", async () => {
        const userAcc = users[0].accounts.get(USER_ACCOUNT);
        const amount = new BN(1 * 10 ** ecosystem.tokenADecimals);
        await users[0].mrgnProgram.provider.sendAndConfirm(
            new Transaction().add(
                await depositIx(users[0].mrgnProgram, {
                    marginfiAccount: userAcc,
                    bank: bankKey,
                    tokenAccount: users[0].tokenAAccount,
                    amount: amount,
                    depositUpToLimit: false,
                })
            )
        );

        const bankAfterDeposit = await program.account.bank.fetch(bankKey);
        assert.equal(bankAfterDeposit.lendingPositionCount, 1);

        await expectFailedTxWithError(
            async () => {
                await groupAdmin.mrgnProgram.provider.sendAndConfirm(
                    new Transaction().add(
                        await closeBank(groupAdmin.mrgnProgram, {
                            bank: bankKey,
                        })
                    )
                );
            },
            "BankCannotClose",
            6081
        );
    });

    it("bank can be closed after the last user withdraws", async () => {
        const userAcc = users[0].accounts.get(USER_ACCOUNT);
        const acc = await users[0].mrgnProgram.account.marginfiAccount.fetch(userAcc);
        dumpAccBalances(acc);

        await users[0].mrgnProgram.provider.sendAndConfirm(
            new Transaction().add(
                await withdrawIx(users[0].mrgnProgram, {
                    marginfiAccount: userAcc,
                    bank: bankKey,
                    tokenAccount: users[0].tokenAAccount,
                    remaining: composeRemainingAccounts([
                        [bankKeypairUsdc.publicKey, oracles.usdcOracle.publicKey],
                        [bankKeypairA.publicKey, oracles.tokenAOracle.publicKey],
                        [bankKey, oracles.tokenAOracle.publicKey],
                    ]),
                    amount: new BN(0),
                    withdrawAll: true,
                })
            )
        );

        const bankAfterWithdraw = await program.account.bank.fetch(bankKey);
        assert.equal(bankAfterWithdraw.lendingPositionCount, 0);

        const groupBefore = await program.account.marginfiGroup.fetch(
            marginfiGroup.publicKey
        );
        await groupAdmin.mrgnProgram.provider.sendAndConfirm(
            new Transaction().add(
                await closeBank(groupAdmin.mrgnProgram, {
                    bank: bankKey,
                })
            )
        );
        const groupAfter = await program.account.marginfiGroup.fetch(
            marginfiGroup.publicKey
        );
        assert.equal(groupAfter.banks, groupBefore.banks - 1);

        const info = await provider.connection.getAccountInfo(bankKey);
        assert.isNull(info);
    });
});