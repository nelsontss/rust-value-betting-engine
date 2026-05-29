use std::{collections::HashMap, fmt};

use chrono::NaiveDateTime;

use crate::domain::{
    Market,
    entities::{Arbitrage, FixtureCluster, Game},
};

#[cfg(test)]
mod tests;

pub struct ClusterService {
    game_id_to_fixture_cluster_key: HashMap<String, String>,
    cluster_id_to_date: HashMap<String, NaiveDateTime>,
    clusters: HashMap<NaiveDateTime, HashMap<String, FixtureCluster>>,
}

impl ClusterService {
    pub fn new(games: Vec<Game>) -> Self {
        let mut cluster = ClusterService {
            clusters: HashMap::new(),
            game_id_to_fixture_cluster_key: HashMap::new(),
            cluster_id_to_date: HashMap::new(),
        };

        cluster.add_games(games);

        cluster
    }

    pub fn add_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();
        for game in games {
            if self.game_id_to_fixture_cluster_key.contains_key(&game.id) {
                continue;
            }

            let mut found = false;
            let game_id = game.id.clone();
            let game_date = game.date.clone();
            let mut pending_game = Some(game);

            if let Some(clusters) = self.clusters.get_mut(&game_date) {
                for (_, cluster) in clusters.iter_mut() {
                    match cluster.try_to_add_game(pending_game.take().unwrap()) {
                        Ok(_) => {
                            found = true;
                            self.game_id_to_fixture_cluster_key
                                .entry(game_id.clone())
                                .or_insert(cluster.key());
                            self.cluster_id_to_date
                                .entry(cluster.key())
                                .insert_entry(game_date);
                            arbitrages.append(&mut cluster.arbitrage_opportunites());

                            break;
                        }
                        Err(game) => pending_game = Some(game),
                    }
                }
            }

            if !found {
                let cluster = FixtureCluster::new(pending_game.unwrap());
                let cluster_key = cluster.key();
                self.clusters
                    .entry(game_date)
                    .or_insert_with(HashMap::new)
                    .entry(cluster_key.clone())
                    .or_insert(cluster);
                self.game_id_to_fixture_cluster_key
                    .entry(game_id)
                    .or_insert(cluster_key.clone());
                self.cluster_id_to_date
                    .entry(cluster_key)
                    .insert_entry(game_date);
            }
        }

        arbitrages
    }

    pub fn update_markets(
        &mut self,
        game_ids_to_markets: Vec<(String, Vec<Market>)>,
    ) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();

        game_ids_to_markets
            .into_iter()
            .for_each(|(game_id, markets)| {
                let game_markets: Vec<&Market> = markets.iter().map(|m| m).collect();

                if let Some(cluster_id) = self.game_id_to_fixture_cluster_key.get(&game_id) {
                    if let Some(&game_date) = self.cluster_id_to_date.get(cluster_id) {
                        self.clusters.entry(game_date).and_modify(|clusters_by_id| {
                            clusters_by_id
                                .entry(cluster_id.clone())
                                .and_modify(|cluster| {
                                    cluster.update_markets(game_id, game_markets)
                                });
                        });
                        arbitrages.append(
                            &mut self
                                .clusters
                                .get(&game_date)
                                .unwrap()
                                .get(cluster_id)
                                .unwrap()
                                .arbitrage_opportunites(),
                        );
                    } else {
                        println!("[update_markets] game_id not in any cluster")
                    }
                }
            });

        arbitrages
    }

    pub fn insert_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();

        games.into_iter().for_each(|game| {
            let game_id = game.id.clone();

            if let Some(cluster_id) = self.game_id_to_fixture_cluster_key.get(&game_id) {
                self.clusters.entry(game.date).and_modify(|clusters_by_id| {
                    clusters_by_id
                        .entry(cluster_id.clone())
                        .and_modify(|cluster| {
                            cluster.update_markets(game_id, game.markets().values().collect())
                        });
                });
                arbitrages.append(
                    &mut self
                        .clusters
                        .get(&game.date)
                        .unwrap()
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
        for (_, clusters_by_date) in &self.clusters {
            for (_, cluster) in clusters_by_date {
                writeln!(f, "{}", cluster)?;
            }
        }

        Ok(())
    }
}
