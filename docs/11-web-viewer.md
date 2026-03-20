# Web Replay Viewer

> **TODO** : cette section est à retravailler — le rendu visuel mérite un design dédié.

## Stack

Vanilla JS + Canvas 2D + ES modules. Zéro build step. GitHub Pages.

## Modules

| Module | Rôle |
|---|---|
| app.js | Chargement replay, boucle rAF |
| state.js | Reconstruction état depuis events |
| renderer.js | Rendu Canvas 2D pixel art |
| garden.js | Sol, eau, rochers |
| plants.js | Plantes, espèces, couleurs par santé |
| symbiosis.js | Liens mycorhiziens, exsudats |
| particles.js | Feuilles, pollen, décomposition |
| timeline.js | Play, pause, scrub, vitesse |
| brain-viz.js | Réseau de neurones de la plante sélectionnée |

## Commandes CLI associées

| Commande | Description | Exemple |
|---|---|---|
| garden replay | Servir le viewer | `garden replay replays/gen500.json` |
