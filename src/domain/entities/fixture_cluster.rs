use core::fmt;
use std::{collections::HashMap, sync::Arc};

use crate::domain::entities::{Arbitrage, MarketGroup, MarketType, SharedGame};

#[cfg(test)]
mod tests;

struct GameMarketPointer {
    game: SharedGame,
    market_type: MarketType,
}

pub struct FixtureCluster {
    key: String,
    games: HashMap<String, SharedGame>,
    // Secondary index by market type. The current market state always lives in SharedGame;
    // this index is only used to find candidate markets efficiently across platforms.
    markets: HashMap<MarketType, Vec<GameMarketPointer>>,
}

enum EngineError {
    PoisonedGameError,
}

impl FixtureCluster {
    pub fn new(game: SharedGame) -> Self {
        let mut fixture_cluster = FixtureCluster {
            key: game.read().unwrap().canonical_name(),
            games: HashMap::new(),
            markets: HashMap::new(),
        };

        fixture_cluster.add_game(game);

        fixture_cluster
    }

    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    pub fn try_to_add_game(&mut self, game: SharedGame) -> bool {
        if self
            .games
            .iter()
            .filter(|(_, other_game)| {
                game.read()
                    .unwrap()
                    .similarity_score(&Arc::clone(other_game).read().unwrap())
                    > 0.85
            })
            .count() as f32
            > self.games.len() as f32 * 0.66
        {
            self.add_game(game);

            return true;
        }

        false
    }

    fn add_game(&mut self, game: SharedGame) {
        let snapshot = {
            let game_ref = game
                .read()
                .expect("game lock poisoned while adding to cluster");
            (
                game_ref.id.clone(),
                game_ref.markets().keys().cloned().collect::<Vec<_>>(),
            )
        };

        let game_id = snapshot.0;
        let market_types = snapshot.1;

        if !self.games.contains_key(&game_id) {
            self.games.entry(game_id).or_insert(Arc::clone(&game));

            for market_type in market_types {
                self.markets
                    .entry(market_type.clone())
                    .or_default()
                    .push(GameMarketPointer {
                        game: Arc::clone(&game),
                        market_type,
                    });
            }
        }
    }

    fn arbitrage_opportunites(&self) -> Vec<Arbitrage> {
        self.markets
            .values()
            .filter_map(|pointers| self.build_market_group(pointers))
            .filter_map(|group| group.arbitrage())
            .collect()
    }

    fn build_market_group(&self, pointers: &[GameMarketPointer]) -> Option<MarketGroup> {
        let mut markets = pointers.iter().filter_map(|pointer| {
            let game = pointer
                .game
                .read()
                .expect("game lock poisoned while building market group");

            game.markets().get(&pointer.market_type).cloned()
        });

        let first_market = markets.next()?;
        let mut group = MarketGroup::from_market(first_market);

        for market in markets {
            group.push_market(market).ok()?;
        }

        Some(group)
    }
}

impl<'a> fmt::Display for FixtureCluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--------------- {} ----------------", self.key)?;

        for (_, game) in self.games.iter() {
            writeln!(f, "{}", game.read().unwrap().canonical_name())?;
        }

        Ok(())
    }
}
