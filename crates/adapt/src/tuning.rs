//! Every number the rules use. The games picked theirs with simulators (see
//! nat-han-adventures' `src/adapt/README.md`); an app starts from [`Tuning::default`] and
//! re-tunes against its own synthetic learners.

use serde::{Deserialize, Serialize};

use crate::band::Band;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuning {
    /// Attempts remembered per skill.
    pub window: usize,

    /// The dial at or below this counts as "off": promotion needs it, and only attempts made this
    /// unassisted count as promotion evidence.
    pub assist_eps: f32,
    /// A clean attempt lowers the dial by this much.
    pub assist_fade: f32,
    /// Each miss beyond what the attempt expected raises the dial this much...
    pub assist_rise: f32,
    /// ...counting at most this many misses per attempt.
    pub assist_rise_cap: u32,
    /// Frustration raises the dial by this times `(1 − dial)`, so repeated frustration doesn't pin
    /// it at max. A frustrated attempt gets the larger of this and the per-miss rise, not both.
    pub frustration_assists: f32,
    /// Picks after a frustration during which [`crate::pick_skill`] favours strengths more.
    pub frustration_cooldown: u8,

    /// Promote: clean rate at the center (unassisted, this epoch) at least this...
    pub promote_at: f32,
    /// ...over at least this many attempts...
    pub min_evidence: usize,
    /// ...and, with at least [`Tuning::min_stretch`] stretch attempts, a stretch clean rate of at
    /// least this.
    pub stretch_at: f32,
    pub min_stretch: usize,
    /// Demote: clean rate at the center (this epoch) below this over
    /// [`Tuning::min_evidence`]+ attempts.
    pub demote_below: f32,
    /// Promoting back into a band the skill fell out of needs this many attempts of evidence.
    pub repromote_evidence: usize,
    /// ...and, if set, the stretch attempts are required rather than merely checked
    /// (second-person's stricter variant).
    pub repromote_needs_stretch: bool,

    /// Widen the spread by [`Tuning::widen_step`] (up to [`Tuning::widen_max`]) while the whole
    /// window is above [`Tuning::widen_at`] clean over [`Tuning::widen_min`]+ attempts.
    pub widen_at: f32,
    pub widen_min: usize,
    pub widen_step: f32,
    pub widen_max: f32,
    /// Narrowing on promote / demote / frustration, and the floors.
    pub promote_narrow: f32,
    pub promote_spread_min: f32,
    pub demote_narrow: f32,
    pub demote_spread_min: f32,
    pub frustration_narrow: f32,
    pub default_spread: f32,

    /// Share of skill picks that go to strengths (the rest to growth areas)...
    pub strength_share: f32,
    /// ...and while frustrated.
    pub frustrated_strength_share: f32,
    /// Clean rate assumed for a skill with no history.
    pub unknown_rate: f32,
    /// A newly unlocked skill starts this many bands under the mean played center.
    pub unlock_offset: u8,

    /// Calibration: the first probe's band, the steps after a clean and a struggling probe...
    pub probe_start: Band,
    pub probe_up: i32,
    pub probe_down: i32,
    /// ...how many probes at least and at most...
    pub min_probes: usize,
    pub max_probes: usize,
    /// ...and the starting dial when no probe was clean, some struggled, or all were clean.
    pub placed_assists_none_clean: f32,
    pub placed_assists_some_struggle: f32,
    pub placed_assists_all_clean: f32,
}

impl Default for Tuning {
    /// nat-han-adventures' simulator-tuned values. A starting point, not a result: re-tune for
    /// flashcards once there's a learner simulator with FSRS in the loop.
    fn default() -> Self {
        Tuning {
            window: 20,
            assist_eps: 0.02,
            assist_fade: 0.07,
            assist_rise: 0.08,
            assist_rise_cap: 3,
            frustration_assists: 0.15,
            frustration_cooldown: 4,
            promote_at: 0.75,
            min_evidence: 4,
            stretch_at: 0.6,
            min_stretch: 2,
            demote_below: 0.5,
            repromote_evidence: 8,
            repromote_needs_stretch: false,
            widen_at: 0.75,
            widen_min: 10,
            widen_step: 0.1,
            widen_max: 0.8,
            promote_narrow: 0.1,
            promote_spread_min: 0.2,
            demote_narrow: 0.15,
            demote_spread_min: 0.1,
            frustration_narrow: 0.15,
            default_spread: 0.5,
            strength_share: 0.6,
            frustrated_strength_share: 0.8,
            unknown_rate: 0.5,
            unlock_offset: 1,
            probe_start: 3,
            probe_up: 2,
            probe_down: 1,
            min_probes: 2,
            max_probes: 3,
            placed_assists_none_clean: 0.4,
            placed_assists_some_struggle: 0.15,
            placed_assists_all_clean: 0.0,
        }
    }
}
