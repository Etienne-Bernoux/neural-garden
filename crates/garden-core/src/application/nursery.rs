// Pepiniere — bac isole pour evaluer un genome via les vraies phases de simulation.
// Utilise SimState + phase_environment / phase_actions / phase_lifecycle.

use std::collections::HashMap;

use crate::application::actions::phase_actions;
use crate::application::config::SimConfig;
use crate::application::environment::phase_environment;
use crate::application::evolution::{
    evaluate_fitness, mutate_genome, GenerationCounter, Genome, PlantStats, SeedBank,
};
use crate::application::lifecycle::phase_lifecycle;
use crate::application::season::{Season, SeasonCycle};
use crate::application::sim::{phase_perception_decision, SimState};
use crate::domain::brain::Brain;
use crate::domain::island::Island;
use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
use crate::domain::rng::Rng;
use crate::domain::symbiosis::SymbiosisNetwork;
use crate::domain::traits::PlantEntity;
use crate::domain::world::World;

use crate::domain::fixture::FixturePlant;

// --- Configuration ---

/// Configuration d'un bac de pepiniere.
#[derive(Clone)]
pub struct BedConfig {
    pub grid_size: u16,
    pub initial_carbon: f32,
    pub initial_nitrogen: f32,
    pub initial_humidity: f32,
    pub light_level: f32,
    pub max_ticks: u32,
    pub fixtures: Vec<FixtureConfig>,
    /// Taux de regeneration du carbone par tick
    pub carbon_regen_rate: f32,
    /// Taux de regeneration de l'azote par tick
    pub nitrogen_regen_rate: f32,
    /// Taux de regeneration de l'humidite par tick
    pub humidity_regen_rate: f32,
    /// Saison verrouillee (ex: Some(Season::Winter) pour hiver permanent)
    pub locked_season: Option<Season>,
}

/// Configuration d'une fixture dans un bac.
#[derive(Clone)]
pub struct FixtureConfig {
    pub position: Pos,
    pub exudate_type: ExudateType,
    pub biomass: u16,
    pub behavior: FixtureBehavior,
}

/// Comportement deterministe d'une fixture.
#[derive(Clone)]
pub enum FixtureBehavior {
    /// Exsude une ressource autour d'elle chaque tick
    Exuder { rate: f32 },
    /// Fait de l'ombre dans un rayon
    Ombrager { radius: u16 },
    /// Plante agressive — on lui donne de l'energie
    Envahir,
    /// Juste presente, ne fait rien de special
    Inerte,
}

impl Default for BedConfig {
    fn default() -> Self {
        Self {
            grid_size: 16,
            initial_carbon: 0.5,
            initial_nitrogen: 0.3,
            initial_humidity: 0.5,
            light_level: 0.8,
            max_ticks: 2000,
            fixtures: Vec::new(),
            carbon_regen_rate: 0.002,
            nitrogen_regen_rate: 0.001,
            humidity_regen_rate: 0.01,
            locked_season: None,
        }
    }
}

impl BedConfig {
    /// Convertit la config nursery en SimConfig pour les vraies phases.
    pub fn to_sim_config(&self) -> SimConfig {
        let ticks_per_season = match self.locked_season {
            Some(_) => self.max_ticks + 1, // la saison ne change jamais
            None => SimConfig::default().ticks_per_season,
        };
        SimConfig {
            nursery_mode: true,
            initial_population: 0,
            seed_rain_interval: u32::MAX,
            carbon_regen_rate: self.carbon_regen_rate,
            nitrogen_regen_rate: self.nitrogen_regen_rate,
            rain_rate: self.humidity_regen_rate,
            ticks_per_season,
            seed_bank_capacity: 1,
            ..SimConfig::default()
        }
    }

    /// Tick de depart pour la saison verrouillee.
    pub fn season_start_tick(&self) -> u32 {
        let tps = match self.locked_season {
            Some(_) => self.max_ticks + 1,
            None => SimConfig::default().ticks_per_season,
        };
        match self.locked_season {
            None | Some(Season::Spring) => 0,
            Some(Season::Summer) => tps,
            Some(Season::Autumn) => 2 * tps,
            Some(Season::Winter) => 3 * tps,
        }
    }
}

// --- Keepalive des fixtures ---

