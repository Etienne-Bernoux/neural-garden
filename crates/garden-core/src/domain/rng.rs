/// Trait pour la generation de nombres aleatoires, injecte dans la logique domaine.
/// Implemente dans infra avec un RNG concret.
pub trait Rng {
    /// Retourne un f32 aleatoire dans [0.0, 1.0).
    fn next_f32(&mut self) -> f32;
}

#[cfg(test)]
pub mod test_utils {
    use super::Rng;

    /// Rng de test deterministe avec un pas configurable.
    pub struct MockRng {
        value: f32,
        step: f32,
    }

    impl MockRng {
        pub fn new(start: f32, step: f32) -> Self {
            Self { value: start, step }
        }
    }

    impl Rng for MockRng {
        fn next_f32(&mut self) -> f32 {
            let v = self.value;
            self.value = (self.value + self.step) % 1.0;
            v
        }
    }
}
