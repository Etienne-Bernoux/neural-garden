// Value objects de l'entite Plant.

use super::events::{DomainEvent, GrowthLayer};

/// Flottant borne dans [0.0, cap]. Base commune pour Vitalite et Energie.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundedF32(f32);

impl BoundedF32 {
    pub fn new(value: f32, cap: f32) -> Self {
        Self(value.clamp(0.0, cap))
    }

    pub fn add(&self, amount: f32, cap: f32) -> Self {
        Self((self.0 + amount).clamp(0.0, cap))
    }

    pub fn sub(&self, amount: f32) -> Self {
        Self((self.0 - amount).max(0.0))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn clamp_to(&self, new_cap: f32) -> Self {
        Self(self.0.clamp(0.0, new_cap))
    }
}

/// Vitalite — sante globale de la plante.
#[derive(Debug, Clone, PartialEq)]
pub struct Vitality(BoundedF32);

impl Vitality {
    pub fn new(value: f32, cap: f32) -> Self {
        Self(BoundedF32::new(value, cap))
    }

    pub fn add(&self, amount: f32, cap: f32) -> Self {
        Self(self.0.add(amount, cap))
    }

    pub fn sub(&self, amount: f32) -> Self {
        Self(self.0.sub(amount))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn value(&self) -> f32 {
        self.0.value()
    }

    pub fn clamp_to(&self, new_cap: f32) -> Self {
        Self(self.0.clamp_to(new_cap))
    }
}

/// Energie — carburant pour les actions.
#[derive(Debug, Clone, PartialEq)]
pub struct Energy(BoundedF32);

impl Energy {
    pub fn new(value: f32, cap: f32) -> Self {
        Self(BoundedF32::new(value, cap))
    }

    pub fn add(&self, amount: f32, cap: f32) -> Self {
        Self(self.0.add(amount, cap))
    }

    pub fn sub(&self, amount: f32) -> Self {
        Self(self.0.sub(amount))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn value(&self) -> f32 {
        self.0.value()
    }

    pub fn clamp_to(&self, new_cap: f32) -> Self {
        Self(self.0.clamp_to(new_cap))
    }
}

/// Biomasse — u16 borne dans [0, max_size].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biomass(u16);

impl Biomass {
    pub fn new(value: u16, max_size: u16) -> Self {
        Self(value.min(max_size))
    }

    pub fn add(&self, amount: u16, max_size: u16) -> Self {
        Self(self.0.saturating_add(amount).min(max_size))
    }

    pub fn sub(&self, amount: u16) -> Self {
        Self(self.0.saturating_sub(amount))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn clamp_to(&self, new_max: u16) -> Self {
        Self(self.0.min(new_max))
    }
}

/// Lignee — identifie la lignee d'une plante a travers les generations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    id: u64,
    generation: u64,
}

impl Lineage {
    pub fn new(id: u64, generation: u64) -> Self {
        Self { id, generation }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// Plafond dynamique de vitalite base sur la biomasse et un facteur genetique.
pub fn vitality_cap(biomass: &Biomass, genetic_factor: f32) -> f32 {
    biomass.value() as f32 * genetic_factor
}

/// Plafond dynamique d'energie base sur la biomasse et un facteur genetique.
pub fn energy_cap(biomass: &Biomass, genetic_factor: f32) -> f32 {
    biomass.value() as f32 * genetic_factor
}

// --- Value objects de l'entite Plant ---

/// Type d'exsudat chimique produit par une plante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExudateType {
    Carbon,
    Nitrogen,
}

/// Etat du cycle de vie d'une plante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantState {
    Seed,
    Growing,
    Mature,
    Stressed,
    Dying,
    Dead,
    Decomposing,
}

/// Position sur la grille 2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

/// Traits genetiques determinant les caracteristiques d'une plante.
/// Les valeurs sont validees et clampees a la construction.
#[derive(Debug, Clone)]
pub struct GeneticTraits {
    max_size: u16,
    carbon_nitrogen_ratio: f32,
    exudate_type: ExudateType,
    hidden_size: u8,
    vitality_factor: f32,
    energy_factor: f32,
}

impl GeneticTraits {
    /// Constructeur avec validation et clamp des bornes.
    /// - max_size: [15, 40]
    /// - carbon_nitrogen_ratio: [0.3, 0.9]
    /// - hidden_size: [6, 14]
    /// - vitality_factor et energy_factor: [0.1, 20.0]
    pub fn new(
        max_size: u16,
        carbon_nitrogen_ratio: f32,
        exudate_type: ExudateType,
        hidden_size: u8,
        vitality_factor: f32,
        energy_factor: f32,
    ) -> Self {
        Self {
            max_size: max_size.clamp(15, 40),
            carbon_nitrogen_ratio: carbon_nitrogen_ratio.clamp(0.3, 0.9),
            exudate_type,
            hidden_size: hidden_size.clamp(6, 14),
            vitality_factor: vitality_factor.clamp(0.1, 20.0),
            energy_factor: energy_factor.clamp(0.1, 20.0),
        }
    }

