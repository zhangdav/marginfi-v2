import { Program, workspace } from "@coral-xyz/anchor";
import { Marginfi } from "../target/types/marginfi";
import { Transaction } from "@solana/web3.js";
import { ASSET_TAG_DEFAULT, CLOSE_ENABLED_FLAG, defaultBankConfig, ORACLE_SETUP_PYTH_PUSH, PYTH_PULL_MIGRATED } from "./utils/types";
import { bankKeypairUsdc, groupAdmin, marginfiGroup, ecosystem, oracles, INIT_POOL_ORIGINATION_FEE, verbose, globalFeeWallet, printBuffers } from "./rootHooks";
import { addBank, configureBankOracle } from "./utils/group-instructions";
import { assert } from "chai";
import { printBufferGroups } from "./utils/tools";
import { assertKeysEqual, assertKeyDefault, assertI80F48Approx, assertI80F48Equal, assertBNEqual } from "./utils/genericTests";
import { deriveLiquidityVault, deriveLiquidityVaultAuthority, deriveInsuranceVault, deriveInsuranceVaultAuthority, deriveFeeVault, deriveFeeVaultAuthority } from "./utils/pdas";

describe("Lending pool add bank (add bank to group)", () => {
    const program = workspace.Marginfi as Program<Marginfi>;

    it("(admin) Add bank (USDC) - happy path", async () => {
        let setConfig = defaultBankConfig();
        let bankKey = bankKeypairUsdc.publicKey;
    
        const now = Date.now() / 1000;
    
        const feeAccSolBefore = await program.provider.connection.getBalance(
          globalFeeWallet
        );
    
        await groupAdmin.mrgnProgram.provider.sendAndConfirm(
          new Transaction().add(
            await addBank(groupAdmin.mrgnProgram, {
              marginfiGroup: marginfiGroup.publicKey,
              feePayer: groupAdmin.wallet.publicKey,
              bankMint: ecosystem.usdcMint.publicKey,
              bank: bankKey,
              // globalFeeWallet: globalFeeWallet,
              config: setConfig,
            })
          ),
          [bankKeypairUsdc]
        );
    
        await groupAdmin.mrgnProgram.provider.sendAndConfirm(
          new Transaction().add(
            await configureBankOracle(groupAdmin.mrgnProgram, {
              bank: bankKey,
              type: ORACLE_SETUP_PYTH_PUSH,
              oracle: oracles.usdcOracle.publicKey,
            })
          )
        );
    
        const feeAccSolAfter = await program.provider.connection.getBalance(
          globalFeeWallet
        );
    
        if (verbose) {
          console.log("*init USDC bank " + bankKey);
          console.log(
            " Origination fee collected: " + (feeAccSolAfter - feeAccSolBefore)
          );
        }
    
        assert.equal(feeAccSolAfter - feeAccSolBefore, INIT_POOL_ORIGINATION_FEE);

        let bankData = (
            await program.provider.connection.getAccountInfo(bankKey)
        ).data.subarray(8);
        if (printBuffers) {
            printBufferGroups(bankData, 16, 896);
        }

        const bank = await program.account.bank.fetch(bankKey);
        const config = bank.config;
        const interest = config.interestRateConfig;
        const id = program.programId;

        assertKeysEqual(bank.mint, ecosystem.usdcMint.publicKey);
        assert.equal(bank.mintDecimals, ecosystem.usdcDecimals);
        assertKeysEqual(bank.group, marginfiGroup.publicKey);

        // Keys and bumps...
        assertKeysEqual(config.oracleKeys[0], oracles.usdcOracle.publicKey);

        const [_liqAuth, liqAuthBump] = deriveLiquidityVaultAuthority(id, bankKey);
        const [liquidityVault, liqVaultBump] = deriveLiquidityVault(id, bankKey);
        assertKeysEqual(bank.liquidityVault, liquidityVault);
        assert.equal(bank.liquidityVaultBump, liqVaultBump);
        assert.equal(bank.liquidityVaultAuthorityBump, liqAuthBump);

        const [_insAuth, insAuthBump] = deriveInsuranceVaultAuthority(id, bankKey);
        const [insuranceVault, insVaultBump] = deriveInsuranceVault(id, bankKey);
        assertKeysEqual(bank.insuranceVault, insuranceVault);
        assert.equal(bank.insuranceVaultBump, insVaultBump);
        assert.equal(bank.insuranceVaultAuthorityBump, insAuthBump);

        const [_feeVaultAuth, feeAuthBump] = deriveFeeVaultAuthority(id, bankKey);
        const [feeVault, feeVaultBump] = deriveFeeVault(id, bankKey);
        assertKeysEqual(bank.feeVault, feeVault);
        assert.equal(bank.feeVaultBump, feeVaultBump);
        assert.equal(bank.feeVaultAuthorityBump, feeAuthBump);

        assertKeyDefault(bank.emissionsMint);

        assertI80F48Equal(bank.assetShareValue, 1);
        assertI80F48Equal(bank.liabilityShareValue, 1);

        assertI80F48Approx(bank.collectedInsuranceFeesOutstanding, 0);
        assertI80F48Approx(bank.collectedGroupFeesOutstanding, 0);

        assertI80F48Equal(bank.totalAssetShares, 0);
        assertI80F48Equal(bank.totalLiabilityShares, 0);

        assertBNEqual(bank.flags, CLOSE_ENABLED_FLAG);
        assertBNEqual(bank.emissionsRate, 0);
        assertI80F48Equal(bank.emissionsRemaining, 0);

        let lastUpdate = bank.lastUpdate.toNumber();
        assert.approximately(now, lastUpdate, 2);
        assertI80F48Equal(config.assetWeightInit, 1);
        assertI80F48Equal(config.assetWeightMaint, 1);
        assertI80F48Equal(config.liabilityWeightInit, 1);
        assertI80F48Equal(config.liabilityWeightMaint, 1);
        assertBNEqual(config.depositLimit, 100_000_000_000);

        const tolerance = 0.000001;
        assertI80F48Approx(interest.optimalUtilizationRate, 0.5, tolerance);
        assertI80F48Approx(interest.plateauInterestRate, 0.6, tolerance);
        assertI80F48Approx(interest.maxInterestRate, 3, tolerance);

        assertI80F48Approx(interest.insuranceFeeFixedApr, 0.01, tolerance);
        assertI80F48Approx(interest.insuranceIrFee, 0.02, tolerance);
        assertI80F48Approx(interest.protocolFixedFeeApr, 0.03, tolerance);
        assertI80F48Approx(interest.protocolIrFee, 0.04, tolerance);
        assertI80F48Approx(interest.protocolOriginationFee, 0.01, tolerance);

        assert.deepEqual(config.operationalState, { operational: {} });
        assert.deepEqual(config.oracleSetup, { pythPushOracle: {} });
        assertBNEqual(config.borrowLimit, 100_000_000_000);
        assert.deepEqual(config.riskTier, { collateral: {} });
        assert.equal(config.assetTag, ASSET_TAG_DEFAULT);
        assert.equal(config.configFlags, PYTH_PULL_MIGRATED);
        assertBNEqual(config.totalAssetValueInitLimit, 1_000_000_000_000);
        assert.equal(config.oracleMaxAge, 240);
    })
})