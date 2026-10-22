//! A neutral review log, and the prediction targets made from it.

use std::collections::{BTreeMap, HashMap};

use fsrs::{FSRSItem, FSRSReview};
use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;

/// How a review happened, as the source app recorded it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewKind {
    /// First time seeing the card, or its learning steps.
    Learning,
    Review,
    /// Learning steps after a lapse.
    Relearning,
    /// Crammed in a filtered deck that didn't reschedule. Never a prediction target.
    Filtered,
    /// Rescheduled by hand, not answered. Never a prediction target.
    Manual,
}

/// One review from any source, in a source-neutral shape.
#[derive(Debug, Clone, PartialEq)]
pub struct LoggedReview {
    pub card_id: i64,
    pub at: Timestamp,
    /// 1 = again … 4 = easy.
    pub rating: u8,
    pub kind: ReviewKind,
    pub duration_ms: u32,
}

/// Where one learning day ends: a time zone and the hour the day rolls over (Anki's default is
/// 4am), so a late-night session counts as the same day.
#[derive(Debug, Clone)]
pub struct DayClock {
    pub tz: TimeZone,
    pub rollover_hour: i8,
}

impl DayClock {
    pub fn day(&self, at: Timestamp) -> Date {
        let shifted = at - jiff::SignedDuration::from_hours(i64::from(self.rollover_hour));
        shifted.to_zoned(self.tz.clone()).date()
    }
}

/// One prediction target: everything known about a card up to a review, and that review's
/// outcome (the last entry of `item`).
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub card_id: i64,
    pub at: Timestamp,
    pub item: FSRSItem,
    /// Cards introduced on the day this card was introduced, itself included.
    pub new_that_day: u32,
}

impl Sample {
    pub fn recalled(&self) -> bool {
        self.item.reviews.last().is_some_and(|r| r.rating > 1)
    }
}

/// Turn a review log into prediction targets, oldest first.
///
/// The same rules as fsrs-rs's own Anki converter (not exported, so ported here): manual
/// reschedules and cram reviews are dropped, each card's history starts at its last first-learn
/// (a reset card starts over), elapsed time is counted in learning days, and same-day reviews
/// are history but never targets.
pub fn samples(reviews: &[LoggedReview], clock: &DayClock) -> Vec<Sample> {
    let mut by_card: BTreeMap<i64, Vec<&LoggedReview>> = BTreeMap::new();
    for r in reviews
        .iter()
        .filter(|r| r.kind != ReviewKind::Manual && r.kind != ReviewKind::Filtered && r.rating > 0)
    {
        by_card.entry(r.card_id).or_default().push(r);
    }
    let histories: Vec<(i64, Vec<&LoggedReview>)> = by_card
        .into_iter()
        .filter_map(|(card, mut rs)| {
            rs.sort_by_key(|r| r.at);
            let start = last_first_learn(&rs)?;
            Some((card, rs.split_off(start)))
        })
        .collect();

    let mut intake: HashMap<Date, u32> = HashMap::new();
    for (_, rs) in &histories {
        *intake.entry(clock.day(rs[0].at)).or_default() += 1;
    }

    let mut out = Vec::new();
    for (card, rs) in histories {
        let new_that_day = intake[&clock.day(rs[0].at)];
        let mut reviews = vec![FSRSReview {
            rating: u32::from(rs[0].rating),
            delta_t: 0,
        }];
        for pair in rs.windows(2) {
            let delta_t = (clock.day(pair[1].at) - clock.day(pair[0].at))
                .get_days()
                .max(0) as u32;
            reviews.push(FSRSReview {
                rating: u32::from(pair[1].rating),
                delta_t,
            });
            if delta_t > 0 {
                out.push(Sample {
                    card_id: card,
                    at: pair[1].at,
                    item: FSRSItem {
                        reviews: reviews.clone(),
                    },
                    new_that_day,
                });
            }
        }
    }
    out.sort_by_key(|s| s.at);
    out
}

/// Index of the first review of the card's last run of learning reviews, if it has one.
fn last_first_learn(rs: &[&LoggedReview]) -> Option<usize> {
    let last_learn = rs.iter().rposition(|r| r.kind == ReviewKind::Learning)?;
    let start = rs[..=last_learn]
        .iter()
        .rposition(|r| r.kind != ReviewKind::Learning)
        .map_or(0, |i| i + 1);
    Some(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock() -> DayClock {
        DayClock {
            tz: TimeZone::UTC,
            rollover_hour: 4,
        }
    }

    fn at(day: i64, hour: i64) -> Timestamp {
        Timestamp::from_second(day * 86_400 + hour * 3600).unwrap()
    }

    fn review(card: i64, at: Timestamp, rating: u8, kind: ReviewKind) -> LoggedReview {
        LoggedReview {
            card_id: card,
            at,
            rating,
            kind,
            duration_ms: 3000,
        }
    }

    #[test]
    fn a_late_night_review_counts_as_the_same_learning_day() {
        assert_eq!(clock().day(at(10, 23)), clock().day(at(11, 3)));
        assert_ne!(clock().day(at(10, 23)), clock().day(at(11, 5)));
    }

    #[test]
    fn same_day_reviews_are_history_but_not_targets() {
        let log = [
            review(1, at(0, 10), 1, ReviewKind::Learning),
            review(
                1,
                at(0, 10) + jiff::SignedDuration::from_mins(10),
                3,
                ReviewKind::Learning,
            ),
            review(1, at(2, 10), 3, ReviewKind::Review),
        ];
        let s = samples(&log, &clock());
        assert_eq!(s.len(), 1);
        let deltas: Vec<u32> = s[0].item.reviews.iter().map(|r| r.delta_t).collect();
        assert_eq!(deltas, [0, 0, 2]);
        assert!(s[0].recalled());
    }

    #[test]
    fn a_reset_card_starts_over_and_manual_reschedules_are_ignored() {
        let log = [
            review(1, at(0, 10), 3, ReviewKind::Learning),
            review(1, at(3, 10), 1, ReviewKind::Review),
            review(1, at(4, 10), 0, ReviewKind::Manual),
            review(1, at(5, 10), 3, ReviewKind::Learning),
            review(1, at(8, 10), 3, ReviewKind::Review),
        ];
        let s = samples(&log, &clock());
        assert_eq!(s.len(), 1);
        assert_eq!(
            s[0].item.reviews.len(),
            2,
            "history before the reset is dropped"
        );
    }

    #[test]
    fn intake_counts_cards_introduced_the_same_day() {
        let mut log = Vec::new();
        for card in 0..3 {
            log.push(review(card, at(0, 10), 3, ReviewKind::Learning));
            log.push(review(card, at(1, 10), 3, ReviewKind::Review));
        }
        log.push(review(9, at(5, 10), 3, ReviewKind::Learning));
        log.push(review(9, at(6, 10), 3, ReviewKind::Review));
        let s = samples(&log, &clock());
        assert_eq!(s.iter().filter(|s| s.new_that_day == 3).count(), 3);
        assert_eq!(s.iter().find(|s| s.card_id == 9).unwrap().new_that_day, 1);
    }

    #[test]
    fn a_card_never_learned_has_no_targets() {
        let log = [
            review(1, at(0, 10), 3, ReviewKind::Review),
            review(1, at(3, 10), 3, ReviewKind::Review),
        ];
        assert!(samples(&log, &clock()).is_empty());
    }
}
