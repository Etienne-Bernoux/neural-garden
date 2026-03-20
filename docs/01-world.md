# Le Jardin : Design du Monde

## L'île — terrain procédural

Le jardin est une **île** générée procéduralement à chaque itération d'entraînement. Cela force les cerveaux à généraliser plutôt que mémoriser un terrain fixe.

- **Génération** : bruit de Perlin pour l'altitude, masque circulaire pour garantir que les bords sont sous l'eau.
- **Mer** : les cellules sous le niveau de la mer sont des obstacles infranchissables (eau salée, non exploitable).
- **Topographie** : l'altitude varie à l'intérieur de l'île. Crêtes, pentes, vallées, creux.

Chaque génération d'entraînement = une île différente.

## Grille et couches

| Paramètre | Valeur | Notes |
|---|---|---|
| Taille | 128×128 cellules | Chaque cellule = 5×5 px à l'écran (640px) |
| Altitude/cellule | 0.0 à 1.0 | Généré par Perlin noise. < seuil mer = obstacle |
| Carbone/cellule | 0 à 100 | Ressource sol — libéré massivement à la décomposition |
| Azote/cellule | 0 à 100 | Ressource sol — libéré plus lentement, fixable par symbiose |
| Humidité/cellule | 0 à 100 | Source : pluie. Ruisselle, s'accumule, s'évapore |
| Lumière/cellule | 0 à 100 | Réduite par canopée + orientation des pentes |
| Exsudats/cellule | 0 à 50 | Injectés par les plantes, diffusent, décroissent |
| Tick rate | 30 ticks/seconde | Simulation déterministe |

## Cycle de l'eau

### Sources
- **Pluie** : seule source d'eau douce. Recharge globale modulée par la saison. Tombe uniformément sur toute l'île.
- La mer est salée : elle ne fournit pas d'humidité exploitable.

### Ruissellement
- **Bassins versants calculés au setup** : à la génération de l'île, on pré-calcule pour chaque cellule vers quel creux elle draine (flow accumulation).
- **Ruissellement périodique** (tous les N ticks, pas chaque tick) : l'eau accumulée coule vers les cellules plus basses. Les vallées et creux collectent l'eau de leur bassin versant.
- **Évaporation** : l'humidité décroît progressivement. Les zones exposées (crêtes, pas de couvert) s'assèchent plus vite.

### Mares émergentes
- Les creux qui accumulent suffisamment d'eau deviennent des **cellules submergées** : obstacles naturels où rien ne pousse.
- Les mares sont dynamiques : elles se remplissent avec la pluie, s'assèchent en été.
- Elles remplacent les points d'eau fixes et les rochers du design initial.

### Végétation et rétention
- **La canopée retient l'humidité** : les cellules sous couvert végétal s'évaporent moins vite. La forêt crée son propre microclimat humide.
- Feedback loop : forêt dense → ombre → humidité retenue → plus de croissance → forêt plus dense.

## Cycle carbone / azote

Deux ressources distinctes dans le sol, consommées par les plantes pour croître.

### Carbone
- Libéré **massivement à la décomposition** (proportionnel à la biomasse de la plante morte).
- Régénération naturelle lente (matière organique de fond).
- Ressource dominante pour la structure/croissance.

### Azote
- Libéré **plus lentement à la décomposition** (proportionnel à l'âge/maturité de la plante).
- Peut être **fixé par les exsudats** : certaines espèces évoluent comme fixatrices d'azote (type légumineuses).
- Ressource limitante — c'est le levier principal de la symbiose.

### Consommation
- Chaque espèce a un **ratio carbone/azote variable**, déterminé par l'évolution.
- Les espèces qui consomment beaucoup d'azote dépendent des fixatrices. Les espèces frugales en azote sont plus autonomes mais poussent moins vite.
- Ce ratio est un paramètre évolutif du cerveau/génome, pas un input du réseau.

## Lumière et topographie

- **Canopée** : les grandes plantes bloquent la lumière pour les cellules en dessous et adjacentes. Effet automatique, pas une décision.
- **Pentes et exposition** : les pentes orientées sud (dans l'hémisphère nord) reçoivent plus de lumière. Les pentes nord sont plus ombragées mais retiennent mieux l'humidité.
- Double trade-off : crêtes ensoleillées mais sèches / vallées humides mais ombragées.

## Saisons

| Saison | Ticks | Lumière | Pluie | Croissance |
|---|---|---|---|---|
| Printemps | 0 – 750 | 0.9 | Forte | x1.5 |
| Été | 750 – 1650 | 1.0 | Faible (sec) | x1.0 |
| Automne | 1650 – 2400 | 0.5 | Modérée | x0.5 |
| Hiver | 2400 – 3000 | 0.25 | Faible (froid) | x0.1 |

La pluie forte au printemps remplit les mares et relance la végétation. L'été sec crée un stress hydrique — les plantes en crête souffrent, celles en vallée résistent. L'automne recharge modérément avant l'hiver.
