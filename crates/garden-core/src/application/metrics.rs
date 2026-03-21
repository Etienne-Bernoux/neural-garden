// Metriques agregees de la simulation, mises a jour a chaque tick.

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::domain::plant::{Plant, PlantState};

use super::highlights::{Highlight, HighlightDetector};
use super::season::Season;

/// Capacite maximale des historiques (derniers 1000 ticks).
const HISTORY_CAPACITY: usize = 1000;

/// Metriques agregees de la simulation, mises a jour a chaque tick.
pub struct SimMetrics {
    /// Nombre de plantes vivantes (ni Dead ni Decomposing).
    pub alive_count: usize,
    /// Repartition par lignee : lineage_id -> nombre de plantes vivantes.
    pub lineage_distribution: HashMap<u64, usize>,
    /// Nombre de liens mycorhiziens actifs.
    pub symbiosis_count: usize,
    /// Historique de population (derniers 1000 ticks).
    pub population_history: VecDeque<usize>,
    /// Historique de best fitness (derniers 1000 ticks).
    pub fitness_history: VecDeque<f32>,
    /// Historique du nombre de liens mycorhiziens (derniers 1000 ticks).
    pub symbiosis_history: VecDeque<usize>,
    /// Nombre de lignees distinctes vivantes.
    pub lineage_count: usize,
    /// Age moyen des plantes vivantes.
    pub average_age: f32,
    /// Biomasse totale.
    pub total_biomass: u32,
    /// Highlights detectes au dernier tick.
    pub recent_highlights: Vec<Highlight>,
    /// Detecteur de highlights (etat interne persiste entre les ticks).
    pub highlight_detector: HighlightDetector,
}

impl SimMetrics {
    /// Cree des metriques vides.
    pub fn new() -> Self {
        Self {
            alive_count: 0,
            lineage_distribution: HashMap::new(),
            symbiosis_count: 0,
            population_history: VecDeque::new(),
            fitness_history: VecDeque::new(),
            symbiosis_history: VecDeque::new(),
            lineage_count: 0,
            average_age: 0.0,
            total_biomass: 0,
            recent_highlights: Vec::new(),
            highlight_detector: HighlightDetector::new(),
        }
    }
}

impl Default for SimMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Met a jour les metriques a partir de l'etat courant de la simulation.
pub fn update_metrics(
    metrics: &mut SimMetrics,
    plants: &[Plant],
    symbiosis_count: usize,
    best_fitness: f32,
) {
    // Reinitialiser les compteurs
    metrics.lineage_distribution.clear();
    metrics.alive_count = 0;
    metrics.symbiosis_count = symbiosis_count;
    let mut total_age: u64 = 0;
    let mut total_biomass: u32 = 0;

    for plant in plants {
        if plant.is_dead() {
            continue;
        }
        // Les graines comptent aussi comme vivantes
        if plant.state() == PlantState::Seed {
            // comptee normalement
        }
        metrics.alive_count += 1;
        *metrics
            .lineage_distribution
            .entry(plant.lineage().id())
            .or_insert(0) += 1;
        total_age += plant.age() as u64;
        total_biomass += plant.biomass().value() as u32;
    }

    metrics.lineage_count = metrics.lineage_distribution.len();
    metrics.average_age = if metrics.alive_count > 0 {
        total_age as f32 / metrics.alive_count as f32
    } else {
        0.0
    };
    metrics.total_biomass = total_biomass;

    // Historiques (garder les derniers HISTORY_CAPACITY)
    metrics.population_history.push_back(metrics.alive_count);
    if metrics.population_history.len() > HISTORY_CAPACITY {
        metrics.population_history.pop_front();
    }
    metrics.fitness_history.push_back(best_fitness);
    if metrics.fitness_history.len() > HISTORY_CAPACITY {
        metrics.fitness_history.pop_front();
    }

    metrics.symbiosis_history.push_back(symbiosis_count);
    if metrics.symbiosis_history.len() > HISTORY_CAPACITY {
        metrics.symbiosis_history.pop_front();
    }
}

/// Detecte les highlights et les stocke dans les metriques.
pub fn detect_highlights(
    metrics: &mut SimMetrics,
    events: &[crate::domain::events::DomainEvent],
    tick: u32,
    best_fitness: f32,
    season_changed: Option<Season>,
) {
    let highlights = metrics.highlight_detector.detect(
        events,
        tick,
        metrics.alive_count,
        best_fitness,
        season_changed,
        &metrics.lineage_distribution,
    );
    metrics.recent_highlights = highlights;
}

#[cfg(test)]
mod tests {
    use crate::application::sim::{run_tick, SimState};
    use crate::domain::rng::test_utils::MockRng;

    #[test]
    fn les_metriques_comptent_les_plantes_vivantes() {
        let mut rng = MockRng::new(0.3, 0.07);
        let mut state = SimState::new(0.5, 5, &mut rng);

        // Faire tourner quelques ticks
        for _ in 0..5 {
            run_tick(&mut state, &mut rng);
        }

        assert!(
            state.metrics.alive_count > 0,
            "il devrait y avoir des plantes vivantes apres quelques ticks"
        );
    }

    #[test]
    fn les_metriques_suivent_les_lignees() {
        let mut rng = MockRng::new(0.3, 0.07);
        let mut state = SimState::new(0.5, 5, &mut rng);

        // Faire tourner un tick pour mettre a jour les metriques
        run_tick(&mut state, &mut rng);

        // Verifier que la distribution de lignees contient au moins une entree
        if state.metrics.alive_count > 0 {
            assert!(
                !state.metrics.lineage_distribution.is_empty(),
                "lineage_distribution devrait contenir au moins une lignee"
            );

            // La somme des compteurs doit correspondre au nombre de vivantes
            let total: usize = state.metrics.lineage_distribution.values().sum();
            assert_eq!(
                total, state.metrics.alive_count,
                "la somme des lignees ({}) doit correspondre a alive_count ({})",
                total, state.metrics.alive_count
            );
        }
    }

    #[test]
    fn lhistorique_population_se_remplit() {
        let mut rng = MockRng::new(0.3, 0.07);
        let mut state = SimState::new(0.5, 5, &mut rng);

        let n = 10;
        for _ in 0..n {
            run_tick(&mut state, &mut rng);
        }

        assert_eq!(
            state.metrics.population_history.len(),
            n,
            "population_history devrait contenir {} entrees apres {} ticks",
            n,
            n
        );
    }
}
