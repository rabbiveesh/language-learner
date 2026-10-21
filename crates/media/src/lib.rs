//! Where pictures, audio and dictionary data come from, and the modesty filter every picture
//! passes before a learner sees it.
//!
//! Each kind of source is a trait so providers can be swapped and faked in tests. No provider is
//! wired up yet: see the `TODO(design)` notes in each module for the planned ones.

pub mod audio;
pub mod image;
pub mod lexicon;
pub mod modesty;

pub use audio::{AudioCandidate, AudioSource};
pub use image::{ImageCandidate, ImageQuery, ImageSource};
pub use lexicon::{Lexicon, LexiconEntry};
pub use modesty::{Filtered, ModestyCheck, ModestyVerdict, STANDARD, filter_candidates};

/// A provider failed. Callers treat this as "no results from this source", never as a crash.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("{source_name} is not configured (missing API key?)")]
    NotConfigured { source_name: &'static str },
    #[error("{source_name} rate limit reached")]
    RateLimited { source_name: &'static str },
    #[error("{source_name}: {message}")]
    Failed {
        source_name: &'static str,
        message: String,
    },
}
