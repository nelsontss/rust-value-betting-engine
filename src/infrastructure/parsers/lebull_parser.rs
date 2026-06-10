use chrono::{DateTime, TimeDelta, Utc};
use serde_json::Value;
use std::collections::HashMap;

use crate::domain::entities::{Market, Platform};
use crate::domain::Game;

#[cfg(test)]
mod tests;

pub struct LeBullParser {}

impl LeBullParser {
    pub fn new() -> Self {
        LeBullParser {}
    }

    pub fn parse_data(data: Value) -> Vec<Game> {
        let leagues = match data.as_array() {
            Some(arr) => arr,
            None => return vec![],
        };

        let mut games = vec![];

        for league in leagues {
            let country = league.get("countryName").and_then(|v| v.as_str()).unwrap_or("");
            let competition = league.get("leagueName").and_then(|v| v.as_str()).unwrap_or("");

            let events = match league.get("games").and_then(|e| e.as_array()) {
                Some(events) => events,
                None => continue,
            };

            for event in events {
                let event_id = match event.get("eventId").and_then(|id| id.as_i64()) {
                    Some(id) => id.to_string(),
                    None => continue,
                };

                let team_a = match event.get("teamA").and_then(|n| n.as_str()) {
                    Some(name) => name,
                    None => continue,
                };

                let team_b = match event.get("teamB").and_then(|n| n.as_str()) {
                    Some(name) => name,
                    None => continue,
                };

                if event.get("isLive").and_then(|v| v.as_bool()).unwrap_or(false) {
                    continue;
                }

                let date = parse_date(event);

                let stake_types = match event.get("stakeTypes").and_then(|m| m.as_array()) {
                    Some(m) => m,
                    None => {
                let game = Game::new_with_id(
                    &event_id,
                    team_a,
                    team_b,
                    country,
                    competition,
                    date,
                    Platform::LeBull,
                    vec![],
                );
                if is_today_or_tomorrow(&game) {
                    games.push(game);
                }
                continue;
                    }
                };

                let mut markets = vec![];

                for st in stake_types {
                    let type_id = st.get("stakeTypeId").and_then(|t| t.as_i64()).unwrap_or(0);
                    let stakes = match st.get("stakes").and_then(|s| s.as_array()) {
                        Some(s) => s,
                        None => continue,
                    };

                    match type_id {
                        1 => parse_1x2(stakes, &event_id, &mut markets),
                        2 => parse_asian_handicap(stakes, &event_id, &mut markets),
                        3 => parse_total(stakes, &event_id, &mut markets),
                        26 | 274556 => parse_2way(stakes, &event_id, &mut markets),
                        37 => parse_double_chance(stakes, &event_id, &mut markets),
                        _ => {}
                    }
                }

                let game = Game::new_with_id(
                    &event_id,
                    team_a,
                    team_b,
                    country,
                    competition,
                    date,
                    Platform::LeBull,
                    markets,
                );
                if is_today_or_tomorrow(&game) {
                    games.push(game);
                }
            }
        }

        games
    }
}

fn is_today_or_tomorrow(game: &Game) -> bool {
    let today = Utc::now().date_naive();
    let tomorrow = today + chrono::Days::new(1);
    game.date.date() == today || game.date.date() == tomorrow
}

fn parse_date(event: &Value) -> chrono::NaiveDateTime {
    if let Some(date_str) = event.get("date").and_then(|d| d.as_str()) {
        if let Some(ts) = extract_ms(date_str) {
            return DateTime::UNIX_EPOCH.naive_utc() + TimeDelta::milliseconds(ts);
        }
    }
    if let Some(ts) = event.get("timestamp").and_then(|t| t.as_i64()) {
        return DateTime::UNIX_EPOCH.naive_utc() + TimeDelta::milliseconds(ts);
    }
    DateTime::UNIX_EPOCH.naive_utc()
}

