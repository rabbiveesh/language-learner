//! The disguised placement test: the first attempts.
//!
//! Probes exercise one skill. The first is at [`Tuning::probe_start`]; a clean probe moves the
//! next one up [`Tuning::probe_up`] bands, a struggle down [`Tuning::probe_down`]. It stops as soon
//! as ability is bracketed (a clean and a struggle), or the learner bottoms out or tops out, and
//! after [`Tuning::max_probes`] at most. Then [`Calibration::placement`] picks the starting band
//! and assists.
//!
//! Not ported yet: nat-han's "fast clean clears raise the placement" rule, which needs a par time
//! per attempt. Add it here if response times turn out to separate learners.

use serde::{Deserialize, Serialize};

use crate::band::{Band, MAX_BAND, MIN_BAND, clamp_band};
use crate::tuning::Tuning;

/// One probe's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Probe {
    pub band: Band,
    pub clean: bool,
}

/// Where calibration landed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    /// Starting center for the probed skill; the others start one lower.
    pub band: Band,
    pub assists: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Calibration {
    pub probes: Vec<Probe>,
    /// Set once the placement has been applied to the profile.
    pub done: bool,
}

impl Calibration {
    /// Calibration that's already over (for profiles set up directly, e.g. in tests).
    pub fn finished() -> Self {
        Calibration {
            probes: Vec::new(),
            done: true,
        }
    }

    /// Band of the next probe.
    pub fn next_band(&self, t: &Tuning) -> Band {
        match self.probes.last() {
            None => t.probe_start,
            Some(p) => clamp_band(p.band as i32 + if p.clean { t.probe_up } else { -t.probe_down }),
        }
    }

    pub fn record(&self, probe: Probe) -> Self {
        let mut probes = self.probes.clone();
        probes.push(probe);
        Calibration {
            probes,
            done: self.done,
        }
    }

    /// Enough probes to place the learner.
    pub fn complete(&self, t: &Tuning) -> bool {
        let n = self.probes.len();
        if n >= t.max_probes {
            return true;
        }
        let Some(last) = self.probes.last().filter(|_| n >= t.min_probes) else {
            return false;
        };
        let bracketed = self.probes.iter().any(|p| p.clean) && self.probes.iter().any(|p| !p.clean);
        let at_floor = !last.clean && last.band <= MIN_BAND;
        let at_ceiling = last.clean && last.band >= MAX_BAND;
        bracketed || at_floor || at_ceiling
    }

    /// Starting band and dial from the probes so far: the highest clean band (band 1 if none).
    pub fn placement(&self, t: &Tuning) -> Placement {
        let band = self
            .probes
            .iter()
            .filter(|p| p.clean)
            .map(|p| p.band)
            .max()
            .unwrap_or(MIN_BAND);
        let any_clean = self.probes.iter().any(|p| p.clean);
        let any_struggle = self.probes.iter().any(|p| !p.clean);
        let assists = if !any_clean {
            t.placed_assists_none_clean
        } else if any_struggle {
            t.placed_assists_some_struggle
        } else {
            t.placed_assists_all_clean
        };
        Placement { band, assists }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(band: Band, clean: bool) -> Probe {
        Probe { band, clean }
    }

    #[test]
    fn walks_up_two_down_one_and_stops_when_bracketed() {
        let t = Tuning::default();
        let c = Calibration::default();
        assert_eq!(c.next_band(&t), 3);
        let c = c.record(probe(3, true));
        assert_eq!(c.next_band(&t), 5);
        let c = c.record(probe(5, false));
        assert_eq!(c.next_band(&t), 4);
        assert!(c.complete(&t));
        assert_eq!(
            c.placement(&t),
            Placement {
                band: 3,
                assists: t.placed_assists_some_struggle
            }
        );
    }

    #[test]
    fn floor_and_ceiling_clamp() {
        let t = Tuning::default();
        assert_eq!(
            Calibration::default().record(probe(1, false)).next_band(&t),
            1
        );
        assert_eq!(
            Calibration::default().record(probe(10, true)).next_band(&t),
            10
        );
    }
}
