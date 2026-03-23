// DTOs pour World, Cell, Island, Symbiosis.

use serde::{Deserialize, Serialize};

use super::PosDto;
use crate::domain::island::Island;
use crate::domain::symbiosis::{MycorrhizalLink, SymbiosisNetwork};
use crate::domain::world::{Cell, World};

// --- Cell ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CellDto {
    pub altitude: f32,
    pub carbon: f32,
    pub nitrogen: f32,
    pub humidity: f32,
    pub light: f32,
    pub exudates: f32,
}

impl From<&Cell> for CellDto {
    fn from(c: &Cell) -> Self {
        Self {
            altitude: c.altitude(),
            carbon: c.carbon(),
            nitrogen: c.nitrogen(),
            humidity: c.humidity(),
            light: c.light(),
            exudates: c.exudates(),
        }
    }
}

impl CellDto {
    pub fn to_domain(&self) -> Cell {
        Cell::from_raw(
            self.altitude,
            self.carbon,
            self.nitrogen,
            self.humidity,
            self.light,
            self.exudates,
        )
    }
}

// --- World ---

#[derive(Serialize, Deserialize, Debug)]
pub struct WorldDto {
    pub size: u16,
    pub cells: Vec<CellDto>,
}

impl From<&World> for WorldDto {
    fn from(w: &World) -> Self {
        let grid_size = w.size();
        let mut cells = Vec::with_capacity(grid_size as usize * grid_size as usize);
        for y in 0..grid_size {
            for x in 0..grid_size {
                let pos = crate::domain::plant::Pos { x, y };
                if let Some(cell) = w.get(&pos) {
                    cells.push(CellDto::from(cell));
                }
            }
        }
        Self {
            size: grid_size,
            cells,
        }
    }
}

impl WorldDto {
    pub fn to_domain(&self) -> World {
        let cells: Vec<Cell> = self.cells.iter().map(|c| c.to_domain()).collect();
        World::from_cells(cells, self.size)
    }
}

// --- Island ---

#[derive(Serialize, Deserialize, Debug)]
pub struct IslandDto {
    pub land_mask: Vec<bool>,
    pub sea_level: f32,
    pub land_cells: Vec<PosDto>,
    pub size: u16,
}

impl From<&Island> for IslandDto {
    fn from(i: &Island) -> Self {
        Self {
            land_mask: i.land_mask().to_vec(),
            sea_level: i.sea_level(),
            land_cells: i.land_cells().iter().map(PosDto::from).collect(),
            size: i.size(),
        }
    }
}

impl IslandDto {
    pub fn to_domain(&self) -> Island {
        Island::from_raw(
            self.land_mask.clone(),
            self.sea_level,
            self.land_cells.iter().map(|p| p.to_domain()).collect(),
            self.size,
        )
    }
}

// --- MycorrhizalLink ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MycorrhizalLinkDto {
    pub plant_a: u64,
    pub plant_b: u64,
}

impl From<&MycorrhizalLink> for MycorrhizalLinkDto {
    fn from(l: &MycorrhizalLink) -> Self {
        Self {
            plant_a: l.plant_a(),
            plant_b: l.plant_b(),
        }
    }
}

impl MycorrhizalLinkDto {
    pub fn to_domain(&self) -> MycorrhizalLink {
        MycorrhizalLink::new(self.plant_a, self.plant_b)
    }
}

// --- SymbiosisNetwork ---

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbiosisNetworkDto {
    pub links: Vec<MycorrhizalLinkDto>,
}

impl From<&SymbiosisNetwork> for SymbiosisNetworkDto {
    fn from(n: &SymbiosisNetwork) -> Self {
        Self {
            links: n.links().iter().map(MycorrhizalLinkDto::from).collect(),
        }
    }
}

impl SymbiosisNetworkDto {
    pub fn to_domain(&self) -> SymbiosisNetwork {
        let links: Vec<MycorrhizalLink> = self.links.iter().map(|l| l.to_domain()).collect();
        SymbiosisNetwork::from_links(links)
    }
}
