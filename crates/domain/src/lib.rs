//! Language Learner's domain: what a learner studies (notes), how it's asked (cards), what
//! happened (the append-only review log), how answers are graded, and when cards come back
//! (FSRS). Pure: no I/O, no clock. Callers pass timestamps in.

pub mod card;
pub mod grading;
pub mod language;
pub mod note;
pub mod review;
pub mod schedule;

pub use card::{Card, CardId, CardKind};
pub use grading::{TypedVerdict, check_typed};
pub use language::Language;
pub use note::{Gender, MediaRef, Note, NoteId, NoteKind, Word};
pub use review::{Grade, Review, Signals};
pub use schedule::{CardState, ScheduleError, Scheduler};
