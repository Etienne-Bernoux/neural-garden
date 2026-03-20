# Les Plantes : Anatomie et Cycle de Vie

## Représentation

Chaque plante est un **ensemble de cellules contiguës** sur la grille. Deux plantes ne partagent jamais une cellule. La plante grandit en ajoutant des cellules adjacentes. Visuellement, chaque cellule est colorée selon l'espèce (teinte) et la santé (luminosité/saturation).

## Stats vitales

| Stat | Plage | Description |
|---|---|---|
| Vitalité | 0 – 100 | Santé globale. 0 = mort. Diminue si ressources insuffisantes. |
| Énergie | 0 – 100 | Carburant pour les actions (croissance, exsudats, défense). Produite par photosynthèse + absorption. |
| Biomasse (nb cellules) | 1 – maxSize | Taille physique. Chaque cellule coûte de l'énergie de maintenance. |
| maxSize | 15 – 40 | Taille max (aléatoire par individu). Plafonne la croissance. |
| Âge | 0 – ∞ | Ticks vécus. Normalisé pour le réseau. |

## Cycle de vie

1. **Graine** : 1 cellule. Absorbe les nutriments locaux. Si le sol est trop pauvre, elle meurt.
2. **Croissance** : la plante ajoute des cellules adjacentes en investissant de l'énergie.
3. **Maturité** : biomasse proche de maxSize. Fleurs visibles. Peut se reproduire (lancer une graine à distance).
4. **Stress** : sol épuisé, ombre, invasion. Couleur qui se ternit. Vitalité qui baisse.
5. **Dépérissement** : vitalité < 20%. La plante perd des cellules périphériques (rétraction).
6. **Mort et décomposition** : vitalité = 0. Toutes les cellules sont libérées. Sol enrichi (+20 nutriments).

## Reproduction

Quand une plante a assez d'énergie (>60) et de biomasse (>8 cellules), elle peut **lancer une graine** à 3-9 cellules de distance dans une direction aléatoire. Si la cellule cible est libre, un nouvel individu naît avec le même cerveau (hérité du parent, pas muté — les mutations sont inter-générationnelles). Coût : 30 énergie.
