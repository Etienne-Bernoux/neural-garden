// Service d'evolution : genome, fitness, banque de graines, crossover et mutation.

use crate::domain::brain::Brain;
use crate::domain::plant::{ExudateType, GeneticTraits};
use crate::domain::rng::Rng;
use std::collections::HashMap;

/// Taux de crossover : probabilite de combiner les poids des deux parents
/// plutot que de cloner le meilleur.
const CROSSOVER_RATE: f32 = 0.7;

/// Genome complet d'une plante : cerveau + traits genetiques.
#[derive(Debug, Clone)]
pub struct Genome {
    pub brain: Brain,
    pub traits: GeneticTraits,
}

/// Statistiques accumulees pendant la vie d'une plante.
/// Utilisees pour calculer la fitness a la mort.
#[derive(Debug, Clone, Default)]
pub struct PlantStats {
    pub max_biomass: u16,
    pub lifetime: u32,
    pub max_territory: u16,
    pub symbiotic_connections: u32,
    pub exudates_emitted: f32,
    pub cn_exchanges: f32,
    pub seeds_produced: u32,
    pub soil_enriched: f32,
    pub soil_depleted: f32,
    pub monoculture_penalty: f32,
    /// Fitness heritee du parent (30% de la fitness estimee du parent a la reproduction).
    pub inherited_fitness: f32,
}

/// Calcule la fitness d'une plante a sa mort.
/// La reproduction (seeds_produced × 500) est le facteur dominant.
/// La cooperation reste importante comme moyen de survivre plus longtemps
/// → plus de reproductions.
pub fn evaluate_fitness(stats: &PlantStats) -> f32 {
    let own_fitness = stats.max_biomass as f32 * 0.5       // grandir = peu
        + stats.lifetime as f32 * 0.01                     // survivre = quasi rien
        + stats.max_territory as f32 * 0.3                 // territoire = peu
        + stats.symbiotic_connections as f32 * 100.0       // liens = bien (reduit de 500)
        + stats.exudates_emitted * 50.0                    // exsuder = moyen (reduit de 100)
        + stats.cn_exchanges * 500.0                       // echanger C/N = bien (reduit de 5000)
        + stats.seeds_produced as f32 * 500.0              // se reproduire = DOMINANT (monte de 50)
        + stats.soil_enriched * 10.0                       // enrichir le sol = moyen
        - stats.soil_depleted * 1.0                        // penalite legere
        - stats.monoculture_penalty * 5.0; // penalite monoculture

    (own_fitness + stats.inherited_fitness).max(0.0)
}

/// Cle de compartiment : (hidden_size, is_carbon)
/// is_carbon = true pour ExudateType::Carbon, false pour Nitrogen
type CompartmentKey = (u8, bool);

/// Convertit un ExudateType en bool pour la cle de compartiment.
fn exudate_to_bool(e: ExudateType) -> bool {
    matches!(e, ExudateType::Carbon)
}

/// Extrait la cle de compartiment d'un genome.
fn compartment_key(genome: &Genome) -> CompartmentKey {
    (
        genome.traits.hidden_size(),
        exudate_to_bool(genome.traits.exudate_type()),
    )
}

/// Banque de graines compartimentee par (hidden_size, exudate_type).
/// Maintient la diversite genetique en repartissant les genomes dans des compartiments.
pub struct SeedBank {
    compartments: HashMap<CompartmentKey, Vec<(Genome, f32)>>,
    capacity: usize,
}

impl SeedBank {
    /// Cree une banque vide avec la capacite donnee.
    pub fn new(capacity: usize) -> Self {
        Self {
            compartments: HashMap::new(),
            capacity,
        }
    }

    /// Remplit la banque avec `count` genomes aleatoires, fitness initiale 0.
    pub fn initialize(&mut self, count: usize, rng: &mut dyn Rng) {
        for _ in 0..count {
            let genome = Self::produce_fresh_seed(rng);
            let key = compartment_key(&genome);
            self.compartments
                .entry(key)
                .or_default()
                .push((genome, 0.0));
        }
    }

