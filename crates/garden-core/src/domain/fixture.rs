// Plante artificielle deterministe pour la pepiniere (tests et benchmarks).

use super::events::{DomainEvent, GrowthLayer};
use super::plant::{
    Biomass, Energy, ExudateType, GeneticTraits, Lineage, PlantState, Pos, Vitality,
};
use super::traits::{
    PlantEntity, PlantGenetics, PlantIdentity, PlantReproduction, PlantSpatial, PlantVitals,
};

/// Plante artificielle pour la pepiniere.
/// Comportement deterministe, immortelle, taille fixe.
pub struct FixturePlant {
    id: u64,
    #[allow(dead_code)]
    position: Pos,
    footprint: Vec<Pos>,
    canopy: Vec<Pos>,
    roots: Vec<Pos>,
    genetics: GeneticTraits,
    lineage: Lineage,
    vitality: Vitality,
    energy: Energy,
    biomass: Biomass,
}

impl FixturePlant {
    pub fn new(id: u64, position: Pos, exudate_type: ExudateType, biomass_size: u16) -> Self {
        let genetics = GeneticTraits::new(40, 0.5, exudate_type, 8, 10.0, 10.0);
        let lineage = Lineage::new(id, 0);

        // Generer un footprint simple autour de la position
        let mut footprint = vec![position];
        let mut canopy = vec![position];
        let mut roots = vec![position];

        for i in 1..biomass_size.min(10) {
            let p = Pos {
                x: position.x + (i % 3),
                y: position.y + (i / 3),
            };
            footprint.push(p);
            canopy.push(p);
            roots.push(p);
        }

        Self {
            id,
            position,
            footprint,
            canopy,
            roots,
            genetics,
            lineage,
            vitality: Vitality::new(100.0, 100.0),
            energy: Energy::new(100.0, 100.0),
            biomass: Biomass::new(biomass_size, 40),
        }
    }
}

// --- Implementations des sous-traits PlantEntity pour FixturePlant ---

impl PlantIdentity for FixturePlant {
    fn id(&self) -> u64 {
        self.id
    }

    fn lineage(&self) -> &Lineage {
        &self.lineage
    }

    fn ancestors(&self) -> &[u64] {
        &[]
    }

    fn parent_id(&self) -> Option<u64> {
        None
    }

    fn generation_depth(&self) -> usize {
        0
    }
}

impl PlantVitals for FixturePlant {
    fn vitality(&self) -> &Vitality {
        &self.vitality
    }

    fn energy(&self) -> &Energy {
        &self.energy
    }

    fn biomass(&self) -> &Biomass {
        &self.biomass
    }

    fn state(&self) -> PlantState {
        PlantState::Mature
    }

    fn is_dead(&self) -> bool {
        false
    }

    fn age(&self) -> u32 {
        0
    }

    // Mutations — toutes no-op (immortelle, invulnerable)
    fn damage(&mut self, _amount: f32) {}
    fn heal(&mut self, _amount: f32) {}
    fn consume_energy(&mut self, _amount: f32) {}
    fn gain_energy(&mut self, _amount: f32) {}
    fn tick(&mut self) {}

    fn update_state(&mut self) -> Option<DomainEvent> {
        None
    }

    fn start_decomposition(&mut self, _ticks: u32) {}

    fn decompose_tick(&mut self, _total_ticks: u32) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn is_fully_decomposed(&self) -> bool {
        false
    }

    fn decomposition_remaining(&self) -> u32 {
        0
    }

    fn carbon_to_release(&self) -> f32 {
        0.0
    }

    fn nitrogen_to_release(&self) -> f32 {
        0.0
    }
}

impl PlantSpatial for FixturePlant {
    fn footprint(&self) -> &[Pos] {
        &self.footprint
    }

    fn canopy(&self) -> &[Pos] {
        &self.canopy
    }

    fn roots(&self) -> &[Pos] {
        &self.roots
    }

    fn max_canopy(&self) -> usize {
        self.footprint.len() * 4
    }

    fn max_roots(&self) -> usize {
        self.footprint.len() * 5
    }

    // Mutations — toutes no-op (taille fixe)
    fn grow_footprint(&mut self, pos: Pos) -> DomainEvent {
        // No-op mais doit retourner un event (signature du trait)
        DomainEvent::Grew {
            plant_id: self.id,
            cell: pos,
            layer: GrowthLayer::Footprint,
        }
    }

    fn grow_canopy(&mut self, _pos: Pos) -> Option<DomainEvent> {
        None
    }

    fn grow_roots(&mut self, _pos: Pos) -> Option<DomainEvent> {
        None
    }

    fn shrink(&mut self) -> Option<DomainEvent> {
        None
    }

    fn remove_footprint_cell(&mut self, _pos: &Pos) -> bool {
        false
    }

    fn germinate(&mut self) -> Option<DomainEvent> {
        None
    }
}

impl PlantGenetics for FixturePlant {
    fn genetics(&self) -> &GeneticTraits {
        &self.genetics
    }
}

impl PlantReproduction for FixturePlant {
    fn seed_progress(&self) -> f32 {
        0.0
    }

    fn add_seed_progress(&mut self, _amount: f32) {}

    fn consume_seed_progress(&mut self, _amount: f32) {}
}

impl PlantEntity for FixturePlant {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fixture() -> FixturePlant {
        FixturePlant::new(1, Pos { x: 5, y: 5 }, ExudateType::Carbon, 5)
    }

    #[test]
    fn fixture_est_immortelle() {
        let mut f = test_fixture();
        // Infliger des degats massifs
        f.damage(1000.0);
        assert!(!f.is_dead(), "la fixture ne doit jamais mourir");
    }

    #[test]
    fn fixture_a_un_etat_mature() {
        let f = test_fixture();
        assert_eq!(f.state(), PlantState::Mature);
    }

    #[test]
    fn fixture_ne_produit_pas_de_graines() {
        let f = test_fixture();
        assert_eq!(f.seed_progress(), 0.0);
    }
}
