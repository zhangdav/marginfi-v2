#!/usr/bin/env sh

ROOT=$(git rev-parse --show-toplevel)
cd $ROOT

set -e

# Configuration
RPC_ENDPOINT=${RPC_ENDPOINT:-"https://api.devnet.solana.com"}
KEYPAIR_PATH=${KEYPAIR_PATH:-"$HOME/.config/solana/cli/test-wallet.json"}
PROGRAM_NAME="marginfi"

# Check if keypair exists
if [ ! -f "$KEYPAIR_PATH" ]; then
    echo "Error: Keypair not found at $KEYPAIR_PATH"
    echo "Please set KEYPAIR_PATH environment variable or ensure keypair exists"
    exit 1
fi

# Check balance (need at least 10 SOL for deployment)
BALANCE=$(solana balance $KEYPAIR_PATH --url $RPC_ENDPOINT)
BALANCE_NUM=$(echo $BALANCE | cut -d' ' -f1)
if [ $(echo "$BALANCE_NUM < 10" | bc -l) -eq 1 ]; then
    echo "Error: Insufficient balance ($BALANCE). Need at least 10 SOL."
    echo "Request devnet SOL at: https://faucet.solana.com/"
    exit 1
fi

# Build program for devnet
./scripts/build-program.sh $PROGRAM_NAME devnet

# Deploy program to devnet
anchor deploy \
    --provider.cluster $RPC_ENDPOINT \
    --provider.wallet $KEYPAIR_PATH \
    --program-name $PROGRAM_NAME

echo "Deployment completed. Program ID: $(solana-keygen pubkey target/deploy/marginfi-keypair.json)"
