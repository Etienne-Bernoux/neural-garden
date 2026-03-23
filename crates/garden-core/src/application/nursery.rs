// Pepiniere — bac isole pour evaluer un genome sans SimState.
// Boucle simplifiee : pas de saisons, pas de reproduction, pas de highlights.

use std::collections::HashMap;

use crate::domain::brain::Brain;
use crate::domain::island::Island;
use crate::domain::plant::{ExudateType, Lineage, Plant, Pos};
use crate::domain::rng::Rng;
use crate::domain::traits::PlantEntity;
use crate::domain::world::World;

use crate::application::evolution::{evaluate_fitness, Genome, PlantStats};
use crate::application::perception::compute_inputs;
use crate::domain::fixture::FixturePlant;

// --- Configuration ---

/// Configuration d'un bac de pepiniere.
pub struct BedConfig {
    pub grid_size: u16,
    pub initial_carbon: f32,
    pub initial_nitrogen: f32,
    pub initial_humidity: f32,
    pub light_level: f32,
    pub max_ticks: u32,
    pub fixtures: Vec<FixtureConfig>,
}

/// Configuration d'une fixture dans un bac.
pub struct FixtureConfig {
    pub position: Pos,
    pub exudate_type: ExudateType,
    pub biomass: u16,
    pub behavior: FixtureBehavior,
}

/// Comportement deterministe d'une fixture.
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
        }
    }
}

// --- Etat du bac ---

/// Etat d'un bac de pepiniere — structure allegee sans SimState.
struct BedState {
    world: World,
    #[allow(dead_code)]
    island: Island,
    plants: Vec<Box<dyn PlantEntity>>,
    brains: HashMap<u64, Brain>,
    plant_stats: HashMap<u64, PlantStats>,
    tick_count: u32,
}

// --- Runner ---

/// Boucle minimale de la pepiniere.
/// 1. Fixtures appliquent leur comportement
/// 2. Perception (18 inputs)
/// 3. Decision (forward pass)
/// 4. Actions simplifiees (absorption, photosynthese, fixation N)
/// 5. Maintenance (vieillissement, famine, cout, update_state)
fn run_bed_tick(bed: &mut BedState, config: &BedConfig) {
    bed.tick_count += 1;

    // Avancer l'age des plantes vivantes
    for plant in &mut bed.plants {
        if !plant.is_dead() {
            plant.tick();
        }
    }

    // 1. Fixtures appliquent leur comportement
    apply_fixtures(bed, config);

    // 2+3. Perception + decision pour la plante testee (id=1)
    let decisions = compute_decisions(bed);

    // 4. Actions simplifiees
    apply_actions(bed, &decisions, config);

    // 5. Maintenance (vieillissement, famine)
    apply_maintenance(bed);
}

/// Applique le comportement des fixtures a chaque tick.
fn apply_fixtures(bed: &mut BedState, config: &BedConfig) {
    for (i, fixture_cfg) in config.fixtures.iter().enumerate() {
        let fixture_id = 100 + i as u64;

        // Maintenir la fixture en vie (immortelle)
        if let Some(plant) = bed.plants.iter_mut().find(|p| p.id() == fixture_id) {
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
                        if let Some(cell) = bed.world.get_mut(&p) {
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
                        if let Some(cell) = bed.world.get_mut(&p) {
                            cell.set_light(0.2);
                        }
                    }
                }
            }
            FixtureBehavior::Envahir => {
                // Donner de l'energie a la fixture pour qu'elle soit agressive
                if let Some(plant) = bed.plants.iter_mut().find(|p| p.id() == fixture_id) {
                    plant.gain_energy(50.0);
                }
            }
            FixtureBehavior::Inerte => {}
        }
    }
}

/// Perception + forward pass pour toutes les plantes vivantes.
/// Retourne (plant_id, outputs) pour chaque plante non-morte.
fn compute_decisions(bed: &BedState) -> Vec<(u64, [f32; 8])> {
    let mut decisions = Vec::new();
    for plant in &bed.plants {
        if plant.is_dead() {
            continue;
        }
        let id = plant.id();
        let inputs = compute_inputs(plant.as_ref(), &bed.world);
        if let Some(brain) = bed.brains.get(&id) {
            let outputs = brain.forward(&inputs);
            decisions.push((id, outputs));
        }
    }
    decisions
}

