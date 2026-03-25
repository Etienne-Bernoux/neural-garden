// Stades de croissance — definitions des 9 stades et table de lookup.
// Pur domain, zero dependance externe.

/// Les 9 stades de croissance d'une plante, de la graine au venerable.
/// L'ordre des variantes est significatif (derive PartialOrd/Ord).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrowthStage {
    Graine,
    Germe,
    Pousse,
    Plantule,
    Arbuste,
    JeuneArbre,
    Arbre,
    ArbreMature,
    Venerable,
}

/// Bonus appliques a un stade de croissance.
/// Les valeurs par defaut sont neutres (false, 1.0, 0.0 selon le type).
#[derive(Debug, Clone)]
pub struct StageBonuses {
    /// La plante est invisible (pas ciblee par l'invasion).
    pub invisibility: bool,
    /// Multiplicateur de vitesse de croissance (1.0 = normal).
    pub growth_speed: f32,
    /// Multiplicateur de resistance au stress (1.0 = normal).
    pub resilience: f32,
    /// Couverture du sol (empeche l'erosion, retient l'humidite).
    pub ground_cover: bool,
    /// Produit de l'ombre pour les plantes en dessous.
    pub canopy_shade: bool,
    /// Multiplicateur de distance de dispersion des graines (1.0 = normal).
    pub seed_distance: f32,
    /// Multiplicateur de photosynthese (1.0 = normal).
    pub photosynthesis_multiplier: f32,
    /// Reseau racinaire etendu (facilite la symbiose).
    pub root_network: bool,
    /// Multiplicateur de reproduction (1.0 = normal).
    pub reproduction_multiplier: f32,
    /// Multiplicateur des echanges symbiose (1.0 = normal, 3.0 = hub).
    pub symbiosis_hub: f32,
    /// Enrichissement du sol (ajoute N aux cases adjacentes).
    pub soil_enrichment: bool,
    /// Taux d'enrichissement du sol par tick par case racine.
    pub soil_enrichment_rate: f32,
    /// Multiplicateur de resistance de base (1.0 = normal).
    pub resistance: f32,
}

impl Default for StageBonuses {
    fn default() -> Self {
        Self {
            invisibility: false,
            growth_speed: 1.0,
            resilience: 1.0,
            ground_cover: false,
            canopy_shade: false,
            seed_distance: 1.0,
            photosynthesis_multiplier: 1.0,
            root_network: false,
            reproduction_multiplier: 1.0,
            symbiosis_hub: 1.0,
            soil_enrichment: false,
            soil_enrichment_rate: 0.0,
            resistance: 1.0,
        }
    }
}

/// Definition d'un stade de croissance.
#[derive(Debug, Clone)]
pub struct StageDefinition {
    pub name: GrowthStage,
    /// Biomasse minimale pour atteindre ce stade.
    pub biomass_min: u16,
    /// Biomasse maximale de ce stade (borne superieure).
    pub biomass_max: u16,
    /// Niveau maximum des cases racine/canopee a ce stade.
    pub cell_level_cap: u8,
    /// Multiplicateur de cout de maintenance.
    pub maintenance_multiplier: f32,
    /// Peut photosynthetiser a ce stade.
    pub can_photosynthesize: bool,
    /// Peut se reproduire a ce stade.
    pub can_reproduce: bool,
    /// Peut etablir des liens mycorhiziens a ce stade.
    pub can_symbiose: bool,
    /// Bonus specifiques au stade.
    pub bonuses: StageBonuses,
}

/// Couts d'upgrade de case par niveau cible.
#[derive(Debug, Clone)]
pub struct UpgradeCosts {
    /// Cout de base pour upgrader une case.
    pub base_cost: f32,
    /// Multiplicateur par niveau cible (cout = base_cost * target_level * level_multiplier).
    pub level_multiplier: f32,
}

/// Configuration de la senescence.
#[derive(Debug, Clone)]
pub struct SenescenceConfig {
    /// Stade a partir duquel la senescence commence.
    pub start_stage: GrowthStage,
    /// Augmentation de la vulnerabilite par tick.
    pub vulnerability_rate: f32,
    /// Plafond de vulnerabilite (ex: 0.5 = 50% de resistance en moins).
    pub max_vulnerability: f32,
}

/// Table de definitions des stades de croissance.
/// Ordonnee par stade (Graine en premier, Venerable en dernier).
#[derive(Debug, Clone)]
pub struct StageTable {
    stages: Vec<StageDefinition>,
    pub upgrade_costs: UpgradeCosts,
    pub senescence: SenescenceConfig,
}

