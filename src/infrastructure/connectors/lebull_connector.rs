use std::thread;
use std::time::Duration;

use tokio::sync::mpsc::Sender;

use crate::application::services::bookmaker_scrapper_service::{BookmakerEvent, Connector};
use crate::domain::Platform;
use crate::infrastructure::parsers::parser_registry::ParserRegistry;
use crate::shared::error::Result;

pub struct LeBullConnector {}

impl Connector for LeBullConnector {
    fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let registry = ParserRegistry::new();

        loop {
            match ureq::get(LeBullConnector::UPCOMING_URL)
                .header("x-auth-tenant-id", LeBullConnector::X_AUTH_TENANT_ID)
                .call()
            {
                Ok(response) => {
                    if let Ok(json) = response.into_body().read_json::<serde_json::Value>() {
                        match registry.parse(&Platform::LeBull, json) {
                            Some(games) => {
                                let _ = sender.blocking_send(BookmakerEvent::InsertGames(games));
                            }
                            None => eprintln!("no parser registered for platform LeBull"),
                        }
                    } else {
                        eprintln!("Error reading body json");
                    }
                }
                Err(e) => {
                    eprintln!("Error making polling request to lebull: {:?}", e)
                }
            }

            thread::sleep(Duration::from_secs(
                LeBullConnector::POLLING_INTERVAL_SECONDS,
            ));
        }
    }
}

impl LeBullConnector {
    const POLLING_INTERVAL_SECONDS: u64 = 2;
    const X_AUTH_TENANT_ID: &str = "126dc7bf-288b-4f72-9536-3aa54648c0f4";
    const UPCOMING_URL: &str = "https://betting-platform.prod.sbteam.xyz/sports/1/leagues/upcoming?languageId=2&stakeTypes=%5B1%2C2%2C3%2C26%2C37%2C274556%5D&isStakeGrouped=true&timeZone=1&checkIsActive=true&setParameterOrder=false&getMainMatch=false";

    pub fn new() -> Self {
        LeBullConnector {}
    }
}
