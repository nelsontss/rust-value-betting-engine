use core::fmt;
use std::collections::HashMap;

use crate::domain::entities::{Arbitrage, Game, MarketGroup, MarketType};

#[cfg(test)]
mod tests;

pub struct FixtureCluster<'a> {
    key: String,
    games: Vec<&'a Game>,
    markets: HashMap<MarketType, MarketGroup<'a>>,
}

impl<'a> FixtureCluster<'a> {
    pub fn new(game: &'a Game) -> Self {
        FixtureCluster {
            key: game.canonical_name(),
            games: vec![game],
            markets: game
                .markets
                .iter()
                .map(|(_, market)| {
                    let group = MarketGroup::from_market(market);
                    let market_type = group.market_type();
                    (market_type, group)
                })
                .collect(),
        }
    }

    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    pub fn try_to_add_game(&mut self, game: &'a Game) -> bool {
        if self
            .games
            .iter()
            .filter(|&g| game.similarity_score(g) > 0.85)
            .count() as f32
            > self.games.len() as f32 * 0.66
        {
            self.add_game(game);

            return true;
        }

        false
    }

    fn add_game(&mut self, game: &'a Game) {
        self.games.push(game);
        for market in game.markets.values() {
            let group = MarketGroup::from_market(market);
            let market_type = group.market_type();

            self.markets
                .entry(market_type)
                .and_modify(|existing_group| existing_group.push_market(market))
                .or_insert(group);
        }
    }

    fn arbitrage_opportunites(&self) -> Vec<Arbitrage> {
        self.markets
            .values()
            .filter_map(MarketGroup::arbitrage)
            .collect()
    }
}

impl<'a> fmt::Display for FixtureCluster<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--------------- {} ----------------", self.key)?;

        for game in self.games.iter() {
            writeln!(f, "{}", game.canonical_name())?;
        }

        Ok(())
    }
}
