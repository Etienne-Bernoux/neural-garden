pub const INPUT_SIZE: usize = 18;
pub const OUTPUT_SIZE: usize = 8;
pub const MIN_HIDDEN_SIZE: u8 = 6;
pub const MAX_HIDDEN_SIZE: u8 = 14;

/// Petit reseau de neurones feedforward (18 → cache → cache → 8).
/// Pilote le comportement des plantes : le "cerveau" de chaque plante.
pub struct Brain {
    hidden_size: u8,
    weights_ih: Vec<f32>,
    weights_hh: Vec<f32>,
    weights_ho: Vec<f32>,
    biases_h1: Vec<f32>,
    biases_h2: Vec<f32>,
    biases_o: Vec<f32>,
}

fn tanh(x: f32) -> f32 {
    f32::tanh(x)
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

impl Brain {
    /// Cree un nouveau cerveau avec des poids et biais aleatoires.
    /// `hidden_size` est borne a [`MIN_HIDDEN_SIZE`, `MAX_HIDDEN_SIZE`].
    pub fn new(hidden_size: u8, rng: &mut dyn super::rng::Rng) -> Self {
        let hs = hidden_size.clamp(MIN_HIDDEN_SIZE, MAX_HIDDEN_SIZE);
        let hs_usize = hs as usize;

        let rand_weight = |rng: &mut dyn super::rng::Rng| rng.next_f32() * 2.0 - 1.0;

        let weights_ih: Vec<f32> = (0..INPUT_SIZE * hs_usize)
            .map(|_| rand_weight(rng))
            .collect();
        let weights_hh: Vec<f32> = (0..hs_usize * hs_usize).map(|_| rand_weight(rng)).collect();
        let weights_ho: Vec<f32> = (0..hs_usize * OUTPUT_SIZE)
            .map(|_| rand_weight(rng))
            .collect();
        let biases_h1: Vec<f32> = (0..hs_usize).map(|_| rand_weight(rng)).collect();
        let biases_h2: Vec<f32> = (0..hs_usize).map(|_| rand_weight(rng)).collect();
        let biases_o: Vec<f32> = (0..OUTPUT_SIZE).map(|_| rand_weight(rng)).collect();

        Self {
            hidden_size: hs,
            weights_ih,
            weights_hh,
            weights_ho,
            biases_h1,
            biases_h2,
            biases_o,
        }
    }

    /// Execute une passe avant a travers le reseau.
    pub fn forward(&self, inputs: &[f32; INPUT_SIZE]) -> [f32; OUTPUT_SIZE] {
        let hs = self.hidden_size as usize;

        // Couche 1 : entree → cache1
        let h1: Vec<f32> = (0..hs)
            .map(|j| {
                let sum = inputs
                    .iter()
                    .enumerate()
                    .fold(self.biases_h1[j], |acc, (i, &inp)| {
                        acc + inp * self.weights_ih[i * hs + j]
                    });
                tanh(sum)
            })
            .collect();

        // Couche 2 : cache1 → cache2
        let h2: Vec<f32> = (0..hs)
            .map(|j| {
                let sum = h1
                    .iter()
                    .enumerate()
                    .fold(self.biases_h2[j], |acc, (i, &val)| {
                        acc + val * self.weights_hh[i * hs + j]
                    });
                tanh(sum)
            })
            .collect();

        // Sortie : cache2 → sortie
        let mut output = [0.0_f32; OUTPUT_SIZE];
        for (j, out) in output.iter_mut().enumerate() {
            let sum = h2
                .iter()
                .enumerate()
                .fold(self.biases_o[j], |acc, (i, &val)| {
                    acc + val * self.weights_ho[i * OUTPUT_SIZE + j]
                });
            *out = sigmoid(sum);
        }

        output
    }

    pub fn hidden_size(&self) -> u8 {
        self.hidden_size
    }

    /// Retourne le nombre total de poids et biais dans le reseau.
    pub fn total_weights(&self) -> usize {
        self.weights_ih.len()
            + self.weights_hh.len()
            + self.weights_ho.len()
            + self.biases_h1.len()
            + self.biases_h2.len()
            + self.biases_o.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRng(f32);
    impl super::super::rng::Rng for MockRng {
        fn next_f32(&mut self) -> f32 {
            let v = self.0;
            self.0 = (self.0 + 0.1) % 1.0;
            v
        }
    }

    #[test]
    fn creation_cerveau_avec_taille_cachee_min() {
        let mut rng = MockRng(0.0);
        let brain = Brain::new(MIN_HIDDEN_SIZE, &mut rng);
        let hs = MIN_HIDDEN_SIZE as usize;
        let expected = INPUT_SIZE * hs + hs * hs + hs * OUTPUT_SIZE + hs + hs + OUTPUT_SIZE;
        assert_eq!(brain.total_weights(), expected);
        assert_eq!(brain.hidden_size(), MIN_HIDDEN_SIZE);
    }

    #[test]
    fn creation_cerveau_avec_taille_cachee_max() {
        let mut rng = MockRng(0.0);
        let brain = Brain::new(MAX_HIDDEN_SIZE, &mut rng);
        let hs = MAX_HIDDEN_SIZE as usize;
        let expected = INPUT_SIZE * hs + hs * hs + hs * OUTPUT_SIZE + hs + hs + OUTPUT_SIZE;
        assert_eq!(brain.total_weights(), expected);
        assert_eq!(brain.hidden_size(), MAX_HIDDEN_SIZE);
    }

    #[test]
    fn le_cerveau_borne_la_taille_cachee() {
        let mut rng = MockRng(0.5);
        let brain_low = Brain::new(3, &mut rng);
        assert_eq!(brain_low.hidden_size(), MIN_HIDDEN_SIZE);

        let mut rng = MockRng(0.5);
        let brain_high = Brain::new(20, &mut rng);
        assert_eq!(brain_high.hidden_size(), MAX_HIDDEN_SIZE);
    }

    #[test]
    fn forward_pass_retourne_des_sorties_valides() {
        let mut rng = MockRng(0.3);
        let brain = Brain::new(8, &mut rng);
        let inputs = [0.5_f32; INPUT_SIZE];
        let outputs = brain.forward(&inputs);
        for (j, &o) in outputs.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&o),
                "output[{j}] = {o} is not in [0.0, 1.0]"
            );
        }
    }

    #[test]
    fn forward_pass_est_deterministe() {
        let mut rng = MockRng(0.0);
        let brain = Brain::new(10, &mut rng);
        let inputs = [0.2_f32; INPUT_SIZE];
        let out1 = brain.forward(&inputs);
        let out2 = brain.forward(&inputs);
        assert_eq!(out1, out2);
    }

    #[test]
    fn tailles_cachees_differentes_donnent_nombres_poids_differents() {
        let mut rng = MockRng(0.0);
        let brain_small = Brain::new(6, &mut rng);
        let mut rng = MockRng(0.0);
        let brain_large = Brain::new(14, &mut rng);
        assert_ne!(brain_small.total_weights(), brain_large.total_weights());
    }
}
