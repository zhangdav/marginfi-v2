import { BN, Program, workspace } from "@coral-xyz/anchor";
import { PublicKey, Transaction } from "@solana/web3.js";
import { editStakedSettings, groupInitialize, initStakedSettings } from "./utils/group-instructions";
import { Marginfi } from "../target/types/marginfi";
import { marginfiGroup, groupAdmin, verbose, PROGRAM_FEE_FIXED, PROGRAM_FEE_RATE, globalFeeWallet } from "./rootHooks";
import { assertKeysEqual, assertI80F48Approx } from "./utils/genericTests";

describe("Init group", () => {
    const program = workspace.Marginfi as Program<Marginfi>;

    it("(admin) Init group - happy path", async () => {
        let tx = new Transaction();

        tx.add(
            await groupInitialize(program, {
                marginfiGroup: marginfiGroup.publicKey,
                admin: groupAdmin.wallet.publicKey,
            })
        );

        await groupAdmin.mrgnProgram.provider.sendAndConfirm(tx, [marginfiGroup]);

        let group = await program.account.marginfiGroup.fetch(
            marginfiGroup.publicKey
        );
        assertKeysEqual(group.admin, groupAdmin.wallet.publicKey);
        if (verbose) {
            console.log("*init group: " + marginfiGroup.publicKey);
            console.log("group admin: " + group.admin);
        }

        const feeCache = group.feeStateCache;
        const tolerance = 0.00001;
        assertI80F48Approx(feeCache.programFeeFixed, PROGRAM_FEE_FIXED, tolerance);
        assertI80F48Approx(feeCache.programFeeRate, PROGRAM_FEE_RATE, tolerance);
        assertKeysEqual(feeCache.globalFeeWallet, globalFeeWallet);
    })
})