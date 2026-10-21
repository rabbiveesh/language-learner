//! The review log: append-only, the source of truth. A card's schedule is replayed from it.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::card::CardId;

/// FSRS's four answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Grade {
    /// FSRS's rating, 1..=4.
    pub fn rating(self) -> u32 {
        match self {
            Grade::Again => 1,
            Grade::Hard => 2,
            Grade::Good => 3,
            Grade::Easy => 4,
        }
    }
}

/// What the app noticed about an answer, beyond the grade. Kept so later models (per-learner
/// timing thresholds, a custom FSRS variant) can learn from it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Signals {
    /// Time from showing the card to answering. Measured silently, never shown.
    pub response_ms: Option<u32>,
    /// What the learner typed, for typed cards.
    pub answer: Option<String>,
    /// The learner asked for a hint.
    pub hinted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Review {
    pub card_id: CardId,
    pub at: Timestamp,
    pub grade: Grade,
    pub signals: Signals,
}
