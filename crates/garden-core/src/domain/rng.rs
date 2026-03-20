/// Trait pour la generation de nombres aleatoires, injecte dans la logique domaine.
/// Implemente dans infra avec un RNG concret.
pub trait Rng {
    /// Retourne un f32 aleatoire dans [0.0, 1.0).
    fn next_f32(&mut self) -> f32;
}
