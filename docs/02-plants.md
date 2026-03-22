# Les Plantes : Anatomie et Cycle de Vie

> **Modèle 3 couches (Phase 6a-R1)** : chaque plante occupe 3 couches spatiales distinctes.
> - **Emprise au sol (footprint)** : position physique exclusive. Seule couche où l'invasion est possible.
> - **Canopée (canopy)** : couche aérienne partagée avec priorité par hauteur. Ombre dynamique — la plante avec la plus grande emprise projette de l'ombre sur les plus petites.
> - **Racines (roots)** : couche souterraine partagée librement. Absorption C/N/H₂O, symbiose mycorhizienne. Pas d'invasion.
>
> Ratios max : `canopy = footprint × 4`, `roots = footprint × 5`.

## Représentation

Chaque plante est un **ensemble de cellules contiguës** sur la grille. Deux plantes ne partagent jamais une cellule d'emprise au sol. La plante grandit en ajoutant des cellules adjacentes. Visuellement, chaque cellule est colorée selon l'espèce (teinte) et la santé (luminosité/saturation).

## Stats vitales

| Stat | Plage | Description |
|---|---|---|
| Vitalité | 0 – 100 | Santé globale. 0 = mort. Diminue si ressources insuffisantes. |
| Énergie | 0 – 100 | Carburant pour les actions (croissance, exsudats, défense). Produite par **photosynthèse** (lumière). |
| Biomasse (nb cellules) | 1 – maxSize | Taille physique. La croissance consomme du **carbone + azote** du sol. Chaque cellule coûte de l'énergie de maintenance. |
| maxSize | 15 – 40 | Taille max. **Trait génétique**, soumis à l'évolution. Espèces "herbes" (petit maxSize, rapides) vs "arbres" (grand maxSize, lents). |
| Âge | 0 – ∞ | Ticks vécus. Normalisé pour le réseau. |

### Énergie vs croissance — deux systèmes séparés

- **Lumière → énergie** : la photosynthèse convertit la lumière reçue en énergie. L'énergie est le carburant de toutes les actions (croissance, exsudats, défense, reproduction).
- **Sol (carbone + azote) → croissance** : pour ajouter une cellule, la plante doit consommer du carbone et de l'azote du sol, en plus de dépenser de l'énergie. Le **ratio carbone/azote est un trait génétique** variable par espèce, soumis à l'évolution.

Une plante bien éclairée mais sur un sol pauvre a de l'énergie mais ne peut pas pousser. Une plante sur un sol riche mais à l'ombre a la matière première mais pas le carburant.

## Traits génétiques

Paramètres hérités du parent, mutés à la phase graine, soumis à la sélection naturelle :

| Trait | Plage | Mutation | Description |
|---|---|---|---|
| maxSize | 15 – 40 | gaussien | Taille max de la plante |
| carbon_nitrogen_ratio | 0.3 – 0.9 | gaussien | Part de carbone vs azote nécessaire à la croissance (0.9 = gros consommateur de carbone, peu d'azote) |
| exudate_type | carbone \| azote | flip rare | Ce que la plante exsude. Les fixatrices d'azote émergent par évolution. |
| hidden_size | 6 – 14 | ±1 | Neurones par couche cachée. Contraint le crossover (même taille requise → spéciation). |
| Poids du réseau | 252 – 724 floats | gaussien | Le cerveau qui pilote le comportement. Taille dépend de `hidden_size`. |

## Cycle de vie

1. **Graine (dormance)** : 1 cellule. Phase d'attente — la graine ne meurt pas immédiatement. Elle peut **survivre plusieurs cycles** en attendant des conditions favorables (seuils de carbone + azote dépendants de l'espèce). C'est à cette phase que les **mutations génétiques sont calculées** (poids du réseau, traits). Pour les graines issues de la **banque de graines**, un crossover entre deux génomes compatibles (même `hidden_size`) peut précéder la mutation (voir 06-evolution.md). Si les conditions ne sont jamais réunies, la graine finit par mourir (durée max de dormance).
2. **Germination** : quand le sol est suffisamment riche (carbone + azote au-dessus des seuils de l'espèce), la graine germe et commence à absorber.
3. **Croissance** : la plante ajoute des cellules adjacentes en investissant de l'énergie + carbone + azote du sol. La vitesse de croissance dépend de `grow_intensity` (output du cerveau, 0 = maintenance pure, 1 = croissance max). La direction est guidée par `grow_dir_x/y`. L'output `canopy_vs_roots` détermine l'allocation : canopée (> 0.5) → emprise au sol + photosynthèse accrue ; racines (≤ 0.5) → zone d'influence + absorption élargie.
4. **Maturité** : biomasse proche de maxSize. Fleurs visibles. Peut se reproduire (lancer une graine à distance).
5. **Stress** : sol épuisé, ombre, invasion. Couleur qui se ternit. Vitalité qui baisse.
6. **Dépérissement** : vitalité < 20%. La plante perd des cellules périphériques (rétraction).
7. **Mort et décomposition** : vitalité = 0. Toutes les cellules sont libérées. Le sol est enrichi :
   - **Carbone** libéré proportionnellement à la **biomasse** (nb de cellules à la mort).
   - **Azote** libéré proportionnellement à l'**âge/maturité** de la plante.

## Reproduction

Quand une plante a assez d'énergie (>60) et de biomasse (>8 cellules), elle peut **lancer une graine** à 3-9 cellules de distance dans une direction aléatoire. Si la cellule cible est libre (y compris : pas submergée), un nouvel individu naît. Coût : 30 énergie.

La graine hérite du génome du parent. Les **mutations sont appliquées pendant la phase de dormance**, pas à la naissance. Les mutations inter-générationnelles classiques (gaussiennes sur les poids, perturbation des traits) s'appliquent ici.