    /// Produit un genome totalement aleatoire avec le biais grow_intensity.
    pub fn produce_fresh_seed(rng: &mut dyn Rng) -> Genome {
        let hidden_size = 6 + (rng.next_f32() * 9.0) as u8;
        let max_size = 15 + (rng.next_f32() * 26.0) as u16;
        let carbon_nitrogen_ratio = 0.3 + rng.next_f32() * 0.6;
        let exudate_type = if rng.next_f32() < 0.5 {
            ExudateType::Carbon
        } else {
            ExudateType::Nitrogen
        };
        let vitality_factor = 8.0 + rng.next_f32() * 4.0;
        let energy_factor = 4.0 + rng.next_f32() * 4.0;

        let brain = Brain::new(hidden_size, rng);

        // Biais de survie : forcer grow_intensity (output[0]) positif par defaut
        let hs = hidden_size as usize;
        let bias_o_start = 18 * hs + hs * hs + hs * 8 + hs + hs;
        let mut weights = brain.weights();
        if bias_o_start + 6 < weights.len() {
            // Biais de survie : grow_intensity (output[0]) positif
            weights[bias_o_start] = weights[bias_o_start].abs() + 0.5;
            // Biais de cooperation : connect_signal (output[6]) positif
            weights[bias_o_start + 6] = weights[bias_o_start + 6].abs() + 0.5;
        }
        let brain = Brain::from_weights(hidden_size, weights).unwrap_or(brain);

        let traits = GeneticTraits::new(
            max_size,
            carbon_nitrogen_ratio,
            exudate_type,
            hidden_size,
            vitality_factor,
            energy_factor,
        );

        Genome { brain, traits }
    }

    /// Tente d'inserer un genome avec sa fitness.
    /// Si la capacite totale est depassee, evince le pire du compartiment le plus peuple.
    /// Retourne true (insere toujours).
    pub fn try_insert(&mut self, genome: Genome, fitness: f32) -> bool {
        let key = compartment_key(&genome);
        self.compartments
            .entry(key)
            .or_default()
            .push((genome, fitness));

        // Si depassement de capacite, evincer le pire du compartiment le plus peuple
        if self.len() > self.capacity {
            self.evict_worst_from_largest();
        }
        true
    }

    /// Insere un genome et remplace la moitie basse de son compartiment par des mutations.
    /// Retourne true (insere toujours).
    pub fn try_insert_and_spread(
        &mut self,
        genome: Genome,
        fitness: f32,
        rng: &mut dyn Rng,
    ) -> bool {
        let key = compartment_key(&genome);
        let compartment = self.compartments.entry(key).or_default();
        compartment.push((genome.clone(), fitness));

        // Trier par fitness decroissante
        compartment.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

        // Remplacer la moitie basse par des mutations du nouveau genome
        let half = compartment.len() / 2;
        let start = compartment.len() - half;
        for entry in compartment.iter_mut().skip(start) {
            let mut mutated = genome.clone();
            mutate_genome(&mut mutated, rng);
            *entry = (mutated, fitness * 0.8);
        }

        // Evincer si depassement de capacite
        while self.len() > self.capacity {
            self.evict_worst_from_largest();
        }
        true
    }

