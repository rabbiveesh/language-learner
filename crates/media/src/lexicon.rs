//! Dictionary data: pronunciation, gender, short definitions.
//!
//! TODO(design): Wiktextract dumps from kaikki.org (JSONL, one per language) loaded into SQLite
//! for Portuguese; JMdict for Japanese, plus a pitch-accent source. Definitions from the dumps
//! may be thin, in which case a Claude gloss fills in (plan: "Claude jobs").

use std::future::Future;

use domain::{Gender, Language};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SourceError;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LexiconEntry {
    pub spelling: String,
    /// IPA, for the learner's dialect when the source distinguishes them.
    pub ipa: Option<String>,
    pub gender: Option<Gender>,
    /// Short definitions, in the target language where the source has them.
    pub glosses: Vec<String>,
}

pub trait Lexicon {
    fn lookup(
        &self,
        language: &Language,
        word: &str,
    ) -> impl Future<Output = Result<Option<LexiconEntry>, SourceError>> + Send;
}
