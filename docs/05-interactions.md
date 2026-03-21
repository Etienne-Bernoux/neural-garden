# Interactions entre Plantes

## Zones et compétition

Chaque plante a deux zones (voir 03-perception.md) :

- **Zone de place** : cellules physiquement occupées. Exclusive — une seule plante par cellule.
- **Zone d'influence** : zone élargie d'absorption et d'interaction. Le rayon scale avec la biomasse et l'investissement en racines (output `canopy_vs_roots`, voir 03-perception.md). Peut se superposer avec celle d'autres plantes.

Quand les zones d'influence de deux plantes se chevauchent, elles sont en **compétition passive** pour les ressources (carbone, azote, humidité) dans la zone de recouvrement. Pas besoin d'invasion — l'absorption est partagée, la plus efficace gagne.

## Exsudats racinaires (coopération publique)

Chaque plante peut injecter des ressources dans le sol autour de ses racines via l'output `exudate_rate`. Le **type exsudé** (carbone ou azote) est un **trait génétique** (`exudate_type`), pas une décision par tick. Les fixatrices d'azote émergent par évolution.

| Aspect | Détail |
|---|---|
| Mécanisme | La plante injecte exudate_rate × exudate_output_rate (0.02 par défaut) de carbone OU d'azote (selon `exudate_type`) par tick dans les cellules racinaires. Les taux exacts sont configurables via `SimConfig`. |
| Coût | Énergie : exudate_rate × exudate_energy_cost_rate (0.015 par défaut) /tick |
| Diffusion | Les exsudats diffusent aux cellules adjacentes, décroissent de 20%/tick |
| Bénéficiaires | Toute plante dont la zone d'influence chevauche les cellules enrichies |
| Visuel | Halo doré/ambré semi-transparent autour des racines émettrices |

**Ce qui émerge :** une fixatrice d'azote (exudate_type = azote) crée un gradient d'azote qui attire les plantes voisines carencées. Les voisins poussent vers elle → zones d'influence se croisent → lien mycorhizien possible. La coopération attire la coopération. À l'inverse, un parasite (exudate 0 mais absorption forte) crée un puits — gradient négatif autour de lui.

## Connexion mycorhizienne (coopération privée)

Quand les **racines** de deux plantes occupent une même cellule, elles peuvent former un **lien direct** — modélisant le réseau de champignons mycorhiziens.

1. **Racines partagées** : les racines de A et B occupent au moins une cellule commune.
2. **Accord mutuel** : `connect_signal > 0.5` des deux côtés.
3. **Lien établi** : filament doré visible entre les deux plantes.
4. **Échange C↔N** : chaque tick, chaque plante connectée donne son **surplus** de carbone ou d'azote, proportionnellement à `connect_generosity`. L'échange est bidirectionnel — une plante riche en carbone donne du carbone et reçoit de l'azote, et inversement.
5. **Rupture** : si les racines ne partagent plus de cellule, ou si une plante envahit la zone de place de l'autre.

### Parasitisme mycorhizien

Une plante peut avoir `connect_signal > 0.5` (accepter le lien) mais `connect_generosity ≈ 0` (ne rien donner en retour). Elle reçoit le surplus de l'autre sans contribuer. Stratégie viable à court terme, mais la victime s'affaiblit → produit moins de surplus → le parasite y perd aussi.

Avec l'échange C↔N, le parasitisme est plus nuancé : une plante peut être généreuse en carbone (qu'elle produit facilement via photosynthèse) mais avare en azote (qu'elle fixe et garde). L'évolution explore ces stratégies mixtes.

## Invasion = croissance agressive

Il n'y a pas de « mode invasion ». L'invasion est le résultat de la **croissance dans la zone de place d'un autre**. Quand une plante essaie d'ajouter une cellule et que cette cellule appartient à la zone de place d'un voisin plus faible, elle la prend.

| Condition | Détail |
|---|---|
| Déclenchement | La plante pousse (`grow_intensity > 0`) et la cellule cible est dans la zone de place d'une autre plante |
| Réussite | Si énergie attaquant > énergie défenseur + 10 |
| Défense | Si `defense > 0.5` chez le défenseur : le seuil passe à +20 au lieu de +10. Coûte 3 énergie/tick. |
| Coût | 12 énergie (au lieu de 8 pour une cellule libre) |
| Rupture symbiose | Si A envahit la zone de place de B, tout lien mycorhizien entre A et B est rompu |
| Perte pour la victime | -3 vitalité + perte de la cellule. Si 0 cellules : mort. |

## Équilibre écosystémique

| Stratégie | Forces | Faiblesses |
|---|---|---|
| Invasion pure | Gains rapides en territoire | Épuisement (énergie), sol détruit, isolée |
| Fixatrice d'azote | Attire les voisins, indispensable à l'écosystème | Coût énergétique des exsudats |
| Exsudats carbone | Enrichit le sol, favorise la décomposition rapide | Profite aussi aux parasites |
| Connexion symbiotique C↔N | Échange efficace, résilience collective | Vulnérable au parasitisme |
| Parasitisme mycorhizien | Ressources gratuites à court terme | La victime meurt → plus de source |
| Défense pure | Résiste aux invasions | Coût énergétique permanent, pas de croissance |
| **Mixte (adaptée)** | **Flexible, durable** | **Plus complexe à évoluer** |
