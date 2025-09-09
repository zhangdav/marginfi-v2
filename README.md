# MarginFi Protocol

A decentralized lending and borrowing protocol built on Solana, inspired by the original MarginFi v2 architecture.

## Overview

This MarginFi implementation provides a complete lending protocol with the following features:

- **Lending Pools**: Users can deposit assets to earn interest
- **Borrowing**: Collateralized borrowing with health factor monitoring  
- **Liquidation**: Automated liquidation of unhealthy positions
- **Multi-Asset Support**: Support for multiple tokens (SOL, USDC, etc.)
- **Risk Management**: Deterministic risk engine with configurable parameters

## Architecture

The protocol consists of several key components:

- **MarginFi Group**: Top-level container managing risk and configuration
- **Banks**: Individual lending markets for each asset type
- **User Accounts**: Per-user accounts tracking balances and borrows
- **Risk Engine**: Real-time monitoring of account health

## Deployment

### Devnet Deployment

The protocol is currently deployed on Solana Devnet:

- **Program ID**: `5tZcX5B6QBaYVykWFCB4HzEiodfY4hy4WDYGE43Wo3G9`
- **Deployer**: `4Uer12PoW6XHmDXatf3Vz8Y57zE6taApRCJD31ZbgAAw`
- **Explorer**: [View on Solana Explorer](https://explorer.solana.com/address/5tZcX5B6QBaYVykWFCB4HzEiodfY4hy4WDYGE43Wo3G9?cluster=devnet)

### Test Tokens

The following devnet tokens are supported:

- **USDC**: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- **WSOL**: `So11111111111111111111111111111111111111112`

## Development

### Prerequisites

- Rust 1.70+
- Solana CLI 1.16+
- Anchor Framework 0.31.1
- Node.js 16+

### Setup

1. Clone the repository:
```bash
git clone <your-repo-url>
cd marginfi-v2
```

2. Install dependencies:
```bash
yarn install
```

3. Build the program:
```bash
anchor build
```

### Testing

#### Rust Tests
Run unit tests:
```bash
cargo test
```

#### Integration Tests
TypeScript integration tests:
```bash
anchor test
```

### Deployment

#### Deploy to Devnet
```bash
# Ensure you have devnet SOL (get from https://faucet.solana.com/)
./scripts/setup-devnet.sh
```

#### Environment Variables
You can customize deployment by setting environment variables:
```bash
export KEYPAIR_PATH="path/to/your/keypair.json"
export RPC_ENDPOINT="https://api.devnet.solana.com"
./scripts/deploy-devnet.sh
```

### Code Quality

Run linter:
```bash
./scripts/lint.sh
```

## Project Structure

```
├── programs/
│   ├── marginfi/           # Main protocol program
│   │   ├── src/
│   │   │   ├── instructions/   # Program instructions (marginfi_group, marginfi_account)
│   │   │   ├── state/         # Account state definitions
│   │   │   ├── constants.rs   # Protocol constants
│   │   │   ├── errors.rs      # Error definitions
│   │   │   └── lib.rs         # Program entry point
│   │   └── tests/             # Rust unit tests
│   └── mocks/              # Mock programs for testing
├── crates/
│   └── test_transfer_hook/ # Transfer hook testing utilities
├── tests/                  # TypeScript integration tests
├── test-utils/            # Shared testing utilities
├── scripts/               # Deployment and utility scripts
└── target/                # Build artifacts and IDL files
```

## Key Features

### Risk Management
- Asset and liability weights for LTV calculations
- Real-time health factor monitoring
- Configurable risk parameters per bank
- Isolated asset support to prevent contagion

### Lending & Borrowing
- Interest rate models with utilization-based rates
- Emission rewards for liquidity providers
- Flash loan support
- Multi-collateral borrowing (up to 16 assets)

### Administration
- Multi-role permission system (admin, curve admin, limits admin)
- Configurable fees and parameters
- Oracle integration (Pyth, Switchboard)
