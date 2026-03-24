// Snapshot leger de l'etat de la pepiniere pour le rendu TUI.
// Sert de pont entre le thread nursery et le thread UI.

use garden_core::application::evolution::PlantStats;
use garden_core::application::nursery::{BedConfig, GenerationReport, NurseryResult};
use garden_core::domain::plant::GeneticTraits;

/// Message envoye du thread nursery vers le thread TUI.
pub enum NurseryUpdate {
    /// Rapport de progression d'une generation pour un environnement.
    Generation {
        env_name: String,
        report: GenerationReport,
    },
    /// La nursery a termine tous les environnements.
    Finished { results: Vec<NurseryResult> },
}

/// Mode d'affichage du TUI nursery.
pub enum NurseryViewMode {
    /// Vue recap : tableau de tous les environnements.
    Recap,
    /// Vue zoom : detail d'un environnement (index).
    Zoom(usize),
}

/// Entree dans l'historique d'une generation.
pub struct GenHistoryEntry {
    pub generation: u32,
    pub best: f32,
    pub avg: f32,
    pub worst: f32,
}

/// Snapshot d'un environnement pour le rendu TUI.
pub struct EnvSnapshot {
    pub name: String,
    pub current_gen: u32,
    pub best_fitness: f32,
    pub avg_fitness: f32,
    pub worst_fitness: f32,
    pub delta_best: f32,
    pub elapsed_secs: f64,
    pub champion_stats: Option<PlantStats>,
    pub champion_traits: Option<GeneticTraits>,
    pub history: Vec<GenHistoryEntry>,
    pub bed_config: BedConfig,
}

/// Etat complet de la pepiniere pour le rendu TUI.
pub struct NurserySnapshot {
    pub total_generations: u32,
    pub population: usize,
    pub seed: u64,
    pub paused: bool,
    pub finished: bool,
    pub envs: Vec<EnvSnapshot>,
    pub selected_env: usize,
    pub results: Option<Vec<NurseryResult>>,
}

impl NurserySnapshot {
    /// Construit un snapshot initial a partir de la config.
    pub fn new(
        envs: &[(String, BedConfig)],
        total_generations: u32,
        population: usize,
        seed: u64,
    ) -> Self {
        let env_snapshots = envs
            .iter()
            .map(|(name, config)| EnvSnapshot {
                name: name.clone(),
                current_gen: 0,
                best_fitness: 0.0,
                avg_fitness: 0.0,
                worst_fitness: 0.0,
                delta_best: 0.0,
                elapsed_secs: 0.0,
                champion_stats: None,
                champion_traits: None,
                history: Vec::new(),
                bed_config: config.clone(),
            })
            .collect();

        Self {
            total_generations,
            population,
            seed,
            paused: false,
            finished: false,
            envs: env_snapshots,
            selected_env: 0,
            results: None,
        }
    }

    /// Integre un message recu du thread nursery.
    pub fn apply_update(&mut self, update: NurseryUpdate) {
        match update {
            NurseryUpdate::Generation { env_name, report } => {
                // Trouver l'env correspondant
                if let Some(env) = self.envs.iter_mut().find(|e| e.name == env_name) {
                    let prev_best = env.best_fitness;
                    env.current_gen = report.generation + 1; // 0-indexed -> affichage 1-indexed
                    env.avg_fitness = report.avg_fitness;
                    env.worst_fitness = report.worst_fitness;
                    env.delta_best = report.best_fitness - prev_best;
                    env.elapsed_secs = report.elapsed_secs;
                    // Le champion n'est mis a jour que si on a un nouveau record
                    if report.best_fitness >= env.best_fitness {
                        env.best_fitness = report.best_fitness;
                        env.champion_stats = report.champion_stats.clone();
                        env.champion_traits = report.champion_traits.clone();
                    }
                    env.history.push(GenHistoryEntry {
                        generation: report.generation,
                        best: report.best_fitness,
                        avg: report.avg_fitness,
                        worst: report.worst_fitness,
                    });
                }
            }
            NurseryUpdate::Finished { results } => {
                self.finished = true;
                self.results = Some(results);
            }
        }
    }

