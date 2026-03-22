// Phase actions — croissance, defense, exsudats, absorption, photosynthese, symbiose, maintenance.

use std::collections::HashMap;

use super::season::SeasonModifiers;
use super::sim::SimState;
use crate::domain::events::DomainEvent;
use crate::domain::plant::{energy_cap, ExudateType, Plant, PlantState, Pos};
use crate::domain::rng::Rng;
use crate::domain::world::{Cell, World, GRID_SIZE};

/// Phase 3 : execution des actions decidees par les plantes.
/// Orchestre les sous-fonctions dans l'ordre pour chaque plante.
pub fn phase_actions(
    state: &mut SimState,
    decisions: &[(u64, [f32; 8])],
    rng: &mut dyn Rng,
) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    let modifiers = state.season_cycle.current_modifiers();

    // Construire la footprint map : pos -> plant_id (pour eliminer find_occupant O(n))
    // L'emprise au sol est exclusive, chaque cellule n'appartient qu'a une plante.
    let mut canopy_map: HashMap<Pos, u64> = HashMap::new();
    for plant in state.plants.iter() {
        if plant.is_dead() {
            continue;
        }
        for &pos in plant.footprint() {
            canopy_map.insert(pos, plant.id());
        }
    }

    // Construire la root map : pos -> liste de plant_ids (pour symbiose et monoculture)
    let mut root_map: HashMap<Pos, Vec<u64>> = HashMap::new();
    for plant in state.plants.iter() {
        if plant.is_dead() {
            continue;
        }
        for &pos in plant.roots() {
            root_map.entry(pos).or_default().push(plant.id());
        }
    }

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

        // Les graines sont inertes : elles ne font aucune action
        if state.plants[plant_idx].state() == PlantState::Seed {
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

        // a) Croissance + invasion
        let mut growth_events = action_growth(
            state,
            decisions,
            plant_id,
            plant_idx,
            grow_intensity,
            grow_dir_x,
            grow_dir_y,
            canopy_vs_roots,
            &mut canopy_map,
            &modifiers,
        );
        events.append(&mut growth_events);

        // b) Defense
        action_defense(state, plant_idx, defense);

        // c) Exsudats racinaires
        action_exudates(state, plant_id, plant_idx, exudate_rate);

        // d) Absorption
        action_absorption(state, plant_id, plant_idx);

        // e) Photosynthese
        action_photosynthesis(state, plant_idx);

        // f+g) Symbiose (echanges + creation de liens)
        let mut symbiosis_events = action_symbiosis(
            state,
            decisions,
            plant_id,
            plant_idx,
            connect_signal,
            connect_generosity,
            &root_map,
        );
        events.append(&mut symbiosis_events);

        // h+i+j) Maintenance, vieillissement, famine, monoculture
        action_maintenance(state, plant_id, plant_idx, &canopy_map, &root_map);
    }

    events
}

/// Type de croissance choisi par le brain.
enum GrowthType {
    Canopy,
    Footprint,
    Roots,
}

