// Phase actions — croissance, defense, exsudats, absorption, photosynthese, symbiose, maintenance.

use std::collections::HashMap;

use super::photosynthesis::photosynthesis_batch;
use super::season::SeasonModifiers;
use super::sim::SimState;
use crate::domain::events::DomainEvent;
use crate::domain::plant::{energy_cap, ExudateType, PlantState, Pos};
use crate::domain::rng::Rng;
use crate::domain::traits::PlantEntity;
use crate::domain::world::{Cell, World};

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
    let mut footprint_map: HashMap<Pos, u64> = HashMap::new();
    for plant in state.plants.iter() {
        if plant.is_dead() {
            continue;
        }
        for &pos in plant.footprint() {
            footprint_map.insert(pos, plant.id());
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

    // Photosynthese batch AVANT la boucle per-plant
    // Chaque couche de canopee filtre la lumiere pour les plantes en dessous
    photosynthesis_batch(state);

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
            &mut footprint_map,
            &modifiers,
        );
        events.append(&mut growth_events);

        // b) Defense
        action_defense(state, plant_idx, defense);

        // c) Exsudats racinaires
        action_exudates(state, plant_id, plant_idx, exudate_rate);

        // d) Absorption
        action_absorption(state, plant_id, plant_idx);

        // e) Photosynthese — traitee en batch avant la boucle (photosynthesis_batch)

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
        action_maintenance(state, plant_id, plant_idx, &footprint_map, &root_map);
    }

    events
}

/// Diviseur pour le cout de croissance proportionnel.
const GROWTH_COST_DIVISOR: f32 = 20.0;

