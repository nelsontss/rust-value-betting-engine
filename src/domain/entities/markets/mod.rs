pub mod asian_handicap;
pub mod double_chance;
pub mod handicap;
pub mod match_result;
pub mod moneyline;
pub mod total;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub f32);

impl Line {
    pub fn key(self) -> i32 {
        (self.0 * 100.0).round() as i32
    }
}

fn guaranteed_profit(scenarios: &[(f64, f64)]) -> f64 {
    let mut candidate_splits = vec![0.0, 1.0];

    for (index, left) in scenarios.iter().enumerate() {
        let left_slope = left.0 - left.1;
        let left_intercept = left.1 - 1.0;

        for right in scenarios.iter().skip(index + 1) {
            let right_slope = right.0 - right.1;
            let right_intercept = right.1 - 1.0;

            if (left_slope - right_slope).abs() < f64::EPSILON {
                continue;
            }

            let split = (right_intercept - left_intercept) / (left_slope - right_slope);

            if (0.0..=1.0).contains(&split) {
                candidate_splits.push(split);
            }
        }
    }

    candidate_splits
        .into_iter()
        .map(|split| {
            scenarios
                .iter()
                .map(|(first_multiplier, second_multiplier)| {
                    split * first_multiplier + (1.0 - split) * second_multiplier - 1.0
                })
                .fold(f64::INFINITY, f64::min)
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

fn floor_int(key: i32) -> i32 {
    key.div_euclid(100)
}

fn ceil_int(key: i32) -> i32 {
    if key.rem_euclid(100) == 0 {
        key.div_euclid(100)
    } else {
        key.div_euclid(100) + 1
    }
}

fn line_components(line: Line) -> Vec<i32> {
    let key = line.key();
    let fractional = key.abs() % 100;

    if fractional == 25 || fractional == 75 {
        let direction = if key.is_negative() { -25 } else { 25 };
        vec![key - direction, key + direction]
    } else {
        vec![key]
    }
}
