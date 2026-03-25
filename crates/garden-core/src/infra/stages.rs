//! Chargement des stades de croissance depuis un fichier YAML.
//!
//! Ce module définit des DTOs serde intermédiaires puis convertit
//! vers les types domain (StageTable, StageDefinition, etc.).

use crate::domain::stages::{
    GrowthStage, SenescenceConfig, StageBonuses, StageDefinition, StageTable, UpgradeCosts,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ── DTOs serde (miroir du YAML) ──────────────────────────────────────

#[derive(Debug, Deserialize)]
struct YamlStageConfig {
    stages: Vec<YamlStageDefinition>,
    upgrade_costs: YamlUpgradeCosts,
    senescence: YamlSenescence,
}

#[derive(Debug, Deserialize)]
struct YamlStageDefinition {
    name: String,
    biomass_min: u16,
    biomass_max: u16,
    cell_level_cap: u8,
    maintenance_multiplier: f32,
    can_photosynthesize: bool,
    can_reproduce: bool,
    can_symbiose: bool,
    #[serde(default)]
    bonuses: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct YamlUpgradeCosts {
    base_cost: f32,
    level_multiplier: f32,
}

#[derive(Debug, Deserialize)]
struct YamlSenescence {
    start_stage: String,
    vulnerability_rate: f32,
    max_vulnerability: f32,
}

// ── Conversion nom → enum ────────────────────────────────────────────

fn parse_stage_name(name: &str) -> Result<GrowthStage, String> {
    match name {
        "Graine" => Ok(GrowthStage::Graine),
        "Germe" => Ok(GrowthStage::Germe),
        "Pousse" => Ok(GrowthStage::Pousse),
        "Plantule" => Ok(GrowthStage::Plantule),
        "Arbuste" => Ok(GrowthStage::Arbuste),
        "Jeune arbre" => Ok(GrowthStage::JeuneArbre),
        "Arbre" => Ok(GrowthStage::Arbre),
        "Arbre mature" => Ok(GrowthStage::ArbreMature),
        "Vénérable" | "Venerable" => Ok(GrowthStage::Venerable),
        autre => Err(format!("Stade inconnu : «{autre}»")),
    }
}

// ── Conversion des bonus ─────────────────────────────────────────────

/// Parse les bonus depuis la HashMap YAML vers les StageBonuses domain.
/// Les valeurs absentes prennent la valeur par défaut (1.0 pour les multiplicateurs, false pour les flags).
fn parse_bonuses(raw: &HashMap<String, serde_yaml::Value>) -> StageBonuses {
    let defaults = StageBonuses::default();

    let get_f32 = |key: &str, default: f32| -> f32 {
        raw.get(key)
            .and_then(|v| match v {
                serde_yaml::Value::Number(n) => n.as_f64().map(|f| f as f32),
                _ => None,
            })
            .unwrap_or(default)
    };
    let get_bool = |key: &str| -> bool { raw.get(key).and_then(|v| v.as_bool()).unwrap_or(false) };

    StageBonuses {
        invisibility: get_bool("invisibility"),
        growth_speed: get_f32("growth_speed", defaults.growth_speed),
        resilience: get_f32("resilience", defaults.resilience),
        ground_cover: get_bool("ground_cover"),
        canopy_shade: get_bool("canopy_shade"),
        seed_distance: get_f32("seed_distance", defaults.seed_distance),
        photosynthesis_multiplier: get_f32(
            "photosynthesis_multiplier",
            defaults.photosynthesis_multiplier,
        ),
        root_network: get_bool("root_network"),
        reproduction_multiplier: get_f32(
            "reproduction_multiplier",
            defaults.reproduction_multiplier,
        ),
        symbiosis_hub: get_f32("symbiosis_hub", defaults.symbiosis_hub),
        soil_enrichment: get_bool("soil_enrichment"),
        soil_enrichment_rate: get_f32("soil_enrichment_rate", defaults.soil_enrichment_rate),
        resistance: get_f32("resistance", defaults.resistance),
    }
}

// ── Point d'entrée ───────────────────────────────────────────────────

/// Charge et parse le fichier YAML des stades de croissance.
///
/// Retourne une `StageTable` prête à être injectée dans la simulation,
/// ou un message d'erreur lisible si le fichier est invalide.
pub fn load_stages(path: &Path) -> Result<StageTable, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Impossible de lire {}: {e}", path.display()))?;

    let yaml: YamlStageConfig =
        serde_yaml::from_str(&content).map_err(|e| format!("YAML invalide : {e}"))?;

    // Conversion des définitions de stades
    let mut definitions = Vec::with_capacity(yaml.stages.len());
    for raw in &yaml.stages {
        let stage = parse_stage_name(&raw.name)?;
        let bonuses = parse_bonuses(&raw.bonuses);
        definitions.push(StageDefinition {
            name: stage,
            biomass_min: raw.biomass_min,
            biomass_max: raw.biomass_max,
            cell_level_cap: raw.cell_level_cap,
            maintenance_multiplier: raw.maintenance_multiplier,
            can_photosynthesize: raw.can_photosynthesize,
            can_reproduce: raw.can_reproduce,
            can_symbiose: raw.can_symbiose,
            bonuses,
        });
    }

    // Coûts d'upgrade
    let upgrade_costs = UpgradeCosts {
        base_cost: yaml.upgrade_costs.base_cost,
        level_multiplier: yaml.upgrade_costs.level_multiplier,
    };

    // Sénescence
    let senescence = SenescenceConfig {
        start_stage: parse_stage_name(&yaml.senescence.start_stage)?,
        vulnerability_rate: yaml.senescence.vulnerability_rate,
        max_vulnerability: yaml.senescence.max_vulnerability,
    };

    Ok(StageTable::new(definitions, upgrade_costs, senescence))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_le_yaml_des_stades() {
        let table = load_stages(Path::new("../../configs/stages.yaml"))
            .expect("le YAML des stades doit être valide");

        // Vérifications sur Graine
        let graine = table.definition(GrowthStage::Graine);
        assert_eq!(graine.biomass_min, 0);
        assert_eq!(graine.biomass_max, 0);
        assert_eq!(graine.cell_level_cap, 0);
        assert!(!graine.can_photosynthesize);
        assert!(!graine.can_reproduce);

        // Vérifications sur Vénérable
        let venerable = table.definition(GrowthStage::Venerable);
        assert_eq!(venerable.cell_level_cap, 5);
        assert_eq!(venerable.biomass_min, 31);
        assert!(venerable.bonuses.soil_enrichment);
        assert!((venerable.bonuses.symbiosis_hub - 3.0).abs() < f32::EPSILON);

        // Vérifications sur Jeune arbre
        let jeune = table.definition(GrowthStage::JeuneArbre);
        assert_eq!(jeune.cell_level_cap, 3);
        assert!(jeune.bonuses.canopy_shade);
        assert!((jeune.bonuses.seed_distance - 1.5).abs() < f32::EPSILON);

        // Sénescence
        assert_eq!(table.senescence.start_stage, GrowthStage::ArbreMature);
        assert!((table.senescence.vulnerability_rate - 0.002).abs() < f32::EPSILON);

        // Coûts d'upgrade
        assert!((table.upgrade_costs.base_cost - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn erreur_si_fichier_inexistant() {
        let result = load_stages(Path::new("inexistant.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn les_bonus_absents_prennent_la_valeur_par_defaut() {
        let table = load_stages(Path::new("../../configs/stages.yaml"))
            .expect("le YAML des stades doit être valide");

        // Graine n'a aucun bonus — tout doit être aux valeurs par défaut
        let graine = table.definition(GrowthStage::Graine);
        let defaults = StageBonuses::default();
        assert_eq!(graine.bonuses.invisibility, defaults.invisibility);
        assert!((graine.bonuses.growth_speed - defaults.growth_speed).abs() < f32::EPSILON);
        assert!((graine.bonuses.resistance - defaults.resistance).abs() < f32::EPSILON);
    }
}
