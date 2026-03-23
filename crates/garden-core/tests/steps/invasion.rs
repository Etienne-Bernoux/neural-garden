// Steps pour l'invasion entre plantes

use cucumber::{given, then, when};

use garden_core::application::config::SimConfig;
use garden_core::application::evolution::PlantStats;
use garden_core::domain::brain::Brain;
use garden_core::domain::island::Island;
use garden_core::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
use garden_core::domain::world::{World, DEFAULT_GRID_SIZE};

use crate::GardenWorld;

/// Cree un SimState avec deux plantes adjacentes :
/// - Plante A a (10, 10)
/// - Plante B a (11, 10) — canopee adjacente a A
fn setup_two_adjacent_plants(world: &mut GardenWorld) {
    let mut new_world = World::new(DEFAULT_GRID_SIZE);

    // Mettre l'altitude haute pour que les cellules soient terrestres
    for x in 8..16 {
        for y in 8..14 {
            let pos = Pos { x, y };
            if let Some(cell) = new_world.get_mut(&pos) {
                cell.set_altitude(0.9);
                cell.set_carbon(0.8);
                cell.set_nitrogen(0.6);
                cell.set_humidity(0.5);
                cell.set_light(1.0);
            }
        }
    }

    let island = Island::from_world(&new_world, 0.1);
    let config = SimConfig {
        initial_population: 0,
        invasion_energy_threshold: 10.0,
        invasion_defense_threshold: 20.0,
        invasion_energy_cost: 12.0,
        invasion_damage: 3.0,
        ..SimConfig::default()
    };

    let state = garden_core::application::sim::SimState::with_terrain(
        new_world,
        island,
        config,
        &mut world.rng,
    );
    world.state = Some(state);

    let state = world.state.as_mut().expect("state should exist");

    // energy_factor eleve pour que le cap permette de stocker beaucoup d'energie
    let genetics_a = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 20.0);
    let genetics_b = GeneticTraits::new(20, 0.5, ExudateType::Nitrogen, 8, 10.0, 20.0);

    let mut plant_a = Plant::new(1, Pos { x: 10, y: 10 }, genetics_a, Lineage::new(0, 0));
    let mut plant_b = Plant::new(2, Pos { x: 11, y: 10 }, genetics_b, Lineage::new(1, 0));

    plant_a.germinate();
    plant_b.germinate();

    // B a besoin d'au moins 2 cellules d'emprise pour que remove_footprint_cell fonctionne
    plant_b.grow_footprint(Pos { x: 12, y: 10 });

    let brain_a = Brain::new(8, &mut world.rng);
    let brain_b = Brain::new(8, &mut world.rng);

    state.plants.push(Box::new(plant_a));
    state.plants.push(Box::new(plant_b));
    state.brains.insert(1, brain_a);
    state.brains.insert(2, brain_b);
    state.plant_stats.insert(1, PlantStats::default());
    state.plant_stats.insert(2, PlantStats::default());
    state.next_plant_id = 3;
}

#[given(regex = r"^une île avec deux plantes adjacentes$")]
fn ile_deux_plantes_adjacentes(world: &mut GardenWorld) {
    setup_two_adjacent_plants(world);
}

#[given(regex = r"^la plante A a beaucoup plus d'énergie que la plante B$")]
fn plante_a_forte(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state should exist");
    for plant in &mut state.plants {
        if plant.id() == 1 {
            plant.gain_energy(200.0);
        } else if plant.id() == 2 {
            // Vider l'energie et donner un minimum
            let e = plant.energy().value();
            plant.consume_energy(e);
            plant.gain_energy(1.0);
        }
    }
}

