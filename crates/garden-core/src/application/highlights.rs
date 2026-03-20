// Service de detection des moments cles (highlights) de la simulation.

use crate::application::season::Season;
use crate::domain::events::DomainEvent;

use std::collections::HashMap;

/// Type de moment cle detecte.
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightType {
    /// Premiere symbiose de la simulation
    FirstSymbiosis,
    /// Une plante envahit > 5 cellules en un tick
    MajorInvasion { invader_id: u64, cells_taken: usize },
    /// Le dernier individu d'une lignee meurt
    LineageExtinction { lineage_id: u64 },
    /// Nouvelle lignee detectee
    NewLineage {
        lineage_id: u64,
        parent_lineage_id: u64,
    },
    /// Nouveau record de fitness dans la banque
    FitnessRecord { fitness: f32 },
    /// Population remonte apres un creux
    PopulationBoom { population: usize },
    /// Plus de 30% de la population meurt en < 50 ticks
    MassDeath {
        deaths: usize,
        population_before: usize,
    },
    /// Transition de saison
    SeasonChange { season: Season },
}

/// Un moment cle detecte dans la simulation.
#[derive(Debug, Clone)]
pub struct Highlight {
    pub tick: u32,
    pub highlight_type: HighlightType,
    pub score: f32,
}

/// Detecteur de moments cles dans la simulation.
pub struct HighlightDetector {
    first_symbiosis_seen: bool,
    best_fitness_seen: f32,
    recent_deaths: Vec<u32>,
    has_initial_population: bool,
    min_population_seen: usize,
    population_history: Vec<usize>,
}

impl HighlightDetector {
    /// Cree un nouveau detecteur.
    pub fn new() -> Self {
        Self {
            first_symbiosis_seen: false,
            best_fitness_seen: 0.0,
            recent_deaths: Vec::new(),
            has_initial_population: false,
            min_population_seen: 0,
            population_history: Vec::new(),
        }
    }

    /// Analyse les events et l'etat de la simulation pour detecter les moments cles.
    pub fn detect(
        &mut self,
        events: &[DomainEvent],
        tick: u32,
        population: usize,
        best_fitness: f32,
        season_changed: Option<Season>,
    ) -> Vec<Highlight> {
        let mut highlights = Vec::new();

        // Mettre a jour l'historique de population
        self.population_history.push(population);
        if self.population_history.len() > 50 {
            self.population_history.remove(0);
        }

        // Mettre a jour le minimum de population
        if !self.has_initial_population {
            self.has_initial_population = true;
            self.min_population_seen = population;
        } else if population < self.min_population_seen {
            self.min_population_seen = population;
        }

        // Nettoyer les morts trop anciennes (> 50 ticks)
        self.recent_deaths.retain(|&t| tick.saturating_sub(t) < 50);

        // Compter les morts de ce tick
        let deaths_this_tick = events
            .iter()
            .filter(|e| matches!(e, DomainEvent::Died { .. }))
            .count();

        // Ajouter les morts de ce tick
        for _ in 0..deaths_this_tick {
            self.recent_deaths.push(tick);
        }

        // 1. FirstSymbiosis
        if !self.first_symbiosis_seen
            && events
                .iter()
                .any(|e| matches!(e, DomainEvent::Linked { .. }))
        {
            self.first_symbiosis_seen = true;
            highlights.push(Highlight {
                tick,
                highlight_type: HighlightType::FirstSymbiosis,
                score: 1.0,
            });
        }

        // 2. MajorInvasion : compter les invasions par invader
        let mut invasion_counts: HashMap<u64, usize> = HashMap::new();
        for event in events {
            if let DomainEvent::Invaded { invader_id, .. } = event {
                *invasion_counts.entry(*invader_id).or_insert(0) += 1;
            }
        }
        for (invader_id, count) in &invasion_counts {
            if *count > 5 {
                highlights.push(Highlight {
                    tick,
                    highlight_type: HighlightType::MajorInvasion {
                        invader_id: *invader_id,
                        cells_taken: *count,
                    },
                    score: 0.7,
                });
            }
        }

        // 3. LineageExtinction — placeholder, skip pour l'instant

        // 4. NewLineage
        for event in events {
            if let DomainEvent::LineageFork {
                parent_lineage,
                child_lineage,
                ..
            } = event
            {
                highlights.push(Highlight {
                    tick,
                    highlight_type: HighlightType::NewLineage {
                        lineage_id: child_lineage.id(),
                        parent_lineage_id: parent_lineage.id(),
                    },
                    score: 0.5,
                });
            }
        }

        // 5. FitnessRecord
        if best_fitness > self.best_fitness_seen {
            self.best_fitness_seen = best_fitness;
            highlights.push(Highlight {
                tick,
                highlight_type: HighlightType::FitnessRecord {
                    fitness: best_fitness,
                },
                score: 0.8,
            });
        }

        // 6. PopulationBoom
        if population > 10
            && self.has_initial_population
            && self.min_population_seen > 0
            && population > self.min_population_seen * 2
        {
            highlights.push(Highlight {
                tick,
                highlight_type: HighlightType::PopulationBoom { population },
                score: 0.6,
            });
        }

        // 7. MassDeath : si les morts recentes > 30% de la population avant les morts
        let total_recent_deaths = self.recent_deaths.len();
        let population_before = population + deaths_this_tick;
        if population_before > 0
            && total_recent_deaths > 0
            && total_recent_deaths as f32 / population_before as f32 > 0.3
        {
            highlights.push(Highlight {
                tick,
                highlight_type: HighlightType::MassDeath {
                    deaths: total_recent_deaths,
                    population_before,
                },
                score: 0.9,
            });
        }

        // 8. SeasonChange
        if let Some(season) = season_changed {
            highlights.push(Highlight {
                tick,
                highlight_type: HighlightType::SeasonChange { season },
                score: 0.3,
            });
        }

        highlights
    }
}

