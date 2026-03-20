// Phase actions — croissance, defense, exsudats, absorption, photosynthese, symbiose, maintenance.

use super::sim::SimState;
use crate::domain::events::DomainEvent;
use crate::domain::plant::{ExudateType, Plant, Pos};
use crate::domain::rng::Rng;
use crate::domain::symbiosis::calculate_exchange;
use crate::domain::world::GRID_SIZE;

/// Phase 3 : execution des actions decidees par les plantes.
pub fn phase_actions(
    state: &mut SimState,
    decisions: &[(u64, [f32; 8])],
    rng: &mut dyn Rng,
) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    let modifiers = state.season_cycle.current_modifiers();

    // Construire l'ordre aleatoire d'iteration (Fisher-Yates shuffle)
    let mut indices: Vec<usize> = (0..decisions.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = (rng.next_f32() * (i + 1) as f32) as usize;
        let j = j.min(i);
        indices.swap(i, j);
    }

    for &decision_idx in &indices {
        let (plant_id, outputs) = decisions[decision_idx];

        // Trouver l'index de la plante
        let plant_idx = match state.plants.iter().position(|p| p.id() == plant_id) {
            Some(idx) => idx,
            None => continue,
        };

        if state.plants[plant_idx].is_dead() {
            continue;
        }

        let grow_intensity = outputs[0];
        let grow_dir_x = outputs[1] * 2.0 - 1.0;
        let grow_dir_y = outputs[2] * 2.0 - 1.0;
        let canopy_vs_roots = outputs[3];
        let exudate_rate = outputs[4];
        let defense = outputs[5];
        let connect_signal = outputs[6];
        let connect_generosity = outputs[7];

        // a) Croissance
        if grow_intensity > state.config.growth_threshold
            && state.plants[plant_idx].energy().value() > state.config.growth_energy_cost
        {
            let target = find_growth_target(&state.plants[plant_idx], grow_dir_x, grow_dir_y);
            if let Some(target_pos) = target {
                if state.island.is_land(&target_pos) && state.world.is_valid(&target_pos) {
                    let is_canopy = canopy_vs_roots > 0.5;

                    // Verifier si la cellule est occupee par une autre plante
                    let occupant_id = find_occupant(&state.plants, &target_pos, plant_id);

                    if let Some(victim_id) = occupant_id {
                        // Tentative d'invasion
                        let attacker_energy = state.plants[plant_idx].energy().value();
                        let victim_idx = state.plants.iter().position(|p| p.id() == victim_id);

                        if let Some(vi) = victim_idx {
                            let defender_energy = state.plants[vi].energy().value();

                            // Seuil de defense
                            let victim_decisions =
                                decisions.iter().find(|(id, _)| *id == victim_id);
                            let victim_defense = victim_decisions.map(|(_, o)| o[5]).unwrap_or(0.0);
                            let threshold = if victim_defense > 0.5 {
                                state.config.invasion_defense_threshold
                            } else {
                                state.config.invasion_energy_threshold
                            };

                            if attacker_energy > defender_energy + threshold {
                                // Invasion reussie
                                state.plants[vi].remove_canopy_cell(&target_pos);
                                let event = state.plants[plant_idx].grow(target_pos, is_canopy);
                                events.push(event);
                                state.plants[plant_idx]
                                    .consume_energy(state.config.invasion_energy_cost);
                                state.plants[vi].damage(state.config.invasion_damage);

                                // Rompre la symbiose entre les deux
                                if state.symbiosis.remove_link(plant_id, victim_id) {
                                    events.push(DomainEvent::Unlinked {
                                        plant_a: plant_id,
                                        plant_b: victim_id,
                                    });
                                }

                                events.push(DomainEvent::Invaded {
                                    invader_id: plant_id,
                                    victim_id,
                                    cell: target_pos,
                                });
                            }
                        }
                    } else {
                        // Cellule libre : verifier les ressources du sol
                        let can_grow = if let Some(cell) = state.world.get(&target_pos) {
                            cell.carbon() >= state.config.growth_carbon_cost
                                && cell.nitrogen() >= state.config.growth_nitrogen_cost
                        } else {
                            false
                        };

                        if can_grow {
                            let event = state.plants[plant_idx].grow(target_pos, is_canopy);
                            events.push(event);
                            state.plants[plant_idx]
                                .consume_energy(state.config.growth_energy_cost / modifiers.growth);

                            // Deduire les ressources du sol
                            if let Some(cell) = state.world.get_mut(&target_pos) {
                                let c = cell.carbon();
                                cell.set_carbon(c - state.config.growth_carbon_cost);
                                let n = cell.nitrogen();
                                cell.set_nitrogen(n - state.config.growth_nitrogen_cost);
                            }
                        }
                    }
                }
            }
        }

        // b) Defense : consommer de l'energie
        if defense > 0.5 {
            state.plants[plant_idx].consume_energy(state.config.defense_energy_cost);
        }

        // c) Exsudats
        if exudate_rate > 0.1 {
            let exudate_type = state.plants[plant_idx].genetics().exudate_type();
            let canopy_cells: Vec<Pos> = state.plants[plant_idx].canopy().to_vec();
            for pos in &canopy_cells {
                if let Some(cell) = state.world.get_mut(pos) {
                    match exudate_type {
                        ExudateType::Carbon => {
                            let c = cell.carbon();
                            cell.set_carbon(c + exudate_rate * state.config.exudate_output_rate);
                        }
                        ExudateType::Nitrogen => {
                            let n = cell.nitrogen();
                            cell.set_nitrogen(n + exudate_rate * state.config.exudate_output_rate);
                        }
                    }
                    let ex = cell.exudates();
                    cell.set_exudates(ex + exudate_rate * state.config.exudate_output_rate);
                }
            }
            state.plants[plant_idx]
                .consume_energy(exudate_rate * state.config.exudate_energy_cost_rate);

            // Mettre a jour les stats
            let exudate_output = state.config.exudate_output_rate;
            if let Some(stats) = state.find_stats_mut(plant_id) {
                stats.exudates_emitted += exudate_rate * exudate_output * canopy_cells.len() as f32;
            }
        }

        // d) Absorption : pour chaque cellule racine
        let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
        let mut total_absorbed = 0.0_f32;
        for pos in &root_cells {
            if let Some(cell) = state.world.get_mut(pos) {
                let c_absorbed = cell.carbon().min(state.config.absorption_rate);
                let n_absorbed = cell.nitrogen().min(state.config.absorption_rate);
                let h_absorbed = cell.humidity().min(state.config.absorption_rate);

                let c = cell.carbon();
                cell.set_carbon(c - c_absorbed);
                let n = cell.nitrogen();
                cell.set_nitrogen(n - n_absorbed);
                let h = cell.humidity();
                cell.set_humidity(h - h_absorbed);

                total_absorbed += c_absorbed + n_absorbed + h_absorbed;
            }
        }
        state.plants[plant_idx].gain_energy(total_absorbed);

        // Mettre a jour les stats (sol deplete)
        if let Some(stats) = state.find_stats_mut(plant_id) {
            stats.soil_depleted += total_absorbed;
        }

        // e) Photosynthese : pour chaque cellule canopee
        let canopy_cells: Vec<Pos> = state.plants[plant_idx].canopy().to_vec();
        let mut photo_gain = 0.0_f32;
        for pos in &canopy_cells {
            if let Some(cell) = state.world.get(pos) {
                photo_gain += cell.light() * state.config.photosynthesis_rate;
            }
        }
        state.plants[plant_idx].gain_energy(photo_gain);

        // f) Echanges mycorhiziens
        let links = state.symbiosis.links_of(plant_id);
        let link_data: Vec<(u64, u64)> = links.iter().map(|l| (l.plant_a(), l.plant_b())).collect();

        for (la, lb) in &link_data {
            let other_id = if *la == plant_id { *lb } else { *la };

            // Verifier connect_signal des deux cotes
            let other_decisions = decisions.iter().find(|(id, _)| *id == other_id);
            let other_connect = other_decisions.map(|(_, o)| o[6]).unwrap_or(0.0);

            if connect_signal > 0.5 && other_connect > 0.5 {
                let other_generosity = other_decisions.map(|(_, o)| o[7]).unwrap_or(0.0);
                let avg_generosity = (connect_generosity + other_generosity) / 2.0;

                let my_energy = state.plants[plant_idx].energy().value();
                let other_idx = state.plants.iter().position(|p| p.id() == other_id);
                if let Some(oi) = other_idx {
                    let other_energy = state.plants[oi].energy().value();
                    let (a_to_b, b_to_a) =
                        calculate_exchange(my_energy, other_energy, avg_generosity * 0.1);

                    if a_to_b > 0.0 {
                        state.plants[plant_idx].consume_energy(a_to_b);
                        state.plants[oi].gain_energy(a_to_b);
                    } else if b_to_a > 0.0 {
                        state.plants[oi].consume_energy(b_to_a);
                        state.plants[plant_idx].gain_energy(b_to_a);
                    }

                    // Stats
                    if let Some(stats) = state.find_stats_mut(plant_id) {
                        stats.cn_exchanges += a_to_b + b_to_a;
                    }
                }
            }
        }

        // Mettre a jour les stats de symbiose
        let symbiotic_count = state.symbiosis.links_of(plant_id).len() as u32;
        if let Some(stats) = state.find_stats_mut(plant_id) {
            stats.symbiotic_connections = symbiotic_count;
        }

        // g) Maintenance : consommer maintenance_rate * biomass energie
        let biomass = state.plants[plant_idx].biomass().value() as f32;
        state.plants[plant_idx].consume_energy(state.config.maintenance_rate * biomass);

        // Mettre a jour les stats de territoire et biomasse
        let territory =
            (state.plants[plant_idx].canopy().len() + state.plants[plant_idx].roots().len()) as u16;
        let b = state.plants[plant_idx].biomass().value();
        let age = state.plants[plant_idx].age();
        if let Some(stats) = state.find_stats_mut(plant_id) {
            if territory > stats.max_territory {
                stats.max_territory = territory;
            }
            if b > stats.max_biomass {
                stats.max_biomass = b;
            }
            stats.lifetime = age;
        }
    }

    events
}

