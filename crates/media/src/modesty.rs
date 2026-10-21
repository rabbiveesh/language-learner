//! The modesty (tznius) filter. Every picture from a search passes through it before the learner
//! sees it, and it fails closed: a picture the check couldn't judge is dropped, never shown.
//!
//! The standard lives in `config/modesty.md` so it reads as plain words and can be handed to a
//! vision model as is. When nothing survives, the app asks the learner to upload their own
//! picture that matches the word's vibe; pictures the learner uploads skip the check.
//!
//! TODO(design): the real check. Planned: claude-haiku-5-5 vision with [`STANDARD`] as the
//! instructions and a structured yes/no answer, verdicts cached by image hash. Whether a cheaper
//! first pass (provider safe-search flags, a local classifier) is worth adding is open.

use std::future::Future;

use crate::image::ImageCandidate;

/// The standard every picture is held to, in the learner's own words.
pub const STANDARD: &str = include_str!("../../../config/modesty.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModestyVerdict {
    Allowed,
    Rejected { reason: String },
}

pub trait ModestyCheck {
    /// Judge one picture against [`STANDARD`]. An `Err` means it couldn't be judged.
    fn check(
        &self,
        candidate: &ImageCandidate,
    ) -> impl Future<Output = Result<ModestyVerdict, String>> + Send;
}

/// What came out of the filter.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Filtered {
    pub allowed: Vec<ImageCandidate>,
    pub rejected: usize,
    /// Couldn't be judged, so dropped too.
    pub unchecked: usize,
}

impl Filtered {
    /// Nothing to show: ask the learner for their own picture.
    pub fn suggest_upload(&self) -> bool {
        self.allowed.is_empty()
    }
}

/// Keep only the candidates the check allows. Errors drop the candidate (fail closed).
pub async fn filter_candidates(
    check: &impl ModestyCheck,
    candidates: Vec<ImageCandidate>,
) -> Filtered {
    let mut out = Filtered::default();
    for c in candidates {
        match check.check(&c).await {
            Ok(ModestyVerdict::Allowed) => out.allowed.push(c),
            Ok(ModestyVerdict::Rejected { .. }) => out.rejected += 1,
            Err(_) => out.unchecked += 1,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allows URLs containing "ok", rejects "bad", fails on anything else.
    struct Fake;

    impl ModestyCheck for Fake {
        async fn check(&self, c: &ImageCandidate) -> Result<ModestyVerdict, String> {
            if c.url.contains("ok") {
                Ok(ModestyVerdict::Allowed)
            } else if c.url.contains("bad") {
                Ok(ModestyVerdict::Rejected {
                    reason: "fake".into(),
                })
            } else {
                Err("timeout".into())
            }
        }
    }

    fn candidate(url: &str) -> ImageCandidate {
        ImageCandidate {
            url: url.into(),
            thumb_url: url.into(),
            source: "test".into(),
            page_url: None,
            license: None,
            author: None,
        }
    }

    #[tokio::test]
    async fn only_allowed_pictures_get_through() {
        let out = filter_candidates(
            &Fake,
            vec![candidate("ok-1"), candidate("bad"), candidate("ok-2")],
        )
        .await;
        assert_eq!(
            out.allowed
                .iter()
                .map(|c| c.url.as_str())
                .collect::<Vec<_>>(),
            ["ok-1", "ok-2"]
        );
        assert_eq!(out.rejected, 1);
        assert!(!out.suggest_upload());
    }

    #[tokio::test]
    async fn a_picture_that_cant_be_judged_is_dropped() {
        let out = filter_candidates(&Fake, vec![candidate("flaky")]).await;
        assert!(out.allowed.is_empty());
        assert_eq!(out.unchecked, 1);
        assert!(out.suggest_upload(), "nothing left means ask for an upload");
    }

    #[test]
    fn the_standard_is_in_the_learners_words() {
        assert!(STANDARD.contains("avoid pictures with women in general"));
    }
}
