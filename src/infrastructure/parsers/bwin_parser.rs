use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

use crate::domain::Game;
use crate::domain::entities::{Market, Platform};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRFrame {
    #[serde(rename = "type")]
    pub msg_type: u8,
    pub target: Option<String>,
    #[serde(rename = "invocationId")]
    pub invocation_id: Option<String>,
    pub arguments: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwitchedFixture {
    #[serde(rename = "preMatchId")]
    pub pre_match_id: String,
    #[serde(rename = "inPlayId")]
    pub in_play_id: String,
}

#[derive(Debug)]
pub enum BwinWSEvent {
    Ping,
    ConnectionAck {
        connection_id: String,
    },
    MainToLiveUpdate {
        switched_fixtures: Vec<SwitchedFixture>,
    },
    OptionMarketUpdate {
        payload: Value,
        fixture_id: String,
    },
    OptionMarketDelete {
        market_id: i64,
        fixture_id: String,
    },
    FixtureUpdate {
        fixture_id: String,
        stage: String,
    },
    ScoreboardSlim {
        scoreboard: Value,
        fixture_id: String,
    },
    Subscribe {
        topics: Vec<String>,
    },
}

impl BwinWSEvent {
    pub fn from_frame(frame: SignalRFrame) -> Option<Self> {
        match frame.msg_type {
            6 => Some(Self::Ping),
            1 => {
                let args = frame.arguments.as_ref()?;
                let data = args.first()?;
                let msg_type = data.get("messageType")?.as_str()?;
                let payload = data.get("payload")?;
                match msg_type {
                    "ConnectionAck" => {
                        let connection_id = payload
                            .get("connectionId")
                            .and_then(|v| v.as_str())?
                            .to_string();
                        Some(Self::ConnectionAck { connection_id })
                    }
                    "MainToLiveUpdate" => {
                        let switched = serde_json::from_value(payload.clone()).ok()?;
                        Some(Self::MainToLiveUpdate {
                            switched_fixtures: switched,
                        })
                    }
                    "OptionMarketUpdate" => {
                        let fixture_id = data
                            .get("fixtureId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(Self::OptionMarketUpdate {
                            payload: payload.clone(),
                            fixture_id,
                        })
                    }
                    "OptionMarketDelete" => {
                        let market_id = payload
                            .get("marketId")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let fixture_id = payload
                            .get("fixtureId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(Self::OptionMarketDelete {
                            market_id,
                            fixture_id,
                        })
                    }
                    "FixtureUpdate" => {
                        let fixture_id = data
                            .get("fixtureId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let stage = payload
                            .get("stage")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(Self::FixtureUpdate { fixture_id, stage })
                    }
                    "ScoreboardSlim" => {
                        let fixture_id = data
                            .get("fixtureId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(Self::ScoreboardSlim {
                            scoreboard: payload.clone(),
                            fixture_id,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn subscribe(topics: Vec<String>) -> SignalRFrame {
        SignalRFrame {
            msg_type: 1,
            target: Some("Subscribe".into()),
            invocation_id: Some("0".into()),
            arguments: Some(vec![serde_json::json!({ "topics": topics })]),
        }
    }
}

pub struct BwinParser {}

impl BwinParser {
    pub fn new() -> Self {
        BwinParser {}
    }

    pub fn parse_ws_event(data: &str) -> Option<BwinWSEvent> {
        let frame: SignalRFrame = serde_json::from_str(data).ok()?;
        BwinWSEvent::from_frame(frame)
    }

    pub fn parse_option_market_update(data: Value) -> Vec<Market> {
        let om = match data.get("optionMarket") {
            Some(om) => om,
            None => return vec![],
        };
        Self::parse_single_market(om).into_iter().collect()
    }

    fn parse_single_market(om: &Value) -> Option<Market> {
        let status = om.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status != "Visible" {
            return None;
        }

        let period = Self::get_param(om, "Period");
        if period.as_deref() != Some("RegularTime") {
            return None;
        }

        let market_type = Self::get_param(om, "MarketType");
        let options = match om.get("options").and_then(|o| o.as_array()) {
            Some(o) => o,
            None => return None,
        };

        let market_id = om.get("id").and_then(|id| id.as_i64()).unwrap_or(0);

        match market_type.as_deref() {
            Some("3way") => {
                if options.len() < 3 {
                    return None;
                }
                let home = Self::odds(&options[0]);
                let draw = Self::odds(&options[1]);
                let away = Self::odds(&options[2]);
                match (home, draw, away) {
                    (Some(h), Some(d), Some(a)) => {
                        Market::match_result(&market_id.to_string(), h, d, a).ok()
                    }
                    _ => None,
                }
            }
            Some("DoubleChance") => {
                if options.len() < 3 {
                    return None;
                }
                let home_or_draw = Self::odds(&options[0]);
                let home_or_away = Self::odds(&options[2]);
                let draw_or_away = Self::odds(&options[1]);
                match (home_or_draw, home_or_away, draw_or_away) {
                    (Some(hd), Some(ha), Some(da)) => {
                        Market::double_chance(&market_id.to_string(), hd, ha, da).ok()
                    }
                    _ => None,
                }
            }
            Some("Over/Under") => {
                if options.len() < 2 {
                    return None;
                }
                let line = Self::get_param(om, "DecimalValue")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let over = Self::odds(&options[0]);
                let under = Self::odds(&options[1]);
                match (over, under) {
                    (Some(o), Some(u)) => Market::total(&market_id.to_string(), line, o, u).ok(),
                    _ => None,
                }
            }
            Some("Handicap") => {
                if options.len() < 3 {
                    return None;
                }
                let line = Self::get_param(om, "Handicap")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let home = Self::odds(&options[0]);
                let draw = Self::odds(&options[1]);
                let away = Self::odds(&options[2]);
                match (home, draw, away) {
                    (Some(h), Some(d), Some(a)) => {
                        Market::handicap(&market_id.to_string(), line, h, d, a).ok()
                    }
                    _ => None,
                }
            }
            Some("DrawNoBet") => {
                if options.len() < 2 {
                    return None;
                }
                let home = Self::odds(&options[0]);
                let away = Self::odds(&options[1]);
                match (home, away) {
                    (Some(h), Some(a)) => Market::moneyline(&market_id.to_string(), h, a).ok(),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn parse_data(data: Value) -> Vec<Game> {
        let fixtures = match data.get("fixtures").and_then(|f| f.as_array()) {
            Some(f) => f,
            None => return vec![],
        };

        let mut games = vec![];

        for fixture in fixtures {
            let fixture_id = match fixture.get("id").and_then(|id| id.as_str()) {
                Some(id) => id,
                None => continue,
            };

            let participants = match fixture.get("participants").and_then(|p| p.as_array()) {
                Some(p) => p,
                None => continue,
            };

            let home_team = participants
                .iter()
                .find(|p| p["properties"]["type"] == "HomeTeam")
                .and_then(|p| p["name"]["value"].as_str());

            let away_team = participants
                .iter()
                .find(|p| p["properties"]["type"] == "AwayTeam")
                .and_then(|p| p["name"]["value"].as_str());

            let (home_team, away_team) = match (home_team, away_team) {
                (Some(h), Some(a)) => (h, a),
                _ => continue,
            };

            let competition = fixture
                .get("competition")
                .and_then(|c| c["name"]["value"].as_str())
                .unwrap_or("");

            let country = fixture
                .get("region")
                .and_then(|r| r["name"]["value"].as_str())
                .unwrap_or("");

            let date = match fixture
                .get("startDate")
                .and_then(|d| d.as_str())
                .and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%SZ").ok())
            {
                Some(d) => d,
                None => continue,
            };

            let option_markets = match fixture.get("optionMarkets").and_then(|m| m.as_array()) {
                Some(m) => m,
                None => {
                    games.push(Game::new_with_id(
                        fixture_id,
                        home_team,
                        away_team,
                        country,
                        competition,
                        date,
                        Platform::Bwin,
                        vec![],
                    ));
                    continue;
                }
            };

            let mut markets = vec![];
            for om in option_markets {
                if let Some(market) = Self::parse_single_market(om) {
                    markets.push(market);
                }
            }

            games.push(Game::new_with_id(
                fixture_id,
                home_team,
                away_team,
                country,
                competition,
                date,
                Platform::Bwin,
                markets,
            ));
        }

        games
    }

    fn get_param(om: &Value, key: &str) -> Option<String> {
        om.get("parameters")
            .and_then(|params| params.as_array())
            .and_then(|params| {
                params
                    .iter()
                    .find(|p| p["key"].as_str() == Some(key))
                    .and_then(|p| p["value"].as_str().map(|s| s.to_string()))
            })
    }

    fn odds(option: &Value) -> Option<f64> {
        option
            .get("price")
            .and_then(|p| p.get("odds"))
            .and_then(|o| if o.is_null() { None } else { o.as_f64() })
    }
}
