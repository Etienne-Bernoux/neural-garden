use super::plant::{Lineage, PlantState, Pos};

/// Couche de croissance pour l'event Grew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowthLayer {
    Footprint,
    Canopy,
    Roots,
}

/// Evenements domaine emis par les entites durant leurs operations.
///
/// Utilises pour decoupler les effets de bord (replay, TUI, surbrillances)
/// et permettre l'event sourcing (Phase 3).
#[derive(Debug, Clone, PartialEq)]
pub enum DomainEvent {
    /// Une plante a pousse en ajoutant une cellule (emprise, canopee ou racines).
    Grew {
        plant_id: u64,
        cell: Pos,
        layer: GrowthLayer,
    },
    /// Une plante est morte.
    Died {
        plant_id: u64,
        position: Pos,
        age: u32,
        biomass: u16,
    },
    /// Une nouvelle plante est nee (graine placee).
    Born {
        plant_id: u64,
        parent_id: Option<u64>,
        position: Pos,
        lineage: Lineage,
    },
    /// Une plante a envahi la cellule d'une autre plante.
    Invaded {
        invader_id: u64,
        victim_id: u64,
        cell: Pos,
    },
    /// Un lien mycorhizien a ete cree.
    Linked { plant_a: u64, plant_b: u64 },
    /// Un lien mycorhizien a ete rompu.
    Unlinked { plant_a: u64, plant_b: u64 },
    /// Une lignee a fourche (evenement de speciation).
    LineageFork {
        parent_lineage: Lineage,
        child_lineage: Lineage,
        plant_id: u64,
    },
    /// Une plante a germe depuis une graine.
    Germinated { plant_id: u64, position: Pos },
    /// Une plante a change d'etat.
    StateChanged {
        plant_id: u64,
        from: PlantState,
        to: PlantState,
    },
    /// Une plante a retreci (perdu une cellule).
    Shrank { plant_id: u64, cell: Pos },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evenement_croissance_a_les_bonnes_donnees() {
        let event = DomainEvent::Grew {
            plant_id: 1,
            cell: Pos { x: 3, y: 7 },
            layer: GrowthLayer::Footprint,
        };
        match event {
            DomainEvent::Grew {
                plant_id,
                cell,
                layer,
            } => {
                assert_eq!(plant_id, 1);
                assert_eq!(cell, Pos { x: 3, y: 7 });
                assert_eq!(layer, GrowthLayer::Footprint);
            }
            _ => panic!("expected Grew"),
        }
    }

    #[test]
    fn evenement_mort_a_les_bonnes_donnees() {
        let event = DomainEvent::Died {
            plant_id: 42,
            position: Pos { x: 10, y: 20 },
            age: 150,
            biomass: 12,
        };
        match event {
            DomainEvent::Died {
                plant_id,
                position,
                age,
                biomass,
            } => {
                assert_eq!(plant_id, 42);
                assert_eq!(position, Pos { x: 10, y: 20 });
                assert_eq!(age, 150);
                assert_eq!(biomass, 12);
            }
            _ => panic!("expected Died"),
        }
    }

    #[test]
    fn evenement_naissance_avec_parent() {
        let event = DomainEvent::Born {
            plant_id: 5,
            parent_id: Some(2),
            position: Pos { x: 1, y: 1 },
            lineage: Lineage::new(10, 3),
        };
        match event {
            DomainEvent::Born {
                plant_id,
                parent_id,
                position,
                lineage,
            } => {
                assert_eq!(plant_id, 5);
                assert_eq!(parent_id, Some(2));
                assert_eq!(position, Pos { x: 1, y: 1 });
                assert_eq!(lineage, Lineage::new(10, 3));
            }
            _ => panic!("expected Born"),
        }
    }

    #[test]
    fn evenement_naissance_depuis_banque_graines() {
        let event = DomainEvent::Born {
            plant_id: 8,
            parent_id: None,
            position: Pos { x: 0, y: 0 },
            lineage: Lineage::new(1, 0),
        };
        match event {
            DomainEvent::Born { parent_id, .. } => {
                assert_eq!(parent_id, None);
            }
            _ => panic!("expected Born"),
        }
    }

    #[test]
    fn evenement_lie_et_delie() {
        let linked = DomainEvent::Linked {
            plant_a: 1,
            plant_b: 2,
        };
        let unlinked = DomainEvent::Unlinked {
            plant_a: 1,
            plant_b: 2,
        };
        match &linked {
            DomainEvent::Linked { plant_a, plant_b } => {
                assert_eq!(*plant_a, 1);
                assert_eq!(*plant_b, 2);
            }
            _ => panic!("expected Linked"),
        }
        match &unlinked {
            DomainEvent::Unlinked { plant_a, plant_b } => {
                assert_eq!(*plant_a, 1);
                assert_eq!(*plant_b, 2);
            }
            _ => panic!("expected Unlinked"),
        }
    }

    #[test]
    fn les_evenements_sont_clonables() {
        let event = DomainEvent::Grew {
            plant_id: 1,
            cell: Pos { x: 5, y: 5 },
            layer: GrowthLayer::Roots,
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
