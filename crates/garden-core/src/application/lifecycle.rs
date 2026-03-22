// Phase vie et mort — reproduction, mortalite, pluie de graines, germination et GC.

use crate::application::evolution::{
    evaluate_fitness, mutate_genome, Genome, PlantStats, SeedBank,
};
use crate::domain::events::DomainEvent;
use crate::domain::plant::{Lineage, Plant, PlantState, Pos};
use crate::domain::rng::Rng;
use crate::domain::world::GRID_SIZE;

use super::sim::SimState;

/// Phase 4 : reproduction, verification des morts, pluie de graines, dormance/germination et GC.
pub fn phase_lifecycle(state: &mut SimState, rng: &mut dyn Rng) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    // a) Production continue de graines — plantes matures avec assez d'energie
    let mut new_seeds: Vec<(Plant, crate::domain::brain::Brain)> = Vec::new();

    for i in 0..state.plants.len() {
        let plant = &state.plants[i];
        if plant.is_dead() || plant.state() == PlantState::Seed {
            continue;
        }

        // Pas encore mature → pas de graines
        if plant.state() != PlantState::Mature {
            continue;
        }

        // Pas assez d'energie
        if plant.energy().value() < state.config.seed_energy_threshold {
            continue;
        }

        // Accumuler seed_progress
        let rate = plant.biomass().value() as f32 * state.config.seed_production_rate;
        state.plants[i].add_seed_progress(rate);

        // Si une graine est prete
        while state.plants[i].seed_progress() >= 1.0 {
            state.plants[i].consume_seed_progress(1.0);

            // Cout en energie
            state.plants[i].consume_energy(state.config.seed_energy_cost);
            if state.plants[i].energy().value() < state.config.seed_energy_threshold {
                break;
            }

            // Dispersion gradient
            let base_pos = state.plants[i].footprint()[0];
            let target = disperse_seed(&base_pos, rng);

            // Verifier cellule valide
            if !state.island.is_land(&target) {
                continue;
            }

            // Clone (10%) ou mute (90%)
            let parent_id = state.plants[i].id();
            let parent_brain = state.brains.get(&parent_id).cloned();
            let parent_brain = match parent_brain {
                Some(b) => b,
                None => continue,
            };

            let mut genome = Genome {
                brain: parent_brain,
                traits: state.plants[i].genetics().clone(),
            };

            if rng.next_f32() > 0.1 {
                // 90% : mute
                mutate_genome(&mut genome, rng);
            }
            // 10% : clone exact (pas de mutation)

            let gen = state.generation_counter.next();
            let parent_lineage_id = state.plants[i].lineage().id();
            let lineage = Lineage::new(parent_lineage_id, gen);
            let child_id = state.next_plant_id;
            state.next_plant_id += 1;

            let child = Plant::with_parent(
                child_id,
                target,
                genome.traits,
                lineage.clone(),
                parent_id,
                state.plants[i].ancestors(),
            );

            new_seeds.push((child, genome.brain));

            events.push(DomainEvent::Born {
                plant_id: child_id,
                parent_id: Some(parent_id),
                position: target,
                lineage,
            });

            // Stats
            if let Some(stats) = state.find_stats_mut(parent_id) {
                stats.seeds_produced += 1;
            }
        }
    }

    // Ajouter les nouvelles plantes avec fitness heritee du parent
    for (child, brain) in new_seeds {
        let child_id = child.id();
        let parent_id = child.parent_id();

        // Calculer la fitness estimee du parent (sur ses stats accumulees)
        let parent_fitness_estimate = parent_id
            .and_then(|pid| state.plant_stats.get(&pid))
            .map(evaluate_fitness)
            .unwrap_or(0.0);

        // Creer les stats du fils avec fitness heritee (30% du parent)
        let child_stats = PlantStats {
            inherited_fitness: parent_fitness_estimate * 0.3,
            ..PlantStats::default()
        };

        state.plants.push(child);
        state.brains.insert(child_id, brain);
        state.plant_stats.insert(child_id, child_stats);
    }

    // b) Verifier les morts
    let mut dead_ids = Vec::new();
    for plant in &mut state.plants {
        if plant.is_dead() {
            continue;
        }
        if let Some(event) = plant.update_state() {
            if plant.state() == PlantState::Dead {
                dead_ids.push(plant.id());
            }
            events.push(event);
        }
    }

    // Pour chaque plante morte : demarrer la decomposition, fitness, retirer les liens
    for dead_id in &dead_ids {
        // Trouver la plante morte
        let plant_data = state
            .plants
            .iter()
            .find(|p| p.id() == *dead_id)
            .map(|p| p.genetics().clone());

        if let Some(genetics) = plant_data {
            // Demarrer la decomposition progressive au lieu d'enrichir immediatement
            if let Some(plant) = state.plants.iter_mut().find(|p| p.id() == *dead_id) {
                plant.start_decomposition(state.config.decomposition_ticks);
            }

            // Evaluer la fitness et tenter d'inserer dans la banque
            if let Some(stats) = state.plant_stats.get(dead_id).cloned() {
                let fitness = evaluate_fitness(&stats);
                // Reconstruire le genome
                if let Some(brain) = state.brains.get(dead_id).cloned() {
                    let genome = Genome {
                        brain,
                        traits: genetics,
                    };
                    state.seed_bank.try_insert(genome, fitness);
                }
            }

            // Retirer les liens symbiotiques
            let removed_links = state.symbiosis.remove_plant(*dead_id);
            for link in &removed_links {
                events.push(DomainEvent::Unlinked {
                    plant_a: link.plant_a(),
                    plant_b: link.plant_b(),
                });
            }
        }
    }

    // c) Pluie de graines (tous les seed_rain_interval ticks)
    // Seulement si le nombre de plantes germees est sous le seuil (graines exclues)
    let germinated_count = state
        .plants
        .iter()
        .filter(|p| !p.is_dead() && p.state() != PlantState::Seed)
        .count();
    // Filet de securite : pluie de graines seulement si tres peu de plantes germees
    if state.tick_count > 0
        && germinated_count < 10
        && state
            .tick_count
            .is_multiple_of(state.config.seed_rain_interval)
    {
        // 10% du temps : graine fraiche (genome aleatoire), sinon depuis la banque
        let genome = if rng.next_f32() < 0.1 || state.seed_bank.is_empty() {
            SeedBank::produce_fresh_seed(rng)
        } else {
            state.seed_bank.produce_seed(rng)
        };

        let land_cells = state.island.land_cells();
        if !land_cells.is_empty() {
            // 80% : placer pres d'une plante existante, 20% : position aleatoire
            let pos = if rng.next_f32() < 0.8 && !state.plants.is_empty() {
                let alive: Vec<&Plant> = state.plants.iter().filter(|p| !p.is_dead()).collect();
                if alive.is_empty() {
                    random_land_pos(land_cells, rng)
                } else {
                    let parent_idx = (rng.next_f32() * alive.len() as f32) as usize;
                    let parent = alive[parent_idx.min(alive.len() - 1)];
                    let base_pos = parent.footprint()[0];
                    // Position a 3-8 cellules de distance
                    let angle = rng.next_f32() * 2.0 * core::f32::consts::PI;
                    let distance = 3.0 + rng.next_f32() * 5.0;
                    let tx = (base_pos.x as f32 + angle.cos() * distance).round();
                    let ty = (base_pos.y as f32 + angle.sin() * distance).round();
                    if tx >= 0.0 && tx < GRID_SIZE as f32 && ty >= 0.0 && ty < GRID_SIZE as f32 {
                        let target = Pos {
                            x: tx as u16,
                            y: ty as u16,
                        };
                        if state.island.is_land(&target) {
                            target
                        } else {
                            random_land_pos(land_cells, rng)
                        }
                    } else {
                        random_land_pos(land_cells, rng)
                    }
                }
            } else {
                random_land_pos(land_cells, rng)
            };

            let occupied = state
                .plants
                .iter()
                .any(|p| !p.is_dead() && p.footprint().contains(&pos));

            if !occupied {
                let child_id = state.next_plant_id;
                state.next_plant_id += 1;
                let gen = state.generation_counter.next();
                let lineage = Lineage::new(child_id, gen);

                let child = Plant::new(child_id, pos, genome.traits, lineage.clone());
                state.plants.push(child);
                state.brains.insert(child_id, genome.brain);
                state.plant_stats.insert(child_id, PlantStats::default());

                events.push(DomainEvent::Born {
                    plant_id: child_id,
                    parent_id: None,
                    position: pos,
                    lineage,
                });
            }
        }
    }

    // d) Dormance/germination des graines
    for plant in &mut state.plants {
        if plant.state() != PlantState::Seed {
            continue;
        }

        // Timeout de dormance
        if plant.age() > state.config.dormancy_timeout {
            plant.damage(plant.vitality().value());
            if let Some(event) = plant.update_state() {
                events.push(event);
            }
            continue;
        }

        // Germination si sol assez riche
        let pos = plant.footprint()[0];
        let can_germinate = state
            .world
            .get(&pos)
            .map(|cell| {
                cell.carbon() > state.config.germination_carbon_min
                    && cell.nitrogen() > state.config.germination_nitrogen_min
            })
            .unwrap_or(false);

        if can_germinate {
            if let Some(event) = plant.germinate() {
                events.push(event);
            }
        }
    }

    // e) GC periodique : retirer les plantes entierement decomposees (tous les 100 ticks)
    if state.tick_count > 0 && state.tick_count.is_multiple_of(100) {
        let fully_decomposed_ids: Vec<u64> = state
            .plants
            .iter()
            .filter(|p| p.is_fully_decomposed())
            .map(|p| p.id())
            .collect();

        for id in &fully_decomposed_ids {
            state.plants.retain(|p| p.id() != *id);
            state.brains.remove(id);
            state.plant_stats.remove(id);
        }
    }

    events
}

