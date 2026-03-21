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
        }
    }
}
