//! Choosing what comes next: which skill, at which band.

use rand::Rng;

use crate::Skill;
use crate::band::{Band, MIN_BAND, band_distribution, sample_band};
use crate::profile::Profile;
use crate::tuning::Tuning;

/// Pick the skill for the next item among `available`: rank by clean rate (no history =
/// [`Tuning::unknown_rate`]), the top half are strengths and the rest growth areas; pick a strength
/// with [`Tuning::strength_share`] (more while frustrated), then uniformly within the group.
/// `None` when nothing is available.
pub fn pick_skill<S: Skill>(
    t: &Tuning,
    profile: &Profile<S>,
    available: &[S],
    rng: &mut impl Rng,
) -> Option<S> {
    let mut ranked: Vec<(S, f32)> = Vec::new();
    for &s in available {
        if !ranked.iter().any(|(r, _)| *r == s) {
            let rate = profile
                .skill(s)
                .and_then(|st| st.window.clean_rate().0)
                .unwrap_or(t.unknown_rate);
            ranked.push((s, rate));
        }
    }
    if ranked.len() <= 1 {
        return ranked.first().map(|r| r.0);
    }
    // Stable: equal rates keep `available` order.
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let (strengths, growth) = ranked.split_at(ranked.len().div_ceil(2));
    let share = if profile.frustrated() {
        t.frustrated_strength_share
    } else {
        t.strength_share
    };
    let group = if rng.random::<f32>() < share {
        strengths
    } else {
        growth
    };
    Some(group[rng.random_range(0..group.len())].0)
}

/// The band for the next item of `skill`: the next probe while calibrating, otherwise a draw
/// around the skill's center.
pub fn next_band<S: Skill>(t: &Tuning, profile: &Profile<S>, skill: S, rng: &mut impl Rng) -> Band {
    if profile.calibrating() {
        return profile.calibration.next_band(t);
    }
    match profile.skill(skill) {
        Some(st) => sample_band(&band_distribution(st.center, st.spread), rng),
        None => MIN_BAND,
    }
}
