use std::io::{Error, Read};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;

use chrono::DateTime;

use crate::application::services::bookmaker_scrapper_service::{BookmakerEvent, Connector};
use crate::infrastructure::bridge::BridgeMessage;
use crate::infrastructure::config::BridgeConfig;
use crate::infrastructure::parsers::parser_registry::ParserRegistry;

pub struct BridgeConnector {}

impl Connector for BridgeConnector {
    fn start(&self, sender: mpsc::Sender<BookmakerEvent>) -> Result<(), Error> {
        self.start_at(sender, BridgeConfig::SOCKET_PATH)
    }
}

impl BridgeConnector {
    pub fn new() -> Self {
        BridgeConnector {}
    }

    fn start_at(
        &self,
        sender: mpsc::Sender<BookmakerEvent>,
        socket_path: &str,
    ) -> Result<(), Error> {
        let registry = ParserRegistry::new();
        let mut stream = UnixStream::connect(socket_path)?;

        let mut len_buf = [0u8; 4];
        loop {
            stream.read_exact(&mut len_buf)?;
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            stream.read_exact(&mut buf)?;
            let msg = String::from_utf8_lossy(&buf);

            match serde_json::from_str::<BridgeMessage>(&msg) {
                Ok(bridge_message) => match bridge_message {
                    BridgeMessage::OddsUpdate {
                        platform,
                        timestamp,
                        data,
                    } => match registry.parse(&platform, data) {
                        Some(games) => {
                            println!(
                                "Inserting {} games from {:?} @ {:?}.",
                                games.len(),
                                platform,
                                DateTime::from_timestamp_millis(timestamp as i64)
                                    .unwrap_or_default()
                            );
                            let _ = sender.send(BookmakerEvent::InsertGames(games));
                        }
                        None => eprintln!("no parser registered for platform {:?}", platform),
                    },
                },
                Err(e) => eprintln!("failed to parse BridgeMessage: {}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests;
