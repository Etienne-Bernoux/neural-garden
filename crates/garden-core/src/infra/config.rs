// Chargement de la configuration TOML et fusion avec les valeurs par defaut.

use crate::application::config::SimConfig;
use serde::Deserialize;
use std::path::Path;

/// Configuration TOML exposee a l'utilisateur.
/// Seuls les parametres de gameplay sont configurables.
#[derive(Deserialize)]
pub struct TomlConfig {
    pub simulation: Option<SimulationConfig>,
}

#[derive(Deserialize)]
pub struct SimulationConfig {
    pub seed: Option<u64>,
    pub initial_population: Option<usize>,
    pub seed_bank_capacity: Option<usize>,
    pub ticks_per_season: Option<u32>,
}

/// Charge un fichier TOML et fusionne avec les defaults de SimConfig.
/// Retourne (SimConfig, seed). Si le fichier n'existe pas, retourne les defaults avec seed = 42.
pub fn load_config(path: &Path) -> Result<(SimConfig, u64), String> {
    let default_seed: u64 = 42;
    let defaults = SimConfig::default();
    let default_ticks_per_season: u32 = 250;

    if !path.exists() {
        return Ok((defaults, default_seed));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Impossible de lire le fichier {:?} : {}", path, e))?;

    let toml_config: TomlConfig =
        toml::from_str(&content).map_err(|e| format!("Erreur de parsing TOML : {}", e))?;

    let sim = toml_config.simulation;

    match sim {
        None => Ok((defaults, default_seed)),
        Some(s) => {
            let seed = s.seed.unwrap_or(default_seed);

            let config = SimConfig {
                initial_population: s.initial_population.unwrap_or(defaults.initial_population),
                seed_bank_capacity: s.seed_bank_capacity.unwrap_or(defaults.seed_bank_capacity),
                ticks_per_season: s.ticks_per_season.unwrap_or(default_ticks_per_season),
                ..defaults
            };

            Ok((config, seed))
        }
    }
}

/// Genere le contenu d'un fichier garden.toml par defaut avec commentaires.
pub fn generate_default_toml() -> String {
    "# Configuration Neural Garden\n\
     \n\
     [simulation]\n\
     # Seed pour la reproductibilite (u64)\n\
     seed = 42\n\
     # Nombre de plantes au demarrage\n\
     initial_population = 30\n\
     # Taille de la banque de graines\n\
     seed_bank_capacity = 50\n\
     # Duree d'une saison en ticks (1 an = 4 * ticks_per_season)\n\
     ticks_per_season = 250\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_toml_par_defaut_est_valide() {
        let content = generate_default_toml();
        let parsed: TomlConfig =
            toml::from_str(&content).expect("Le TOML par defaut doit etre valide");

        let sim = parsed
            .simulation
            .expect("La section [simulation] doit exister");
        assert_eq!(sim.seed, Some(42));
        assert_eq!(sim.initial_population, Some(30));
        assert_eq!(sim.seed_bank_capacity, Some(50));
        assert_eq!(sim.ticks_per_season, Some(250));
    }

    #[test]
    fn le_toml_partiel_fusionne_avec_defaults() {
        // Ecrit un TOML temporaire avec seulement seed = 99
        let dir = std::env::temp_dir().join("neural_garden_test_partiel");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("garden.toml");
        std::fs::write(&path, "[simulation]\nseed = 99\n").expect("Ecriture du fichier temporaire");

        let (config, seed) = load_config(&path).expect("Le chargement doit reussir");

        // Le seed est ecrase
        assert_eq!(seed, 99);
        // Le reste garde les defaults
        let defaults = SimConfig::default();
        assert_eq!(config.initial_population, defaults.initial_population);
        assert_eq!(config.seed_bank_capacity, defaults.seed_bank_capacity);

        // Nettoyage
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn le_toml_invalide_retourne_une_erreur() {
        let dir = std::env::temp_dir().join("neural_garden_test_invalide");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("garden.toml");
        std::fs::write(&path, "[simulation\ninvalid = }").expect("Ecriture du fichier temporaire");

        let result = load_config(&path);
        assert!(
            result.is_err(),
            "un TOML invalide doit retourner une erreur"
        );

        // Nettoyage
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn le_toml_vide_donne_les_defaults() {
        let dir = std::env::temp_dir().join("neural_garden_test_vide");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("garden.toml");
        std::fs::write(&path, "").expect("Ecriture du fichier temporaire");

        let (config, seed) = load_config(&path).expect("Le chargement doit reussir");

        let defaults = SimConfig::default();
        assert_eq!(seed, 42);
        assert_eq!(config.initial_population, defaults.initial_population);
        assert_eq!(config.seed_bank_capacity, defaults.seed_bank_capacity);

        // Nettoyage
        std::fs::remove_file(&path).ok();
    }
}
