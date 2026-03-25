---
date: 2026-03-24
topic: growth-stages
---

# Phase 10 — Growth Stages (Stades de croissance)

## Problem Frame

Les plantes dans Neural Garden ont un cycle de vie plat : elles poussent, se reproduisent, meurent. Il n'y a pas de diversité morphologique — un trèfle et un chêne sont le même type d'entité avec des paramètres différents.

L'écosystème a besoin de **niches écologiques distinctes** : herbes/trèfles qui colonisent vite, buissons qui stabilisent, arbres qui dominent en canopée, vénérables qui structurent l'écosystème. Chaque "espèce" doit émerger du génome, pas être codée en dur.

## Requirements

### Stades de croissance

- R1. Chaque plante passe par **9 stades** séquentiels : Graine → Germe → Pousse → Plantule → Arbuste → Jeune arbre → Arbre → Arbre mature → Vénérable.
- R2. Le gène `max_size` du génome détermine le **stade maximum** atteignable. Pas d'espèces prédéfinies — les "types" émergent de l'évolution.
- R3. Les seuils de biomasse pour chaque stade sont fixes et universels :

| Stade | Biomasse | Niveau max racine/canopée |
|-------|----------|--------------------------|
| Graine | 0 | — |
| Germe | 1 | Niv 0 (1 racine + 1 tronc) |
| Pousse | 2-3 | Niv 1 |
| Plantule | 4-6 | Niv 1 |
| Arbuste | 7-10 | Niv 2 |
| Jeune arbre | 11-15 | Niv 3 |
| Arbre | 16-22 | Niv 4 |
| Arbre mature | 23-30 | Niv 5 |
| Vénérable | 31+ | Niv 5 + super bonus |

### Système de niveaux (cases racine/canopée)

- R4. Chaque case racine et canopée a un **niveau individuel** (0 à 5) qui détermine sa capacité (absorption N, photosynthèse).
- R5. Le brain peut choisir d'**upgrader** une case existante (augmenter son niveau) ou d'**étendre** (ajouter une nouvelle case). Coût en énergie dans les deux cas.
- R6. Le niveau max d'une case est plafonné par le stade actuel de la plante.
- R7. Le **tronc** évolue automatiquement au changement de stade (pas de choix du brain).
- R8. L'action "grow" existante encode les deux stratégies (étendre vs upgrader) via l'intensité de l'output. Pas de changement d'architecture du réseau de neurones.

### Format de configuration

- R17. Les définitions de stades (seuils de biomasse, niveaux max, bonus, coûts, multiplicateurs) sont stockées dans un **fichier YAML** chargé au démarrage. Ceci permet de lire, ajuster et comprendre les stades sans toucher au code Rust.

### Bonus et coûts par stade

- R9. Chaque montée de stade apporte un **vrai bonus** (nouvelle capacité ou multiplicateur) ET une **augmentation de la consommation** en ressources. La plante est plus puissante mais plus gourmande.
- R10. Chaque stade est **adapté à un type d'environnement** — les petits stades excellent dans les environnements hostiles/compétitifs, les grands dans les environnements riches/stables.

### Sénescence

- R11. La sénescence fonctionne par **vulnérabilité croissante** : pas de déclin passif de vitalité, mais la résistance aux stress (sécheresse, invasion, hiver) diminue avec le temps passé aux stades avancés (Arbre mature, Vénérable).
- R12. La mort d'un vénérable est un événement majeur — libération massive de nutriments et d'espace.

### Vénérable — Keystone species

- R13. Le Vénérable bénéficie d'un **super bonus double** :
  - **Hub mycorhizien** : multiplicateur de symbiose massif. Le vénérable devient le noeud central du réseau, boostant tous les voisins connectés.
  - **Enrichissement du sol** : les racines génèrent de l'azote dans les cases adjacentes, créant un micro-environnement fertile pour les petites plantes.
- R14. Retirer un vénérable doit avoir un impact visible sur l'écosystème local (effondrement du réseau de symbiose, chute de fertilité).

### Nursery — Environnements par niche