    // --- Getters ---

    pub fn max_size(&self) -> u16 {
        self.max_size
    }

    pub fn carbon_nitrogen_ratio(&self) -> f32 {
        self.carbon_nitrogen_ratio
    }

    pub fn exudate_type(&self) -> ExudateType {
        self.exudate_type
    }

    pub fn hidden_size(&self) -> u8 {
        self.hidden_size
    }

    pub fn vitality_factor(&self) -> f32 {
        self.vitality_factor
    }

    pub fn energy_factor(&self) -> f32 {
        self.energy_factor
    }
}

// --- Entite Plant ---

/// Une plante vivant sur la grille.
/// Modele 3 couches : footprint (emprise au sol, exclusive),
/// canopy (canopee aerienne, partagee), roots (racines, partagees).
#[derive(Debug, Clone)]
pub struct Plant {
    id: u64,
    state: PlantState,
    age: u32,
    vitality: Vitality,
    energy: Energy,
    biomass: Biomass,
    footprint: Vec<Pos>,
    canopy: Vec<Pos>,
    roots: Vec<Pos>,
    genetics: GeneticTraits,
    lineage: Lineage,
    decomposition_remaining: u32,
    carbon_to_release: f32,
    nitrogen_to_release: f32,
    ancestors: Vec<u64>,
    seed_progress: f32,
}

impl Plant {
    /// Cree une nouvelle plante sous forme de graine a la position donnee.
    /// La position initiale est presente dans les 3 couches (footprint, canopy, roots).
    pub fn new(id: u64, position: Pos, genetics: GeneticTraits, lineage: Lineage) -> Self {
        let biomass = Biomass::new(1, genetics.max_size());
        let v_cap = vitality_cap(&biomass, genetics.vitality_factor());
        let e_cap = energy_cap(&biomass, genetics.energy_factor());
        Self {
            id,
            state: PlantState::Seed,
            age: 0,
            vitality: Vitality::new(v_cap, v_cap),
            energy: Energy::new(e_cap, e_cap),
            biomass,
            footprint: vec![position],
            canopy: vec![position],
            roots: vec![position],
            genetics,
            lineage,
            decomposition_remaining: 0,
            carbon_to_release: 0.0,
            nitrogen_to_release: 0.0,
            ancestors: Vec::new(),
            seed_progress: 0.0,
        }
    }