impl StageTable {
    /// Cree une nouvelle table de stades.
    /// Les definitions doivent etre ordonnees par stade croissant.
    pub fn new(
        stages: Vec<StageDefinition>,
        upgrade_costs: UpgradeCosts,
        senescence: SenescenceConfig,
    ) -> Self {
        Self {
            stages,
            upgrade_costs,
            senescence,
        }
    }

    /// Retourne le stade correspondant a la biomasse, plafonne par max_stage.
    /// Parcourt les stades du plus eleve au plus bas et retourne le premier
    /// dont la biomasse minimale est atteinte, sans depasser max_stage.
    pub fn stage_for_biomass(&self, biomass: u16, max_stage: GrowthStage) -> GrowthStage {
        let mut result = GrowthStage::Graine;
        for def in &self.stages {
            if def.name > max_stage {
                break;
            }
            if biomass >= def.biomass_min {
                result = def.name;
            }
        }
        result
    }

    /// Retourne la definition d'un stade.
    /// Panique si le stade n'existe pas dans la table (invariant de construction).
    pub fn definition(&self, stage: GrowthStage) -> &StageDefinition {
        self.stages
            .iter()
            .find(|d| d.name == stage)
            .expect("stade absent de la table — invariant de construction viole")
    }

    /// Construit la table par defaut avec les valeurs du plan.
    /// Utilise pour les tests et comme fallback si pas de YAML.
    pub fn default_table() -> Self {
        let stages = vec![
            StageDefinition {
                name: GrowthStage::Graine,
                biomass_min: 0,
                biomass_max: 0,
                cell_level_cap: 0,
                maintenance_multiplier: 0.0,
                can_photosynthesize: false,
                can_reproduce: false,
                can_symbiose: false,
                bonuses: StageBonuses::default(),
            },
            StageDefinition {
                name: GrowthStage::Germe,
                biomass_min: 1,
                biomass_max: 1,
                cell_level_cap: 0,
                maintenance_multiplier: 0.2,
                can_photosynthesize: false,
                can_reproduce: false,
                can_symbiose: false,
                bonuses: StageBonuses {
                    invisibility: true,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::Pousse,
                biomass_min: 2,
                biomass_max: 3,
                cell_level_cap: 1,
                maintenance_multiplier: 0.5,
                can_photosynthesize: true,
                can_reproduce: false,
                can_symbiose: false,
                bonuses: StageBonuses {
                    growth_speed: 1.5,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::Plantule,
                biomass_min: 4,
                biomass_max: 6,
                cell_level_cap: 1,
                maintenance_multiplier: 0.7,
                can_photosynthesize: true,
                can_reproduce: false,
                can_symbiose: true,
                bonuses: StageBonuses {
                    resilience: 1.2,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::Arbuste,
                biomass_min: 7,
                biomass_max: 10,
                cell_level_cap: 2,
                maintenance_multiplier: 1.0,
                can_photosynthesize: true,
                can_reproduce: true,
                can_symbiose: true,
                bonuses: StageBonuses {
                    ground_cover: true,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::JeuneArbre,
                biomass_min: 11,
                biomass_max: 15,
                cell_level_cap: 3,
                maintenance_multiplier: 1.5,
                can_photosynthesize: true,
                can_reproduce: true,
                can_symbiose: true,
                bonuses: StageBonuses {
                    canopy_shade: true,
                    seed_distance: 1.5,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::Arbre,
                biomass_min: 16,
                biomass_max: 22,
                cell_level_cap: 4,
                maintenance_multiplier: 2.0,
                can_photosynthesize: true,
                can_reproduce: true,
                can_symbiose: true,
                bonuses: StageBonuses {
                    photosynthesis_multiplier: 1.5,
                    root_network: true,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::ArbreMature,
                biomass_min: 23,
                biomass_max: 30,
                cell_level_cap: 5,
                maintenance_multiplier: 2.5,
                can_photosynthesize: true,
                can_reproduce: true,
                can_symbiose: true,
                bonuses: StageBonuses {
                    reproduction_multiplier: 2.0,
                    symbiosis_hub: 1.5,
                    ..StageBonuses::default()
                },
            },
            StageDefinition {
                name: GrowthStage::Venerable,
                biomass_min: 31,
                biomass_max: 9999,
                cell_level_cap: 5,
                maintenance_multiplier: 3.0,
                can_photosynthesize: true,
                can_reproduce: true,
                can_symbiose: true,
                bonuses: StageBonuses {
                    symbiosis_hub: 3.0,
                    soil_enrichment: true,
                    soil_enrichment_rate: 0.05,
                    resistance: 2.0,
                    ..StageBonuses::default()
                },
            },
        ];

        Self::new(
            stages,
            UpgradeCosts {
                base_cost: 3.0,
                level_multiplier: 1.0,
            },
            SenescenceConfig {
                start_stage: GrowthStage::ArbreMature,
                vulnerability_rate: 0.002,
                max_vulnerability: 0.5,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> StageTable {
        StageTable::default_table()
    }

    #[test]
    fn le_stade_correspond_a_la_biomasse() {
        let t = table();
        assert_eq!(
            t.stage_for_biomass(0, GrowthStage::Venerable),
            GrowthStage::Graine
        );
        assert_eq!(
            t.stage_for_biomass(1, GrowthStage::Venerable),
            GrowthStage::Germe
        );
        assert_eq!(
            t.stage_for_biomass(2, GrowthStage::Venerable),
            GrowthStage::Pousse
        );
        assert_eq!(
            t.stage_for_biomass(4, GrowthStage::Venerable),
            GrowthStage::Plantule
        );
        assert_eq!(
            t.stage_for_biomass(7, GrowthStage::Venerable),
            GrowthStage::Arbuste
        );
        assert_eq!(
            t.stage_for_biomass(11, GrowthStage::Venerable),
            GrowthStage::JeuneArbre
        );
        assert_eq!(
            t.stage_for_biomass(16, GrowthStage::Venerable),
            GrowthStage::Arbre
        );
        assert_eq!(
            t.stage_for_biomass(23, GrowthStage::Venerable),
            GrowthStage::ArbreMature
        );
        assert_eq!(
            t.stage_for_biomass(31, GrowthStage::Venerable),
            GrowthStage::Venerable
        );
        assert_eq!(
            t.stage_for_biomass(100, GrowthStage::Venerable),
            GrowthStage::Venerable
        );
    }

    #[test]
    fn max_stage_plafonne_le_stade() {
        let t = table();
        // Biomasse suffisante pour Venerable, mais max_stage = Arbuste
        assert_eq!(
            t.stage_for_biomass(50, GrowthStage::Arbuste),
            GrowthStage::Arbuste
        );
        // Biomasse suffisante pour Arbre, max_stage = Pousse
        assert_eq!(
            t.stage_for_biomass(20, GrowthStage::Pousse),
            GrowthStage::Pousse
        );
        // Biomasse insuffisante — le max_stage n'a pas d'effet
        assert_eq!(
            t.stage_for_biomass(1, GrowthStage::Venerable),
            GrowthStage::Germe
        );
    }

    #[test]
    fn definition_retourne_le_bon_stade() {
        let t = table();
        let def = t.definition(GrowthStage::Pousse);
        assert_eq!(def.name, GrowthStage::Pousse);
        assert_eq!(def.biomass_min, 2);
        assert_eq!(def.biomass_max, 3);
        assert_eq!(def.cell_level_cap, 1);
        assert!(def.can_photosynthesize);
        assert!(!def.can_reproduce);
        assert!((def.bonuses.growth_speed - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn la_table_par_defaut_a_neuf_stades() {
        let t = table();
        assert_eq!(t.stages.len(), 9);
    }

    #[test]
    fn les_bonus_par_defaut_sont_neutres() {
        let b = StageBonuses::default();
        assert!(!b.invisibility);
        assert!((b.growth_speed - 1.0).abs() < f32::EPSILON);
        assert!((b.resilience - 1.0).abs() < f32::EPSILON);
        assert!(!b.ground_cover);
        assert!(!b.canopy_shade);
        assert!((b.seed_distance - 1.0).abs() < f32::EPSILON);
        assert!((b.photosynthesis_multiplier - 1.0).abs() < f32::EPSILON);
        assert!(!b.root_network);
        assert!((b.reproduction_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((b.symbiosis_hub - 1.0).abs() < f32::EPSILON);
        assert!(!b.soil_enrichment);
        assert!(b.soil_enrichment_rate.abs() < f32::EPSILON);
        assert!((b.resistance - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn le_venerable_a_les_bons_bonus() {
        let t = table();
        let def = t.definition(GrowthStage::Venerable);
        assert!((def.bonuses.symbiosis_hub - 3.0).abs() < f32::EPSILON);
        assert!(def.bonuses.soil_enrichment);
        assert!((def.bonuses.soil_enrichment_rate - 0.05).abs() < f32::EPSILON);
        assert!((def.bonuses.resistance - 2.0).abs() < f32::EPSILON);
    }
}