/// Calcule le cout de croissance proportionnel au nombre de cellules existantes.
/// Les premieres cellules coutent quasi rien, les dernieres coutent cher.
/// Formule : base_cost × (nb_cellules_existantes + 1) / GROWTH_COST_DIVISOR
fn growth_cost(base_cost: f32, existing_cells: usize) -> f32 {
    base_cost * (existing_cells as f32 + 1.0) / GROWTH_COST_DIVISOR
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
    footprint_map: &mut HashMap<Pos, u64>,
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
    if needs_energy {
        let existing = match growth_type {
            GrowthType::Footprint => state.plants[plant_idx].footprint().len(),
            GrowthType::Canopy => state.plants[plant_idx].canopy().len(),
            GrowthType::Roots => 0, // racines gratuites en energie
        };
        let energy_cost = growth_cost(state.config.growth_energy_cost, existing);
        if state.plants[plant_idx].energy().value() <= energy_cost {
            return events;
        }
    }

    // Trouver la cellule cible selon le type de croissance
    let grid_size = state.world.size();
    let target = match growth_type {
        GrowthType::Footprint => find_growth_target(
            state.plants[plant_idx].as_ref(),
            grow_dir_x,
            grow_dir_y,
            grid_size,
        ),
        GrowthType::Canopy => {
            // Chercher voisin de l'emprise OU de la canopee existante
            find_canopy_growth_target(
                state.plants[plant_idx].as_ref(),
                grow_dir_x,
                grow_dir_y,
                grid_size,
            )
        }
        GrowthType::Roots => {
            // Chimiotaxie d'abord, fallback sur la direction du brain
            find_root_growth_toward_neighbor(state, plant_idx, plant_id).or_else(|| {
                find_root_growth_target(
                    state.plants[plant_idx].as_ref(),
                    grow_dir_x,
                    grow_dir_y,
                    grid_size,
                )
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
            let occupant_id = footprint_map
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
                        footprint_map.insert(target_pos, plant_id);
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

                            footprint_map.insert(target_pos, plant_id);

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
                // Les fixatrices d'azote n'ont PAS besoin de N du sol (elles le fabriquent)
                let is_fixer =
                    state.plants[plant_idx].genetics().exudate_type() == ExudateType::Nitrogen;

                // Couts proportionnels : footprint = bois = C dominant, peu de N
                let fp_count = state.plants[plant_idx].footprint().len();
                let energy_cost = growth_cost(state.config.growth_energy_cost, fp_count);
                let carbon_cost =
                    growth_cost(state.config.growth_carbon_cost * 2.0, fp_count); // C dominant pour le bois
                let nitrogen_cost =
                    growth_cost(state.config.growth_nitrogen_cost * 0.3, fp_count); // peu de N pour le bois

                let can_grow = if let Some(cell) = state.world.get(&target_pos) {
                    cell.carbon() >= carbon_cost
                        && (is_fixer || cell.nitrogen() >= nitrogen_cost)
                } else {
                    false
                };

                if can_grow {
                    let event = state.plants[plant_idx].grow_footprint(target_pos);
                    events.push(event);
                    state.plants[plant_idx]
                        .consume_energy(energy_cost / modifiers.growth);

                    // Bonus croissance : la plante qui grandit gagne de l'energie
                    state.plants[plant_idx].gain_energy(2.0);

                    footprint_map.insert(target_pos, plant_id);

                    // Deduire les ressources du sol (les fixatrices ne consomment pas de N)
                    if let Some(cell) = state.world.get_mut(&target_pos) {
                        let c = cell.carbon();
                        cell.set_carbon(c - carbon_cost);
                        if !is_fixer {
                            let n = cell.nitrogen();
                            cell.set_nitrogen(n - nitrogen_cost);
                        }
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
            // Verifier les ressources du sol (fixatrices n'ont pas besoin de N)
            let is_fixer =
                state.plants[plant_idx].genetics().exudate_type() == ExudateType::Nitrogen;

            // Couts proportionnels : canopee = feuilles = C standard, N dominant (chlorophylle)
            let canopy_count = state.plants[plant_idx].canopy().len();
            let energy_cost = growth_cost(state.config.growth_energy_cost, canopy_count);
            let carbon_cost =
                growth_cost(state.config.growth_carbon_cost, canopy_count); // C standard pour les feuilles
            let nitrogen_cost =
                growth_cost(state.config.growth_nitrogen_cost * 2.0, canopy_count); // N dominant pour la chlorophylle

            let can_grow = if let Some(cell) = state.world.get(&target_pos) {
                cell.carbon() >= carbon_cost
                    && (is_fixer || cell.nitrogen() >= nitrogen_cost)
            } else {
                false
            };
            if can_grow {
                if let Some(event) = state.plants[plant_idx].grow_canopy(target_pos) {
                    events.push(event);
                }
                state.plants[plant_idx]
                    .consume_energy(energy_cost / modifiers.growth);

                // Deduire les ressources du sol (fixatrices ne consomment pas de N)
                if let Some(cell) = state.world.get_mut(&target_pos) {
                    let c = cell.carbon();
                    cell.set_carbon(c - carbon_cost);
                    if !is_fixer {
                        let n = cell.nitrogen();
                        cell.set_nitrogen(n - nitrogen_cost);
                    }
                }
            }
        }
        GrowthType::Roots => {
            // Les racines sont partagees — pas d'invasion, gratuit en energie
            if state.plants[plant_idx].roots().len() >= state.plants[plant_idx].max_roots() {
                return events;
            }

            // Couts proportionnels : racines = tres peu de C, pas de N
            let root_count = state.plants[plant_idx].roots().len();
            let carbon_cost =
                growth_cost(state.config.growth_carbon_cost * 0.2, root_count); // tres peu de C

            let can_grow = if let Some(cell) = state.world.get(&target_pos) {
                cell.carbon() >= carbon_cost
            } else {
                false
            };

            if can_grow {
                if let Some(event) = state.plants[plant_idx].grow_roots(target_pos) {
                    events.push(event);
                }

                // Deduire le carbone du sol
                if let Some(cell) = state.world.get_mut(&target_pos) {
                    let c = cell.carbon();
                    cell.set_carbon(c - carbon_cost);
                }
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
/// Les fixatrices d'azote (exudate_type = Nitrogen) beneficient d'un bonus :
/// elles creent de l'azote a partir de rien (fixation atmospherique).
fn action_exudates(state: &mut SimState, plant_id: u64, plant_idx: usize, exudate_rate: f32) {
    let exudate_type = state.plants[plant_idx].genetics().exudate_type();

    // Fixation atmospherique d'azote — la fixatrice GARDE le N pour elle
    // Elle ne l'injecte PAS dans le sol. Le N n'arrive dans le sol que par :
    // 1. Decomposition (a sa mort)
    // 2. Echange via lien mycorhizien (troc N contre energie)
    // Avantage competitif : elle pousse la ou les autres ne peuvent pas (sol sans N)
    if exudate_type == ExudateType::Nitrogen {
        let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
        let root_count = root_cells.len().max(1) as f32;
        let mut total_light = 0.0_f32;
        let mut total_carbon = 0.0_f32;
        for pos in &root_cells {
            if let Some(cell) = state.world.get(pos) {
                total_light += cell.light();
                total_carbon += cell.carbon();
            }
        }
        let avg_light = total_light / root_count;
        let avg_carbon = total_carbon / root_count;

        // Fixation = lumiere × carbone × efficacite
        let fixation_amount = avg_light * avg_carbon * state.config.nitrogen_fixation_rate;

        if fixation_amount > 0.001 {
            // Cout : consommer du carbone du sol (la fixation convertit C → N)
            // Pas de cout en energie — le gain est net positif
            let c_cost_per_cell = fixation_amount * 0.3 / root_count;
            let has_carbon = root_cells.iter().any(|pos| {
                state
                    .world
                    .get(pos)
                    .map(|c| c.carbon() > c_cost_per_cell)
                    .unwrap_or(false)
            });

            if has_carbon {
                for pos in &root_cells {
                    if let Some(cell) = state.world.get_mut(pos) {
                        let c = cell.carbon();
                        cell.set_carbon(c - c_cost_per_cell);
                    }
                }

                // La fixatrice GARDE le N pour sa croissance (pas de gain d'energie)
                // L'energie vient de la photosynthese comme tout le monde
                // Le profit vient du COMMERCE : vendre du N via symbiose en echange d'energie

                // Stats
                if let Some(stats) = state.find_stats_mut(plant_id) {
                    stats.exudates_emitted += fixation_amount;
                }
            }
        }
    }

    // Exsudation classique — les deux types exsudent normalement
    if exudate_rate <= 0.1 {
        return;
    }

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

/// e) Photosynthese batch — chaque couche de canopee filtre la lumiere.
/// Triees par taille (footprint) decroissante : la plus grande capte la lumiere pleine,
/// chaque couche traversee attenue la lumiere restante par canopy_light (transmittance).
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
            let _my_roots: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
            let other_idx = state.plants.iter().position(|p| p.id() == other_id);

            if let Some(oi) = other_idx {
                let other_roots: Vec<Pos> = state.plants[oi].roots().to_vec();

                // Troc asymetrique : N contre Energie
                // Si une des deux est fixatrice (Nitrogen), elle VEND du N
                // et recoit de l'energie en echange.
                let my_type = state.plants[plant_idx].genetics().exudate_type();
                let other_type = state.plants[oi].genetics().exudate_type();

                let mut total_exchanged = 0.0_f32;

                if my_type == ExudateType::Nitrogen && other_type == ExudateType::Carbon {
                    // Je suis fixatrice, l'autre est Carbon → je donne N, je recois energie
                    let n_amount = avg_generosity * transfer_rate * 5.0;
                    // Injecter N dans le sol sous les racines du partenaire
                    let n_per = n_amount / other_roots.len().max(1) as f32;
                    for pos in &other_roots {
                        if let Some(cell) = state.world.get_mut(pos) {
                            let n = cell.nitrogen();
                            cell.set_nitrogen(n + n_per);
                        }
                    }
                    // Recevoir de l'energie du partenaire
                    let energy_payment = n_amount * 10.0;
                    let available = state.plants[oi].energy().value() * 0.1; // max 10% de son energie
                    let actual_payment = energy_payment.min(available);
                    state.plants[oi].consume_energy(actual_payment);
                    state.plants[plant_idx].gain_energy(actual_payment);
                    total_exchanged = n_amount + actual_payment;
                } else if my_type == ExudateType::Carbon && other_type == ExudateType::Nitrogen {
                    // L'autre est fixatrice → elle donne N, je donne energie
                    let n_amount = avg_generosity * transfer_rate * 5.0;
                    // L'autre injecte N dans le sol sous mes racines
                    let my_roots_vec: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
                    let n_per = n_amount / my_roots_vec.len().max(1) as f32;
                    for pos in &my_roots_vec {
                        if let Some(cell) = state.world.get_mut(pos) {
                            let n = cell.nitrogen();
                            cell.set_nitrogen(n + n_per);
                        }
                    }
                    // Je paie en energie
                    let energy_payment = n_amount * 10.0;
                    let available = state.plants[plant_idx].energy().value() * 0.1;
                    let actual_payment = energy_payment.min(available);
                    state.plants[plant_idx].consume_energy(actual_payment);
                    state.plants[oi].gain_energy(actual_payment);
                    total_exchanged = n_amount + actual_payment;
                } else {
                    // Meme type → echange d'energie simple (du riche vers le pauvre)
                    let my_energy = state.plants[plant_idx].energy().value();
                    let other_energy = state.plants[oi].energy().value();
                    let energy_diff = my_energy - other_energy;
                    let energy_transfer = energy_diff * avg_generosity * 0.05;
                    if energy_transfer > 0.1 {
                        state.plants[plant_idx].consume_energy(energy_transfer);
                        state.plants[oi].gain_energy(energy_transfer);
                        total_exchanged = energy_transfer;
                    } else if energy_transfer < -0.1 {
                        state.plants[oi].consume_energy(-energy_transfer);
                        state.plants[plant_idx].gain_energy(-energy_transfer);
                        total_exchanged = -energy_transfer;
                    }
                }

                // Accumuler les echanges du tick
                state.metrics.tick_exchanges += total_exchanged;

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
    footprint_map: &HashMap<Pos, u64>,
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

    // h-bis) Consommation d'azote du sol proportionnelle a la biomasse
    // Les grosses plantes consomment plus de N, les petites quasi rien
    // Les fixatrices d'azote ne consomment pas de N du sol (elles le fabriquent)
    let is_fixer = state.plants[plant_idx].genetics().exudate_type() == ExudateType::Nitrogen;
    if !is_fixer {
        let n_consumption = biomass * 0.001;
        let footprint_cells: Vec<Pos> = state.plants[plant_idx].footprint().to_vec();
        let n_per_cell = n_consumption / footprint_cells.len().max(1) as f32;
        for pos in &footprint_cells {
            if let Some(cell) = state.world.get_mut(pos) {
                let n = cell.nitrogen();
                cell.set_nitrogen(n - n_per_cell);
            }
        }
    }

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
            if let Some(&occ_id) = footprint_map.get(pos) {
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
pub fn find_growth_target(
    plant: &dyn PlantEntity,
    dir_x: f32,
    dir_y: f32,
    grid_size: u16,
) -> Option<Pos> {
    let mut best: Option<(Pos, f32)> = None;

    for fp_pos in plant.footprint() {
        let neighbors = [
            (fp_pos.x.wrapping_sub(1), fp_pos.y, -1.0_f32, 0.0_f32),
            (fp_pos.x + 1, fp_pos.y, 1.0, 0.0),
            (fp_pos.x, fp_pos.y.wrapping_sub(1), 0.0, -1.0),
            (fp_pos.x, fp_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= grid_size || *ny >= grid_size {
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
pub fn find_canopy_growth_target(
    plant: &dyn PlantEntity,
    dir_x: f32,
    dir_y: f32,
    grid_size: u16,
) -> Option<Pos> {
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
            if *nx >= grid_size || *ny >= grid_size {
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
    let plant = state.plants[plant_idx].as_ref();
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
    let grid_size = state.world.size();
    find_root_growth_target(plant, closest_dir.0, closest_dir.1, grid_size)
}

/// Trouve la cellule voisine d'une racine la plus alignee avec la direction souhaitee.
fn find_root_growth_target(
    plant: &dyn PlantEntity,
    dir_x: f32,
    dir_y: f32,
    grid_size: u16,
) -> Option<Pos> {
    let mut best: Option<(Pos, f32)> = None;

    for root_pos in plant.roots() {
        let neighbors = [
            (root_pos.x.wrapping_sub(1), root_pos.y, -1.0_f32, 0.0_f32),
            (root_pos.x + 1, root_pos.y, 1.0, 0.0),
            (root_pos.x, root_pos.y.wrapping_sub(1), 0.0, -1.0),
            (root_pos.x, root_pos.y + 1, 0.0, 1.0),
        ];

        for (nx, ny, ndx, ndy) in &neighbors {
            if *nx >= grid_size || *ny >= grid_size {
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::application::config::SimConfig;
    use crate::application::evolution::{GenerationCounter, PlantStats, SeedBank};
    use crate::application::season::SeasonCycle;
    use crate::application::sim::SimState;
    use crate::domain::island::Island;
    use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
    use crate::domain::rng::test_utils::MockRng;
    use crate::domain::symbiosis::SymbiosisNetwork;
    use crate::domain::world::{World, DEFAULT_GRID_SIZE};

    /// Cree un SimState minimal avec une seule plante germee au centre de la grille.
    /// Le sol sous la plante est enrichi. Retourne (state, plant_id).
    fn test_state_with_plant(exudate_type: ExudateType) -> (SimState, u64) {
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let pos = Pos { x: 64, y: 64 };

        // Enrichir le sol sous la plante
        if let Some(cell) = world.get_mut(&pos) {
            cell.set_altitude(0.8); // au-dessus du sea_level
            cell.set_carbon(0.5);
            cell.set_nitrogen(0.3);
            cell.set_humidity(0.5);
            cell.set_light(1.0);
        }

        // Ile : tout est terre (altitude > 0 dans le world)
        let island = Island::from_world(&world, 0.0);

        let genetics = GeneticTraits::new(20, 0.5, exudate_type, 8, 10.0, 10.0);
        let lineage = Lineage::new(1, 0);
        let mut plant = Plant::new(1, pos, genetics, lineage);
        // Faire germer la plante
        plant.germinate();
        // Donner de l'energie
        plant.gain_energy(50.0);

        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        plant_stats.insert(1_u64, PlantStats::default());

        let state = SimState::from_raw(
            world,
            island,
            vec![Box::new(plant) as Box<dyn PlantEntity>],
            HashMap::new(), // pas de brains necessaires pour les tests unitaires
            SymbiosisNetwork::new(),
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            2, // next_plant_id
            0, // tick_count
            config,
        );

        (state, 1)
    }

    // --- Tests : cout de croissance proportionnel ---

    #[test]
    fn cout_croissance_proportionnel_a_la_taille() {
        // 1 cellule existante : base × 2/20 = base × 0.1
        assert!((growth_cost(5.0, 1) - 0.5).abs() < 1e-5);
        // 10 cellules : base × 11/20 = base × 0.55
        assert!((growth_cost(5.0, 10) - 2.75).abs() < 1e-5);
        // 20 cellules : base × 21/20 = base × 1.05
        assert!((growth_cost(5.0, 20) - 5.25).abs() < 1e-5);
    }

    #[test]
    fn premiere_cellule_coute_presque_rien() {
        // 0 cellules existantes : base × 1/20 = 0.25
        let cost = growth_cost(5.0, 0);
        assert!(
            cost < 0.3,
            "premiere cellule devrait etre quasi gratuite, got {cost}"
        );
    }

    // --- Test 1 : la fixation d'azote enrichit le sol ---

    #[test]
    fn la_fixation_ne_modifie_pas_lenergie() {
        let (mut state, plant_id) = test_state_with_plant(ExudateType::Nitrogen);
        let plant_idx = 0;

        let energy_before = state.plants[plant_idx].energy().value();

        // La fixation consomme du C du sol mais ne touche PAS l'energie
        // L'energie vient de la photosynthese, pas de la fixation
        action_exudates(&mut state, plant_id, plant_idx, 0.0);

        let energy_after = state.plants[plant_idx].energy().value();
        assert!(
            (energy_after - energy_before).abs() < 0.001,
            "la fixation ne devrait PAS modifier l'energie : avant={energy_before}, apres={energy_after}"
        );
    }

    // --- Test 2 : la fixation d'azote coute de l'energie ---

    #[test]
    fn la_fixation_azote_consomme_du_carbone_du_sol() {
        let (mut state, plant_id) = test_state_with_plant(ExudateType::Nitrogen);
        let plant_idx = 0;

        let root_pos = state.plants[plant_idx].roots()[0];
        let carbon_before = state
            .world
            .get(&root_pos)
            .map(|c| c.carbon())
            .unwrap_or(0.0);

        action_exudates(&mut state, plant_id, plant_idx, 0.0);

        let carbon_after = state
            .world
            .get(&root_pos)
            .map(|c| c.carbon())
            .unwrap_or(0.0);

        assert!(
            carbon_after < carbon_before,
            "la fixation devrait consommer du carbone du sol : avant={carbon_before}, apres={carbon_after}"
        );
    }

    // --- Test 3 : la fixation skip si pas assez d'energie ---

    #[test]
    fn la_fixation_skip_si_pas_assez_energie() {
        let (mut state, plant_id) = test_state_with_plant(ExudateType::Nitrogen);
        let plant_idx = 0;

        // Vider l'energie pour etre en dessous du cout de fixation
        let energy = state.plants[plant_idx].energy().value();
        state.plants[plant_idx].consume_energy(energy);
        state.plants[plant_idx].gain_energy(0.1); // < nitrogen_fixation_energy_cost (0.5)

        let root_pos = state.plants[plant_idx].roots()[0];
        let nitrogen_before = state
            .world
            .get(&root_pos)
            .map(|c| c.nitrogen())
            .unwrap_or(0.0);

        action_exudates(&mut state, plant_id, plant_idx, 0.0);

        let nitrogen_after = state
            .world
            .get(&root_pos)
            .map(|c| c.nitrogen())
            .unwrap_or(0.0);

        assert!(
            (nitrogen_after - nitrogen_before).abs() < f32::EPSILON,
            "pas de fixation si energie insuffisante : avant={nitrogen_before}, apres={nitrogen_after}"
        );
    }

    // --- Test 4 : les graines ne font aucune action ---

    #[test]
    fn les_graines_ne_font_aucune_action() {
        // Creer un SimState avec une plante en etat Seed (pas germee)
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let pos = Pos { x: 64, y: 64 };
        if let Some(cell) = world.get_mut(&pos) {
            cell.set_altitude(0.8);
            cell.set_carbon(0.5);
            cell.set_nitrogen(0.3);
            cell.set_humidity(0.5);
            cell.set_light(1.0);
        }
        let island = Island::from_world(&world, 0.0);
        let genetics = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let lineage = Lineage::new(1, 0);
        let plant = Plant::new(1, pos, genetics, lineage);
        // La plante reste en Seed (pas de germinate())

        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        plant_stats.insert(1_u64, PlantStats::default());

        let mut state = SimState::from_raw(
            world,
            island,
            vec![Box::new(plant) as Box<dyn PlantEntity>],
            HashMap::new(),
            SymbiosisNetwork::new(),
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            2,
            0,
            config,
        );

        // Mesurer l'etat avant
        let energy_before = state.plants[0].energy().value();
        let nitrogen_before = state.world.get(&pos).map(|c| c.nitrogen()).unwrap_or(0.0);
        let carbon_before = state.world.get(&pos).map(|c| c.carbon()).unwrap_or(0.0);

        // Simuler un appel a phase_actions avec des decisions pour la graine
        let decisions = vec![(1_u64, [1.0, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5])];
        let mut rng = MockRng::new(0.5, 0.1);
        let _events = phase_actions(&mut state, &decisions, &mut rng);

        // Verifier : aucun changement
        let energy_after = state.plants[0].energy().value();
        let nitrogen_after = state.world.get(&pos).map(|c| c.nitrogen()).unwrap_or(0.0);
        let carbon_after = state.world.get(&pos).map(|c| c.carbon()).unwrap_or(0.0);

        assert!(
            (energy_after - energy_before).abs() < f32::EPSILON,
            "l'energie d'une graine ne devrait pas changer : avant={energy_before}, apres={energy_after}"
        );
        assert!(
            (nitrogen_after - nitrogen_before).abs() < f32::EPSILON,
            "l'azote sous une graine ne devrait pas changer"
        );
        assert!(
            (carbon_after - carbon_before).abs() < f32::EPSILON,
            "le carbone sous une graine ne devrait pas changer"
        );
    }

    // --- Test 5 : l'invasion de graine est gratuite ---

    #[test]
    fn linvasion_de_graine_est_gratuite() {
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let pos_a = Pos { x: 64, y: 64 };
        let pos_b = Pos { x: 65, y: 64 }; // cellule adjacente

        // Enrichir le sol pour les deux cellules
        for pos in [&pos_a, &pos_b] {
            if let Some(cell) = world.get_mut(pos) {
                cell.set_altitude(0.8);
                cell.set_carbon(0.5);
                cell.set_nitrogen(0.3);
                cell.set_humidity(0.5);
                cell.set_light(1.0);
            }
        }

        let island = Island::from_world(&world, 0.0);

        // Plante A : germee avec de l'energie
        let genetics_a = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let mut plant_a = Plant::new(1, pos_a, genetics_a, Lineage::new(1, 0));
        plant_a.germinate();
        plant_a.gain_energy(50.0);

        // Plante B : graine sur la cellule adjacente
        let genetics_b = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let plant_b = Plant::new(2, pos_b, genetics_b, Lineage::new(2, 0));
        // B reste en Seed

        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        plant_stats.insert(1_u64, PlantStats::default());
        plant_stats.insert(2_u64, PlantStats::default());

        let mut state = SimState::from_raw(
            world,
            island,
            vec![Box::new(plant_a) as Box<dyn PlantEntity>, Box::new(plant_b)],
            HashMap::new(),
            SymbiosisNetwork::new(),
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            3,
            0,
            config,
        );

        let energy_before = state.plants[0].energy().value();

        // Croissance de A vers pos_b (direction +x) avec canopy_vs_roots = 0.5 (footprint)
        // grow_intensity = 1.0 (au-dessus du seuil), dir_x = 1.0, dir_y = 0.0
        let mut footprint_map: HashMap<Pos, u64> = HashMap::new();
        footprint_map.insert(pos_a, 1);
        footprint_map.insert(pos_b, 2);

        let modifiers = state.season_cycle.current_modifiers();
        let decisions = vec![
            (1_u64, [1.0, 1.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0]),
            (2_u64, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ];

        let events = action_growth(
            &mut state,
            &decisions,
            1,
            0,
            1.0, // grow_intensity
            1.0, // grow_dir_x (+x)
            0.0, // grow_dir_y
            0.5, // canopy_vs_roots → footprint
            &mut footprint_map,
            &modifiers,
        );

        let energy_after = state.plants[0].energy().value();

        // Verifier : A a pris la cellule
        assert!(
            state.plants[0].footprint().contains(&pos_b),
            "A devrait avoir pris la cellule de B"
        );
        // La graine a recu des degats egaux a sa vitalite — sa vitalite est a zero
        assert!(
            state.plants[1].vitality().value() == 0.0,
            "B (graine) devrait avoir une vitalite a zero, got {}",
            state.plants[1].vitality().value()
        );
        // Verifier : A n'a pas paye le cout d'invasion (pas de consume_energy pour invasion)
        assert!(
            (energy_after - energy_before).abs() < f32::EPSILON,
            "l'invasion de graine devrait etre gratuite : avant={energy_before}, apres={energy_after}"
        );
        // Verifier qu'un evenement d'invasion n'est PAS emis (c'est un ecrasement, pas une invasion)
        let has_invaded_event = events
            .iter()
            .any(|e| matches!(e, DomainEvent::Invaded { .. }));
        assert!(
            !has_invaded_event,
            "l'ecrasement de graine ne devrait pas emettre d'evenement Invaded"
        );
    }

    // --- Test 6 : l'ombre reduit la photosynthese ---

    #[test]
    fn lombre_reduit_la_photosynthese() {
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let shared_pos = Pos { x: 64, y: 64 };
        let pos_a_extra = Pos { x: 63, y: 64 }; // cellule en plus pour A (footprint plus grand)

        // Enrichir le sol
        for pos in [&shared_pos, &pos_a_extra] {
            if let Some(cell) = world.get_mut(pos) {
                cell.set_altitude(0.8);
                cell.set_carbon(0.5);
                cell.set_nitrogen(0.3);
                cell.set_humidity(0.5);
                cell.set_light(1.0);
            }
        }

        let island = Island::from_world(&world, 0.0);

        // Plante A : grande (footprint = 2 cellules), canopee sur shared_pos
        let genetics_a = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let mut plant_a = Plant::new(1, shared_pos, genetics_a, Lineage::new(1, 0));
        plant_a.germinate();
        plant_a.grow_footprint(pos_a_extra); // footprint = 2
                                             // La canopee inclut deja shared_pos (position initiale)

        // Plante B : petite (footprint = 1 cellule), canopee sur shared_pos aussi
        let pos_b = Pos { x: 65, y: 64 };
        if let Some(cell) = world.get_mut(&pos_b) {
            cell.set_altitude(0.8);
            cell.set_light(1.0);
        }
        let genetics_b = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let mut plant_b = Plant::new(2, pos_b, genetics_b, Lineage::new(2, 0));
        plant_b.germinate();
        // Ajouter shared_pos a la canopee de B
        plant_b.grow_canopy(shared_pos);

        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        plant_stats.insert(1_u64, PlantStats::default());
        plant_stats.insert(2_u64, PlantStats::default());

        let mut state = SimState::from_raw(
            world,
            island,
            vec![Box::new(plant_a) as Box<dyn PlantEntity>, Box::new(plant_b)],
            HashMap::new(),
            SymbiosisNetwork::new(),
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            3,
            0,
            config,
        );

        // Vider l'energie des deux plantes pour mesurer le gain net
        let energy_a = state.plants[0].energy().value();
        state.plants[0].consume_energy(energy_a);
        let energy_b = state.plants[1].energy().value();
        state.plants[1].consume_energy(energy_b);

        // Photosynthese batch (ombre calculee pour toutes les plantes)
        photosynthesis_batch(&mut state);

        let energy_b_shaded = state.plants[1].energy().value();

        // Calculer le gain attendu sans ombre
        // B a 2 cellules de canopee : pos_b (light=1.0) + shared_pos (light=1.0)
        // Sans ombre, le gain serait : 2 * 1.0 * photosynthesis_rate
        let full_light_gain = 2.0 * 1.0 * state.config.photosynthesis_rate;

        // Avec l'ombre, le gain sur shared_pos est canopy_light * photosynthesis_rate au lieu de 1.0 * photosynthesis_rate
        assert!(
            energy_b_shaded < full_light_gain,
            "la plante ombragee devrait gagner moins d'energie : gain={energy_b_shaded}, pleine lumiere={full_light_gain}"
        );
        assert!(
            energy_b_shaded > 0.0,
            "la plante ombragee devrait quand meme gagner de l'energie"
        );
    }

    // --- Test 7 : l'echange d'energie va du riche au pauvre ---

    #[test]
    fn lechange_energie_va_du_riche_au_pauvre() {
        let mut world = World::new(DEFAULT_GRID_SIZE);
        let pos_a = Pos { x: 64, y: 64 };
        let pos_b = Pos { x: 66, y: 64 };
        let shared_root = Pos { x: 65, y: 64 }; // racine partagee

        // Enrichir le sol
        for pos in [&pos_a, &pos_b, &shared_root] {
            if let Some(cell) = world.get_mut(pos) {
                cell.set_altitude(0.8);
                cell.set_carbon(0.5);
                cell.set_nitrogen(0.5);
                cell.set_humidity(0.5);
                cell.set_light(1.0);
            }
        }

        let island = Island::from_world(&world, 0.0);

        // Plante A : riche en energie
        let genetics_a = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 10.0);
        let mut plant_a = Plant::new(1, pos_a, genetics_a, Lineage::new(1, 0));
        plant_a.germinate();
        plant_a.gain_energy(50.0); // riche

        // Plante B : pauvre en energie
        let genetics_b = GeneticTraits::new(20, 0.5, ExudateType::Nitrogen, 8, 10.0, 10.0);
        let mut plant_b = Plant::new(2, pos_b, genetics_b, Lineage::new(2, 0));
        plant_b.germinate();
        // Vider l'energie de B
        let energy_b = plant_b.energy().value();
        plant_b.consume_energy(energy_b);
        plant_b.gain_energy(2.0); // pauvre

        // Donner une racine partagee a chacune
        plant_a.grow_roots(shared_root);
        plant_b.grow_roots(shared_root);

        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        plant_stats.insert(1_u64, PlantStats::default());
        plant_stats.insert(2_u64, PlantStats::default());

        // Creer le lien symbiotique
        let mut symbiosis = SymbiosisNetwork::new();
        symbiosis.create_link(1, 2);

        let mut state = SimState::from_raw(
            world,
            island,
            vec![Box::new(plant_a) as Box<dyn PlantEntity>, Box::new(plant_b)],
            HashMap::new(),
            symbiosis,
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            3,
            0,
            config,
        );

        let energy_a_before = state.plants[0].energy().value();
        let energy_b_before = state.plants[1].energy().value();

        // Root map pour la symbiose
        let mut root_map: HashMap<Pos, Vec<u64>> = HashMap::new();
        root_map.entry(pos_a).or_default().push(1);
        root_map.entry(shared_root).or_default().push(1);
        root_map.entry(shared_root).or_default().push(2);
        root_map.entry(pos_b).or_default().push(2);

        // Decisions : les deux veulent se connecter (connect_signal > 0.5)
        // et sont genereux (connect_generosity = 1.0)
        let decisions = vec![
            (1_u64, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]),
            (2_u64, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]),
        ];

        let _events = action_symbiosis(
            &mut state, &decisions, 1,   // plant_id
            0,   // plant_idx
            1.0, // connect_signal
            1.0, // connect_generosity
            &root_map,
        );

        let energy_a_after = state.plants[0].energy().value();
        let energy_b_after = state.plants[1].energy().value();

        // A (riche) devrait avoir perdu de l'energie
        assert!(
            energy_a_after < energy_a_before,
            "A (riche) devrait avoir perdu de l'energie : avant={energy_a_before}, apres={energy_a_after}"
        );
        // B (pauvre) devrait avoir gagne de l'energie
        assert!(
            energy_b_after > energy_b_before,
            "B (pauvre) devrait avoir gagne de l'energie : avant={energy_b_before}, apres={energy_b_after}"
        );
    }
}
