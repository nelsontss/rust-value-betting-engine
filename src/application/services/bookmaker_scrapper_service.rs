use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::domain::{ClusterService, Game, Market};
use crate::infrastructure::connectors::bridge_connector::BridgeConnector;
use crate::infrastructure::connectors::bwin_connector::BwinConnector;
use crate::infrastructure::connectors::lebull_connector::LeBullConnector;
use crate::infrastructure::connectors::polymarket_connector::PolymarketConnector;
use crate::shared::error::Result;

pub enum BookmakerEvent {
    Error,
    InsertGames(Vec<Game>),
    UpdateMarkets((String, Vec<Market>)),
}

pub enum ConnectorKind {
    Bridge(BridgeConnector),
    LeBull(LeBullConnector),
    Bwin(BwinConnector),
    Polymarket(PolymarketConnector),
}

impl ConnectorKind {
    pub async fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        match self {
            Self::Bridge(c) => c.start(sender).await,
            Self::LeBull(c) => c.start(sender).await,
            Self::Bwin(c) => c.start(sender).await,
            Self::Polymarket(c) => c.start(sender).await,
        }
    }
}

pub struct BookmakerScrapperService {
    cluster_service: Arc<ClusterService>,
    tx: Sender<BookmakerEvent>,
    rx: Receiver<BookmakerEvent>,
    connectors: Vec<ConnectorKind>,
}

impl BookmakerScrapperService {
    pub fn new(cluster_service: Arc<ClusterService>) -> Self {
        let (tx, rx) = channel::<BookmakerEvent>(100);
        BookmakerScrapperService {
            cluster_service,
            tx,
            rx,
            connectors: vec![
                ConnectorKind::Bridge(BridgeConnector::new()),
                ConnectorKind::LeBull(LeBullConnector::new()),
                ConnectorKind::Bwin(BwinConnector::new()),
                ConnectorKind::Polymarket(PolymarketConnector::new()),
            ],
        }
    }

    pub async fn run(&mut self) {
        for connector in self.connectors.drain(..) {
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let _ = connector.start(tx).await;
            });
        }

        while let Some(bookmaker_event) = self.rx.recv().await {
            match bookmaker_event {
                BookmakerEvent::InsertGames(games) => {
                    self.cluster_service.insert_games(games);
                }
                BookmakerEvent::UpdateMarkets((game_id, markets)) => {
                    self.cluster_service.insert_markets(&game_id, markets);
                }
                BookmakerEvent::Error => (),
            }
        }
    }
}

#[cfg(test)]
mod tests;
