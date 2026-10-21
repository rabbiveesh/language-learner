//! Cards: one way of asking about a note. Each card has its own review history and schedule.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::note::{Note, NoteId, NoteKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
pub struct CardId(pub i64);

/// How a card asks. Recognition and production are different skills, and so are typing and
/// speaking, so each kind gets its own FSRS parameters and its own adaptive skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CardKind {
    /// See the picture, type the word.
    PictureToWord,
    /// See the word, recall the picture (self-graded).
    WordToPicture,
    /// Hear the word, type it.
    AudioToWord,
    /// See the picture, say the word.
    // TODO(design): grading needs speech-to-text (plan: "Spoken answers"); self-graded until then.
    Speaking,
}

impl CardKind {
    /// Typed answers are graded by the app, never by the learner.
    pub fn typed(self) -> bool {
        matches!(self, CardKind::PictureToWord | CardKind::AudioToWord)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Card {
    pub id: CardId,
    pub note_id: NoteId,
    pub kind: CardKind,
}

/// The card kinds a note asks for. Kinds that need media the note lacks are left out.
pub fn kinds_for(note: &Note) -> Vec<CardKind> {
    match &note.kind {
        NoteKind::Word(w) => {
            let mut kinds = Vec::new();
            if w.image.is_some() {
                kinds.extend([
                    CardKind::PictureToWord,
                    CardKind::WordToPicture,
                    CardKind::Speaking,
                ]);
            }
            if w.audio.is_some() {
                kinds.push(CardKind::AudioToWord);
            }
            kinds
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use crate::note::{MediaRef, Word};

    fn word(image: bool, audio: bool) -> Note {
        let w = Word {
            spelling: "copo".into(),
            ipa: Some("ˈkɔ.pu".into()),
            gender: None,
            gloss: None,
            personal: None,
            image: image.then(|| MediaRef("img".into())),
            audio: audio.then(|| MediaRef("snd".into())),
        };
        Note {
            id: NoteId(1),
            language: Language::new("pt-BR"),
            kind: NoteKind::Word(w),
        }
    }

    #[test]
    fn a_word_without_media_asks_nothing_yet() {
        assert!(kinds_for(&word(false, false)).is_empty());
    }

    #[test]
    fn audio_adds_a_listening_card() {
        assert_eq!(kinds_for(&word(false, true)), vec![CardKind::AudioToWord]);
        assert_eq!(kinds_for(&word(true, true)).len(), 4);
    }
}
