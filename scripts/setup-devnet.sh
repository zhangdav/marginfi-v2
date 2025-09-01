#!/usr/bin/env bash

ROOT=$(git rev-parse --show-toplevel)
cd $ROOT

set -e

# Configuration
RPC_ENDPOINT=${RPC_ENDPOINT:-"https://api.devnet.solana.com"}
KEYPAIR_PATH=${KEYPAIR_PATH:-"$HOME/.config/solana/cli/test-wallet.json"}
PROGRAM_NAME="marginfi"

# Deploy the program to devnet
./scripts/deploy-devnet.sh

# Get deployed program ID
PROGRAM_ID=$(solana-keygen pubkey target/deploy/marginfi-keypair.json)

# Define test tokens
DEVNET_USDC="4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
DEVNET_WSOL="So11111111111111111111111111111111111111112"

echo "Setup completed:"
echo "  Program ID: $PROGRAM_ID"
echo "  USDC Token: $DEVNET_USDC"
echo "  WSOL Token: $DEVNET_WSOL"
echo ""
echo "Next: Run TypeScript tests to initialize groups and banks"
