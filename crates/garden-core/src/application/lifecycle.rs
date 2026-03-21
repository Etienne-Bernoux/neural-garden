// Phase vie et mort — reproduction, mortalite, pluie de graines, germination et GC.

use crate::application::evolution::{evaluate_fitness, mutate_genome, Genome, PlantStats};
use crate::domain::events::DomainEvent;
use crate::domain::plant::{Lineage, Plant, PlantState, Pos};
use crate::domain::rng::Rng;
use crate::domain::world::GRID_SIZE;

use super::sim::SimState;

/// Phase 4 : reproduction, verification des morts, pluie de graines, dormance/germination et GC.
pub fn phase_lifecycle(state: &mut SimState, rng: &mut dyn Rng) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    // a) Reproduction : plantes avec assez d'energie et de biomasse
    // Collecter d'abord les candidats a la reproduction pour eviter les conflits de borrow
    let mut reproduction_candidates: Vec<(usize, Pos)> = Vec::new();
    for i in 0..state.plants.len() {
        let plant = &state.plants[i];
        if plant.is_dead() || plant.state() == PlantState::Seed {
            continue;
        }
        if plant.energy().value() > state.config.reproduction_energy_min
            && plant.biomass().value() > state.config.reproduction_biomass_min
        {
            let base_pos = plant.canopy()[0];
            reproduction_candidates.push((i, base_pos));
        }
    }

    let mut new_plants: Vec<(Plant, crate::domain::brain::Brain)> = Vec::new();
    for (plant_idx, base_pos) in &reproduction_candidates {
        // Direction aleatoire, distance 3-9
        let angle = rng.next_f32() * 2.0 * core::f32::consts::PI;
        let min_dist = state.config.reproduction_min_distance as f32;
        let max_dist = state.config.reproduction_max_distance as f32;
        let distance = min_dist + rng.next_f32() * (max_dist - min_dist);
        let tx = (base_pos.x as f32 + angle.cos() * distance).round();
        let ty = (base_pos.y as f32 + angle.sin() * distance).round();

        if tx < 0.0 || tx >= GRID_SIZE as f32 || ty < 0.0 || ty >= GRID_SIZE as f32 {
            continue;
        }
        let target = Pos {
            x: tx as u16,
            y: ty as u16,
        };

        if !state.island.is_land(&target) {
            continue;
        }

        let occupied = state
            .plants
            .iter()
            .any(|p| !p.is_dead() && p.canopy().contains(&target));
        let occupied_new = new_plants.iter().any(|(p, _)| p.canopy().contains(&target));

        if occupied || occupied_new {
            continue;
        }

        // Reproduction vivante : cloner le genome du parent et appliquer des mutations
        let parent_id = state.plants[*plant_idx].id();
        let parent_brain = state.brains.get(&parent_id).cloned();
        let parent_brain = match parent_brain {
            Some(b) => b,
            None => continue,
        };
        let mut genome = Genome {
            brain: parent_brain,
            traits: state.plants[*plant_idx].genetics().clone(),
        };
        mutate_genome(&mut genome, rng);

        let gen = state.generation_counter.next();
        let parent_lineage_id = state.plants[*plant_idx].lineage().id();
        let lineage = Lineage::new(parent_lineage_id, gen);
        let child_id = state.next_plant_id;
        state.next_plant_id += 1;

        let child = Plant::new(child_id, target, genome.traits, lineage.clone());
        new_plants.push((child, genome.brain));

        state.plants[*plant_idx].consume_energy(state.config.reproduction_energy_cost);

        events.push(DomainEvent::Born {
            plant_id: child_id,
            parent_id: Some(parent_id),
            position: target,
            lineage,
        });
    }

    // Mettre a jour les stats de seeds_produced
    for event in &events {
        if let DomainEvent::Born {
            parent_id: Some(pid),
            ..
        } = event
        {
            if let Some(stats) = state.find_stats_mut(*pid) {
                stats.seeds_produced += 1;
            }
        }
    }

    // Ajouter les nouvelles plantes
    for (child, brain) in new_plants {
        let child_id = child.id();
        state.plants.push(child);
        state.brains.insert(child_id, brain);
        state.plant_stats.insert(child_id, PlantStats::default());
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

    // c) Pluie de graines (tous les 50 ticks)
    if state.tick_count > 0
        && state
            .tick_count
            .is_multiple_of(state.config.seed_rain_interval)
        && !state.seed_bank.is_empty()
    {
        let genome = state.seed_bank.produce_seed(rng);
        let land_cells = state.island.land_cells();
        if !land_cells.is_empty() {
            let idx = (rng.next_f32() * land_cells.len() as f32) as usize;
            let idx = idx.min(land_cells.len() - 1);
            let pos = land_cells[idx];

            let occupied = state
                .plants
                .iter()
                .any(|p| !p.is_dead() && p.canopy().contains(&pos));

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
        let pos = plant.canopy()[0];
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
