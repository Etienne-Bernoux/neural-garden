# Moteur de Neuroévolution

## Modèle : simulation continue

Pas de vagues ni de cycles artificiels. La simulation tourne en continu. L'évolution se fait "in vivo" : les plantes naissent, vivent, se reproduisent et meurent. La sélection naturelle est directe — tu survis ou pas.

## Banque de graines

La banque de graines est le réservoir génétique de la simulation. Elle contient les **50 meilleurs génomes** observés au cours de la simulation.

### Initialisation
- Au démarrage, la banque est remplie avec **50 génomes aléatoires** avec un biais de survie minimal : les biais de `grow_intensity` sont positifs (la plante pousse par défaut), ce qui garantit un comportement minimal viable.
- L'île est peuplée depuis cette banque initiale.

### Alimentation
- **À la mort de chaque plante**, sa fitness est évaluée (voir ci-dessous).
- Si sa fitness est **supérieure au pire génome de la banque**, elle le remplace.
- La banque maintient toujours 50 slots, triés par fitness.

### Injection — pluie de graines
- La banque injecte des graines **en continu** sur l'île, à un taux configurable (pluie de graines).
- Chaque graine injectée est produite par **crossover + mutation** :
  1. Deux génomes sont tirés aléatoirement dans la banque.
  2. Si même `hidden_size` → crossover uniforme des poids + mutation.
  3. Si `hidden_size` différent → clone du **meilleur parent en fitness** + mutation.
- La graine est placée à une position aléatoire sur l'île (cellule libre, non submergée).
- Ce mécanisme maintient la diversité génétique et garantit que la simulation ne s'éteint jamais.

## Deux voies de reproduction

| Voie | Source | Crossover | Mutation | Placement |
|---|---|---|---|---|
| **Reproduction vivante** | Plante mère (énergie > 60, biomasse > 8) | Non — clone du parent | Oui (pendant la dormance) | 3-9 cellules de la mère, direction aléatoire |
| **Banque de graines** | 2 génomes tirés de la banque | Oui (si même `hidden_size`) | Oui (pendant la dormance) | Position aléatoire sur l'île |

Les deux voies coexistent : la reproduction vivante assure la continuité locale (les bonnes stratégies se propagent dans leur voisinage), la banque assure le brassage global et la résilience.

## Mutations

Les mutations s'appliquent **pendant la phase de dormance** de la graine (voir 02-plants.md), quelle que soit la voie de reproduction.

| Trait | Type de mutation | Paramètres |
|---|---|---|
| Poids du réseau | Gaussien | Probabilité 0.3, amplitude σ = 0.2 |
| carbon_nitrogen_ratio | Gaussien | σ = 0.05, clampé [0.3, 0.9] |
| maxSize | Gaussien | σ = 2, clampé [15, 40] |
| exudate_type | Flip | Probabilité rare (~0.01) |
| hidden_size | ±1 | Probabilité ~0.05. Les poids ajoutés/retirés sont initialisés à 0 (neurone inerte) ou supprimés. |

Tous les paramètres de mutation sont configurables (TOML).

## Fonction de fitness

La fitness est calculée **à la mort de chaque plante**, sur la base de sa vie entière.

| Composante | Poids | Raison |
|---|---|---|
| Biomasse max atteinte | + 2.0 | Récompense la croissance |
| Durée de vie | + 1.0 | Encourage la durabilité |
| Territoire max contrôlé | + 1.5 | Récompense l'expansion |
| Connexions symbiotiques (cumul) | + 4.0 | Fort bonus coopération |
| Exsudats émis (cumul) | + 2.0 | Encourage la générosité publique |
| Échanges C↔N via liens directs (cumul) | + 1.5 | Encourage la coopération privée |
| Graines lancées (reproduction vivante) | + 3.0 | Succès biologique |
| Sol enrichi à la mort (décomposition) | + 2.0 | Même la mort contribue |
| Sol épuisé sous soi (cumul) | - 2.0 | Pénalise la surexploitation |
| Monoculture autour de soi | - 1.5 | Si > 80% des cellules dans la zone d'influence sont de la même espèce |

**La pénalité monoculture** : un sol dominé par une seule espèce s'appauvrit naturellement (diversité du sol), et la fitness pénalise les plantes qui contribuent à cette monoculture. L'évolution favorise la coexistence.

## Compteur de génération

Le numéro de génération est un **index global** qui s'incrémente de 1 à chaque graine plantée (reproduction vivante ou injection depuis la banque). C'est une mesure du temps évolutif, pas un cycle artificiel.

## Spéciation émergente

La spéciation n'est pas programmée — elle émerge de la contrainte de crossover sur `hidden_size`. Les cerveaux de même taille se croisent entre eux, formant des clusters. Combiné avec les traits génétiques (`exudate_type`, `carbon_nitrogen_ratio`, `maxSize`), des **espèces fonctionnelles** apparaissent naturellement :

- Fixatrices d'azote à petit cerveau (coopératrices simples)
- Arbres à grand cerveau (stratégies complexes, lentes à émerger)
- Herbes rapides (petit maxSize, convergence rapide)
- Parasites (generosity ≈ 0, exploitation des liens)

## Paramètres configurables

| Paramètre | Valeur par défaut | Description |
|---|---|---|
| seed_bank_size | 50 | Nombre de slots dans la banque |
| seed_rain_rate | 1 graine / 50 ticks | Fréquence d'injection depuis la banque |
| initial_population | 30 | Nombre de graines plantées au démarrage |
| mutation_weight_prob | 0.3 | Probabilité de mutation d'un poids |
| mutation_weight_sigma | 0.2 | Amplitude de la mutation gaussienne |
| mutation_hidden_size_prob | 0.05 | Probabilité de mutation ±1 de hidden_size |
| mutation_exudate_flip_prob | 0.01 | Probabilité de flip exudate_type |
| crossover_rate | 0.7 | Probabilité de crossover (vs clone) quand même hidden_size |
