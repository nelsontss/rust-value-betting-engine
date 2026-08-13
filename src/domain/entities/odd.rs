use std::cmp::Ordering;

use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Odd(f64, Decimal);

impl Odd {
    pub fn new(value: f64) -> Result<Self, OddError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(OddError::NonPositive(value));
        }

        let prob = Decimal::from_f64(1.0 / value).ok_or(OddError::NonPositive(value))?;
        Ok(Self(value, prob))
    }

    pub fn new_from_prob(value: Decimal) -> Result<Self, OddError> {
        if value <= Decimal::ZERO || value > Decimal::ONE {
            return Err(OddError::InvalidProbability(value));
        }

        let odd = Decimal::ONE / value;
        let odd = odd.to_f64().ok_or(OddError::InvalidProbability(value))?;
        Ok(Self(odd, value))
    }

    pub fn get(&self) -> f64 {
        self.0
    }

    pub fn get_implied_probability(&self) -> f64 {
        1.0 / self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OddError {
    NonPositive(f64),
    InvalidProbability(Decimal),
}

pub fn best_odd_with_id<I>(markets: I) -> (Odd, String)
where
    I: IntoIterator<Item = (Odd, String)>,
{
    markets
        .into_iter()
        .max_by(|left, right| {
            left.0
                .get()
                .partial_cmp(&right.0.get())
                .unwrap_or(Ordering::Equal)
        })
        .expect("markets must be non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;
    use polymarket_client_sdk_v2::types::dec;

    #[test]
    fn odd_rejects_initialization_with_non_positive_doubles() {
        assert!(matches!(Odd::new(-1.0), Err(OddError::NonPositive(v)) if v == -1.0));
        assert!(matches!(Odd::new(0.0), Err(OddError::NonPositive(v)) if v == 0.0));
        assert!(matches!(Odd::new(f64::NAN), Err(OddError::NonPositive(v)) if v.is_nan()));
    }

    #[test]
    fn odd_rejects_initialization_with_invalid_probabilities() {
        assert_eq!(
            OddError::InvalidProbability(Decimal::ZERO),
            Odd::new_from_prob(Decimal::ZERO).unwrap_err()
        );
        assert_eq!(
            OddError::InvalidProbability(dec!(1.5)),
            Odd::new_from_prob(dec!(1.5)).unwrap_err()
        );
    }

    #[test]
    fn odd_round_trips_between_decimal_and_probability() {
        let odd = Odd::new(2.5).unwrap();
        let prob = odd.1;
        let reconstructed = Odd::new_from_prob(prob).unwrap();
        assert_eq!(odd.0, reconstructed.0);
    }
}