/// Disperse une graine avec un gradient de distance :
/// 70% proche (1-3 cellules), 20% moyen (3-6), 10% loin (6-15).
fn disperse_seed(base: &Pos, rng: &mut dyn Rng) -> Pos {
    let angle = rng.next_f32() * 2.0 * core::f32::consts::PI;
    let roll = rng.next_f32();

    let distance = if roll < 0.7 {
        // 70% : proche (1-3 cellules)
        1.0 + rng.next_f32() * 2.0
    } else if roll < 0.9 {
        // 20% : moyen (3-6 cellules)
        3.0 + rng.next_f32() * 3.0
    } else {
        // 10% : loin (6-15 cellules)
        6.0 + rng.next_f32() * 9.0
    };

    let tx = (base.x as f32 + angle.cos() * distance).round();
    let ty = (base.y as f32 + angle.sin() * distance).round();

    Pos {
        x: tx.clamp(0.0, (GRID_SIZE - 1) as f32) as u16,
        y: ty.clamp(0.0, (GRID_SIZE - 1) as f32) as u16,
    }
}

/// Choisit une position aleatoire parmi les cellules terrestres.
fn random_land_pos(land_cells: &[Pos], rng: &mut dyn Rng) -> Pos {
    let idx = (rng.next_f32() * land_cells.len() as f32) as usize;
    let idx = idx.min(land_cells.len() - 1);
    land_cells[idx]
}