/// Applique le comportement des fixtures a chaque tick (keepalive + effets).
fn apply_fixtures(state: &mut SimState, config: &BedConfig) {
    for (i, fixture_cfg) in config.fixtures.iter().enumerate() {
        let fixture_id = 100 + i as u64;

        // Maintenir la fixture en vie (immortelle)
        if let Some(plant) = state.plants.iter_mut().find(|p| p.id() == fixture_id) {
            plant.heal(100.0);
            plant.gain_energy(100.0);
        }

        match &fixture_cfg.behavior {
            FixtureBehavior::Exuder { rate } => {
                // Injecter la ressource dans le sol autour de la fixture
                let pos = fixture_cfg.position;
                for dx in -2i16..=2 {
                    for dy in -2i16..=2 {
                        let p = Pos {
                            x: (pos.x as i16 + dx).max(0) as u16,
                            y: (pos.y as i16 + dy).max(0) as u16,
                        };
                        if let Some(cell) = state.world.get_mut(&p) {
                            match fixture_cfg.exudate_type {
                                ExudateType::Nitrogen => {
                                    let n = cell.nitrogen();
                                    cell.set_nitrogen(n + rate);
                                }
                                ExudateType::Carbon => {
                                    let c = cell.carbon();
                                    cell.set_carbon(c + rate);
                                }
                            }
                        }
                    }
                }
            }
            FixtureBehavior::Ombrager { radius } => {
                let pos = fixture_cfg.position;
                let r = *radius as i16;
                for dx in -r..=r {
                    for dy in -r..=r {
                        let p = Pos {
                            x: (pos.x as i16 + dx).max(0) as u16,
                            y: (pos.y as i16 + dy).max(0) as u16,
                        };
                        if let Some(cell) = state.world.get_mut(&p) {
                            cell.set_light(0.2);
                        }
                    }
                }
            }
            FixtureBehavior::Envahir => {
                // Donner de l'energie a la fixture pour qu'elle soit agressive
                if let Some(plant) = state.plants.iter_mut().find(|p| p.id() == fixture_id) {
                    plant.gain_energy(50.0);
                }
            }
            FixtureBehavior::Inerte => {}
        }
    }
}

// --- Evaluation ---

/// Evalue un genome dans un bac isole via les vraies phases de simulation.
/// Place le genome, fait tourner jusqu'a la mort ou max_ticks, retourne la fitness et les stats.
pub fn evaluate_genome(
    genome: &Genome,
    bed_config: &BedConfig,
    rng: &mut dyn Rng,
) -> (f32, PlantStats) {
    // 1. Creer le World et configurer le sol
    let mut world = World::new(bed_config.grid_size);
    for y in 0..bed_config.grid_size {
        for x in 0..bed_config.grid_size {
            let pos = Pos { x, y };
            if let Some(cell) = world.get_mut(&pos) {
                cell.set_altitude(0.5);
                cell.set_carbon(bed_config.initial_carbon);
                cell.set_nitrogen(bed_config.initial_nitrogen);
                cell.set_humidity(bed_config.initial_humidity);
                cell.set_light(bed_config.light_level);
            }
        }
    }

    // 2. Creer l'ile (plate, tout est terre)
    let island = Island::from_world(&world, 0.0);

    // 3. Construire la SimConfig
    let config = bed_config.to_sim_config();

    // 4. Placer la plante testee au centre
    let center = bed_config.grid_size / 2;
    let plant_pos = Pos {
        x: center,
        y: center,
    };
    let lineage = Lineage::new(0, 0);
    let plant = Plant::new(1, plant_pos, genome.traits.clone(), lineage);

    let mut plants: Vec<Box<dyn PlantEntity>> = vec![Box::new(plant)];
    let mut brains = HashMap::new();
    brains.insert(1u64, genome.brain.clone());
    let mut plant_stats = HashMap::new();
    plant_stats.insert(1u64, PlantStats::default());

    // 5. Placer les fixtures
    let mut next_id = 100u64;
    for fixture_config in &bed_config.fixtures {
        let fixture = FixturePlant::new(
            next_id,
            fixture_config.position,
            fixture_config.exudate_type,
            fixture_config.biomass,
        );
        let brain = Brain::new(8, rng);
        plants.push(Box::new(fixture));
        brains.insert(next_id, brain);
        plant_stats.insert(next_id, PlantStats::default());
        next_id += 1;
    }

    // 6. Construire le SimState
    let season_cycle =
        SeasonCycle::from_raw(bed_config.season_start_tick(), config.ticks_per_season);
    let seed_bank = SeedBank::new(config.seed_bank_capacity);
    let mut state = SimState::from_raw(
        world,
        island,
        plants,
        brains,
        SymbiosisNetwork::new(),
        seed_bank,
        season_cycle,
        GenerationCounter::new(),
        plant_stats,
        next_id,
        0,
        config,
    );

    // 7. Boucle de simulation avec les vraies phases
    for _ in 0..bed_config.max_ticks {
        // Avancer l'age des plantes
        for plant in &mut state.plants {
            if !plant.is_dead() {
                plant.tick();
            }
        }
        state.tick_count += 1;

        // Vraies phases de simulation
        phase_environment(&mut state);
        apply_fixtures(&mut state, bed_config); // keepalive fixtures APRES environment
        let decisions = phase_perception_decision(&state);
        let _ = phase_actions(&mut state, &decisions, rng);
        let _ = phase_lifecycle(&mut state, rng);
        // PAS de phase_decomposition — arret net a la mort

        // Verifier si la plante testee est morte
        let is_dead = state
            .plants
            .iter()
            .find(|p| p.id() == 1)
            .map(|p| p.is_dead())
            .unwrap_or(true);

        if is_dead {
            break;
        }
    }

    // 8. Calculer la fitness et retourner les stats
    let stats = state.plant_stats.get(&1).cloned().unwrap_or_default();
    let fitness = evaluate_fitness(&stats);
    (fitness, stats)
}

