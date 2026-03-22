// Snapshot leger de l'etat de la simulation pour le rendu TUI.
// Sert de pont entre le thread de simulation et le thread UI.

use garden_core::application::season::Season;
use std::collections::HashMap;
use std::collections::VecDeque;

/// Snapshot leger de l'etat de la simulation pour le rendu TUI.
/// Certains champs ne sont pas encore utilises par les widgets mais font
/// partie du contrat de donnees pour les futurs panneaux.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SimSnapshot {
    pub tick: u32,
    pub year: u32,
    pub season: Season,
    pub alive_count: usize,
    pub lineage_count: usize,
    pub symbiosis_count: usize,
    pub average_age: f32,
    pub total_biomass: u32,
    pub best_fitness: f32,
    pub worst_fitness: f32,
    pub generation: u64,
    pub population_history: VecDeque<usize>,
    pub fitness_history: VecDeque<f32>,
    /// Historique du nombre de liens mycorhiziens (derniers 1000 ticks).
    pub symbiosis_history: VecDeque<usize>,
    pub lineage_distribution: HashMap<u64, usize>,
    /// Highlights formates en texte lisible.
    pub recent_highlights: Vec<String>,
    pub paused: bool,
    pub ticks_per_second: f64,
    /// Mini-carte : grille simplifiee (chaque pixel = 4x4 cellules).
    /// 0=mer, 1=terre vide, 2=plante, 3=plante mature
    pub minimap: Vec<Vec<u8>>,

    /// Calques de l'ile (minimaps 32x32, valeurs 0-255)
    /// Chaque calque est une grille simplifiee ou 0=min, 255=max
    pub layer_carbon: Vec<Vec<u8>>,
    pub layer_nitrogen: Vec<Vec<u8>>,
    pub layer_humidity: Vec<Vec<u8>>,
    pub layer_roots: Vec<Vec<u8>>,
    pub layer_canopy: Vec<Vec<u8>>,
    pub layer_footprint: Vec<Vec<u8>>,
    /// Calque actif (0=vue plantes, 1=C, 2=N, 3=Hum, 4=Racines, 5=Canopee, 6=Footprint)
    pub active_layer: u8,

    /// Donnees du calque ile actif (128x128, f32). Vide si pas de calque actif.
    pub island_layer_data: Vec<f32>,
    /// Numero du calque (0=plantes, 1=C, 2=N, 3=H, 4=racines, 5=canopee, 6=footprint)
    pub island_layer_id: u8,

    // --- Demographie ---
    /// Naissances depuis le dernier reset annuel.
    pub births_count: u32,
    /// Morts depuis le dernier reset annuel.
    pub deaths_count: u32,
    /// Naissances l'annee precedente.
    pub births_last_year: u32,
    /// Morts l'annee precedente.
    pub deaths_last_year: u32,

    // --- Distribution des ages ---
    /// [0-100, 100-300, 300-500, 500+]
    pub age_buckets: [u32; 4],

    // --- Distribution Carbon vs Nitrogen ---
    pub carbon_count: usize,
    pub nitrogen_count: usize,

    // --- Cooperation detaillee ---
    pub cooperators_count: usize,
    pub cooperators_ratio: f32,

    // --- Ressources sol moyennes ---
    pub avg_soil_carbon: f32,
    pub avg_soil_nitrogen: f32,

    // --- Couverture ---
    pub land_coverage: f32,
    pub empty_land_cells: usize,

    // --- Echanges cumules ---
    pub total_exchanges_2y: f32,

    // --- Banque de graines ---
    pub bank_compartments: usize,
    pub bank_total_genomes: usize,
    pub bank_best_fitness: f32,
    pub bank_worst_fitness: f32,
    pub bank_spread: f32,
    /// Top 5 genomes de la banque : (fitness, hidden_size, exudate_type, max_size)
    pub bank_top5: Vec<(f32, u8, String, u16)>,
}

impl Default for SimSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            year: 0,
            season: Season::Spring,
            alive_count: 0,
            lineage_count: 0,
            symbiosis_count: 0,
            average_age: 0.0,
            total_biomass: 0,
            best_fitness: 0.0,
            worst_fitness: 0.0,
            generation: 0,
            population_history: VecDeque::new(),
            fitness_history: VecDeque::new(),
            symbiosis_history: VecDeque::new(),
            lineage_distribution: HashMap::new(),
            recent_highlights: Vec::new(),
            paused: false,
            ticks_per_second: 0.0,
            minimap: Vec::new(),
            layer_carbon: Vec::new(),
            layer_nitrogen: Vec::new(),
            layer_humidity: Vec::new(),
            layer_roots: Vec::new(),
            layer_canopy: Vec::new(),
            layer_footprint: Vec::new(),
            active_layer: 0,
            island_layer_data: Vec::new(),
            island_layer_id: 0,
            births_count: 0,
            deaths_count: 0,
            births_last_year: 0,
            deaths_last_year: 0,
            age_buckets: [0; 4],
            carbon_count: 0,
            nitrogen_count: 0,
            cooperators_count: 0,
            cooperators_ratio: 0.0,
            avg_soil_carbon: 0.0,
            avg_soil_nitrogen: 0.0,
            land_coverage: 0.0,
            empty_land_cells: 0,
            total_exchanges_2y: 0.0,
            bank_compartments: 0,
            bank_total_genomes: 0,
            bank_best_fitness: 0.0,
            bank_worst_fitness: 0.0,
            bank_spread: 0.0,
            bank_top5: Vec::new(),
        }
    }
}
