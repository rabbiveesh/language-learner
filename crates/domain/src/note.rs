//! Notes: the things a learner studies. Cards are generated from them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::language::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
pub struct NoteId(pub i64);

/// A stored media file, named by the hash of its content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MediaRef(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// A word (or a short fixed phrase) in the target language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Word {
    pub spelling: String,
    /// Pronunciation in IPA, from the dictionary data.
    pub ipa: Option<String>,
    pub gender: Option<Gender>,
    /// A short definition, target language first. Fluent Forever avoids translations, so this is
    /// a fallback, never the prompt.
    pub gloss: Option<String>,
    /// The learner's own mnemonic or personal connection.
    pub personal: Option<String>,
    pub image: Option<MediaRef>,
    pub audio: Option<MediaRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum NoteKind {
    Word(Word),
    // TODO(design): `GrammarPoint`, scheduled as a whole, with each review generated fresh from
    // known words (plan: "Grammar cards"). Its shape waits on that design.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Note {
    pub id: NoteId,
    pub language: Language,
    pub kind: NoteKind,
}
