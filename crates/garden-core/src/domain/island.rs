// Island — couche topographique au-dessus de la grille World.

use super::plant::Pos;
use super::rng::Rng;
use super::world::{World, GRID_SIZE};

/// Topographie de l'ile : determine quelles cellules sont terre ou mer.
pub struct Island {
    land_mask: Vec<bool>,
    land_cells: Vec<Pos>,
    sea_level: f32,
}

impl Island {
    /// Genere une ile en remplissant l'altitude de chaque cellule via `rng.next_f32()`
    /// et en calculant le masque terre/mer a partir du seuil de niveau de la mer.
    pub fn generate(world: &mut World, sea_level: f32, rng: &mut dyn Rng) -> Self {
        let size = GRID_SIZE as usize * GRID_SIZE as usize;
        let mut land_mask = Vec::with_capacity(size);
        let mut land_cells = Vec::new();

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let pos = Pos { x, y };
                let altitude = rng.next_f32();
                if let Some(cell) = world.get_mut(&pos) {
                    cell.set_altitude(altitude);
                }
                let is_land = altitude > sea_level;
                land_mask.push(is_land);
                if is_land {
                    land_cells.push(pos);
                }
            }
        }

        Self {
            land_mask,
            land_cells,
            sea_level,
        }
    }

    /// Retourne true si la position est sur terre. Les positions hors limites retournent false.
    pub fn is_land(&self, pos: &Pos) -> bool {
        if pos.x >= GRID_SIZE || pos.y >= GRID_SIZE {
            return false;
        }
        let index = pos.y as usize * GRID_SIZE as usize + pos.x as usize;
        self.land_mask.get(index).copied().unwrap_or(false)
    }

    /// Retourne true si la position est en mer (inverse de is_land).
    pub fn is_sea(&self, pos: &Pos) -> bool {
        !self.is_land(pos)
    }

    /// Retourne le seuil du niveau de la mer.
    pub fn sea_level(&self) -> f32 {
        self.sea_level
    }

    /// Retourne toutes les positions terrestres (cache a la generation).
    pub fn land_cells(&self) -> &[Pos] {
        &self.land_cells
    }

    /// Retourne le nombre de cellules terrestres.
    pub fn land_count(&self) -> usize {
        self.land_cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::rng::test_utils::MockRng;

    #[test]
    fn ile_a_terre_et_mer() {
        let mut world = World::new();
        let mut rng = MockRng::new(0.0, 0.07);
        let island = Island::generate(&mut world, 0.5, &mut rng);

        let land = island.land_count();
        let total = GRID_SIZE as usize * GRID_SIZE as usize;
        assert!(land > 0, "l'ile devrait avoir de la terre");
        assert!(land < total, "l'ile devrait avoir de la mer");
    }

    #[test]
    fn masque_terre_coherent_avec_altitude() {
        let mut world = World::new();
        let mut rng = MockRng::new(0.0, 0.07);
        let island = Island::generate(&mut world, 0.5, &mut rng);

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let pos = Pos { x, y };
                let altitude = world.get(&pos).unwrap().altitude();
                assert_eq!(
                    island.is_land(&pos),
                    altitude > 0.5,
                    "mismatch at ({x}, {y}): altitude={altitude}"
                );
            }
        }
    }

    #[test]
    fn hors_limites_est_mer() {
        let mut world = World::new();
        let mut rng = MockRng::new(0.0, 0.07);
        let island = Island::generate(&mut world, 0.5, &mut rng);

        let pos = Pos { x: 200, y: 200 };
        assert!(island.is_sea(&pos));
        assert!(!island.is_land(&pos));
    }

    #[test]
    fn land_count_correspond_a_land_cells() {
        let mut world = World::new();
        let mut rng = MockRng::new(0.0, 0.07);
        let island = Island::generate(&mut world, 0.5, &mut rng);

        assert_eq!(island.land_count(), island.land_cells().len());
    }

    #[test]
    fn accesseur_niveau_mer() {
        let mut world = World::new();
        let mut rng = MockRng::new(0.0, 0.07);
        let island = Island::generate(&mut world, 0.42, &mut rng);

        assert!((island.sea_level() - 0.42).abs() < f32::EPSILON);
    }
}
