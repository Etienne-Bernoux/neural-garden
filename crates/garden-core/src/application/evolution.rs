// Service d'evolution : genome, fitness, banque de graines, crossover et mutation.

use crate::domain::brain::Brain;
use crate::domain::plant::{ExudateType, GeneticTraits};
use crate::domain::rng::Rng;

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
}

/// Calcule la fitness d'une plante a sa mort.
pub fn evaluate_fitness(stats: &PlantStats) -> f32 {
    let fitness = stats.max_biomass as f32 * 2.0
        + stats.lifetime as f32 * 1.0
        + stats.max_territory as f32 * 1.5
        + stats.symbiotic_connections as f32 * 4.0
        + stats.exudates_emitted * 2.0
        + stats.cn_exchanges * 1.5
        + stats.seeds_produced as f32 * 3.0
        + stats.soil_enriched * 2.0
        - stats.soil_depleted * 2.0
        - stats.monoculture_penalty * 1.5;
    fitness.max(0.0)
}

/// Banque de graines contenant les meilleurs genomes.
pub struct SeedBank {
    entries: Vec<(Genome, f32)>,
    capacity: usize,
}

impl SeedBank {
    /// Cree une banque vide avec la capacite donnee.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    /// Remplit la banque avec `count` genomes aleatoires, fitness initiale 0.
    pub fn initialize(&mut self, count: usize, rng: &mut dyn Rng) {
        for _ in 0..count {
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
            let traits = GeneticTraits::new(
                max_size,
                carbon_nitrogen_ratio,
                exudate_type,
                hidden_size,
                vitality_factor,
                energy_factor,
            );

            self.entries.push((Genome { brain, traits }, 0.0));
        }
    }

    /// Tente d'inserer un genome avec sa fitness.
    /// Retourne true si l'insertion a eu lieu.
    pub fn try_insert(&mut self, genome: Genome, fitness: f32) -> bool {
        if self.entries.len() < self.capacity {
            self.entries.push((genome, fitness));
            self.entries
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
            return true;
        }
        // La banque est pleine, verifier si meilleur que le pire
        if let Some(last) = self.entries.last() {
            if fitness > last.1 {
                self.entries.pop();
                self.entries.push((genome, fitness));
                self.entries
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
                return true;
            }
        }
        false
    }

    /// Produit une graine en combinant deux parents de la banque.
    pub fn produce_seed(&self, rng: &mut dyn Rng) -> Genome {
        let idx_a = (rng.next_f32() * self.entries.len() as f32) as usize;
        let mut idx_b = (rng.next_f32() * self.entries.len() as f32) as usize;
        // Eviter de tirer le meme parent
        if idx_b == idx_a && self.entries.len() > 1 {
            idx_b = (idx_b + 1) % self.entries.len();
        }
        let idx_a = idx_a.min(self.entries.len() - 1);
        let idx_b = idx_b.min(self.entries.len() - 1);

        let (parent_a, fitness_a) = &self.entries[idx_a];
        let (parent_b, fitness_b) = &self.entries[idx_b];

        let mut child = if parent_a.traits.hidden_size() == parent_b.traits.hidden_size() {
            // Crossover possible
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

    /// Fitness du meilleur genome.
    pub fn best_fitness(&self) -> f32 {
        self.entries.first().map_or(0.0, |e| e.1)
    }

    /// Fitness du pire genome.
    pub fn worst_fitness(&self) -> f32 {
        self.entries.last().map_or(0.0, |e| e.1)
    }

    /// Nombre d'entrees dans la banque.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Indique si la banque est vide.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
        };
        let fitness = evaluate_fitness(&stats);
        assert!(fitness > 0.0);
        // Verifier que chaque composante positive contribue
        // fitness = 20 + 100 + 7.5 + 12 + 4 + 1.5 + 12 + 6 - 2 - 0.75 = 160.25
        assert!((fitness - 160.25).abs() < 0.01);
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
        let mut bank = SeedBank::new(2);
        let mut rng = MockRng::new(0.2, 0.13);
        let g1 = make_genome(&mut rng);
        let g2 = make_genome(&mut rng);
        let g3 = make_genome(&mut rng);

        bank.try_insert(g1, 10.0);
        bank.try_insert(g2, 20.0);
        assert_eq!(bank.len(), 2);

        // Inserer un meilleur que le pire → remplace
        let inserted = bank.try_insert(g3, 15.0);
        assert!(inserted);
        assert_eq!(bank.len(), 2);
        assert_eq!(bank.worst_fitness(), 15.0);
    }

    #[test]
    fn la_banque_refuse_les_mauvais() {
        let mut bank = SeedBank::new(2);
        let mut rng = MockRng::new(0.3, 0.13);
        let g1 = make_genome(&mut rng);
        let g2 = make_genome(&mut rng);
        let g3 = make_genome(&mut rng);

        bank.try_insert(g1, 10.0);
        bank.try_insert(g2, 20.0);

        // Inserer un pire que le pire → refuse
        let inserted = bank.try_insert(g3, 5.0);
        assert!(!inserted);
        assert_eq!(bank.len(), 2);
        assert_eq!(bank.worst_fitness(), 10.0);
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
}
