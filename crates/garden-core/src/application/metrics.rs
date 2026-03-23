// Metriques agregees de la simulation, mises a jour a chaque tick.

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::domain::plant::{ExudateType, PlantState};
use crate::domain::traits::PlantEntity;
use crate::domain::symbiosis::SymbiosisNetwork;

use super::evolution::SeedBank;
use super::highlights::{Highlight, HighlightDetector};
use super::season::Season;

/// Capacite maximale des historiques (derniers 1000 ticks).
const HISTORY_CAPACITY: usize = 1000;

/// Taille de la fenetre glissante d'echanges (2 ans = 2880 ticks).
const EXCHANGES_WINDOW_SIZE: usize = 2880;

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

    // --- Demographie ---
    /// Naissances depuis le dernier reset annuel.
    pub births_count: u32,
    /// Morts depuis le dernier reset annuel.
    pub deaths_count: u32,
    /// Naissances l'annee precedente (gele au changement d'annee).
    pub births_last_year: u32,
    /// Morts l'annee precedente.
    pub deaths_last_year: u32,

    // --- Distribution des ages (buckets) ---
    /// [0-100, 100-300, 300-500, 500+]
    pub age_buckets: [u32; 4],

    // --- Distribution Carbon vs Nitrogen ---
    /// Nombre de plantes vivantes avec exudate_type = Carbon.
    pub carbon_count: usize,
    /// Nombre de plantes vivantes avec exudate_type = Nitrogen.
    pub nitrogen_count: usize,

    // --- Cooperation detaillee ---
    /// Nombre de plantes vivantes avec au moins 1 lien mycorhizien.
    pub cooperators_count: usize,
    /// cooperators_count / alive_count.
    pub cooperators_ratio: f32,

    // --- Ressources sol moyennes ---
    /// Carbone moyen des cellules terrestres.
    pub avg_soil_carbon: f32,
    /// Azote moyen des cellules terrestres.
    pub avg_soil_nitrogen: f32,

    // --- Couverture ---
    /// Pourcentage de cellules terrestres occupees par un footprint.
    pub land_coverage: f32,
    /// Nombre de cellules terrestres vides.
    pub empty_land_cells: usize,

    // --- Echanges cumules (fenetre glissante 2 ans = 2880 ticks) ---
    /// Echanges par tick sur les 2880 derniers ticks.
    pub exchanges_window: VecDeque<f32>,
    /// Somme de la fenetre d'echanges.
    pub total_exchanges_2y: f32,
    /// Echanges du tick courant (remis a 0 a chaque tick, incremente dans action_symbiosis).
    pub tick_exchanges: f32,

    // --- Banque de graines stats ---
    /// Nombre de compartiments actifs dans la banque.
    pub bank_compartments: usize,
    /// Nombre total de genomes dans la banque.
    pub bank_total_genomes: usize,
    /// Meilleure fitness dans la banque.
    pub bank_best_fitness: f32,
    /// Pire fitness dans la banque.
    pub bank_worst_fitness: f32,
    /// Indicateur de diversite (best - worst) / best.
    pub bank_spread: f32,
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
            exchanges_window: VecDeque::new(),
            total_exchanges_2y: 0.0,
            tick_exchanges: 0.0,
            bank_compartments: 0,
            bank_total_genomes: 0,
            bank_best_fitness: 0.0,
            bank_worst_fitness: 0.0,
            bank_spread: 0.0,
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
    plants: &[Box<dyn PlantEntity>],
    symbiosis: &SymbiosisNetwork,
    world: &crate::domain::world::World,
    island: &crate::domain::island::Island,
    best_fitness: f32,
    seed_bank: &SeedBank,
) {
    let symbiosis_count = symbiosis.link_count();

    // Reinitialiser les compteurs
    metrics.lineage_distribution.clear();
    metrics.alive_count = 0;
    metrics.symbiosis_count = symbiosis_count;
    let mut total_age: u64 = 0;
    let mut total_biomass: u32 = 0;

    // Distribution des ages
    metrics.age_buckets = [0; 4];
    // Carbon vs Nitrogen
    metrics.carbon_count = 0;
    metrics.nitrogen_count = 0;

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

        // Distribution des ages (exclure les graines)
        if plant.state() != PlantState::Seed {
            let age = plant.age();
            if age < 100 {
                metrics.age_buckets[0] += 1;
            } else if age < 300 {
                metrics.age_buckets[1] += 1;
            } else if age < 500 {
                metrics.age_buckets[2] += 1;
            } else {
                metrics.age_buckets[3] += 1;
            }
        }

        // Carbon vs Nitrogen (exclure les graines)
        if plant.state() != PlantState::Seed {
            match plant.genetics().exudate_type() {
                ExudateType::Carbon => metrics.carbon_count += 1,
                ExudateType::Nitrogen => metrics.nitrogen_count += 1,
            }
        }
    }

    metrics.lineage_count = metrics.lineage_distribution.len();
    metrics.average_age = if metrics.alive_count > 0 {
        total_age as f32 / metrics.alive_count as f32
    } else {
        0.0
    };
    metrics.total_biomass = total_biomass;

    // Cooperateurs : plantes vivantes avec au moins 1 lien mycorhizien
    let mut linked_plants: HashSet<u64> = HashSet::new();
    for link in symbiosis.links() {
        linked_plants.insert(link.plant_a());
        linked_plants.insert(link.plant_b());
    }
    metrics.cooperators_count = plants
        .iter()
        .filter(|p| !p.is_dead() && linked_plants.contains(&p.id()))
        .count();
    metrics.cooperators_ratio = if metrics.alive_count > 0 {
        metrics.cooperators_count as f32 / metrics.alive_count as f32
    } else {
        0.0
    };

    // Ressources moyennes sol (cellules terrestres)
    let land_cells = island.land_cells();
    let land_count = land_cells.len();
    let mut total_c = 0.0_f32;
    let mut total_n = 0.0_f32;
    for pos in land_cells {
        if let Some(cell) = world.get(pos) {
            total_c += cell.carbon();
            total_n += cell.nitrogen();
        }
    }
    metrics.avg_soil_carbon = if land_count > 0 {
        total_c / land_count as f32
    } else {
        0.0
    };
    metrics.avg_soil_nitrogen = if land_count > 0 {
        total_n / land_count as f32
    } else {
        0.0
    };

    // Couverture : compter les cellules occupees par un footprint
    let mut occupied = 0_usize;
    for plant in plants {
        if plant.is_dead() {
            continue;
        }
        occupied += plant.footprint().len();
    }
    metrics.land_coverage = if land_count > 0 {
        occupied as f32 / land_count as f32
    } else {
        0.0
    };
    metrics.empty_land_cells = land_count.saturating_sub(occupied);

    // Fenetre glissante d'echanges (2 ans = 2880 ticks)
    metrics.exchanges_window.push_back(metrics.tick_exchanges);
    if metrics.exchanges_window.len() > EXCHANGES_WINDOW_SIZE {
        metrics.exchanges_window.pop_front();
    }
    metrics.total_exchanges_2y = metrics.exchanges_window.iter().sum();
    // Remettre tick_exchanges a 0 pour le prochain tick
    metrics.tick_exchanges = 0.0;

    // Banque de graines
    metrics.bank_compartments = seed_bank.compartment_count();
    metrics.bank_total_genomes = seed_bank.len();
    metrics.bank_best_fitness = best_fitness;
    metrics.bank_worst_fitness = seed_bank.worst_fitness();
    metrics.bank_spread = seed_bank.diversity_spread();

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
