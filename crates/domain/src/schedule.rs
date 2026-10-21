//! When a card comes back: FSRS, replayed over the card's review log.

use fsrs::{FSRS, MemoryState};
use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::review::{Grade, Review};

#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error("FSRS rejected the parameters or a state: {0}")]
    Fsrs(#[from] fsrs::FSRSError),
    #[error("reviews must be in time order")]
    OutOfOrder,
}

/// Where a card stands after replaying its reviews.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardState {
    pub reps: u32,
    pub lapses: u32,
    /// FSRS stability in days, once reviewed.
    pub stability: Option<f32>,
    /// FSRS difficulty 1..10, once reviewed.
    pub difficulty: Option<f32>,
    pub last_review: Option<Timestamp>,
    /// When the card is due. `None` for a new card: new cards are introduced by the adaptive
    /// engine, not by a date.
    pub due: Option<Timestamp>,
}

/// FSRS with one set of parameters (one per card kind, eventually fitted to the learner's own
/// log with `fsrs::compute_parameters`).
pub struct Scheduler {
    fsrs: FSRS,
    pub desired_retention: f32,
    /// How soon a forgotten card comes back the same day.
    // TODO(design): proper same-day relearning steps, and a "learning day" that starts at a
    // local hour (e.g. 4am) instead of counting 24-hour periods.
    pub relearn_after: SignedDuration,
}

const DAY: i64 = 86_400;

impl Scheduler {
    /// `parameters` empty = FSRS's defaults for the average learner.
    pub fn new(parameters: &[f32], desired_retention: f32) -> Result<Self, ScheduleError> {
        Ok(Scheduler {
            fsrs: FSRS::new(parameters)?,
            desired_retention,
            relearn_after: SignedDuration::from_mins(10),
        })
    }

    /// Replay `reviews` (one card's, oldest first).
    pub fn replay(&self, reviews: &[Review]) -> Result<CardState, ScheduleError> {
        let mut state = CardState {
            reps: 0,
            lapses: 0,
            stability: None,
            difficulty: None,
            last_review: None,
            due: None,
        };
        let mut memory: Option<MemoryState> = None;
        for r in reviews {
            let elapsed_days = match state.last_review {
                Some(last) if r.at < last => return Err(ScheduleError::OutOfOrder),
                Some(last) => (r.at.duration_since(last).as_secs() / DAY) as u32,
                None => 0,
            };
            let next = self
                .fsrs
                .next_states(memory, self.desired_retention, elapsed_days)?;
            let chosen = match r.grade {
                Grade::Again => next.again,
                Grade::Hard => next.hard,
                Grade::Good => next.good,
                Grade::Easy => next.easy,
            };
            memory = Some(chosen.memory);
            let wait = if r.grade == Grade::Again {
                self.relearn_after
            } else {
                SignedDuration::from_secs(chosen.interval.round().max(1.0) as i64 * DAY)
            };
            state = CardState {
                reps: state.reps + 1,
                lapses: state.lapses + u32::from(r.grade == Grade::Again && state.reps > 0),
                stability: Some(chosen.memory.stability),
                difficulty: Some(chosen.memory.difficulty),
                last_review: Some(r.at),
                due: Some(r.at.checked_add(wait).unwrap_or(Timestamp::MAX)),
            };
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardId;
    use crate::review::Signals;

    fn review(day: i64, grade: Grade) -> Review {
        Review {
            card_id: CardId(1),
            at: Timestamp::from_second(day * DAY).unwrap(),
            grade,
            signals: Signals::default(),
        }
    }

    #[test]
    fn a_new_card_has_no_due_date() {
        let s = Scheduler::new(&[], 0.9).unwrap();
        assert_eq!(s.replay(&[]).unwrap().due, None);
    }

    #[test]
    fn remembering_pushes_the_card_further_out_each_time() {
        let s = Scheduler::new(&[], 0.9).unwrap();
        let first = s.replay(&[review(0, Grade::Good)]).unwrap();
        let gap1 = first
            .due
            .unwrap()
            .duration_since(first.last_review.unwrap());
        let d = first.due.unwrap().as_second() / DAY;
        let second = s
            .replay(&[review(0, Grade::Good), review(d, Grade::Good)])
            .unwrap();
        let gap2 = second
            .due
            .unwrap()
            .duration_since(second.last_review.unwrap());
        assert!(gap2 > gap1, "{gap2:?} should exceed {gap1:?}");
    }

    #[test]
    fn forgetting_brings_it_back_today_and_counts_a_lapse() {
        let s = Scheduler::new(&[], 0.9).unwrap();
        let st = s
            .replay(&[review(0, Grade::Good), review(3, Grade::Again)])
            .unwrap();
        assert_eq!(st.lapses, 1);
        assert_eq!(
            st.due.unwrap().duration_since(st.last_review.unwrap()),
            s.relearn_after
        );
    }

    #[test]
    fn rejects_reviews_out_of_order() {
        let s = Scheduler::new(&[], 0.9).unwrap();
        assert!(matches!(
            s.replay(&[review(3, Grade::Good), review(1, Grade::Good)]),
            Err(ScheduleError::OutOfOrder)
        ));
    }
}