fn extract_ms(s: &str) -> Option<i64> {
    if !s.starts_with("/Date(") || !s.ends_with(")/") {
        return None;
    }
    let inner = s.trim_start_matches("/Date(").trim_end_matches(")/");
    let digits: String = inner.chars().take_while(|c| c.is_ascii_digit()).collect();
    let ts = digits.parse::<i64>().ok()?;

    let rest = &inner[digits.len()..];
    if rest.is_empty() {
        return Some(ts);
    }

    let (sign, num_str) = if let Some(s) = rest.strip_prefix('+') {
        (1, s)
    } else if let Some(s) = rest.strip_prefix('-') {
        (-1, s)
    } else {
        return Some(ts);
    };

    if let Ok(num) = num_str.parse::<i64>() {
        let hours = num / 100;
        let minutes = num % 100;
        let offset_ms = sign * (hours * 3600 + minutes * 60) * 1000;
        Some(ts + offset_ms)
    } else {
        Some(ts)
    }
}

fn stake_value(stakes: &[Value], code: i64) -> Option<f64> {
    stakes
        .iter()
        .find(|s| s.get("stakeCode").and_then(|c| c.as_i64()) == Some(code))
        .and_then(|s| s.get("betFactor").and_then(|b| b.as_f64()))
}

fn parse_1x2(stakes: &[Value], event_id: &str, markets: &mut Vec<Market>) {
    let home = stake_value(stakes, 1).unwrap_or(0.0);
    let draw = stake_value(stakes, 2).unwrap_or(0.0);
    let away = stake_value(stakes, 3).unwrap_or(0.0);

    if home > 0.0 && draw > 0.0 && away > 0.0 {
        if let Ok(m) = Market::match_result(event_id, home, draw, away) {
            markets.push(m);
        }
    }
}

fn parse_2way(stakes: &[Value], event_id: &str, markets: &mut Vec<Market>) {
    let home = stake_value(stakes, 1).unwrap_or(0.0);
    let away = stake_value(stakes, 2).unwrap_or(0.0);

    if home > 0.0 && away > 0.0 {
        if let Ok(m) = Market::moneyline(event_id, home, away) {
            markets.push(m);
        }
    }
}

fn parse_double_chance(stakes: &[Value], event_id: &str, markets: &mut Vec<Market>) {
    let home_or_draw = stake_value(stakes, 1).unwrap_or(0.0);
    let home_or_away = stake_value(stakes, 2).unwrap_or(0.0);
    let draw_or_away = stake_value(stakes, 3).unwrap_or(0.0);

    if home_or_draw > 0.0 && home_or_away > 0.0 && draw_or_away > 0.0 {
        if let Ok(m) = Market::double_chance(event_id, home_or_draw, home_or_away, draw_or_away) {
            markets.push(m);
        }
    }
}

fn parse_total(stakes: &[Value], event_id: &str, markets: &mut Vec<Market>) {
    let mut lines: HashMap<i32, (f64, f64)> = HashMap::new();

    for stake in stakes {
        let code = stake.get("stakeCode").and_then(|c| c.as_i64()).unwrap_or(0);
        let argument = stake
            .get("stakeArgument")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);
        let bet = stake
            .get("betFactor")
            .and_then(|b| b.as_f64())
            .unwrap_or(0.0);
        let key = (argument * 100.0).round() as i32;

        let entry = lines.entry(key).or_insert((0.0, 0.0));
        if code == 1 {
            entry.0 = bet;
        } else if code == 2 {
            entry.1 = bet;
        }
    }

    for (key, (over, under)) in lines {
        if over > 0.0 && under > 0.0 {
            if let Ok(m) = Market::total(event_id, key as f32 / 100.0, over, under) {
                markets.push(m);
            }
        }
    }
}

fn parse_asian_handicap(stakes: &[Value], event_id: &str, markets: &mut Vec<Market>) {
    let mut lines: HashMap<i32, (f64, f64)> = HashMap::new();

    for stake in stakes {
        let code = stake.get("stakeCode").and_then(|c| c.as_i64()).unwrap_or(0);
        let argument = stake
            .get("stakeArgument")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);
        let bet = stake
            .get("betFactor")
            .and_then(|b| b.as_f64())
            .unwrap_or(0.0);

        let normalized = if code == 2 { -argument } else { argument };
        let key = (normalized * 100.0).round() as i32;

        let entry = lines.entry(key).or_insert((0.0, 0.0));
        if code == 1 {
            entry.0 = bet;
        } else {
            entry.1 = bet;
        }
    }

    for (key, (home, away)) in lines {
        if home > 0.0 && away > 0.0 {
            if let Ok(m) = Market::asian_handicap(event_id, key as f32 / 100.0, home, away) {
                markets.push(m);
            }
        }
    }
}
