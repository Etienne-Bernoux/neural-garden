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
│   │   │   │   ├── season.rs               # Cycle saisonnier (4 saisons, 360 ticks chacune)
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
│           ├── runner.rs                   # Boucle de simulation (headless ou TUI)
│           ├── tui.rs                      # ratatui (dashboard, rendu Braille)
│           ├── server.rs                   # Serveur WebSocket pour le mode live
│           ├── live.rs                     # Mode live (simulation + WebSocket)
│           ├── snapshot.rs                 # Génération de snapshots
│           └── ui/                         # Widgets TUI spécialisés
│               ├── mod.rs
│               ├── alerts.rs              # Alertes et highlights
│               ├── island.rs              # Rendu de l'île
│               ├── diversity.rs           # Panneau diversité
│               ├── cooperation.rs         # Panneau coopération
│               └── fitness.rs             # Panneau fitness
├── web/                                    # Viewer 3D Three.js (DDD JS)
│   ├── index.html
│   ├── style.css
│   ├── js/
│   │   ├── domain/                         # État pur — zéro Three.js/DOM
│   │   │   ├── state.js                    # SimState, reconstruction d'état
│   │   │   ├── state.test.js               # Tests unitaires state
│   │   │   ├── clips.js                    # ClipManager, navigation clips
│   │   │   └── clips.test.js              # Tests unitaires clips
│   │   ├── application/                    # Orchestration
│   │   │   ├── timeline.js                 # Play, pause, scrub, vitesse
│   │   │   └── timeline.test.js           # Tests unitaires timeline
│   │   ├── infra/                          # Three.js (dépendance externe)
│   │   │   ├── terrain.js                  # Mesh terrain voxel + eau
│   │   │   ├── textures.js                # Textures procédurales
│   │   │   ├── plants.js                   # PlantRenderer (tronc, canopée, graines)
│   │   │   ├── plant-archetypes.js        # Archétypes visuels de plantes
│   │   │   ├── particles.js               # Système de particules (spores, pluie)
│   │   │   ├── symbiosis.js                # Interactions (liens, exsudats, flash)
│   │   │   ├── lighting.js                 # Éclairage saisonnier
│   │   │   ├── camera.js                   # Caméra orthographique iso
│   │   │   └── camera-explore.js          # Caméra d'exploration libre
│   │   ├── ui/                             # DOM
│   │   │   ├── panel.js                    # Panneau latéral
│   │   │   └── brain-viz.js               # Visualisation du réseau de neurones
│   │   └── app.js                          # Point d'entrée
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

**Three.js + ES modules + structure DDD**. Zéro build step (importmap CDN).

Le viewer suit la même séparation DDD que le moteur Rust :

| Couche | Dépend de | Contenu |
|---|---|---|
| `domain/` | Rien (état pur) | `state.js` (SimState, reconstruction d'état depuis events + keyframes), `clips.js` (ClipManager, navigation clips) |
| `application/` | `domain/` | `timeline.js` (play, pause, scrub, vitesse) |
| `infra/` | `domain/` + `application/` + Three.js | `terrain.js` (mesh voxel + eau), `plants.js` (tronc, canopée, graines), `symbiosis.js` (liens, exsudats, flash), `lighting.js` (éclairage saisonnier), `camera.js` (ortho iso) |
| `ui/` | Tout | `panel.js` (panneau latéral DOM) |

`app.js` est le point d'entrée : il connecte toutes les couches et lance la boucle rAF.

Mode replay : chargement d'un fichier montage JSON, navigation entre clips. Mode live : connexion WebSocket au moteur Rust, mise à jour en temps réel.
