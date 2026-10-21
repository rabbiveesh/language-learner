//! Picture search.
//!
//! TODO(design): providers, in order of preference (plan: "Sources"): Brave image search,
//! Openverse, Pixabay, Pexels, Wikimedia Commons. Google's Custom Search JSON API shuts down
//! 2027-01-01, so it's out. How results from several providers are merged and ranked is open.

use std::future::Future;

use domain::Language;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SourceError;

/// What to search for: usually the word itself, sometimes the English gloss when the target
/// language finds nothing useful.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageQuery {
    pub language: Language,
    pub text: String,
}

/// One search result, before the modesty filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImageCandidate {
    pub url: String,
    pub thumb_url: String,
    /// The provider's name, e.g. "openverse".
    pub source: String,
    /// The page the picture came from, for attribution.
    pub page_url: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
}

pub trait ImageSource {
    fn name(&self) -> &'static str;
    fn search(
        &self,
        query: &ImageQuery,
    ) -> impl Future<Output = Result<Vec<ImageCandidate>, SourceError>> + Send;
}
