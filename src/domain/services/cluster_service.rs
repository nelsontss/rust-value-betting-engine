use std::{collections::HashMap, fmt};

use crate::domain::entities::{Arbitrage, FixtureCluster, Game};

#[cfg(test)]
mod tests;

pub struct ClusterService {
    game_id_to_fixture_cluster_key: HashMap<String, String>,
    clusters: HashMap<String, FixtureCluster>,
}

impl ClusterService {
    pub fn new(games: Vec<Game>) -> Self {
        let mut cluster = ClusterService {
            clusters: HashMap::new(),
            game_id_to_fixture_cluster_key: HashMap::new(),
        };

        cluster.add_games(games);

        cluster
    }

    pub fn add_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();
        for game in games {
            let mut found = false;
            let game_id = game.id.clone();
            let mut pending_game = Some(game);

            for (_, cluster) in self.clusters.iter_mut() {
                match cluster.try_to_add_game(pending_game.take().unwrap()) {
                    Ok(_) => {
                        found = true;
                        self.game_id_to_fixture_cluster_key
                            .entry(game_id.clone())
                            .or_insert(cluster.key());
                        arbitrages.append(&mut cluster.arbitrage_opportunites());

                        break;
                    }
                    Err(game) => pending_game = Some(game),
                }
            }

            if !found {
                let cluster = FixtureCluster::new(pending_game.unwrap());
                let cluster_key = cluster.key();
                self.clusters.entry(cluster_key.clone()).or_insert(cluster);
                self.game_id_to_fixture_cluster_key
                    .entry(game_id)
                    .or_insert(cluster_key);
            }
        }

        arbitrages
    }

    pub fn update_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();

        games.into_iter().for_each(|game| {
            let game_id = game.id.clone();

            if let Some(cluster_id) = self.game_id_to_fixture_cluster_key.get(&game_id) {
                self.clusters
                    .entry(cluster_id.clone())
                    .and_modify(|cluster| {
                        cluster.update_markets(game_id, game.markets().values().collect())
                    });
                arbitrages.append(
                    &mut self
                        .clusters
                        .get(cluster_id)
                        .unwrap()
                        .arbitrage_opportunites(),
                );
            } else {
                arbitrages.append(&mut self.add_games(vec![game]));
            }
        });

        arbitrages
    }
}

impl fmt::Display for ClusterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, cluster) in &self.clusters {
            writeln!(f, "{}", cluster)?;
        }

        Ok(())
    }
}
