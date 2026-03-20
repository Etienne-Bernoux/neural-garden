# Interface TUI — Dashboard d'entraînement

La TUI est un **dashboard de monitoring**, pas un film. Le spectacle c'est le web viewer. La TUI permet de suivre la progression de l'évolution et de piloter la simulation.

## Les 5 questions du dashboard

Chaque zone du dashboard répond à une question concrète.

### 1. Est-ce que ça converge ?

- Courbe de fitness : meilleure, moyenne et pire de la banque de graines dans le temps
- Fitness de la dernière plante morte
- Compteur de génération (nb total de graines plantées)

### 2. Est-ce que c'est diversifié ?

- Nombre de lignées vivantes
- Répartition des `hidden_size` (histogramme : combien de plantes par taille de cerveau)
- Répartition `exudate_type` (fixatrices N vs exsudatrices C)
- Âge de la plus vieille lignée

### 3. La coopération émerge ?

- Nombre de liens mycorhiziens actifs / population vivante
- Volume d'échanges C↔N (total par tick, moyenne glissante)
- Volume d'exsudats injectés (carbone + azote séparés)
- Ratio invasions / symbioses (moyenne glissante)

### 4. L'île est en bonne santé ?

- Population vivante (nb plantes)
- Couverture végétale (% de cellules occupées)
- Carbone moyen et azote moyen dans le sol
- Nombre de mares actives
- Saison en cours

### 5. Faut-il ajuster ?

- Taux de mortalité (morts par tick, moyenne glissante)
- Taux de germination (graines qui germent / graines plantées)
- Durée de vie moyenne à la mort
- Banque de graines : taille, fitness min/max, diversité des `hidden_size` représentés

## Layout

| Zone | Position | Contenu |
|---|---|---|
| Fitness | Haut-gauche (50%) | Courbe de fitness (meilleure/moyenne/pire banque) dans le temps |
| Diversité | Haut-droite (50%) | Histogramme hidden_size, nb lignées, répartition exudate_type |
| Coopération | Milieu-gauche (50%) | Courbe ratio symbioses/invasions, volume échanges C↔N |
| Santé île | Milieu-droite (50%) | Population, couverture, C/N sol, mares, saison |
| Alertes | Bas (100%) | Derniers événements marquants : record fitness, extinction lignée, fork lignée, mort de masse |

## Contrôles de simulation

| Touche | Action |
|---|---|
| Espace | Pause / Reprendre la simulation |
| q | Arrêter la simulation et quitter |
| s | Sauvegarder l'état (banque de graines + île + état simulation) |
| r | Exporter le montage de clips (highlights) en JSON |
| + / - | Vitesse de simulation (tick rate) |
| 1-5 | Focus sur un panneau (plein écran temporaire) |
| Escape | Retour au layout complet |
