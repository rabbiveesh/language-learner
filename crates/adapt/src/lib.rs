//! The adaptive difficulty engine, shared shape of robot-game, nat-han-adventures and
//! second-person, generic over the caller's skills. Pure logic: no I/O, all randomness passed in.
//!
//! - [`Profile`] + [`reduce`]: a pure reducer over [`Event`]s. Per skill: a center band 1..=10,
//!   a spread 0..1 and a rolling window of recent outcomes. Globally: the assist dial 0..1
//!   (separate from difficulty), calibration, a display-only streak.
//! - [`pick_skill`] and [`next_band`]: what comes next. [`band_distribution`] keeps most items at
//!   the center, with some reinforcement below and some stretch above.
//! - [`Calibration`]: the first attempts are a disguised placement test.
//! - [`Tuning`]: every number the rules use. Each app passes its own.
//!
//! What stays with the app: what a skill is, how an attempt is judged ([`Outcome`], including
//! the carelessness filter that turns a quick miss after a run of clean answers into a
//! [`Outcome::Slip`]), what frustration looks like, and what the assist dial concretely does.
//!
//! Never tell the learner: nothing here produces a label, and [`Cue`]s are for logs and for the
//! app's own reactions (an encouraging line, easier picks), not for display.
//!
//! # Rules
//!
//! - **Assists fade before difficulty rises.** A clean attempt lowers the dial by
//!   [`Tuning::assist_fade`]; each miss beyond what the attempt expected raises it by
//!   [`Tuning::assist_rise`]. A promotion needs the dial ≤ [`Tuning::assist_eps`], and only
//!   attempts made that unassisted count as promotion evidence.
//! - **Promote** a skill at ≥ [`Tuning::promote_at`] clean over ≥ [`Tuning::min_evidence`]
//!   attempts at its center (and ≥ [`Tuning::stretch_at`] on stretch attempts if there were
//!   enough); **demote** below [`Tuning::demote_below`]. Evidence is per epoch: a band change bumps
//!   the epoch, so old window entries never count toward the new center. Going back up into a
//!   band just fallen out of needs [`Tuning::repromote_evidence`] (hysteresis).
//! - **Frustration** (detected by the app): ease the skills that were pushing, narrow their
//!   spread, raise assists, favour strengths for a few picks, and emit [`Cue::Encourage`].

mod band;
mod calibration;
mod choose;
mod profile;
mod tuning;
mod window;

pub use band::{Band, MAX_BAND, MIN_BAND, band_distribution, clamp_band, sample_band};
pub use calibration::{Calibration, Placement, Probe};
pub use choose::{next_band, pick_skill};
pub use profile::{
    Attempt, Cue, Event, Evidence, Profile, SkillState, reduce, should_demote, should_promote,
};
pub use tuning::Tuning;
pub use window::{Outcome, RollingWindow, WindowEntry};

/// What the engine needs from the caller's skill type. Any small, ordered, serializable enum
/// qualifies (blanket impl).
pub trait Skill: Copy + Ord + std::fmt::Debug {}
impl<T: Copy + Ord + std::fmt::Debug> Skill for T {}
