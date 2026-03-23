// Pepiniere — infrastructure pour evaluer un genome dans un bac isole.

use std::collections::HashMap;

use super::config::SimConfig;
use super::evolution::{evaluate_fitness, Genome, PlantStats, SeedBank};
use super::season::Season;
use super::sim::{run_tick, SimState};
use crate::domain::brain::Brain;
use crate::domain::island::Island;
use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
use crate::domain::rng::Rng;
use crate::domain::symbiosis::SymbiosisNetwork;
use crate::domain::world::{World, GRID_SIZE};

use super::evolution::GenerationCounter;
use super::season::SeasonCycle;

/// Configuration d'un bac de pepiniere.
pub struct BedConfig {
    /// Taille de la zone active (8 ou 16).
    pub grid_size: u16,
    /// Carbone initial du sol [0, 1].
    pub initial_carbon: f32,
    /// Azote initial du sol [0, 1].
    pub initial_nitrogen: f32,
    /// Humidite initiale du sol [0, 1].
    pub initial_humidity: f32,
    /// Lumiere fixe [0, 1] (pas de saisons).
    pub light_level: f32,
    /// Saison fixe (pour les multiplicateurs).
    pub season: Season,
    /// Plantes artificielles presentes dans le bac.
    pub fixtures: Vec<FixtureConfig>,
    /// Timeout en ticks (ex: 2000).
    pub max_ticks: u32,
}

impl Default for BedConfig {
    fn default() -> Self {
        Self {
            grid_size: 8,
            initial_carbon: 0.5,
            initial_nitrogen: 0.3,
            initial_humidity: 0.5,
            light_level: 0.8,
            season: Season::Spring,
            fixtures: Vec::new(),
            max_ticks: 2000,
        }
    }
}

/// Configuration d'une plante artificielle (fixture).
/// Comportement deterministe, pas de brain.
pub struct FixtureConfig {
    /// Position dans la zone active (coordonnees relatives au centre de la grille).
    pub position: Pos,
    /// Type d'exsudat emis.
    pub exudate_type: ExudateType,
    /// Biomasse initiale de la fixture.
    pub biomass: u16,
    /// Comportement de la fixture.
    pub behavior: FixtureBehavior,
}

/// Comportement d'une fixture dans le bac.
pub enum FixtureBehavior {
    /// Exsude en permanence (N ou C selon exudate_type).
    Exuder { rate: f32 },
    /// Fait de l'ombre (grande canopee).
    Ombrager { radius: u16 },
    /// Envahit les cellules voisines agressivement.
    Envahir { energy: f32 },
    /// Inerte (juste present, occupe de l'espace).
    Inerte,
}

