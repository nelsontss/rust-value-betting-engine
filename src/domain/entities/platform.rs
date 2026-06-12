use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Betano,
    LeBull,
    Bwin,
}
