#![cfg_attr(target_arch = "wasm32", no_main)]

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Request, Response, Schema, SimpleObject};
use linera_sdk::{
    ensure, http,
    graphql::GraphQLMutationRoot,
    linera_base_types::{Amount, WithServiceAbi},
    views::View,
    Service, ServiceRuntime,
};
use linera_price_market::Operation;

use linera_price_market::{
    state::{Asset, Bet, Direction, PriceMarket, Round, RoundStatus},
    PriceMarketAbi,
};

// ── DTO 型別（避免直接暴露含外部型別的 state struct）──────────────────────

#[derive(SimpleObject, Clone)]
pub struct BetInfo {
    pub owner: String,
    pub direction: Direction,
    pub amount: Amount,
}

#[derive(SimpleObject, Clone)]
pub struct RoundInfo {
    pub id: u64,
    pub asset: Asset,
    pub start_price: u64,
    pub end_price: u64,
    pub status: RoundStatus,
    pub bets: Vec<BetInfo>,
    pub deadline_micros: u64,
    pub claimed: Vec<String>,
}

fn to_bet_info(bet: &Bet) -> BetInfo {
    BetInfo {
        owner: bet.owner.to_string(),
        direction: bet.direction,
        amount: bet.amount,
    }
}

fn to_round_info(id: u64, round: Round) -> RoundInfo {
    RoundInfo {
        id,
        asset: round.asset,
        start_price: round.start_price,
        end_price: round.end_price,
        status: round.status,
        bets: round.bets.iter().map(to_bet_info).collect(),
        deadline_micros: round.deadline.micros(),
        claimed: round.claimed.iter().map(|o| o.to_string()).collect(),
    }
}

// ── Service 主體 ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct PriceMarketService {
    state: Arc<PriceMarket>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(PriceMarketService);

impl WithServiceAbi for PriceMarketService {
    type Abi = PriceMarketAbi;
}

impl Service for PriceMarketService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = PriceMarket::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        PriceMarketService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryRoot { service: self.clone() },
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}

// ── GraphQL Query 根節點 ───────────────────────────────────────────────────

struct QueryRoot {
    service: PriceMarketService,
}

#[Object]
impl QueryRoot {
    /// 查詢單一 round 詳情
    async fn round(&self, id: u64) -> async_graphql::Result<Option<RoundInfo>> {
        let maybe = self
            .service
            .state
            .rounds
            .get(&id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(maybe.map(|r| to_round_info(id, r)))
    }

    /// 查詢所有 rounds
    async fn rounds(&self) -> async_graphql::Result<Vec<RoundInfo>> {
        let mut result = Vec::new();
        self.service
            .state
            .rounds
            .for_each_index_value(|id, round| {
                result.push(to_round_info(id, round.into_owned()));
                Ok(())
            })
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(result)
    }

    /// 呼叫 CoinGecko 取即時價格，回傳 USD × 100 精度的 u64
    async fn price(&self, asset: Asset) -> async_graphql::Result<u64> {
        self.service.fetch_price(asset)
    }
}

// ── HTTP Oracle ────────────────────────────────────────────────────────────

impl PriceMarketService {
    fn fetch_price(&self, asset: Asset) -> async_graphql::Result<u64> {
        const URL: &str = "https://api.coingecko.com/api/v3/simple/price\
            ?ids=bitcoin,ethereum,solana&vs_currencies=usd";

        let response = self.runtime.http_request(http::Request::get(URL));

        ensure!(
            response.status == 200,
            async_graphql::Error::new(format!(
                "CoinGecko request failed with status {}",
                response.status
            ))
        );

        let json: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| async_graphql::Error::new(format!("JSON parse error: {e}")))?;

        let coin_id = match asset {
            Asset::Btc => "bitcoin",
            Asset::Eth => "ethereum",
            Asset::Sol => "solana",
        };

        let price_f64 = json
            .get(coin_id)
            .and_then(|c| c.get("usd"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                async_graphql::Error::new(format!("Missing price for {coin_id}"))
            })?;

        // USD 乘以 100 存為 u64（精度 0.01 USD）
        Ok((price_f64 * 100.0) as u64)
    }
}
