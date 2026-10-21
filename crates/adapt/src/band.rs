//! Difficulty bands and how the next one is drawn.

use rand::Rng;

/// Difficulty band of one skill: 1 (gentlest) ..= 10 (hardest).
pub type Band = u8;
pub const MIN_BAND: Band = 1;
pub const MAX_BAND: Band = 10;

/// Clamp any integer into `MIN_BAND..=MAX_BAND`.
pub fn clamp_band(b: i32) -> Band {
    b.clamp(MIN_BAND as i32, MAX_BAND as i32) as Band
}

/// Probability of each band 1..=10 (index 0 = band 1) for a skill centered at `center` with
/// `spread` (0..1). The center gets `0.9 − 0.6·spread`; the rest goes to ±1 (`0.05 + 0.15·s`),
/// ±2 (`0.1·s − 0.005`) and ±3 (`0.1·(s − 0.5)`), split evenly between the sides. A side that
/// falls off the 1..=10 range folds back one step toward the center. Then normalized to sum to 1
/// (so the center really gets ~95% at spread 0, ~47% at spread 1). The same formula all three
/// games use.
pub fn band_distribution(center: Band, spread: f32) -> [f32; 10] {
    let s = spread.clamp(0.0, 1.0);
    let center = center.clamp(MIN_BAND, MAX_BAND) as i32;
    let offsets = [
        (0, 0.9 - 0.6 * s),
        (1, 0.05 + 0.15 * s),
        (2, (0.1 * s - 0.005).max(0.0)),
        (3, (0.1 * (s - 0.5)).max(0.0)),
    ];
    let mut raw = [0.0f32; 10];
    let fold = |b: i32, toward: i32| -> usize {
        let b = if (MIN_BAND as i32..=MAX_BAND as i32).contains(&b) {
            b
        } else {
            b - toward
        };
        (b.clamp(MIN_BAND as i32, MAX_BAND as i32) - 1) as usize
    };
    for (d, w) in offsets {
        if d == 0 {
            raw[(center - 1) as usize] += w;
        } else {
            raw[fold(center + d, 1)] += w / 2.0;
            raw[fold(center - d, -1)] += w / 2.0;
        }
    }
    let total: f32 = raw.iter().sum();
    for v in &mut raw {
        *v /= total;
    }
    raw
}

/// Draw a band from a [`band_distribution`].
pub fn sample_band(dist: &[f32; 10], rng: &mut impl Rng) -> Band {
    let r: f32 = rng.random();
    let mut acc = 0.0;
    for (i, p) in dist.iter().enumerate() {
        acc += p;
        if r < acc {
            return (i + 1) as Band;
        }
    }
    MAX_BAND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mid_range_weights_match_the_games() {
        // Raw weights 0.3 / 0.2 / 0.095 / 0.05 (sum 0.645), normalized.
        let d = band_distribution(5, 1.0);
        let t = 0.645;
        assert!((d[4] - 0.3 / t).abs() < 1e-5);
        assert!((d[3] - 0.1 / t).abs() < 1e-5 && (d[5] - 0.1 / t).abs() < 1e-5);
        assert!((d[2] - 0.0475 / t).abs() < 1e-5);
        assert!((d[1] - 0.025 / t).abs() < 1e-5);
    }

    #[test]
    fn edges_fold_back_and_sum_to_one() {
        for c in MIN_BAND..=MAX_BAND {
            let d = band_distribution(c, 0.8);
            assert!((d.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            assert_eq!(
                d[(c - 1) as usize],
                d.iter().cloned().fold(0.0, f32::max),
                "center is the mode"
            );
        }
    }
}
