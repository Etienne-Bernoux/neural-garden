# Perception par Gradients Chimiques

## Principe

Les plantes réelles ne « voient » pas leurs voisines. Elles perçoivent des **gradients chimiques** dans le sol (nutriments, signaux allomonés) et des **gradients lumineux** (ombre portée). Neural Garden reproduit ce modèle.

Pour chaque gradient, on échantillonne les cellules dans un **rayon de 10 cellules** autour du centre de la plante. On somme les valeurs dans chaque demi-plan (nord vs sud, est vs ouest) et la différence normalisée donne un vecteur 2D indiquant la direction du gradient.

## Pourquoi pas des slots individuels ?

- **Réalisme** : les plantes n'ont pas d'yeux. Elles sentent des concentrations chimiques.
- **Compacité** : 6 inputs de gradient encodent plus d'info que 3×5 = 15 inputs de slots individuels.
- **Stratégies émergentes** : le réseau doit apprendre à interpréter des gradients, pas des coordonnées. C'est plus riche.
- **Le gradient de biomasse** encode à la fois la présence de voisins ET leur direction. Un gradient fort = grosse compétition ou opportunité d'invasion.

## Calcul du gradient (Rust)

Pour un champ `F` (nutriments, humidité, biomasse) et une plante centrée en `(cx, cy)` :

```
grad_x = Σ F(x,y) * sign(x - cx) / count
grad_y = Σ F(x,y) * sign(y - cy) / count
```

pour toutes les cellules `(x,y)` dans le rayon 10. Résultat normalisé dans [-1, 1].
