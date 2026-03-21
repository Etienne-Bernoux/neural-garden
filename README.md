# Neural Garden

Un simulateur d'écosystème où des plantes, pilotées par de petits réseaux de neurones, apprennent à croître, coopérer et survivre par sélection naturelle.

Les plantes perçoivent des gradients chimiques dans le sol (pas leurs voisines) et développent des stratégies émergentes : symbiose mycorhizienne, invasion territoriale, fixation d'azote.

## Prérequis

- Rust 1.70+ (`rustup`)
- macOS / Linux

## Installation

```bash
git clone git@github.com:Etienne-Bernoux/neural-garden.git
cd neural-garden
cargo build --release
```

## Usage

### Lancer la simulation (TUI)

```bash
cargo run --release -- run
```

Dashboard ratatui avec 5 panneaux : fitness, diversité, coopération, santé de l'île, alertes. Mini-carte Unicode.

Contrôles : `espace` pause, `q` quit, `s` sauvegarde.

### Mode headless (sans TUI)

```bash
cargo run --release -- run --no-tui
```

Logs texte toutes les 100 ticks. Ctrl+C pour arrêter (sauvegarde automatique).

### Reprendre une simulation

```bash
cargo run --release -- run --resume saves/auto_001.json
```

### Générer la config par défaut

```bash
cargo run --release -- config init
```

Crée un fichier `garden.toml` avec les paramètres configurables :

```toml
[simulation]
seed = 42
initial_population = 30
seed_bank_capacity = 50
ticks_per_season = 250
```

## Architecture

```
crates/
├── garden-core/     # Moteur de simulation (DDD : domain/application/infra)
└── garden-cli/      # CLI + TUI ratatui
```

- **Domain** : types purs, zéro dépendance externe
- **Application** : game loop, neuroévolution, perception, saisons
- **Infra** : serde, Perlin noise, config TOML, persistence JSON

130 tests unitaires + 16 scénarios Gherkin (cucumber-rs).

## Comment ça marche

1. Des plantes naissent sur une île générée par Perlin noise
2. Chaque plante a un petit réseau de neurones (18 inputs, 8 outputs) qui décide de ses actions
3. Les plantes perçoivent des gradients chimiques via leurs racines
4. Elles peuvent coopérer (liens mycorhiziens, exsudats) ou envahir leurs voisines
5. À la mort, la fitness est évaluée et le génome entre dans la banque de graines
6. Les nouvelles graines héritent des meilleurs génomes avec mutations
7. La coopération émerge parce qu'elle fonctionne, pas parce qu'elle est codée

## Documentation

Les docs de design sont dans [docs/](docs/README.md).

## Licence

MIT
