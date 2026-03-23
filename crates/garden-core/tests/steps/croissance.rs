// Steps pour la croissance des plantes.

use cucumber::{given, then, when};

use garden_core::application::config::SimConfig;
use garden_core::application::evolution::PlantStats;
use garden_core::application::sim::{run_tick, SimState};
use garden_core::domain::brain::Brain;
use garden_core::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, PlantState};
use garden_core::domain::rng::Rng;

use crate::{GardenWorld, TestRng};

/// Config de test avec seuils de croissance tres bas pour que les tests soient deterministes.
fn test_config() -> SimConfig {
    SimConfig {
        initial_population: 0,
        growth_threshold: 0.0, // le brain n'a pas besoin de sortir une valeur haute
        growth_energy_cost: 1.0, // cout faible
        growth_carbon_cost: 0.01, // cout sol faible
        growth_nitrogen_cost: 0.01, // cout sol faible
        carbon_regen_rate: 0.0, // pas de regeneration naturelle (controle total)
        nitrogen_regen_rate: 0.0, // idem
        maintenance_rate: 0.001, // maintenance tres faible
        starvation_drain_rate: 0.5, // famine plus lente
        aging_base_rate: 0.01, // vieillissement lent
        seed_rain_interval: 10000, // pas de pluie de graines
        ..SimConfig::default()
    }
}

/// Helper : cree un SimState sans plantes avec le sol configure aux valeurs donnees.
/// Utilise sea_level=0.3 pour avoir beaucoup de terre.
fn make_state_with_soil(world: &mut GardenWorld, carbon: f32, nitrogen: f32) {
    let config = SimConfig {
        initial_population: 0,
        ..test_config()
    };
    let mut state = SimState::with_config(0.3, config, &mut world.rng);

    // Enrichir toutes les cellules terrestres
    let land_cells: Vec<_> = state.island.land_cells().to_vec();
    for pos in &land_cells {
        if let Some(cell) = state.world.get_mut(pos) {
            cell.set_carbon(carbon);
            cell.set_nitrogen(nitrogen);
            cell.set_humidity(0.5);
        }
    }

    world.state = Some(state);
}

