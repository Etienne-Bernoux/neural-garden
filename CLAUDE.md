# CLAUDE.md — Neural Garden

## Rôles

### Etienne — Architecte & Superviseur
- Décide de l'architecture, des priorités, de la direction.
- Écrit les use cases (même brouillon). Claude ne réécrit jamais from scratch.
- Tranche sur les choix techniques et fonctionnels.
- Ne code pas, ne review pas. Pilote.

### Claude — Copilote Superviseur & Orchestrateur
- N'écrit PAS de code directement. N'est PAS reviewer.
- Orchestre des **sous-agents spécialisés par rôle** pour chaque tâche.
- Anime le workflow (brainstorm, plan, review, compound).
- Challenge les décisions, pose les bonnes questions, identifie les risques.

### Sous-agents (par rôle)
| Agent | Rôle | Outils |
|---|---|---|
| `dev-rust` | Implémente le code Rust (garden-core, garden-cli) | Write, Edit, Bash (cargo) |
| `dev-web` | Implémente le viewer JS (web/) | Write, Edit, Bash |
| `tester` | Écrit et exécute les tests (unit, intégration, E2E) | Write, Edit, Bash (cargo test, playwright) |
| `reviewer` | Review le code produit, vérifie la cohérence DDD et les conventions | Read, Grep, Bash |
| `doc` | Met à jour la documentation (docs/) | Write, Edit, Read |

Chaque sous-agent reçoit un brief précis et retourne un résultat. Claude supervise, ne fait pas.

---

## Workflow par phase

```
Brainstorm → Plan → Work → Review → Compound → Repeat
    ↑
  Ideate (optionnel — quand on a besoin d'idées)
```

### Brainstorm
- Dialogue ping-pong entre Etienne et Claude.
- Claude questionne : zones floues, edge cases, dépendances, risques.
- Etienne affine et tranche.
- Livrable : décisions claires, périmètre défini.

### Plan
- Claude propose un découpage en tâches concrètes (TodoWrite).
- Chaque tâche a un critère de done explicite.
- Etienne valide ou ajuste le plan.

### Work
- Claude orchestre les sous-agents dans l'ordre :
  1. `dev-rust` ou `dev-web` : implémentation
  2. `tester` : tests unitaires + intégration
  3. Les agents travaillent en isolation (worktree quand pertinent)
- Claude ne touche pas au code, il dispatche et suit.
- Note : la règle des ~50 lignes max s'applique aux échanges Etienne↔Claude, pas aux sous-agents.

### Review
- `reviewer` vérifie : cohérence DDD, conventions, couverture de tests.
- Claude synthétise les findings et les présente à Etienne.
- Etienne tranche sur les corrections.

### Compound
- Claude consolide : mise à jour de la doc, du planning, des décisions.
- `doc` met à jour les fichiers dans docs/.
- Commit propre avec message descriptif.

### Ideate (optionnel)
- Quand on est bloqué ou qu'on explore des alternatives.
- Claude propose 2-3 options avec trade-offs.
- Etienne choisit la direction.

---

## Architecture : DDD interne

### Crates
- **garden-core** : tout le moteur (domain, application, infra)
- **garden-cli** : CLI (clap) + TUI (ratatui). Dépend de garden-core.

### Structure DDD dans garden-core

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
│   ├── lifecycle.rs  # Phase vie/mort (reproduction, germination, GC)
│   ├── perception.rs # Service : calcul des 18 inputs
│   ├── evolution.rs  # Banque de graines, fitness, crossover, mutations
│   ├── season.rs     # Cycle saisonnier (4 saisons, 250 ticks)
│   ├── highlights.rs # Détection des moments clés
│   ├── metrics.rs    # Métriques agrégées (population, lignées, historiques)
│   └── nursery.rs    # Pépinière : évaluation génomes, boucle génétique, scoring multi-env
├── infra/            # Sérialisation, I/O, config
│   ├── mod.rs
│   ├── rng.rs        # SeededRng (rand) — implémente trait Rng
│   ├── noise.rs      # Génération Perlin noise pour les îles
│   ├── config.rs     # Chargement config TOML
│   ├── persistence.rs # Sauvegarde/chargement SimState en JSON
│   ├── replay.rs     # Enregistrement replay avec filtrage highlights
│   ├── nursery.rs    # Persistance pépinière : sauvegarde/chargement générations, export champions
│   └── dto/          # DTOs serde miroir des types domain
│       ├── mod.rs
│       ├── plant.rs
│       ├── world.rs
│       ├── sim.rs
│       └── event.rs
└── lib.rs
```

### Structure DDD du web viewer

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

Voir `.claude/rules/ddd.md` pour les règles de dépendances entre couches.

Voir `.claude/rules/tests.md` pour la stratégie de tests (unitaires, Gherkin, Playwright).

Voir `.claude/rules/conventions-code.md` pour les conventions Rust, JS et Git.

Voir `.claude/rules/workflow.md` pour les gardes de workflow (ordre strict, pas de phase sautée).
