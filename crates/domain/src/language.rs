//! Which language a note belongs to. Languages are data, never code paths: anything that
//! differs between languages (scripts, voices, dictionaries) is looked up by tag.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A BCP 47 language tag, e.g. `pt-BR` or `ja`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Language(pub String);

impl Language {
    pub fn new(tag: impl Into<String>) -> Self {
        Language(tag.into())
    }
}
