# Replay : Event Sourcing

## Principe

La simulation tourne en continu. Le replay n'est pas un enregistrement brut — c'est un **montage de moments clés**, comme un documentaire nature. Le moteur détecte automatiquement les événements intéressants et capture des **clips** courts autour de chaque moment.

## Clips et highlights

### Détection automatique

Le moteur surveille des heuristiques pour identifier les moments forts :

| Trigger | Description |
|---|---|
| Première symbiose | Deux plantes forment leur premier lien mycorhizien |
| Invasion majeure | Une plante prend > 5 cellules d'un coup |
| Extinction de lignée | Le dernier individu d'une lignée meurt |
| Nouvelle lignée | Fork d'une lignée (divergence génétique > seuil) |
| Record de fitness | Un individu bat le record de la banque de graines |
| Explosion démographique | La population dépasse un seuil après un creux |
| Mort de masse | > 30% de la population meurt en < 50 ticks |
| Formation de mare | Un creux se remplit et devient obstacle |
| Changement de saison | Transition entre saisons |

### Format des clips

Chaque clip capture **150-300 ticks** (5-10 secondes à 30 ticks/s) autour de l'événement déclencheur. Le montage total ne dépasse pas **1-2 minutes** (3600 ticks max). Si le budget est plein, les clips les moins intéressants (score bas) sont écartés.

## Lignées et identification des espèces

### Principe

L'espèce est identifiée par **lignée** : un `lineage_id` hérité du parent. La lignée fork quand la distance génétique (divergence des poids par rapport à l'ancêtre fondateur de la lignée) dépasse un seuil configurable.

### Tracking

- Chaque plante porte un `lineage_id`.
- À la naissance, la graine hérite du `lineage_id` du parent.
- À chaque mutation, la distance génétique par rapport à l'ancêtre fondateur est mesurée.
- Si la distance dépasse le seuil → nouvelle lignée, nouveau `lineage_id`.
- Les graines issues de la banque (crossover) démarrent une nouvelle lignée.

### Visuel

- Chaque lignée a une **teinte unique** attribuée à la création.
- La santé se lit par la saturation/luminosité dans la teinte de la lignée.
- Le viewer peut afficher l'arbre de lignées et leur généalogie.

## Mode live

En plus du replay par clips, le viewer peut se brancher sur la simulation en cours pour voir l'état **en temps réel**. On peut explorer l'île, sélectionner une plante, voir ses stats et son cerveau.

## Format JSON — clip

```json
{
  "clip": {
    "trigger": "first_symbiosis",
    "tick_start": 4200,
    "tick_end": 4450,
    "score": 0.85
  },
  "header": {
    "grid_size": 128,
    "altitude": [[...], ...],
    "initial_carbon": [[...], ...],
    "initial_nitrogen": [[...], ...],
    "initial_humidity": [[...], ...],
    "submerged": [[...], ...],
    "plants": [
      {
        "id": 0,
        "lineage_id": 12,
        "cells": [[45,72],[46,72]],
        "vitality": 80,
        "energy": 55,
        "traits": {
          "hidden_size": 10,
          "exudate_type": "nitrogen",
          "carbon_nitrogen_ratio": 0.6,
          "max_size": 25
        }
      }
    ]
  },
  "events": [
    {"t":4201,"e":"grow","p":0,"x":47,"y":72},
    {"t":4210,"e":"born","p":5,"lin":12,"x":50,"y":68,"parent":0},
    {"t":4215,"e":"link","a":0,"b":3},
    {"t":4230,"e":"invade","p":2,"x":60,"y":70,"victim":4},
    {"t":4250,"e":"exudate","p":0,"type":"nitrogen","rate":0.7,"cells":[[44,71],[45,71]]},
    {"t":4300,"e":"season","name":"summer"},
    {"t":4400,"e":"died","p":4,"fitness":12.5},
    {"t":4401,"e":"decompose","p":4,"carbon":18,"nitrogen":8}
  ],
  "keyframes": {
    "4300": {
      "owners": [[...], ...],
      "carbon": [[...], ...],
      "nitrogen": [[...], ...],
      "humidity": [[...], ...],
      "submerged": [[...], ...]
    }
  }
}
```

## Format JSON — montage

Un fichier replay complet est une collection de clips :

```json
{
  "version": 1,
  "metadata": {
    "generation_range": [0, 15000],
    "total_ticks": 450000,
    "island_seed": 42
  },
  "clips": [
    { "clip": {...}, "header": {...}, "events": [...], "keyframes": {...} },
    { "clip": {...}, "header": {...}, "events": [...], "keyframes": {...} }
  ]
}
```

## Types d'events

| Event | Champs | Description |
|---|---|---|
| grow | plant_id, x, y | Ajout d'une cellule à la zone de place |
| shrink | plant_id, x, y | Perte d'une cellule (dépérissement) |
| born | plant_id, lineage_id, x, y, parent_id | Naissance (reproduction vivante ou banque) |
| germinate | plant_id | Fin de dormance, début de croissance |
| died | plant_id, fitness | Mort, fitness finale |
| decompose | plant_id, carbon, nitrogen | Libération C/N dans le sol |
| invade | plant_id, x, y, victim_id | Prise d'une cellule d'un autre |
| link | plant_a, plant_b | Formation de lien mycorhizien |
| unlink | plant_a, plant_b | Rupture de lien |
| exudate | plant_id, type, rate, cells[] | Injection d'exsudats (échantillonné) |
| season | name | Changement de saison |
| flood | cells[] | Cellules devenues submergées (mare) |
| drain | cells[] | Cellules redevenues libres (mare asséchée) |
| lineage_fork | new_lineage_id, parent_lineage_id, plant_id | Nouvelle lignée détectée |

## Scrub avec keyframes

Pour naviguer au tick T dans un clip, le viewer : (1) charge le keyframe le plus proche ≤ T, (2) rejoue les events jusqu'à T. Keyframe tous les 100 ticks dans un clip = max 100 events à rejouer pour n'importe quel scrub.

## Estimation taille

Un clip de 250 ticks × ~20 events/tick = ~5000 events × ~40 octets = ~200 Ko brut. Un montage de 12 clips ≈ 2.4 Mo brut. Après gzip : **~400-600 Ko**. Compatible GitHub Pages.
