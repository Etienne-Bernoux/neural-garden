# Réseau de Neurones : 13→10→10→8

## Couche d'entrée — 13 neurones

**État interne (4 inputs) :**

| Input | Plage | Description |
|---|---|---|
| vitality | [0, 1] | PV / 100 |
| energy | [0, 1] | Énergie / 100 |
| biomass_ratio | [0, 1] | nb_cellules / maxSize (marge de croissance restante) |
| age_ratio | [0, 1] | ticks vécus / durée max estimée |

**Sol local (3 inputs) :**

| Input | Plage | Description |
|---|---|---|
| nutrients_here | [0, 1] | Moyenne des nutriments sous les cellules de la plante |
| humidity_here | [0, 1] | Moyenne de l'humidité sous les cellules |
| light_here | [0, 1] | Lumière reçue (réduite si ombragée) |

**Gradients environnementaux (6 inputs) :**

| Input | Plage | Description |
|---|---|---|
| nutrient_grad_x | [-1, 1] | Direction X vers le sol le plus riche en nutriments |
| nutrient_grad_y | [-1, 1] | Direction Y |
| humidity_grad_x | [-1, 1] | Direction X vers la zone la plus humide |
| humidity_grad_y | [-1, 1] | Direction Y |
| biomass_grad_x | [-1, 1] | Direction X vers la plus forte densité de végétation |
| biomass_grad_y | [-1, 1] | Direction Y (négatif = zone vide, positif = zone dense) |

## Couches cachées

**2 couches de 10 neurones**. Activation : `tanh`. Topologie fixe (v1), seuls les poids évoluent.

Taille mémoire : 13×10 + 10×10 + 10×8 = **310 poids + 28 biais = 338 paramètres ≈ 1.4 Ko** par cerveau. Population de 200 = 280 Ko.

## Couche de sortie — 8 neurones

| Output | Activation | Description |
|---|---|---|
| grow_dir_x | tanh → [-1, 1] | Direction X privilégiée pour l'extension des racines/cellules |
| grow_dir_y | tanh → [-1, 1] | Direction Y privilégiée |
| grow_intensity | sigmoid → [0, 1] | Part d'énergie allouée à la croissance (0 = maintenance pure, 1 = croissance max) |
| canopy_vs_roots | sigmoid → [0, 1] | Investir en canopée (> 0.5, lumière) ou en racines (≤ 0.5, absorption) |
| exudate_rate | sigmoid → [0, 1] | Quantité d'exsudats injectés dans le sol (coopération publique). Coûte de l'énergie. |
| connect_signal | sigmoid → [0, 1] | Accepter une connexion mycorhizienne directe (> 0.5 = oui) |
| connect_generosity | sigmoid → [0, 1] | Part de nutriments donnés via le lien direct (0 = parasitisme, 1 = générosité) |
| defense | sigmoid → [0, 1] | Durcir les racines (> 0.5). Réduit l'invasion de 50% mais coûte 3 énergie/tick |

## Propagation et implémentation

Le forward pass est un simple produit matrice-vecteur par couche, suivi de l'activation. Implémenté en Rust pur :

```rust
struct Brain {
    weights: Vec<Vec<Vec<f32>>>,
    biases: Vec<Vec<f32>>,
}

fn forward(&self, inputs: &[f32; 13]) -> [f32; 8]
```
