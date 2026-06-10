pub mod state;

use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{Amount, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub use state::{Asset, Direction};

pub struct PriceMarketAbi;

impl ContractAbi for PriceMarketAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for PriceMarketAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    CreateRound { asset: Asset, duration_secs: u64, start_price: u64 },
    PlaceBet { round_id: u64, direction: Direction, amount: Amount },
    ResolveRound { round_id: u64, final_price: u64 },
    Claim { round_id: u64 },
}
