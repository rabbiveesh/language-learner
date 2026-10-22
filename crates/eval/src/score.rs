//! Scoring a scheduler variant on held-out reviews.

use fsrs::{
    ComputeParametersInput, DEFAULT_PARAMETERS, FSRS, FSRSError, FSRSItem, ModelEvaluation,
    compute_parameters, current_retrievability,
};

use crate::history::Sample;

/// Train on the older reviews, test on the newer ones: the only honest split for a model that
/// will predict the future.
#[derive(Debug, Clone)]
pub struct Split {
    pub train: Vec<Sample>,
    pub test: Vec<Sample>,
}

/// `samples` must be oldest first (as [`crate::samples`] returns them).
pub fn split_by_time(samples: Vec<Sample>, test_fraction: f64) -> Split {
    let n_test = ((samples.len() as f64) * test_fraction.clamp(0.0, 1.0)).round() as usize;
    let mut train = samples;
    let test = train.split_off(train.len() - n_test);
    Split { train, test }
}

/// Fit FSRS parameters to `train` (what Anki's "optimize" does).
pub fn fit(train: &[Sample]) -> Result<Vec<f32>, FSRSError> {
    compute_parameters(ComputeParametersInput {
        train_set: train.iter().map(|s| s.item.clone()).collect(),
        card_ids: Some(train.iter().map(|s| s.card_id).collect()),
        ..Default::default()
    })
}

/// Log loss and RMSE over recall bins on `test`, via fsrs's own evaluator (the benchmark's
/// metrics). `parameters` empty = FSRS's defaults.
pub fn score(parameters: &[f32], test: &[Sample]) -> Result<ModelEvaluation, FSRSError> {
    let items: Vec<FSRSItem> = test.iter().map(|s| s.item.clone()).collect();
    let card_ids = test.iter().map(|s| s.card_id).collect();
    FSRS::new(parameters)?.evaluate_with_card_ids(items, card_ids, |_| true)
}

/// The predicted chance of recalling `sample`'s last review, from everything before it.
pub fn predict(fsrs: &FSRS, decay: f32, sample: &Sample) -> Result<f32, FSRSError> {
    let reviews = &sample.item.reviews;
    let (target, history) = reviews.split_last().ok_or(FSRSError::InvalidInput)?;
    let state = fsrs.memory_state(
        FSRSItem {
            reviews: history.to_vec(),
        },
        None,
    )?;
    Ok(current_retrievability(state, target.delta_t as f32, decay))
}

/// Predicted vs actual recall for one slice of the test set. A slice whose `actual` sits well
/// below `predicted` is one the model is too optimistic about.
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    pub label: String,
    pub n: usize,
    pub predicted: f64,
    pub actual: f64,
}

/// Slice `test` by `key` (`edges` are the lower bounds of each slice, ascending) and compare
/// predicted with actual recall in each. Keyed by [`Sample::new_that_day`], this is the binge
/// question: do cards learned in a big batch fade faster than the model expects? Keyed by
/// [`Sample::reviews_that_day`] or [`Sample::nth_that_day`], it's the load question: is recall
/// worse on heavy days, or late in a long session?
pub fn recall_by(
    parameters: &[f32],
    test: &[Sample],
    edges: &[u32],
    key: impl Fn(&Sample) -> u32,
) -> Result<Vec<Bucket>, FSRSError> {
    let fsrs = FSRS::new(parameters)?;
    let decay = parameters
        .get(20)
        .copied()
        .unwrap_or(DEFAULT_PARAMETERS[20]);
    let mut buckets: Vec<Bucket> = edges
        .iter()
        .enumerate()
        .map(|(i, lo)| {
            let label = match edges.get(i + 1) {
                Some(hi) if *hi == lo + 1 => format!("{lo}"),
                Some(hi) => format!("{lo}–{}", hi - 1),
                None => format!("{lo}+"),
            };
            Bucket {
                label,
                n: 0,
                predicted: 0.0,
                actual: 0.0,
            }
        })
        .collect();
    for s in test {
        let Some(i) = edges.iter().rposition(|lo| key(s) >= *lo) else {
            continue;
        };
        let b = &mut buckets[i];
        b.n += 1;
        b.predicted += f64::from(predict(&fsrs, decay, s)?);
        b.actual += if s.recalled() { 1.0 } else { 0.0 };
    }
    for b in &mut buckets {
        if b.n > 0 {
            b.predicted /= b.n as f64;
            b.actual /= b.n as f64;
        }
    }
    Ok(buckets)
}
