use std::{collections::HashMap, fmt, sync::Arc};

use chrono::NaiveDateTime;

use crate::domain::entities::{FixtureCluster, SharedGame};

#[cfg(test)]
mod tests;

pub struct ClusterService {
    clusters: HashMap<NaiveDateTime, Vec<FixtureCluster>>,
}

impl ClusterService {
    pub fn new(games: Vec<SharedGame>) -> Self {
        let clusters: HashMap<NaiveDateTime, Vec<FixtureCluster>> = HashMap::new();

        let mut cluster = ClusterService { clusters };

        cluster.add_games(games);

        cluster
    }

    pub fn add_games(&mut self, games: Vec<SharedGame>) {
        for game in games {
            let clusters_on_game_date = self.clusters.entry(game.read().unwrap().date).or_default();
            let mut found = false;

            for cluster in clusters_on_game_date.iter_mut() {
                if cluster.try_to_add_game(Arc::clone(&game)) {
                    found = true;
                    break;
                }
            }

            if !found {
                clusters_on_game_date.push(FixtureCluster::new(game));
            }
        }
    }
}

impl fmt::Display for ClusterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, cluster_on_date) in &self.clusters {
            for cluster in cluster_on_date {
                writeln!(f, "{}", cluster)?;
            }
        }

        Ok(())
    }
}