/// a) Croissance et invasion — gestion de la pousse vers une cellule cible.
/// Le brain output[3] (canopy_vs_roots) controle 3 directions :
/// - > 0.66 : canopee aerienne (partagee, pas d'invasion)
/// - 0.33 - 0.66 : emprise au sol (exclusive, invasion possible)
/// - < 0.33 : racines (partagees, chimiotaxie, gratuit)
#[allow(clippy::too_many_arguments)]
fn action_growth(
    state: &mut SimState,
    decisions: &[(u64, [f32; 8])],
    plant_id: u64,
    plant_idx: usize,
    grow_intensity: f32,
    grow_dir_x: f32,
    grow_dir_y: f32,
    canopy_vs_roots: f32,
    canopy_map: &mut HashMap<Pos, u64>,
    modifiers: &SeasonModifiers,
) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    // Determiner le type de croissance
    let growth_type = if canopy_vs_roots > 0.66 {
        GrowthType::Canopy
    } else if canopy_vs_roots > 0.33 {
        GrowthType::Footprint
    } else {
        GrowthType::Roots
    };

    // Les racines sont gratuites, les autres types coutent de l'energie
    let needs_energy = !matches!(growth_type, GrowthType::Roots);
    if grow_intensity <= state.config.growth_threshold {
        return events;
    }
    if needs_energy && state.plants[plant_idx].energy().value() <= state.config.growth_energy_cost {
        return events;
    }

    // Trouver la cellule cible selon le type de croissance
    let target = match growth_type {
        GrowthType::Footprint => {
            find_growth_target(&state.plants[plant_idx], grow_dir_x, grow_dir_y)
        }
        GrowthType::Canopy => {
            // Chercher voisin de l'emprise OU de la canopee existante
            find_canopy_growth_target(&state.plants[plant_idx], grow_dir_x, grow_dir_y)
        }
        GrowthType::Roots => {
            // Chimiotaxie d'abord, fallback sur la direction du brain
            find_root_growth_toward_neighbor(state, plant_idx, plant_id).or_else(|| {
                find_root_growth_target(&state.plants[plant_idx], grow_dir_x, grow_dir_y)
            })
        }
    };

    let target_pos = match target {
        Some(pos) if state.island.is_land(&pos) && state.world.is_valid(&pos) => pos,
        _ => return events,
    };

    match growth_type {
        GrowthType::Footprint => {
            // Verifier si la cellule est occupee par une autre plante (lookup O(1))
            let occupant_id = canopy_map
                .get(&target_pos)
                .copied()
                .filter(|&id| id != plant_id);

            if let Some(victim_id) = occupant_id {
                // Verifier si la victime est une graine — invasion automatique et gratuite
                let victim_idx = state.plants.iter().position(|p| p.id() == victim_id);
                if let Some(vi) = victim_idx {
                    if state.plants[vi].state() == PlantState::Seed {
                        // Invasion gratuite : la graine meurt
                        let seed_vitality = state.plants[vi].vitality().value();
                        state.plants[vi].damage(seed_vitality);
                        if let Some(event) = state.plants[vi].update_state() {
                            events.push(event);
                        }
                        // L'attaquant gagne la cellule en emprise
                        let event = state.plants[plant_idx].grow_footprint(target_pos);
                        events.push(event);
                        canopy_map.insert(target_pos, plant_id);
                    } else {
                        // Tentative d'invasion classique
                        let attacker_energy = state.plants[plant_idx].energy().value();
                        let defender_energy = state.plants[vi].energy().value();

                        // Seuil de defense
                        let victim_decisions = decisions.iter().find(|(id, _)| *id == victim_id);
                        let victim_defense = victim_decisions.map(|(_, o)| o[5]).unwrap_or(0.0);
                        let threshold = if victim_defense > 0.5 {
                            state.config.invasion_defense_threshold
                        } else {
                            state.config.invasion_energy_threshold
                        };

                        if attacker_energy > defender_energy + threshold {
                            // Invasion reussie — on retire de l'emprise de la victime
                            state.plants[vi].remove_footprint_cell(&target_pos);
                            // L'attaquant gagne la cellule en emprise (footprint)
                            let event = state.plants[plant_idx].grow_footprint(target_pos);
                            events.push(event);
                            state.plants[plant_idx]
                                .consume_energy(state.config.invasion_energy_cost);
                            state.plants[vi].damage(state.config.invasion_damage);

                            canopy_map.insert(target_pos, plant_id);

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
                    let event = state.plants[plant_idx].grow_footprint(target_pos);
                    events.push(event);
                    state.plants[plant_idx]
                        .consume_energy(state.config.growth_energy_cost / modifiers.growth);

                    // Bonus croissance : la plante qui grandit gagne de l'energie
                    state.plants[plant_idx].gain_energy(2.0);

                    canopy_map.insert(target_pos, plant_id);

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
        GrowthType::Canopy => {
            // La canopee est partagee — pas d'invasion, pas de check d'occupation
            // Verifier la limite de canopee
            if state.plants[plant_idx].canopy().len() >= state.plants[plant_idx].max_canopy() {
                return events;
            }
            // Verifier les ressources du sol
            let can_grow = if let Some(cell) = state.world.get(&target_pos) {
                cell.carbon() >= state.config.growth_carbon_cost
                    && cell.nitrogen() >= state.config.growth_nitrogen_cost
            } else {
                false
            };
            if can_grow {
                if let Some(event) = state.plants[plant_idx].grow_canopy(target_pos) {
                    events.push(event);
                }
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
        GrowthType::Roots => {
            // Les racines sont partagees — pas d'invasion, gratuit en energie
            if state.plants[plant_idx].roots().len() >= state.plants[plant_idx].max_roots() {
                return events;
            }
            if let Some(event) = state.plants[plant_idx].grow_roots(target_pos) {
                events.push(event);
            }
        }
    }

    events
}

/// b) Defense — consomme de l'energie si defense active.
fn action_defense(state: &mut SimState, plant_idx: usize, defense: f32) {
    if defense > 0.5 {
        state.plants[plant_idx].consume_energy(state.config.defense_energy_cost);
    }
}

/// c) Exsudats racinaires — depose des nutriments sur les cellules racinaires.
fn action_exudates(state: &mut SimState, plant_id: u64, plant_idx: usize, exudate_rate: f32) {
    if exudate_rate <= 0.1 {
        return;
    }

    let exudate_type = state.plants[plant_idx].genetics().exudate_type();
    let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
    for pos in &root_cells {
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
    state.plants[plant_idx].consume_energy(exudate_rate * state.config.exudate_energy_cost_rate);

    // Mettre a jour les stats
    let exudate_output = state.config.exudate_output_rate;
    if let Some(stats) = state.find_stats_mut(plant_id) {
        stats.exudates_emitted += exudate_rate * exudate_output * root_cells.len() as f32;
    }
}

/// d) Absorption — extrait les nutriments du sol sous les racines.
fn action_absorption(state: &mut SimState, plant_id: u64, plant_idx: usize) {
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

    if let Some(stats) = state.find_stats_mut(plant_id) {
        stats.soil_depleted += total_absorbed;
    }
}

/// e) Photosynthese — gain d'energie proportionnel a la lumiere sur la canopee.
/// Ombre dynamique : si plusieurs plantes ont de la canopee sur une meme cellule,
/// la plante avec la plus grande emprise au sol (= la plus haute) recoit la lumiere pleine,
/// les autres recoivent la lumiere ombragee (canopy_light).
fn action_photosynthesis(state: &mut SimState, plant_idx: usize) {
    let plant_id = state.plants[plant_idx].id();
    let my_fp_size = state.plants[plant_idx].footprint().len();
    let canopy_cells: Vec<Pos> = state.plants[plant_idx].canopy().to_vec();
    let mut photo_gain = 0.0_f32;

    for pos in &canopy_cells {
        let base_light = state.world.get(pos).map(|c| c.light()).unwrap_or(0.0);

        // Verifier si une autre plante plus grande a de la canopee sur cette cellule
        let is_shaded = state.plants.iter().any(|other| {
            other.id() != plant_id
                && !other.is_dead()
                && other.footprint().len() > my_fp_size
                && other.canopy().contains(pos)
        });

        if is_shaded {
            // Lumiere reduite sous la canopee d'une plante plus grande
            photo_gain += state.config.canopy_light * state.config.photosynthesis_rate;
        } else {
            photo_gain += base_light * state.config.photosynthesis_rate;
        }
    }
    state.plants[plant_idx].gain_energy(photo_gain);
}

/// f+g) Symbiose — echanges mycorhiziens et creation de liens.
fn action_symbiosis(
    state: &mut SimState,
    decisions: &[(u64, [f32; 8])],
    plant_id: u64,
    plant_idx: usize,
    connect_signal: f32,
    connect_generosity: f32,
    root_map: &HashMap<Pos, Vec<u64>>,
) -> Vec<DomainEvent> {
    let mut events = Vec::new();

    // f) Echanges mycorhiziens existants — transfert bidirectionnel C/N via le sol
    let links = state.symbiosis.links_of(plant_id);
    let link_data: Vec<(u64, u64)> = links.iter().map(|l| (l.plant_a(), l.plant_b())).collect();

    for (la, lb) in &link_data {
        let other_id = if *la == plant_id { *lb } else { *la };

        let other_decisions = decisions.iter().find(|(id, _)| *id == other_id);
        let other_connect = other_decisions.map(|(_, o)| o[6]).unwrap_or(0.0);

        if connect_signal > 0.5 && other_connect > 0.5 {
            let other_generosity = other_decisions.map(|(_, o)| o[7]).unwrap_or(0.0);
            let avg_generosity = (connect_generosity + other_generosity) / 2.0;
            let transfer_rate = avg_generosity * 0.02; // taux de transfert par tick

            // Calculer les ressources moyennes sous les racines de chaque plante
            let my_roots: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
            let other_idx = state.plants.iter().position(|p| p.id() == other_id);

            if let Some(oi) = other_idx {
                let other_roots: Vec<Pos> = state.plants[oi].roots().to_vec();

                // Carbone moyen sous mes racines vs les siennes
                let my_carbon = avg_resource(&state.world, &my_roots, Cell::carbon);
                let other_carbon = avg_resource(&state.world, &other_roots, Cell::carbon);

                // Azote moyen sous mes racines vs les siennes
                let my_nitrogen = avg_resource(&state.world, &my_roots, Cell::nitrogen);
                let other_nitrogen = avg_resource(&state.world, &other_roots, Cell::nitrogen);

                // Echange C : du plus riche vers le plus pauvre
                let c_diff = my_carbon - other_carbon;
                let c_transfer = c_diff * transfer_rate;

                // Echange N : du plus riche vers le plus pauvre
                let n_diff = my_nitrogen - other_nitrogen;
                let n_transfer = n_diff * transfer_rate;

                // Appliquer les transferts dans le sol
                if c_transfer.abs() > 0.001 {
                    apply_transfer(
                        &mut state.world,
                        &my_roots,
                        &other_roots,
                        c_transfer,
                        Cell::carbon,
                        Cell::set_carbon,
                    );
                }
                if n_transfer.abs() > 0.001 {
                    apply_transfer(
                        &mut state.world,
                        &my_roots,
                        &other_roots,
                        n_transfer,
                        Cell::nitrogen,
                        Cell::set_nitrogen,
                    );
                }

                // Gain d'energie via l'echange (l'echange nourrit les deux plantes)
                let total_exchanged = c_transfer.abs() + n_transfer.abs();
                state.plants[plant_idx].gain_energy(total_exchanged * 0.5);
                state.plants[oi].gain_energy(total_exchanged * 0.5);

                if let Some(stats) = state.find_stats_mut(plant_id) {
                    stats.cn_exchanges += total_exchanged;
                }
            }
        }
    }

    // g) Creation de liens mycorhiziens
    if connect_signal > 0.5 {
        let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();

        let mut link_candidates: Vec<u64> = Vec::new();
        for root_pos in &root_cells {
            if let Some(neighbors) = root_map.get(root_pos) {
                for &other_id in neighbors {
                    if other_id == plant_id {
                        continue;
                    }
                    let is_valid = state.plants.iter().any(|p| {
                        p.id() == other_id
                            && !p.is_dead()
                            && p.state() != crate::domain::plant::PlantState::Seed
                    });
                    if !is_valid {
                        continue;
                    }
                    let other_connect = decisions
                        .iter()
                        .find(|(id, _)| *id == other_id)
                        .map(|(_, o)| o[6])
                        .unwrap_or(0.0);
                    if other_connect > 0.5 && !link_candidates.contains(&other_id) {
                        link_candidates.push(other_id);
                    }
                }
            }
        }

        for other_id in link_candidates {
            if state.symbiosis.create_link(plant_id, other_id) {
                events.push(DomainEvent::Linked {
                    plant_a: plant_id,
                    plant_b: other_id,
                });
            }
        }
    }

    // Mettre a jour les stats de symbiose
    let symbiotic_count = state.symbiosis.links_of(plant_id).len() as u32;
    if let Some(stats) = state.find_stats_mut(plant_id) {
        stats.symbiotic_connections = symbiotic_count;
    }

    events
}

/// h+i+j) Maintenance, vieillissement, famine et penalite monoculture.
fn action_maintenance(
    state: &mut SimState,
    plant_id: u64,
    plant_idx: usize,
    canopy_map: &HashMap<Pos, u64>,
    root_map: &HashMap<Pos, Vec<u64>>,
) {
    let config = &state.config;

    // h) Maintenance : consommer maintenance_rate * biomass energie
    // Les plantes matures ont un cout de maintenance reduit
    let maintenance_multiplier =
        if state.plants[plant_idx].state() == crate::domain::plant::PlantState::Mature {
            0.5
        } else {
            1.0
        };
    let biomass = state.plants[plant_idx].biomass().value() as f32;
    state.plants[plant_idx]
        .consume_energy(config.maintenance_rate * biomass * maintenance_multiplier);

    // i) Vieillissement : drain de vitalite proportionnel a l'age
    let age = state.plants[plant_idx].age() as f32;
    let aging_drain = config.aging_base_rate * (age / 1000.0);
    state.plants[plant_idx].damage(aging_drain);

    // j) Famine : si energie < seuil, la vitalite draine de plus en plus vite
    let energy_val = state.plants[plant_idx].energy().value();
    let e_cap = energy_cap(
        state.plants[plant_idx].biomass(),
        state.plants[plant_idx].genetics().energy_factor(),
    );
    let starvation_threshold = e_cap * config.starvation_threshold;

    if energy_val < starvation_threshold && starvation_threshold > 0.0 {
        let ratio = 1.0 - (energy_val / starvation_threshold);
        let drain = config.starvation_drain_rate * (ratio * ratio * ratio);
        state.plants[plant_idx].damage(drain);
    }

    // Mettre a jour les stats de territoire et biomasse
    let territory =
        (state.plants[plant_idx].footprint().len() + state.plants[plant_idx].roots().len()) as u16;
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

    // Penalite monoculture : si > 80% des voisins sont de la meme lignee
    let plant_lineage_id = state.plants[plant_idx].lineage().id();
    let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
    if root_cells.len() > 1 {
        let mut same_lineage_count = 0usize;
        let mut total_occupied = 0usize;
        let mut counted: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for pos in &root_cells {
            if let Some(&occ_id) = canopy_map.get(pos) {
                if occ_id != plant_id && counted.insert(occ_id) {
                    total_occupied += 1;
                    if let Some(other) = state.plants.iter().find(|p| p.id() == occ_id) {
                        if other.lineage().id() == plant_lineage_id {
                            same_lineage_count += 1;
                        }
                    }
                }
            }
            if let Some(root_ids) = root_map.get(pos) {
                for &occ_id in root_ids {
                    if occ_id != plant_id && counted.insert(occ_id) {
                        if let Some(other) = state.plants.iter().find(|p| p.id() == occ_id) {
                            if !other.is_dead() {
                                total_occupied += 1;
                                if other.lineage().id() == plant_lineage_id {
                                    same_lineage_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        if total_occupied > 0 && (same_lineage_count as f32 / total_occupied as f32) > 0.8 {
            if let Some(stats) = state.find_stats_mut(plant_id) {
                stats.monoculture_penalty += 1.0;
            }
        }
    }
}

/// Trouve la cellule cible de croissance la plus proche de la direction souhaitee.
/// Cherche parmi les voisins de l'emprise au sol (footprint).
pub fn find_growth_target(plant: &Plant, dir_x: f32, dir_y: f32) -> Option<Pos> {
    let mut best: Option<(Pos, f32)> = None;

    for fp_pos in plant.footprint() {
        let neighbors = [
            (fp_pos.x.wrapping_sub(1), fp_pos.y, -1.0_f32, 0.0_f32),
            (fp_pos.x + 1, fp_pos.y, 1.0, 0.0),
            (fp_pos.x, fp_pos.y.wrapping_sub(1), 0.0, -1.0),
            (fp_pos.x, fp_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= GRID_SIZE || *ny >= GRID_SIZE {
                continue;
            }
            let candidate = Pos { x: *nx, y: *ny };
            if plant.footprint().contains(&candidate) || plant.roots().contains(&candidate) {
                continue;
            }
            let score = ndx * dir_x + ndy * dir_y;
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((candidate, score)),
            }
        }
    }

    best.map(|(pos, _)| pos)
}

/// Trouve la cellule cible de croissance de canopee la plus alignee avec la direction souhaitee.
/// Cherche parmi les voisins de l'emprise au sol (footprint) ET de la canopee existante.
pub fn find_canopy_growth_target(plant: &Plant, dir_x: f32, dir_y: f32) -> Option<Pos> {
    let mut best: Option<(Pos, f32)> = None;

    // Collecter toutes les positions sources (footprint + canopy)
    let sources: Vec<Pos> = plant
        .footprint()
        .iter()
        .chain(plant.canopy().iter())
        .copied()
        .collect();

    for src_pos in &sources {
        let neighbors = [
            (src_pos.x.wrapping_sub(1), src_pos.y, -1.0_f32, 0.0_f32),
            (src_pos.x + 1, src_pos.y, 1.0, 0.0),
            (src_pos.x, src_pos.y.wrapping_sub(1), 0.0, -1.0),
            (src_pos.x, src_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= GRID_SIZE || *ny >= GRID_SIZE {
                continue;
            }
            let candidate = Pos { x: *nx, y: *ny };
            // Ne pas pousser sur une cellule deja occupee par la canopee ou le footprint
            if plant.footprint().contains(&candidate) || plant.canopy().contains(&candidate) {
                continue;
            }
            let score = ndx * dir_x + ndy * dir_y;
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((candidate, score)),
            }
        }
    }

    best.map(|(pos, _)| pos)
}

/// Trouve la cellule cible de croissance racinaire orientee vers le voisin le plus proche.
/// Simule la chimiotaxie : les racines poussent vers les exsudats des voisins.
fn find_root_growth_toward_neighbor(
    state: &SimState,
    plant_idx: usize,
    plant_id: u64,
) -> Option<Pos> {
    let plant = &state.plants[plant_idx];
    let my_center = plant.footprint()[0];

    // Trouver la plante vivante la plus proche (autre que soi)
    let mut closest_dist = f32::MAX;
    let mut closest_dir = (0.0_f32, 0.0_f32);
    let mut found_neighbor = false;

    for other in &state.plants {
        if other.id() == plant_id || other.is_dead() {
            continue;
        }
        if other.state() == crate::domain::plant::PlantState::Seed {
            continue;
        }
        let other_center = other.footprint()[0];
        let dx = other_center.x as f32 - my_center.x as f32;
        let dy = other_center.y as f32 - my_center.y as f32;
        let dist = dx * dx + dy * dy;
        if dist < closest_dist && dist > 0.0 {
            closest_dist = dist;
            let sqrt_dist = dist.sqrt();
            closest_dir = (dx / sqrt_dist, dy / sqrt_dist);
            found_neighbor = true;
        }
    }

    if !found_neighbor {
        return None;
    }

    // Chercher parmi les voisins des racines existantes
    find_root_growth_target(plant, closest_dir.0, closest_dir.1)
}

/// Trouve la cellule voisine d'une racine la plus alignee avec la direction souhaitee.
fn find_root_growth_target(plant: &Plant, dir_x: f32, dir_y: f32) -> Option<Pos> {
    let mut best: Option<(Pos, f32)> = None;

    for root_pos in plant.roots() {
        let neighbors = [
            (root_pos.x.wrapping_sub(1), root_pos.y, -1.0_f32, 0.0_f32),
            (root_pos.x + 1, root_pos.y, 1.0, 0.0),
            (root_pos.x, root_pos.y.wrapping_sub(1), 0.0, -1.0),
            (root_pos.x, root_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= GRID_SIZE || *ny >= GRID_SIZE {
                continue;
            }
            let candidate = Pos { x: *nx, y: *ny };
            if plant.footprint().contains(&candidate) || plant.roots().contains(&candidate) {
                continue;
            }
            let score = ndx * dir_x + ndy * dir_y;
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((candidate, score)),
            }
        }
    }

    best.map(|(pos, _)| pos)
}

/// Moyenne d'une ressource sur les cellules racines.
fn avg_resource(world: &World, roots: &[Pos], getter: fn(&Cell) -> f32) -> f32 {
    if roots.is_empty() {
        return 0.0;
    }
    let sum: f32 = roots
        .iter()
        .filter_map(|pos| world.get(pos))
        .map(getter)
        .sum();
    sum / roots.len() as f32
}

/// Transfere une ressource du sol d'un groupe de cellules vers un autre.
/// transfer > 0 : de `from_roots` vers `to_roots`
/// transfer < 0 : de `to_roots` vers `from_roots`
fn apply_transfer(
    world: &mut World,
    from_roots: &[Pos],
    to_roots: &[Pos],
    transfer: f32,
    getter: fn(&Cell) -> f32,
    setter: fn(&mut Cell, f32),
) {
    if from_roots.is_empty() || to_roots.is_empty() {
        return;
    }

    let (donors, receivers) = if transfer > 0.0 {
        (from_roots, to_roots)
    } else {
        (to_roots, from_roots)
    };

    let per_cell_donor = transfer.abs() / donors.len() as f32;
    let per_cell_receiver = transfer.abs() / receivers.len() as f32;

    for pos in donors {
        if let Some(cell) = world.get_mut(pos) {
            let current = getter(cell);
            setter(cell, current - per_cell_donor);
        }
    }
    for pos in receivers {
        if let Some(cell) = world.get_mut(pos) {
            let current = getter(cell);
            setter(cell, current + per_cell_receiver);
        }
    }
}
