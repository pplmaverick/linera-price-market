# Linera Price Market

![Network](https://img.shields.io/badge/Linera-Conway_Testnet-blue)
![Rust](https://img.shields.io/badge/Rust-1.86.0-orange)
![License](https://img.shields.io/badge/license-MIT-green)

BTC/ETH/SOL short-term price direction prediction market built natively on Linera microchains. Purpose-built for Linera's architecture — not a port from any EVM chain.

**Deployed on Linera Conway Testnet**

| Field | Value |
|---|---|
| Network | Linera Conway Testnet |
| Application ID | `a788ba8f89da75939e1b59b4bedcf8914132ba1ce7268dad3b85bafacd8b6a1c` |
| Chain ID | `199a717ddd587bbf9cd786d32f7d4cdf6e23056ed256a142e07bfa378ba0227a` |
| SDK Version | linera-sdk 0.15.18 |

---

## Why Linera-Native

This project is not ported from EVM. Every design decision maps directly to a Linera primitive that has no equivalent on traditional blockchains.

| Problem | Generic EVM approach | Linera-native approach |
|---|---|---|
| Multi-asset concurrent rounds | Separate contracts or mutex locks | Independent microchain state per asset — no contention |
| User participation | EOA sends tx to shared contract | User's single-owner microchain sends cross-chain message to market chain |
| Winnings distribution | Pull-based withdraw from shared pool | Cross-chain message routes payout back to user's own chain |
| Historical query | Off-chain indexer (The Graph) | GraphQL exposed natively from service layer — no indexer needed |

---

## Architecture

```
  User Microchain (single-owner)
         │
         │  cross-chain message: PlaceBet
         ▼
  Market Microchain (multi-owner)
  ┌──────────────────────────────────┐
  │  PriceMarket Contract            │
  │  ├── rounds: MapView<u64, Round> │
  │  └── round_counter: u64          │
  │                                  │
  │  Operations:                     │
  │  ├── CreateRound(asset, secs,    │
  │  │               start_price)    │
  │  ├── PlaceBet(round_id, dir,     │
  │  │            amount)            │
  │  ├── ResolveRound(id, price)     │
  │  └── Claim(round_id)             │
  └──────────────────────────────────┘
         │
         │  cross-chain message: payout
         ▼
  User Microchain

  Service Layer (read-only, off-chain)
  ├── GraphQL: round(id), rounds(), price(asset)
  └── CoinGecko HTTP oracle (requires committee allow-list on testnet)
```

---

## Core Features

### 1. Multi-Asset Concurrent Rounds
BTC, ETH, and SOL rounds run simultaneously with independent state. Each asset's rounds do not interfere with one another — a property that falls naturally out of Linera's microchain architecture rather than requiring any locking mechanism.

### 2. Cross-Chain Message Betting Flow
Users interact from their own single-owner microchain. Bets are submitted as asynchronous cross-chain messages to the market's multi-owner chain. Winnings are routed back via the same mechanism. This is the core Linera primitive that EVM cannot replicate natively.

### 3. GraphQL Leaderboard via Service Layer
The service layer exposes a native GraphQL interface for querying round state, bet history, and real-time prices. No external indexer required — this is built into the Linera SDK's service architecture.

### 4. Experimental Events (SDK v0.15)
Every bet placement and round settlement emits an on-chain event using the `emit_event!` macro introduced in linera-sdk 0.15. This provides a complete, queryable history of all market activity — the Linera equivalent of Solidity's `emit Event()`.

---

## Deployed Application

**Linera Conway Testnet**

| Field | Value |
|---|---|
| Application ID | `a788ba8f89da75939e1b59b4bedcf8914132ba1ce7268dad3b85bafacd8b6a1c` |
| Module ID | `9ad22a7cf489...5e300` |
| Owner | `0xd54394fafd1259181a0a68a04241c6405d54b4777b91f42f54f7a960c0843dec` |

---

## Quick Start

**Prerequisites**
- Rust 1.86.0
- `wasm32-unknown-unknown` target
- `protoc` (libprotoc 29+)
- Linera CLI v0.15.18 (built from `testnet_conway` branch)

```bash
# 1. Clone and enter the project
git clone https://github.com/pplmaverick/linera-price-market.git
cd linera-price-market

# 2. Build
cargo build --release --target wasm32-unknown-unknown

# 3. Initialize Conway testnet wallet
linera wallet init --faucet https://faucet.testnet-conway.linera.net
linera wallet request-chain --faucet https://faucet.testnet-conway.linera.net

# 4. Deploy
linera publish-and-create \
  target/wasm32-unknown-unknown/release/linera_price_market_contract.wasm \
  target/wasm32-unknown-unknown/release/linera_price_market_service.wasm \
  --json-parameters 'null'

# 5. Start local node service
linera service --port 8080
```

**Environment**

| Variable | Description |
|---|---|
| `LINERA_WALLET` | Path to wallet file (default: `~/Library/Application Support/linera`) |
| `LINERA_STORAGE` | Storage backend URL |

---

## Contract Interface

```rust
// Create a new prediction round
CreateRound { asset: Asset, duration_secs: u64, start_price: u64 }

// Place a directional bet
PlaceBet { round_id: u64, direction: Direction, amount: Amount }

// Settle the round with final price (called by oracle bot)
ResolveRound { round_id: u64, final_price: u64 }

// Claim winnings after settlement
Claim { round_id: u64 }
```

**Asset enum**: `BTC` | `ETH` | `SOL`

**Direction enum**: `UP` | `DOWN`

**Price encoding**: USD × 100 as `u64` (e.g. $95,000.00 → `9500000`)

---

## Settlement Logic

Prices are stored as `u64` with two decimal places of USD precision:

```
$95,000.00  →  9500000
$95,123.45  →  9512345
```

Resolution:

| Condition | Result |
|---|---|
| `final_price > start_price` | UP wins |
| `final_price < start_price` | DOWN wins |
| `final_price == start_price` | All bets refunded |

Winners receive a proportional share of the total pot based on their bet size relative to all winning bets.

---

## Fees & Security

**Fees**
- No protocol fee in current implementation
- No winner: all bets refunded in full

**Security**
- Duplicate claim prevented via `claimed: Vec<AccountOwner>` per round
- Bet rejected after round deadline
- Bet rejected if round is not `Open`
- `ResolveRound` can only transition `Open` → `Settled`

---

## Implementation Notes

**HTTP Oracle Requires Committee Allow-List**
Linera's execution engine enforces an `http_request_allow_list` at the validator/committee level. On Conway testnet, `api.coingecko.com` is not on the allow-list, so the `price(asset)` GraphQL query returns `UnauthorizedHttpRequest` when called against the live testnet. The core contract logic is unaffected — `start_price` and `final_price` are passed in by the caller (external oracle bot pattern), which is the correct production architecture regardless.

**Linera CLI Must Be Built from Source**
`cargo install linera-service` from crates.io does not work for Conway testnet. The CLI must be compiled from the `testnet_conway` branch of `linera-io/linera-protocol`. Version mismatch causes silent connection failures.

**Rust Version Pinned via `rust-toolchain.toml`**
Conway testnet requires exactly Rust 1.86.0. The project includes `rust-toolchain.toml` so the correct toolchain activates automatically per-directory without affecting the global default.

**Dependency Version Pinning**
Several transitive dependencies (`serde_with`, `tonic`, `async-graphql`, `allocative`) have releases requiring Rust 1.88+. All are pinned in `Cargo.lock` to versions compatible with 1.86.0, matched against the reference `Cargo.lock` from `linera-protocol-src`.

---

## Stack

| Layer | Technology |
|---|---|
| Smart contract | Rust 1.86.0 → WebAssembly |
| SDK | linera-sdk 0.15.18 |
| Development | cargo + wasm32-unknown-unknown target |
| Oracle | External bot pattern (HTTP oracle gated by testnet committee) |
| Query layer | GraphQL via linera-sdk service layer |
| Testnet | Linera Conway Testnet (faucet: 100 LINERA) |

---

## Roadmap

**✅ M1 — Conway Testnet Deployment (completed)**
- Environment setup: Rust 1.86.0, wasm32 target, protoc, Linera CLI v0.15.18
- Conway testnet wallet initialized, 100 LINERA funded
- Contract deployed: CreateRound / PlaceBet / ResolveRound / Claim
- Full e2e flow confirmed on-chain (tx hashes recorded)
- Four chain-native features: multi-asset, cross-chain messaging, GraphQL leaderboard, experimental events
- Unit tests: 5 cases covering all operations and edge cases

**⬜ M2 — Mainnet**
- Redeploy on Linera mainnet once launched
- Update Application ID and owner chain references
- Validate HTTP oracle allow-list status on mainnet committee

---

## Developer

GitHub: [pplmaverick](https://github.com/pplmaverick)  
Wallet: `0xed2B5717c9b936ecC76d75401026A99143e278F5`

## License

MIT