    /// Generation maximale atteinte parmi tous les envs.
    pub fn max_gen(&self) -> u32 {
        self.envs.iter().map(|e| e.current_gen).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use garden_core::application::evolution::Genome;
    use garden_core::domain::brain::Brain;
    use garden_core::domain::plant::ExudateType;

    /// Helper : construit un Genome minimal pour les tests.
    fn dummy_genome() -> Genome {
        // hidden_size=6 -> 18*6 + 6*6 + 6*8 + 6 + 6 + 8 = 212 poids
        let weights = vec![0.0_f32; 212];
        Genome {
            brain: Brain::from_weights(6, weights).unwrap(),
            traits: GeneticTraits::new(20, 0.5, ExudateType::Carbon, 6, 1.0, 1.0),
        }
    }

    /// Helper : construit un GenerationReport minimal pour les tests.
    fn make_report(generation: u32, best: f32, avg: f32, worst: f32) -> GenerationReport {
        GenerationReport {
            generation,
            best_fitness: best,
            avg_fitness: avg,
            worst_fitness: worst,
            elapsed_secs: 0.1,
            champion_stats: None,
            champion_traits: None,
        }
    }

    /// Helper : construit 2 envs par defaut pour les tests.
    fn two_envs() -> Vec<(String, BedConfig)> {
        vec![
            ("Solo riche".to_string(), BedConfig::default()),
            ("Solo pauvre".to_string(), BedConfig::default()),
        ]
    }

    #[test]
    fn snapshot_initial_a_le_bon_nombre_d_envs() {
        let envs = two_envs();
        let snap = NurserySnapshot::new(&envs, 50, 20, 42);

        assert_eq!(snap.envs.len(), 2);
        assert_eq!(snap.total_generations, 50);
        assert_eq!(snap.population, 20);
        assert_eq!(snap.seed, 42);
        assert!(!snap.finished);
    }

    #[test]
    fn apply_update_generation_met_a_jour_l_env() {
        let envs = two_envs();
        let mut snap = NurserySnapshot::new(&envs, 50, 20, 42);

        let report = make_report(0, 10.0, 5.0, 1.0);
        snap.apply_update(NurseryUpdate::Generation {
            env_name: "Solo riche".to_string(),
            report,
        });

        // L'env "Solo riche" doit etre mis a jour
        let solo_riche = &snap.envs[0];
        assert_eq!(solo_riche.current_gen, 1); // 0-indexed + 1
        assert!((solo_riche.best_fitness - 10.0).abs() < f32::EPSILON);
        assert_eq!(solo_riche.history.len(), 1);

        // L'env "Solo pauvre" ne doit pas avoir change
        let solo_pauvre = &snap.envs[1];
        assert_eq!(solo_pauvre.current_gen, 0);
        assert!((solo_pauvre.best_fitness - 0.0).abs() < f32::EPSILON);
        assert!(solo_pauvre.history.is_empty());
    }

    #[test]
    fn apply_update_calcule_le_delta_best() {
        let envs = two_envs();
        let mut snap = NurserySnapshot::new(&envs, 50, 20, 42);

        // Generation 0 : best = 10.0, delta = 10.0 - 0.0 = 10.0
        snap.apply_update(NurseryUpdate::Generation {
            env_name: "Solo riche".to_string(),
            report: make_report(0, 10.0, 5.0, 1.0),
        });
        assert!((snap.envs[0].delta_best - 10.0).abs() < f32::EPSILON);

        // Generation 1 : best = 15.0, delta = 15.0 - 10.0 = 5.0
        snap.apply_update(NurseryUpdate::Generation {
            env_name: "Solo riche".to_string(),
            report: make_report(1, 15.0, 8.0, 3.0),
        });
        assert!((snap.envs[0].delta_best - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_update_finished_marque_termine() {
        let envs = two_envs();
        let mut snap = NurserySnapshot::new(&envs, 50, 20, 42);

        snap.apply_update(NurseryUpdate::Finished {
            results: vec![NurseryResult {
                env_name: "Solo riche".to_string(),
                champion: dummy_genome(),
                fitness: 42.0,
                generations_run: 50,
            }],
        });

        assert!(snap.finished);
        assert!(snap.results.is_some());
    }

    #[test]
    fn max_gen_retourne_la_generation_la_plus_avancee() {
        let envs = two_envs();
        let mut snap = NurserySnapshot::new(&envs, 50, 20, 42);

        // Avancer "Solo riche" a gen 3
        for g in 0..3 {
            snap.apply_update(NurseryUpdate::Generation {
                env_name: "Solo riche".to_string(),
                report: make_report(g, 10.0, 5.0, 1.0),
            });
        }
        // Avancer "Solo pauvre" a gen 5
        for g in 0..5 {
            snap.apply_update(NurseryUpdate::Generation {
                env_name: "Solo pauvre".to_string(),
                report: make_report(g, 8.0, 4.0, 0.5),
            });
        }

        // max_gen = max(3, 5) = 5 (current_gen est 1-indexed)
        assert_eq!(snap.max_gen(), 5);
    }
}