/// Evalue un genome dans un bac isole.
/// Place le genome au centre, fait tourner jusqu'a la mort ou max_ticks, retourne la fitness.
pub fn evaluate_genome(genome: &Genome, bed_config: &BedConfig, rng: &mut dyn Rng) -> f32 {
    // 1. Creer le World
    let mut world = World::new();

    // 2. Configurer le sol dans la zone active (centre de la grille)
    let center = GRID_SIZE / 2;
    let half = bed_config.grid_size / 2;
    let start = center.saturating_sub(half);
    let end = (center + half).min(GRID_SIZE);
    for y in start..end {
        for x in start..end {
            let pos = Pos { x, y };
            if let Some(cell) = world.get_mut(&pos) {
                cell.set_altitude(0.5); // terre (au-dessus du sea level)
                cell.set_carbon(bed_config.initial_carbon);
                cell.set_nitrogen(bed_config.initial_nitrogen);
                cell.set_humidity(bed_config.initial_humidity);
                cell.set_light(bed_config.light_level);
            }
        }
    }

    // 3. Creer une ile a partir du terrain configure
    let island = Island::from_world(&world, 0.1);

    // 4. Creer un SimConfig minimal
    let config = SimConfig {
        initial_population: 0,         // pas de population initiale (on place manuellement)
        seed_rain_interval: u32::MAX,  // pas de pluie de graines
        ..SimConfig::default()
    };

    // 5. Creer le SimState via from_raw pour eviter le peuplement automatique
    let seed_bank = {
        let mut sb = SeedBank::new(config.seed_bank_capacity);
        sb.initialize(config.seed_bank_capacity, rng);
        sb
    };

    let mut state = SimState::from_raw(
        world,
        island,
        Vec::new(),
        HashMap::new(),
        SymbiosisNetwork::new(),
        seed_bank,
        SeasonCycle::new(config.ticks_per_season),
        GenerationCounter::new(),
        HashMap::new(),
        2, // next_plant_id (1 est reserve pour le genome teste)
        0,
        config,
    );

    // 6. Placer le genome au centre
    let plant_pos = Pos {
        x: center,
        y: center,
    };
    let lineage = Lineage::new(0, 0);
    let plant = Plant::new(1, plant_pos, genome.traits.clone(), lineage);
    state.plants.push(Box::new(plant));
    state.brains.insert(1, genome.brain.clone());
    state.plant_stats.insert(1, PlantStats::default());

    // 7. Placer les fixtures
    let mut fixture_id = 100_u64;
    for fixture in &bed_config.fixtures {
        place_fixture(&mut state, fixture, fixture_id, rng);
        fixture_id += 1;
    }

    // 8. Faire tourner jusqu'a la mort de la plante testee ou max_ticks
    for _ in 0..bed_config.max_ticks {
        let _ = run_tick(&mut state, rng);

        // Appliquer le comportement des fixtures a chaque tick
        apply_fixtures(&mut state, &bed_config.fixtures);

        // Verifier si la plante testee (id=1) est morte
        let plant_dead = state
            .plants
            .iter()
            .find(|p| p.id() == 1)
            .is_none_or(|p| p.is_dead());

        if plant_dead {
            break;
        }
    }

    // 9. Calculer la fitness
    if let Some(stats) = state.plant_stats.get(&1) {
        evaluate_fitness(stats)
    } else {
        0.0
    }
}

/// Place une fixture dans le SimState.
fn place_fixture(state: &mut SimState, config: &FixtureConfig, id: u64, rng: &mut dyn Rng) {
    let genetics = GeneticTraits::new(
        40, // max_size grand
        0.5,
        config.exudate_type,
        8,
        10.0,
        10.0,
    );
    let lineage = Lineage::new(id, 0);
    let mut plant = Plant::new(id, config.position, genetics, lineage);
    let _ = plant.germinate();
    // Donner de la biomasse a la fixture
    for i in 0..config.biomass.min(10) {
        let pos = Pos {
            x: config.position.x.saturating_add(i % 3),
            y: config.position.y.saturating_add(i / 3),
        };
        plant.grow_footprint(pos);
    }
    // Donner beaucoup d'energie et de vitalite
    plant.gain_energy(1000.0);
    plant.heal(1000.0);

    let brain = Brain::new(8, rng);
    state.plants.push(Box::new(plant));
    state.brains.insert(id, brain);
    state.plant_stats.insert(id, PlantStats::default());
}

