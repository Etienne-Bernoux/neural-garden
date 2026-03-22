// Phase environnement — mise à jour du sol, climat, lumière et décomposition.

use super::sim::SimState;
use crate::domain::plant::{PlantState, Pos};
use crate::domain::world::GRID_SIZE;

/// Phase 1 : mise a jour de l'environnement (pluie, evaporation, regeneration, ombrage).
/// Retourne Some(Season) si la saison a change pendant ce tick.
pub fn phase_environment(state: &mut SimState) -> Option<super::season::Season> {
    // Avancer la saison
    let new_season = state.season_cycle.advance();

    let modifiers = state.season_cycle.current_modifiers();

    // Construire un ensemble des cellules occupees par l'emprise physique (footprint)
    // L'ombrage est lie a l'emprise au sol pour l'instant (T4 ajoutera l'ombre de la canopee aerienne)
    let mut occupied = vec![false; GRID_SIZE as usize * GRID_SIZE as usize];
    for plant in &state.plants {
        if plant.is_dead() {
            continue;
        }
        for pos in plant.footprint() {
            if pos.x < GRID_SIZE && pos.y < GRID_SIZE {
                occupied[pos.y as usize * GRID_SIZE as usize + pos.x as usize] = true;
            }
        }
    }

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let pos = Pos { x, y };
            let is_land = state.island.is_land(&pos);
            let idx = y as usize * GRID_SIZE as usize + x as usize;
            let under_canopy = occupied[idx];

            if let Some(cell) = state.world.get_mut(&pos) {
                // Pluie : cellules terrestres
                if is_land {
                    let h = cell.humidity();
                    cell.set_humidity(h + state.config.rain_rate * modifiers.rain);
                }

                // Evaporation
                let h = cell.humidity();
                if under_canopy {
                    cell.set_humidity(h - state.config.evaporation_canopy_rate);
                } else {
                    cell.set_humidity(h - state.config.evaporation_rate);
                }

                // Regeneration sol : cellules terrestres
                if is_land {
                    let c = cell.carbon();
                    cell.set_carbon(c + state.config.carbon_regen_rate * modifiers.soil_regen);
                    let n = cell.nitrogen();
                    cell.set_nitrogen(n + state.config.nitrogen_regen_rate * modifiers.soil_regen);
                }

                // Diffusion exsudats : decroissance
                let ex = cell.exudates();
                cell.set_exudates(ex * state.config.exudate_decay);

                // Ombrage
                if !is_land {
                    cell.set_light(0.0);
                } else if under_canopy {
                    cell.set_light(state.config.canopy_light * modifiers.light);
                } else {
                    cell.set_light(modifiers.light);
                }
            }
        }
    }

    new_season
}

/// Decomposition progressive des plantes mortes — restitution des nutriments au sol.
pub fn phase_decomposition(state: &mut SimState) {
    let decomposition_ticks = state.config.decomposition_ticks;

    // Collecter les mises a jour de stats (on ne peut pas acceder aux stats pendant l'iteration mutable)
    let mut decomposition_updates: Vec<(u64, f32, f32)> = Vec::new();

    for plant in state.plants.iter_mut() {
        if plant.state() == PlantState::Decomposing {
            let (carbon, nitrogen) = plant.decompose_tick(decomposition_ticks);
            let footprint_cells: Vec<Pos> = plant.footprint().to_vec();
            for pos in &footprint_cells {
                if let Some(cell) = state.world.get_mut(pos) {
                    let c = cell.carbon();
                    cell.set_carbon(c + carbon);
                    let n = cell.nitrogen();
                    cell.set_nitrogen(n + nitrogen);
                }
            }
            decomposition_updates.push((plant.id(), carbon, nitrogen));
        }
    }

    // Mettre a jour soil_enriched apres la boucle
    for (plant_id, total_carbon, total_nitrogen) in decomposition_updates {
        if let Some(stats) = state.find_stats_mut(plant_id) {
            stats.soil_enriched += total_carbon + total_nitrogen;
        }
    }
}
