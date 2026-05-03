use std::{collections::HashMap, fmt};

use chrono::NaiveDateTime;

use crate::domain::entities::Game;

#[cfg(test)]
mod tests;

struct FixtureCluster {
    key: String,
    game_ids: Vec<String>,
}

struct ClusterService {
    pub games: HashMap<String, Game>,
    pub clusters: HashMap<NaiveDateTime, Vec<FixtureCluster>>,
}

impl ClusterService {
    pub fn new(games: Vec<Game>) -> Self {
        let self_games: HashMap<String, Game> = HashMap::new();
        let clusters: HashMap<NaiveDateTime, Vec<FixtureCluster>> = HashMap::new();

        let mut cluster = ClusterService {
            games: self_games,
            clusters,
        };

        cluster.add_games(games);

        cluster
    }

    pub fn add_games(&mut self, games: Vec<Game>) {
        for game in games {
            if self.games.contains_key(&game.id) {
                continue;
            }

            self.games.entry(game.id.clone()).or_insert(game.clone());

            let clusters_on_game_date = self.clusters.entry(game.date).or_default();
            let mut found = false;

            for cluster in clusters_on_game_date.iter_mut() {
                if cluster
                    .game_ids
                    .iter()
                    .filter_map(|g| self.games.get(g))
                    .filter(|&g| game.similarity_score(g) > 0.85)
                    .count() as f32
                    > cluster.game_ids.len() as f32 * 0.66
                {
                    cluster.game_ids.push(game.id.clone());
                    found = true;
                    break;
                }
            }

            if !found {
                clusters_on_game_date.push(FixtureCluster {
                    key: game.canonical_name(),
                    game_ids: vec![game.id.clone()],
                });
            }
        }
    }
}

impl fmt::Display for ClusterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, cluster_on_date) in &self.clusters {
            for cluster in cluster_on_date {
                writeln!(f, "--------------- {} ----------------", cluster.key)?;

                for game_id in &cluster.game_ids {
                    if let Some(game) = self.games.get(game_id) {
                        writeln!(f, "{}", game.canonical_name())?;
                    }
                }
            }
        }

        Ok(())
    }
}
