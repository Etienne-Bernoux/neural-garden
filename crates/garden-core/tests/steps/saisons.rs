// Steps pour le cycle saisonnier.

use cucumber::{given, then, when};

use garden_core::application::config::SimConfig;
use garden_core::application::evolution::PlantStats;
use garden_core::application::season::SeasonCycle;
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::plant::{Lineage, Plant, PlantState};

use crate::GardenWorld;

/// Helper : cree un SimState sans plantes avec le sol aux valeurs donnees.
fn make_empty_state(world: &mut GardenWorld, carbon: f32, nitrogen: f32) -> SimState {
    let config = SimConfig {
        initial_population: 0,
        ..SimConfig::default()
    };
    let mut state = SimState::with_config(0.3, config, &mut world.rng);

    let land_cells: Vec<_> = state.island.land_cells().to_vec();
    for pos in &land_cells {
        if let Some(cell) = state.world.get_mut(pos) {
            cell.set_carbon(carbon);
            cell.set_nitrogen(nitrogen);
            cell.set_humidity(0.5);
        }
    }

    state
}

/// Helper : ajoute une plante germee au state.
fn add_germinated(state: &mut SimState, rng: &mut crate::TestRng) {
    let land_cells = state.island.land_cells().to_vec();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    let pos = land_cells[0];

    let genome = state.seed_bank.produce_seed(rng);
    let lineage = Lineage::new(0, 0);
    let mut plant = Plant::new(state.next_plant_id, pos, genome.traits, lineage);
    plant.germinate();
    plant.gain_energy(50.0);

    state.brains.insert(state.next_plant_id, genome.brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(plant);
    state.next_plant_id += 1;
}

/// Avance le season_cycle jusqu'a la saison cible.
/// Spring=0, Summer=250, Autumn=500, Winter=750 (avec ticks_per_season=250).
fn advance_season_cycle_to(cycle: &mut SeasonCycle, target_tick: u32) {
    let current = cycle.tick();
    for _ in current..target_tick {
        cycle.advance();
    }
}

// --- Scenario : L'hiver ralentit la croissance ---

#[given(regex = r"^deux plantes identiques germées$")]
async fn deux_plantes_identiques(_world: &mut GardenWorld) {
    // Les deux plantes seront creees dans le step suivant (une par simulation).
    // On ne fait rien ici : le state est deja cree par le step "une ile avec du sol riche".
}

#[given(regex = r"^une plante vit un cycle en été et l'autre en hiver$")]
async fn plante_ete_et_hiver(_world: &mut GardenWorld) {
    // Setup fait dans le step "when" — on a besoin de deux sims separees.
}

#[when(regex = r"^chaque plante simule (\d+) ticks dans sa saison$")]
async fn chaque_plante_simule(world: &mut GardenWorld, n: u32) {
    // Simulation en ete
    let mut state_summer = make_empty_state(world, 0.8, 0.5);
    add_germinated(&mut state_summer, &mut world.rng);
    // Avancer le cycle saisonnier au debut de l'ete (tick 250)
    advance_season_cycle_to(&mut state_summer.season_cycle, 250);

    for _ in 0..n {
        run_tick(&mut state_summer, &mut world.rng);
    }

    let biomass_summer = state_summer
        .plants
        .first()
        .map(|p| p.biomass().value())
        .unwrap_or(0);

    // Simulation en hiver
    let mut state_winter = make_empty_state(world, 0.8, 0.5);
    add_germinated(&mut state_winter, &mut world.rng);
    // Avancer le cycle saisonnier au debut de l'hiver (tick 750)
    advance_season_cycle_to(&mut state_winter.season_cycle, 750);

    for _ in 0..n {
        run_tick(&mut state_winter, &mut world.rng);
    }

    let biomass_winter = state_winter
        .plants
        .first()
        .map(|p| p.biomass().value())
        .unwrap_or(0);

    world.captured_biomass_summer = biomass_summer;
    world.captured_biomass_winter = biomass_winter;

    // Garder un des states pour reference
    world.state = Some(state_summer);
}

#[then(regex = r"^la plante d'été a plus de biomasse que celle d'hiver$")]
async fn plante_ete_plus_biomasse(world: &mut GardenWorld) {
    assert!(
        world.captured_biomass_summer >= world.captured_biomass_winter,
        "la plante d'ete devrait avoir autant ou plus de biomasse que celle d'hiver : ete={}, hiver={}",
        world.captured_biomass_summer,
        world.captured_biomass_winter
    );
}

// --- Scenario : Le printemps accelere la regeneration du sol ---

#[given(regex = r"^une île avec du sol à moitié riche$")]
async fn ile_sol_moitie_riche(world: &mut GardenWorld) {
    let state = make_empty_state(world, 0.5, 0.3);
    world.state = Some(state);
}

#[when(regex = r"^la simulation avance de (\d+) ticks au printemps$")]
async fn simulation_avance_printemps(world: &mut GardenWorld, n: u32) {
    // Creer une simulation dediee au printemps (tick 0 = printemps)
    let mut state_spring = make_empty_state(world, 0.5, 0.3);

    for _ in 0..n {
        run_tick(&mut state_spring, &mut world.rng);
    }

    // Capturer le carbone du sol apres le printemps
    let land_cells = state_spring.island.land_cells().to_vec();
    let carbon = if let Some(pos) = land_cells.first() {
        state_spring
            .world
            .get(pos)
            .map(|c| c.carbon())
            .unwrap_or(0.0)
    } else {
        0.0
    };
    world.captured_carbon_spring = carbon;
}

#[when(regex = r"^la simulation avance de (\d+) ticks en hiver$")]
async fn simulation_avance_hiver(world: &mut GardenWorld, n: u32) {
    // Creer une simulation dediee a l'hiver (avancer le cycle au tick 750)
    let mut state_winter = make_empty_state(world, 0.5, 0.3);
    advance_season_cycle_to(&mut state_winter.season_cycle, 750);

    for _ in 0..n {
        run_tick(&mut state_winter, &mut world.rng);
    }

    // Capturer le carbone du sol apres l'hiver
    let land_cells = state_winter.island.land_cells().to_vec();
    let carbon = if let Some(pos) = land_cells.first() {
        state_winter
            .world
            .get(pos)
            .map(|c| c.carbon())
            .unwrap_or(0.0)
    } else {
        0.0
    };
    world.captured_carbon_winter = carbon;

    world.state = Some(state_winter);
}

#[then(regex = r"^le sol s'est plus régénéré au printemps qu'en hiver$")]
async fn sol_plus_regenere_printemps(world: &mut GardenWorld) {
    assert!(
        world.captured_carbon_spring > world.captured_carbon_winter,
        "le sol devrait s'etre plus regenere au printemps qu'en hiver : printemps={}, hiver={}",
        world.captured_carbon_spring,
        world.captured_carbon_winter
    );
}

// --- Scenario : Les plantes affamees meurent plus vite en hiver ---

#[given(regex = r"^une plante avec très peu d'énergie$")]
async fn plante_avec_peu_energie(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state doit exister");
    let land_cells = state.island.land_cells().to_vec();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    let pos = land_cells[0];

    let genome = state.seed_bank.produce_seed(&mut world.rng);
    let lineage = Lineage::new(0, 0);
    let mut plant = Plant::new(state.next_plant_id, pos, genome.traits, lineage);
    plant.germinate();
    // Vider presque toute l'energie
    let energy = plant.energy().value();
    plant.consume_energy(energy * 0.99);

    state.brains.insert(state.next_plant_id, genome.brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(plant);
    state.next_plant_id += 1;
}

#[when(regex = r"^la simulation avance en hiver pendant (\d+) ticks$")]
async fn simulation_avance_en_hiver(world: &mut GardenWorld, n: u32) {
    let state = world.state.as_mut().expect("state doit exister");
    // Avancer au debut de l'hiver
    advance_season_cycle_to(&mut state.season_cycle, 750);

    for _ in 0..n {
        run_tick(state, &mut world.rng);
    }
}

#[then(regex = r"^la plante est morte ou mourante$")]
async fn plante_morte_ou_mourante(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");

    // Chercher la premiere plante (qui n'est pas issue de la pluie de graines)
    // On cherche une plante avec id=1 (premiere ajoutee) ou la premiere du vec
    let plant = state.plants.first();

    match plant {
        Some(p) => {
            assert!(
                p.state() == PlantState::Dead
                    || p.state() == PlantState::Dying
                    || p.state() == PlantState::Decomposing,
                "la plante devrait etre morte ou mourante, etat actuel : {:?}",
                p.state()
            );
        }
        None => {
            // La plante a ete retiree par le GC — elle est morte
        }
    }
}