/// Trouve la cellule cible de croissance la plus proche de la direction souhaitee.
pub fn find_growth_target(plant: &Plant, dir_x: f32, dir_y: f32) -> Option<Pos> {
    // Chercher parmi les voisins de toutes les cellules de canopee
    let mut best: Option<(Pos, f32)> = None;

    for canopy_pos in plant.canopy() {
        // 4 voisins cardinaux
        let neighbors = [
            (
                canopy_pos.x.wrapping_sub(1),
                canopy_pos.y,
                -1.0_f32,
                0.0_f32,
            ),
            (canopy_pos.x + 1, canopy_pos.y, 1.0, 0.0),
            (canopy_pos.x, canopy_pos.y.wrapping_sub(1), 0.0, -1.0),
            (canopy_pos.x, canopy_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= GRID_SIZE || *ny >= GRID_SIZE {
                continue;
            }
            let candidate = Pos { x: *nx, y: *ny };
            // Ne pas pousser sur une cellule deja occupee par cette plante
            if plant.canopy().contains(&candidate) || plant.roots().contains(&candidate) {
                continue;
            }
            // Score = produit scalaire avec la direction souhaitee
            let score = ndx * dir_x + ndy * dir_y;
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((candidate, score)),
            }
        }
    }

    best.map(|(pos, _)| pos)
}

/// Trouve l'id de la plante occupant une cellule (canopee), exclut plant_id donne.
pub fn find_occupant(plants: &[Plant], pos: &Pos, exclude: u64) -> Option<u64> {
    for plant in plants {
        if plant.id() == exclude || plant.is_dead() {
            continue;
        }
        if plant.canopy().contains(pos) {
            return Some(plant.id());
        }
    }
    None
}
