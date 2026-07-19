# Architecture

This document describes the current on-chain design of `linera-price-market`
and the planned cross-chain message flow that will replace it in a future
milestone (M3, see [Roadmap](README.md#roadmap)).

> **Status note:** As of this document, the deployed contract is a
> **single-chain design**. `execute_message` is unimplemented and panics if
> called — the contract accepts no cross-chain messages today. The
> "Future Design" section below is a technical proposal, not a description
> of shipped behavior.

---

## 1. Current Design (Implemented)

### 1.1 Execution model

All four operations run synchronously on one application chain:

```
                    Application Chain
┌───────────────────────────────────────────────────┐
│  PriceMarketContract                               │
│                                                     │
│  state: PriceMarket (RootView)                     │
│  ├── rounds:        MapView<u64, Round>            │
│  ├── round_counter: RegisterView<u64>              │
│  └── oracle_owner:  RegisterView<Option<AccountOwner>>│
│                                                     │
│  execute_operation(Operation) -> ()                │
│  ├── CreateRound  { asset, duration_secs,          │
│  │                  start_price }                  │
│  ├── PlaceBet     { round_id, direction, amount }   │
│  ├── ResolveRound { round_id, final_price }         │
│  └── Claim        { round_id }                      │
│                                                     │
│  execute_message(()) -> panic!()                   │
│  (Message = (); no cross-chain messages accepted)  │
└───────────────────────────────────────────────────┘
                        │
                        │ GraphQL queries (read-only)
                        ▼
              Service Layer (off-chain)
              ├── round(id), rounds(), price(asset)
              └── CoinGecko HTTP oracle
                  (gated by committee http_request_allow_list)
```

Every caller — bettor or oracle bot — submits a `linera_sdk::Operation`
directly to this one chain. There is no message routing: `type Message = ()`
in `Contract for PriceMarketContract`, and the `execute_message` handler
exists only to `panic!("PriceMarket does not support cross-chain messages")`
if the runtime ever tries to deliver one.

### 1.2 State layout

| Field | Type | Purpose |
|---|---|---|
| `rounds` | `MapView<u64, Round>` | One entry per prediction round, keyed by sequential `round_id` |
| `round_counter` | `RegisterView<u64>` | Next `round_id` to assign |
| `oracle_owner` | `RegisterView<Option<AccountOwner>>` | The single account authorized to call `ResolveRound`; set once in `instantiate` from `application_parameters()` |

`Round` itself holds `asset`, `start_price`, `end_price`, `status`
(`Open`/`Settled`), the full `bets: Vec<Bet>` ledger, a `deadline: Timestamp`,
and `claimed: Vec<AccountOwner>` for duplicate-claim prevention.

### 1.3 Per-operation flow

**CreateRound** — asserts `duration_secs > 0`, computes `deadline = now +
duration_secs`, inserts a fresh `Round` at `round_counter`, increments the
counter.

**PlaceBet** — requires an `authenticated_signer`; asserts the round is
`Open`, the deadline hasn't passed, and `amount > 0`; appends a `Bet` to
`round.bets`. Token custody is a stub — no funds actually move (see §1.4).

**ResolveRound** — restricted to `oracle_owner` via
`assert_eq!(authenticated_signer(), oracle_owner)`; requires the deadline to
have passed; writes `end_price` and flips status to `Settled`.

**Claim** — requires an authenticated signer who hasn't claimed yet;
determines the winning direction by comparing `end_price` to `start_price`
(tie → both sides refunded); computes a proportional payout using a
overflow-safe quotient/remainder split (`quotient * caller_winning +
remainder * caller_winning / total_winning`, each step through
`checked_mul`/`checked_add`) so a payout is guaranteed to either compute
correctly or panic explicitly rather than wrap silently.

### 1.4 Known stubs

- **Token transfer in `Claim`** — the payout amount (`payout_attos`) is
  computed correctly but never disbursed. See the `STUB` comment in the
  `claim` handler in [`src/contract.rs`](src/contract.rs). This is not
  currently tracked as a GitHub issue.
- **Bet custody in `PlaceBet`** — bets are ledger entries only; no token
  deposit occurs.

Both stubs exist because Linera's fungible-token integration is planned for
M3 alongside cross-chain messaging (§2).

### 1.5 Why this still uses Linera-native primitives

Even without cross-chain messages, the design isn't a generic EVM port:

| Concern | Generic EVM pattern | This contract |
|---|---|---|
| Multi-asset concurrency | One shared contract, mutex-like state accounting | Independent `Round` entries in a `MapView`; BTC/ETH/SOL rounds never contend for the same storage slot |
| Historical queries | Off-chain indexer (e.g. The Graph) | Native GraphQL exposed directly by the SDK's service layer — no indexer |
| Oracle input | Often a pull-based on-chain oracle contract | External bot passes `start_price`/`final_price` as operation arguments, sidestepping the committee's `http_request_allow_list` restriction on `api.coingecko.com` |

> **Correction:** an earlier draft of this document (and README's "Core
> Features" section) claimed bet placement and settlement are logged via an
> `emit_event!` macro. That is not accurate. `linera-sdk` 0.15.18 has no
> `emit_event!` macro — the real API is the `ContractRuntime::emit(name,
> value)` method. Neither `emit_event!` nor `runtime.emit()` is called
> anywhere in `src/contract.rs`; `type EventValue = ()` is an unused
> placeholder. **No on-chain event log exists in the current
> implementation.**

---

## 2. Future Design — Cross-Chain Message Flow (Not Yet Implemented)

This section is a **design proposal** for M3. Nothing described here exists
in the current contract; it's included so the intended microchain
architecture is documented ahead of implementation.

### 2.1 Motivation

Linera's core primitive is that every user can own a single-owner
**microchain** and interact with shared application chains asynchronously,
without contending for the same chain's execution slot. The current
single-chain design doesn't use this: every bettor sends an operation
directly to the market's chain, which means the market chain — not the
user — pays for and serializes every bet. The cross-chain design would move
bet submission and payout delivery onto the user's own chain, which is the
pattern Linera's documentation describes for scaling user-facing dApps.

### 2.2 Proposed topology

```
   User Microchain A (single-owner)        User Microchain B (single-owner)
   ┌───────────────────────────┐           ┌───────────────────────────┐
   │ local proxy operation:    │           │ local proxy operation:    │
   │ PlaceBet(round_id, dir,   │           │ PlaceBet(round_id, dir,   │
   │          amount)          │           │          amount)          │
   └──────────────┬────────────┘           └──────────────┬────────────┘
                  │ Message::PlaceBet                     │ Message::PlaceBet
                  │ (cross-chain, via                      │
                  │  ChainId::send_message)                │
                  ▼                                        ▼
              ┌────────────────────────────────────────────────┐
              │           Market Chain (multi-owner)            │
              │  execute_message(Message::PlaceBet) handles     │
              │  the same validation PlaceBet does today        │
              │  (Open status, deadline, amount > 0), then      │
              │  records the bet keyed by (round_id, sender     │
              │  chain, owner)                                  │
              └───────────────────────┬──────────────────────────┘
                                      │ Message::Payout(amount)
                                      │ sent back to the
                                      │ originating chain
                    ┌─────────────────┴─────────────────┐
                    ▼                                    ▼
          User Microchain A                    User Microchain B
          receives Payout message,              receives Payout message,
          credits local balance                 credits local balance
```

### 2.3 What would change in the contract

- `type Message = ()` → a real enum, e.g.
  `enum Message { PlaceBet { round_id, direction, amount }, Payout { amount } }`.
- `execute_message` would replace its current `panic!` with handlers mirroring
  today's `place_bet`/payout logic, but keyed by `(round_id, message_id.chain_id, owner)`
  instead of assuming the caller shares the market chain's execution context.
- `Claim` would no longer write `claimed` on the market chain in isolation —
  it would trigger `self.runtime.send_message(origin_chain_id,
  Message::Payout { amount })` so winnings land back on the bettor's own
  chain rather than sitting on the market chain waiting to be pulled.
- Token custody (currently stubbed, §1.4) would piggyback on this change:
  a real deposit would move funds from the user chain into an
  application-chain-owned balance when `PlaceBet` is proxied over, and the
  `Payout` message would carry actual token transfer, not just a ledger
  update.

### 2.4 Open questions before implementation

- **Ordering guarantees** — Linera cross-chain messages are asynchronous;
  `ResolveRound` needs to be certain all `PlaceBet` messages for a round
  have been received and applied before it locks in `end_price`. This likely
  requires a message-count or explicit "round sealed" step rather than
  relying on `deadline` alone.
- **Multi-owner market chain** — accepting messages from many user chains
  concurrently suggests the market chain should be multi-owner, which has
  different validator/latency characteristics than the current single-owner
  setup used for the App ID `a788ba8f89da75939e1b59b4bedcf8914132ba1ce7268dad3b85bafacd8b6a1c`.
  This will require a fresh deployment, not an in-place migration.
- **Fungible token dependency** — real payout delivery depends on Linera's
  native fungible-token application being integrated, which is why this is
  scoped as one M3 item rather than split further.

---

## References

- Current contract source: [`src/contract.rs`](src/contract.rs)
- State layout: [`src/state.rs`](src/state.rs)
- Roadmap: [`README.md`](README.md#roadmap)
