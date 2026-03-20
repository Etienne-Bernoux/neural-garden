# Architecture Technique

## Structure du projet

```
neural-garden/
├── Cargo.toml                     # Workspace
├── crates/
│   ├── garden-core/               # Tout le moteur
│   │   └── src/
│   │       ├── lib.rs             # Ré-exports
│   │       ├── brain.rs           # Réseau 13→10→10→8, forward pass
│   │       ├── world.rs           # Grille 128×128, sol, eau, exsudats, ombre
│   │       ├── plant.rs           # Struct Plant, cellules, stats, liens
│   │       ├── sim.rs             # Boucle de simulation (1 tick)
│   │       ├── perception.rs      # Calcul des gradients (rayon 10)
│   │       ├── evolution.rs       # Population, sélection, crossover, mutation
│   │       ├── replay.rs          # Event sourcing, sérialisation JSON, keyframes
│   │       └── config.rs          # Désérialisation TOML
│   └── garden-cli/                # CLI + TUI
│       └── src/
│           ├── main.rs            # Commandes clap
│           └── tui.rs             # ratatui (dashboard, rendu Braille)
├── web/                           # Replay viewer (vanilla JS)
│   ├── index.html
│   ├── style.css
│   └── js/
│       ├── app.js                 # Chargement replay, boucle rAF
│       ├── state.js               # Reconstruction état depuis events
│       ├── renderer.js            # Rendu Canvas 2D pixel art
│       ├── garden.js              # Sol, eau, rochers
│       ├── plants.js              # Plantes, espèces, couleurs par santé
│       ├── symbiosis.js           # Liens mycorhiziens, exsudats
│       ├── particles.js           # Feuilles, pollen, décomposition
│       ├── timeline.js            # Play, pause, scrub, vitesse
│       └── brain-viz.js           # Réseau de neurones de la plante sélectionnée
└── replays/                       # Fichiers JSON
```

## Crates Rust

| Crate | Usage |
|---|---|
| ratatui + crossterm | TUI (semaine 2) |
| clap | CLI |
| serde + serde_json | Events, replays, sauvegardes |
| rand + rand_distr | Mutations, spawns, positions aléatoires |
| rayon | Parallélisme des simulations |
| toml | Config |

**Zéro dépendance ML.** Réseau from scratch.

## Viewer web

**Vanilla JS + Canvas 2D + ES modules**. Zéro build step. Le fichier `state.js` est le cœur : il prend le header + les events et reconstruit l'état de la grille à n'importe quel tick via les keyframes. Le renderer dessine cet état.
