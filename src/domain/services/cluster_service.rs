use std::{collections::HashMap, fmt};

use chrono::NaiveDateTime;

use crate::domain::{entities::Game, services::FixtureCluster};

#[cfg(test)]
mod tests;

struct ClusterService<'a> {
    clusters: HashMap<NaiveDateTime, Vec<FixtureCluster<'a>>>,
}

impl<'a> ClusterService<'a> {
    pub fn new(games: &'a [Game]) -> Self {
        let clusters: HashMap<NaiveDateTime, Vec<FixtureCluster>> = HashMap::new();

        let mut cluster = ClusterService { clusters };

        cluster.add_games(games);

        cluster
    }

    pub fn add_games(&mut self, games: &'a [Game]) {
        for game in games {
            let clusters_on_game_date = self.clusters.entry(game.date).or_default();
            let mut found = false;

            for cluster in clusters_on_game_date.iter_mut() {
                if cluster.try_to_add_game(game) {
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

impl<'a> fmt::Display for ClusterService<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, cluster_on_date) in &self.clusters {
            for cluster in cluster_on_date {
                writeln!(f, "{}", cluster)?;
            }
        }

        Ok(())
    }
}
