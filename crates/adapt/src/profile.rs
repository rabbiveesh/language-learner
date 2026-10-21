//! [`Profile`] and its pure reducer, [`reduce`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Skill;
use crate::band::{Band, MAX_BAND, MIN_BAND, clamp_band};
use crate::calibration::{Calibration, Placement, Probe};
use crate::tuning::Tuning;
use crate::window::{Outcome, RollingWindow, WindowEntry};

// ─── State ───────────────────────────────────────────────────────────────────

/// One skill's difficulty state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillState {
    /// Center band; items are drawn around it ([`crate::band_distribution`]).
    pub center: Band,
    /// 0 = nearly always the center, 1 = lots of reinforcement and stretch.
    pub spread: f32,
    pub window: RollingWindow,
    /// Bumps on every band change; window entries from older epochs don't count toward the new
    /// center.
    pub epoch: u32,
    /// The band this skill last fell out of (demotion or frustration), until it's re-earned.
    pub fell_from: Option<Band>,
}

impl SkillState {
    pub fn new(center: Band, t: &Tuning) -> Self {
        SkillState {
            center,
            spread: t.default_spread,
            window: RollingWindow::default(),
            epoch: 0,
            fell_from: None,
        }
    }

    fn back_into_fallen_band(&self) -> bool {
        self.fell_from == Some(self.center + 1)
    }

    /// Attempts at the center needed before a promotion.
    pub fn evidence_needed(&self, t: &Tuning) -> usize {
        if self.back_into_fallen_band() {
            t.repromote_evidence
        } else {
            t.min_evidence
        }
    }
}

/// Everything the engine knows about the learner. Update it only with [`reduce`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "S: Serialize + Ord",
    deserialize = "S: Deserialize<'de> + Ord"
))]
pub struct Profile<S> {
    pub skills: BTreeMap<S, SkillState>,
    /// The global assist dial, 0..1. What it does concretely is the app's business.
    pub assists: f32,
    /// Clean attempts in a row. Display only: it never moves a band.
    pub streak: u32,
    pub calibration: Calibration,
    /// Attempts left of "go easy" skill picking after a frustration.
    pub frustration_cooldown: u8,
    pub attempts: u32,
}

impl<S: Skill> Default for Profile<S> {
    fn default() -> Self {
        Profile {
            skills: BTreeMap::new(),
            assists: 0.0,
            streak: 0,
            calibration: Calibration::default(),
            frustration_cooldown: 0,
            attempts: 0,
        }
    }
}

impl<S: Skill> Profile<S> {
    /// A learner past calibration with `skills` centered at `band` (tests, tools).
    pub fn calibrated(skills: &[S], band: Band, t: &Tuning) -> Self {
        Profile {
            skills: skills
                .iter()
                .map(|&s| (s, SkillState::new(band.clamp(MIN_BAND, MAX_BAND), t)))
                .collect(),
            calibration: Calibration::finished(),
            ..Self::default()
        }
    }

    /// The skill's state, if it has one yet.
    pub fn skill(&self, skill: S) -> Option<&SkillState> {
        self.skills.get(&skill)
    }

    /// Still in the go-easy period after a frustration.
    pub fn frustrated(&self) -> bool {
        self.frustration_cooldown > 0
    }

    pub fn calibrating(&self) -> bool {
        !self.calibration.done
    }

    /// Where a skill with no state starts: band 1 before any play; afterwards the mean center of
    /// the skills that have been played, minus [`Tuning::unlock_offset`] (so a strong learner
    /// doesn't grind a new skill up from band 1).
    fn starting_center(&self, t: &Tuning) -> Band {
        let played: Vec<Band> = self
            .skills
            .values()
            .filter(|s| !s.window.entries.is_empty())
            .map(|s| s.center)
            .collect();
        if played.is_empty() || self.calibrating() {
            return MIN_BAND;
        }
        let mean = played.iter().map(|b| *b as f32).sum::<f32>() / played.len() as f32;
        clamp_band(mean.round() as i32 - t.unlock_offset as i32)
    }

    fn ensure(&mut self, skill: S, t: &Tuning) {
        if !self.skills.contains_key(&skill) {
            let center = self.starting_center(t);
            self.skills.insert(skill, SkillState::new(center, t));
        }
    }
}

