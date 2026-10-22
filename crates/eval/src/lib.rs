//! Offline evaluation of scheduling ideas against real review history.
//!
//! Read a review log (an Anki collection, or one user of the FSRS project's dataset), turn it into prediction targets, and score how
//! well a scheduler variant predicts whether each review was remembered. Prediction quality is
//! how the FSRS benchmark compares schedulers, and it takes seconds instead of a week of use.
//!
//! What this can't measure is a *policy* (budgets, drills, intake order), since the history only
//! contains the schedule that actually ran. That needs the simulated learner (plan: "Scheduling").

pub mod anki;
pub mod history;
pub mod revlogs_10k;
pub mod score;

pub use history::{DayClock, LearningDay, LoggedReview, ReviewKind, Sample, samples};
pub use score::{Bucket, Split, by_intake, fit, predict, score, split_by_time};