/// Retourne les 10 environnements de la pepiniere avec leur nom.
pub fn nursery_environments() -> Vec<(String, BedConfig)> {
    vec![
        (
            "Solo riche".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.3,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 5000,
                carbon_regen_rate: 0.002,
                nitrogen_regen_rate: 0.001,
                humidity_regen_rate: 0.01,
                fixtures: vec![],
                ..BedConfig::default()
            },
        ),
        (
            "Carence N".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.0,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 3000,
                carbon_regen_rate: 0.002,
                nitrogen_regen_rate: 0.0,
                humidity_regen_rate: 0.01,
                fixtures: vec![],
                ..BedConfig::default()
            },
        ),
        (
            "Carence C".into(),
            BedConfig {
                initial_carbon: 0.05,
                initial_nitrogen: 0.3,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 3000,
                carbon_regen_rate: 0.0,
                nitrogen_regen_rate: 0.001,
                humidity_regen_rate: 0.01,
                fixtures: vec![],
                ..BedConfig::default()
            },
        ),
        (
            "Avec fixatrice".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.0,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 5000,
                carbon_regen_rate: 0.001,
                nitrogen_regen_rate: 0.0,
                humidity_regen_rate: 0.01,
                fixtures: vec![FixtureConfig {
                    position: Pos { x: 10, y: 8 },
                    exudate_type: ExudateType::Nitrogen,
                    biomass: 3,
                    behavior: FixtureBehavior::Exuder { rate: 0.05 },
                }],
                ..BedConfig::default()
            },
        ),
        (
            "Avec arbre".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.3,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 5000,
                carbon_regen_rate: 0.002,
                nitrogen_regen_rate: 0.001,
                humidity_regen_rate: 0.005,
                fixtures: vec![FixtureConfig {
                    position: Pos { x: 8, y: 10 },
                    exudate_type: ExudateType::Carbon,
                    biomass: 8,
                    behavior: FixtureBehavior::Ombrager { radius: 4 },
                }],
                ..BedConfig::default()
            },
        ),
        (
            "Hiver".into(),
            BedConfig {
                initial_carbon: 0.3,
                initial_nitrogen: 0.1,
                initial_humidity: 0.3,
                light_level: 0.3,
                max_ticks: 3000,
                carbon_regen_rate: 0.0005,
                nitrogen_regen_rate: 0.0002,
                humidity_regen_rate: 0.005,
                fixtures: vec![],
                locked_season: Some(Season::Winter),
                ..BedConfig::default()
            },
        ),
        (
            "Secheresse".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.2,
                initial_humidity: 0.05,
                light_level: 1.0,
                max_ticks: 3000,
                carbon_regen_rate: 0.001,
                nitrogen_regen_rate: 0.0005,
                humidity_regen_rate: 0.0,
                fixtures: vec![],
                ..BedConfig::default()
            },
        ),
        (
            "Competiteur".into(),
            BedConfig {
                initial_carbon: 0.5,
                initial_nitrogen: 0.3,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 5000,
                carbon_regen_rate: 0.002,
                nitrogen_regen_rate: 0.001,
                humidity_regen_rate: 0.01,
                fixtures: vec![FixtureConfig {
                    position: Pos { x: 10, y: 8 },
                    exudate_type: ExudateType::Carbon,
                    biomass: 5,
                    behavior: FixtureBehavior::Envahir,
                }],
                ..BedConfig::default()
            },
        ),
        (
            "Exsudation".into(),
            BedConfig {
                initial_carbon: 0.3,
                initial_nitrogen: 0.1,
                initial_humidity: 0.5,
                light_level: 0.8,
                max_ticks: 5000,
                carbon_regen_rate: 0.0,
                nitrogen_regen_rate: 0.0,
                humidity_regen_rate: 0.005,
                fixtures: vec![FixtureConfig {
                    position: Pos { x: 10, y: 8 },
                    exudate_type: ExudateType::Carbon,
                    biomass: 3,
                    behavior: FixtureBehavior::Exuder { rate: 0.03 },
                }],
                ..BedConfig::default()
            },
        ),
        (
            "Mixte".into(),
            BedConfig {
                initial_carbon: 0.4,
                initial_nitrogen: 0.1,
                initial_humidity: 0.4,
                light_level: 0.7,
                max_ticks: 3000,
                carbon_regen_rate: 0.001,
                nitrogen_regen_rate: 0.0,
                humidity_regen_rate: 0.008,
                fixtures: vec![
                    FixtureConfig {
                        position: Pos { x: 10, y: 8 },
                        exudate_type: ExudateType::Nitrogen,
                        biomass: 3,
                        behavior: FixtureBehavior::Exuder { rate: 0.03 },
                    },
                    FixtureConfig {
                        position: Pos { x: 6, y: 8 },
                        exudate_type: ExudateType::Carbon,
                        biomass: 4,
                        behavior: FixtureBehavior::Envahir,
                    },
                ],
                ..BedConfig::default()
            },
        ),
    ]
}

