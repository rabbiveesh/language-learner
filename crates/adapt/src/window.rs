//! A skill's rolling window of recent outcomes.

use serde::{Deserialize, Serialize};

use crate::band::Band;

/// How one attempt went for one skill, as the app judged it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// As well as the attempt expected. Time never matters here: slow but right is clean.
    Clean,
    /// Worse than the attempt expected.
    Struggle,
    /// A miss the app judged careless (e.g. one quick miss right after a run of clean answers).
    /// Counts neither for nor against: it's left out of every rate and moves nothing.
    Slip,
}

/// One attempt, as seen by one of the skills it exercised.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowEntry {
    pub outcome: Outcome,
    /// The band this skill was exercised at.
    pub band: Band,
    /// The skill's center when the attempt started.
    pub center: Band,
    /// The skill's epoch when the attempt started. The epoch bumps on every band change, so old
    /// entries never count as evidence for the new center ("fresh evidence").
    pub epoch: u32,
    /// The assist dial when the attempt started. Only (nearly) unassisted attempts count toward
    /// a promotion.
    pub assists: f32,
}

impl WindowEntry {
    fn counts(&self) -> bool {
        self.outcome != Outcome::Slip
    }
}

/// The last `capacity` entries ([`crate::Tuning::window`]). Immutable: [`RollingWindow::push`]
/// returns a new window.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RollingWindow {
    pub entries: Vec<WindowEntry>,
}

/// `(clean fraction, count)` over the entries that count; `None` with no data.
fn rate<'a>(it: impl Iterator<Item = &'a WindowEntry>) -> (Option<f32>, usize) {
    let (mut clean, mut n) = (0usize, 0usize);
    for e in it.filter(|e| e.counts()) {
        n += 1;
        if e.outcome == Outcome::Clean {
            clean += 1;
        }
    }
    if n == 0 {
        (None, 0)
    } else {
        (Some(clean as f32 / n as f32), n)
    }
}

impl RollingWindow {
    pub fn push(&self, entry: WindowEntry, capacity: usize) -> Self {
        let mut entries = self.entries.clone();
        entries.push(entry);
        if entries.len() > capacity {
            entries.drain(..entries.len() - capacity);
        }
        RollingWindow { entries }
    }

    /// Clean rate over everything remembered (slips excluded).
    pub fn clean_rate(&self) -> (Option<f32>, usize) {
        rate(self.entries.iter())
    }

    /// Clean rate at `center` in `epoch`. `max_assists` keeps only attempts made with the dial at
    /// or below it.
    pub fn at_center(
        &self,
        center: Band,
        epoch: u32,
        max_assists: Option<f32>,
    ) -> (Option<f32>, usize) {
        rate(self.entries.iter().filter(|e| {
            e.epoch == epoch && e.band == center && max_assists.is_none_or(|eps| e.assists <= eps)
        }))
    }

    /// Clean rate of everything remembered at `band` or harder, any epoch.
    pub fn at_or_above(&self, band: Band) -> (Option<f32>, usize) {
        rate(self.entries.iter().filter(|e| e.band >= band))
    }

    /// Clean rate of stretch attempts (above the center) in `epoch`.
    pub fn above_center(&self, center: Band, epoch: u32) -> (Option<f32>, usize) {
        rate(
            self.entries
                .iter()
                .filter(|e| e.epoch == epoch && e.band > center),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(outcome: Outcome) -> WindowEntry {
        WindowEntry {
            outcome,
            band: 3,
            center: 3,
            epoch: 0,
            assists: 0.0,
        }
    }

    #[test]
    fn slips_are_left_out_of_every_rate() {
        let w = [Outcome::Clean, Outcome::Slip, Outcome::Struggle]
            .into_iter()
            .fold(RollingWindow::default(), |w, o| w.push(entry(o), 20));
        assert_eq!(w.clean_rate(), (Some(0.5), 2));
    }

    #[test]
    fn keeps_only_the_last_capacity_entries() {
        let w = (0..30).fold(RollingWindow::default(), |w, _| {
            w.push(entry(Outcome::Clean), 20)
        });
        assert_eq!(w.entries.len(), 20);
    }
}
