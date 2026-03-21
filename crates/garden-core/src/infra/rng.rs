// Implementation du trait Rng avec rand, seedable.

use crate::domain::rng::Rng;
use rand::rngs::StdRng;
use rand::{Rng as RandRng, SeedableRng};

/// Generateur de nombres aleatoires seedable.
pub struct SeededRng {
    inner: StdRng,
}

impl SeededRng {
    /// Cree un nouveau generateur a partir d'un seed.
    pub fn new(seed: u64) -> Self {
        Self {
            inner: StdRng::seed_from_u64(seed),
        }
    }
}

impl Rng for SeededRng {
    fn next_f32(&mut self) -> f32 {
        self.inner.gen::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rng::Rng;

    #[test]
    fn le_seeded_rng_est_reproductible() {
        // Deux generateurs avec le meme seed doivent produire la meme sequence.
        let mut rng1 = SeededRng::new(42);
        let mut rng2 = SeededRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_f32(), rng2.next_f32());
        }
    }

    #[test]
    fn le_seeded_rng_produit_des_valeurs_dans_0_1() {
        // Toutes les valeurs doivent etre dans [0.0, 1.0).
        let mut rng = SeededRng::new(123);

        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!(v >= 0.0, "valeur negative: {v}");
            assert!(v < 1.0, "valeur >= 1.0: {v}");
        }
    }

    #[test]
    fn des_seeds_differents_donnent_des_sequences_differentes() {
        // Deux seeds differents doivent produire des sequences differentes.
        let mut rng1 = SeededRng::new(42);
        let mut rng2 = SeededRng::new(43);

        let seq1: Vec<f32> = (0..100).map(|_| rng1.next_f32()).collect();
        let seq2: Vec<f32> = (0..100).map(|_| rng2.next_f32()).collect();

        assert_ne!(seq1, seq2);
    }
}