impl Default for HighlightDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plant::Pos;

    #[test]
    fn premiere_symbiose_detectee() {
        let mut detector = HighlightDetector::new();
        let events = vec![DomainEvent::Linked {
            plant_a: 1,
            plant_b: 2,
        }];

        let highlights = detector.detect(&events, 10, 5, 0.0, None);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].highlight_type, HighlightType::FirstSymbiosis);
        assert!((highlights[0].score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn premiere_symbiose_une_seule_fois() {
        let mut detector = HighlightDetector::new();
        let events = vec![DomainEvent::Linked {
            plant_a: 1,
            plant_b: 2,
        }];

        // Premiere detection
        let highlights = detector.detect(&events, 10, 5, 0.0, None);
        assert!(highlights
            .iter()
            .any(|h| h.highlight_type == HighlightType::FirstSymbiosis));

        // Deuxieme appel — pas de FirstSymbiosis
        let events2 = vec![DomainEvent::Linked {
            plant_a: 3,
            plant_b: 4,
        }];
        let highlights2 = detector.detect(&events2, 20, 5, 0.0, None);
        assert!(!highlights2
            .iter()
            .any(|h| h.highlight_type == HighlightType::FirstSymbiosis));
    }

    #[test]
    fn invasion_majeure_detectee() {
        let mut detector = HighlightDetector::new();
        let events: Vec<DomainEvent> = (0..6)
            .map(|i| DomainEvent::Invaded {
                invader_id: 42,
                victim_id: 100 + i,
                cell: Pos { x: i as u16, y: 0 },
            })
            .collect();

        let highlights = detector.detect(&events, 10, 20, 0.0, None);

        let invasion = highlights
            .iter()
            .find(|h| matches!(h.highlight_type, HighlightType::MajorInvasion { .. }));
        assert!(invasion.is_some());
        if let HighlightType::MajorInvasion {
            invader_id,
            cells_taken,
        } = &invasion.unwrap().highlight_type
        {
            assert_eq!(*invader_id, 42);
            assert_eq!(*cells_taken, 6);
        }
    }

    #[test]
    fn record_fitness_detecte() {
        let mut detector = HighlightDetector::new();

        // Premier appel avec fitness 10 → record
        let highlights = detector.detect(&[], 10, 5, 10.0, None);
        let record = highlights
            .iter()
            .find(|h| matches!(h.highlight_type, HighlightType::FitnessRecord { .. }));
        assert!(record.is_some());

        // Deuxieme appel avec fitness 10 → pas de record
        let highlights2 = detector.detect(&[], 20, 5, 10.0, None);
        assert!(!highlights2
            .iter()
            .any(|h| matches!(h.highlight_type, HighlightType::FitnessRecord { .. })));

        // Troisieme appel avec fitness 15 → nouveau record
        let highlights3 = detector.detect(&[], 30, 5, 15.0, None);
        let record3 = highlights3
            .iter()
            .find(|h| matches!(h.highlight_type, HighlightType::FitnessRecord { .. }));
        assert!(record3.is_some());
        if let HighlightType::FitnessRecord { fitness } = &record3.unwrap().highlight_type {
            assert!((fitness - 15.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn changement_saison_detecte() {
        let mut detector = HighlightDetector::new();

        let highlights = detector.detect(&[], 250, 10, 0.0, Some(Season::Summer));

        let season_change = highlights
            .iter()
            .find(|h| matches!(h.highlight_type, HighlightType::SeasonChange { .. }));
        assert!(season_change.is_some());
        if let HighlightType::SeasonChange { season } = &season_change.unwrap().highlight_type {
            assert_eq!(*season, Season::Summer);
        }
        assert!((season_change.unwrap().score - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn mort_de_masse_detectee() {
        let mut detector = HighlightDetector::new();

        // 10 morts en 1 tick, population actuelle = 10 (donc avant = 20)
        // 10 / 20 = 50% > 30%
        let events: Vec<DomainEvent> = (0..10)
            .map(|i| DomainEvent::Died {
                plant_id: i,
                position: Pos { x: 0, y: 0 },
                age: 100,
                biomass: 5,
            })
            .collect();

        let highlights = detector.detect(&events, 10, 10, 0.0, None);

        let mass_death = highlights
            .iter()
            .find(|h| matches!(h.highlight_type, HighlightType::MassDeath { .. }));
        assert!(mass_death.is_some());
        if let HighlightType::MassDeath {
            deaths,
            population_before,
        } = &mass_death.unwrap().highlight_type
        {
            assert_eq!(*deaths, 10);
            assert_eq!(*population_before, 20);
        }
        assert!((mass_death.unwrap().score - 0.9).abs() < f32::EPSILON);
    }
}
