//! Word pronunciations recorded by people.
//!
//! TODO(design): providers. Forvo first, filtered to speakers from the learner's region (dialect
//! matters: Brazilian, not European, Portuguese). Its non-commercial plan is about $2/month with
//! 500 requests a day; whether we may store the files is still to be checked. Wikimedia Commons /
//! Lingua Libre as the free fallback. Recordings by the learner's own people (e.g. their mom)
//! should be first-class too, through upload.

use std::future::Future;

use domain::Language;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SourceError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AudioCandidate {
    pub url: String,
    pub source: String,
    /// Where the speaker is from, as the provider reports it (e.g. "Brazil").
    pub region: Option<String>,
    pub speaker: Option<String>,
    pub license: Option<String>,
    /// The provider's own rating, if it has one; higher is better.
    pub votes: Option<i32>,
}

pub trait AudioSource {
    fn name(&self) -> &'static str;
    fn pronunciations(
        &self,
        language: &Language,
        word: &str,
    ) -> impl Future<Output = Result<Vec<AudioCandidate>, SourceError>> + Send;
}
