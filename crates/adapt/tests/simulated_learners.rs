//! Synthetic learners played through the real reducer: the engine should settle each one near
//! its true level, in the clean-rate range it aims for. A first harness; the flashcard version
//! needs FSRS in the loop (see the plan's milestone 4).

use adapt::{Attempt, Event, Evidence, Outcome, Profile, Tuning, next_band, pick_skill, reduce};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Sk {
    Recognition,
    Production,
}

/// p(clean) is logistic in the band: `ability` is the band answered right half the time.
struct Learner {
    ability: [f32; 2],
    slope: f32,
}

impl Learner {
    fn p_clean(&self, skill: Sk, band: u8, assists: f32) -> f32 {
        let a = self.ability[skill as usize];
        let p = 1.0 / (1.0 + ((band as f32 - a) / self.slope).exp());
        // A full dial removes half the remaining failure chance.
        p + (1.0 - p) * 0.5 * assists
    }
}

fn play(learner: &Learner, seed: u64, n: usize) -> (Profile<Sk>, f32) {
    let t = Tuning::default();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut p = Profile::default();
    let mut late_clean = 0;
    for i in 0..n {
        let skill = pick_skill(&t, &p, &[Sk::Recognition, Sk::Production], &mut rng).unwrap();
        let band = next_band(&t, &p, skill, &mut rng);
        let clean = rng.random::<f32>() < learner.p_clean(skill, band, p.assists);
        if i >= n / 2 && clean {
            late_clean += 1;
        }
        let outcome = if clean {
            Outcome::Clean
        } else {
            Outcome::Struggle
        };
        let attempt = Attempt {
            evidence: vec![Evidence {
                skill,
                band,
                outcome,
            }],
            misses: u32::from(!clean),
            assists: p.assists,
            frustrated: false,
        };
        p = reduce(&t, p, Event::Attempted(attempt)).0;
    }
    (p, late_clean as f32 / (n - n / 2) as f32)
}

#[test]
fn settles_each_skill_near_the_learners_level() {
    let learner = Learner {
        ability: [7.0, 4.0],
        slope: 0.8,
    };
    for seed in 0..5 {
        let (p, late_clean) = play(&learner, seed, 400);
        let rec = p.skill(Sk::Recognition).unwrap().center;
        let prod = p.skill(Sk::Production).unwrap().center;
        assert!(
            (5..=8).contains(&rec),
            "seed {seed}: recognition settled at {rec}"
        );
        assert!(
            (2..=5).contains(&prod),
            "seed {seed}: production settled at {prod}"
        );
        assert!(rec > prod, "seed {seed}: the stronger skill sits higher");
        assert!(
            (0.55..=0.95).contains(&late_clean),
            "seed {seed}: clean rate {late_clean}"
        );
    }
}
