use super::market::{Line, Odd};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct StakeRecommendation {
    pub outcome: &'static str,
    pub market_id: String,
    pub odd: Odd,
    pub stake: f64,
    pub payout: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StakeDistribution {
    pub total_stake: f64,
    pub guaranteed_payout: f64,
    pub guaranteed_profit: f64,
    pub roi: f64,
    pub stakes: Vec<StakeRecommendation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResultArbitrage {
    best_home: (Odd, String),
    best_draw: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

impl MatchResultArbitrage {
    pub fn new(
        best_home: (Odd, String),
        best_draw: (Odd, String),
        best_away: (Odd, String),
        implied_probability_sum: f64,
    ) -> Self {
        Self {
            best_home,
            best_draw,
            best_away,
            implied_probability_sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoWayArbitrage {
    best_first: (Odd, String),
    best_second: (Odd, String),
    implied_probability_sum: f64,
}

impl TwoWayArbitrage {
    pub fn new(
        best_first: (Odd, String),
        best_second: (Odd, String),
        implied_probability_sum: f64,
    ) -> Self {
        Self {
            best_first,
            best_second,
            implied_probability_sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoWayLineArbitrage {
    line: Line,
    best_home: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

impl TwoWayLineArbitrage {
    pub fn new(
        line: Line,
        best_home: (Odd, String),
        best_away: (Odd, String),
        implied_probability_sum: f64,
    ) -> Self {
        Self {
            line,
            best_home,
            best_away,
            implied_probability_sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeWayLineArbitrage {
    line: Line,
    best_home: (Odd, String),
    best_draw: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

impl ThreeWayLineArbitrage {
    pub fn new(
        line: Line,
        best_home: (Odd, String),
        best_draw: (Odd, String),
        best_away: (Odd, String),
        implied_probability_sum: f64,
    ) -> Self {
        Self {
            line,
            best_home,
            best_draw,
            best_away,
            implied_probability_sum,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arbitrage {
    MatchResultArbitrage(MatchResultArbitrage),
    TwoWayArbitrage(TwoWayArbitrage),
    TwoWayLineArbitrage(TwoWayLineArbitrage),
    ThreeWayLineArbitrage(ThreeWayLineArbitrage),
}

impl Arbitrage {
    pub fn implied_probability_sum(&self) -> f64 {
        match self {
            Arbitrage::MatchResultArbitrage(arbitrage) => arbitrage.implied_probability_sum,
            Arbitrage::TwoWayArbitrage(arbitrage) => arbitrage.implied_probability_sum,
            Arbitrage::TwoWayLineArbitrage(arbitrage) => arbitrage.implied_probability_sum,
            Arbitrage::ThreeWayLineArbitrage(arbitrage) => arbitrage.implied_probability_sum,
        }
    }

    pub fn roi(&self) -> f64 {
        (1.0 / self.implied_probability_sum()) - 1.0
    }

    pub fn guaranteed_payout(&self, total_stake: f64) -> Option<f64> {
        self.valid_total_stake(total_stake)
            .then(|| total_stake / self.implied_probability_sum())
    }

    pub fn guaranteed_profit(&self, total_stake: f64) -> Option<f64> {
        self.guaranteed_payout(total_stake)
            .map(|payout| payout - total_stake)
    }

    pub fn stake_distribution(&self, total_stake: f64) -> Option<StakeDistribution> {
        if !self.valid_total_stake(total_stake) {
            return None;
        }

        let implied_probability_sum = self.implied_probability_sum();
        let guaranteed_payout = total_stake / implied_probability_sum;
        let guaranteed_profit = guaranteed_payout - total_stake;
        let roi = guaranteed_profit / total_stake;

        let stakes = self
            .outcomes()
            .into_iter()
            .map(|(outcome, (odd, market_id))| {
                let stake = total_stake * ((1.0 / odd.get()) / implied_probability_sum);

                StakeRecommendation {
                    outcome,
                    market_id: market_id.clone(),
                    odd: *odd,
                    stake,
                    payout: stake * odd.get(),
                }
            })
            .collect();

        Some(StakeDistribution {
            total_stake,
            guaranteed_payout,
            guaranteed_profit,
            roi,
            stakes,
        })
    }

    fn valid_total_stake(&self, total_stake: f64) -> bool {
        total_stake.is_finite()
            && total_stake > 0.0
            && self.implied_probability_sum().is_finite()
            && self.implied_probability_sum() > 0.0
    }

    fn outcomes(&self) -> Vec<(&'static str, &(Odd, String))> {
        match self {
            Arbitrage::MatchResultArbitrage(arbitrage) => vec![
                ("home", &arbitrage.best_home),
                ("draw", &arbitrage.best_draw),
                ("away", &arbitrage.best_away),
            ],
            Arbitrage::TwoWayArbitrage(arbitrage) => vec![
                ("first", &arbitrage.best_first),
                ("second", &arbitrage.best_second),
            ],
            Arbitrage::TwoWayLineArbitrage(arbitrage) => vec![
                ("home", &arbitrage.best_home),
                ("away", &arbitrage.best_away),
            ],
            Arbitrage::ThreeWayLineArbitrage(arbitrage) => vec![
                ("home", &arbitrage.best_home),
                ("draw", &arbitrage.best_draw),
                ("away", &arbitrage.best_away),
            ],
        }
    }
}