    /// Evince le genome avec la pire fitness du compartiment le plus peuple.
    fn evict_worst_from_largest(&mut self) {
        // Trouver le compartiment avec le plus de genomes
        let largest_key = self
            .compartments
            .iter()
            .max_by_key(|(_, v)| v.len())
            .map(|(k, _)| *k);

        if let Some(key) = largest_key {
            if let Some(compartment) = self.compartments.get_mut(&key) {
                // Trouver l'index du pire
                if let Some((worst_idx, _)) =
                    compartment.iter().enumerate().min_by(|(_, a), (_, b)| {
                        a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal)
                    })
                {
                    compartment.swap_remove(worst_idx);
                }
                // Supprimer le compartiment s'il est vide
                if compartment.is_empty() {
                    self.compartments.remove(&key);
                }
            }
        }
    }

    /// Produit une graine en combinant deux parents du meme compartiment.
    /// Choisit un compartiment aleatoire pondere par le nombre de genomes.
    pub fn produce_seed(&self, rng: &mut dyn Rng) -> Genome {
        // Choix pondere d'un compartiment
        let total = self.len();
        let mut pick = (rng.next_f32() * total as f32) as usize;
        pick = pick.min(total.saturating_sub(1));

        let mut selected_entries: Option<&Vec<(Genome, f32)>> = None;
        let mut cumul = 0;
        for entries in self.compartments.values() {
            cumul += entries.len();
            if pick < cumul {
                selected_entries = Some(entries);
                break;
            }
        }

        // Fallback : premier compartiment non vide
        let entries = selected_entries.unwrap_or_else(|| {
            self.compartments
                .values()
                .next()
                .expect("produce_seed appelee sur banque non vide")
        });

        let idx_a = (rng.next_f32() * entries.len() as f32) as usize;
        let idx_a = idx_a.min(entries.len() - 1);
        let mut idx_b = (rng.next_f32() * entries.len() as f32) as usize;
        // Eviter de tirer le meme parent
        if idx_b == idx_a && entries.len() > 1 {
            idx_b = (idx_b + 1) % entries.len();
        }
        let idx_b = idx_b.min(entries.len() - 1);

        let (parent_a, fitness_a) = &entries[idx_a];
        let (parent_b, fitness_b) = &entries[idx_b];

        // Dans le meme compartiment, hidden_size est forcement identique
        let mut child = if rng.next_f32() < CROSSOVER_RATE {
            let brain = crossover_brains(&parent_a.brain, &parent_b.brain, rng);
            let traits = crossover_traits(&parent_a.traits, &parent_b.traits, rng);
            Genome { brain, traits }
        } else {
            // Clone du meilleur parent
            if fitness_a >= fitness_b {
                parent_a.clone()
            } else {
                parent_b.clone()
            }
        };

        mutate_genome(&mut child, rng);
        child
    }

    /// Fitness du meilleur genome (tous compartiments confondus).
    pub fn best_fitness(&self) -> f32 {
        self.compartments
            .values()
            .flat_map(|v| v.iter())
            .map(|(_, f)| *f)
            .fold(0.0_f32, f32::max)
    }

    /// Fitness du pire genome (tous compartiments confondus).
    pub fn worst_fitness(&self) -> f32 {
        self.compartments
            .values()
            .flat_map(|v| v.iter())
            .map(|(_, f)| *f)
            .fold(f32::MAX, f32::min)
            .min(self.best_fitness()) // Si vide, retourner 0.0 via best_fitness
    }

    /// Nombre total de genomes dans la banque.
    pub fn len(&self) -> usize {
        self.compartments.values().map(|v| v.len()).sum()
    }

    /// Indique si la banque est vide.
    pub fn is_empty(&self) -> bool {
        self.compartments.values().all(|v| v.is_empty())
    }

    /// Nombre de compartiments actifs (non vides).
    pub fn compartment_count(&self) -> usize {
        self.compartments.values().filter(|v| !v.is_empty()).count()
    }

    /// Indicateur de diversite : (best - worst) / best si best > 0, sinon 0.
    pub fn diversity_spread(&self) -> f32 {
        let best = self.best_fitness();
        if best > 0.0 {
            (best - self.worst_fitness()) / best
        } else {
            0.0
        }
    }

    /// Retourne tous les genomes a plat (pour serialisation).
    pub fn entries(&self) -> Vec<(&Genome, f32)> {
        self.compartments
            .values()
            .flat_map(|v| v.iter().map(|(g, f)| (g, *f)))
            .collect()
    }

    /// Reconstruit les compartiments depuis une liste plate (pour deserialisation).
    pub fn from_entries(entries: Vec<(Genome, f32)>, capacity: usize) -> Self {
        let mut compartments: HashMap<CompartmentKey, Vec<(Genome, f32)>> = HashMap::new();
        for (genome, fitness) in entries {
            let key = compartment_key(&genome);
            compartments.entry(key).or_default().push((genome, fitness));
        }
        Self {
            compartments,
            capacity,
        }
    }

    /// Retourne la capacite maximale de la banque.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Crossover uniforme de deux cerveaux de meme hidden_size.
