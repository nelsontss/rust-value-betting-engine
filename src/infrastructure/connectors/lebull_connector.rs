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
    const LIVE_URL: &str = "https://betting-platform.prod.sbteam.xyz/sports/1/leagues/inplay?languageId=2&stakeTypes=%5B1%2C%2080%2C%20356%2C%20702%2C%20176415%2C%20183254%2C%20217797%2C%20357318%2C%202%2C%203%2C%2026%2C%2037%2C%20545%2C%20144%2C%20724%2C%20274556%2C%20313638%2C%20313639%5D&isStakeGrouped=true&timeZone=1&checkIsActive=true&setParameterOrder=false&getMainMatch=false";

    pub fn new() -> Self {
        LeBullConnector {
            client: reqwest::Client::new(),
        }
    }

    pub async fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let registry = ParserRegistry::new();

        loop {
            if let Some(json) = self.fetch_json(Self::UPCOMING_URL).await {
                if let Some(games) = registry.parse(&Platform::LeBull, json) {
                    let _ = sender.send(BookmakerEvent::InsertGames(games)).await;
                }
            }
            if let Some(json) = self.fetch_json(Self::LIVE_URL).await {
                if let Some(games) = registry.parse(&Platform::LeBull, json) {
                    let _ = sender.send(BookmakerEvent::InsertGames(games)).await;
                }
            }

            sleep(Duration::from_secs(
                LeBullConnector::POLLING_INTERVAL_SECONDS,
            ))
            .await;
        }
    }

    async fn fetch_json(&self, url: &str) -> Option<serde_json::Value> {
        match self
            .client
            .get(url)
            .header("x-auth-tenant-id", Self::X_AUTH_TENANT_ID)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => Some(json),
                Err(_) => {
                    eprintln!("Error reading body json from {url}");
                    None
                }
            },
            Err(e) => {
                eprintln!("Error making polling request to lebull ({url}): {:?}", e);
                None
            }
        }
    }
}