- R15. Ajouter des environnements de nursery qui favorisent spécifiquement chaque taille de plante :
  - Environnements "prairie" : avantage aux petites plantes (herbes, trèfles)
  - Environnements "forêt dense" : avantage aux grandes plantes (arbres, vénérables)
  - Environnements "lisière" : avantage aux tailles moyennes (arbustes, jeunes arbres)
- R16. L'objectif est de forcer la pression évolutive pour que **chaque niche de taille ait ses champions**.

## Success Criteria

- L'évolution produit naturellement des génomes avec des `max_size` variés (pas de convergence vers une seule taille).
- En simulation libre, on observe une succession écologique : pionniers → stabilisateurs → dominants → keystone.
- La mort d'un vénérable a un impact visible et mesurable sur l'écosystème local.
- Visuellement (TUI + viewer 3D), chaque stade est immédiatement distinguable.
- Le brain apprend des stratégies différentes selon le stade (observé via les métriques nursery).

## Scope Boundaries

- **Hors scope** : types de plantes codés en dur (fleur, fruit, etc.) — tout émerge du génome.
- **Hors scope** : interactions inter-espèces spécifiques (pollinisation, parasitisme) — pas dans cette phase.
- **Hors scope** : viewer 3D — cette phase concerne le domain, l'application, la nursery et le TUI. Le viewer 3D sera traité dans une phase ultérieure.
- **Hors scope** : refonte complète du brain/réseau de neurones — on réutilise l'architecture existante.

## Key Decisions

- **Émergence vs. codé en dur** : tout émerge du génome via `max_size`. Pas d'espèces prédéfinies.
- **Niveaux de cases** : upgrade individuel par le brain (coût énergie), pas automatique. Trade-off stratégique étendre vs améliorer.
- **Tronc** : seul élément qui évolue automatiquement au changement de stade.
- **Sénescence** : vulnérabilité croissante, pas de déclin passif. Les grands arbres persistent tant que l'environnement les soutient.
- **Vénérable = keystone** : hub mycorhizien + enrichissement sol. Double bonus qui justifie le coût d'atteindre ce stade.
- **Action grow** : l'output existant encode étendre vs upgrader via l'intensité. Pas de changement d'architecture réseau.

## Grandes étapes (jalons)

- **10a — Domain : stades + niveaux** : enum GrowthStage, système de niveaux sur les cases, transition de stade, seuils de biomasse.
- **10b — Application : mécaniques** : upgrade de cases, coûts/bonus par stade, sénescence, super bonus vénérable, enrichissement sol.
- **10c — Nursery : niches** : nouveaux environnements par taille, adaptation des fixtures, pression évolutive diversifiée.
- **10d — TUI : affichage stades** : visualisation des stades dans les panneaux existants, distinction visuelle.
- **10e — Calibrage** : ajustement des seuils, coûts, bonus pour obtenir la succession écologique et la diversité.

## Outstanding Questions

### Resolve Before Planning

_(aucune — toutes les questions produit sont tranchées)_

### Deferred to Planning

- [Affects R3][Needs research] Les seuils de biomasse par stade sont-ils bien calibrés ? Vérifier avec les paramètres actuels de la simulation.
- [Affects R5][Technical] Comment encoder étendre vs upgrader dans l'intensité d'un seul output ? Seuil, binarisation, ou gradient continu ?
- [Affects R9][Needs research] Quels bonus concrets par stade ? (multiplicateurs exacts, nouvelles capacités)
- [Affects R11][Technical] Comment modéliser la vulnérabilité croissante ? Coefficient sur la résistance, ou probabilité de dégât augmentée ?
- [Affects R13][Technical] Comment le hub mycorhizien du vénérable interagit avec le système de symbiose existant ?
- [Affects R15][Needs research] Quels environnements nursery spécifiques ? Combien ? Comment favoriser chaque niche sans être trop prescriptif ?
- [Affects R4][Technical] Comment le niveau de case interagit avec le modèle 3 couches existant (footprint/canopy/roots) ?

## Next Steps

→ `/ce:plan` for structured implementation planning
