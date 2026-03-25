---
date: 2026-03-24
topic: doc-update
---

# Mise à jour documentation + diagrammes PlantUML

## Problem Frame

La documentation (12 fichiers dans `docs/`) date du 20-23 mars et a potentiellement divergé du code (absorption proportionnelle à la biomasse, variation inter-génération nursery, etc.). Il manque des schémas visuels pour comprendre rapidement comment les algorithmes clés s'articulent. La doc sert principalement à Etienne pour retrouver le contexte après une pause et onboarder Claude plus efficacement.

## Requirements

- R1. Resynchroniser chaque doc existante avec le code actuel — corriger les descriptions obsolètes, ajouter les mécanismes manquants
- R2. Ajouter des activity diagrams PlantUML **inline** (blocs ```plantuml dans le markdown) pour chaque algorithme clé :
  - R2a. **Game loop** (tick complet : environnement → perception → actions → vie/mort) → `07-simulation.md`
  - R2b. **Flux de ressources** (circulation C, N, eau, énergie entre sol, plantes, exsudats, mycorhizes) → `05-interactions.md`
  - R2c. **Cycle évolutif** (naissance → vie → mort → fitness → banque de graines → crossover → germination) → `06-evolution.md`
  - R2d. **Photosynthèse** (calcul batch, atténuation par couche de canopée) → `07-simulation.md`
  - R2e. **Forward pass du brain** (18 inputs → hidden layers → 8 outputs, activation, décision) → `04-brain.md`
  - R2f. **Flow nursery** (boucle génétique, évaluation génomes, variation inter-gen) → `06-evolution.md` ou section dédiée
  - R2g. **Absorption** (proportionnelle à la biomasse, compétition passive) → `05-interactions.md`
- R3. Les diagrammes doivent être extraits du code actuel (pas de la doc existante) pour refléter le comportement réel
- R4. Chaque diagramme doit être placé dans la doc la plus pertinente, juste avant ou après la section qu'il illustre

## Success Criteria

- Chaque doc reflète fidèlement le code actuel (zéro description orpheline)
- Les 7+ diagrammes PlantUML sont présents inline et rendent correctement
- En relisant une doc, Etienne retrouve le contexte du module en < 2 minutes

## Scope Boundaries

- Pas de refonte de la structure des docs (les 12 fichiers restent)
- Pas de diagrammes de composants ou de séquence — uniquement des activity diagrams
- Pas de génération d'images (les .puml restent en texte inline)
- Pas de doc de la couche web viewer (hors périmètre sauf si explicitement demandé)
- Le README.md (`docs/README.md`) est mis à jour si nécessaire mais pas réécrit

## Key Decisions

- **Audience** : Etienne lui-même (retour après pause + onboarding Claude)
- **Format** : Activity diagrams PlantUML, inline dans les .md
- **Source de vérité** : le code Rust actuel, pas les docs existantes
- **Pas de dossier séparé** : les diagrammes vivent dans les docs, pas dans `docs/diagrams/`

## Outstanding Questions

### Deferred to Planning
- [Affects R2f][Needs research] Où vit la doc de la nursery actuellement ? Faut-il une section dans `06-evolution.md` ou un fichier dédié ?
- [Affects R1][Needs research] Quels fichiers de doc sont les plus décalés par rapport au code actuel ? Le plan devra prioriser.
- [Affects R2][Technical] Faut-il d'autres diagrammes au-delà des 7 identifiés ? L'analyse du code pendant le planning pourrait en révéler.

## Next Steps

→ `/ce:plan` for structured implementation planning
