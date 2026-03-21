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
            carbon_regen_rate: 0.0005, // 0.002 → 0.0005 (sol se regenere 4x moins vite)
            nitrogen_regen_rate: 0.0003, // 0.001 → 0.0003 (azote encore plus rare)
            exudate_decay: 0.8,
            canopy_light: 0.2, // 0.3 → 0.2 (plus d'ombre sous canopee)

            growth_threshold: 0.1,
            growth_energy_cost: 5.0,    // 8.0 → 5.0 croissance moins chere
            growth_carbon_cost: 0.05,   // 0.1 → 0.05
            growth_nitrogen_cost: 0.05, // 0.1 → 0.05
            invasion_energy_threshold: 10.0,
            invasion_defense_threshold: 20.0,
            invasion_energy_cost: 12.0,
            invasion_damage: 3.0,
            defense_energy_cost: 3.0,
            exudate_output_rate: 0.02,
            exudate_energy_cost_rate: 0.015,
            absorption_rate: 0.03,     // 0.02 → 0.03 plus d'absorption
            photosynthesis_rate: 0.08, // 0.05 → 0.08 plus de photosynthese
            maintenance_rate: 0.02,    // 0.005 → 0.02 entretien 4x plus cher (carrying capacity)

            reproduction_energy_min: 50.0, // 40.0 → 50.0 reproduction plus couteuse
            reproduction_biomass_min: 6,   // 5 → 6
            reproduction_energy_cost: 25.0, // 20.0 → 25.0
            reproduction_min_distance: 3,
            reproduction_max_distance: 9,
            seed_rain_interval: 30, // 50 → 30 plus de brassage genetique
            germination_carbon_min: 0.1,  // 0.3 → 0.1 (germer plus facilement)
            germination_nitrogen_min: 0.08, // 0.2 → 0.08
            dormancy_timeout: 200,

            aging_base_rate: 0.5, // 0.3 → 0.5 vieillissement accelere (carrying capacity)
            starvation_threshold: 0.1,
            starvation_drain_rate: 3.0, // 2.0 → 3.0 famine plus severe

            decomposition_ticks: 50,

            seed_bank_capacity: 100,
            initial_population: 50, // 80 → 50 population initiale reduite

            ticks_per_season: 250,
        }
    }
}
