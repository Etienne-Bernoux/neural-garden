// Photosynthese batch — calcul de l'energie solaire captee par chaque plante.
// Chaque couche de canopee filtre la lumiere pour les plantes en dessous.

use std::collections::HashMap;

use super::sim::SimState;
use crate::domain::plant::Pos;

/// Calcule la photosynthese pour toutes les plantes en une seule passe.
/// Les plantes sont triees par taille (footprint) decroissante par cellule.
/// La plus grande capte la lumiere pleine, chaque couche traversee
/// attenue la lumiere restante par `canopy_light` (transmittance).
pub fn photosynthesis_batch(state: &mut SimState) {
    // 1. Carte canopee : cellule → [(plant_idx, footprint_size)]
    let mut canopy_map: HashMap<Pos, Vec<(usize, usize)>> = HashMap::new();
    for (idx, plant) in state.plants.iter().enumerate() {
        if plant.is_dead() {
            continue;
        }
        let size = plant.footprint().len();
        for pos in plant.canopy() {
            canopy_map.entry(*pos).or_default().push((idx, size));
        }
    }

    // 2. Par cellule : trier par taille, attenuer la lumiere couche par couche
    let mut gains = vec![0.0_f32; state.plants.len()];
    let transmittance = state.config.canopy_light;

    for (pos, mut occupants) in canopy_map {
        let base_light = state.world.get(&pos).map(|c| c.light()).unwrap_or(0.0);
        occupants.sort_by(|a, b| b.1.cmp(&a.1)); // plus grand d'abord

        let mut remaining = base_light;
        for (plant_idx, _) in occupants {
            gains[plant_idx] += remaining * state.config.photosynthesis_rate;
            remaining *= transmittance; // chaque couche absorbe, le reste passe
        }
    }

    // 3. Appliquer les gains d'energie
    for (idx, gain) in gains.iter().enumerate() {
        if *gain > 0.0 {
            state.plants[idx].gain_energy(*gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::config::SimConfig;
    use crate::application::evolution::{GenerationCounter, PlantStats, SeedBank};
    use crate::application::season::SeasonCycle;
    use crate::application::sim::SimState;
    use crate::domain::island::Island;
    use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant, Pos};
    use crate::domain::symbiosis::SymbiosisNetwork;
    use crate::domain::traits::PlantEntity;
    use crate::domain::world::World;
    use std::collections::HashMap;

    /// Cree un SimState minimal avec un world 8x8, light uniforme.
    fn make_state_with_plants(
        light: f32,
        plants: Vec<Box<dyn PlantEntity>>,
    ) -> SimState {
        let mut world = World::new(8);
        for y in 0..8u16 {
            for x in 0..8u16 {
                let pos = Pos { x, y };
                if let Some(cell) = world.get_mut(&pos) {
                    cell.set_altitude(0.5);
                    cell.set_carbon(0.5);
                    cell.set_nitrogen(0.3);
                    cell.set_humidity(0.5);
                    cell.set_light(light);
                }
            }
        }

        let island = Island::from_world(&world, 0.0);
        let config = SimConfig::default();
        let mut plant_stats = HashMap::new();
        for p in &plants {
            plant_stats.insert(p.id(), PlantStats::default());
        }

        let next_id = plants.iter().map(|p| p.id()).max().unwrap_or(0) + 1;

        SimState::from_raw(
            world,
            island,
            plants,
            HashMap::new(),
            SymbiosisNetwork::new(),
            SeedBank::new(10),
            SeasonCycle::new(config.ticks_per_season),
            GenerationCounter::new(),
            plant_stats,
            next_id,
            0,
            config,
        )
    }

    #[test]
    fn une_plante_seule_capte_toute_la_lumiere() {
        let light = 0.8;
        let traits = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let lineage = Lineage::new(0, 0);
        let mut plant = Plant::new(1, Pos { x: 4, y: 4 }, traits, lineage);
        plant.germinate();

        // Vider l'energie pour mesurer le gain net
        let initial_energy = plant.energy().value();
        plant.consume_energy(initial_energy);

        let mut state = make_state_with_plants(
            light,
            vec![Box::new(plant) as Box<dyn PlantEntity>],
        );

        photosynthesis_batch(&mut state);

        let energy_after = state.plants[0].energy().value();
        let expected = light * state.config.photosynthesis_rate;

        assert!(
            (energy_after - expected).abs() < 1e-5,
            "la plante seule devrait capter light * rate = {expected}, got {energy_after}"
        );
    }

    #[test]
    fn la_lumiere_s_attenue_par_couche() {
        // 3 plantes avec canopee sur la meme cellule (4,4).
        // Toutes ont un footprint de 1 cellule → meme taille.
        // L'ordre de tri est stable (meme taille), la premiere dans l'ordre
        // capte la lumiere pleine, les suivantes recoivent la lumiere attenuee.
        let light = 1.0;
        let rate = SimConfig::default().photosynthesis_rate;
        let transmittance = SimConfig::default().canopy_light;

        // Creer 3 plantes a des positions differentes, chacune avec canopee sur (4,4)
        let shared = Pos { x: 4, y: 4 };

        let traits_a = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_a = Plant::new(1, Pos { x: 3, y: 4 }, traits_a, Lineage::new(1, 0));
        plant_a.germinate();
        plant_a.grow_canopy(shared);
        let e = plant_a.energy().value();
        plant_a.consume_energy(e);

        let traits_b = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_b = Plant::new(2, Pos { x: 5, y: 4 }, traits_b, Lineage::new(2, 0));
        plant_b.germinate();
        plant_b.grow_canopy(shared);
        let e = plant_b.energy().value();
        plant_b.consume_energy(e);

        let traits_c = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_c = Plant::new(3, Pos { x: 4, y: 3 }, traits_c, Lineage::new(3, 0));
        plant_c.germinate();
        plant_c.grow_canopy(shared);
        let e = plant_c.energy().value();
        plant_c.consume_energy(e);

        let mut state = make_state_with_plants(
            light,
            vec![
                Box::new(plant_a) as Box<dyn PlantEntity>,
                Box::new(plant_b),
                Box::new(plant_c),
            ],
        );

        photosynthesis_batch(&mut state);

        // Chaque plante a aussi sa propre position dans la canopee (sa position initiale)
        // donc chaque plante gagne au minimum light * rate pour sa cellule propre.
        // Sur la cellule partagee (4,4), les gains s'attenuent couche par couche.
        //
        // Collecter les gains totaux et verifier la relation d'attenuation :
        // Le gain total de chaque plante = gain sur sa cellule propre + gain sur la cellule partagee
        // Pour la cellule partagee, l'attenuation donne :
        //   couche 1 : light * rate
        //   couche 2 : light * t * rate
        //   couche 3 : light * t^2 * rate
        let gains: Vec<f32> = (0..3)
            .map(|i| state.plants[i].energy().value())
            .collect();

        // Chaque plante a gagne au moins sa propre cellule (light * rate)
        let base_gain = light * rate;
        for (i, g) in gains.iter().enumerate() {
            assert!(
                *g >= base_gain - 1e-5,
                "plante {i} devrait gagner au moins {base_gain} (sa cellule propre), got {g}"
            );
        }

        // La somme des gains sur la cellule partagee = light * rate * (1 + t + t^2)
        let total_shared = gains.iter().sum::<f32>() - 3.0 * base_gain;
        let expected_shared = light * rate * (1.0 + transmittance + transmittance * transmittance);
        assert!(
            (total_shared - expected_shared).abs() < 1e-4,
            "les gains sur la cellule partagee devraient totaliser {expected_shared}, got {total_shared}"
        );
    }

    #[test]
    fn cellule_sans_lumiere_donne_zero_gain() {
        // World avec light = 0.0 partout
        let traits = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let lineage = Lineage::new(0, 0);
        let mut plant = Plant::new(1, Pos { x: 4, y: 4 }, traits, lineage);
        plant.germinate();

        let initial_energy = plant.energy().value();
        plant.consume_energy(initial_energy);

        let mut state = make_state_with_plants(
            0.0, // pas de lumiere
            vec![Box::new(plant) as Box<dyn PlantEntity>],
        );

        photosynthesis_batch(&mut state);

        let energy_after = state.plants[0].energy().value();
        assert!(
            energy_after < 1e-6,
            "sans lumiere, la plante ne devrait rien capter, got {energy_after}"
        );
    }

    #[test]
    fn plante_morte_ignoree_dans_le_calcul() {
        let light = 0.8;
        let traits_a = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_a = Plant::new(1, Pos { x: 3, y: 3 }, traits_a, Lineage::new(1, 0));
        plant_a.germinate();
        // Tuer la plante A : infliger des degats superieurs a la vitalite
        let v = plant_a.vitality().value();
        plant_a.damage(v + 1.0);
        plant_a.update_state();
        assert!(plant_a.is_dead(), "la plante A devrait etre morte");
        // Vider l'energie residuelle pour mesurer le gain net
        let e = plant_a.energy().value();
        plant_a.consume_energy(e);

        let traits_b = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_b = Plant::new(2, Pos { x: 5, y: 5 }, traits_b, Lineage::new(2, 0));
        plant_b.germinate();
        let e = plant_b.energy().value();
        plant_b.consume_energy(e);

        let mut state = make_state_with_plants(
            light,
            vec![
                Box::new(plant_a) as Box<dyn PlantEntity>,
                Box::new(plant_b),
            ],
        );

        photosynthesis_batch(&mut state);

        // La plante morte ne gagne rien
        let energy_a = state.plants[0].energy().value();
        assert!(
            energy_a < 1e-6,
            "la plante morte ne devrait rien recevoir, got {energy_a}"
        );

        // La plante vivante capte normalement
        let energy_b = state.plants[1].energy().value();
        let expected = light * state.config.photosynthesis_rate;
        assert!(
            (energy_b - expected).abs() < 1e-5,
            "la plante vivante devrait capter {expected}, got {energy_b}"
        );
    }

    #[test]
    fn la_plus_grande_plante_capte_en_premier() {
        // 2 plantes sur la meme cellule partagee (4,4).
        // Plante A : footprint de 1 cellule (taille par defaut).
        // Plante B : footprint de 3 cellules (on fait grossir son emprise).
        // B est plus grande → B capte la lumiere pleine sur (4,4), A recoit l'attenuee.
        let light = 1.0;
        let rate = SimConfig::default().photosynthesis_rate;
        let transmittance = SimConfig::default().canopy_light;
        let shared = Pos { x: 4, y: 4 };

        // Plante A : petite (1 cellule de footprint), canopee sur shared
        let traits_a = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_a = Plant::new(1, Pos { x: 3, y: 4 }, traits_a, Lineage::new(1, 0));
        plant_a.germinate();
        plant_a.grow_canopy(shared);
        let e = plant_a.energy().value();
        plant_a.consume_energy(e);

        // Plante B : grande (3 cellules de footprint), canopee sur shared
        let traits_b = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_b = Plant::new(2, Pos { x: 5, y: 4 }, traits_b, Lineage::new(2, 0));
        plant_b.germinate();
        plant_b.grow_footprint(Pos { x: 6, y: 4 });
        plant_b.grow_footprint(Pos { x: 7, y: 4 });
        plant_b.grow_canopy(shared);
        let e = plant_b.energy().value();
        plant_b.consume_energy(e);

        let mut state = make_state_with_plants(
            light,
            vec![
                Box::new(plant_a) as Box<dyn PlantEntity>,
                Box::new(plant_b),
            ],
        );

        photosynthesis_batch(&mut state);

        let energy_a = state.plants[0].energy().value();
        let energy_b = state.plants[1].energy().value();

        // Sur la cellule partagee (4,4), B (footprint=3) capte en premier (light * rate)
        // puis A (footprint=1) recoit light * transmittance * rate.
        // Chaque plante gagne aussi sa propre cellule (ou cellules).
        // B a 3 cellules de footprint → 3 cellules de canopee propres + shared
        // A a 1 cellule de footprint → 1 cellule de canopee propre + shared
        //
        // Gain A sur shared = light * transmittance * rate (car B capte en premier)
        // Gain B sur shared = light * rate
        // Donc B devrait avoir plus que A
        assert!(
            energy_b > energy_a,
            "la plante B (plus grande) devrait capter plus que A. B={energy_b}, A={energy_a}"
        );

        // Verification precise : sur la cellule shared, A recoit l'attenuee
        // Gain total A = propre cellule (light * rate) + shared (light * transmittance * rate)
        let expected_a = light * rate + light * transmittance * rate;
        assert!(
            (energy_a - expected_a).abs() < 1e-5,
            "la plante A devrait capter {expected_a}, got {energy_a}"
        );
    }

    #[test]
    fn canopees_non_chevauchantes_captent_independamment() {
        // 2 plantes sur des cellules differentes, pas de chevauchement
        let light = 0.8;
        let rate = SimConfig::default().photosynthesis_rate;

        let traits_a = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_a = Plant::new(1, Pos { x: 2, y: 2 }, traits_a, Lineage::new(1, 0));
        plant_a.germinate();
        let e = plant_a.energy().value();
        plant_a.consume_energy(e);

        let traits_b = GeneticTraits::new(15, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let mut plant_b = Plant::new(2, Pos { x: 6, y: 6 }, traits_b, Lineage::new(2, 0));
        plant_b.germinate();
        let e = plant_b.energy().value();
        plant_b.consume_energy(e);

        let mut state = make_state_with_plants(
            light,
            vec![
                Box::new(plant_a) as Box<dyn PlantEntity>,
                Box::new(plant_b),
            ],
        );

        photosynthesis_batch(&mut state);

        let expected = light * rate;
        let energy_a = state.plants[0].energy().value();
        let energy_b = state.plants[1].energy().value();

        assert!(
            (energy_a - expected).abs() < 1e-5,
            "plante A devrait capter {expected} (pas d'attenuation), got {energy_a}"
        );
        assert!(
            (energy_b - expected).abs() < 1e-5,
            "plante B devrait capter {expected} (pas d'attenuation), got {energy_b}"
        );
    }

    // Note : le cas "plante sans canopee" n'existe pas — une plante germinee
    // a toujours 1 racine, 1 footprint, 1 canopee. Le cas "plante morte"
    // est couvert par plante_morte_ignoree_dans_le_calcul.
}
