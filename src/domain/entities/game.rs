use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock, RwLock},
};

use chrono::NaiveDateTime;
use deunicode::deunicode;
use strsim;
use uuid::Uuid;

use crate::domain::entities::market::{Market, MarketType};

#[cfg(test)]
mod tests;

pub type SharedGame = Arc<RwLock<Game>>;

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    home_team: String,
    away_team: String,
    country: String,
    competition: String,
    platform: String,
    pub date: NaiveDateTime,
    pub markets: HashMap<MarketType, Market>,
}

struct SimilarityWeights;

impl SimilarityWeights {
    const COUNTRY: f64 = 0.1;
    const COMPETITION: f64 = 0.1;
    const HOME_TEAM: f64 = 0.4;
    const AWAY_TEAM: f64 = 0.4;
}

impl Game {
    pub fn new(
        home_team: &str,
        away_team: &str,
        country: &str,
        competition: &str,
        date: NaiveDateTime,
        platform: &str,
    ) -> Self {
        Game {
            id: Uuid::new_v4().to_string(),
            home_team: home_team.to_string(),
            away_team: away_team.to_string(),
            country: country.to_string(),
            competition: competition.to_string(),
            platform: platform.to_string(),
            date,
            markets: HashMap::new(),
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
        let (weak, strong): (Vec<&str>, Vec<&str>) =
            tokens.into_iter().partition(|token| is_weak_token(token));

        weak.into_iter().chain(strong).collect::<Vec<_>>().join(" ")
    }

    pub fn canonical_name(&self) -> String {
        format!(
            "{} vs {} @ {}",
            Self::normalize_name(&self.home_team),
            Self::normalize_name(&self.away_team),
            self.date
        )
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
        "fc", "cf", "sc", "ac", "sl", "cp", "cd", "fk", "bk", "if", "nk", "sk",
    ])
});

fn resolve_alias(name: &str) -> &str {
    TEAM_ALIASES.get(name).copied().unwrap_or(name)
}

fn is_weak_token(name: &str) -> bool {
    WEAK_TOKENS.contains(name)
}
