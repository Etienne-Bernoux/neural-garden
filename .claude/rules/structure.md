## Structure des fichiers

### garden-core/src/

```
garden-core/src/
├── domain/           # Cœur métier — zéro dépendance externe
│   ├── mod.rs
│   ├── rng.rs        # Trait Rng pour l'injection de l'aléatoire
│   ├── plant.rs      # BoundedF32, Vitality, Energy, Biomass, Lineage, GeneticTraits, Entity Plant (3 couches : footprint/canopy/roots)
│   ├── brain.rs      # Entity Brain, forward pass 18→H→H→8
│   ├── world.rs      # Entity World, Grid 128×128, Cell
│   ├── island.rs     # Île : topographie, masque terre/mer
│   ├── symbiosis.rs  # Liens mycorhiziens, échanges C↔N
│   ├── events.rs     # Domain events (Grew, Died, Born, Invaded, Linked...)
│   ├── traits.rs     # Sous-traits PlantEntity (Identity, Vitals, Spatial, Genetics, Reproduction)
│   └── fixture.rs    # FixturePlant — plante artificielle déterministe pour la pépinière
├── application/      # Cas d'usage, orchestration
│   ├── mod.rs
│   ├── config.rs     # SimConfig — paramètres de simulation
│   ├── sim.rs        # SimState, run_tick — orchestration du game loop
│   ├── environment.rs # Phase environnement (pluie, ombrage, décomposition)
│   ├── actions.rs    # Phase actions (croissance, invasion, défense, symbiose)
│   ├── absorption.rs # Absorption nutriments — proportionnelle à la biomasse
│   ├── photosynthesis.rs # Photosynthèse batch — atténuation par couche de canopée
│   ├── lifecycle.rs  # Phase vie/mort (reproduction, germination, GC)
│   ├── perception.rs # Service : calcul des 18 inputs
│   ├── evolution.rs  # Banque de graines, fitness, crossover, mutations
│   ├── season.rs     # Cycle saisonnier (4 saisons, 250 ticks)
│   ├── highlights.rs # Détection des moments clés
│   ├── metrics.rs    # Métriques agrégées (population, lignées, historiques)
│   └── nursery.rs    # Pépinière : évaluation génomes, boucle génétique, variation inter-gen
├── infra/            # Sérialisation, I/O, config
│   ├── mod.rs
│   ├── rng.rs        # SeededRng (rand) — implémente trait Rng
│   ├── noise.rs      # Génération Perlin noise pour les îles
│   ├── config.rs     # Chargement config TOML
│   ├── persistence.rs # Sauvegarde/chargement SimState en JSON
│   ├── replay.rs     # Enregistrement replay avec filtrage highlights
│   ├── nursery.rs    # Persistance pépinière : export/chargement seed bank
│   └── dto/          # DTOs serde miroir des types domain
│       ├── mod.rs
│       ├── plant.rs
│       ├── world.rs
│       ├── sim.rs
│       └── event.rs
└── lib.rs
```

### garden-cli/src/

```
garden-cli/src/
├── main.rs              # Point d'entrée CLI (clap)
├── runner.rs            # Thread simulation + contrôles (pause/quit)
├── snapshot.rs          # Snapshot état simulation pour le TUI
├── tui.rs               # Boucle TUI ratatui (simulation + nursery)
├── nursery_runner.rs    # Thread nursery + NurseryControls (pause/quit)
├── nursery_snapshot.rs  # Snapshot état nursery pour le TUI
├── live.rs              # Mode live (WebSocket)
├── server.rs            # Serveur WebSocket
└── ui/                  # Widgets TUI
    ├── mod.rs
    ├── population.rs    # Panneau population
    ├── evolution.rs     # Panneau évolution / fitness
    ├── cooperation.rs   # Panneau coopération / symbiose
    ├── island.rs        # Panneau santé de l'île
    ├── logs.rs          # Panneau logs / alertes
    ├── nursery_recap.rs # Vue récap nursery (tableau envs + champion)
    └── nursery_zoom.rs  # Vue zoom nursery (historique + détail + config env)
```

### web/js/

```
web/js/
├── domain/          # État pur — zéro Three.js/DOM
│   ├── state.js     # SimState, reconstruction d'état
│   └── clips.js     # ClipManager, navigation clips
├── application/     # Orchestration
│   └── timeline.js  # Play, pause, scrub, vitesse
├── infra/           # Three.js (dépendance externe)
│   ├── terrain.js   # Mesh terrain voxel + eau
│   ├── plants.js    # PlantRenderer (tronc, canopée, graines)
│   ├── symbiosis.js # Interactions (liens, exsudats, flash)
│   ├── lighting.js  # Éclairage saisonnier
│   └── camera.js    # Caméra orthographique iso
├── ui/              # DOM
│   └── panel.js     # Panneau latéral
└── app.js           # Point d'entrée
```