/// Retourne un clone du parent A si la reconstruction echoue (ne devrait pas arriver).
fn crossover_brains(a: &Brain, b: &Brain, rng: &mut dyn Rng) -> Brain {
    let weights_a = a.weights();
    let weights_b = b.weights();
    let child_weights: Vec<f32> = weights_a
        .iter()
        .zip(weights_b.iter())
        .map(|(&wa, &wb)| if rng.next_f32() < 0.5 { wa } else { wb })
        .collect();
    Brain::from_weights(a.hidden_size(), child_weights).unwrap_or_else(|| a.clone())
}

/// Crossover des traits genetiques (meme hidden_size requis).
fn crossover_traits(a: &GeneticTraits, b: &GeneticTraits, rng: &mut dyn Rng) -> GeneticTraits {
    let max_size = ((a.max_size() as f32 + b.max_size() as f32) / 2.0).round() as u16;
    let carbon_nitrogen_ratio = (a.carbon_nitrogen_ratio() + b.carbon_nitrogen_ratio()) / 2.0;
    let exudate_type = if rng.next_f32() < 0.5 {
        a.exudate_type()
    } else {
        b.exudate_type()
    };
    let vitality_factor = (a.vitality_factor() + b.vitality_factor()) / 2.0;
    let energy_factor = (a.energy_factor() + b.energy_factor()) / 2.0;

    GeneticTraits::new(
        max_size,
        carbon_nitrogen_ratio,
        exudate_type,
        a.hidden_size(),
        vitality_factor,
        energy_factor,
    )
}

/// Genere un bruit gaussien via Box-Muller.
fn gaussian(rng: &mut dyn Rng, sigma: f32) -> f32 {
    let u1 = rng.next_f32().max(1e-7);
    let u2 = rng.next_f32();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * core::f32::consts::PI * u2).cos();
    z * sigma
}

/// Applique des mutations a un genome.
pub fn mutate_genome(genome: &mut Genome, rng: &mut dyn Rng) {
    // Mutation des poids du reseau
    let mut weights = genome.brain.weights();
    for w in weights.iter_mut() {
        if rng.next_f32() < 0.3 {
            *w += gaussian(rng, 0.2);
        }
    }
    if let Some(new_brain) = Brain::from_weights(genome.brain.hidden_size(), weights) {
        genome.brain = new_brain;
    }

    // Mutation de carbon_nitrogen_ratio
    if rng.next_f32() < 0.3 {
        let new_cnr = genome.traits.carbon_nitrogen_ratio() + gaussian(rng, 0.05);
        genome.traits = GeneticTraits::new(
            genome.traits.max_size(),
            new_cnr,
            genome.traits.exudate_type(),
            genome.traits.hidden_size(),
            genome.traits.vitality_factor(),
            genome.traits.energy_factor(),
        );
    }

    // Mutation de max_size
    if rng.next_f32() < 0.1 {
        let delta = gaussian(rng, 2.0).round() as i32;
        let new_max = (genome.traits.max_size() as i32 + delta).clamp(15, 40) as u16;
        genome.traits = GeneticTraits::new(
            new_max,
            genome.traits.carbon_nitrogen_ratio(),
            genome.traits.exudate_type(),
            genome.traits.hidden_size(),
            genome.traits.vitality_factor(),
            genome.traits.energy_factor(),
        );
    }

    // Mutation de exudate_type
    if rng.next_f32() < 0.01 {
        let flipped = match genome.traits.exudate_type() {
            ExudateType::Carbon => ExudateType::Nitrogen,
            ExudateType::Nitrogen => ExudateType::Carbon,
        };
        genome.traits = GeneticTraits::new(
            genome.traits.max_size(),
            genome.traits.carbon_nitrogen_ratio(),
            flipped,
            genome.traits.hidden_size(),
            genome.traits.vitality_factor(),
            genome.traits.energy_factor(),
        );
    }

    // Mutation de hidden_size
    if rng.next_f32() < 0.05 {
        let delta: i8 = if rng.next_f32() < 0.5 { -1 } else { 1 };
        let new_hs = (genome.traits.hidden_size() as i8 + delta).clamp(6, 14) as u8;
        if new_hs != genome.traits.hidden_size() {
            genome.brain = genome.brain.clone().resize(new_hs);
            genome.traits = GeneticTraits::new(
                genome.traits.max_size(),
                genome.traits.carbon_nitrogen_ratio(),
                genome.traits.exudate_type(),
                new_hs,
                genome.traits.vitality_factor(),
                genome.traits.energy_factor(),
            );
        }
    }
}

