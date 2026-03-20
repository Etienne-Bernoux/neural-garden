# Interactions entre Plantes

## Exsudats racinaires (coopération publique)

Chaque plante peut injecter des nutriments dans le sol autour de ses racines via l'output `exudate_rate`. C'est une contribution au bien commun : n'importe quelle plante à proximité en bénéficie via l'absorption normale.

| Aspect | Détail |
|---|---|
| Mécanisme | La plante injecte exudate_rate × 2.0 nutriments/tick dans les cellules de ses racines |
| Coût | Énergie : exudate_rate × 1.5 /tick |
| Diffusion | Les exsudats diffusent aux cellules adjacentes, décroissent de 20%/tick |
| Bénéficiaires | Toute plante dont les racines chevauchent les cellules enrichies |
| Visuel | Halo doré/ambré semi-transparent autour des racines émettrices |

**Ce qui émerge :** une plante généreuse (exudate élevé) crée un gradient de nutriments qui attire les racines voisines. Les voisins poussent vers elle → contact racinaire → connexion directe possible. La coopération attire la coopération. À l'inverse, un parasite (exudate 0 mais absorption forte) crée un puits de nutriments — gradient négatif autour de lui.

## Connexion mycorhizienne (coopération privée)

Quand deux systèmes racinaires se touchent (cellules adjacentes), les plantes peuvent former un **lien direct** — modélisant le réseau de champignons mycorhiziens.

1. **Proximité** : cellules de A et B adjacentes (distance ≤ 2).
2. **Accord mutuel** : connect_signal > 0.5 des deux côtés.
3. **Lien établi** : filament doré visible entre les deux plantes.
4. **Transfert** : chaque tick, transfert proportionnel à connect_generosity. La plante la plus riche donne à la plus pauvre.
5. **Rupture** : si les racines ne se touchent plus, ou si une plante tente d'envahir l'autre.

**Parasitisme :** une plante peut avoir connect_signal > 0.5 (accepter le lien) mais connect_generosity ≈ 0 (ne rien donner). Elle reçoit sans contribuer. Stratégie viable à court terme, mais la victime finit par mourir → plus de source.

## Invasion = croissance agressive

Il n'y a pas de « mode invasion ». L'invasion est simplement le résultat de la **croissance dans une cellule occupée**. Quand une plante essaie d'ajouter une cellule et que cette cellule appartient à un voisin plus faible (énergie inférieure), elle la prend.

| Condition | Détail |
|---|---|
| Déclenchement | La plante pousse (grow_intensity > 0) et la cellule cible est occupée |
| Réussite | Si énergie attaquant > énergie défenseur + 10 |
| Défense | Si defense > 0.5 chez le défenseur : le seuil passe à +20 au lieu de +10. Coûte 3 énergie/tick. |
| Coût | 12 énergie (au lieu de 8 pour une cellule libre) |
| Rupture symbiose | Si A envahit B, tout lien mycorhizien entre A et B est rompu |
| Perte pour la victime | -3 vitalité + perte de la cellule. Si 0 cellules : mort. |

## Équilibre écosystémique

| Stratégie | Forces | Faiblesses |
|---|---|---|
| Invasion pure | Gains rapides en territoire | Épuisement (énergie), sol détruit, isolée |
| Exsudats généreux | Attire les voisins, enrichit le sol | Coût énergétique, profite aussi aux parasites |
| Connexion symbiotique | Transfert efficace, résilience collective | Vulnérable au parasitisme |
| Parasitisme | Ressources gratuites | Les victimes meurent → plus de source |
| Défense pure | Résiste aux invasions | Coût énergétique permanent, pas de croissance |
| **Mixte (adaptée)** | **Flexible, durable** | **Plus complexe à évoluer** |