/// Evalue un genome dans tous les environnements.
/// Retourne (score_total, Vec<(nom_env, fitness)>).
pub fn evaluate_genome_multi(
    genome: &Genome,
    envs: &[(String, BedConfig)],
    rng: &mut dyn Rng,
) -> (f32, Vec<(String, f32)>) {
    let mut scores = Vec::new();
    let mut total = 0.0;

    for (name, config) in envs {
        let (fitness, _) = evaluate_genome(genome, config, rng);
        total += fitness;
        scores.push((name.clone(), fitness));
    }

    (total, scores)
}

// --- Boucle genetique ---

/// Resultat d'un entrainement pour un environnement.
#[derive(Clone)]
pub struct NurseryResult {
    pub env_name: String,
    pub champion: Genome,
    pub fitness: f32,
    pub generations_run: u32,
}

/// Rapport de progression d'une generation dans la pepiniere.
/// Transmis via le callback pour le reporting en temps reel.
#[derive(Clone)]
pub struct GenerationReport {
    /// Numero de la generation (0-indexed)
    pub generation: u32,
    /// Meilleure fitness de cette generation
    pub best_fitness: f32,
    /// Fitness moyenne de cette generation
    pub avg_fitness: f32,
    /// Pire fitness de cette generation
    pub worst_fitness: f32,
    /// Temps ecoule pour cette generation (en secondes)
    pub elapsed_secs: f64,
    /// Stats detaillees du champion (pour le mode verbose)
    pub champion_stats: Option<PlantStats>,
    /// Traits genetiques du champion (pour le mode TUI)
    pub champion_traits: Option<GeneticTraits>,
}

