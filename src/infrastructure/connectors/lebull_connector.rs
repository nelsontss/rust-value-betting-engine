use std::time::Duration;

use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::application::services::bookmaker_scrapper_service::BookmakerEvent;
use crate::domain::Platform;
use crate::infrastructure::parsers::parser_registry::ParserRegistry;
use crate::shared::error::Result;

pub struct LeBullConnector {
    client: reqwest::Client,
}

impl LeBullConnector {
    const POLLING_INTERVAL_SECONDS: u64 = 2;
    const X_AUTH_TENANT_ID: &str = "126dc7bf-288b-4f72-9536-3aa54648c0f4";
    const UPCOMING_URL: &str = "https://betting-platform.prod.sbteam.xyz/sports/1/leagues/upcoming?leagueTimeFilter=14&languageId=2&stakeTypes=%5B1%2C%2080%2C%20356%2C%20702%2C%20176415%2C%20183254%2C%20217797%2C%20357318%2C%202%2C%203%2C%2026%2C%2037%2C%20545%2C%20144%2C%20724%2C%20274556%2C%20313638%2C%20313639%5D&isStakeGrouped=true&timeZone=1&checkIsActive=true&setParameterOrder=false&getMainMatch=false";

    pub fn new() -> Self {
        LeBullConnector {
            client: reqwest::Client::new(),
        }
    }

    pub async fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let registry = ParserRegistry::new();

        loop {
            match self
                .client
                .get(LeBullConnector::UPCOMING_URL)
                .header("x-auth-tenant-id", LeBullConnector::X_AUTH_TENANT_ID)
                .send()
                .await
            {
                Ok(response) => {
                    if let Ok(json) = response.json::<serde_json::Value>().await {
                        match registry.parse(&Platform::LeBull, json) {
                            Some(games) => {
                                let _ = sender.send(BookmakerEvent::InsertGames(games)).await;
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

            sleep(Duration::from_secs(
                LeBullConnector::POLLING_INTERVAL_SECONDS,
            ))
            .await;
        }
    }
}