    /// Cree une plante-graine issue d'un parent, avec la genealogie heritee.
    /// Conserve les 20 derniers ancetres (du plus ancien au parent direct).
    pub fn with_parent(
        id: u64,
        position: Pos,
        genetics: GeneticTraits,
        lineage: Lineage,
        parent_id: u64,
        parent_ancestors: &[u64],
    ) -> Self {
        let mut ancestors = parent_ancestors.to_vec();
        ancestors.push(parent_id);
        if ancestors.len() > 20 {
            ancestors.drain(0..ancestors.len() - 20);
        }
        let mut plant = Self::new(id, position, genetics, lineage);
        plant.ancestors = ancestors;
        plant
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> PlantState {
        self.state
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn vitality(&self) -> &Vitality {
        &self.vitality
    }

    pub fn energy(&self) -> &Energy {
        &self.energy
    }

    pub fn biomass(&self) -> &Biomass {
        &self.biomass
    }

    /// Emprise au sol — cellules exclusives (presence physique).
    pub fn footprint(&self) -> &[Pos] {
        &self.footprint
    }

    /// Canopee aerienne — cellules partagees.
    pub fn canopy(&self) -> &[Pos] {
        &self.canopy
    }

    /// Racines sous-sol — cellules partagees.
    pub fn roots(&self) -> &[Pos] {
        &self.roots
    }

    /// Nombre maximum de cellules de canopee (4x l'emprise).
    pub fn max_canopy(&self) -> usize {
        self.footprint.len() * 4
    }

    /// Nombre maximum de cellules de racines (5x l'emprise).
    pub fn max_roots(&self) -> usize {
        self.footprint.len() * 5
    }

    pub fn genetics(&self) -> &GeneticTraits {
        &self.genetics
    }

    pub fn lineage(&self) -> &Lineage {
        &self.lineage
    }

    pub fn is_dead(&self) -> bool {
        self.state == PlantState::Dead || self.state == PlantState::Decomposing
    }

    /// Avance l'age d'un tick.
    pub fn tick(&mut self) {
        self.age += 1;
    }

    /// Tente de germer. Retourne l'evenement Germinated si la plante etait une graine.
    pub fn germinate(&mut self) -> Option<DomainEvent> {
        if self.state == PlantState::Seed {
            self.state = PlantState::Growing;
            Some(DomainEvent::Germinated {
                plant_id: self.id,
                position: self.footprint[0],
            })
        } else {
            None
        }
    }

    /// Pousse l'emprise au sol dans une nouvelle cellule. Incremente la biomasse.
    pub fn grow_footprint(&mut self, pos: Pos) -> DomainEvent {
        self.footprint.push(pos);
        self.biomass = self.biomass.add(1, self.genetics.max_size());
        // Re-clamper les stats aux nouveaux plafonds
        let v_cap = vitality_cap(&self.biomass, self.genetics.vitality_factor());
        let e_cap = energy_cap(&self.biomass, self.genetics.energy_factor());
        self.vitality = self.vitality.clamp_to(v_cap);
        self.energy = self.energy.clamp_to(e_cap);
        DomainEvent::Grew {
            plant_id: self.id,
            cell: pos,
            layer: GrowthLayer::Footprint,
        }
    }

    /// Pousse la canopee aerienne dans une nouvelle cellule.
    /// Retourne None si la limite max_canopy est atteinte.
    pub fn grow_canopy(&mut self, pos: Pos) -> Option<DomainEvent> {
        if self.canopy.len() >= self.max_canopy() {
            return None;
        }
        self.canopy.push(pos);
        Some(DomainEvent::Grew {
            plant_id: self.id,
            cell: pos,
            layer: GrowthLayer::Canopy,
        })
    }

    /// Pousse les racines dans une nouvelle cellule.
    /// Retourne None si la limite max_roots est atteinte.
    pub fn grow_roots(&mut self, pos: Pos) -> Option<DomainEvent> {
        if self.roots.len() >= self.max_roots() {
            return None;
        }
        self.roots.push(pos);
        Some(DomainEvent::Grew {
            plant_id: self.id,
            cell: pos,
            layer: GrowthLayer::Roots,
        })
    }

    /// Retrecit en retirant la derniere cellule d'emprise (garde au moins 1).
    /// Retourne l'evenement Shrank si une cellule a ete retiree.
    pub fn shrink(&mut self) -> Option<DomainEvent> {
        if self.footprint.len() > 1 {
            let removed_cell = self.footprint.pop().expect("footprint non vide");
            self.biomass = self.biomass.sub(1);
            // Re-clamper les stats aux nouveaux plafonds
            let v_cap = vitality_cap(&self.biomass, self.genetics.vitality_factor());
            let e_cap = energy_cap(&self.biomass, self.genetics.energy_factor());
            self.vitality = self.vitality.clamp_to(v_cap);
            self.energy = self.energy.clamp_to(e_cap);
            Some(DomainEvent::Shrank {
                plant_id: self.id,
                cell: removed_cell,
            })
        } else {
            None
        }
    }

    /// Transition d'etat basee sur la vitalite et la biomasse actuelles.
    /// Retourne un evenement si l'etat change (StateChanged ou Died).
    pub fn update_state(&mut self) -> Option<DomainEvent> {
        // Les graines avec vitalite 0 meurent (timeout de dormance)
        if self.state == PlantState::Seed {
            if self.vitality.is_zero() {
                self.state = PlantState::Dead;
                return Some(DomainEvent::Died {
                    plant_id: self.id,
                    position: self.footprint[0],
                    age: self.age,
                    biomass: self.biomass.value(),
                });
            }
            return None;
        }
        let old_state = self.state;
        let v_cap = vitality_cap(&self.biomass, self.genetics.vitality_factor());
        let current = self.vitality.value();
        if current == 0.0 {
            self.state = PlantState::Dead;
        } else if current < v_cap * 0.2 {
            self.state = PlantState::Dying;
        } else if current < v_cap * 0.5 {
            self.state = PlantState::Stressed;
        } else if self.biomass.value() >= (self.genetics.max_size() * 80) / 100 {
            self.state = PlantState::Mature;
        } else {
            self.state = PlantState::Growing;
        }
        if self.state == old_state {
            return None;
        }
        if self.state == PlantState::Dead {
            Some(DomainEvent::Died {
                plant_id: self.id,
                position: self.footprint[0],
                age: self.age,
                biomass: self.biomass.value(),
            })
        } else {
            Some(DomainEvent::StateChanged {
                plant_id: self.id,
                from: old_state,
                to: self.state,
            })
        }
    }

    pub fn consume_energy(&mut self, amount: f32) {
        self.energy = self.energy.sub(amount);
    }

    pub fn gain_energy(&mut self, amount: f32) {
        let cap = energy_cap(&self.biomass, self.genetics.energy_factor());
        self.energy = self.energy.add(amount, cap);
    }

    pub fn damage(&mut self, amount: f32) {
        self.vitality = self.vitality.sub(amount);
    }

    pub fn heal(&mut self, amount: f32) {
        let cap = vitality_cap(&self.biomass, self.genetics.vitality_factor());
        self.vitality = self.vitality.add(amount, cap);
    }

    /// Retire une cellule d'emprise specifique. Retourne true si retiree.
    /// Ne retire pas si c'est la derniere cellule d'emprise.
    pub fn remove_footprint_cell(&mut self, pos: &Pos) -> bool {
        if self.footprint.len() <= 1 {
            return false;
        }
        if let Some(idx) = self.footprint.iter().position(|p| p == pos) {
            self.footprint.swap_remove(idx);
            self.biomass = self.biomass.sub(1);
            let v_cap = vitality_cap(&self.biomass, self.genetics.vitality_factor());
            let e_cap = energy_cap(&self.biomass, self.genetics.energy_factor());
            self.vitality = self.vitality.clamp_to(v_cap);
            self.energy = self.energy.clamp_to(e_cap);
            true
        } else {
            false
        }
    }

    /// Demarre la decomposition progressive. Passe en etat Decomposing.
    /// Calcule les ressources a liberer sur la duree de decomposition.
    pub fn start_decomposition(&mut self, decomposition_ticks: u32) {
        self.state = PlantState::Decomposing;
        self.carbon_to_release = self.biomass.value() as f32 * 0.01;
        self.nitrogen_to_release = self.age as f32 * 0.001;
        self.decomposition_remaining = decomposition_ticks;
    }

    /// Avance la decomposition d'un tick. Retourne (carbone, azote) liberes ce tick.
    pub fn decompose_tick(&mut self, decomposition_ticks: u32) -> (f32, f32) {
        if self.decomposition_remaining > 0 {
            let carbon = self.carbon_to_release / decomposition_ticks as f32;
            let nitrogen = self.nitrogen_to_release / decomposition_ticks as f32;
            self.decomposition_remaining -= 1;
            (carbon, nitrogen)
        } else {
            (0.0, 0.0)
        }
    }

    /// Indique si la plante a termine sa decomposition.
    pub fn is_fully_decomposed(&self) -> bool {
        self.decomposition_remaining == 0 && self.state == PlantState::Decomposing
    }

    /// Reconstruit une plante a partir de tous ses champs bruts.
    /// Utilise pour la deserialisation — bypass du constructeur normal.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_raw(
        id: u64,
        state: PlantState,
        age: u32,
        vitality: f32,
        energy: f32,
        biomass: u16,
        footprint: Vec<Pos>,
        canopy: Vec<Pos>,
        roots: Vec<Pos>,
        genetics: GeneticTraits,
        lineage: Lineage,
        decomposition_remaining: u32,
        carbon_to_release: f32,
        nitrogen_to_release: f32,
        ancestors: Vec<u64>,
        seed_progress: f32,
    ) -> Self {
        let v_cap = vitality_cap(
            &Biomass::new(biomass, genetics.max_size()),
            genetics.vitality_factor(),
        );
        let e_cap = energy_cap(
            &Biomass::new(biomass, genetics.max_size()),
            genetics.energy_factor(),
        );
        Self {
            id,
            state,
            age,
            vitality: Vitality::new(vitality, v_cap),
            energy: Energy::new(energy, e_cap),
            biomass: Biomass::new(biomass, genetics.max_size()),
            footprint,
            canopy,
            roots,
            genetics,
            lineage,
            decomposition_remaining,
            carbon_to_release,
            nitrogen_to_release,
            ancestors,
            seed_progress,
        }
    }

    /// Accesseur pour decomposition_remaining (pour la serialisation).
    pub fn decomposition_remaining(&self) -> u32 {
        self.decomposition_remaining
    }

    /// Accesseur pour carbon_to_release (pour la serialisation).
    pub fn carbon_to_release(&self) -> f32 {
        self.carbon_to_release
    }

    /// Accesseur pour nitrogen_to_release (pour la serialisation).
    pub fn nitrogen_to_release(&self) -> f32 {
        self.nitrogen_to_release
    }

    /// Ancetres de la plante (du plus ancien au parent direct).
    pub fn ancestors(&self) -> &[u64] {
        &self.ancestors
    }

    /// Identifiant du parent direct (dernier element des ancetres).
    pub fn parent_id(&self) -> Option<u64> {
        self.ancestors.last().copied()
    }

    /// Profondeur de generation (nombre d'ancetres connus).
    pub fn generation_depth(&self) -> usize {
        self.ancestors.len()
    }

    /// Progression vers la prochaine graine.
    pub fn seed_progress(&self) -> f32 {
        self.seed_progress
    }

    /// Accumule de la progression vers la prochaine graine.
    pub fn add_seed_progress(&mut self, amount: f32) {
        self.seed_progress += amount;
    }

    /// Consomme de la progression de graine (quand une graine est produite).
    pub fn consume_seed_progress(&mut self, amount: f32) {
        self.seed_progress -= amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::GrowthLayer;

    #[test]
    fn bounded_f32_clampe_a_la_creation() {
        let b = BoundedF32::new(150.0, 100.0);
        assert_eq!(b.value(), 100.0);
        let b = BoundedF32::new(-5.0, 100.0);
        assert_eq!(b.value(), 0.0);
    }

    #[test]
    fn bounded_f32_sub_ne_descend_pas_sous_zero() {
        let b = BoundedF32::new(10.0, 100.0);
        let b = b.sub(50.0);
        assert_eq!(b.value(), 0.0);
        assert!(b.is_zero());
    }

    #[test]
    fn la_vitalite_est_bornee_au_cap() {
        let v = Vitality::new(150.0, 100.0);
        assert_eq!(v.value(), 100.0);
    }

    #[test]
    fn la_vitalite_sub_est_bornee_a_zero() {
        let v = Vitality::new(10.0, 100.0);
        let v = v.sub(50.0);
        assert_eq!(v.value(), 0.0);
    }

    #[test]
    fn energie_add_respecte_le_cap() {
        let e = Energy::new(80.0, 100.0);
        let e = e.add(50.0, 100.0);
        assert_eq!(e.value(), 100.0);
    }

    #[test]
    fn la_biomasse_ne_depasse_pas_max_size() {
        let b = Biomass::new(10, 100);
        let b = b.add(200, 100);
        assert_eq!(b.value(), 100);
    }

    #[test]
    fn clamp_to_reduit_la_valeur() {
        let v = Vitality::new(80.0, 100.0);
        let v = v.clamp_to(50.0);
        assert_eq!(v.value(), 50.0);
    }

    #[test]
    fn egalite_de_lignee() {
        let a = Lineage::new(42, 7);
        let b = Lineage::new(42, 7);
        assert_eq!(a, b);
    }

    // --- Tests de l'entite Plant ---

    fn test_genetics() -> GeneticTraits {
        GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 5.0)
    }

    fn test_plant() -> Plant {
        Plant::new(1, Pos { x: 5, y: 5 }, test_genetics(), Lineage::new(1, 0))
    }

    #[test]
    fn la_plante_demarre_en_graine() {
        let p = test_plant();
        assert_eq!(p.state(), PlantState::Seed);
        assert_eq!(p.biomass().value(), 1);
    }

    #[test]
    fn la_plante_germe() {
        let mut p = test_plant();
        assert!(p.germinate().is_some());
        assert_eq!(p.state(), PlantState::Growing);
        // Ne peut pas germer deux fois
        assert!(p.germinate().is_none());
    }

    #[test]
    fn la_plante_fait_pousser_lemprise() {
        let mut p = test_plant();
        let _ = p.germinate();
        let _ = p.grow_footprint(Pos { x: 6, y: 5 });
        assert_eq!(p.footprint().len(), 2);
        assert_eq!(p.biomass().value(), 2);
    }

    #[test]
    fn la_plante_fait_pousser_la_canopee() {
        let mut p = test_plant();
        let _ = p.germinate();
        let event = p.grow_canopy(Pos { x: 6, y: 5 });
        assert!(event.is_some());
        assert_eq!(p.canopy().len(), 2);
        // La biomasse ne change pas pour la canopee aerienne
        assert_eq!(p.biomass().value(), 1);
    }

    #[test]
    fn la_plante_fait_pousser_les_racines() {
        let mut p = test_plant();
        let _ = p.germinate();
        let event = p.grow_roots(Pos { x: 6, y: 5 });
        assert!(event.is_some());
        assert_eq!(p.roots().len(), 2);
        // La biomasse ne change pas pour la croissance racinaire
        assert_eq!(p.biomass().value(), 1);
    }

    #[test]
    fn la_plante_retrecit() {
        let mut p = test_plant();
        let _ = p.germinate();
        let _ = p.grow_footprint(Pos { x: 6, y: 5 });
        assert_eq!(p.biomass().value(), 2);
        assert!(p.shrink().is_some());
        assert_eq!(p.footprint().len(), 1);
        assert_eq!(p.biomass().value(), 1);
    }

    #[test]
    fn la_plante_meurt_quand_vitalite_a_zero() {
        let mut p = test_plant();
        let _ = p.germinate();
        // plafond vitalite = 1 * 10.0 = 10.0
        p.damage(10.0);
        let _ = p.update_state();
        assert_eq!(p.state(), PlantState::Dead);
        assert!(p.is_dead());
    }

    #[test]
    fn la_plante_devient_stressee() {
        let mut p = test_plant();
        let _ = p.germinate();
        // plafond vitalite = 10.0, seuil stress = 50% = 5.0
        // Degats pour amener la vitalite a 4.0 (< 50% mais >= 20%)
        p.damage(6.0);
        let _ = p.update_state();
        assert_eq!(p.state(), PlantState::Stressed);
    }

    #[test]
    fn la_plante_devient_mourante() {
        let mut p = test_plant();
        let _ = p.germinate();
        // plafond vitalite = 10.0, seuil mourant = 20% = 2.0
        // Degats pour amener la vitalite a 1.0 (< 20%)
        p.damage(9.0);
        let _ = p.update_state();
        assert_eq!(p.state(), PlantState::Dying);
    }

    #[test]
    fn la_plante_atteint_la_maturite() {
        let mut p = test_plant();
        let _ = p.germinate();
        // max_size = 20, 80% = 16. Biomasse >= 16 requise.
        // Commence a 1, pousse 15 cellules d'emprise en plus.
        for i in 0..15 {
            let _ = p.grow_footprint(Pos { x: 6 + i, y: 5 });
        }
        assert_eq!(p.biomass().value(), 16);
        // Soigner a fond pour que la vitalite passe le check
        p.heal(1000.0);
        let _ = p.update_state();
        assert_eq!(p.state(), PlantState::Mature);
    }

    #[test]
    fn la_plante_clampe_les_stats_au_retrecissement() {
        let mut p = test_plant();
        let _ = p.germinate();
        // Pousse jusqu'a biomasse 3 → plafond vitalite = 30.0, plafond energie = 15.0
        let _ = p.grow_footprint(Pos { x: 6, y: 5 });
        let _ = p.grow_footprint(Pos { x: 7, y: 5 });
        // Soigner/gagner pour remplir les plafonds a biomasse 3
        p.heal(100.0);
        p.gain_energy(100.0);
        assert_eq!(p.vitality().value(), 30.0);
        assert_eq!(p.energy().value(), 15.0);

        // Retrecir a biomasse 2 → plafond vitalite = 20.0, plafond energie = 10.0
        let _ = p.shrink();
        assert_eq!(p.vitality().value(), 20.0);
        assert_eq!(p.energy().value(), 10.0);
    }

    // --- Tests d'encapsulation GeneticTraits ---

    #[test]
    fn les_traits_genetiques_clampent_les_bornes() {
        // max_size trop petit → clampe a 15
        let g = GeneticTraits::new(5, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        assert_eq!(g.max_size(), 15);

        // max_size trop grand → clampe a 40
        let g = GeneticTraits::new(50, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        assert_eq!(g.max_size(), 40);

        // hidden_size trop petit → clampe a 6
        let g = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 3, 10.0, 5.0);
        assert_eq!(g.hidden_size(), 6);

        // hidden_size trop grand → clampe a 14
        let g = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 20, 10.0, 5.0);
        assert_eq!(g.hidden_size(), 14);

        // carbon_nitrogen_ratio hors bornes
        let g = GeneticTraits::new(20, 0.1, ExudateType::Carbon, 8, 10.0, 5.0);
        assert!((g.carbon_nitrogen_ratio() - 0.3).abs() < f32::EPSILON);
        let g = GeneticTraits::new(20, 1.5, ExudateType::Carbon, 8, 10.0, 5.0);
        assert!((g.carbon_nitrogen_ratio() - 0.9).abs() < f32::EPSILON);

        // vitality_factor et energy_factor hors bornes
        let g = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 0.01, 25.0);
        assert!((g.vitality_factor() - 0.1).abs() < f32::EPSILON);
        assert!((g.energy_factor() - 20.0).abs() < f32::EPSILON);

        // exudate_type passe tel quel
        let g = GeneticTraits::new(20, 0.5, ExudateType::Nitrogen, 8, 10.0, 5.0);
        assert_eq!(g.exudate_type(), ExudateType::Nitrogen);
    }

