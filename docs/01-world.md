# Le Jardin : Design du Monde

## Forêt sauvage pixel art

Vue top-down d'un sous-bois. Le sol varie du brun riche au vert mousse. Chaque espèce a une **teinte unique** (vert, teal, bleu, violet, lime, orange, rose, bleu clair) pour être identifiable instantanément. La santé se lit par la saturation et la luminosité dans la teinte de l'espèce.

## Grille et ressources

| Paramètre | Valeur | Notes |
|---|---|---|
| Taille | 128×128 cellules | Chaque cellule = 5×5 px à l'écran (640px) |
| Nutriments/cellule | 0 à 100 | Consommés par les racines, régénèrent lentement |
| Humidité/cellule | 0 à 100 | Diffuse depuis les points d'eau |
| Lumière/cellule | 0 à 100 | Réduite sous les canopées (ombre) |
| Exsudats/cellule | 0 à 50 | Injectés par les plantes, diffusent, décroissent |
| Points d'eau | 5 par jardin | Générés aléatoirement, rayon 3-8 cellules |
| Rochers | ~12 par jardin | Obstacles, micro-habitats |
| Tick rate | 30 ticks/seconde | Simulation déterministe |

## Cycle des ressources

- **Surexploitation** : une plante trop gourmande épuise le sol sous elle et dépérit.
- **Décomposition** : les plantes mortes enrichissent le sol (+20 nutriments sur les cellules occupées).
- **Gradient hydrique** : l'humidité diffuse depuis les points d'eau, créant des zones fertiles.
- **Ombre** : les grandes canopées bloquent la lumière. Automatique, pas une décision.
- **Diversité du sol** : un sol exploité par une seule espèce s'appauvrit plus vite qu'un sol partagé entre plusieurs espèces. Cela récompense naturellement la biodiversité sans bonus artificiel.
- **Exsudats** : les plantes injectent des nutriments dans le sol via leurs racines. Ces exsudats diffusent aux cellules voisines et décroissent sur ~5 ticks. C'est le mécanisme de coopération diffuse.

## Saisons

| Saison | Ticks | Lumière | Régen sol | Croissance |
|---|---|---|---|---|
| Printemps | 0 – 750 | 0.9 | 0.12/tick | x1.5 |
| Été | 750 – 1650 | 1.0 | 0.08/tick | x1.0 |
| Automne | 1650 – 2400 | 0.5 | 0.15/tick | x0.5 |
| Hiver | 2400 – 3000 | 0.25 | 0.04/tick | x0.1 |