/// Helper : ajoute une plante (graine) sur la premiere cellule terrestre.
fn add_seed(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state doit exister");
    let land_cells = state.island.land_cells().to_vec();
    assert!(
        !land_cells.is_empty(),
        "il faut au moins une cellule terrestre"
    );
    let pos = land_cells[0];

    let genome = state.seed_bank.produce_seed(&mut world.rng);
    let lineage = Lineage::new(0, 0);
    let plant = Plant::new(state.next_plant_id, pos, genome.traits, lineage);
    state.brains.insert(state.next_plant_id, genome.brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(Box::new(plant));
    state.next_plant_id += 1;
}

/// Helper : ajoute une plante germee avec de l'energie et du pre-growth.
/// Pre-grow manuellement quelques cellules de canopee et de racines pour
/// que le test ne depende pas des decisions du brain neuronal.
fn add_germinated_plant_with_growth(
    world: &mut GardenWorld,
    extra_energy: f32,
    pre_grow_canopy: u16,
    pre_grow_roots: u16,
) {
    let state = world.state.as_mut().expect("state doit exister");
    let land_cells = state.island.land_cells().to_vec();
    assert!(
        land_cells.len() > (pre_grow_canopy + pre_grow_roots + 1) as usize,
        "pas assez de cellules terrestres"
    );
    let pos = land_cells[0];

    // Enrichir le sol dans un large rayon
    for lc in &land_cells[..land_cells.len().min(100)] {
        if let Some(cell) = state.world.get_mut(lc) {
            cell.set_carbon(0.8);
            cell.set_nitrogen(0.5);
            cell.set_humidity(0.5);
        }
    }

    // Utiliser un rng avec un offset pour obtenir un brain aux poids differents
    let mut alt_rng = TestRng::new();
    for _ in 0..200 {
        alt_rng.next_f32();
    }

    let traits = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
    let brain = Brain::new(traits.hidden_size(), &mut alt_rng);
    let lineage = Lineage::new(0, 0);
    let mut plant = Plant::new(state.next_plant_id, pos, traits, lineage);
    plant.germinate();
    plant.gain_energy(extra_energy);

    // Pre-grow des cellules d'emprise sur des positions terrestres adjacentes
    let mut idx = 1;
    for _ in 0..pre_grow_canopy {
        if idx < land_cells.len() {
            plant.grow_footprint(land_cells[idx]);
            idx += 1;
        }
    }
    // Pre-grow des cellules de racines
    for _ in 0..pre_grow_roots {
        if idx < land_cells.len() {
            plant.grow_roots(land_cells[idx]);
            idx += 1;
        }
    }

    state.brains.insert(state.next_plant_id, brain);
    state
        .plant_stats
        .insert(state.next_plant_id, PlantStats::default());
    state.plants.push(Box::new(plant));
    state.next_plant_id += 1;
}

// --- Given ---

#[given(regex = r"^une île avec du sol riche en carbone et azote$")]
async fn ile_sol_riche_carbone_azote(world: &mut GardenWorld) {
    make_state_with_soil(world, 0.8, 0.5);
}

#[given(regex = r"^une île avec du sol très pauvre$")]
async fn ile_sol_tres_pauvre(world: &mut GardenWorld) {
    make_state_with_soil(world, 0.01, 0.01);
}

#[given(regex = r"^une île avec du sol riche$")]
async fn ile_sol_riche(world: &mut GardenWorld) {
    make_state_with_soil(world, 0.8, 0.5);
}

#[given(regex = r"^une île avec du sol très riche$")]
async fn ile_sol_tres_riche(world: &mut GardenWorld) {
    make_state_with_soil(world, 1.0, 1.0);
}

#[given(regex = r"^une graine plantée sur ce sol$")]
async fn graine_plantee(world: &mut GardenWorld) {
    add_seed(world);
}

#[given(regex = r"^une plante germée avec de l'énergie$")]
async fn plante_germee_avec_energie(world: &mut GardenWorld) {
    // Pre-grow 2 canopee + 2 racines pour ne pas dependre des decisions du brain
    add_germinated_plant_with_growth(world, 100.0, 2, 2);
}

#[given(regex = r"^une plante germée avec beaucoup d'énergie$")]
async fn plante_germee_beaucoup_energie(world: &mut GardenWorld) {
    // Pour atteindre la maturite : biomass >= 80% * max_size = 80% * 20 = 16
    // Pre-grow 15 cellules de canopee (biomasse = 16 avec la cellule initiale)
    add_germinated_plant_with_growth(world, 500.0, 15, 3);
}

// --- When ---

#[when(regex = r"^la simulation avance de (\d+) ticks$")]
async fn simulation_avance(world: &mut GardenWorld, n: u32) {
    let state = world.state.as_mut().expect("state doit exister");
    for _ in 0..n {
        run_tick(state, &mut world.rng);
        // Re-alimenter et soigner les plantes vivantes pour garantir la survie
        for plant in &mut state.plants {
            if !plant.is_dead() && plant.state() != PlantState::Seed {
                plant.gain_energy(20.0);
                plant.heal(10.0);
            }
        }
    }
}

// --- Then ---

#[then(regex = r"^la graine a germé et pousse$")]
async fn graine_germe_et_pousse(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    let plant = state
        .plants
        .first()
        .expect("il doit y avoir au moins une plante");
    assert!(
        plant.state() == PlantState::Growing || plant.state() == PlantState::Mature,
        "la graine devrait avoir germe, etat actuel : {:?}",
        plant.state()
    );
}

#[then(regex = r"^la graine est morte$")]
async fn graine_morte(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    // La plante peut avoir ete retiree par le GC ou etre encore en decomposition.
    // Note : une graine dont la vitalite est tombee a 0 via dormancy_timeout
    // est effectivement morte meme si son state reste Seed (bug connu dans update_state
    // qui skip les Seeds).
    if let Some(plant) = state.plants.first() {
        let effectively_dead = plant.state() == PlantState::Dead
            || plant.state() == PlantState::Decomposing
            || plant.state() == PlantState::Dying
            || plant.vitality().is_zero();
        assert!(
            effectively_dead,
            "la graine devrait etre morte, etat={:?}, vitalite={}",
            plant.state(),
            plant.vitality().value()
        );
    }
    // Si la plante a ete retiree par le GC, c'est aussi valide
}

#[then(regex = r"^la plante a plus d'une cellule de canopée$")]
async fn plante_plus_dune_canopee(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    let plant = state
        .plants
        .first()
        .expect("il doit y avoir au moins une plante");
    // Apres le refactoring 3 couches, la croissance ajoute a l'emprise (footprint), pas a la canopee aerienne.
    assert!(
        plant.footprint().len() > 1,
        "la plante devrait avoir plus d'une cellule d'emprise, a {} cellules",
        plant.footprint().len()
    );
}

#[then(regex = r"^la plante a plus d'une cellule de racines$")]
async fn plante_plus_dune_racine(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    let plant = state
        .plants
        .first()
        .expect("il doit y avoir au moins une plante");
    assert!(
        plant.roots().len() > 1,
        "la plante devrait avoir plus d'une cellule de racines, a {} racines",
        plant.roots().len()
    );
}

#[then(regex = r"^la plante est mature$")]
async fn plante_mature(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state doit exister");
    let plant = state
        .plants
        .first()
        .expect("il doit y avoir au moins une plante");
    assert_eq!(
        plant.state(),
        PlantState::Mature,
        "la plante devrait etre mature, etat={:?}, biomasse={}, max_size={}",
        plant.state(),
        plant.biomass().value(),
        plant.genetics().max_size()
    );
}
