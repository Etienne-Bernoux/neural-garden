# 🌱 Neural Garden — Design

**Un écosystème qui évolue, apprend et s'adapte**

*Rust · Neuroévolution · Gradients · Event Sourcing · Vanilla JS · GitHub Pages*

## Vision

Neural Garden est un simulateur d'écosystème où des plantes, pilotées par de petits réseaux de neurones entraînés par sélection naturelle, apprennent à croître, envahir, coopérer et survivre dans un jardin sauvage. Les plantes ne « voient » pas leurs voisines — elles perçoivent des gradients chimiques dans le sol, exactement comme dans la nature. La coopération émerge naturellement parce qu'elle fonctionne, pas parce qu'on l'a codée.

## Principes fondateurs

- **From scratch** : aucune lib ML. Réseau de neurones codé à la main en Rust.
- **Local first** : Mac Apple Silicon. Pas d'API, pas de cloud.
- **Émergence > Programmation** : on ne code ni stratégie ni coopération. On crée un environnement où elles émergent.
- **Observable** : TUI temps réel + replay web cinématique.
- **Zéro toolchain web** : Vanilla JS + Canvas 2D. GitHub Pages.

## Décisions clés

- **Perception par gradients** : les plantes perçoivent des gradients chimiques/lumineux (pas d'individus), 13 inputs
- **Double modèle de symbiose** : exsudats racinaires (coopération diffuse, publique) + connexion mycorhizienne directe (privée)
- **Invasion = croissance agressive** : pas de mode spécial, pousser ses racines dans une cellule occupée suffit
- **Réseau 13→10→10→8** : petit, rapide, lisible. ~340 paramètres par cerveau
- **Event sourcing** : le replay stocke les événements, pas la grille. Le viewer JS reconstruit l'état
- **2 crates Rust** : garden-core (tout) + garden-cli (CLI + TUI)
- **Diversité du sol** plutôt que bonus biodiversité artificiel

## Documents

| Doc | Contenu |
|---|---|
| [01-world.md](01-world.md) | Grille, ressources, cycle des ressources, saisons |
| [02-plants.md](02-plants.md) | Anatomie, stats vitales, cycle de vie, reproduction |
| [03-perception.md](03-perception.md) | Gradients chimiques, calcul, justification |
| [04-brain.md](04-brain.md) | Réseau 13→10→10→8, inputs, outputs, forward pass |
| [05-interactions.md](05-interactions.md) | Exsudats, mycorhizes, invasion, équilibre |
| [06-evolution.md](06-evolution.md) | Neuroévolution, fitness, sélection, paramètres |
| [07-simulation.md](07-simulation.md) | Game loop, performance |
| [08-replay.md](08-replay.md) | Event sourcing, format JSON, keyframes |
| [09-architecture.md](09-architecture.md) | Structure projet, crates Rust, dépendances |
| [10-tui.md](10-tui.md) | Interface terminal, layout, hotkeys |
| [11-web-viewer.md](11-web-viewer.md) | Viewer JS, Canvas, modules *(à retravailler)* |
| [12-planning.md](12-planning.md) | Roadmap, risques, évolutions post-v1, CLI |
