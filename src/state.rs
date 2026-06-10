use linera_sdk::{
    linera_base_types::{AccountOwner, Amount, Timestamp},
    views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext},
};
use serde::{Deserialize, Serialize};

#[derive(async_graphql::Enum, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Asset {
    Btc,
    Eth,
    Sol,
}

#[derive(async_graphql::Enum, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(async_graphql::Enum, Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum RoundStatus {
    #[default]
    Open,
    Locked,
    Settled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bet {
    pub owner: AccountOwner,
    pub direction: Direction,
    pub amount: Amount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Round {
    pub asset: Asset,
    pub start_price: u64,
    pub end_price: u64,
    pub status: RoundStatus,
    pub bets: Vec<Bet>,
    pub deadline: Timestamp,
    pub claimed: Vec<AccountOwner>,
}

#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct PriceMarket {
    pub rounds: MapView<u64, Round>,
    pub round_counter: RegisterView<u64>,
}
