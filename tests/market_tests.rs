// Integration-style tests for the public library types.
// The 5 contract behaviour tests live in src/contract.rs (inline mod tests)
// because PriceMarketContract is a [[bin]] target and cannot be imported here.

use linera_price_market::{
    state::{Asset, Direction, RoundStatus},
    Operation,
};
use linera_sdk::linera_base_types::Amount;

#[test]
fn test_asset_variants_are_distinct() {
    assert_ne!(Asset::Btc, Asset::Eth);
    assert_ne!(Asset::Eth, Asset::Sol);
    assert_ne!(Asset::Btc, Asset::Sol);
}

#[test]
fn test_direction_variants_are_distinct() {
    assert_ne!(Direction::Up, Direction::Down);
}

#[test]
fn test_round_status_default_is_open() {
    assert_eq!(RoundStatus::default(), RoundStatus::Open);
}

#[test]
fn test_operation_create_round_serializes() {
    let op = Operation::CreateRound {
        asset: Asset::Btc,
        duration_secs: 300,
        start_price: 9_500_000,
    };
    let json = serde_json::to_string(&op).expect("serialization failed");
    assert!(json.contains("CreateRound"));
    assert!(json.contains("9500000"));
}

#[test]
fn test_operation_place_bet_roundtrips() {
    let op = Operation::PlaceBet {
        round_id: 0,
        direction: Direction::Down,
        amount: Amount::from_tokens(2),
    };
    let json = serde_json::to_string(&op).expect("serialization failed");
    let decoded: Operation = serde_json::from_str(&json).expect("deserialization failed");
    let re_json = serde_json::to_string(&decoded).expect("re-serialization failed");
    assert_eq!(json, re_json);
}