    #[test]
    fn lineage_getters() {
        let l = Lineage::new(42, 7);
        assert_eq!(l.id(), 42);
        assert_eq!(l.generation(), 7);
    }

    // --- Tests de retour d'events ---

    #[test]
    fn germinate_retourne_evenement_germination() {
        let mut p = test_plant();
        let event = p.germinate();
        assert_eq!(
            event,
            Some(DomainEvent::Germinated {
                plant_id: 1,
                position: Pos { x: 5, y: 5 },
            })
        );
    }

    #[test]
    fn grow_footprint_retourne_evenement_croissance() {
        let mut p = test_plant();
        let _ = p.germinate();
        let event = p.grow_footprint(Pos { x: 6, y: 5 });
        assert_eq!(
            event,
            DomainEvent::Grew {
                plant_id: 1,
                cell: Pos { x: 6, y: 5 },
                layer: GrowthLayer::Footprint,
            }
        );
    }

    #[test]
    fn grow_roots_retourne_evenement_croissance() {
        let mut p = test_plant();
        let _ = p.germinate();
        let event = p.grow_roots(Pos { x: 4, y: 5 });
        assert_eq!(
            event,
            Some(DomainEvent::Grew {
                plant_id: 1,
                cell: Pos { x: 4, y: 5 },
                layer: GrowthLayer::Roots,
            })
        );
    }

