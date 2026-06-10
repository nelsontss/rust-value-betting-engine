use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Odd(f64);

impl Odd {
    pub fn new(value: f64) -> Result<Self, OddError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(OddError::NonPositive(value));
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OddError {
    NonPositive(f64),
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

    #[test]
    fn odd_rejects_initialization_with_non_positive_doubles() {
        assert_eq!(OddError::NonPositive(-1.0), Odd::new(-1.0).unwrap_err());
        assert_eq!(OddError::NonPositive(0.0), Odd::new(0.0).unwrap_err());
    }
}
