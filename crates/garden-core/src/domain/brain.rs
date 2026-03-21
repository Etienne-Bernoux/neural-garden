pub const INPUT_SIZE: usize = 18;
pub const OUTPUT_SIZE: usize = 8;
pub const MIN_HIDDEN_SIZE: u8 = 6;
pub const MAX_HIDDEN_SIZE: u8 = 14;

/// Petit reseau de neurones feedforward (18 → cache → cache → 8).
/// Pilote le comportement des plantes : le "cerveau" de chaque plante.
#[derive(Debug, Clone)]
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

    /// Retourne tous les poids et biais en un seul Vec plat.
    /// Ordre : weights_ih, weights_hh, weights_ho, biases_h1, biases_h2, biases_o.
    pub fn weights(&self) -> Vec<f32> {
        let mut w = Vec::with_capacity(self.total_weights());
        w.extend_from_slice(&self.weights_ih);
        w.extend_from_slice(&self.weights_hh);
        w.extend_from_slice(&self.weights_ho);
        w.extend_from_slice(&self.biases_h1);
        w.extend_from_slice(&self.biases_h2);
        w.extend_from_slice(&self.biases_o);
        w
    }

    /// Reconstruit un Brain depuis un Vec plat de poids.
    /// Ordre attendu : weights_ih, weights_hh, weights_ho, biases_h1, biases_h2, biases_o.
    /// Retourne None si la taille du vecteur ne correspond pas a la taille attendue.
    pub fn from_weights(hidden_size: u8, weights: Vec<f32>) -> Option<Self> {
        let hs = hidden_size.clamp(MIN_HIDDEN_SIZE, MAX_HIDDEN_SIZE);
        let h = hs as usize;
        let expected = INPUT_SIZE * h + h * h + h * OUTPUT_SIZE + h + h + OUTPUT_SIZE;
        if weights.len() != expected {
            return None;
        }

        let mut offset = 0;

        let weights_ih = weights[offset..offset + INPUT_SIZE * h].to_vec();
        offset += INPUT_SIZE * h;

        let weights_hh = weights[offset..offset + h * h].to_vec();
        offset += h * h;

        let weights_ho = weights[offset..offset + h * OUTPUT_SIZE].to_vec();
        offset += h * OUTPUT_SIZE;

        let biases_h1 = weights[offset..offset + h].to_vec();
        offset += h;

        let biases_h2 = weights[offset..offset + h].to_vec();
        offset += h;

        let biases_o = weights[offset..offset + OUTPUT_SIZE].to_vec();

        Some(Self {
            hidden_size: hs,
            weights_ih,
            weights_hh,
            weights_ho,
            biases_h1,
            biases_h2,
            biases_o,
        })
    }

    /// Redimensionne le cerveau a une nouvelle taille cachee.
    /// Ajoute des poids a 0 si la taille augmente, tronque si elle diminue.
    pub fn resize(self, new_hidden_size: u8) -> Brain {
        let new_hs = new_hidden_size.clamp(MIN_HIDDEN_SIZE, MAX_HIDDEN_SIZE);
        if new_hs == self.hidden_size {
            return self;
        }
        let old_h = self.hidden_size as usize;
        let new_h = new_hs as usize;

        // Redimensionne une matrice (rows x old_cols) → (rows x new_cols)
        let resize_matrix =
            |src: &[f32], rows: usize, old_cols: usize, new_cols: usize| -> Vec<f32> {
                let mut dst = vec![0.0; rows * new_cols];
                let cols = old_cols.min(new_cols);
                for r in 0..rows {
                    for c in 0..cols {
                        dst[r * new_cols + c] = src[r * old_cols + c];
                    }
                }
                dst
            };

        // weights_ih : INPUT_SIZE x hidden
        let weights_ih = resize_matrix(&self.weights_ih, INPUT_SIZE, old_h, new_h);
        // weights_hh : hidden x hidden
        let weights_hh = resize_matrix(&self.weights_hh, old_h.min(new_h), old_h, new_h);
        let weights_hh = if new_h > old_h {
            let mut full = vec![0.0; new_h * new_h];
            full[..weights_hh.len()].copy_from_slice(&weights_hh);
            full
        } else {
            weights_hh
        };
        // weights_ho : hidden x OUTPUT_SIZE
        let weights_ho =
            resize_matrix(&self.weights_ho, old_h.min(new_h), OUTPUT_SIZE, OUTPUT_SIZE);
        let weights_ho = if new_h > old_h {
            let mut full = vec![0.0; new_h * OUTPUT_SIZE];
            for (i, v) in weights_ho.iter().enumerate() {
                full[i] = *v;
            }
            full
        } else {
            let mut truncated = vec![0.0; new_h * OUTPUT_SIZE];
            truncated.copy_from_slice(&weights_ho[..new_h * OUTPUT_SIZE]);
            truncated
        };

        // Biais : redimensionner en tronquant ou ajoutant des 0
        let resize_bias = |src: &[f32], new_len: usize| -> Vec<f32> {
            let mut dst = vec![0.0; new_len];
            let copy_len = src.len().min(new_len);
            dst[..copy_len].copy_from_slice(&src[..copy_len]);
            dst
        };

        let biases_h1 = resize_bias(&self.biases_h1, new_h);
        let biases_h2 = resize_bias(&self.biases_h2, new_h);

        Self {
            hidden_size: new_hs,
            weights_ih,
            weights_hh,
            weights_ho,
            biases_h1,
            biases_h2,
            biases_o: self.biases_o,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::rng::test_utils::MockRng;

    #[test]
    fn creation_cerveau_avec_taille_cachee_min() {
        let mut rng = MockRng::new(0.0, 0.1);
        let brain = Brain::new(MIN_HIDDEN_SIZE, &mut rng);
        let hs = MIN_HIDDEN_SIZE as usize;
        let expected = INPUT_SIZE * hs + hs * hs + hs * OUTPUT_SIZE + hs + hs + OUTPUT_SIZE;
        assert_eq!(brain.total_weights(), expected);
        assert_eq!(brain.hidden_size(), MIN_HIDDEN_SIZE);
    }

    #[test]
    fn creation_cerveau_avec_taille_cachee_max() {
        let mut rng = MockRng::new(0.0, 0.1);
        let brain = Brain::new(MAX_HIDDEN_SIZE, &mut rng);
        let hs = MAX_HIDDEN_SIZE as usize;
        let expected = INPUT_SIZE * hs + hs * hs + hs * OUTPUT_SIZE + hs + hs + OUTPUT_SIZE;
        assert_eq!(brain.total_weights(), expected);
        assert_eq!(brain.hidden_size(), MAX_HIDDEN_SIZE);
    }

    #[test]
    fn le_cerveau_borne_la_taille_cachee() {
        let mut rng = MockRng::new(0.5, 0.1);
        let brain_low = Brain::new(3, &mut rng);
        assert_eq!(brain_low.hidden_size(), MIN_HIDDEN_SIZE);

        let mut rng = MockRng::new(0.5, 0.1);
        let brain_high = Brain::new(20, &mut rng);
        assert_eq!(brain_high.hidden_size(), MAX_HIDDEN_SIZE);
    }

    #[test]
    fn forward_pass_retourne_des_sorties_valides() {
        let mut rng = MockRng::new(0.3, 0.1);
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
        let mut rng = MockRng::new(0.0, 0.1);
        let brain = Brain::new(10, &mut rng);
        let inputs = [0.2_f32; INPUT_SIZE];
        let out1 = brain.forward(&inputs);
        let out2 = brain.forward(&inputs);
        assert_eq!(out1, out2);
    }

    #[test]
    fn tailles_cachees_differentes_donnent_nombres_poids_differents() {
        let mut rng = MockRng::new(0.0, 0.1);
        let brain_small = Brain::new(6, &mut rng);
        let mut rng = MockRng::new(0.0, 0.1);
        let brain_large = Brain::new(14, &mut rng);
        assert_ne!(brain_small.total_weights(), brain_large.total_weights());
    }

    #[test]
    fn le_cerveau_resize_augmente_la_taille() {
        // Creer un cerveau de taille 8, le redimensionner a 10
        let mut rng = MockRng::new(0.0, 0.1);
        let brain = Brain::new(8, &mut rng);
        let brain = brain.resize(10);
        assert_eq!(brain.hidden_size(), 10);
        let h = 10_usize;
        let expected = INPUT_SIZE * h + h * h + h * OUTPUT_SIZE + h + h + OUTPUT_SIZE;
        assert_eq!(brain.total_weights(), expected);
    }

    #[test]
    fn le_cerveau_resize_diminue_la_taille() {
        // Creer un cerveau de taille 10, le redimensionner a 8
        let mut rng = MockRng::new(0.0, 0.1);
        let brain = Brain::new(10, &mut rng);
        let brain = brain.resize(8);
        assert_eq!(brain.hidden_size(), 8);
    }

    #[test]
    fn le_cerveau_roundtrip_weights() {
        // Verifier que brain → weights() → from_weights() → weights() donne le meme vecteur
        let mut rng = MockRng::new(0.5, 0.1);
        let brain = Brain::new(8, &mut rng);
        let w1 = brain.weights();
        let brain2 = Brain::from_weights(8, w1.clone());
        assert!(
            brain2.is_some(),
            "from_weights devrait reussir avec un vecteur valide"
        );
        let w2 = brain2.unwrap().weights();
        assert_eq!(w1, w2);
    }

    #[test]
    fn from_weights_refuse_vecteur_invalide() {
        // Un vecteur trop court doit retourner None
        let result = Brain::from_weights(8, vec![0.0; 10]);
        assert!(
            result.is_none(),
            "from_weights devrait refuser un vecteur de taille invalide"
        );

        // Un vecteur vide aussi
        let result = Brain::from_weights(8, vec![]);
        assert!(
            result.is_none(),
            "from_weights devrait refuser un vecteur vide"
        );
    }
}
