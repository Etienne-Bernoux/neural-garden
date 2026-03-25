// Traits definissant les facettes d'une entite plante.
// Chaque sous-trait isole une responsabilite ; le super-trait PlantEntity les regroupe.

use super::events::DomainEvent;
use super::plant::{Biomass, CellSlot, Energy, GeneticTraits, Lineage, PlantState, Pos, Vitality};
use super::stages::GrowthStage;

/// Identite d'une plante — id, lignee, genealogie.
pub trait PlantIdentity {
    fn id(&self) -> u64;
    fn lineage(&self) -> &Lineage;
    fn ancestors(&self) -> &[u64];
    fn parent_id(&self) -> Option<u64>;
    fn generation_depth(&self) -> usize;
}

/// Stats vitales — vie, energie, biomasse, etat.
pub trait PlantVitals {
    fn vitality(&self) -> &Vitality;
    fn energy(&self) -> &Energy;
    fn biomass(&self) -> &Biomass;
    fn state(&self) -> PlantState;
    fn is_dead(&self) -> bool;
    fn age(&self) -> u32;

    // Mutations
    fn damage(&mut self, amount: f32);
    fn heal(&mut self, amount: f32);
    fn consume_energy(&mut self, amount: f32);
    fn gain_energy(&mut self, amount: f32);
    fn tick(&mut self);
    fn update_state(&mut self) -> Option<DomainEvent>;
    fn start_decomposition(&mut self, ticks: u32);
    fn decompose_tick(&mut self, total_ticks: u32) -> (f32, f32);
    fn is_fully_decomposed(&self) -> bool;
    fn decomposition_remaining(&self) -> u32;
    fn carbon_to_release(&self) -> f32;
    fn nitrogen_to_release(&self) -> f32;
}

/// Presence spatiale — 3 couches (footprint, canopy, roots).
pub trait PlantSpatial {
    fn footprint(&self) -> &[Pos];
    /// Positions de la canopee (extraites des CellSlots).
    fn canopy(&self) -> Vec<Pos>;
    /// Positions des racines (extraites des CellSlots).
    fn roots(&self) -> Vec<Pos>;
    /// CellSlots de la canopee avec niveaux.
    fn canopy_slots(&self) -> &[CellSlot];
    /// CellSlots des racines avec niveaux.
    fn root_slots(&self) -> &[CellSlot];
    /// Stade de croissance actuel.
    fn growth_stage(&self) -> GrowthStage;
    fn max_canopy(&self) -> usize;
    fn max_roots(&self) -> usize;

    // Mutations
    fn grow_footprint(&mut self, pos: Pos) -> DomainEvent;
    fn grow_canopy(&mut self, pos: Pos) -> Option<DomainEvent>;
    fn grow_roots(&mut self, pos: Pos) -> Option<DomainEvent>;
    fn shrink(&mut self) -> Option<DomainEvent>;
    fn remove_footprint_cell(&mut self, pos: &Pos) -> bool;
    fn germinate(&mut self) -> Option<DomainEvent>;
}

/// Traits genetiques — caracteristiques heritees.
pub trait PlantGenetics {
    fn genetics(&self) -> &GeneticTraits;
}

/// Reproduction — progression vers les graines.
pub trait PlantReproduction {
    fn seed_progress(&self) -> f32;
    fn add_seed_progress(&mut self, amount: f32);
    fn consume_seed_progress(&mut self, amount: f32);
}

/// Super-trait regroupant tous les aspects d'une entite plante.
/// Send + Sync requis pour la parallelisation (rayon).
pub trait PlantEntity:
    PlantIdentity + PlantVitals + PlantSpatial + PlantGenetics + PlantReproduction + Send + Sync
{
}
