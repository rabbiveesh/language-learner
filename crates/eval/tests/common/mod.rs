//! A synthetic learner whose memory follows FSRS's default model, optionally with a planted
//! weakness, reviewing on FSRS's own schedule. Test data with a known answer.

use eval::{LoggedReview, ReviewKind};
use fsrs::{DEFAULT_PARAMETERS, FSRS, MemoryState, current_retrievability};
use jiff::Timestamp;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Learner {
    /// Cards introduced on a day with at least this many new cards...
    pub binge_at: u32,
    /// ...really have this share of the stability the model thinks they have.
    pub binge_stability: f32,
}

impl Learner {
    pub fn average() -> Self {
        Learner {
            binge_at: u32::MAX,
            binge_stability: 1.0,
        }
    }
}

struct Card {
    id: i64,
    memory: MemoryState,
    last_day: i64,
    due_day: i64,
    true_share: f32,
}

pub fn at(day: i64, nth: i64) -> Timestamp {
    Timestamp::from_second(day * 86_400 + 12 * 3600 + nth).unwrap()
}

/// `new_on(day)` cards are introduced each day for `days` days; every due card is reviewed.
pub fn simulate(
    learner: &Learner,
    days: i64,
    new_on: impl Fn(i64) -> u32,
    seed: u64,
) -> Vec<LoggedReview> {
    let fsrs = FSRS::new(&[]).unwrap();
    let decay = DEFAULT_PARAMETERS[20];
    let mut rng = StdRng::seed_from_u64(seed);
    let mut cards: Vec<Card> = Vec::new();
    let mut log = Vec::new();
    for day in 0..days {
        let mut nth = 0;
        let stamp = |nth: &mut i64| {
            *nth += 1;
            at(day, *nth)
        };
        for c in cards.iter_mut().filter(|c| c.due_day <= day) {
            let elapsed = (day - c.last_day) as u32;
            let truth = MemoryState {
                stability: c.memory.stability * c.true_share,
                ..c.memory
            };
            let recalled =
                rng.random::<f32>() < current_retrievability(truth, elapsed as f32, decay);
            let next = fsrs.next_states(Some(c.memory), 0.9, elapsed).unwrap();
            let (state, rating) = if recalled {
                (next.good, 3)
            } else {
                (next.again, 1)
            };
            c.memory = state.memory;
            c.last_day = day;
            c.due_day = day
                + if recalled {
                    (state.interval.round() as i64).max(1)
                } else {
                    1
                };
            log.push(LoggedReview {
                card_id: c.id,
                at: stamp(&mut nth),
                rating,
                kind: ReviewKind::Review,
                duration_ms: 4000,
            });
        }
        let n = new_on(day);
        let true_share = if n >= learner.binge_at {
            learner.binge_stability
        } else {
            1.0
        };
        for _ in 0..n {
            let id = cards.len() as i64 + 1;
            let first = fsrs.next_states(None, 0.9, 0).unwrap().good;
            log.push(LoggedReview {
                card_id: id,
                at: stamp(&mut nth),
                rating: 3,
                kind: ReviewKind::Learning,
                duration_ms: 6000,
            });
            cards.push(Card {
                id,
                memory: first.memory,
                last_day: day,
                due_day: day + (first.interval.round() as i64).max(1),
                true_share,
            });
        }
    }
    log
}
