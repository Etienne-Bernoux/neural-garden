// DTOs pour SimState, SimConfig, SeedBank, SeasonCycle.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::plant::{BrainDto, GenomeDto, PlantDto, PlantStatsDto};
use super::world::{IslandDto, SymbiosisNetworkDto, WorldDto};
use crate::application::config::SimConfig;
use crate::application::evolution::{GenerationCounter, SeedBank};
use crate::application::season::{Season, SeasonCycle};
use crate::application::sim::SimState;

// --- Season ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SeasonDto {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl From<Season> for SeasonDto {
    fn from(s: Season) -> Self {
        match s {
            Season::Spring => Self::Spring,
            Season::Summer => Self::Summer,
            Season::Autumn => Self::Autumn,
            Season::Winter => Self::Winter,
        }
    }
}

impl SeasonDto {
    pub fn to_domain(&self) -> Season {
        match self {
            Self::Spring => Season::Spring,
            Self::Summer => Season::Summer,
            Self::Autumn => Season::Autumn,
            Self::Winter => Season::Winter,
        }
    }
}

// --- SeasonCycle ---

#[derive(Serialize, Deserialize, Debug)]
pub struct SeasonCycleDto {
    pub current_tick: u32,
    pub ticks_per_season: u32,
}

impl From<&SeasonCycle> for SeasonCycleDto {
    fn from(c: &SeasonCycle) -> Self {
        Self {
            current_tick: c.tick(),
            ticks_per_season: c.ticks_per_season(),
        }
    }
}

impl SeasonCycleDto {
    pub fn to_domain(&self) -> SeasonCycle {
        SeasonCycle::from_raw(self.current_tick, self.ticks_per_season)
    }
}

// --- SeedBank ---

#[derive(Serialize, Deserialize, Debug)]
pub struct SeedBankDto {
    pub entries: Vec<(GenomeDto, f32)>,
    pub capacity: usize,
}

impl From<&SeedBank> for SeedBankDto {
    fn from(b: &SeedBank) -> Self {
        Self {
            entries: b
                .entries()
                .into_iter()
                .map(|(g, f)| (GenomeDto::from(g), f))
                .collect(),
            capacity: b.capacity(),
        }
    }
}

impl SeedBankDto {
    /// Convertit en type domaine. Retourne None si un genome est invalide.
    pub fn to_domain(&self) -> Option<SeedBank> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for (genome_dto, fitness) in &self.entries {
            let genome = genome_dto.to_domain()?;
            entries.push((genome, *fitness));
        }
        Some(SeedBank::from_entries(entries, self.capacity))
    }
}

// --- SimConfig ---

#[derive(Serialize, Deserialize, Debug)]
pub struct SimConfigDto {
    pub rain_rate: f32,
    pub evaporation_rate: f32,
    pub evaporation_canopy_rate: f32,
    pub carbon_regen_rate: f32,
    pub nitrogen_regen_rate: f32,
    pub exudate_decay: f32,
    pub canopy_light: f32,
    pub growth_threshold: f32,
    pub growth_energy_cost: f32,
    pub growth_carbon_cost: f32,
    pub growth_nitrogen_cost: f32,
    pub invasion_energy_threshold: f32,
    pub invasion_defense_threshold: f32,
    pub invasion_energy_cost: f32,
    pub invasion_damage: f32,
    pub defense_energy_cost: f32,
    pub exudate_output_rate: f32,
    pub exudate_energy_cost_rate: f32,
    pub absorption_rate: f32,
    pub photosynthesis_rate: f32,
    pub maintenance_rate: f32,
    pub reproduction_energy_min: f32,
    pub reproduction_biomass_min: u16,
    pub reproduction_energy_cost: f32,
    pub reproduction_min_distance: u16,
    pub reproduction_max_distance: u16,
    pub seed_rain_interval: u32,
    pub germination_carbon_min: f32,
    pub germination_nitrogen_min: f32,
    pub dormancy_timeout: u32,
    #[serde(default = "default_aging_base_rate")]
    pub aging_base_rate: f32,
    #[serde(default = "default_starvation_threshold")]
    pub starvation_threshold: f32,
    #[serde(default = "default_starvation_drain_rate")]
    pub starvation_drain_rate: f32,
    #[serde(default = "default_nitrogen_fixation_rate")]
    pub nitrogen_fixation_rate: f32,
    #[serde(default = "default_nitrogen_fixation_energy_cost")]
    pub nitrogen_fixation_energy_cost: f32,
    pub decomposition_ticks: u32,
    pub seed_bank_capacity: usize,
    pub initial_population: usize,
    #[serde(default = "default_seed_production_rate")]
    pub seed_production_rate: f32,
    #[serde(default = "default_seed_energy_cost")]
    pub seed_energy_cost: f32,
    #[serde(default = "default_seed_energy_threshold")]
    pub seed_energy_threshold: f32,
    #[serde(default = "default_ticks_per_season")]
    pub ticks_per_season: u32,
    #[serde(default)]
    pub nursery_mode: bool,
}

