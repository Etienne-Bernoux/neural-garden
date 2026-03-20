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
│   ├── plant.rs      # Entity Plant, value objects (Vitality, Energy, Biomass)
│   ├── brain.rs      # Entity Brain, forward pass
│   ├── world.rs      # Entity World, Grid, Cell
│   ├── species.rs    # Value object Species
│   ├── symbiosis.rs  # Liens mycorhiziens, exsudats
│   └── events.rs     # Domain events (Grew, Died, Invaded, Linked...)
├── application/      # Cas d'usage, orchestration
│   ├── sim.rs        # Use case : run_tick, run_simulation
│   ├── evolution.rs  # Use case : run_generation, evaluate_fitness
│   └── perception.rs # Service : calcul des gradients
├── infra/            # Sérialisation, I/O, config
│   ├── replay.rs     # Event sourcing → JSON
│   ├── config.rs     # Désérialisation TOML
│   └── persistence.rs # Sauvegarde/chargement populations
└── lib.rs            # Ré-exports publics
```

Voir `.claude/rules/ddd.md` pour les règles de dépendances entre couches.

Voir `.claude/rules/tests.md` pour la stratégie de tests (unitaires, Gherkin, Playwright).

Voir `.claude/rules/conventions-code.md` pour les conventions Rust, JS et Git.
