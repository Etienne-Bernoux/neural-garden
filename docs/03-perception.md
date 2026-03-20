# Perception par Gradients Chimiques

## Principe

Les plantes réelles ne « voient » pas leurs voisines. Elles perçoivent des **gradients chimiques** dans le sol (carbone, azote, signaux allomonés) et des **gradients lumineux** (ombre portée). Neural Garden reproduit ce modèle.

Chaque plante possède deux zones concentriques :

- **Zone de place** : les cellules physiquement occupées par la plante. Aucune autre plante ne peut s'y trouver. C'est la biomasse.
- **Zone d'influence** : zone élargie autour de la plante. Le rayon scale avec la **biomasse** et l'investissement en **racines** (output `canopy_vs_roots` ≤ 0.5 → rayon plus grand). C'est la zone de récupération des ressources et d'interaction avec les voisins. Les gradients sont calculés sur cette zone.

Une graine (1 cellule) a une zone d'influence minimale — ce qui importe c'est l'endroit où elle se trouve. Un arbre mature a une large zone d'influence, perçoit plus loin, absorbe sur un rayon plus grand.

## Pourquoi des gradients ?

- **Réalisme** : les plantes n'ont pas d'yeux. Elles sentent des concentrations chimiques.
- **Compacité** : des inputs de gradient encodent plus d'info que des slots individuels par cellule.
- **Stratégies émergentes** : le réseau doit apprendre à interpréter des gradients, pas des coordonnées. C'est plus riche.
- **Le gradient de biomasse** encode à la fois la présence de voisins ET leur direction. Un gradient fort = grosse compétition ou opportunité d'invasion.

## Gradients calculés

Les gradients sont calculés sur la **zone d'influence**, centrée sur le barycentre de la zone de place.

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

pour toutes les cellules `(x,y)` dans la zone d'influence. Résultat normalisé dans [-1, 1].

## Zone d'influence — scaling

| Biomasse | Rayon d'influence | Justification |
|---|---|---|
| 1 (graine) | 1-2 cellules | Perception minimale, ce qui compte c'est le sol local |
| 5-10 | ~6 cellules | Jeune plante, commence à sentir les voisins |
| 15-25 | ~10 cellules | Plante établie, bonne perception |
| 30-40 | ~14 cellules | Arbre mature, large réseau racinaire |

Formule indicative : `rayon = base + sqrt(biomasse) * facteur * roots_bonus`. Où `roots_bonus` est modulé par l'output `canopy_vs_roots` (investir en racines agrandit la zone d'influence). À calibrer.
