// Steps pour la decomposition des plantes mortes.

use cucumber::{given, then, when};

use garden_core::application::config::SimConfig;
use garden_core::application::evolution::PlantStats;
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::plant::{Lineage, Plant, PlantState};

use crate::GardenWorld;

/// Helper : cree un SimState avec sol pauvre, ajoute une plante morte en decomposition.
/// Capture le carbone du sol avant decomposition.
fn setup_decomposing_plant(world: &mut GardenWorld, carbon: f32, nitrogen: f32) {
    let config = SimConfig {
        initial_population: 0,
        ..SimConfig::default()
    };
    let mut state = SimState::with_config(0.3, config, &mut world.rng);

    // Configurer le sol
    let land_cells: Vec<_> = state.island.land_cells().to_vec();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    for pos in &land_cells {
        if let Some(cell) = state.world.get_mut(pos) {
            cell.set_carbon(carbon);
            cell.set_nitrogen(nitrogen);
            cell.set_humidity(0.5);
        }
    }

    let pos = land_cells[0];

    // Creer une plante, la faire germer, vieillir, puis la tuer
    let genome = state.seed_bank.produce_seed(&mut world.rng);
    let lineage = Lineage::new(0, 0);
    let mut plant = Plant::new(state.next_plant_id, pos, genome.traits, lineage);
    plant.germinate();

    // Vieillir la plante pour que nitrogen_to_release > 0
    for _ in 0..50 {
        plant.tick();
    }

    // Tuer la plante
    plant.damage(1000.0);
    let _ = plant.update_state();
    plant.start_decomposition(state.config.decomposition_ticks);

    state.brains.insert(state.next_plant_id, genome.brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(plant);
    state.next_plant_id += 1;

    // Capturer le carbone avant
    world.captured_carbon_before = state.world.get(&pos).map(|c| c.carbon()).unwrap_or(0.0);

    world.state = Some(state);
}

// --- Given ---

#[given(regex = r"^une île avec du sol pauvre$")]
async fn ile_sol_pauvre(world: &mut GardenWorld) {
    // Ce step ne cree que le sol pauvre — la plante morte est ajoutee au step suivant.
    // On stocke les valeurs pour le helper.
    let config = SimConfig {
        initial_population: 0,
        ..SimConfig::default()
    };
    let mut state = SimState::with_config(0.3, config, &mut world.rng);

    let land_cells: Vec<_> = state.island.land_cells().to_vec();
    for pos in &land_cells {
        if let Some(cell) = state.world.get_mut(pos) {
            cell.set_carbon(0.1);
            cell.set_nitrogen(0.1);
            cell.set_humidity(0.5);
        }
    }

    world.state = Some(state);
}

#[given(regex = r"^une plante morte en décomposition sur ce sol$")]
async fn plante_morte_decomposition_sur_sol(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state doit exister");
    let land_cells: Vec<_> = state.island.land_cells().to_vec();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    let pos = land_cells[0];

    let genome = state.seed_bank.produce_seed(&mut world.rng);
    let lineage = Lineage::new(0, 0);
    let mut plant = Plant::new(state.next_plant_id, pos, genome.traits, lineage);
    plant.germinate();

    for _ in 0..50 {
        plant.tick();
    }

    plant.damage(1000.0);
    let _ = plant.update_state();
    plant.start_decomposition(state.config.decomposition_ticks);

    state.brains.insert(state.next_plant_id, genome.brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(plant);
    state.next_plant_id += 1;

    // Capturer le carbone du sol avant decomposition
    world.captured_carbon_before = state.world.get(&pos).map(|c| c.carbon()).unwrap_or(0.0);
}

#[given(regex = r"^une plante morte en décomposition$")]
async fn plante_morte_decomposition(world: &mut GardenWorld) {
    setup_decomposing_plant(world, 0.1, 0.1);
}

// --- When ---

#[when(regex = r"^la simulation avance de (\d+) ticks supplémentaires$")]
async fn simulation_avance_supplementaire(world: &mut GardenWorld, n: u32) {
    let state = world.state.as_mut().expect("state doit exister");
    for _ in 0..n {
        run_tick(state, &mut world.rng);
    }
}

// --- Then ---

#[then(regex = r"^le sol sous la plante est plus riche qu'avant$")]
async fn sol_plus_riche(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    let land_cells = state.island.land_cells();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    let pos = &land_cells[0];

    let carbon_after = state.world.get(pos).map(|c| c.carbon()).unwrap_or(0.0);
    assert!(
        carbon_after > world.captured_carbon_before,
        "le sol devrait etre plus riche : avant={}, apres={}",
        world.captured_carbon_before,
        carbon_after
    );
}

#[then(regex = r"^la plante est toujours en décomposition$")]
async fn plante_toujours_decomposition(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    // Chercher la plante en decomposition
    let decomposing = state
        .plants
        .iter()
        .find(|p| p.state() == PlantState::Decomposing);
    assert!(
        decomposing.is_some(),
        "il devrait y avoir une plante en decomposition"
    );
}

#[then(regex = r"^la plante est complètement décomposée$")]
async fn plante_completement_decomposee(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    // La plante est soit is_fully_decomposed, soit retiree par le GC
    let still_decomposing = state
        .plants
        .iter()
        .any(|p| p.state() == PlantState::Decomposing && !p.is_fully_decomposed());
    assert!(
        !still_decomposing,
        "il ne devrait plus y avoir de plante en cours de decomposition"
    );
}