    #[test]
    fn shrink_retourne_evenement_retrecissement() {
        let mut p = test_plant();
        let _ = p.germinate();
        let _ = p.grow_footprint(Pos { x: 6, y: 5 });
        let event = p.shrink();
        assert_eq!(
            event,
            Some(DomainEvent::Shrank {
                plant_id: 1,
                cell: Pos { x: 6, y: 5 },
            })
        );
        // Ne peut pas retrecir en dessous de 1
        assert!(p.shrink().is_none());
    }

    #[test]
    fn update_state_retourne_evenement_changement_etat() {
        let mut p = test_plant();
        let _ = p.germinate();
        // Infliger des degats pour passer en Stressed
        p.damage(6.0);
        let event = p.update_state();
        assert_eq!(
            event,
            Some(DomainEvent::StateChanged {
                plant_id: 1,
                from: PlantState::Growing,
                to: PlantState::Stressed,
            })
        );
    }

    #[test]
    fn update_state_retourne_evenement_mort() {
        let mut p = test_plant();
        let _ = p.germinate();
        p.damage(10.0);
        let event = p.update_state();
        assert_eq!(
            event,
            Some(DomainEvent::Died {
                plant_id: 1,
                position: Pos { x: 5, y: 5 },
                age: 0,
                biomass: 1,
            })
        );
    }

