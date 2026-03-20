# Architecture Technique

## Structure du projet

```
neural-garden/
├── Cargo.toml                              # Workspace
├── CLAUDE.md                               # Rôles, workflow, architecture
├── .claude/rules/                          # Règles réutilisables
│   ├── conventions-code.md
│   ├── ddd.md
│   └── tests.md
├── docs/                                   # Design docs
├── crates/
│   ├── garden-core/                        # Tout le moteur (DDD interne)
│   │   ├── src/
│   │   │   ├── domain/                     # Cœur métier — zéro dépendance externe
│   │   │   │   ├── mod.rs
│   │   │   │   ├── plant.rs                # Entity Plant, value objects (Vitality, Energy, Biomass)
│   │   │   │   ├── brain.rs                # Entity Brain, forward pass 18→H→H→8
│   │   │   │   ├── world.rs                # Entity World, Grid, Cell, Altitude
│   │   │   │   ├── island.rs               # Île : topographie, bassins versants, mares
│   │   │   │   ├── species.rs              # Value object Lineage, traits génétiques
│   │   │   │   ├── symbiosis.rs            # Liens mycorhiziens, échanges C↔N
│   │   │   │   └── events.rs               # Domain events (Grew, Died, Invaded, Linked, Flood...)
│   │   │   ├── application/                # Cas d'usage, orchestration
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sim.rs                  # Use case : run_tick (game loop continue)
│   │   │   │   ├── evolution.rs            # Use case : banque de graines, fitness, pluie de graines
│   │   │   │   ├── perception.rs           # Service : calcul des gradients sur zone d'influence
│   │   │   │   └── highlights.rs           # Service : détection des moments clés pour le replay
│   │   │   ├── infra/                      # Sérialisation, I/O, config
│   │   │   │   ├── mod.rs
│   │   │   │   ├── replay.rs               # Event sourcing → JSON, clips, montage
│   │   │   │   ├── config.rs               # Désérialisation TOML
│   │   │   │   ├── persistence.rs          # Sauvegarde/chargement banque de graines
│   │   │   │   ├── rng.rs                  # Implémentation du trait Rng (rand)
│   │   │   │   └── noise.rs                # Génération Perlin noise pour les îles
│   │   │   └── lib.rs                      # Ré-exports publics
│   │   └── tests/
│   │       ├── features/                   # Fichiers .feature (Gherkin, français)
│   │       │   ├── croissance.feature
│   │       │   ├── symbiose.feature
│   │       │   ├── invasion.feature
│   │       │   ├── decomposition.feature
│   │       │   └── saisons.feature
│   │       └── steps/                      # Step definitions (cucumber-rs)
│   │           └── mod.rs
│   └── garden-cli/                         # CLI + TUI
│       └── src/
│           ├── main.rs                     # Commandes clap
│           └── tui.rs                      # ratatui (dashboard, rendu Braille)
├── web/                                    # Replay viewer (vanilla JS)
│   ├── index.html
│   ├── style.css
│   ├── js/
│   │   ├── app.js                          # Chargement replay/montage, boucle rAF
│   │   ├── state.js                        # Reconstruction état depuis events + keyframes
│   │   ├── renderer.js                     # Rendu Canvas 2D pixel art
│   │   ├── island.js                       # Île, altitude, mares, humidité
│   │   ├── plants.js                       # Plantes, lignées, couleurs par santé
│   │   ├── symbiosis.js                    # Liens mycorhiziens, exsudats
│   │   ├── particles.js                    # Feuilles, pollen, décomposition
│   │   ├── timeline.js                     # Play, pause, scrub entre clips, vitesse
│   │   ├── clips.js                        # Navigation entre clips du montage
│   │   └── brain-viz.js                    # Réseau de neurones de la plante sélectionnée
│   └── tests/
│       ├── features/                       # Fichiers .feature (Gherkin, français)
│       │   └── replay.feature
│       └── steps/                          # Step definitions (Playwright + playwright-bdd)
│           └── replay.steps.ts
└── replays/                                # Fichiers JSON (montages de clips)
```

## DDD interne (garden-core)

Voir `.claude/rules/ddd.md` pour les règles complètes.

| Couche | Dépend de | Contenu |
|---|---|---|
| `domain/` | Rien d'externe | Entities (Plant, Brain, World), value objects (Vitality, Energy, Lineage), domain events, traits pour l'injection de dépendances (Rng) |
| `application/` | `domain/` uniquement | Use cases (run_tick, banque de graines, fitness), services (perception, highlights) |
| `infra/` | `domain/` + `application/` | Sérialisation JSON (serde), config TOML, Perlin noise, implémentation Rng (rand), persistence |

## Crates Rust

| Crate | Usage | Couche DDD |
|---|---|---|
| clap | CLI | garden-cli |
| ratatui + crossterm | TUI | garden-cli |
| serde + serde_json | Events, replays, sauvegardes | infra |
| rand + rand_distr | Mutations, spawns, positions | infra (via trait Rng) |
| noise | Perlin noise pour la génération d'île | infra |
| rayon | Parallélisme (perception, ruissellement) | infra |
| toml | Config | infra |
| cucumber | Tests d'intégration Gherkin | dev-dependency |

**Zéro dépendance ML.** Réseau from scratch.

## Viewer web

**Vanilla JS + Canvas 2D + ES modules**. Zéro build step.

Le fichier `state.js` est le cœur : il prend le header + les events d'un clip et reconstruit l'état de la grille à n'importe quel tick via les keyframes. `clips.js` gère la navigation entre clips du montage. Le renderer dessine l'état courant.

Mode live possible : le viewer se connecte à la simulation en cours (via WebSocket ou polling du fichier d'état).