fn default_aging_base_rate() -> f32 {
    0.05
}

fn default_starvation_threshold() -> f32 {
    0.1
}

fn default_starvation_drain_rate() -> f32 {
    2.0
}

fn default_nitrogen_fixation_rate() -> f32 {
    0.03
}

fn default_nitrogen_fixation_energy_cost() -> f32 {
    0.5
}

fn default_seed_production_rate() -> f32 {
    0.01
}

fn default_seed_energy_cost() -> f32 {
    5.0
}

fn default_seed_energy_threshold() -> f32 {
    15.0
}

fn default_ticks_per_season() -> u32 {
    250
}

impl From<&SimConfig> for SimConfigDto {
    fn from(c: &SimConfig) -> Self {
        Self {
            rain_rate: c.rain_rate,
            evaporation_rate: c.evaporation_rate,
            evaporation_canopy_rate: c.evaporation_canopy_rate,
            carbon_regen_rate: c.carbon_regen_rate,
            nitrogen_regen_rate: c.nitrogen_regen_rate,
            exudate_decay: c.exudate_decay,
            canopy_light: c.canopy_light,
            growth_threshold: c.growth_threshold,
            growth_energy_cost: c.growth_energy_cost,
            growth_carbon_cost: c.growth_carbon_cost,
            growth_nitrogen_cost: c.growth_nitrogen_cost,
            invasion_energy_threshold: c.invasion_energy_threshold,
            invasion_defense_threshold: c.invasion_defense_threshold,
            invasion_energy_cost: c.invasion_energy_cost,
            invasion_damage: c.invasion_damage,
            defense_energy_cost: c.defense_energy_cost,
            exudate_output_rate: c.exudate_output_rate,
            exudate_energy_cost_rate: c.exudate_energy_cost_rate,
            absorption_rate: c.absorption_rate,
            photosynthesis_rate: c.photosynthesis_rate,
            maintenance_rate: c.maintenance_rate,
            reproduction_energy_min: c.reproduction_energy_min,
            reproduction_biomass_min: c.reproduction_biomass_min,
            reproduction_energy_cost: c.reproduction_energy_cost,
            reproduction_min_distance: c.reproduction_min_distance,
            reproduction_max_distance: c.reproduction_max_distance,
            seed_rain_interval: c.seed_rain_interval,
            germination_carbon_min: c.germination_carbon_min,
            germination_nitrogen_min: c.germination_nitrogen_min,
            dormancy_timeout: c.dormancy_timeout,
            aging_base_rate: c.aging_base_rate,
            starvation_threshold: c.starvation_threshold,
            starvation_drain_rate: c.starvation_drain_rate,
            nitrogen_fixation_rate: c.nitrogen_fixation_rate,
            nitrogen_fixation_energy_cost: c.nitrogen_fixation_energy_cost,
            decomposition_ticks: c.decomposition_ticks,
            seed_bank_capacity: c.seed_bank_capacity,
            initial_population: c.initial_population,
            seed_production_rate: c.seed_production_rate,
            seed_energy_cost: c.seed_energy_cost,
            seed_energy_threshold: c.seed_energy_threshold,
            ticks_per_season: c.ticks_per_season,
            nursery_mode: c.nursery_mode,
        }
    }
}

