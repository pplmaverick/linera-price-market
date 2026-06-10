#![cfg_attr(target_arch = "wasm32", no_main)]

use linera_sdk::{
    linera_base_types::{Amount, TimeDelta, WithContractAbi},
    views::{RootView, View},
    Contract, ContractRuntime,
};

use linera_price_market::{
    state::{Asset, Bet, Direction, PriceMarket, Round, RoundStatus},
    Operation, PriceMarketAbi,
};

pub struct PriceMarketContract {
    state: PriceMarket,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(PriceMarketContract);

impl WithContractAbi for PriceMarketContract {
    type Abi = PriceMarketAbi;
}

impl Contract for PriceMarketContract {
    type Message = ();
    type InstantiationArgument = ();
    type Parameters = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = PriceMarket::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        PriceMarketContract { state, runtime }
    }

    async fn instantiate(&mut self, _: ()) {
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Operation) -> () {
        match operation {
            Operation::CreateRound { asset, duration_secs, start_price } => {
                self.create_round(asset, duration_secs, start_price).await;
            }
            Operation::PlaceBet { round_id, direction, amount } => {
                self.place_bet(round_id, direction, amount).await;
            }
            Operation::ResolveRound { round_id, final_price } => {
                self.resolve_round(round_id, final_price).await;
            }
            Operation::Claim { round_id } => {
                self.claim(round_id).await;
            }
        }
    }

    async fn execute_message(&mut self, _: ()) {
        panic!("PriceMarket does not support cross-chain messages");
    }

    async fn store(self) {
        self.state
            .save_and_drop()
            .await
            .expect("Failed to save state");
    }
}

impl PriceMarketContract {
    async fn create_round(&mut self, asset: Asset, duration_secs: u64, start_price: u64) {
        let now = self.runtime.system_time();
        let deadline = now.saturating_add(TimeDelta::from_secs(duration_secs));
        let counter = *self.state.round_counter.get();

        let round = Round {
            asset,
            start_price,
            end_price: 0,
            status: RoundStatus::Open,
            bets: Vec::new(),
            deadline,
            claimed: Vec::new(),
        };

        self.state
            .rounds
            .insert(&counter, round)
            .expect("Failed to insert round");
        self.state.round_counter.set(counter + 1);
    }

    async fn place_bet(&mut self, round_id: u64, direction: Direction, amount: Amount) {
        let caller = self
            .runtime
            .authenticated_signer()
            .expect("No authenticated signer");

        let mut round = self
            .state
            .rounds
            .get(&round_id)
            .await
            .expect("Failed to load round")
            .expect("Round not found");

        assert_eq!(round.status, RoundStatus::Open, "Round is not open");
        assert!(
            self.runtime.system_time() < round.deadline,
            "Round deadline has passed"
        );
        assert!(amount > Amount::ZERO, "Bet amount must be positive");

        round.bets.push(Bet { owner: caller, direction, amount });
        self.state
            .rounds
            .insert(&round_id, round)
            .expect("Failed to update round");
    }

    async fn resolve_round(&mut self, round_id: u64, final_price: u64) {
        let mut round = self
            .state
            .rounds
            .get(&round_id)
            .await
            .expect("Failed to load round")
            .expect("Round not found");

        assert_eq!(round.status, RoundStatus::Open, "Round is not open");

        round.end_price = final_price;
        round.status = RoundStatus::Settled;
        self.state
            .rounds
            .insert(&round_id, round)
            .expect("Failed to update round");
    }

    async fn claim(&mut self, round_id: u64) {
        let caller = self
            .runtime
            .authenticated_signer()
            .expect("No authenticated signer");

        let mut round = self
            .state
            .rounds
            .get(&round_id)
            .await
            .expect("Failed to load round")
            .expect("Round not found");

        assert_eq!(round.status, RoundStatus::Settled, "Round not settled");
        assert!(!round.claimed.contains(&caller), "Already claimed");

        let winning_direction = if round.end_price > round.start_price {
            Some(Direction::Up)
        } else if round.end_price < round.start_price {
            Some(Direction::Down)
        } else {
            None // tie — full refund
        };

        let caller_bets: Vec<&Bet> = round.bets.iter().filter(|b| b.owner == caller).collect();
        assert!(!caller_bets.is_empty(), "No bets for caller in this round");

        match winning_direction {
            None => {
                // 平局：退款（狀態記錄，實際 token 轉帳留待 service layer）
                round.claimed.push(caller);
            }
            Some(ref win_dir) => {
                let caller_winning = caller_bets
                    .iter()
                    .filter(|b| &b.direction == win_dir)
                    .fold(Amount::ZERO, |mut acc, b| {
                        acc.saturating_add_assign(b.amount);
                        acc
                    });
                assert!(caller_winning > Amount::ZERO, "No winning bets for caller");

                let total_winning = round
                    .bets
                    .iter()
                    .filter(|b| &b.direction == win_dir)
                    .fold(Amount::ZERO, |mut acc, b| {
                        acc.saturating_add_assign(b.amount);
                        acc
                    });

                let total_pot = round
                    .bets
                    .iter()
                    .fold(Amount::ZERO, |mut acc, b| {
                        acc.saturating_add_assign(b.amount);
                        acc
                    });

                // payout = total_pot * caller_winning / total_winning
                let _payout = Amount::from_attos(
                    total_pot.to_attos() * caller_winning.to_attos() / total_winning.to_attos(),
                );
                // token 轉帳到 caller 留待後續整合 fungible token

                round.claimed.push(caller);
            }
        }

        self.state
            .rounds
            .insert(&round_id, round)
            .expect("Failed to update round");
    }
}

