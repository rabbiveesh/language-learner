//! Grading typed answers. The app grades typed cards so the learner never has to.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::review::Grade;

/// How a typed answer compares with the expected one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedVerdict {
    Exact,
    /// Right letters, wrong or missing accents on Latin letters (`agua` for `água`).
    AccentOnly,
    /// One small typo away (edit distance 1 on a word of 5+ letters).
    Typo,
    Wrong,
}

impl TypedVerdict {
    /// The FSRS grade for this verdict.
    // TODO(design): use response time too ("fast and right = Easy, slow and right = Hard", with
    // thresholds learned per card kind from the learner's own medians; plan: "SRS").
    pub fn grade(self) -> Grade {
        match self {
            TypedVerdict::Exact => Grade::Good,
            TypedVerdict::AccentOnly | TypedVerdict::Typo => Grade::Hard,
            TypedVerdict::Wrong => Grade::Again,
        }
    }
}

/// Case, surrounding space and repeated spaces never matter; Unicode forms are unified.
fn normalize(s: &str) -> String {
    s.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Drop accents from Latin letters only. Marks on other scripts carry meaning (Japanese dakuten
/// turn か into が), so they stay.
fn strip_latin_accents(s: &str) -> String {
    let mut out = String::new();
    let mut after_latin = false;
    for c in s.nfd() {
        if is_combining_mark(c) {
            if !after_latin {
                out.push(c);
            }
            continue;
        }
        after_latin = c.is_ascii_alphabetic();
        out.push(c);
    }
    out.nfc().collect()
}

pub fn check_typed(expected: &str, given: &str) -> TypedVerdict {
    let (expected, given) = (normalize(expected), normalize(given));
    if expected == given {
        return TypedVerdict::Exact;
    }
    if strip_latin_accents(&expected) == strip_latin_accents(&given) {
        return TypedVerdict::AccentOnly;
    }
    if expected.chars().count() >= 5 && strsim::damerau_levenshtein(&expected, &given) == 1 {
        return TypedVerdict::Typo;
    }
    TypedVerdict::Wrong
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_and_spacing_never_matter() {
        assert_eq!(
            check_typed("Você toma água", "  você  toma água "),
            TypedVerdict::Exact
        );
    }

    #[test]
    fn missing_accents_are_hard_not_wrong() {
        assert_eq!(check_typed("água", "agua"), TypedVerdict::AccentOnly);
        assert_eq!(TypedVerdict::AccentOnly.grade(), Grade::Hard);
    }

    #[test]
    fn japanese_voicing_marks_are_letters_not_accents() {
        assert_eq!(check_typed("がっこう", "かっこう"), TypedVerdict::Wrong);
    }

    #[test]
    fn a_single_typo_on_a_long_word_is_hard() {
        assert_eq!(check_typed("obrigado", "obirgado"), TypedVerdict::Typo);
        assert_eq!(check_typed("copo", "cópo").grade(), Grade::Hard);
        assert_eq!(check_typed("copo", "capo"), TypedVerdict::Wrong);
    }
}
