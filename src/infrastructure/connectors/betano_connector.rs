use chrono::{DateTime, TimeDelta};
use serde_json::Value;

use crate::domain::entities::{Market, Platform};
use crate::domain::Game;

pub struct BetanoParser {}

impl BetanoParser {
    pub fn new() -> Self {
        BetanoParser {}
    }

    pub fn parse_data(data: Value) -> Vec<Game> {
        let blocks = match data.get("blocks").and_then(|b| b.as_array()) {
            Some(blocks) => blocks,
            None => return vec![],
        };

        let mut games = vec![];

        for block in blocks {
            let competition = block.get("name").and_then(|v| v.as_str()).unwrap_or("");

            let events = match block.get("events").and_then(|e| e.as_array()) {
                Some(events) => events,
                None => continue,
            };

            for event in events {
                let event_id = match event.get("id").and_then(|id| id.as_str()) {
                    Some(id) => id,
                    None => continue,
                };

                let event_name = match event.get("name").and_then(|n| n.as_str()) {
                    Some(name) => name,
                    None => continue,
                };

                let parts: Vec<&str> = event_name.split(" - ").collect();
                if parts.len() != 2 {
                    continue;
                }
                let home_team = parts[0].trim();
                let away_team = parts[1].trim();

                let country = event
                    .get("regionName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let start_time_ms = match event.get("startTime").and_then(|t| t.as_i64()) {
                    Some(ts) => ts,
                    None => continue,
                };
                let date =
                    DateTime::UNIX_EPOCH.naive_utc() + TimeDelta::milliseconds(start_time_ms);

                let markets = match event.get("markets").and_then(|m| m.as_array()) {
                    Some(m) => m,
                    None => {
                        games.push(Game::new_with_id(
                            event_id,
                            home_team,
                            away_team,
                            country,
                            competition,
                            date,
                            Platform::Betano,
                            vec![],
                        ));
                        continue;
                    }
                };

                let mut parsed_markets = vec![];
                for market in markets {
                    let type_id = market.get("typeId").and_then(|t| t.as_i64()).unwrap_or(0);
                    let market_id = market.get("id").and_then(|id| id.as_str()).unwrap_or("");
                    let handicap = market
                        .get("handicap")
                        .and_then(|h| h.as_f64())
                        .unwrap_or(0.0);
                    let selections = match market.get("selections").and_then(|s| s.as_array()) {
                        Some(s) => s,
                        None => continue,
                    };

                    match type_id {
                        1 => {
                            if selections.len() >= 3 {
                                let home = selections[0]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                let draw = selections[1]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                let away = selections[2]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                if let Ok(m) = Market::match_result(market_id, home, draw, away) {
                                    parsed_markets.push(m);
                                }
                            }
                        }
                        13 => {
                            if selections.len() >= 2 {
                                let over = selections[0]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                let under = selections[1]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                if let Ok(m) =
                                    Market::total(market_id, handicap as f32, over, under)
                                {
                                    parsed_markets.push(m);
                                }
                            }
                        }
                        10 | 15 => {
                            if selections.len() >= 2 {
                                let home = selections[0]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                let away = selections[1]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                if let Ok(m) = Market::moneyline(market_id, home, away) {
                                    parsed_markets.push(m);
                                }
                            }
                        }
                        14 => {
                            if selections.len() >= 2 {
                                let over = selections[0]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                let under = selections[1]
                                    .get("price")
                                    .and_then(|p| p.as_f64())
                                    .unwrap_or(0.0);
                                if let Ok(m) =
                                    Market::total(market_id, handicap as f32, over, under)
                                {
                                    parsed_markets.push(m);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                games.push(Game::new_with_id(
                    event_id,
                    home_team,
                    away_team,
                    country,
                    competition,
                    date,
                    Platform::Betano,
                    parsed_markets,
                ));
            }
        }

        games
    }
}
