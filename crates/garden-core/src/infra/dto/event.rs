// DTO pour les DomainEvent (replay / event sourcing).

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::domain::events::{DomainEvent, GrowthLayer};

/// DTO generique pour les events du domaine.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DomainEventDto {
    pub tick: u32,
    pub event_type: String,
    pub data: serde_json::Value,
}

impl DomainEventDto {
    /// Convertit un event domaine + tick en DTO serialisable.
    pub fn from_event(tick: u32, event: &DomainEvent) -> Self {
        match event {
            DomainEvent::Grew {
                plant_id,
                cell,
                layer,
            } => Self {
                tick,
                event_type: "Grew".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": cell.x,
                    "y": cell.y,
                    "layer": match layer {
                        GrowthLayer::Footprint => "footprint",
                        GrowthLayer::Canopy => "canopy",
                        GrowthLayer::Roots => "roots",
                    },
                    // Compat temporaire : is_canopy = true si footprint (ancien comportement)
                    "is_canopy": matches!(layer, GrowthLayer::Footprint),
                }),
            },
            DomainEvent::Died {
                plant_id,
                position,
                age,
                biomass,
            } => Self {
                tick,
                event_type: "Died".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": position.x,
                    "y": position.y,
                    "age": age,
                    "biomass": biomass,
                }),
            },
            DomainEvent::Born {
                plant_id,
                parent_id,
                position,
                lineage,
            } => Self {
                tick,
                event_type: "Born".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "parent_id": parent_id,
                    "x": position.x,
                    "y": position.y,
                    "lineage_id": lineage.id(),
                    "lineage_gen": lineage.generation(),
                }),
            },
            DomainEvent::Invaded {
                invader_id,
                victim_id,
                cell,
            } => Self {
                tick,
                event_type: "Invaded".to_string(),
                data: json!({
                    "invader_id": invader_id,
                    "victim_id": victim_id,
                    "x": cell.x,
                    "y": cell.y,
                }),
            },
            DomainEvent::Linked { plant_a, plant_b } => Self {
                tick,
                event_type: "Linked".to_string(),
                data: json!({
                    "plant_a": plant_a,
                    "plant_b": plant_b,
                }),
            },
            DomainEvent::Unlinked { plant_a, plant_b } => Self {
                tick,
                event_type: "Unlinked".to_string(),
                data: json!({
                    "plant_a": plant_a,
                    "plant_b": plant_b,
                }),
            },
            DomainEvent::LineageFork {
                parent_lineage,
                child_lineage,
                plant_id,
            } => Self {
                tick,
                event_type: "LineageFork".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "parent_lineage_id": parent_lineage.id(),
                    "parent_lineage_gen": parent_lineage.generation(),
                    "child_lineage_id": child_lineage.id(),
                    "child_lineage_gen": child_lineage.generation(),
                }),
            },
            DomainEvent::Germinated { plant_id, position } => Self {
                tick,
                event_type: "Germinated".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": position.x,
                    "y": position.y,
                }),
            },
            DomainEvent::StateChanged { plant_id, from, to } => Self {
                tick,
                event_type: "StateChanged".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "from": format!("{:?}", from),
                    "to": format!("{:?}", to),
                }),
            },
            DomainEvent::Shrank { plant_id, cell } => Self {
                tick,
                event_type: "Shrank".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": cell.x,
                    "y": cell.y,
                }),
            },
            DomainEvent::StageReached { plant_id, stage } => Self {
                tick,
                event_type: "StageReached".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "stage": format!("{:?}", stage),
                }),
            },
            DomainEvent::CellUpgraded {
                plant_id,
                cell,
                layer,
                new_level,
            } => Self {
                tick,
                event_type: "CellUpgraded".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": cell.x,
                    "y": cell.y,
                    "layer": match layer {
                        GrowthLayer::Footprint => "footprint",
                        GrowthLayer::Canopy => "canopy",
                        GrowthLayer::Roots => "roots",
                    },
                    "new_level": new_level,
                }),
            },
            DomainEvent::VenerableDied { plant_id, pos } => Self {
                tick,
                event_type: "VenerableDied".to_string(),
                data: json!({
                    "plant_id": plant_id,
                    "x": pos.x,
                    "y": pos.y,
                }),
            },
        }
    }
}