/// Actions simplifiees : absorption C/N/H + photosynthese + fixation N.
/// Pas de croissance spatiale ni de symbiose dans cette version.
fn apply_actions(bed: &mut BedState, decisions: &[(u64, [f32; 8])], config: &BedConfig) {
    for &(plant_id, _outputs) in decisions {
        let plant_idx = match bed.plants.iter().position(|p| p.id() == plant_id) {
            Some(idx) => idx,
            None => continue,
        };

        if bed.plants[plant_idx].is_dead() {
            continue;
        }

        // Germination automatique des graines
        if bed.plants[plant_idx].state() == crate::domain::plant::PlantState::Seed {
            bed.plants[plant_idx].germinate();
            continue;
        }

        // Absorption : absorber C/N/H du sol sous les racines
        let roots: Vec<Pos> = bed.plants[plant_idx].roots().to_vec();
        let mut total_absorbed = 0.0_f32;
        for root_pos in &roots {
            if let Some(cell) = bed.world.get_mut(root_pos) {
                // Absorber un peu de chaque ressource
                let c = cell.carbon();
                let absorbed_c = (c * 0.05).min(0.1);
                cell.set_carbon(c - absorbed_c);

                let n = cell.nitrogen();
                let absorbed_n = (n * 0.05).min(0.1);
                cell.set_nitrogen(n - absorbed_n);

                let h = cell.humidity();
                let absorbed_h = (h * 0.05).min(0.1);
                cell.set_humidity(h - absorbed_h);

                total_absorbed += absorbed_c + absorbed_n + absorbed_h;
            }
        }
        // Convertir les ressources absorbees en energie
        bed.plants[plant_idx].gain_energy(total_absorbed * 5.0);

        // Photosynthese : gain energie proportionnel a la lumiere sur la canopee
        let canopy: Vec<Pos> = bed.plants[plant_idx].canopy().to_vec();
        let mut light_sum = 0.0_f32;
        for canopy_pos in &canopy {
            if let Some(cell) = bed.world.get(canopy_pos) {
                light_sum += cell.light();
            }
        }
        let photo_gain = light_sum * config.light_level * 0.3;
        bed.plants[plant_idx].gain_energy(photo_gain);

        // Fixation N : les plantes de type Nitrogen fixent de l'azote dans le sol
        let exudate_type = bed.plants[plant_idx].genetics().exudate_type();
        if exudate_type == ExudateType::Nitrogen {
            for root_pos in &roots {
                if let Some(cell) = bed.world.get_mut(root_pos) {
                    let n = cell.nitrogen();
                    cell.set_nitrogen(n + 0.01);
                }
            }
            // Tracker les exsudats emis
            if let Some(stats) = bed.plant_stats.get_mut(&plant_id) {
                stats.exudates_emitted += 0.01 * roots.len() as f32;
            }
        }

        // Tracker max_biomass et lifetime
        if let Some(stats) = bed.plant_stats.get_mut(&plant_id) {
            let biomass = bed.plants[plant_idx].biomass().value();
            if biomass > stats.max_biomass {
                stats.max_biomass = biomass;
            }
            stats.lifetime = bed.plants[plant_idx].age();
        }
    }
}

/// Maintenance : vieillissement, famine, cout proportionnel a la biomasse, update_state.
fn apply_maintenance(bed: &mut BedState) {
    for plant in &mut bed.plants {
        if plant.is_dead() {
            continue;
        }

        let age = plant.age();
        let biomass = plant.biomass().value();

        // Vieillissement : degats proportionnels a l'age (tres leger)
        let age_damage = (age as f32 / 5000.0).min(0.5);
        plant.damage(age_damage);

        // Cout de maintenance : proportionnel a la biomasse
        let maintenance_cost = biomass as f32 * 0.02;
        plant.consume_energy(maintenance_cost);

        // Famine : si energie a zero, drain de vitalite
        if plant.energy().value() <= 0.0 {
            plant.damage(0.5);
        }

        // Update state (check mort, transitions)
        plant.update_state();
    }
}

// --- Evaluation ---

/// Evalue un genome dans un bac isole.
/// Place le genome, fait tourner jusqu'a la mort ou max_ticks, retourne la fitness.
pub fn evaluate_genome(genome: &Genome, bed_config: &BedConfig, rng: &mut dyn Rng) -> f32 {
    // 1. Creer le World
    let mut world = World::new(bed_config.grid_size);

    // 2. Configurer le sol (toutes les cellules)
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

    // 3. Creer l'ile (tout est terre)
    let island = Island::from_world(&world, 0.1);

    // 4. Placer le genome au centre
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

    // 6. Creer le BedState
    let mut bed = BedState {
        world,
        island,
        plants,
        brains,
        plant_stats,
        tick_count: 0,
    };

    // 7. Faire tourner
    for _ in 0..bed_config.max_ticks {
        run_bed_tick(&mut bed, bed_config);

        // Verifier si la plante testee (id=1) est morte
        let is_dead = bed
            .plants
            .iter()
            .find(|p| p.id() == 1)
            .map(|p| p.is_dead())
            .unwrap_or(true);

        if is_dead {
            break;
        }
    }

    // 8. Calculer la fitness
    bed.plant_stats.get(&1).map(evaluate_fitness).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::evolution::SeedBank;
    use crate::domain::rng::test_utils::MockRng;

    fn make_test_genome(rng: &mut dyn Rng) -> Genome {
        SeedBank::produce_fresh_seed(rng)
    }

    #[test]
    fn evaluer_genome_sol_riche() {
        let mut rng = MockRng::new(0.42, 0.07);
        let genome = make_test_genome(&mut rng);
        let config = BedConfig::default();
        let fitness = evaluate_genome(&genome, &config, &mut rng);
        assert!(
            fitness > 0.0,
            "fitness sur sol riche devrait etre > 0, got {fitness}"
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
        let fitness = evaluate_genome(&genome, &config, &mut rng);
        // Sur sol vide sans lumiere, la plante devrait mourir tres vite
        // Fitness peut etre > 0 car lifetime compte (meme petit)
        assert!(fitness >= 0.0);
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
        let fitness = evaluate_genome(&genome, &config, &mut rng);
        assert!(
            fitness > 0.0,
            "avec une fixture fixatrice, la plante devrait survivre, got {fitness}"
        );
    }
}
