use serde::{Deserialize, Serialize};

use crate::domain::entities::Platform;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeMessage {
    #[serde(rename = "odds_update")]
    OddsUpdate {
        platform: Platform,
        timestamp: u64,
        data: serde_json::Value,
    },
}
