// Service de perception — calcule les 18 inputs du cerveau pour une plante.

use crate::domain::plant::Plant;
use crate::domain::world::{Cell, World};

/// Signe d'un flottant : -1.0, 0.0 ou 1.0.
fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Calcule la moyenne d'un champ sur les cellules d'emprise (footprint).
fn average_field(plant: &Plant, world: &World, field: fn(&Cell) -> f32) -> f32 {
    let footprint = plant.footprint();
    if footprint.is_empty() {
        return 0.0;
    }
    let sum: f32 = footprint
        .iter()
        .filter_map(|pos| world.get(pos))
        .map(field)
        .sum();
    sum / footprint.len() as f32
}

/// Calcule les 18 inputs de perception pour une plante.
///
/// Layout :
/// - [0..4]  : etat interne (vitality, energy, biomass, age) normalises [0, 1]
/// - [4..8]  : sol local (carbon, nitrogen, humidity, light) moyennes sur la canopee
/// - [8..18] : gradients (5 champs x 2 composantes x/y) sur les racines, bornes [-1, 1]
pub fn compute_inputs(plant: &Plant, world: &World) -> [f32; 18] {
    let mut inputs = [0.0_f32; 18];

    // --- Etat interne (normalise [0, 1]) ---
    let genetics = plant.genetics();
    let biomass = plant.biomass();

    let v_cap = crate::domain::plant::vitality_cap(biomass, genetics.vitality_factor());
    let e_cap = crate::domain::plant::energy_cap(biomass, genetics.energy_factor());

    inputs[0] = if v_cap > 0.0 {
        (plant.vitality().value() / v_cap).clamp(0.0, 1.0)
    } else {
        0.0
    };
    inputs[1] = if e_cap > 0.0 {
        (plant.energy().value() / e_cap).clamp(0.0, 1.0)
    } else {
        0.0
    };
    inputs[2] = (biomass.value() as f32 / genetics.max_size() as f32).clamp(0.0, 1.0);
    inputs[3] = (plant.age() as f32 / 2000.0).clamp(0.0, 1.0);

    // --- Sol local (moyenne sur la canopee) ---
    inputs[4] = average_field(plant, world, Cell::carbon);
    inputs[5] = average_field(plant, world, Cell::nitrogen);
    inputs[6] = average_field(plant, world, Cell::humidity);
    inputs[7] = average_field(plant, world, Cell::light);

    // --- Gradients sur les racines ---
    let roots = plant.roots();
    let count = roots.len();

    // Avec une seule racine, pas de differentiel possible
    if count <= 1 {
        return inputs;
    }

    // Barycentre des racines
    let cx: f32 = roots.iter().map(|r| r.x as f32).sum::<f32>() / count as f32;
    let cy: f32 = roots.iter().map(|r| r.y as f32).sum::<f32>() / count as f32;

    // Extracteurs de champs pour les 5 gradients
    let fields: [fn(&Cell) -> f32; 5] = [
        Cell::carbon,
        Cell::nitrogen,
        Cell::humidity,
        Cell::exudates, // proxy pour biomasse
        Cell::light,
    ];

    for (field_idx, field_fn) in fields.iter().enumerate() {
        let mut grad_x = 0.0_f32;
        let mut grad_y = 0.0_f32;

        for root in roots {
            if let Some(cell) = world.get(root) {
                let value = field_fn(cell);
                grad_x += value * sign(root.x as f32 - cx);
                grad_y += value * sign(root.y as f32 - cy);
            }
        }

        // Normaliser par le nombre de racines et clamper
        grad_x = (grad_x / count as f32).clamp(-1.0, 1.0);
        grad_y = (grad_y / count as f32).clamp(-1.0, 1.0);

        inputs[8 + field_idx * 2] = grad_x;
        inputs[8 + field_idx * 2 + 1] = grad_y;
    }

    inputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
    use crate::domain::world::World;

    fn test_genetics() -> GeneticTraits {
        GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 5.0)
    }

    fn test_plant_at(pos: Pos) -> Plant {
        Plant::new(1, pos, test_genetics(), Lineage::new(1, 0))
    }

    #[test]
    fn une_graine_a_des_gradients_nuls() {
        // Une graine a 1 seule racine → tous les gradients doivent etre 0
        let plant = test_plant_at(Pos { x: 5, y: 5 });
        let world = World::new();
        let inputs = compute_inputs(&plant, &world);

        for i in 8..18 {
            assert_eq!(
                inputs[i], 0.0,
                "gradient input[{i}] devrait etre 0.0, vaut {}",
                inputs[i]
            );
        }
    }

    #[test]
    fn les_inputs_internes_sont_normalises() {
        let plant = test_plant_at(Pos { x: 5, y: 5 });
        let world = World::new();
        let inputs = compute_inputs(&plant, &world);

        for i in 0..4 {
            assert!(
                (0.0..=1.0).contains(&inputs[i]),
                "input[{i}] = {} n'est pas dans [0, 1]",
                inputs[i]
            );
        }
    }

    #[test]
    fn le_sol_local_reflete_la_canopee() {
        let plant = test_plant_at(Pos { x: 5, y: 5 });
        let mut world = World::new();

        // Mettre du carbone sous la canopee (une seule cellule)
        if let Some(cell) = world.get_mut(&Pos { x: 5, y: 5 }) {
            cell.set_carbon(0.5);
        }

        let inputs = compute_inputs(&plant, &world);
        assert!(
            (inputs[4] - 0.5).abs() < f32::EPSILON,
            "input[4] (local_carbon) devrait etre 0.5, vaut {}",
            inputs[4]
        );
    }

    #[test]
    fn le_gradient_pointe_vers_la_ressource() {
        // Plante avec 3 racines en ligne : x=4, x=5, x=6, y=5
        let mut plant = test_plant_at(Pos { x: 5, y: 5 });
        plant.germinate();
        plant.grow_roots(Pos { x: 4, y: 5 }); // racine
        plant.grow_roots(Pos { x: 6, y: 5 }); // racine

        let mut world = World::new();
        // Carbon seulement a la racine droite
        if let Some(cell) = world.get_mut(&Pos { x: 6, y: 5 }) {
            cell.set_carbon(1.0);
        }

        let inputs = compute_inputs(&plant, &world);
        assert!(
            inputs[8] > 0.0,
            "grad_carbon_x devrait etre > 0 (pointe vers x+), vaut {}",
            inputs[8]
        );
    }

    #[test]
    fn les_gradients_sont_bornes() {
        // Plante avec plusieurs racines
        let mut plant = test_plant_at(Pos { x: 5, y: 5 });
        plant.germinate();
        plant.grow_roots(Pos { x: 4, y: 5 });
        plant.grow_roots(Pos { x: 6, y: 5 });

        let mut world = World::new();
        // Mettre des valeurs extremes partout
        for x in 4..=6 {
            if let Some(cell) = world.get_mut(&Pos { x, y: 5 }) {
                cell.set_carbon(1.0);
                cell.set_nitrogen(1.0);
                cell.set_humidity(1.0);
                cell.set_exudates(1.0);
                cell.set_light(1.0);
            }
        }

        let inputs = compute_inputs(&plant, &world);
        for i in 8..18 {
            assert!(
                (-1.0..=1.0).contains(&inputs[i]),
                "gradient input[{i}] = {} n'est pas dans [-1, 1]",
                inputs[i]
            );
        }
    }
}
