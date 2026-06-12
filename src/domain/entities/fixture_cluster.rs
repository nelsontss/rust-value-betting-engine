use core::fmt;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::domain::{
    Platform,
    entities::{Arbitrage, Game, Market, MarketGroup, MarketType},
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct FixtureCluster {
    key: String,
    games: HashMap<String, Game>,
    // Secondary index by market type. The current market state always lives in Game;
    // this index is only used to find candidate markets efficiently across platforms.
    market_type_to_game_ids: HashMap<MarketType, HashSet<String>>,
    updated_at: DateTime<Utc>,
    representative_game: Option<Game>,
}

impl FixtureCluster {
    const REPRESENTATIVE_PLATFORM: Platform = Platform::Betano;

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn new(game: Game) -> Self {
        let mut fixture_cluster = FixtureCluster {
            key: game.canonical_name(),
            games: HashMap::new(),
            market_type_to_game_ids: HashMap::new(),
            updated_at: chrono::Utc::now(),
            representative_game: Some(game.clone()),
        };

        fixture_cluster.add_game(game);

        fixture_cluster
    }

    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    pub fn try_to_add_game(&mut self, game: Game) -> Result<(), Game> {
        if self
            .games
            .iter()
            .filter(|(_, other_game)| {
                let score = game.similarity_score(*other_game);

                score > 0.85 && game.date == other_game.date
            })
            .count() as f32
            > self.games.len() as f32 * 0.66
        {
            self.add_game(game);

            return Ok(());
        }

        Err(game)
    }

    fn add_game(&mut self, game: Game) {
        let market_types = game.markets().keys().cloned().collect::<Vec<_>>();
        let game_id = game.id.clone();

        if !self.games.contains_key(&game_id) {
            if game.platform() == FixtureCluster::REPRESENTATIVE_PLATFORM {
                self.representative_game = Some(game.clone());
            }

            self.games.entry(game_id.clone()).or_insert(game);

            for market_type in market_types {
                self.market_type_to_game_ids
                    .entry(market_type.clone())
                    .or_default()
                    .insert(game_id.clone());
            }

            self.updated_at = chrono::Utc::now();
        }
    }

    pub fn update_markets(&mut self, game_id: &str, markets: Vec<Market>) {
        if self.games.contains_key(game_id) {
            self.games
                .entry(game_id.to_string())
                .and_modify(|g| g.update_markets(markets));

            let game = self.games.get(game_id).unwrap();
            let market_types = game.markets().keys().cloned().collect::<Vec<_>>();

            if game.platform() == FixtureCluster::REPRESENTATIVE_PLATFORM {
                self.representative_game = Some(game.clone());
            }

            for market_type in market_types {
                self.market_type_to_game_ids
                    .entry(market_type.clone())
                    .or_default()
                    .insert(game_id.to_string());
            }

            self.updated_at = chrono::Utc::now();
        }
    }

    pub fn arbitrage_opportunites(&self) -> Vec<Arbitrage> {
        self.market_type_to_game_ids
            .iter()
            .filter_map(|entry| self.build_market_group(entry))
            .filter_map(|group| group.arbitrage())
            .collect()
    }

    fn build_market_group(&self, entry: (&MarketType, &HashSet<String>)) -> Option<MarketGroup> {
        let mut markets = entry.1.iter().filter_map(|game_id| {
            if let Some(game) = self.games.get(game_id) {
                game.markets().get(entry.0).cloned()
            } else {
                None
            }
        });

        let first_market = markets.next()?;
        let mut group = MarketGroup::from_market(first_market);

        for market in markets {
            group.push_market(market).ok()?;
        }

        Some(group)
    }

    pub fn print_games_list(&self) {
        for (_, game) in self.games.iter() {
            let platform = format!("{:?}", game.platform()).to_lowercase();
            println!(
                "{} vs {} @ {}",
                game.home_team(),
                game.away_team(),
                platform
            );
        }
    }

    pub fn games(&self) -> impl Iterator<Item = &Game> {
        self.games.values().into_iter()
    }

    pub fn platform_games(&self, platform: &Platform) -> impl Iterator<Item = &Game> {
        self.games
            .values()
            .filter(|g| g.platform() == *platform)
            .into_iter()
    }

    pub fn representative_game(&self) -> Option<&Game> {
        self.representative_game.as_ref()
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at.clone()
    }
}

impl<'a> fmt::Display for FixtureCluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--------------- {} ----------------", self.key)?;

        for (_, game) in self.games.iter() {
            writeln!(f, "{}", game.canonical_name())?;
        }

        Ok(())
    }
}
