// DTOs serde pour la serialisation/deserialisation.
// Miroirs des types domain/application avec derive Serialize/Deserialize.

mod event;
mod plant;
mod sim;
mod world;

pub use event::DomainEventDto;
pub use plant::{
    BrainDto, ExudateTypeDto, GeneticTraitsDto, GenomeDto, LineageDto, PlantDto, PlantStateDto,
    PlantStatsDto,
};
pub use sim::{SeasonCycleDto, SeasonDto, SeedBankDto, SimConfigDto, SimStateDto};
pub use world::{CellDto, IslandDto, MycorrhizalLinkDto, SymbiosisNetworkDto, WorldDto};

use crate::domain::plant::Pos;
use serde::{Deserialize, Serialize};

/// DTO pour une position 2D.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PosDto {
    pub x: u16,
    pub y: u16,
}

impl From<&Pos> for PosDto {
    fn from(pos: &Pos) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}

impl PosDto {
    /// Convertit en type domaine.
    pub fn to_domain(&self) -> Pos {
        Pos {
            x: self.x,
            y: self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::brain::Brain;
    use crate::domain::events::DomainEvent;
    use crate::domain::plant::{ExudateType, GeneticTraits, Lineage, Plant};
    use crate::domain::rng::test_utils::MockRng;
    use crate::domain::world::World;

    fn test_genetics() -> GeneticTraits {
        GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 5.0)
    }

    fn test_plant() -> Plant {
        Plant::new(1, Pos { x: 5, y: 5 }, test_genetics(), Lineage::new(1, 0))
    }

    #[test]
    fn aller_retour_plant_dto() {
        let mut plant = test_plant();
        let _ = plant.germinate();
        let _ = plant.grow_footprint(Pos { x: 6, y: 5 });
        let _ = plant.grow_roots(Pos { x: 5, y: 6 });
        plant.damage(2.0);

        // Plant → PlantDto → JSON → PlantDto → Plant
        let dto = PlantDto::from(&plant);
        let json = serde_json::to_string(&dto).expect("serialisation echouee");
        let dto2: PlantDto = serde_json::from_str(&json).expect("deserialisation echouee");
        let plant2 = dto2.to_domain();

        assert_eq!(plant.id(), plant2.id());
        assert_eq!(plant.state(), plant2.state());
        assert_eq!(plant.age(), plant2.age());
        assert!((plant.vitality().value() - plant2.vitality().value()).abs() < f32::EPSILON);
        assert!((plant.energy().value() - plant2.energy().value()).abs() < f32::EPSILON);
        assert_eq!(plant.biomass().value(), plant2.biomass().value());
        assert_eq!(plant.footprint().len(), plant2.footprint().len());
        assert_eq!(plant.canopy().len(), plant2.canopy().len());
        assert_eq!(plant.roots().len(), plant2.roots().len());
        assert_eq!(plant.genetics().max_size(), plant2.genetics().max_size());
        assert_eq!(plant.lineage().id(), plant2.lineage().id());
        assert_eq!(plant.lineage().generation(), plant2.lineage().generation());
    }

    #[test]
    fn aller_retour_brain_dto() {
        let mut rng = MockRng::new(0.5, 0.1);
        let brain = Brain::new(8, &mut rng);

        // Brain → BrainDto → JSON → BrainDto → Brain
        let dto = BrainDto::from(&brain);
        let json = serde_json::to_string(&dto).expect("serialisation echouee");
        let dto2: BrainDto = serde_json::from_str(&json).expect("deserialisation echouee");
        let brain2 = dto2.to_domain().expect("reconstruction echouee");

        assert_eq!(brain.hidden_size(), brain2.hidden_size());
        assert_eq!(brain.weights(), brain2.weights());
    }

    #[test]
    fn aller_retour_world_dto() {
        let mut world = World::new();
        // Modifier quelques cellules
        let pos = Pos { x: 10, y: 20 };
        if let Some(cell) = world.get_mut(&pos) {
            cell.set_carbon(0.7);
            cell.set_nitrogen(0.3);
            cell.set_humidity(0.5);
        }

        // World → WorldDto → JSON → WorldDto → World
        let dto = WorldDto::from(&world);
        let json = serde_json::to_string(&dto).expect("serialisation echouee");
        let dto2: WorldDto = serde_json::from_str(&json).expect("deserialisation echouee");
        let world2 = dto2.to_domain();

        // Verifier la cellule modifiee
        let cell2 = world2.get(&pos).expect("cellule absente");
        assert!((cell2.carbon() - 0.7).abs() < f32::EPSILON);
        assert!((cell2.nitrogen() - 0.3).abs() < f32::EPSILON);
        assert!((cell2.humidity() - 0.5).abs() < f32::EPSILON);

        // Verifier une cellule par defaut
        let default_pos = Pos { x: 0, y: 0 };
        let default_cell = world2.get(&default_pos).expect("cellule absente");
        assert!((default_cell.light() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn aller_retour_domain_event_dto() {
        use crate::domain::events::GrowthLayer;

        let event = DomainEvent::Grew {
            plant_id: 42,
            cell: Pos { x: 3, y: 7 },
            layer: GrowthLayer::Footprint,
        };

        let dto = DomainEventDto::from_event(10, &event);
        let json = serde_json::to_string(&dto).expect("serialisation echouee");

        // Verifier le format JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("deserialisation echouee");
        assert_eq!(parsed["tick"], 10);
        assert_eq!(parsed["event_type"], "Grew");
        assert_eq!(parsed["data"]["plant_id"], 42);
        assert_eq!(parsed["data"]["x"], 3);
        assert_eq!(parsed["data"]["y"], 7);
        assert_eq!(parsed["data"]["layer"], "footprint");
    }
}
