// DTOs pour les types lies aux plantes, cerveaux et evolution.

use serde::{Deserialize, Serialize};

use super::PosDto;
use crate::application::evolution::{Genome, PlantStats};
use crate::domain::brain::Brain;
use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, PlantState};

// --- ExudateType ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ExudateTypeDto {
    Carbon,
    Nitrogen,
}

impl From<ExudateType> for ExudateTypeDto {
    fn from(e: ExudateType) -> Self {
        match e {
            ExudateType::Carbon => Self::Carbon,
            ExudateType::Nitrogen => Self::Nitrogen,
        }
    }
}

impl ExudateTypeDto {
    pub fn to_domain(&self) -> ExudateType {
        match self {
            Self::Carbon => ExudateType::Carbon,
            Self::Nitrogen => ExudateType::Nitrogen,
        }
    }
}

// --- PlantState ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlantStateDto {
    Seed,
    Growing,
    Mature,
    Stressed,
    Dying,
    Dead,
    Decomposing,
}

impl From<PlantState> for PlantStateDto {
    fn from(s: PlantState) -> Self {
        match s {
            PlantState::Seed => Self::Seed,
            PlantState::Growing => Self::Growing,
            PlantState::Mature => Self::Mature,
            PlantState::Stressed => Self::Stressed,
            PlantState::Dying => Self::Dying,
            PlantState::Dead => Self::Dead,
            PlantState::Decomposing => Self::Decomposing,
        }
    }
}

impl PlantStateDto {
    pub fn to_domain(&self) -> PlantState {
        match self {
            Self::Seed => PlantState::Seed,
            Self::Growing => PlantState::Growing,
            Self::Mature => PlantState::Mature,
            Self::Stressed => PlantState::Stressed,
            Self::Dying => PlantState::Dying,
            Self::Dead => PlantState::Dead,
            Self::Decomposing => PlantState::Decomposing,
        }
    }
}

// --- Lineage ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LineageDto {
    pub id: u64,
    pub generation: u64,
}

impl From<&Lineage> for LineageDto {
    fn from(l: &Lineage) -> Self {
        Self {
            id: l.id(),
            generation: l.generation(),
        }
    }
}

impl LineageDto {
    pub fn to_domain(&self) -> Lineage {
        Lineage::new(self.id, self.generation)
    }
}

// --- GeneticTraits ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneticTraitsDto {
    pub max_size: u16,
    pub carbon_nitrogen_ratio: f32,
    pub exudate_type: ExudateTypeDto,
    pub hidden_size: u8,
    pub vitality_factor: f32,
    pub energy_factor: f32,
}

impl From<&GeneticTraits> for GeneticTraitsDto {
    fn from(g: &GeneticTraits) -> Self {
        Self {
            max_size: g.max_size(),
            carbon_nitrogen_ratio: g.carbon_nitrogen_ratio(),
            exudate_type: g.exudate_type().into(),
            hidden_size: g.hidden_size(),
            vitality_factor: g.vitality_factor(),
            energy_factor: g.energy_factor(),
        }
    }
}

impl GeneticTraitsDto {
    pub fn to_domain(&self) -> GeneticTraits {
        GeneticTraits::new(
            self.max_size,
            self.carbon_nitrogen_ratio,
            self.exudate_type.to_domain(),
            self.hidden_size,
            self.vitality_factor,
            self.energy_factor,
        )
    }
}

// --- Brain ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainDto {
    pub hidden_size: u8,
    pub weights: Vec<f32>,
}

impl From<&Brain> for BrainDto {
    fn from(b: &Brain) -> Self {
        Self {
            hidden_size: b.hidden_size(),
            weights: b.weights(),
        }
    }
}

impl BrainDto {
    /// Convertit en type domaine. Retourne None si les poids sont invalides.
    pub fn to_domain(&self) -> Option<Brain> {
        Brain::from_weights(self.hidden_size, self.weights.clone())
    }
}

// --- Plant ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlantDto {
    pub id: u64,
    pub state: PlantStateDto,
    pub age: u32,
    pub vitality: f32,
    pub energy: f32,
    pub biomass: u16,
    pub footprint: Vec<PosDto>,
    pub canopy: Vec<PosDto>,
    pub roots: Vec<PosDto>,
    pub genetics: GeneticTraitsDto,
    pub lineage: LineageDto,
    pub decomposition_remaining: u32,
    pub carbon_to_release: f32,
    pub nitrogen_to_release: f32,
    #[serde(default)]
    pub ancestors: Vec<u64>,
    #[serde(default)]
    pub seed_progress: f32,
}

