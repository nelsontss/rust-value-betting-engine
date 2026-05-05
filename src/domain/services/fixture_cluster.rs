use core::fmt;
use std::collections::HashMap;

use crate::domain::entities::{Game, Market, MarketType};

#[cfg(test)]
mod tests;

pub struct FixtureCluster<'a> {
    key: String,
    games: Vec<&'a Game>,
    markets: HashMap<MarketType, Vec<&'a Market>>,
}

impl<'a> FixtureCluster<'a> {
    pub fn new(game: &'a Game) -> Self {
        FixtureCluster {
            key: game.canonical_name(),
            games: vec![game],
            markets: game
                .markets
                .iter()
                .map(|(market_type, market)| (market_type.clone(), vec![market]))
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
        for (market_type, market) in game.markets.iter() {
            self.markets
                .entry(market_type.clone())
                .or_default()
                .push(market);
        }
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
