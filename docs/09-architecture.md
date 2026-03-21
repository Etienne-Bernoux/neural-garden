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
│   │   │   │   ├── plant.rs                # Entity Plant, value objects (Vitality, Energy, Biomass, Lineage, GeneticTraits)
│   │   │   │   ├── brain.rs                # Entity Brain, forward pass 18→H→H→8
│   │   │   │   ├── world.rs                # Entity World, Grid, Cell
│   │   │   │   ├── island.rs               # Île : topographie, masque terre/mer
│   │   │   │   ├── rng.rs                  # Trait Rng pour l'injection de l'aléatoire
│   │   │   │   ├── symbiosis.rs            # Liens mycorhiziens, échanges C↔N
│   │   │   │   └── events.rs               # Domain events (Grew, Died, Born, Invaded, Linked...)
│   │   │   ├── application/                # Cas d'usage, orchestration
│   │   │   │   ├── mod.rs
│   │   │   │   ├── config.rs              # SimConfig — paramètres de simulation
│   │   │   │   ├── sim.rs                  # SimState, run_tick — orchestration du game loop
│   │   │   │   ├── environment.rs          # Phase environnement (pluie, ombrage, décomposition)
│   │   │   │   ├── actions.rs              # Phase actions (croissance, invasion, défense, exsudats, symbiose)
│   │   │   │   ├── lifecycle.rs            # Phase vie/mort (reproduction, germination, pluie de graines, GC)
│   │   │   │   ├── perception.rs           # Service : calcul des 18 inputs (gradients sur racines)
│   │   │   │   ├── evolution.rs            # Banque de graines, fitness, crossover, mutations
│   │   │   │   ├── season.rs               # Cycle saisonnier (4 saisons, 250 ticks chacune)
│   │   │   │   ├── highlights.rs           # Service : détection des moments clés pour le replay
│   │   │   │   └── metrics.rs              # Métriques agrégées (population, lignées, historiques)
│   │   │   ├── infra/                      # Sérialisation, I/O, config
│   │   │   │   ├── mod.rs
│   │   │   │   ├── rng.rs                  # SeededRng (rand) — implémente trait domain::Rng
│   │   │   │   ├── noise.rs                # Génération Perlin noise pour les îles
│   │   │   │   ├── config.rs               # Chargement config TOML
│   │   │   │   ├── persistence.rs          # Sauvegarde/chargement SimState en JSON
│   │   │   │   ├── replay.rs               # Enregistrement replay avec filtrage highlights
│   │   │   │   └── dto/                    # DTOs serde miroir des types domain
│   │   │   │       ├── mod.rs
│   │   │   │       ├── plant.rs
│   │   │   │       ├── world.rs
│   │   │   │       ├── sim.rs
│   │   │   │       └── event.rs
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
| ctrlc | Signal handler Ctrl+C | garden-cli |
| toml | Config | infra |
| cucumber | Tests d'intégration Gherkin | dev-dependency |

**Zéro dépendance ML.** Réseau from scratch.

## Viewer web

**Vanilla JS + Canvas 2D + ES modules**. Zéro build step.

Le fichier `state.js` est le cœur : il prend le header + les events d'un clip et reconstruit l'état de la grille à n'importe quel tick via les keyframes. `clips.js` gère la navigation entre clips du montage. Le renderer dessine l'état courant.

Mode live possible : le viewer se connecte à la simulation en cours (via WebSocket ou polling du fichier d'état).