    #[test]
    fn la_plante_demarre_la_decomposition() {
        // Tuer la plante puis demarrer la decomposition
        let mut p = test_plant();
        let _ = p.germinate();
        p.damage(10.0);
        let _ = p.update_state();
        assert_eq!(p.state(), PlantState::Dead);

        p.start_decomposition(50);
        assert_eq!(p.state(), PlantState::Decomposing);
    }

    #[test]
    fn la_decomposition_libere_des_ressources() {
        // Plante avec de la biomasse et de l'age pour avoir des ressources a liberer
        let mut p = test_plant();
        let _ = p.germinate();
        // Pousser pour augmenter la biomasse
        for i in 0..5 {
            let _ = p.grow_footprint(Pos { x: 6 + i, y: 5 });
        }
        // Avancer l'age pour que nitrogen_to_release > 0
        for _ in 0..100 {
            p.tick();
        }
        p.damage(1000.0);
        let _ = p.update_state();
        p.start_decomposition(50);

        let (carbon, nitrogen) = p.decompose_tick(50);
        // biomass = 6, carbon_to_release = 6 * 0.01 = 0.06, par tick = 0.06/50
        assert!(carbon > 0.0, "le carbone libere doit etre > 0");
        // age = 100, nitrogen_to_release = 100 * 0.001 = 0.1, par tick = 0.1/50
        assert!(nitrogen > 0.0, "l'azote libere doit etre > 0");
    }

    #[test]
    fn la_plante_est_completement_decomposee() {
        // Apres N ticks de decomposition, is_fully_decomposed doit etre true
        let mut p = test_plant();
        let _ = p.germinate();
        p.damage(1000.0);
        let _ = p.update_state();

        let ticks = 10;
        p.start_decomposition(ticks);
        assert!(!p.is_fully_decomposed());

        // Faire tourner tous les ticks de decomposition
        for _ in 0..ticks {
            let _ = p.decompose_tick(ticks);
        }
        assert!(
            p.is_fully_decomposed(),
            "la plante devrait etre entierement decomposee apres {} ticks",
            ticks
        );
    }
}
