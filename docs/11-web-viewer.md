# Web Viewer — 3D Isométrique

## Vision

Le viewer est le "documentaire nature" de Neural Garden. Un monde voxel en 3D isométrique, beau et immersif, qui donne vie à l'évolution. Deux modes de caméra : vue Dieu pour comprendre l'écosystème, exploration au sol pour l'immersion.

## Stack

- **Three.js** (import CDN, pas de bundler)
- **Vanilla JS + ES modules**
- **Zéro build step**
- **GitHub Pages** pour le déploiement

## Monde voxel

Le monde est construit en blocs, mapping direct entre la grille 128×128 et le rendu.

### Terrain
- Chaque cellule de la grille = une colonne de blocs. La hauteur = altitude (Perlin noise).
- Couleur du sol par altitude + humidité : brun sec en crête, vert mousse en vallée, sable clair en côte.
- Greedy meshing pour fusionner les faces adjacentes (performance).

### Eau
- **Mer** : plan d'eau infini autour de l'île. Shader de vagues douces, couleur profonde.
- **Mares** : blocs d'eau semi-transparents dans les creux. Le niveau monte et descend avec la pluie et les saisons. Reflets simples.

### Plantes
- **Style organique voxelisé** : le tronc est une colonne de blocs fins, la canopée un cluster de blocs arrondis (sphère voxelisée, comme les arbres Minecraft).
- La **taille** scale avec la biomasse : une graine = 1 petit bloc au sol, un arbre mature = tronc de 3-5 blocs + canopée large.
- La **couleur** = teinte de la lignée. La **saturation/luminosité** = santé/vitalité.
- **Graines en dormance** : petit bloc lumineux au sol, couleur atténuée.
- **Dépérissement** : les blocs de canopée se ternissent, brunissent, disparaissent progressivement.
- **Fleurs (maturité)** : petits blocs de couleur vive au sommet de la canopée.

### Interactions visibles
- **Liens mycorhiziens** : filaments dorés lumineux entre les bases des plantes connectées, au niveau du sol. Pulsation subtile.
- **Exsudats** : halo coloré au sol autour de la plante (doré pour carbone, bleuté pour azote), qui s'étend et s'estompe.
- **Invasion** : flash rouge bref sur le bloc pris.
- **Décomposition** : les blocs de la plante morte s'effondrent et se dissolvent en particules qui rejoignent le sol.

### Ambiance et éclairage
- **DirectionalLight** qui change d'angle, d'intensité et de teinte selon la saison :

| Saison | Lumière | Teinte | Ambiance |
|---|---|---|---|
| Printemps | Douce, montante | Dorée chaude | Renouveau, couleurs vives |
| Été | Directe, forte | Blanche | Contrastes marqués, ombres courtes |
| Automne | Rasante | Orangée | Teintes chaudes, couleurs qui se ternissent |
| Hiver | Faible, diffuse | Bleutée froide | Ambiance calme, couleurs désaturées |

- **Ombres projetées** par les canopées sur le sol (shadow map).
- **Brouillard** léger en hiver, clarté en été.

## Deux modes de caméra

### Mode Dieu (vue isométrique)
- **Caméra orthographique** inclinée (45° classique iso).
- Vue d'ensemble de l'île.
- **Contrôles** : scroll pour zoomer, clic-drag pour se déplacer, molette pour tourner.
- Clic sur une plante = sélection (panneau latéral affiche ses stats).
- Pas d'overlay sur le terrain — le visuel reste pur.

### Mode Exploration (balade au sol)
- **Caméra perspective** à hauteur de plante.
- **Contrôles** : WASD pour se déplacer, souris pour regarder autour.
- On marche entre les troncs, sous les canopées, au bord des mares.
- La caméra suit le relief (monte sur les collines, descend dans les vallées).
- Les liens mycorhiziens, les exsudats, les particules de décomposition sont visibles de près.

### Switch
- Touche pour basculer entre les deux modes (ex: `Tab` ou `V`).
- Transition fluide (la caméra anime entre les deux positions).

## Interface

### Panneau latéral (toujours visible)
- **Timeline** : position dans le clip, play/pause, scrub, vitesse, navigation entre clips du montage.
- **Stats globales** : population, nb lignées, saison, compteur de génération.
- **Plante sélectionnée** (si clic) : vitalité, énergie, biomasse, traits génétiques, lignée, liens actifs.

### Panneau cerveau (toggle)
- Visualisation du réseau de neurones de la plante sélectionnée.
- 18 inputs → H → H → 8 outputs, avec les valeurs actuelles colorées (positif = vert, négatif = rouge).
- Les outputs actifs sont mis en évidence (grow, exudate, defense...).

### Contrôles

| Touche | Action |
|---|---|
| V | Switch mode Dieu / Exploration |
| Espace | Play / Pause du replay |
| ← → | Clip précédent / suivant |
| + / - | Vitesse de lecture |
| B | Toggle panneau cerveau |
| Clic | Sélection plante (mode Dieu) |
| WASD | Déplacement (mode Exploration) |
| Souris | Regarder autour (mode Exploration) |
| Scroll | Zoom (mode Dieu) |

## Modules JS

| Module | Rôle |
|---|---|
| app.js | Init Three.js, chargement montage, boucle rAF |
| state.js | Reconstruction état depuis events + keyframes |
| terrain.js | Génération du mesh voxel terrain + eau |
| plants.js | Génération des plantes voxel (tronc, canopée, graines) |
| symbiosis.js | Liens mycorhiziens (filaments), exsudats (halos) |
| particles.js | Particules de décomposition, pollen, feuilles |
| lighting.js | Éclairage saisonnier, ombres, brouillard |
| camera-god.js | Caméra orthographique + contrôles iso |
| camera-explore.js | Caméra perspective + contrôles WASD |
| clips.js | Navigation entre clips du montage |
| timeline.js | Play, pause, scrub, vitesse |
| panel.js | Panneau latéral (stats, plante sélectionnée) |
| brain-viz.js | Visualisation du réseau de neurones |

## Mode live

En complément du replay par clips, le viewer peut se connecter à la simulation en cours via **WebSocket** pour une vue temps réel. Même rendu, même interface, mais les données arrivent en streaming au lieu d'être lues depuis un fichier JSON.

## Commandes CLI associées

| Commande | Description | Exemple |
|---|---|---|
| garden replay | Servir le viewer + un fichier de montage | `garden replay replays/montage_001.json` |
| garden live | Servir le viewer en mode live (WebSocket) | `garden live --port 8080` |
