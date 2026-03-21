// Configuration de la simulation — toutes les constantes regroupées ici.

/// Configuration de la simulation. Toutes les constantes sont ici.
pub struct SimConfig {
    // Environnement
    pub rain_rate: f32,
    pub evaporation_rate: f32,
    pub evaporation_canopy_rate: f32,
    pub carbon_regen_rate: f32,
    pub nitrogen_regen_rate: f32,
    pub exudate_decay: f32,
    pub canopy_light: f32,

    // Actions
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

    // Vie et mort
    pub reproduction_energy_min: f32,
    pub reproduction_biomass_min: u16,
    pub reproduction_energy_cost: f32,
    pub reproduction_min_distance: u16,
    pub reproduction_max_distance: u16,
    pub seed_rain_interval: u32,
    pub germination_carbon_min: f32,
    pub germination_nitrogen_min: f32,
    pub dormancy_timeout: u32,

    // Mortalite naturelle
    pub aging_base_rate: f32,
    pub starvation_threshold: f32,
    pub starvation_drain_rate: f32,

    // Decomposition
    pub decomposition_ticks: u32,

    // Banque de graines
    pub seed_bank_capacity: usize,
    pub initial_population: usize,

    // Saisons
    pub ticks_per_season: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            rain_rate: 0.01,
            evaporation_rate: 0.005,
            evaporation_canopy_rate: 0.002,
            carbon_regen_rate: 0.002,
            nitrogen_regen_rate: 0.001,
            exudate_decay: 0.8,
            canopy_light: 0.3,

            growth_threshold: 0.3,
            growth_energy_cost: 8.0,
            growth_carbon_cost: 0.1,
            growth_nitrogen_cost: 0.1,
            invasion_energy_threshold: 10.0,
            invasion_defense_threshold: 20.0,
            invasion_energy_cost: 12.0,
            invasion_damage: 3.0,
            defense_energy_cost: 3.0,
            exudate_output_rate: 0.02,
            exudate_energy_cost_rate: 0.015,
            absorption_rate: 0.02,
            photosynthesis_rate: 0.05,
            maintenance_rate: 0.01,

            reproduction_energy_min: 60.0,
            reproduction_biomass_min: 8,
            reproduction_energy_cost: 30.0,
            reproduction_min_distance: 3,
            reproduction_max_distance: 9,
            seed_rain_interval: 50,
            germination_carbon_min: 0.3,
            germination_nitrogen_min: 0.2,
            dormancy_timeout: 200,

            aging_base_rate: 0.05,
            starvation_threshold: 0.1,
            starvation_drain_rate: 2.0,

            decomposition_ticks: 50,

            seed_bank_capacity: 50,
            initial_population: 30,

            ticks_per_season: 250,
        }
    }
}
