use core::fmt;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use num_traits::ToPrimitive as _;
use tracing::warn;

use crate::domain::{
    Platform,
    entities::{Arbitrage, Game, Market, MarketGroup, MarketType, Outcome},
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
    diffs: HashMap<MarketType, HashMap<Outcome, Vec<(f64, f64)>>>,
    mean_diffs: HashMap<MarketType, HashMap<Outcome, f64>>,
    closed: bool,
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
            diffs: HashMap::new(),
            mean_diffs: HashMap::new(),
            closed: false,
        };

        fixture_cluster.add_game(game);

        fixture_cluster
    }

    pub fn from_persisted(
        key: String,
        games: Vec<Game>,
        updated_at: DateTime<Utc>,
        mean_diffs: HashMap<MarketType, HashMap<Outcome, f64>>,
        closed: bool,
    ) -> Self {
        let mut fixture_cluster = FixtureCluster {
            key,
            games: HashMap::new(),
            market_type_to_game_ids: HashMap::new(),
            updated_at,
            representative_game: None,
            diffs: HashMap::new(),
            mean_diffs,
            closed,
        };

        // Reconstructing a persisted cluster must not fabricate tick history;
        // games are inserted without recording diffs.
        for game in games {
            fixture_cluster.insert_game(game);
        }

        fixture_cluster.updated_at = updated_at;

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
        self.insert_game(game);
        self.record_live_diffs();
    }

    fn insert_game(&mut self, game: Game) {
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

    fn record_live_diffs(&mut self) {
        for (market_type, inner) in self.live_statistics_diffs(&[]) {
            for (outcome, diff) in inner {
                self.diffs
                    .entry(market_type)
                    .or_default()
                    .entry(outcome)
                    .or_default()
                    .push(diff);
            }
        }
    }

    pub fn update_markets(&mut self, game_id: &str, markets: Vec<Market>) -> Vec<MarketType> {
        if markets.is_empty() {
            return Vec::new();
        }

        let Some(game) = self.games.get_mut(game_id) else {
            return Vec::new();
        };

        let updated_markets: Vec<MarketType> = game.update_markets(markets).into_iter().collect();
        let market_types: Vec<MarketType> = game.markets().keys().cloned().collect();
        let game_id = game_id.to_string();

        for market_type in market_types {
            self.market_type_to_game_ids
                .entry(market_type)
                .or_default()
                .insert(game_id.clone());
        }

        self.record_live_diffs();

        self.updated_at = chrono::Utc::now();

        updated_markets
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

    /// Diffs concluídos da fixture: as médias persistidas quando a fixture
    /// foi carregada da BD, ou a média dos ticks acumulados em memória.
    pub fn statistics_diffs(&self) -> HashMap<MarketType, HashMap<Outcome, f64>> {
        if !self.mean_diffs.is_empty() {
            return self.mean_diffs.clone();
        } else if !self.diffs.is_empty() {
            let mut out: HashMap<MarketType, HashMap<Outcome, f64>> = HashMap::new();
            for (mt, inner) in &self.diffs {
                let mut inner_out = HashMap::new();
                for (outcome, v) in inner {
                    let sum: f64 = v.iter().map(|(a, _)| a).sum();
                    inner_out.insert(*outcome, sum / v.len() as f64);
                }
                out.insert(*mt, inner_out);
            }
            return out;
        }

        HashMap::new()
    }

    pub fn live_statistics_diffs(
        &self,
        for_markets: &[MarketType],
    ) -> HashMap<MarketType, HashMap<Outcome, (f64, f64)>> {
        let mut map: HashMap<MarketType, HashMap<Outcome, (f64, f64)>> = HashMap::new();
        let market_types: Box<dyn Iterator<Item = &MarketType>> = if for_markets.is_empty() {
            Box::new(self.market_type_to_game_ids.keys())
        } else {
            Box::new(for_markets.iter())
        };

        for market_type in market_types {
            let mut inner = HashMap::new();
            for outcome in market_type.outcomes() {
                if let Some(diff) = self.live_diff_for_outcome(market_type, &outcome) {
                    inner.insert(outcome, diff);
                }
            }
            if !inner.is_empty() {
                map.insert(*market_type, inner);
            }
        }
        map
    }

    fn live_diff_for_outcome(
        &self,
        market_type: &MarketType,
        outcome: &Outcome,
    ) -> Option<(f64, f64)> {
        let mut poly_value: Option<(f64, f64)> = None;
        let mut other_values: Vec<f64> = Vec::new();

        for game in self.games() {
            let Some(market) = game.markets().get(market_type) else {
                continue;
            };
            let Some(odd) = market.odd_for_outcome(outcome) else {
                continue;
            };

            if game.platform() == Platform::Polymarket {
                let Some(prob) = odd.get_implied_probability().to_f64() else {
                    warn!(
                        cluster = %self.key(),
                        home = game.home_team(),
                        away = game.away_team(),
                        ?outcome,
                        "implied probability not representable as f64; skipping polymarket sample"
                    );
                    continue;
                };
                let Some(prob_no) = odd
                    .get_implied_probability_derived_from_no()
                    .and_then(|p| p.to_f64())
                else {
                    warn!(
                        cluster = %self.key(),
                        home = game.home_team(),
                        away = game.away_team(),
                        ?outcome,
                        "missing implied probability derived from NO; skipping polymarket sample"
                    );
                    continue;
                };
                poly_value = Some((prob, prob_no));
            } else {
                let Some(prob) = odd.get_implied_probability().to_f64() else {
                    warn!(
                        cluster = %self.key(),
                        home = game.home_team(),
                        away = game.away_team(),
                        ?outcome,
                        "implied probability not representable as f64; skipping sample"
                    );
                    continue;
                };
                other_values.push(prob);
            }
        }

        let (poly_value, poly_value_from_no) = poly_value?;
        let median_other = median_of(&mut other_values)?;

        Some((poly_value - median_other, poly_value_from_no - median_other))
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

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn close(&mut self) {
        if !self.closed {
            self.closed = true;
            self.updated_at = chrono::Utc::now();
        }
    }

    pub fn get_game(&self, game_id: &str) -> Option<&Game> {
        self.games.get(game_id)
    }

    pub fn get_polymarket_impl_prob_of_market_and_outcome(
        &self,
        market_type: &MarketType,
        outcome: &Outcome,
    ) -> Option<f64> {
        if let Some((_, polymarket_game)) = self
            .games
            .iter()
            .find(|(_, g)| g.platform() == Platform::Polymarket)
        {
            let market = polymarket_game.markets().get(market_type)?;

            let odd = market.odd_for_outcome(outcome)?;

            return if let Some(prob) = odd.get_implied_probability().to_f64() {
                Some(prob)
            } else {
                None
            };
        }

        None
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

fn median_of(values: &mut Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let middle = values.len() / 2;

    if values.len() % 2 == 0 {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Some(values[middle])
    }
}
