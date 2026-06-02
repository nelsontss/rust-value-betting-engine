use std::io::Error;
use std::sync::mpsc;
use std::thread;

use crate::domain::{ClusterService, Game};
use crate::infrastructure::connectors::bridge_connector::BridgeConnector;
use crate::infrastructure::connectors::lebull_connector::LeBullConnector;

pub enum BookmakerEvent {
    Error,
    InsertGames(Vec<Game>),
}

pub trait Connector: Send + Sync {
    fn start(&self, sender: mpsc::Sender<BookmakerEvent>) -> Result<(), Error>;
}

pub struct BookmakerScrapperService {
    cluster_service: ClusterService,
    tx: mpsc::Sender<BookmakerEvent>,
    rx: mpsc::Receiver<BookmakerEvent>,
    connectors: Vec<Box<dyn Connector>>,
}

impl BookmakerScrapperService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<BookmakerEvent>();
        BookmakerScrapperService {
            cluster_service: ClusterService::new(),
            tx,
            rx,
            connectors: vec![
                Box::new(BridgeConnector::new()),
                Box::new(LeBullConnector::new()),
            ],
        }
    }

    pub fn run(&mut self) {
        for connector in self.connectors.drain(..) {
            let tx = self.tx.clone();
            thread::spawn(move || {
                let _ = connector.start(tx);
            });
        }

        for bookmaker_event in &self.rx {
            match bookmaker_event {
                BookmakerEvent::InsertGames(games) => {
                    self.cluster_service.insert_games(games);
                }
                BookmakerEvent::Error => (),
            }
        }
    }
}

#[cfg(test)]
mod tests;