/// Compteur global de generation.
pub struct GenerationCounter {
    count: u64,
}

impl GenerationCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.count += 1;
        self.count
    }

    pub fn current(&self) -> u64 {
        self.count
    }

    /// Reconstruit un compteur a partir d'une valeur brute.
    /// Utilise pour la deserialisation.
    pub(crate) fn from_count(count: u64) -> Self {
        Self { count }
    }
}

impl Default for GenerationCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::rng::test_utils::MockRng;

    fn make_genome(rng: &mut dyn Rng) -> Genome {
        let traits = GeneticTraits::new(20, 0.5, ExudateType::Carbon, 8, 10.0, 5.0);
        let brain = Brain::new(8, rng);
        Genome { brain, traits }
    }

    fn make_genome_nitrogen(rng: &mut dyn Rng) -> Genome {
        let traits = GeneticTraits::new(20, 0.5, ExudateType::Nitrogen, 10, 10.0, 5.0);
        let brain = Brain::new(10, rng);
        Genome { brain, traits }
    }

    #[test]
    fn la_fitness_est_positive() {
        let stats = PlantStats::default();
        assert_eq!(evaluate_fitness(&stats), 0.0);
    }

    #[test]
    fn la_fitness_inclut_toutes_les_composantes() {
        let stats = PlantStats {
            max_biomass: 10,
            lifetime: 100,
            max_territory: 5,
            symbiotic_connections: 3,
            exudates_emitted: 2.0,
            cn_exchanges: 1.0,
            seeds_produced: 4,
            soil_enriched: 3.0,
            soil_depleted: 1.0,
            monoculture_penalty: 0.5,
            inherited_fitness: 0.0,
        };
        let fitness = evaluate_fitness(&stats);
        assert!(fitness > 0.0);
        // own_fitness = 5 + 1 + 1.5 + 300 + 100 + 500 + 2000 + 30 - 1 - 2.5 = 2934.0
        // inherited_fitness = 0.0 (default)
        assert!((fitness - 2934.0).abs() < 0.01);
    }

    #[test]
    fn la_banque_sinsere_et_trie() {
        let mut bank = SeedBank::new(10);
        let mut rng = MockRng::new(0.1, 0.13);
        let g1 = make_genome(&mut rng);
        let g2 = make_genome(&mut rng);
        let g3 = make_genome(&mut rng);

        bank.try_insert(g1, 10.0);
        bank.try_insert(g2, 30.0);
        bank.try_insert(g3, 20.0);

        assert_eq!(bank.len(), 3);
        assert_eq!(bank.best_fitness(), 30.0);
        assert_eq!(bank.worst_fitness(), 10.0);
    }

    #[test]
    fn la_banque_remplace_le_pire() {
        let mut bank = SeedBank::new(3);
        let mut rng = MockRng::new(0.2, 0.13);
        let g1 = make_genome(&mut rng);
        let g2 = make_genome(&mut rng);
        let g3 = make_genome(&mut rng);
        let g4 = make_genome(&mut rng);

        bank.try_insert(g1, 10.0);
        bank.try_insert(g2, 20.0);
        bank.try_insert(g3, 5.0);
        assert_eq!(bank.len(), 3);

        // Inserer un 4eme → evince le pire du compartiment le plus peuple
        let inserted = bank.try_insert(g4, 15.0);
        assert!(inserted);
        assert_eq!(bank.len(), 3);
        // Le pire (5.0) a ete evince
        assert!(bank.worst_fitness() >= 10.0);
    }

    #[test]
    fn la_banque_maintient_la_diversite() {
        let mut bank = SeedBank::new(20);
        let mut rng = MockRng::new(0.3, 0.07);

        // Inserer des genomes Carbon hidden_size=8
        for i in 0..5 {
            let g = make_genome(&mut rng);
            bank.try_insert(g, 10.0 + i as f32);
        }

        // Inserer des genomes Nitrogen hidden_size=10
        for i in 0..5 {
            let g = make_genome_nitrogen(&mut rng);
            bank.try_insert(g, 20.0 + i as f32);
        }

        assert_eq!(bank.len(), 10);
        assert!(
            bank.compartment_count() >= 2,
            "il devrait y avoir au moins 2 compartiments, trouve: {}",
            bank.compartment_count()
        );
    }

    #[test]
    fn les_graines_fraiches_sont_valides() {
        let mut rng = MockRng::new(0.5, 0.11);
        let genome = SeedBank::produce_fresh_seed(&mut rng);

        assert!(genome.brain.hidden_size() >= 6);
        assert!(genome.brain.hidden_size() <= 14);
        assert!(genome.traits.max_size() >= 15);
        assert!(genome.traits.max_size() <= 40);
        assert!(genome.traits.carbon_nitrogen_ratio() >= 0.3);
        assert!(genome.traits.carbon_nitrogen_ratio() <= 0.9);
    }

    #[test]
    fn produce_seed_retourne_un_genome() {
        let mut bank = SeedBank::new(10);
        let mut rng = MockRng::new(0.4, 0.13);
        bank.initialize(5, &mut rng);
        assert_eq!(bank.len(), 5);

        let seed = bank.produce_seed(&mut rng);
        // Le genome produit a un brain valide
        assert!(seed.brain.hidden_size() >= 6);
        assert!(seed.brain.hidden_size() <= 14);
    }

    #[test]
    fn le_compteur_de_generation_sincrement() {
        let mut counter = GenerationCounter::new();
        assert_eq!(counter.current(), 0);
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.next(), 2);
        assert_eq!(counter.current(), 2);
    }

    #[test]
    fn la_mutation_modifie_les_poids() {
        let mut rng = MockRng::new(0.1, 0.13);
        let genome_original = make_genome(&mut rng);
        let weights_before = genome_original.brain.weights();

        let mut genome_mutated = genome_original.clone();
        mutate_genome(&mut genome_mutated, &mut rng);

        let weights_after = genome_mutated.brain.weights();
        // Au moins un poids devrait avoir change (avec proba 0.3 par poids)
        let changed = weights_before
            .iter()
            .zip(weights_after.iter())
            .any(|(a, b)| (a - b).abs() > f32::EPSILON);
        assert!(changed, "la mutation devrait modifier au moins un poids");
    }

    #[test]
    fn entries_et_from_entries_roundtrip() {
        let mut bank = SeedBank::new(20);
        let mut rng = MockRng::new(0.2, 0.11);
        bank.initialize(8, &mut rng);

        let flat: Vec<(Genome, f32)> = bank
            .entries()
            .into_iter()
            .map(|(g, f)| (g.clone(), f))
            .collect();
        let count = flat.len();
        let bank2 = SeedBank::from_entries(flat, 20);

        assert_eq!(bank2.len(), count);
        assert_eq!(bank2.capacity(), 20);
    }

    #[test]
    fn diversity_spread_est_coherent() {
        let mut bank = SeedBank::new(10);
        let mut rng = MockRng::new(0.1, 0.13);

        // Banque vide → spread = 0
        assert_eq!(bank.diversity_spread(), 0.0);

        let g1 = make_genome(&mut rng);
        bank.try_insert(g1, 10.0);
        let g2 = make_genome(&mut rng);
        bank.try_insert(g2, 20.0);

        // spread = (20 - 10) / 20 = 0.5
        assert!((bank.diversity_spread() - 0.5).abs() < 0.01);
    }
}
