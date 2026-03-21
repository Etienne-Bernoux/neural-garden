// Sauvegarde et chargement du SimState complet (JSON).

use std::fs;
use std::path::Path;

use crate::application::sim::SimState;

use super::dto::SimStateDto;

/// Sauvegarde un SimState dans un fichier JSON.
pub fn save_state(state: &SimState, path: &Path) -> Result<(), String> {
    // Creer le dossier parent si necessaire
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("impossible de creer le dossier {:?}: {}", parent, e))?;
    }

    let dto = SimStateDto::from(state);
    let json = serde_json::to_string_pretty(&dto)
        .map_err(|e| format!("erreur de serialisation: {}", e))?;

    fs::write(path, json).map_err(|e| format!("impossible d'ecrire {:?}: {}", path, e))
}

/// Charge un SimState depuis un fichier JSON.
pub fn load_state(path: &Path) -> Result<SimState, String> {
    let json =
        fs::read_to_string(path).map_err(|e| format!("impossible de lire {:?}: {}", path, e))?;

    let dto: SimStateDto =
        serde_json::from_str(&json).map_err(|e| format!("erreur de deserialisation: {}", e))?;

    dto.to_domain()
        .ok_or_else(|| "reconstruction du SimState echouee (composant invalide)".to_string())
}

/// Sauvegarde automatique dans un slot numerote (auto_001.json, auto_002.json, ...).
pub fn auto_save(state: &SimState, save_dir: &Path, slot: usize) -> Result<(), String> {
    let filename = format!("auto_{:03}.json", slot);
    let path = save_dir.join(filename);
    save_state(state, &path)
}

/// Retourne le slot d'auto-save a utiliser pour un tick donne.
/// Rotation circulaire sur `num_slots` slots, avec un intervalle de `interval` ticks.
pub fn get_auto_save_slot(tick: u32, num_slots: usize, interval: u32) -> usize {
    ((tick / interval) % num_slots as u32) as usize + 1
}

/// Indique si une sauvegarde automatique doit etre effectuee a ce tick.
pub fn should_auto_save(tick: u32, interval: u32) -> bool {
    tick > 0 && tick.is_multiple_of(interval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rng::test_utils::MockRng;

    #[test]
    fn sauvegarde_et_chargement_roundtrip() {
        let mut rng = MockRng::new(0.3, 0.07);
        let mut state = SimState::new(0.5, 3, &mut rng);
        // Avancer quelques ticks pour avoir un etat non-trivial
        state.tick_count = 42;

        let dir = std::env::temp_dir().join("neural_garden_test_persistence");
        let path = dir.join("test_roundtrip.json");

        // Sauvegarder
        save_state(&state, &path).expect("sauvegarde echouee");

        // Charger
        let loaded = load_state(&path).expect("chargement echoue");

        // Verifier
        assert_eq!(loaded.tick_count, 42);
        assert_eq!(loaded.plants.len(), state.plants.len());

        // Nettoyage
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn le_json_corrompu_retourne_une_erreur() {
        let dir = std::env::temp_dir().join("neural_garden_test_json_corrompu");
        fs::create_dir_all(&dir).ok();
        let path = dir.join("corrompu.json");
        fs::write(&path, "{ ceci n'est pas du json valide }").expect("ecriture fichier temporaire");

        let result = load_state(&path);
        assert!(
            result.is_err(),
            "un JSON corrompu doit retourner une erreur"
        );

        // Nettoyage
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn le_slot_auto_save_est_correct() {
        // 3 slots, intervalle 1000
        // tick/interval = 1,2,3,4,5 → %3 = 1,2,0,1,2 → +1 = 2,3,1,2,3
        assert_eq!(get_auto_save_slot(1000, 3, 1000), 2);
        assert_eq!(get_auto_save_slot(2000, 3, 1000), 3);
        assert_eq!(get_auto_save_slot(3000, 3, 1000), 1);
        // Rotation : tick 4000 revient au slot 2
        assert_eq!(get_auto_save_slot(4000, 3, 1000), 2);
        assert_eq!(get_auto_save_slot(5000, 3, 1000), 3);
    }

    #[test]
    fn should_auto_save_correct() {
        // tick 0 ne declenche jamais
        assert!(!should_auto_save(0, 1000));
        // tick multiple de l'intervalle
        assert!(should_auto_save(1000, 1000));
        assert!(should_auto_save(2000, 1000));
        // tick non-multiple
        assert!(!should_auto_save(500, 1000));
        assert!(!should_auto_save(1001, 1000));
    }
}
