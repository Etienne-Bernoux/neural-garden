// Phase environnement — mise à jour du sol, climat, lumière et décomposition.

use super::sim::SimState;
use crate::domain::plant::{PlantState, Pos};
use crate::domain::world::GRID_SIZE;

/// Phase 1 : mise a jour de l'environnement (pluie, evaporation, regeneration, ombrage).
pub fn phase_environment(state: &mut SimState) {
    // Avancer la saison
    let _new_season = state.season_cycle.advance();

    let modifiers = state.season_cycle.current_modifiers();

    // Construire un ensemble des cellules occupees par une plante (canopee)
    let mut occupied = vec![false; GRID_SIZE as usize * GRID_SIZE as usize];
    for plant in &state.plants {
        if plant.is_dead() {
            continue;
        }
        for pos in plant.canopy() {
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
}

/// Decomposition progressive des plantes mortes — restitution des nutriments au sol.
pub fn phase_decomposition(state: &mut SimState) {
    let decomposition_ticks = state.config.decomposition_ticks;
    for plant in state.plants.iter_mut() {
        if plant.state() == PlantState::Decomposing {
            let (carbon, nitrogen) = plant.decompose_tick(decomposition_ticks);
            let canopy_cells: Vec<Pos> = plant.canopy().to_vec();
            for pos in &canopy_cells {
                if let Some(cell) = state.world.get_mut(pos) {
                    let c = cell.carbon();
                    cell.set_carbon(c + carbon);
                    let n = cell.nitrogen();
                    cell.set_nitrogen(n + nitrogen);
                }
            }
        }
    }
}
