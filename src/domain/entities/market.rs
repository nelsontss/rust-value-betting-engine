#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Odd(pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub f32);

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResultMarket {
    id: String,
    home: Odd,
    draw: Odd,
    away: Odd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResultArbitrage {
    best_home: (Odd, String),
    best_draw: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoWayArbitrage {
    best_first: (Odd, String),
    best_second: (Odd, String),
    implied_probability_sum: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoWayLineArbitrage {
    line: Line,
    best_home: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeWayLineArbitrage {
    line: Line,
    best_home: (Odd, String),
    best_draw: (Odd, String),
    best_away: (Odd, String),
    implied_probability_sum: f64,
}

fn best_odd_with_id<I>(markets: I) -> (Odd, String)
where
    I: IntoIterator<Item = (Odd, String)>,
{
    markets
        .into_iter()
        .fold((Odd(f64::NEG_INFINITY), String::new()), |best, current| {
            if current.0.0 > best.0.0 {
                current
            } else {
                best
            }
        })
}

impl MatchResultMarket {
    pub fn as_arbitrage(markets: &[MatchResultMarket]) -> Option<MatchResultArbitrage> {
        if markets.is_empty() {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );

        let best_draw = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.draw, market.id.clone())),
        );

        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum =
            (1.0 / best_home.0.0) + (1.0 / best_draw.0.0) + (1.0 / best_away.0.0);

        if implied_probability_sum < 1.0 {
            Some(MatchResultArbitrage {
                best_home,
                best_draw,
                best_away,
                implied_probability_sum,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneylineMarket {
    id: String,
    pub home: Odd,
    pub away: Odd,
}

impl MoneylineMarket {
    pub fn new(id: &str, home: Odd, away: Odd) -> Self {
        Self {
            id: id.to_string(),
            home,
            away,
        }
    }

    pub fn as_arbitrage(markets: &[MoneylineMarket]) -> Option<TwoWayArbitrage> {
        if markets.is_empty() {
            return None;
        }

        let best_first = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );
        let best_second = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_first.0.0) + (1.0 / best_second.0.0);

        if implied_probability_sum < 1.0 {
            Some(TwoWayArbitrage {
                best_first,
                best_second,
                implied_probability_sum,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TotalMarket {
    id: String,
    pub line: Line,
    pub over: Odd,
    pub under: Odd,
}

impl TotalMarket {
    pub fn new(id: &str, line: Line, over: Odd, under: Odd) -> Self {
        Self {
            id: id.to_string(),
            line,
            over,
            under,
        }
    }

    pub fn as_arbitrage(markets: &[TotalMarket]) -> Option<TwoWayLineArbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line != line) {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.over, market.id.clone())),
        );
        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.under, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_home.0.0) + (1.0 / best_away.0.0);

        if implied_probability_sum < 1.0 {
            Some(TwoWayLineArbitrage {
                line,
                best_home,
                best_away,
                implied_probability_sum,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub draw: Odd,
    pub away: Odd,
}

impl HandicapMarket {
    pub fn as_arbitrage(markets: &[HandicapMarket]) -> Option<ThreeWayLineArbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line != line) {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );
        let best_draw = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.draw, market.id.clone())),
        );
        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum =
            (1.0 / best_home.0.0) + (1.0 / best_draw.0.0) + (1.0 / best_away.0.0);

        if implied_probability_sum < 1.0 {
            Some(ThreeWayLineArbitrage {
                line,
                best_home,
                best_draw,
                best_away,
                implied_probability_sum,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsianHandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub away: Odd,
}

impl AsianHandicapMarket {
    pub fn as_arbitrage(markets: &[AsianHandicapMarket]) -> Option<TwoWayLineArbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line != line) {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );
        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_home.0.0) + (1.0 / best_away.0.0);

        if implied_probability_sum < 1.0 {
            Some(TwoWayLineArbitrage {
                line,
                best_home,
                best_away,
                implied_probability_sum,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Market {
    MatchResult(MatchResultMarket),
    Moneyline(MoneylineMarket),
    Total(TotalMarket),
    Handicap(HandicapMarket),
    AsianHandicap(AsianHandicapMarket),
}

impl Market {
    pub fn as_match_result_arbitrage(
        markets: &[MatchResultMarket],
    ) -> Option<MatchResultArbitrage> {
        MatchResultMarket::as_arbitrage(markets)
    }

    pub fn as_moneyline_arbitrage(markets: &[MoneylineMarket]) -> Option<TwoWayArbitrage> {
        MoneylineMarket::as_arbitrage(markets)
    }

    pub fn as_total_arbitrage(markets: &[TotalMarket]) -> Option<TwoWayLineArbitrage> {
        TotalMarket::as_arbitrage(markets)
    }

    pub fn as_handicap_arbitrage(markets: &[HandicapMarket]) -> Option<ThreeWayLineArbitrage> {
        HandicapMarket::as_arbitrage(markets)
    }

    pub fn as_asian_handicap_arbitrage(
        markets: &[AsianHandicapMarket],
    ) -> Option<TwoWayLineArbitrage> {
        AsianHandicapMarket::as_arbitrage(markets)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketType {
    MatchResult,
    Moneyline,
    Total { line: i32 },
    Handicap { line: i32 },
    AsianHandicap { line: i32 },
}