// ─── Events ──────────────────────────────────────────────────────────────────

/// How one skill fared in an attempt, at the band it was exercised at.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Evidence<S> {
    pub skill: S,
    pub band: Band,
    pub outcome: Outcome,
}

/// One finished attempt (a card answered, a room cleared, a round played), judged by the app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attempt<S> {
    /// Each skill the attempt exercised. The first one is the probe during calibration.
    pub evidence: Vec<Evidence<S>>,
    /// Misses beyond what the attempt expected (0 when it went as expected). Drives the dial.
    pub misses: u32,
    /// The assist dial when the attempt started.
    pub assists: f32,
    /// The app judged this attempt frustrating. Its response replaces the per-miss dial rise.
    pub frustrated: bool,
}

impl<S> Attempt<S> {
    /// The attempt as a whole: a struggle if any skill struggled, a slip if any slipped.
    pub fn outcome(&self) -> Outcome {
        let has = |o| self.evidence.iter().any(|e| e.outcome == o);
        if has(Outcome::Struggle) {
            Outcome::Struggle
        } else if has(Outcome::Slip) {
            Outcome::Slip
        } else {
            Outcome::Clean
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event<S> {
    Attempted(Attempt<S>),
    /// A frustration signal the app noticed while an attempt was in progress (e.g. idling after a
    /// miss), with the skills and bands that attempt exercises. Handling it once per attempt is
    /// the app's job.
    Frustrated {
        exercising: Vec<(S, Band)>,
    },
    /// A skill became available. A skill with no state yet starts at band 1 before calibration,
    /// and afterwards [`Tuning::unlock_offset`] under the mean center of the skills played.
    Unlocked(S),
}

/// Something the app should react to, produced by the last [`reduce`]. Never shown as a label.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cue<S> {
    /// Say something encouraging.
    Encourage,
    Promoted(S, Band),
    /// A skill's center went down after sustained struggle.
    Demoted(S, Band),
    /// A skill's center was eased down because of frustration.
    Eased(S, Band),
    /// The placement attempts are over; bands and assists were set.
    Calibrated(Placement),
}

// ─── Reducer ─────────────────────────────────────────────────────────────────

/// The next profile after `event`, and what it asks of the app. Pure: same input, same output.
pub fn reduce<S: Skill>(
    t: &Tuning,
    state: Profile<S>,
    event: Event<S>,
) -> (Profile<S>, Vec<Cue<S>>) {
    match event {
        Event::Attempted(attempt) => attempted(t, state, attempt),
        Event::Frustrated { exercising } => ease(t, state, &exercising, Vec::new()),
        Event::Unlocked(skill) => {
            let mut s = state;
            s.ensure(skill, t);
            (s, Vec::new())
        }
    }
}

fn attempted<S: Skill>(
    t: &Tuning,
    state: Profile<S>,
    attempt: Attempt<S>,
) -> (Profile<S>, Vec<Cue<S>>) {
    let mut s = state;
    for e in &attempt.evidence {
        s.ensure(e.skill, t);
    }
    let outcome = attempt.outcome();
    s.attempts += 1;
    s.frustration_cooldown = s.frustration_cooldown.saturating_sub(1);
    s.streak = match outcome {
        Outcome::Clean => s.streak + 1,
        Outcome::Slip => s.streak,
        Outcome::Struggle => 0,
    };
    let exercising: Vec<(S, Band)> = attempt.evidence.iter().map(|e| (e.skill, e.band)).collect();

    if s.calibrating() {
        return calibration_attempt(t, s, &attempt, outcome);
    }

    // The dial: clean attempts fade it, misses raise it. A frustrated attempt gets the larger of
    // the per-miss rise and the frustration bump (applied by `ease`), not both.
    let miss_rise = t.assist_rise * attempt.misses.min(t.assist_rise_cap) as f32;
    s.assists = match outcome {
        Outcome::Clean => s.assists - t.assist_fade,
        Outcome::Slip => s.assists,
        Outcome::Struggle if attempt.frustrated => {
            s.assists + (miss_rise - frustration_bump(t, s.assists)).max(0.0)
        }
        Outcome::Struggle => s.assists + miss_rise,
    }
    .clamp(0.0, 1.0);

    for e in &attempt.evidence {
        let st = &s.skills[&e.skill];
        let entry = WindowEntry {
            outcome: e.outcome,
            band: e.band.clamp(MIN_BAND, MAX_BAND),
            center: st.center,
            epoch: st.epoch,
            assists: attempt.assists,
        };
        let window = st.window.push(entry, t.window);
        s.skills.insert(
            e.skill,
            SkillState {
                window,
                ..st.clone()
            },
        );
    }

    if attempt.frustrated {
        return ease(t, s, &exercising, Vec::new());
    }
    if outcome == Outcome::Slip {
        return (s, Vec::new());
    }
    let mut judged: Vec<S> = exercising.iter().map(|e| e.0).collect();
    judged.sort();
    judged.dedup();
    judge(t, s, &judged)
}

/// Promote / demote / widen each of `judged`.
fn judge<S: Skill>(t: &Tuning, mut s: Profile<S>, judged: &[S]) -> (Profile<S>, Vec<Cue<S>>) {
    let mut cues = Vec::new();
    for &skill in judged {
        let st = &s.skills[&skill];
        let next = if st.center < MAX_BAND && s.assists <= t.assist_eps && should_promote(t, st) {
            cues.push(Cue::Promoted(skill, st.center + 1));
            SkillState {
                center: st.center + 1,
                spread: (st.spread - t.promote_narrow).max(t.promote_spread_min),
                epoch: st.epoch + 1,
                fell_from: st.fell_from.filter(|b| *b > st.center + 1),
                ..st.clone()
            }
        } else if st.center > MIN_BAND && should_demote(t, st) {
            cues.push(Cue::Demoted(skill, st.center - 1));
            SkillState {
                center: st.center - 1,
                spread: (st.spread - t.demote_narrow).max(t.demote_spread_min),
                epoch: st.epoch + 1,
                fell_from: Some(st.center),
                ..st.clone()
            }
        } else {
            match st.window.clean_rate() {
                (Some(rate), n)
                    if n >= t.widen_min && rate > t.widen_at && st.spread < t.widen_max =>
                {
                    SkillState {
                        spread: (st.spread + t.widen_step).min(t.widen_max),
                        ..st.clone()
                    }
                }
                _ => st.clone(),
            }
        };
        s.skills.insert(skill, next);
    }
    (s, cues)
}

/// Promote: ≥ [`Tuning::promote_at`] clean over enough unassisted attempts at the center since the
/// last band change, and stretch attempts (if ≥ [`Tuning::min_stretch`]) ≥ [`Tuning::stretch_at`].
pub fn should_promote(t: &Tuning, st: &SkillState) -> bool {
    let (at, n) = st.window.at_center(st.center, st.epoch, Some(t.assist_eps));
    if n < st.evidence_needed(t) || at.unwrap_or(0.0) < t.promote_at {
        return false;
    }
    let (above, m) = st.window.above_center(st.center, st.epoch);
    let stretch_ok = above.unwrap_or(0.0) >= t.stretch_at;
    if t.repromote_needs_stretch && st.back_into_fallen_band() {
        return m >= t.min_stretch && stretch_ok;
    }
    m < t.min_stretch || stretch_ok
}

/// Demote: < [`Tuning::demote_below`] clean over ≥ [`Tuning::min_evidence`] attempts at the center
/// since the last band change (assisted or not).
pub fn should_demote(t: &Tuning, st: &SkillState) -> bool {
    let (at, n) = st.window.at_center(st.center, st.epoch, None);
    n >= t.min_evidence && at.unwrap_or(1.0) < t.demote_below
}

/// The frustration response: ease the skills that were pushing (exercised at or above their
/// center; the first one if none), narrow their spread, raise assists, go easy on skill picks for
/// a while, and ask for encouragement. Easing drops the center by 1, except after a *stretch*
/// attempt: then the stretch was the problem, not the center, so only the spread narrows
/// (dropping a sound center there caused re-climb oscillation in nat-han's simulator).
fn ease<S: Skill>(
    t: &Tuning,
    state: Profile<S>,
    exercising: &[(S, Band)],
    mut cues: Vec<Cue<S>>,
) -> (Profile<S>, Vec<Cue<S>>) {
    let mut s = state;
    cues.push(Cue::Encourage);
    for &(skill, _) in exercising {
        s.ensure(skill, t);
    }
    let pushing: Vec<(S, Band)> = exercising
        .iter()
        .copied()
        .filter(|(skill, band)| *band >= s.skills[skill].center)
        .collect();
    let targets: Vec<(S, Band)> = if pushing.is_empty() {
        exercising.iter().copied().take(1).collect()
    } else {
        pushing
    };
    if !s.calibrating() {
        for (skill, band) in targets {
            let st = &s.skills[&skill];
            let stretch = band > st.center;
            let center = if stretch {
                st.center
            } else {
                st.center.saturating_sub(1).max(MIN_BAND)
            };
            if center != st.center {
                cues.push(Cue::Eased(skill, center));
            }
            let next = SkillState {
                center,
                spread: (st.spread - t.frustration_narrow).max(t.demote_spread_min),
                epoch: if center != st.center {
                    st.epoch + 1
                } else {
                    st.epoch
                },
                fell_from: if center != st.center {
                    Some(st.center)
                } else {
                    st.fell_from
                },
                ..st.clone()
            };
            s.skills.insert(skill, next);
        }
    }
    s.assists = (s.assists + frustration_bump(t, s.assists)).min(1.0);
    s.frustration_cooldown = t.frustration_cooldown;
    (s, cues)
}

/// How much frustration raises the dial from `assists`, diminishing near the top.
fn frustration_bump(t: &Tuning, assists: f32) -> f32 {
    t.frustration_assists * (1.0 - assists)
}

fn calibration_attempt<S: Skill>(
    t: &Tuning,
    s: Profile<S>,
    attempt: &Attempt<S>,
    outcome: Outcome,
) -> (Profile<S>, Vec<Cue<S>>) {
    let Some(probed) = attempt.evidence.first().copied() else {
        return (s, Vec::new());
    };
    // No clean run to judge against yet: a slip counts as clean rather than misplacing a strong
    // learner by several bands.
    let calibration = s.calibration.record(Probe {
        band: probed.band,
        clean: outcome != Outcome::Struggle,
    });
    let mut s = Profile { calibration, ..s };
    let mut cues = Vec::new();
    if attempt.frustrated {
        let exercising: Vec<(S, Band)> =
            attempt.evidence.iter().map(|e| (e.skill, e.band)).collect();
        (s, cues) = ease(t, s, &exercising, cues);
    }
    if !s.calibration.complete(t) {
        return (s, cues);
    }
    let p = s.calibration.placement(t);
    for (skill, st) in s.skills.iter_mut() {
        let center = if *skill == probed.skill {
            p.band
        } else {
            p.band.saturating_sub(1).max(MIN_BAND)
        };
        *st = SkillState {
            center,
            epoch: st.epoch + 1,
            ..SkillState::new(center, t)
        };
    }
    s.assists = s.assists.max(p.assists);
    s.calibration.done = true;
    cues.push(Cue::Calibrated(p));
    (s, cues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    enum Sk {
        Listening,
        Speaking,
    }

    fn attempt(skill: Sk, band: Band, outcome: Outcome) -> Event<Sk> {
        Event::Attempted(Attempt {
            evidence: vec![Evidence {
                skill,
                band,
                outcome,
            }],
            misses: u32::from(outcome == Outcome::Struggle),
            assists: 0.0,
            frustrated: false,
        })
    }

    fn run(
        t: &Tuning,
        p: Profile<Sk>,
        events: impl IntoIterator<Item = Event<Sk>>,
    ) -> (Profile<Sk>, Vec<Cue<Sk>>) {
        events.into_iter().fold((p, Vec::new()), |(p, mut all), e| {
            let (p, cues) = reduce(t, p, e);
            all.extend(cues);
            (p, all)
        })
    }

    #[test]
    fn steady_clean_answers_at_the_center_promote() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Listening], 3, &t);
        let (p, cues) = run(
            &t,
            p,
            (0..t.min_evidence).map(|_| attempt(Sk::Listening, 3, Outcome::Clean)),
        );
        assert_eq!(p.skill(Sk::Listening).unwrap().center, 4);
        assert!(cues.contains(&Cue::Promoted(Sk::Listening, 4)));
    }

