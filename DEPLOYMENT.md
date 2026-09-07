# Deployment Record

## Conway Testnet

| Field | Value |
|---|---|
| Network | Linera Conway Testnet |
| SDK Version | linera-sdk 0.15.18 |
| Deployed | 2026-06-11 |
| Module ID | 9ad22a7cf489...5e300 |
| Application ID | a788ba8f89da75939e1b59b4bedcf8914132ba1ce7268dad3b85bafacd8b6a1c |
| Chain ID | 199a717ddd587bbf9cd786d32f7d4cdf6e23056ed256a142e07bfa378ba0227a |
| Owner | 0xd54394fafd1259181a0a68a04241c6405d54b4777b91f42f54f7a960c0843dec |

## Deploy Commands

```bash
# Build
cargo build --release --target wasm32-unknown-unknown

# Deploy
linera publish-and-create \
  target/wasm32-unknown-unknown/release/linera_price_market_contract.wasm \
  target/wasm32-unknown-unknown/release/linera_price_market_service.wasm \
  --json-parameters 'null'

# Start local node service
linera service --port 8080
```

## Transaction History

### 2026-06-11 — M1 e2e (BTC)
| Operation | Tx Hash |
|---|---|
| createRound | c40c8e81... |
| placeBet | ee137866... |
| resolveRound | dec9ddfd... |
| claim | 327abcbd... |

### 2026-06-11 — Week 1 e2e (ETH)
| Operation | Tx Hash |
|---|---|
| createRound | c8377ebd75850c0b7bc17ef3b65d508eb403a4146ef5b2930825375e907637e5 |
| placeBet | 347e371ec3534511aa54672d4d0ec0a622ea5de44175e2b52f054323dedf94a1 |
| resolveRound | 6cc2415fd851e9e3d1ab878c6027b87d52cab2ca73934e76ac3e64131d0c000e |
| claim | a8e4a330b39ae3d7a994da87255dbe8fdc501e0c0525f33694bd18be300993ed |

### 2026-06-11 — Week 2 e2e (SOL)
| Operation | Tx Hash |
|---|---|
| createRound | 61406d2ede34ccd3907b3b75ca63461a10d9e1ebb78a5fc03e22f16a2601c5d9 |
| placeBet | d3093f186fb0b1209a41fc2ca8c9af6f79c4bfec0f95c1fa175949b673d98c17 |
| resolveRound | 53db187439cc4aba5441bf3015f0bc0fc9865f4595266514f31e49ea8d7c87af |
| claim | e3523bbf1bca46caf9c155e9ad0a8423b9e9335079dc3d0718cfaa444581cc00 |

### 2026-06-18 — Week 4 e2e (BTC)
| Operation | Tx Hash |
|---|---|
| createRound | b37840e1869197f136d96a2ed9caba890262e34f5345ac35ba8590ab444d1107 |
| placeBet | 9be2dc0818bbd2911ec89bec39da93d01d75efa28f956e34aec4f3f5cb8b4624 |
| resolveRound | f51333bc4260b1527bbd6e2818880b7ce78dc9742ca13c1acc6ed4e68af90182 |
| claim | 93a042729511018bae606c02931888ff0ea48d52627a5e6b6910888bb77d7d42 |

### 2026-07-12 — Week 5 e2e (SOL)
| Operation | Tx Hash |
|---|---|
| createRound | 5a5867c2ffc2fd5c700717e07432e9316528fb07a5a28a6ba6b0638e39f5a825 |
| placeBet | 06f04f5aae201f1670ffe9cc1907bf2023cef730d1b96ad50db4c695eac03808 |
| resolveRound | 67c6962acf70f66bd8d3b4ad01d36d9bc1ab626e39417496300aa2a38eae7afb |
| claim | 03bbf12fc10c1c1242daace21f24c55b329c9def5d004978266504401881a1c5 |

### 2026-07-19 — Week 7 e2e (BTC)
| Operation | Tx Hash |
|---|---|
| createRound | 1ba1760c6e0a6a3766ce67841defc1b723da768d782854ef602f79a760a00c0d |
| placeBet | ec29973e7decde54dc74429f8b7b8746456c26b5d4f65df37be8503669ea6d36 |
| resolveRound | 9195e01d2b0582cc040e34add8cb9e3e382a3954e7d75ad7a9611123b44f084c |
| claim | 36998ef7425c17875b392abce02823a5c882f898f57b9168e1c701ab4e4a0d8a |

### 2026-07-31 — Week 8 e2e (ETH)
| Operation | Tx Hash |
|---|---|
| createRound | 72ee4f48bdb042968a77b0f9175df85ef4552382915ee2cb41db173c1ec031bb |
| placeBet | 44b3cd41e784ed60b3e1288802d5ae5db17f4df05805cc53334b92ad8dfce574 |
| resolveRound | 1776814a4d5a9af87de2fbe4af154663c9b5893acb1e2b9108e5b214ffe02090 |
| claim | cc4f4656778c6b55d0d6a362837f2170dfd1c83ad4e005bcb7db899ac5b5af09 |

### 2026-09-07 — Week 9 e2e (SOL)
| Operation | Tx Hash |
|---|---|
| createRound | d056179f7151ab5e90b80459bef8b87430488998495a8bd60e2e62c2ab1bd64d |
| placeBet | 1a49f1e274569071230f211ac246bfdb42cd082d4f6fef462e998d30ecfd2c0d |
| resolveRound | 2f6121111d59a2b634459246149955059bfa881ecc27a52ce592e7bdd2a765af |
| claim | f8a8576feca426ee99d728365a615384cb08c8c426d14b145431f9f7bdd10c29 |

Round 7 (SOL): startPrice 10514 ($105.14) → endPrice 10516 ($105.16), UP wins.

### 2026-09-07 — Week 10 e2e (BTC), round open — resolves 2026-09-21
| Operation | Tx Hash |
|---|---|
| createRound | 65ed9a808b54d76b0e7650855ed67404367e794fde0f6b27684caf262e63a6ff |

Round 8 (BTC): startPrice 7980400 ($79,804), durationSecs 1209600 (14 days), deadline 2026-09-21 14:08:38 CST. No bets placed yet; placeBet/resolveRound/claim rows to be added when the round settles.

## Validation Runs

Ad-hoc verification runs, not part of the biweekly maintenance rotation.

### 2026-09-07 — `scripts/multi_bet.sh` validation (round 9, BTC)
Verifies the contract allows a single caller to place multiple bets (UP then DOWN) in the same round, and that a single `claim` call correctly settles only the winning direction.

| Operation | Tx Hash |
|---|---|
| createRound | 8ee6f82ab4909c7545bbd9bb2e8948e0df9acf5b4b2fe9468e9d2777b242152d |
| placeBet (UP) | 009d082e2351d371e426fd2131e746e5af8d36fadf2e3e051741ff8c792602af |
| placeBet (DOWN) | e1554c9d9b2dee6d09360eb487ab2793e914e097c06b4f5f9f1b71fd3fff456e |
| resolveRound | eb836dee17e7461051064e925c7ee0f24adf254537a6a921bc373941dc8d5f95 |
| claim | 74416eba88e27a50012b462190143f0e75e4a1560f2a006130b60359c63633cd |

Round 9 (BTC): startPrice 6500000 ($65,000) → endPrice 6550000 ($65,500), UP wins. Single `claim` call settled both bets from the same owner, paying only the winning UP bet.