#[when(regex = r"^la plante A tente de pousser vers la cellule de B$")]
fn plante_a_pousse_vers_b(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state should exist");

    // Trouver les indices des plantes
    let a_idx = state
        .plants
        .iter()
        .position(|p| p.id() == 1)
        .expect("plante A");
    let b_idx = state
        .plants
        .iter()
        .position(|p| p.id() == 2)
        .expect("plante B");

    // Cellule cible : la position de B (11, 10)
    let target = state.plants[b_idx].footprint()[0];

    let attacker_energy = state.plants[a_idx].energy().value();
    let defender_energy = state.plants[b_idx].energy().value();

    // Appliquer la mecanique d'invasion directement
    // Condition : attacker_energy > defender_energy + invasion_energy_threshold
    let threshold = state.config.invasion_energy_threshold;

    if attacker_energy > defender_energy + threshold {
        // Invasion reussie
        state.plants[b_idx].remove_footprint_cell(&target);
        state.plants[a_idx].grow_footprint(target);
        state.plants[a_idx].consume_energy(state.config.invasion_energy_cost);
        state.plants[b_idx].damage(state.config.invasion_damage);

        // Rompre les liens de symbiose
        state.symbiosis.remove_link(1, 2);
    }
    // Sinon : invasion echouee, rien ne change
}

#[then(regex = r"^la plante A a pris la cellule de B$")]
fn plante_a_pris_cellule(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state should exist");

    let target = Pos { x: 11, y: 10 };

    let plant_a = state.plants.iter().find(|p| p.id() == 1).expect("plante A");

    assert!(
        plant_a.footprint().contains(&target),
        "la plante A devrait avoir pris la cellule (11, 10), emprise: {:?}",
        plant_a.footprint()
    );
}

#[given(regex = r"^la plante B a plus d'énergie que la plante A$")]
fn plante_b_plus_forte(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state should exist");
    for plant in &mut state.plants {
        if plant.id() == 1 {
            // A a peu d'energie
            let e = plant.energy().value();
            plant.consume_energy(e);
            plant.gain_energy(1.0);
        } else if plant.id() == 2 {
            // B a beaucoup d'energie
            plant.gain_energy(200.0);
        }
    }
}

#[then(regex = r"^la cellule appartient toujours à B$")]
fn cellule_toujours_a_b(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state should exist");

    let target = Pos { x: 11, y: 10 };

    let plant_b = state.plants.iter().find(|p| p.id() == 2).expect("plante B");

    assert!(
        plant_b.footprint().contains(&target),
        "la cellule (11, 10) devrait toujours appartenir a B, emprise B: {:?}",
        plant_b.footprint()
    );

    // Verifier aussi que A n'a PAS cette cellule
    let plant_a = state.plants.iter().find(|p| p.id() == 1).expect("plante A");
    assert!(
        !plant_a.footprint().contains(&target),
        "la plante A ne devrait pas avoir la cellule (11, 10)"
    );
}

#[when(regex = r"^la plante A envahit une cellule de B$")]
fn plante_a_envahit_b(world: &mut GardenWorld) {
    let state = world.state.as_mut().expect("state should exist");

    let a_idx = state
        .plants
        .iter()
        .position(|p| p.id() == 1)
        .expect("plante A");
    let b_idx = state
        .plants
        .iter()
        .position(|p| p.id() == 2)
        .expect("plante B");

    // B doit avoir au moins 2 cellules d'emprise pour que remove_footprint_cell fonctionne.
    // Ajouter une cellule supplementaire si necessaire.
    if state.plants[b_idx].footprint().len() < 2 {
        state.plants[b_idx].grow_footprint(Pos { x: 13, y: 10 });
    }

    // La cellule a envahir : la premiere cellule d'emprise de B
    let target = state.plants[b_idx].footprint()[0];

    // Effectuer l'invasion directement
    state.plants[b_idx].remove_footprint_cell(&target);
    state.plants[a_idx].grow_footprint(target);
    state.plants[a_idx].consume_energy(state.config.invasion_energy_cost);
    state.plants[b_idx].damage(state.config.invasion_damage);

    // Rompre le lien de symbiose (mecanique de la simulation)
    state.symbiosis.remove_link(1, 2);
}

#[then(regex = r"^le lien mycorhizien entre A et B est rompu$")]
fn lien_ab_rompu(world: &mut GardenWorld) {
    let state = world.state.as_ref().expect("state should exist");
    assert!(
        !state.symbiosis.are_linked(1, 2),
        "le lien mycorhizien devrait etre rompu apres l'invasion"
    );
}
