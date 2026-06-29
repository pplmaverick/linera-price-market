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
