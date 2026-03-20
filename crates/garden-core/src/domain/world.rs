// Entite World — la grille 2D de cellules.

use super::plant::Pos;

pub const GRID_SIZE: u16 = 128;

/// Une cellule de la grille, contenant les ressources environnementales.
/// Tous les champs sont bornes dans [0.0, 1.0].
#[derive(Debug, Clone)]
pub struct Cell {
    altitude: f32,
    carbon: f32,
    nitrogen: f32,
    humidity: f32,
    light: f32,
    exudates: f32,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            altitude: 0.0,
            carbon: 0.0,
            nitrogen: 0.0,
            humidity: 0.0,
            light: 1.0,
            exudates: 0.0,
        }
    }

    // --- Getters ---

    pub fn altitude(&self) -> f32 {
        self.altitude
    }

    pub fn carbon(&self) -> f32 {
        self.carbon
    }

    pub fn nitrogen(&self) -> f32 {
        self.nitrogen
    }

    pub fn humidity(&self) -> f32 {
        self.humidity
    }

    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn exudates(&self) -> f32 {
        self.exudates
    }

    // --- Setters avec clamp [0.0, 1.0] ---

    pub fn set_altitude(&mut self, value: f32) {
        self.altitude = value.clamp(0.0, 1.0);
    }

    pub fn set_carbon(&mut self, value: f32) {
        self.carbon = value.clamp(0.0, 1.0);
    }

    pub fn set_nitrogen(&mut self, value: f32) {
        self.nitrogen = value.clamp(0.0, 1.0);
    }

    pub fn set_humidity(&mut self, value: f32) {
        self.humidity = value.clamp(0.0, 1.0);
    }

    pub fn set_light(&mut self, value: f32) {
        self.light = value.clamp(0.0, 1.0);
    }

    pub fn set_exudates(&mut self, value: f32) {
        self.exudates = value.clamp(0.0, 1.0);
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

/// La grille du monde — un tableau plat de GRID_SIZE x GRID_SIZE cellules.
pub struct World {
    cells: Vec<Cell>,
}

impl World {
    pub fn new() -> Self {
        let size = GRID_SIZE as usize * GRID_SIZE as usize;
        Self {
            cells: (0..size).map(|_| Cell::new()).collect(),
        }
    }

    pub fn get(&self, pos: &Pos) -> Option<&Cell> {
        if !self.is_valid(pos) {
            return None;
        }
        let index = pos.y as usize * GRID_SIZE as usize + pos.x as usize;
        self.cells.get(index)
    }

    pub fn get_mut(&mut self, pos: &Pos) -> Option<&mut Cell> {
        if !self.is_valid(pos) {
            return None;
        }
        let index = pos.y as usize * GRID_SIZE as usize + pos.x as usize;
        self.cells.get_mut(index)
    }

    pub fn is_valid(&self, pos: &Pos) -> bool {
        pos.x < GRID_SIZE && pos.y < GRID_SIZE
    }

    pub fn neighbors(&self, pos: &Pos) -> Vec<Pos> {
        let mut result = Vec::with_capacity(4);
        if pos.y > 0 {
            result.push(Pos {
                x: pos.x,
                y: pos.y - 1,
            }); // Nord
        }
        if pos.y + 1 < GRID_SIZE {
            result.push(Pos {
                x: pos.x,
                y: pos.y + 1,
            }); // Sud
        }
        if pos.x + 1 < GRID_SIZE {
            result.push(Pos {
                x: pos.x + 1,
                y: pos.y,
            }); // Est
        }
        if pos.x > 0 {
            result.push(Pos {
                x: pos.x - 1,
                y: pos.y,
            }); // Ouest
        }
        result
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_monde_a_la_bonne_taille() {
        let world = World::new();
        assert_eq!(world.cells.len(), GRID_SIZE as usize * GRID_SIZE as usize);
    }

    #[test]
    fn get_cellule_valide() {
        let world = World::new();
        assert!(world.get(&Pos { x: 0, y: 0 }).is_some());
        assert!(world.get(&Pos { x: 127, y: 127 }).is_some());
    }

    #[test]
    fn get_hors_limites() {
        let world = World::new();
        assert!(world.get(&Pos { x: 128, y: 0 }).is_none());
        assert!(world.get(&Pos { x: 0, y: 128 }).is_none());
    }

    #[test]
    fn get_mut_modifie_la_cellule() {
        let mut world = World::new();
        let pos = Pos { x: 10, y: 10 };
        if let Some(cell) = world.get_mut(&pos) {
            cell.set_carbon(0.8);
        }
        assert!((world.get(&pos).unwrap().carbon() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn voisins_coin() {
        let world = World::new();
        let neighbors = world.neighbors(&Pos { x: 0, y: 0 });
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn voisins_bord() {
        let world = World::new();
        let neighbors = world.neighbors(&Pos { x: 0, y: 64 });
        assert_eq!(neighbors.len(), 3);
    }

    #[test]
    fn voisins_centre() {
        let world = World::new();
        let neighbors = world.neighbors(&Pos { x: 64, y: 64 });
        assert_eq!(neighbors.len(), 4);
    }

    #[test]
    fn lumiere_par_defaut_de_la_cellule() {
        let cell = Cell::new();
        assert_eq!(cell.light(), 1.0);
    }

    #[test]
    fn la_cellule_clampe_les_valeurs() {
        let mut cell = Cell::new();

        // Valeur au-dessus de 1.0 → clampee a 1.0
        cell.set_carbon(1.5);
        assert_eq!(cell.carbon(), 1.0);

        // Valeur en-dessous de 0.0 → clampee a 0.0
        cell.set_carbon(-0.5);
        assert_eq!(cell.carbon(), 0.0);

        // Valeur dans les bornes → inchangee
        cell.set_nitrogen(0.7);
        assert!((cell.nitrogen() - 0.7).abs() < f32::EPSILON);

        // Tous les setters clampent
        cell.set_altitude(2.0);
        assert_eq!(cell.altitude(), 1.0);
        cell.set_humidity(-1.0);
        assert_eq!(cell.humidity(), 0.0);
        cell.set_light(5.0);
        assert_eq!(cell.light(), 1.0);
        cell.set_exudates(-0.1);
        assert_eq!(cell.exudates(), 0.0);
    }
}