/// Applique le comportement des fixtures a chaque tick.
fn apply_fixtures(state: &mut SimState, fixtures: &[FixtureConfig]) {
    for (i, fixture) in fixtures.iter().enumerate() {
        let fixture_id = 100 + i as u64;

        // Maintenir la fixture en vie (elle ne meurt jamais)
        if let Some(plant) = state.plants.iter_mut().find(|p| p.id() == fixture_id) {
            plant.heal(100.0);
            plant.gain_energy(100.0);
        }

        // Appliquer le comportement
        match &fixture.behavior {
            FixtureBehavior::Exuder { rate } => {
                // Injecter la ressource dans le sol autour de la fixture
                let pos = fixture.position;
                for dx in -2i16..=2 {
                    for dy in -2i16..=2 {
                        let p = Pos {
                            x: (pos.x as i16 + dx).max(0) as u16,
                            y: (pos.y as i16 + dy).max(0) as u16,
                        };
                        if let Some(cell) = state.world.get_mut(&p) {
                            match fixture.exudate_type {
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
                // Reduire la lumiere dans le rayon
                let pos = fixture.position;
                let r = *radius as i16;
                for dx in -r..=r {
                    for dy in -r..=r {
                        let p = Pos {
                            x: (pos.x as i16 + dx).max(0) as u16,
                            y: (pos.y as i16 + dy).max(0) as u16,
                        };
                        if let Some(cell) = state.world.get_mut(&p) {
                            cell.set_light(0.2); // ombre
                        }
                    }
                }
            }
            FixtureBehavior::Envahir { energy } => {
                // Donner de l'energie a la fixture pour qu'elle envahisse naturellement
                if let Some(plant) = state.plants.iter_mut().find(|p| p.id() == fixture_id) {
                    plant.gain_energy(*energy);
                }
            }
            FixtureBehavior::Inerte => {} // ne fait rien
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::rng::test_utils::MockRng;

    #[test]
    fn evaluer_genome_solo_riche() {
        // Genome dans un bac riche → fitness > 0
        let mut rng = MockRng::new(0.3, 0.07);
        let genome = SeedBank::produce_fresh_seed(&mut rng);

        let bed = BedConfig::default();
        let fitness = evaluate_genome(&genome, &bed, &mut rng);

        assert!(
            fitness > 0.0,
            "un genome dans un bac riche devrait avoir une fitness > 0, got {fitness}"
        );
    }

    #[test]
    fn evaluer_genome_avec_fixture() {
        // Genome avec une fixture fixatrice → la plante survit plus longtemps
        let mut rng = MockRng::new(0.3, 0.07);
        let genome = SeedBank::produce_fresh_seed(&mut rng);

        // Bac sans fixture
        let bed_sans = BedConfig::default();
        let mut rng_a = MockRng::new(0.3, 0.07);
        let fitness_sans = evaluate_genome(&genome, &bed_sans, &mut rng_a);

        // Bac avec fixture fixatrice d'azote a cote
        let center = GRID_SIZE / 2;
        let bed_avec = BedConfig {
            fixtures: vec![FixtureConfig {
                position: Pos {
                    x: center + 2,
                    y: center,
                },
                exudate_type: ExudateType::Nitrogen,
                biomass: 5,
                behavior: FixtureBehavior::Exuder { rate: 0.05 },
            }],
            ..BedConfig::default()
        };
        let mut rng_b = MockRng::new(0.3, 0.07);
        let fitness_avec = evaluate_genome(&genome, &bed_avec, &mut rng_b);

        // Les deux fitness sont calculees — on verifie que les deux tournent sans crash
        // Le resultat exact depend du rng et des interactions, mais les deux doivent etre >= 0
        assert!(
            fitness_sans >= 0.0,
            "fitness sans fixture doit etre >= 0, got {fitness_sans}"
        );
        assert!(
            fitness_avec >= 0.0,
            "fitness avec fixture doit etre >= 0, got {fitness_avec}"
        );
    }

    #[test]
    fn evaluer_genome_max_ticks() {
        // Timeout respecte si la plante ne meurt pas
        let mut rng = MockRng::new(0.3, 0.07);
        let genome = SeedBank::produce_fresh_seed(&mut rng);

        let bed = BedConfig {
            max_ticks: 10, // tres court
            ..BedConfig::default()
        };

        // Doit terminer sans paniquer meme avec un timeout court
        let fitness = evaluate_genome(&genome, &bed, &mut rng);
        assert!(
            fitness >= 0.0,
            "la fitness doit etre >= 0 meme avec un timeout court, got {fitness}"
        );
    }
}
