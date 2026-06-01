use std::sync::mpsc;
use std::thread;

use crate::domain::{ClusterService, Game};
use crate::infrastructure::connectors::bridge_connector::BridgeConnector;

pub enum BookmakerEvent {
    Error,
    InsertGames(Vec<Game>),
}

pub struct BookmakerScrapperService {
    cluster_service: ClusterService,
    tx: mpsc::Sender<BookmakerEvent>,
    rx: mpsc::Receiver<BookmakerEvent>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl BookmakerScrapperService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<BookmakerEvent>();
        BookmakerScrapperService {
            cluster_service: ClusterService::new(),
            tx,
            rx,
            handles: vec![],
        }
    }

    pub fn run(&mut self) {
        let tx = self.tx.clone();
        let connector = BridgeConnector::new();
        self.handles.push(thread::spawn(move || {
            connector.start(tx);
        }));

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
