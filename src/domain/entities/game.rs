use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use chrono::NaiveDateTime;
use deunicode::deunicode;
use strsim;
use uuid::Uuid;

use crate::domain::entities::Platform;
use crate::domain::entities::market::{Market, MarketType};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    home_team: String,
    away_team: String,
    country: String,
    competition: String,
    platform: Platform,
    pub date: NaiveDateTime,
    markets: HashMap<MarketType, Market>,
}

struct SimilarityWeights;

impl SimilarityWeights {
    const COUNTRY: f64 = 0.1;
    const COMPETITION: f64 = 0.02;
    const HOME_TEAM: f64 = 0.44;
    const AWAY_TEAM: f64 = 0.44;
}

impl Game {
    pub fn new_with_id(
        id: &str,
        home_team: &str,
        away_team: &str,
        country: &str,
        competition: &str,
        date: NaiveDateTime,
        platform: Platform,
        markets: Vec<Market>,
    ) -> Self {
        let mut game = Game::new(
            home_team,
            away_team,
            country,
            competition,
            date,
            platform,
            markets,
        );

        game.id = id.to_string();

        game
    }

    pub fn new(
        home_team: &str,
        away_team: &str,
        country: &str,
        competition: &str,
        date: NaiveDateTime,
        platform: Platform,
        markets: Vec<Market>,
    ) -> Self {
        Game {
            id: Uuid::new_v4().to_string(),
            home_team: home_team.to_string(),
            away_team: away_team.to_string(),
            country: country.to_string(),
            competition: competition.to_string(),
            platform,
            date,
            markets: markets
                .into_iter()
                .map(|market| (MarketType::from(&market), market))
                .collect(),
        }
    }

    pub fn same_fixture_as(&self, other_game: &Self) -> bool {
        if !self.date.eq(&other_game.date) {
            return false;
        }

        let country_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.country),
            &Self::normalize_name(&other_game.country),
        );
        let competition_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.competition),
            &Self::normalize_name(&other_game.competition),
        );
        let home_team_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.home_team),
            &Self::normalize_name(&other_game.home_team),
        );
        let away_team_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.away_team),
            &Self::normalize_name(&other_game.away_team),
        );

        if country_similarity >= 0.6
            && competition_similarity >= 0.6
            && home_team_similarity >= 0.85
            && away_team_similarity >= 0.85
        {
            return true;
        }

        false
    }

    pub fn similarity_score(&self, other_game: &Self) -> f64 {
        if !self.date.eq(&other_game.date) {
            return 0.0;
        }

        let country_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.country),
            &Self::normalize_name(&other_game.country),
        );
        let competition_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.competition),
            &Self::normalize_name(&other_game.competition),
        );
        let home_team_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.home_team),
            &Self::normalize_name(&other_game.home_team),
        );
        let away_team_similarity = strsim::jaro_winkler(
            &Self::normalize_name(&self.away_team),
            &Self::normalize_name(&other_game.away_team),
        );

        (country_similarity * SimilarityWeights::COUNTRY)
            + (competition_similarity * SimilarityWeights::COMPETITION)
            + (home_team_similarity * SimilarityWeights::HOME_TEAM)
            + (away_team_similarity * SimilarityWeights::AWAY_TEAM)
    }

fn normalize_name(name: &str) -> String {
    let binding = deunicode(name).to_lowercase();
    let tokens: Vec<&str> = binding.split_whitespace().map(resolve_alias).collect();
    tokens
        .into_iter()
        .filter(|token| !is_weak_token(token))
        .collect::<Vec<_>>()
        .join(" ")
}

    pub fn canonical_name(&self) -> String {
        format!(
            "{} vs {} @ {}",
            Self::normalize_name(&self.home_team),
            Self::normalize_name(&self.away_team),
            self.date
        )
    }

    pub fn update_markets(&mut self, markets: Vec<&Market>) {
        markets.into_iter().for_each(|market| {
            let market_type = MarketType::from(&market);

            self.markets.entry(market_type).insert_entry(market.clone());
        });
    }

    pub fn markets(&self) -> &HashMap<MarketType, Market> {
        &self.markets
    }

    pub fn home_team(&self) -> &str {
        &self.home_team
    }

    pub fn away_team(&self) -> &str {
        &self.away_team
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn competition(&self) -> &str {
        &self.competition
    }

    pub fn country(&self) -> &str {
        &self.country
    }
}

static TEAM_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("man", "manchester"),
        ("utd", "united"),
        ("st", "saint"),
        ("ste", "sainte"),
        ("sp", "sporting"),
        ("ath", "athletic"),
        ("dep", "deportivo"),
        ("int", "internacional"),
    ])
});

static WEAK_TOKENS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "ca", "fc", "cf", "sc", "ac", "sl", "cp", "cd", "fk", "bk", "if", "nk", "sk",
    ])
});

fn resolve_alias(name: &str) -> &str {
    TEAM_ALIASES.get(name).copied().unwrap_or(name)
}

fn is_weak_token(name: &str) -> bool {
    WEAK_TOKENS.contains(name)
}
