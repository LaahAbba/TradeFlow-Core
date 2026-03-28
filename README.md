# TradeFlow-Core: Decentralized Trade Finance on Soroban

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Soroban](https://img.shields.io/badge/soroban-ready-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

**TradeFlow-Core** is the smart contract layer for the TradeFlow protocol. It enables Real-World Asset (RWA) tokenization and decentralized factoring on the Stellar network.

## 🏗 Architecture

The system consists of multiple smart contracts:

1.  **`invoice_nft`**: A standard-compliant NFT representing a verified invoice. It holds metadata (IPFS hash, face value, currency, due date).
2.  **`lending_pool`**: An escrow vault where liquidity providers deposit stablecoins (USDC). It accepts `invoice_nft` as collateral to automate loan origination and repayment.
3.  **`factory`**: Factory contract for deploying liquidity pools with specific fee tiers.
4.  **`amm_pool`**: Automated Market Maker pool contract with configurable fee tiers.

## 💰 Fee Tiers

The Factory contract supports creating pools with different fee tiers to optimize for various token pair characteristics:

| Fee Tier | Basis Points | Percentage | Use Case |
| :--- | :--- | :--- | :--- |
| **Stable** | 5 | 0.05% | Stablecoin pairs (USDC/USDT, DAI/USDC) |
| **Standard** | 30 | 0.30% | Standard token pairs (ETH/USDC, BTC/USDC) |
| **Volatile** | 100 | 1.00% | Highly volatile exotic pairs |

### Creating a Pool with Specific Fee Tier

```rust
// Create a stablecoin pool with 0.05% fee
let pool_address = factory.create_pool(
    token_a, 
    token_b, 
    5  // 5 basis points = 0.05%
);

// Create a standard pool with 0.30% fee
let pool_address = factory.create_pool(
    token_a, 
    token_b, 
    30  // 30 basis points = 0.30%
);

// Create a volatile pool with 1.00% fee
let pool_address = factory.create_pool(
    token_a, 
    token_b, 
    100  // 100 basis points = 1.00%
);
```

**Important:** Only fee tiers of 5, 30, or 100 basis points are supported. Any other value will cause the transaction to fail.

## 💾 Storage Architecture

To optimize for ledger rent costs and scalability, the protocol uses a tiered storage approach:

- **Instance Storage**: Global configuration settings (Admin, Paused state) and counters (Loan IDs).
- **Persistent Storage**: High-cardinality user data (Loans, Invoices, Whitelists).
- **Temporary Storage**: Used for transient data where applicable.

## ⛓️ Live Testnet Deployments

The following contracts are currently active for frontend integration and testing.

| Contract Name | Network | Contract ID |
| :--- | :--- | :--- |
| **Invoice NFT** | Testnet | `CCYU3LOQI34VHVN3ZOSEBHHKL4YK36FMTOEGLRYDUDRGS7JOLLRKCEQM` |
| **Lending Pool** | Testnet | `CDVJMVPLZJKXSJFDY5AWBOUIRN73BKU2SG674MQDH4GRE6BGBPQD33IQ` |

- **Network Passphrase:** `Test SDF Network ; September 2015`
- **RPC Endpoint:** `https://soroban-testnet.stellar.org`

## 🚀 Quick Start

### Prerequisites
- Rust & Cargo (latest stable)
- Stellar CLI v25.1.0+ (`cargo install stellar-cli`)
- WASM Target: `rustup target add wasm32v1-none`

### Build & Test
```bash
# Build all contracts (optimized for WASM)
stellar contract build

# Run the test suite
cargo test