use std::sync::Arc;
use std::thread;

use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::domain::{ClusterService, Game, Market};
use crate::infrastructure::connectors::bridge_connector::BridgeConnector;
use crate::infrastructure::connectors::bwin_connector::BwinConnector;
use crate::infrastructure::connectors::lebull_connector::LeBullConnector;
use crate::shared::error::Result;

pub enum BookmakerEvent {
    Error,
    InsertGames(Vec<Game>),
    UpdateMarkets((String, Vec<Market>)),
}

pub trait Connector: Send + Sync {
    fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()>;
}

pub struct BookmakerScrapperService {
    cluster_service: Arc<RwLock<ClusterService>>,
    tx: Sender<BookmakerEvent>,
    rx: Receiver<BookmakerEvent>,
    connectors: Vec<Box<dyn Connector>>,
}

impl BookmakerScrapperService {
    pub fn new(cluster_service: Arc<RwLock<ClusterService>>) -> Self {
        let (tx, rx) = channel::<BookmakerEvent>(100);
        BookmakerScrapperService {
            cluster_service: cluster_service,
            tx,
            rx,
            connectors: vec![
                Box::new(BridgeConnector::new()),
                Box::new(LeBullConnector::new()),
                Box::new(BwinConnector::new()),
            ],
        }
    }

    pub async fn run(&mut self) {
        for connector in self.connectors.drain(..) {
            let tx = self.tx.clone();
            thread::spawn(move || {
                let _ = connector.start(tx);
            });
        }

        while let Some(bookmaker_event) = self.rx.recv().await {
            match bookmaker_event {
                BookmakerEvent::InsertGames(games) => {
                    self.cluster_service.write().await.insert_games(games);
                }
                BookmakerEvent::UpdateMarkets((game_id, markets)) => {
                    self.cluster_service
                        .write()
                        .await
                        .insert_markets(&game_id, markets);
                }
                BookmakerEvent::Error => (),
            }
        }
    }
}

#[cfg(test)]
mod tests;