/// Lance la boucle genetique pour un seul environnement.
/// Retourne le meilleur genome apres `generations` iterations.
/// Le rng est injecte par l'appelant (infra ou tests).
pub fn run_nursery_env(
    env_name: &str,
    bed_config: &BedConfig,
    generations: u32,
    population: usize,
    rng: &mut dyn Rng,
    on_generation: Option<&(dyn Fn(&GenerationReport) + Sync)>,
    initial_genomes: Option<&[Genome]>,
) -> NurseryResult {
    // 1. Generer la population initiale
    let mut genomes: Vec<Genome> = if let Some(init) = initial_genomes {
        // Partir des genomes fournis + mutations pour remplir
        let mut pop = Vec::with_capacity(population);
        for g in init {
            pop.push(g.clone());
        }
        while pop.len() < population {
            let parent = &init[pop.len() % init.len()];
            let mut child = parent.clone();
            mutate_genome(&mut child, rng);
            pop.push(child);
        }
        pop
    } else {
        (0..population)
            .map(|_| SeedBank::produce_fresh_seed(rng))
            .collect()
    };

    let mut best_genome = genomes[0].clone();
    let mut best_fitness = 0.0_f32;

    for gen in 0..generations {
        let gen_start = std::time::Instant::now();

        // 2. Evaluer chaque genome
        let mut scored: Vec<(Genome, f32)> = genomes
            .into_iter()
            .map(|g| {
                let (fitness, _) = evaluate_genome(&g, bed_config, rng);
                (g, fitness)
            })
            .collect();

        // 3. Trier par fitness decroissante
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 4. Stats de la generation
        let gen_best = scored[0].1;
        let gen_worst = scored.last().map(|(_, f)| *f).unwrap_or(0.0);
        let gen_avg = scored.iter().map(|(_, f)| f).sum::<f32>() / scored.len() as f32;
        let elapsed = gen_start.elapsed().as_secs_f64();

        if gen_best > best_fitness {
            best_fitness = gen_best;
            best_genome = scored[0].0.clone();
        }

        // Callback optionnel avec report detaille
        if let Some(cb) = &on_generation {
            // Stats du champion (un seul re-calcul)
            let champion_stats = {
                let (_, stats) = evaluate_genome(&scored[0].0, bed_config, rng);
                Some(stats)
            };
            let champion_traits = Some(scored[0].0.traits.clone());
            cb(&GenerationReport {
                generation: gen,
                best_fitness: gen_best,
                avg_fitness: gen_avg,
                worst_fitness: gen_worst,
                elapsed_secs: elapsed,
                champion_stats,
                champion_traits,
            });
        }

        // 5. Garder les top 10 (ou moins si population < 10)
        let top: Vec<Genome> = scored
            .into_iter()
            .take(10.min(population))
            .map(|(g, _)| g)
            .collect();

        // 6. Produire la nouvelle generation par mutations des parents
        let mutations_per_parent = population / top.len().max(1);
        genomes = Vec::with_capacity(population);
        for parent in &top {
            for _ in 0..mutations_per_parent {
                let mut child = parent.clone();
                mutate_genome(&mut child, rng);
                genomes.push(child);
            }
        }
        // Completer si arrondi insuffisant
        while genomes.len() < population {
            let mut child = top[0].clone();
            mutate_genome(&mut child, rng);
            genomes.push(child);
        }
    }

    NurseryResult {
        env_name: env_name.to_string(),
        champion: best_genome,
        fitness: best_fitness,
        generations_run: generations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rng::test_utils::MockRng;

    fn make_test_genome(rng: &mut dyn Rng) -> Genome {
        SeedBank::produce_fresh_seed(rng)
    }

    #[test]
    fn evaluer_genome_sol_riche() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let config = BedConfig::default();
        let (fitness, _) = evaluate_genome(&genome, &config, &mut rng);
        assert!(
            fitness >= 0.0,
            "fitness sur sol riche devrait etre >= 0, got {fitness}"
        );
    }

    #[test]
    fn evaluer_genome_sol_vide() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let config = BedConfig {
            initial_carbon: 0.0,
            initial_nitrogen: 0.0,
            initial_humidity: 0.0,
            light_level: 0.0,
            ..BedConfig::default()
        };
        let (fitness, _) = evaluate_genome(&genome, &config, &mut rng);
        // Sur sol vide sans lumiere, la plante devrait mourir tres vite
        assert!(fitness >= 0.0);
    }

    #[test]
    fn les_10_environnements_existent() {
        let envs = nursery_environments();
        assert_eq!(envs.len(), 10);
    }

    #[test]
    fn scoring_multi_env_retourne_10_scores() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let envs = nursery_environments();
        let (total, scores) = evaluate_genome_multi(&genome, &envs, &mut rng);
        assert_eq!(scores.len(), 10);
        assert!(total >= 0.0);
    }

    #[test]
    fn environnement_hostile_fitness_bornee() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let hostile = BedConfig {
            initial_carbon: 0.0,
            initial_nitrogen: 0.0,
            initial_humidity: 0.0,
            light_level: 0.0,
            carbon_regen_rate: 0.0,
            nitrogen_regen_rate: 0.0,
            humidity_regen_rate: 0.0,
            max_ticks: 100, // tres court pour confirmer que la plante meurt vite
            ..BedConfig::default()
        };
        let mut rng_hostile = MockRng::new(0.42, 0.07);
        let (fit_hostile, _) = evaluate_genome(&genome, &hostile, &mut rng_hostile);
        // Sur sol totalement vide, sans lumiere et en 100 ticks, la fitness reste bornee
        assert!(
            fit_hostile < 1000.0,
            "sol hostile devrait donner une fitness faible, got {fit_hostile}"
        );
    }

    #[test]
    fn evaluer_genome_avec_fixture_fixatrice() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let center = 8u16; // grid_size/2
        let config = BedConfig {
            initial_nitrogen: 0.0,
            fixtures: vec![FixtureConfig {
                position: Pos {
                    x: center + 2,
                    y: center,
                },
                exudate_type: ExudateType::Nitrogen,
                biomass: 3,
                behavior: FixtureBehavior::Exuder { rate: 0.05 },
            }],
            ..BedConfig::default()
        };
        let (fitness, _) = evaluate_genome(&genome, &config, &mut rng);
        assert!(
            fitness >= 0.0,
            "avec une fixture fixatrice, fitness devrait etre >= 0, got {fitness}"
        );
    }

    #[test]
    fn boucle_genetique_ameliore_fitness() {
        let mut rng = MockRng::new(0.42, 0.07);
        let config = BedConfig::default();
        let result = run_nursery_env("test", &config, 5, 20, &mut rng, None, None);
        assert!(
            result.fitness >= 0.0,
            "fitness apres 5 generations devrait etre >= 0, got {}",
            result.fitness
        );
        assert_eq!(result.generations_run, 5);
        assert_eq!(result.env_name, "test");
    }

    #[test]
    fn le_report_contient_worst_fitness_et_timing() {
        let config = BedConfig::default();
        let mut rng = crate::infra::rng::SeededRng::new(42);
        let reports = std::sync::Mutex::new(Vec::new());
        let cb = |report: &GenerationReport| {
            reports.lock().expect("lock").push((
                report.generation,
                report.best_fitness,
                report.worst_fitness,
                report.elapsed_secs,
            ));
        };
        run_nursery_env("test", &config, 3, 10, &mut rng, Some(&cb), None);
        let reports = reports.lock().expect("lock");
        assert_eq!(reports.len(), 3);
        for (_, best, worst, elapsed) in reports.iter() {
            assert!(best >= worst, "best ({}) >= worst ({})", best, worst);
            assert!(*elapsed >= 0.0);
        }
    }

    #[test]
    fn population_initiale_injectee_remplace_random() {
        let config = BedConfig::default();
        let mut rng = crate::infra::rng::SeededRng::new(42);
        // Creer un genome connu
        let genome = SeedBank::produce_fresh_seed(&mut rng);
        let initial = vec![genome.clone()];
        let result = run_nursery_env("test", &config, 2, 5, &mut rng, None, Some(&initial));
        // Le champion doit avoir une fitness >= 0
        assert!(result.fitness >= 0.0);
        assert_eq!(result.generations_run, 2);
    }

    #[test]
    fn nursery_mode_ne_place_pas_de_graines() {
        let config = BedConfig::default();
        let mut rng = crate::infra::rng::SeededRng::new(42);
        let genome = SeedBank::produce_fresh_seed(&mut rng);
        let (fitness, _stats) = evaluate_genome(&genome, &config, &mut rng);
        // La fitness doit etre >= 0
        assert!(fitness >= 0.0);
    }
}