#[cfg(test)]
mod tests {
    use linera_sdk::{
        linera_base_types::{AccountOwner, Amount, Timestamp},
        util::BlockingWait,
        views::View,
        Contract, ContractRuntime,
    };
    use linera_price_market::{
        state::{Asset, Direction, PriceMarket, RoundStatus},
        Operation,
    };

    use super::PriceMarketContract;

    fn owner(seed: u8) -> AccountOwner {
        AccountOwner::from([seed; 32])
    }

    fn make_contract(signer: AccountOwner) -> PriceMarketContract {
        let runtime = ContractRuntime::new()
            .with_application_parameters(())
            .with_system_time(Timestamp::from(0))
            .with_authenticated_signer(Some(signer));
        let state = PriceMarket::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to load state");
        let mut contract = PriceMarketContract { state, runtime };
        contract.instantiate(()).blocking_wait();
        contract
    }

    #[test]
    fn test_create_round() {
        let mut c = make_contract(owner(1));
        c.execute_operation(Operation::CreateRound {
            asset: Asset::Btc,
            duration_secs: 300,
            start_price: 9_500_000,
        })
        .blocking_wait();

        let round = c.state.rounds.get(&0).blocking_wait().unwrap().unwrap();
        assert_eq!(round.status, RoundStatus::Open);
        assert_eq!(round.start_price, 9_500_000);
        assert_eq!(round.asset, Asset::Btc);
        assert!(round.bets.is_empty());
    }

    #[test]
    fn test_place_bet() {
        let mut c = make_contract(owner(1));
        c.execute_operation(Operation::CreateRound {
            asset: Asset::Btc,
            duration_secs: 300,
            start_price: 9_500_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::PlaceBet {
            round_id: 0,
            direction: Direction::Up,
            amount: Amount::from_tokens(1),
        })
        .blocking_wait();

        let round = c.state.rounds.get(&0).blocking_wait().unwrap().unwrap();
        assert_eq!(round.bets.len(), 1);
        assert_eq!(round.bets[0].direction, Direction::Up);
        assert_eq!(round.bets[0].owner, owner(1));
    }

    #[test]
    fn test_resolve_round_up_wins() {
        let mut c = make_contract(owner(1));
        c.execute_operation(Operation::CreateRound {
            asset: Asset::Btc,
            duration_secs: 300,
            start_price: 9_500_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::PlaceBet {
            round_id: 0,
            direction: Direction::Up,
            amount: Amount::from_tokens(1),
        })
        .blocking_wait();
        c.execute_operation(Operation::ResolveRound {
            round_id: 0,
            final_price: 9_600_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::Claim { round_id: 0 }).blocking_wait();

        let round = c.state.rounds.get(&0).blocking_wait().unwrap().unwrap();
        assert_eq!(round.status, RoundStatus::Settled);
        assert_eq!(round.end_price, 9_600_000);
        assert!(round.claimed.contains(&owner(1)));
    }

    #[test]
    fn test_resolve_round_down_wins() {
        let mut c = make_contract(owner(2));
        c.execute_operation(Operation::CreateRound {
            asset: Asset::Eth,
            duration_secs: 300,
            start_price: 3_000_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::PlaceBet {
            round_id: 0,
            direction: Direction::Down,
            amount: Amount::from_tokens(1),
        })
        .blocking_wait();
        c.execute_operation(Operation::ResolveRound {
            round_id: 0,
            final_price: 2_900_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::Claim { round_id: 0 }).blocking_wait();

        let round = c.state.rounds.get(&0).blocking_wait().unwrap().unwrap();
        assert_eq!(round.status, RoundStatus::Settled);
        assert!(round.claimed.contains(&owner(2)));
    }

    #[test]
    #[should_panic(expected = "Already claimed")]
    fn test_claim_already_claimed() {
        let mut c = make_contract(owner(3));
        c.execute_operation(Operation::CreateRound {
            asset: Asset::Sol,
            duration_secs: 300,
            start_price: 15_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::PlaceBet {
            round_id: 0,
            direction: Direction::Up,
            amount: Amount::from_tokens(1),
        })
        .blocking_wait();
        c.execute_operation(Operation::ResolveRound {
            round_id: 0,
            final_price: 16_000,
        })
        .blocking_wait();
        c.execute_operation(Operation::Claim { round_id: 0 }).blocking_wait();
        // 第二次應 panic "Already claimed"
        c.execute_operation(Operation::Claim { round_id: 0 }).blocking_wait();
    }
}
