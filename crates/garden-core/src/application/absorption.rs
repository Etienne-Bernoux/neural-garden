// Absorption — extraction des nutriments du sol par les racines.
// L'absorption totale est proportionnelle a la biomasse, repartie sur les racines.
// Plus la plante est grosse, plus elle absorbe au total, mais moins par cellule.

use super::sim::SimState;
use crate::domain::plant::Pos;

/// Calcule le taux d'absorption par cellule de racine.
/// Formule : absorption_rate × biomass / nb_racines
/// - Petite plante (bio=1, 1 racine) : 0.03 × 1 / 1 = 0.03 par racine (total 0.03)
/// - Grande plante (bio=10, 20 racines) : 0.03 × 10 / 20 = 0.015 par racine (total 0.30)
pub fn absorption_rate_per_root(base_rate: f32, biomass: u16, root_count: usize) -> f32 {
    if root_count == 0 {
        return 0.0;
    }
    base_rate * biomass as f32 / root_count as f32
}

/// Absorbe les nutriments du sol sous les racines de la plante.
/// L'absorption par racine diminue avec le nombre de racines (dilution).
/// L'absorption totale augmente avec la biomasse.
pub fn action_absorption(state: &mut SimState, plant_id: u64, plant_idx: usize) {
    let root_cells: Vec<Pos> = state.plants[plant_idx].roots().to_vec();
    let biomass = state.plants[plant_idx].biomass().value();
    let rate = absorption_rate_per_root(state.config.absorption_rate, biomass, root_cells.len());

    let mut total_absorbed = 0.0_f32;
    for pos in &root_cells {
        if let Some(cell) = state.world.get_mut(pos) {
            let c_absorbed = cell.carbon().min(rate);
            let n_absorbed = cell.nitrogen().min(rate);
            let h_absorbed = cell.humidity().min(rate);

            let c = cell.carbon();
            cell.set_carbon(c - c_absorbed);
            let n = cell.nitrogen();
            cell.set_nitrogen(n - n_absorbed);
            let h = cell.humidity();
            cell.set_humidity(h - h_absorbed);

            total_absorbed += c_absorbed + n_absorbed + h_absorbed;
        }
    }
    state.plants[plant_idx].gain_energy(total_absorbed);

    if let Some(stats) = state.find_stats_mut(plant_id) {
        stats.soil_depleted += total_absorbed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taux_absorption_petite_plante() {
        // biomass=1, 1 racine : taux plein
        let rate = absorption_rate_per_root(0.03, 1, 1);
        assert!(
            (rate - 0.03).abs() < 1e-6,
            "bio=1, 1 racine devrait donner 0.03, got {rate}"
        );
    }

    #[test]
    fn taux_absorption_grande_plante_dilue() {
        // biomass=10, 20 racines : 0.03 × 10 / 20 = 0.015
        let rate = absorption_rate_per_root(0.03, 10, 20);
        assert!(
            (rate - 0.015).abs() < 1e-6,
            "bio=10, 20 racines devrait donner 0.015, got {rate}"
        );
    }

    #[test]
    fn taux_absorption_total_augmente_avec_biomasse() {
        // Petite : 0.03 × 1 / 1 × 1 racine = 0.03 total
        let rate_small = absorption_rate_per_root(0.03, 1, 1);
        let total_small = rate_small * 1.0; // 1 racine

        // Grande : 0.03 × 10 / 20 × 20 racines = 0.30 total
        let rate_big = absorption_rate_per_root(0.03, 10, 20);
        let total_big = rate_big * 20.0; // 20 racines

        assert!(
            total_big > total_small,
            "une grande plante devrait absorber plus au total: small={total_small}, big={total_big}"
        );
    }

    #[test]
    fn taux_absorption_par_cellule_diminue_avec_racines() {
        // Plus de racines = moins d'absorption par cellule
        let rate_few = absorption_rate_per_root(0.03, 5, 5);
        let rate_many = absorption_rate_per_root(0.03, 5, 20);
        assert!(
            rate_few > rate_many,
            "plus de racines devrait diluer l'absorption: few={rate_few}, many={rate_many}"
        );
    }

    #[test]
    fn zero_racines_donne_zero_absorption() {
        let rate = absorption_rate_per_root(0.03, 5, 0);
        assert!(rate.abs() < 1e-6, "0 racines devrait donner 0 absorption");
    }
}
