# Planning et Roadmap

## Semaine 1 — Core Engine

| Jour | Tâche | Livrable |
|---|---|---|
| J1 | Setup workspace, Plant (cellules), World (grille 128×128, sol, eau, rochers) | Compilation + tests unitaires |
| J2 | Brain (13→10→10→8, forward pass) + Perception (gradients, rayon 10) | Test: outputs pour inputs aléatoires |
| J3 | Croissance (ajout cellules) + absorption + photosynthèse + reproduction | Test: plantes qui poussent et se multiplient |
| J4 | Invasion (croissance dans cellule occupée) + défense + mort/décomposition | Test: compétition territoriale fonctionnelle |
| J5 | Exsudats (injection + diffusion) + connexion mycorhizienne + transfert | Test: coopération fonctionnelle |
| J6 | Evolution (population, selection, crossover, mutation, fitness) | Test: fitness augmente sur 50 générations |
| J7 | Saisons + config TOML + polish + premiers vrais entraînements | Entraînement complet CLI |

## Semaine 2 — TUI + Replay export

| Jour | Tâche | Livrable |
|---|---|---|
| J8-J9 | TUI ratatui: layout, rendu Braille du jardin, couleurs par espèce | Visu temps réel |
| J10 | Courbe fitness, logs, stats, toggle mycélium/exsudats | Dashboard complet |
| J11 | Event sourcing: émission des events + keyframes + sérialisation JSON | Replays exploitables |
| J12 | Parallélisme rayon pour les simulations | x4 speedup |
| J13-J14 | Polish TUI, hotkeys, focus plante, ajustements équilibre | TUI prêt démo |

## Semaine 3 — Web Replay Viewer

| Jour | Tâche | Livrable |
|---|---|---|
| J15-J16 | state.js (reconstruction depuis events), renderer basique Canvas | Replay jouable |
| J17-J18 | Rendu pixel art: espèces colorées, fleurs, exsudats, liens dorés | Rendu forêt complet |
| J19 | Particules saisonnières, heatmaps nutriments/humidité | Rendu spectaculaire |
| J20 | Panneau plante, brain-viz, timeline, graphe biodiversité | Viewer complet |
| J21 | Déploiement GitHub Pages, README, démo GIF/vidéo | Projet publié |

## Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| La symbiose n'émerge jamais | Élevé | Réduire coût exsudats, augmenter le bonus fitness coopération |
| L'invasion domine toujours | Moyen | Augmenter coût invasion (15 au lieu de 12), renforcer défense |
| Gradients mal calibrés | Moyen | Rayon et normalisation configurables. Tester 8, 10, 12. |
| Event sourcing complexe à implémenter | Moyen | Commencer par un format simple (grow/died/born), enrichir après |
| state.js trop lent pour le scrub | Faible | Keyframes toutes les 100 ticks si nécessaire |
| Scope creep | Élevé | Pas de son, pas de 3D, pas de pollinisation en v1 |

## Évolutions post-v1

- **Toxicité (allélopathie)** : plantes qui empoisonnent le sol. Nouvel output toxic_rate.
- **Pollinisation** : insectes attirés par les fleurs, boostent la reproduction.
- **Herbivores** : agents externes qui mangent les canopées (pression de sélection).
- **NEAT** : topologie dynamique (ajout/suppression de neurones par mutation).
- **Dispersion active** : output pour contrôler la direction de lancer de graine.
- **Climat évolutif** : sécheresses, inondations, changement climatique progressif.
- **Multi-espèces (stats différentes)** : arbre = lent résilient, herbe = rapide fragile.
- **Son procédural** : ambiance forêt (vent, pluie, oiseaux).
- **Commentateur IA** : LLM local qui narre l'évolution.
- **3D isometrique** : Three.js pour le replay.

## Commandes CLI

| Commande | Description | Exemple |
|---|---|---|
| garden grow | Lancer un entraînement | `garden grow --gen 500 --pop 200` |
| garden grow --resume | Reprendre | `garden grow --resume saves/gen_150.json` |
| garden plant | Simulation unique | `garden plant brain1.json brain2.json --replay out.json` |
| garden replay | Servir le viewer | `garden replay replays/gen500.json` |
| garden inspect | Détails d'un cerveau | `garden inspect brain.json` |
| garden benchmark | Perf test | `garden benchmark --ticks 10000` |
| garden config init | Générer config | `garden config init` |
