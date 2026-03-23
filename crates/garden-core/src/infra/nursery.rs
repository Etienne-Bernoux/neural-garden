// Chargement des environnements nursery depuis un fichier YAML.
// Persistance intermediaire et export/import des champions.
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

// --- Parallelisation ---

use rayon::prelude::*;

use crate::application::evolution::Genome;
use crate::application::nursery::{run_nursery_env, NurseryResult};
use crate::infra::rng::SeededRng;

/// Lance la pepiniere sur tous les environnements en parallele.
/// Chaque environnement recoit un seed distinct derive du seed de base.
/// La parallelisation (rayon) et la creation des SeededRng vivent ici dans infra.
pub fn run_nursery_all(
    envs: &[(String, BedConfig)],
    generations: u32,
    population: usize,
    seed: u64,
) -> Vec<NurseryResult> {
    envs.par_iter()
        .enumerate()
        .map(|(i, (name, config))| {
            // Chaque env a son propre seed pour la reproductibilite
            let mut rng = SeededRng::new(seed + i as u64);
            run_nursery_env(name, config, generations, population, &mut rng, None)
        })
        .collect()
}

// --- Persistance intermediaire ---

use crate::infra::dto::GenomeDto;
use serde::Serialize;
use std::fs;

/// Struct intermediaire pour sauvegarder un genome avec sa fitness.
#[derive(Serialize, Deserialize)]
struct ScoredGenomeDto {
    genome: GenomeDto,
    fitness: f32,
}

/// Sauvegarde les top genomes d'une generation dans un fichier JSON.
pub fn save_generation(
    dir: &Path,
    env_name: &str,
    gen: u32,
    top: &[(Genome, f32)],
) -> Result<(), String> {
    let env_dir = dir.join(env_name.replace(' ', "_"));
    fs::create_dir_all(&env_dir).map_err(|e| e.to_string())?;

    let path = env_dir.join(format!("gen_{:04}.json", gen));

    let dtos: Vec<ScoredGenomeDto> = top
        .iter()
        .map(|(g, f)| ScoredGenomeDto {
            genome: GenomeDto::from(g),
            fitness: *f,
        })
        .collect();

    let json = serde_json::to_string_pretty(&dtos).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Exporte les champions (1 par env) dans des fichiers JSON individuels.
pub fn export_champions(results: &[NurseryResult], output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    for result in results {
        let filename = result.env_name.to_lowercase().replace(' ', "_") + ".json";
        let path = output_dir.join(filename);

        let dto = GenomeDto::from(&result.champion);
        let json = serde_json::to_string_pretty(&dto).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Charge les champions depuis un dossier contenant des fichiers JSON de genomes.
pub fn load_champions(dir: &Path) -> Result<Vec<Genome>, String> {
    let mut genomes = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let dto: GenomeDto = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            let genome = dto
                .to_domain()
                .ok_or_else(|| format!("Genome invalide dans {}", path.display()))?;
            genomes.push(genome);
        }
    }

    Ok(genomes)
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

    #[test]
    fn export_et_reload_champions() {
        use crate::application::nursery::BedConfig;
        use crate::infra::rng::SeededRng;

        let mut rng = SeededRng::new(42);
        let config = BedConfig::default();
        let result = run_nursery_env("test_export", &config, 3, 10, &mut rng, None);

        let dir = std::env::temp_dir().join("neural_garden_test_seeds");
        let _ = fs::remove_dir_all(&dir);

        export_champions(&[result], &dir).expect("export devrait fonctionner");
        let loaded = load_champions(&dir).expect("load devrait fonctionner");

        assert_eq!(loaded.len(), 1, "devrait charger 1 champion");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_nursery_all_retourne_un_resultat_par_env() {
        use crate::application::nursery::BedConfig;

        let envs = vec![
            ("env_a".to_string(), BedConfig::default()),
            (
                "env_b".to_string(),
                BedConfig {
                    light_level: 0.5,
                    ..BedConfig::default()
                },
            ),
        ];
        let results = run_nursery_all(&envs, 2, 10, 42);
        assert_eq!(results.len(), 2);
        // Chaque resultat a le bon nom d'environnement
        let names: Vec<&str> = results.iter().map(|r| r.env_name.as_str()).collect();
        assert!(names.contains(&"env_a"));
        assert!(names.contains(&"env_b"));
    }

    #[test]
    fn save_generation_cree_le_fichier() {
        use crate::application::evolution::SeedBank;
        use crate::domain::rng::test_utils::MockRng;

        let dir = std::env::temp_dir().join("neural_garden_test_gen");
        let _ = fs::remove_dir_all(&dir);

        let mut rng = MockRng::new(0.42, 0.07);
        let g1 = SeedBank::produce_fresh_seed(&mut rng);
        let g2 = SeedBank::produce_fresh_seed(&mut rng);
        let top = vec![(g1, 100.0), (g2, 80.0)];

        save_generation(&dir, "Solo riche", 3, &top).expect("save devrait fonctionner");

        let path = dir.join("Solo_riche").join("gen_0003.json");
        assert!(path.exists(), "le fichier gen_0003.json devrait exister");

        let _ = fs::remove_dir_all(&dir);
    }
}
