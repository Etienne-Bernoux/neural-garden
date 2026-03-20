# Moteur de Neuroévolution

## Cycle

1. **Plantation** : 200 cerveaux. Chaque simulation place 12 plantes aléatoires dans un jardin frais.
2. **Croissance** : 3000 ticks (~100s simulées). Les plantes poussent, interagissent, meurent.
3. **Évaluation** : fitness calculée. Chaque cerveau est évalué sur 3 simulations (réduire la variance).
4. **Sélection** : tournament selection, taille 5.
5. **Reproduction** : crossover uniforme des poids + mutation gaussienne.
6. **Élitisme** : 5% des meilleurs copiés directement.

## Paramètres

| Paramètre | Valeur | Configurable |
|---|---|---|
| Taille population | 200 | Oui (TOML) |
| Plantes par simulation | 12 | Oui |
| Durée simulation | 3000 ticks | Oui |
| Mutation poids | 0.3 (probabilité) | Oui |
| Amplitude mutation | 0.2 (σ gaussien) | Oui |
| Crossover | 0.7 | Oui |
| Tournoi | 5 | Oui |
| Élitisme | 5% | Oui |
| Rounds d'évaluation | 3 | Oui |

## Fonction de fitness

| Composante | Poids | Raison |
|---|---|---|
| Biomasse finale (nb cellules) | + 2.0 | Récompense la croissance |
| Durée de vie | + 1.0 | Encourage la durabilité |
| Territoire contrôlé | + 1.5 | Récompense l'expansion |
| Connexions symbiotiques actives | + 4.0 | Fort bonus coopération |
| Nutriments partagés (exsudats émis) | + 2.0 | Encourage la générosité publique |
| Nutriments transférés via lien direct | + 1.5 | Encourage la coopération privée |
| Graines lancées (reproduction) | + 3.0 | Succès biologique |
| Sol enrichi (à la mort, décomposition) | + 2.0 | Même la mort contribue |
| Sol épuisé sous soi | - 2.0 | Pénalise la surexploitation |
| Monoculture autour de soi | - 1.5 | Si > 80% des cellules dans le rayon 10 sont de la même espèce |

**La pénalité monoculture** remplace le bonus biodiversité artificiel. Un sol dominé par une seule espèce s'appauvrit naturellement (diversité du sol), et la fitness pénalise les plantes qui contribuent à cette monoculture. Résultat : l'évolution favorise la coexistence.