impl SimConfigDto {
    pub fn to_domain(&self) -> SimConfig {
        SimConfig {
            rain_rate: self.rain_rate,
            evaporation_rate: self.evaporation_rate,
            evaporation_canopy_rate: self.evaporation_canopy_rate,
            carbon_regen_rate: self.carbon_regen_rate,
            nitrogen_regen_rate: self.nitrogen_regen_rate,
            exudate_decay: self.exudate_decay,
            canopy_light: self.canopy_light,
            growth_threshold: self.growth_threshold,
            growth_energy_cost: self.growth_energy_cost,
            growth_carbon_cost: self.growth_carbon_cost,
            growth_nitrogen_cost: self.growth_nitrogen_cost,
            invasion_energy_threshold: self.invasion_energy_threshold,
            invasion_defense_threshold: self.invasion_defense_threshold,
            invasion_energy_cost: self.invasion_energy_cost,
            invasion_damage: self.invasion_damage,
            defense_energy_cost: self.defense_energy_cost,
            exudate_output_rate: self.exudate_output_rate,
            exudate_energy_cost_rate: self.exudate_energy_cost_rate,
            absorption_rate: self.absorption_rate,
            photosynthesis_rate: self.photosynthesis_rate,
            maintenance_rate: self.maintenance_rate,
            reproduction_energy_min: self.reproduction_energy_min,
            reproduction_biomass_min: self.reproduction_biomass_min,
            reproduction_energy_cost: self.reproduction_energy_cost,
            reproduction_min_distance: self.reproduction_min_distance,
            reproduction_max_distance: self.reproduction_max_distance,
            seed_rain_interval: self.seed_rain_interval,
            germination_carbon_min: self.germination_carbon_min,
            germination_nitrogen_min: self.germination_nitrogen_min,
            dormancy_timeout: self.dormancy_timeout,
            aging_base_rate: self.aging_base_rate,
            starvation_threshold: self.starvation_threshold,
            starvation_drain_rate: self.starvation_drain_rate,
            nitrogen_fixation_rate: self.nitrogen_fixation_rate,
            nitrogen_fixation_energy_cost: self.nitrogen_fixation_energy_cost,
            decomposition_ticks: self.decomposition_ticks,
            seed_bank_capacity: self.seed_bank_capacity,
            initial_population: self.initial_population,
            seed_production_rate: self.seed_production_rate,
            seed_energy_cost: self.seed_energy_cost,
            seed_energy_threshold: self.seed_energy_threshold,
            ticks_per_season: self.ticks_per_season,
            nursery_mode: self.nursery_mode,
        }
    }
}

// --- SimState ---

/// DTO pour la sauvegarde complete du SimState.
#[derive(Serialize, Deserialize, Debug)]
pub struct SimStateDto {
    pub world: WorldDto,
    pub island: IslandDto,
    pub plants: Vec<PlantDto>,
    pub brains: Vec<(u64, BrainDto)>,
    pub symbiosis: SymbiosisNetworkDto,
    pub seed_bank: SeedBankDto,
    pub season_cycle: SeasonCycleDto,
    pub generation_counter: u64,
    pub plant_stats: Vec<(u64, PlantStatsDto)>,
    pub next_plant_id: u64,
    pub tick_count: u32,
    pub config: SimConfigDto,
}

impl From<&SimState> for SimStateDto {
    fn from(s: &SimState) -> Self {
        Self {
            world: WorldDto::from(&s.world),
            island: IslandDto::from(&s.island),
            plants: s
                .plants
                .iter()
                .map(|p| PlantDto::from(p.as_ref()))
                .collect(),
            brains: s
                .brains
                .iter()
                .map(|(id, b)| (*id, BrainDto::from(b)))
                .collect(),
            symbiosis: SymbiosisNetworkDto::from(&s.symbiosis),
            seed_bank: SeedBankDto::from(&s.seed_bank),
            season_cycle: SeasonCycleDto::from(&s.season_cycle),
            generation_counter: s.generation_counter.current(),
            plant_stats: s
                .plant_stats
                .iter()
                .map(|(id, ps)| (*id, PlantStatsDto::from(ps)))
                .collect(),
            next_plant_id: s.next_plant_id,
            tick_count: s.tick_count,
            config: SimConfigDto::from(&s.config),
        }
    }
}

impl SimStateDto {
    /// Convertit en type domaine. Retourne None si un composant est invalide.
    pub fn to_domain(&self) -> Option<SimState> {
        let mut brains = HashMap::new();
        for (id, brain_dto) in &self.brains {
            let brain = brain_dto.to_domain()?;
            brains.insert(*id, brain);
        }

        let mut plant_stats = HashMap::new();
        for (id, stats_dto) in &self.plant_stats {
            plant_stats.insert(*id, stats_dto.to_domain());
        }

        let seed_bank = self.seed_bank.to_domain()?;

        Some(SimState::from_raw(
            self.world.to_domain(),
            self.island.to_domain(),
            self.plants
                .iter()
                .map(|p| Box::new(p.to_domain()) as Box<dyn crate::domain::traits::PlantEntity>)
                .collect(),
            brains,
            self.symbiosis.to_domain(),
            seed_bank,
            self.season_cycle.to_domain(),
            GenerationCounter::from_count(self.generation_counter),
            plant_stats,
            self.next_plant_id,
            self.tick_count,
            self.config.to_domain(),
        ))
    }
}