    #[test]
    fn assists_fade_before_difficulty_rises() {
        let t = Tuning::default();
        let p = Profile {
            assists: 0.3,
            ..Profile::calibrated(&[Sk::Listening], 3, &t)
        };
        let (p, cues) = run(
            &t,
            p,
            (0..t.min_evidence).map(|_| attempt(Sk::Listening, 3, Outcome::Clean)),
        );
        assert!(cues.is_empty(), "no promotion while the dial is up");
        assert!(p.assists < 0.3);
        assert_eq!(p.skill(Sk::Listening).unwrap().center, 3);
    }

    #[test]
    fn sustained_struggle_demotes_and_sets_up_hysteresis() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Speaking], 5, &t);
        let (p, cues) = run(
            &t,
            p,
            (0..t.min_evidence).map(|_| attempt(Sk::Speaking, 5, Outcome::Struggle)),
        );
        assert!(cues.contains(&Cue::Demoted(Sk::Speaking, 4)));
        let st = p.skill(Sk::Speaking).unwrap();
        assert_eq!(st.evidence_needed(&t), t.repromote_evidence);
    }

    #[test]
    fn slips_move_nothing() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Speaking], 5, &t);
        let (q, cues) = reduce(&t, p.clone(), attempt(Sk::Speaking, 5, Outcome::Slip));
        assert!(cues.is_empty());
        assert_eq!(q.assists, p.assists);
        assert_eq!(q.skill(Sk::Speaking).unwrap().center, 5);
    }

    #[test]
    fn frustration_eases_the_center_but_only_narrows_after_a_stretch() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Listening, Sk::Speaking], 5, &t);
        let (q, cues) = reduce(
            &t,
            p.clone(),
            Event::Frustrated {
                exercising: vec![(Sk::Speaking, 5)],
            },
        );
        assert_eq!(q.skill(Sk::Speaking).unwrap().center, 4);
        assert!(cues.contains(&Cue::Encourage) && cues.contains(&Cue::Eased(Sk::Speaking, 4)));
        assert!(q.frustrated() && q.assists > 0.0);

        let (q, _) = reduce(
            &t,
            p,
            Event::Frustrated {
                exercising: vec![(Sk::Speaking, 7)],
            },
        );
        let st = q.skill(Sk::Speaking).unwrap();
        assert_eq!(st.center, 5);
        assert!(st.spread < t.default_spread);
    }

    #[test]
    fn calibration_places_the_probed_skill_and_one_lower_for_the_rest() {
        let t = Tuning::default();
        let p = Profile::<Sk> {
            skills: [(Sk::Speaking, SkillState::new(1, &t))].into(),
            ..Profile::default()
        };
        let (p, cues) = run(
            &t,
            p,
            [
                attempt(Sk::Listening, 3, Outcome::Clean),
                attempt(Sk::Listening, 5, Outcome::Struggle),
            ],
        );
        assert!(!p.calibrating());
        assert!(matches!(
            cues.last(),
            Some(Cue::Calibrated(Placement { band: 3, .. }))
        ));
        assert_eq!(p.skill(Sk::Listening).unwrap().center, 3);
        assert_eq!(p.skill(Sk::Speaking).unwrap().center, 2);
    }

    #[test]
    fn a_new_skill_starts_just_under_the_played_ones() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Listening], 6, &t);
        let (p, _) = reduce(&t, p, attempt(Sk::Listening, 6, Outcome::Clean));
        let (p, _) = reduce(&t, p, Event::Unlocked(Sk::Speaking));
        assert_eq!(p.skill(Sk::Speaking).unwrap().center, 5);
    }

    #[test]
    fn round_trips_through_json() {
        let t = Tuning::default();
        let p = Profile::calibrated(&[Sk::Listening, Sk::Speaking], 4, &t);
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Profile<Sk>>(&json).unwrap(), p);
    }
}
