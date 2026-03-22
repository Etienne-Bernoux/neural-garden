# Perception par Gradients Chimiques

> **Note d'implémentation** : la zone d'influence est définie par les cellules racines physiques de la plante (pas un rayon calculé). Les gradients sont calculés sur les racines, le barycentre est celui des racines. Une graine (1 racine) a des gradients nuls.

## Principe

Les plantes réelles ne « voient » pas leurs voisines. Elles perçoivent des **gradients chimiques** dans le sol (carbone, azote, signaux allomonés) et des **gradients lumineux** (ombre portée). Neural Garden reproduit ce modèle.

Chaque plante possède trois couches spatiales (voir 02-plants.md) :

- **Emprise au sol (footprint)** : cellules exclusives. Le sol local (4 inputs) est calculé en moyenne sur cette zone.
- **Canopée (canopy)** : couche aérienne partagée. Photosynthèse.
- **Racines (roots)** : couche souterraine partagée. Les **gradients sont calculés sur les racines** et le barycentre est celui des racines. C'est aussi la zone d'absorption et d'interaction (exsudats, symbiose).

Une graine (1 cellule) n'a qu'une seule racine — les gradients sont nuls (pas de différentiel possible). Un arbre mature a un large réseau racinaire (jusqu'à 5× l'emprise), perçoit plus loin, absorbe sur plus de cellules.

## Pourquoi des gradients ?

- **Réalisme** : les plantes n'ont pas d'yeux. Elles sentent des concentrations chimiques.
- **Compacité** : des inputs de gradient encodent plus d'info que des slots individuels par cellule.
- **Stratégies émergentes** : le réseau doit apprendre à interpréter des gradients, pas des coordonnées. C'est plus riche.
- **Le gradient de biomasse** encode à la fois la présence de voisins ET leur direction. Un gradient fort = grosse compétition ou opportunité d'invasion.

## Gradients calculés

Les gradients sont calculés sur les **cellules racines** de la plante, centrés sur le barycentre des racines.

| Gradient | Inputs | Description |
|---|---|---|
| Carbone | grad_x, grad_y | Direction vers le sol le plus riche en carbone |
| Azote | grad_x, grad_y | Direction vers le sol le plus riche en azote |
| Humidité | grad_x, grad_y | Direction vers les zones les plus humides (vallées, mares) |
| Biomasse | grad_x, grad_y | Direction vers la plus forte densité de végétation. Négatif = zone vide. |
| Lumière | grad_x, grad_y | Direction vers les zones les plus éclairées (sortir de l'ombre) |

Total : **10 inputs de gradient** (5 champs × 2 composantes x/y). Combinés avec les 4 inputs d'état interne et 4 inputs de sol local (voir 04-brain.md), le réseau reçoit **18 inputs** au total.

## Calcul du gradient

Pour un champ `F` (carbone, azote, humidité, biomasse, lumière) et une plante dont le barycentre est en `(cx, cy)` :

```
grad_x = Σ F(x,y) * sign(x - cx) / count
grad_y = Σ F(x,y) * sign(y - cy) / count
```

pour toutes les cellules `(x,y)` racines de la plante. Résultat normalisé dans [-1, 1].

## Étendue des racines

La zone de perception n'est pas un rayon calculé — ce sont les **cellules racines physiques** de la plante. Plus la plante investit en racines (`canopy_vs_roots` < 0.33), plus elle ajoute de cellules racinaires (jusqu'à `footprint × 5`), élargissant sa zone de perception et d'absorption.

| Biomasse | Racines max | Perception |
|---|---|---|
| 1 (graine) | 5 cellules | Gradients nuls (1 seule racine au départ) |
| 5-10 | 25-50 cellules | Commence à percevoir les gradients |
| 15-25 | 75-125 cellules | Perception étendue |
| 30-40 | 150-200 cellules | Large réseau racinaire, perception maximale |

La croissance racinaire est gratuite en énergie et suit un mécanisme de chimiotaxie vers les plantes voisines (facilite la formation de liens symbiotiques).