impl From<&Plant> for PlantDto {
    fn from(p: &Plant) -> Self {
        Self {
            id: p.id(),
            state: p.state().into(),
            age: p.age(),
            vitality: p.vitality().value(),
            energy: p.energy().value(),
            biomass: p.biomass().value(),
            footprint: p.footprint().iter().map(PosDto::from).collect(),
            canopy: p.canopy().iter().map(PosDto::from).collect(),
            roots: p.roots().iter().map(PosDto::from).collect(),
            genetics: GeneticTraitsDto::from(p.genetics()),
            lineage: LineageDto::from(p.lineage()),
            decomposition_remaining: p.decomposition_remaining(),
            carbon_to_release: p.carbon_to_release(),
            nitrogen_to_release: p.nitrogen_to_release(),
            ancestors: p.ancestors().to_vec(),
            seed_progress: p.seed_progress(),
        }
    }
}

impl PlantDto {
    /// Convertit en type domaine.
    pub fn to_domain(&self) -> Plant {
        Plant::from_raw(
            self.id,
            self.state.to_domain(),
            self.age,
            self.vitality,
            self.energy,
            self.biomass,
            self.footprint.iter().map(|p| p.to_domain()).collect(),
            self.canopy.iter().map(|p| p.to_domain()).collect(),
            self.roots.iter().map(|p| p.to_domain()).collect(),
            self.genetics.to_domain(),
            self.lineage.to_domain(),
            self.decomposition_remaining,
            self.carbon_to_release,
            self.nitrogen_to_release,
            self.ancestors.clone(),
            self.seed_progress,
        )
    }
}

// --- PlantStats ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlantStatsDto {
    pub max_biomass: u16,
    pub lifetime: u32,
    pub max_territory: u16,
    pub symbiotic_connections: u32,
    pub exudates_emitted: f32,
    pub cn_exchanges: f32,
    pub seeds_produced: u32,
    pub soil_enriched: f32,
    pub soil_depleted: f32,
    pub monoculture_penalty: f32,
    #[serde(default)]
    pub inherited_fitness: f32,
}

impl From<&PlantStats> for PlantStatsDto {
    fn from(s: &PlantStats) -> Self {
        Self {
            max_biomass: s.max_biomass,
            lifetime: s.lifetime,
            max_territory: s.max_territory,
            symbiotic_connections: s.symbiotic_connections,
            exudates_emitted: s.exudates_emitted,
            cn_exchanges: s.cn_exchanges,
            seeds_produced: s.seeds_produced,
            soil_enriched: s.soil_enriched,
            soil_depleted: s.soil_depleted,
            monoculture_penalty: s.monoculture_penalty,
            inherited_fitness: s.inherited_fitness,
        }
    }
}

impl PlantStatsDto {
    pub fn to_domain(&self) -> PlantStats {
        PlantStats {
            max_biomass: self.max_biomass,
            lifetime: self.lifetime,
            max_territory: self.max_territory,
            symbiotic_connections: self.symbiotic_connections,
            exudates_emitted: self.exudates_emitted,
            cn_exchanges: self.cn_exchanges,
            seeds_produced: self.seeds_produced,
            soil_enriched: self.soil_enriched,
            soil_depleted: self.soil_depleted,
            monoculture_penalty: self.monoculture_penalty,
            inherited_fitness: self.inherited_fitness,
        }
    }
}

// --- Genome ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenomeDto {
    pub brain: BrainDto,
    pub traits_dto: GeneticTraitsDto,
}

impl From<&Genome> for GenomeDto {
    fn from(g: &Genome) -> Self {
        Self {
            brain: BrainDto::from(&g.brain),
            traits_dto: GeneticTraitsDto::from(&g.traits),
        }
    }
}

impl GenomeDto {
    /// Convertit en type domaine. Retourne None si le brain est invalide.
    pub fn to_domain(&self) -> Option<Genome> {
        let brain = self.brain.to_domain()?;
        Some(Genome {
            brain,
            traits: self.traits_dto.to_domain(),
        })
    }
}
