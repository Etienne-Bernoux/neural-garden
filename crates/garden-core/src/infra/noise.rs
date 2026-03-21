// Generation de terrain via Perlin noise — couche infra.

use noise::{NoiseFn, Perlin};

use crate::domain::island::Island;
use crate::domain::plant::Pos;
use crate::domain::world::{World, GRID_SIZE};

/// Genere les altitudes de l'ile via Perlin noise + masque circulaire.
/// Le masque circulaire force les bords de la grille a etre sous le niveau de la mer,
/// creant une ile naturelle au centre.
pub fn generate_terrain(world: &mut World, seed: u32) {
    let perlin = Perlin::new(seed);
    let size = GRID_SIZE as f64;
    let center = size / 2.0;
    let max_radius = center * 0.85; // l'ile occupe ~85% de la grille

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            let pos = Pos { x, y };

            // Bruit Perlin multi-octaves
            let nx = x as f64 / size;
            let ny = y as f64 / size;
            let noise_val = perlin.get([nx * 4.0, ny * 4.0]) * 0.5
                + perlin.get([nx * 8.0, ny * 8.0]) * 0.25
                + perlin.get([nx * 16.0, ny * 16.0]) * 0.125;

            // Masque circulaire : distance au centre normalisee
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let mask = 1.0 - (dist / max_radius).clamp(0.0, 1.0);
            let mask = mask * mask; // courbe plus douce

            // Altitude finale : combinaison bruit + masque, normalisee [0, 1]
            let altitude = ((noise_val + 1.0) / 2.0 * mask).clamp(0.0, 1.0) as f32;

            if let Some(cell) = world.get_mut(&pos) {
                cell.set_altitude(altitude);
            }
        }
    }
}

/// Genere une ile complete : terrain Perlin + construction Island.
pub fn generate_island(world: &mut World, seed: u32, sea_level: f32) -> Island {
    generate_terrain(world, seed);
    Island::from_world(world, sea_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_terrain_perlin_a_des_altitudes_variees() {
        let mut world = World::new();
        generate_terrain(&mut world, 42);

        // Collecter quelques altitudes et verifier qu'elles ne sont pas toutes identiques
        let mut altitudes = Vec::new();
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let pos = Pos { x, y };
                if let Some(cell) = world.get(&pos) {
                    altitudes.push(cell.altitude());
                }
            }
        }

        let first = altitudes[0];
        let all_same = altitudes.iter().all(|&a| (a - first).abs() < f32::EPSILON);
        assert!(
            !all_same,
            "les altitudes ne devraient pas etre toutes identiques"
        );
    }

    #[test]
    fn le_masque_circulaire_force_les_bords_a_zero() {
        let mut world = World::new();
        generate_terrain(&mut world, 42);

        // Les coins de la grille doivent avoir une altitude tres basse
        let coins = [
            Pos { x: 0, y: 0 },
            Pos {
                x: GRID_SIZE - 1,
                y: 0,
            },
            Pos {
                x: 0,
                y: GRID_SIZE - 1,
            },
            Pos {
                x: GRID_SIZE - 1,
                y: GRID_SIZE - 1,
            },
        ];

        for coin in &coins {
            let altitude = world.get(coin).expect("coin valide").altitude();
            assert!(
                altitude < 0.05,
                "le coin ({}, {}) devrait avoir une altitude tres basse, mais a {altitude}",
                coin.x,
                coin.y
            );
        }
    }

    #[test]
    fn lile_perlin_a_terre_et_mer() {
        let mut world = World::new();
        let island = generate_island(&mut world, 42, 0.3);

        let land = island.land_count();
        let total = GRID_SIZE as usize * GRID_SIZE as usize;
        assert!(land > 0, "l'ile devrait avoir de la terre");
        assert!(land < total, "l'ile devrait avoir de la mer");
    }

    #[test]
    fn le_meme_seed_donne_la_meme_ile() {
        let mut world1 = World::new();
        generate_terrain(&mut world1, 123);

        let mut world2 = World::new();
        generate_terrain(&mut world2, 123);

        // Verifier que toutes les altitudes sont identiques
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let pos = Pos { x, y };
                let a1 = world1.get(&pos).expect("valide").altitude();
                let a2 = world2.get(&pos).expect("valide").altitude();
                assert!(
                    (a1 - a2).abs() < f32::EPSILON,
                    "altitudes differentes a ({x}, {y}): {a1} vs {a2}"
                );
            }
        }
    }
}
