// Chargement des environnements nursery depuis un fichier YAML.
// Les structs YAML intermediaires vivent ici (infra) pour ne pas polluer
// application/ avec des derives serde.

use std::path::Path;

use serde::Deserialize;

use crate::application::nursery::{BedConfig, FixtureBehavior, FixtureConfig};
use crate::domain::plant::{ExudateType, Pos};

// --- Structs YAML intermediaires ---

#[derive(Deserialize)]
struct YamlEnvFile {
    environments: Vec<YamlBedConfig>,
}

#[derive(Deserialize)]
struct YamlBedConfig {
    name: String,
    grid_size: u16,
    initial_carbon: f32,
    initial_nitrogen: f32,
    initial_humidity: f32,
    light_level: f32,
    carbon_regen_rate: f32,
    nitrogen_regen_rate: f32,
    humidity_regen_rate: f32,
    max_ticks: u32,
    fixtures: Vec<YamlFixtureConfig>,
}

#[derive(Deserialize)]
struct YamlFixtureConfig {
    position: [u16; 2],
    exudate_type: String,
    biomass: u16,
    behavior: YamlFixtureBehavior,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum YamlFixtureBehavior {
    Exuder { rate: f32 },
    Ombrager { radius: u16 },
    Envahir,
    Inerte,
}

// --- Conversion ---

impl From<YamlFixtureBehavior> for FixtureBehavior {
    fn from(yaml: YamlFixtureBehavior) -> Self {
        match yaml {
            YamlFixtureBehavior::Exuder { rate } => FixtureBehavior::Exuder { rate },
            YamlFixtureBehavior::Ombrager { radius } => FixtureBehavior::Ombrager { radius },
            YamlFixtureBehavior::Envahir => FixtureBehavior::Envahir,
            YamlFixtureBehavior::Inerte => FixtureBehavior::Inerte,
        }
    }
}

fn parse_exudate_type(s: &str) -> ExudateType {
    match s {
        "Nitrogen" => ExudateType::Nitrogen,
        _ => ExudateType::Carbon,
    }
}

impl From<YamlFixtureConfig> for FixtureConfig {
    fn from(yaml: YamlFixtureConfig) -> Self {
        FixtureConfig {
            position: Pos {
                x: yaml.position[0],
                y: yaml.position[1],
            },
            exudate_type: parse_exudate_type(&yaml.exudate_type),
            biomass: yaml.biomass,
            behavior: yaml.behavior.into(),
        }
    }
}

impl From<YamlBedConfig> for (String, BedConfig) {
    fn from(yaml: YamlBedConfig) -> Self {
        let config = BedConfig {
            grid_size: yaml.grid_size,
            initial_carbon: yaml.initial_carbon,
            initial_nitrogen: yaml.initial_nitrogen,
            initial_humidity: yaml.initial_humidity,
            light_level: yaml.light_level,
            carbon_regen_rate: yaml.carbon_regen_rate,
            nitrogen_regen_rate: yaml.nitrogen_regen_rate,
            humidity_regen_rate: yaml.humidity_regen_rate,
            max_ticks: yaml.max_ticks,
            fixtures: yaml.fixtures.into_iter().map(Into::into).collect(),
        };
        (yaml.name, config)
    }
}

// --- API publique ---

/// Charge les environnements depuis un fichier YAML.
pub fn load_nursery_environments(path: &Path) -> Result<Vec<(String, BedConfig)>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Erreur lecture {}: {}", path.display(), e))?;

    let yaml: YamlEnvFile =
        serde_yaml::from_str(&content).map_err(|e| format!("Erreur YAML: {}", e))?;

    let envs = yaml.environments.into_iter().map(Into::into).collect();

    Ok(envs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charger_envs_depuis_yaml() {
        let envs = load_nursery_environments(Path::new("../../configs/nursery/environments.yaml"));
        assert!(envs.is_ok(), "Erreur chargement YAML: {:?}", envs.err());
        let envs = envs.expect("already checked");
        assert_eq!(
            envs.len(),
            10,
            "Expected 10 environments, got {}",
            envs.len()
        );
    }

    #[test]
    fn yaml_contient_les_bons_noms() {
        let envs = load_nursery_environments(Path::new("../../configs/nursery/environments.yaml"))
            .expect("YAML should load");
        let names: Vec<&str> = envs.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"Solo riche"));
        assert!(names.contains(&"Mixte"));
        assert!(names.contains(&"Hiver"));
    }

    #[test]
    fn yaml_fixtures_deserializees_correctement() {
        let envs = load_nursery_environments(Path::new("../../configs/nursery/environments.yaml"))
            .expect("YAML should load");
        let mixte = envs
            .iter()
            .find(|(n, _)| n == "Mixte")
            .expect("Mixte should exist");
        assert_eq!(mixte.1.fixtures.len(), 2);
    }
}
