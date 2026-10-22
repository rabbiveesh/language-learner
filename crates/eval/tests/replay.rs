//! The evaluator finds what we planted in a synthetic learner.

mod common;

use common::{Learner, simulate};
use eval::{recall_by, samples, score, split_by_time};
use fsrs::DEFAULT_PARAMETERS;

#[test]
fn the_true_model_scores_better_than_a_wrong_one() {
    let log = simulate(&Learner::average(), 150, |_| 8, 1);
    let split = split_by_time(samples(&log), 0.2);
    assert!(
        split.test.len() > 500,
        "enough targets to judge: {}",
        split.test.len()
    );

    let mut wrong = DEFAULT_PARAMETERS.to_vec();
    for w in &mut wrong[..4] {
        *w *= 8.0; // thinks every new card is far stronger than it is
    }
    let truth = score(&[], &split.test).unwrap();
    let wrong = score(&wrong, &split.test).unwrap();
    assert!(truth.log_loss < wrong.log_loss, "{truth:?} vs {wrong:?}");
    assert!(truth.rmse_bins < wrong.rmse_bins, "{truth:?} vs {wrong:?}");
}

#[test]
fn a_planted_binge_effect_shows_up_as_a_gap_on_big_intake_days() {
    let binger = Learner {
        binge_at: 40,
        binge_stability: 0.25,
        ..Learner::average()
    };
    let log = simulate(&binger, 150, |day| if day % 7 == 0 { 50 } else { 4 }, 2);
    let split = split_by_time(samples(&log), 0.3);

    let buckets = recall_by(&[], &split.test, &[1, 40], |s| s.new_that_day).unwrap();
    let (calm, binge) = (&buckets[0], &buckets[1]);
    assert!(calm.n > 100 && binge.n > 100, "{buckets:?}");
    let calm_gap = calm.actual - calm.predicted;
    let binge_gap = binge.actual - binge.predicted;
    assert!(calm_gap.abs() < 0.05, "calm days match the model: {calm:?}");
    assert!(
        binge_gap < calm_gap - 0.04,
        "binge days fade faster than predicted: {buckets:?}"
    );
}

#[test]
fn no_planted_effect_means_no_gap() {
    let log = simulate(
        &Learner::average(),
        150,
        |day| if day % 7 == 0 { 50 } else { 4 },
        3,
    );
    let split = split_by_time(samples(&log), 0.3);
    for b in recall_by(&[], &split.test, &[1, 40], |s| s.new_that_day).unwrap() {
        assert!((b.actual - b.predicted).abs() < 0.05, "{b:?}");
    }
}

#[test]
fn a_planted_tiredness_effect_shows_up_late_in_long_sessions() {
    let tired = Learner {
        tired_after: 60,
        tired_recall: 0.8,
        ..Learner::average()
    };
    let log = simulate(&tired, 150, |_| 12, 4);
    let split = split_by_time(samples(&log), 0.3);

    let buckets = recall_by(&[], &split.test, &[1, 61], |s| s.nth_that_day).unwrap();
    let (fresh, late) = (&buckets[0], &buckets[1]);
    assert!(fresh.n > 100 && late.n > 100, "{buckets:?}");
    assert!(
        (fresh.actual - fresh.predicted).abs() < 0.05,
        "early reviews match the model: {fresh:?}"
    );
    assert!(
        late.actual - late.predicted < -0.1,
        "late reviews fall short of it: {late:?}"
    );
}
