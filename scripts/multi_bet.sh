#!/bin/bash
# multi_bet.sh — 模擬多筆下注：同一地址依序下注 UP 與 DOWN，結算後 claim
#
# 目前 wallet 只有一組可簽章的 AccountOwner（見 `linera wallet show`：
# ADMIN chain 為 "No owner key"），無法用兩個獨立地址各自下注。
# 因此本腳本改為示範「同一地址在同一 round 依序累積兩筆下注」（UP 後 DOWN），
# 驗證合約允許單一 caller 多次 place_bet，並在結算後以一次 claim
# 依獲勝方向加總結算金額。
#
# Usage: ./scripts/multi_bet.sh
#
# Requires: linera service running on port 8080 (see DEPLOYMENT.md)
# App ID / Chain ID below match DEPLOYMENT.md's Conway Testnet record.

set -e

APP_ID="a788ba8f89da75939e1b59b4bedcf8914132ba1ce7268dad3b85bafacd8b6a1c"
CHAIN_ID="199a717ddd587bbf9cd786d32f7d4cdf6e23056ed256a142e07bfa378ba0227a"
ENDPOINT="http://localhost:8080"
URL="$ENDPOINT/chains/$CHAIN_ID/applications/$APP_ID"

echo "=== Step 1: Create BTC round ==="
# createRound 只會把 operation 排入下一區塊執行，回傳固定 []，不含 round_id。
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d '{"query": "mutation { createRound(asset: BTC, durationSecs: 300, startPrice: 6500000) }"}'
echo ""

echo "=== Step 2: Look up the new round's id ==="
ROUNDS_RESP=$(curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d '{"query": "query { rounds { id } }"}')
echo "$ROUNDS_RESP"
ROUND_ID=$(echo "$ROUNDS_RESP" | python3 -c "import sys,json; rs=json.load(sys.stdin)['data']['rounds']; print(max(r['id'] for r in rs))")
echo "Round ID: $ROUND_ID"

echo ""
echo "=== Step 3: Place UP bet (amount 1) ==="
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"mutation { placeBet(roundId: $ROUND_ID, direction: UP, amount: \\\"1\\\") }\"}"

echo ""
echo "=== Step 4: Place DOWN bet (amount 1, same address) ==="
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"mutation { placeBet(roundId: $ROUND_ID, direction: DOWN, amount: \\\"1\\\") }\"}"

echo ""
echo "=== Step 5: Resolve round — UP wins ==="
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"mutation { resolveRound(roundId: $ROUND_ID, finalPrice: 6550000) }\"}"

echo ""
echo "=== Step 6: Claim (pays out the winning UP bet only) ==="
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"mutation { claim(roundId: $ROUND_ID) }\"}"

echo ""
echo "=== Step 7: Query round result ==="
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"query { round(id: $ROUND_ID) { asset status startPrice endPrice bets { direction amount } claimed } }\"}"

echo ""
echo "Done. Round $ROUND_ID complete."
