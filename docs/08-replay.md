# Replay : Event Sourcing

## Principe

Le replay ne stocke pas la grille à chaque tick. Il stocke l'**état initial** + la **liste ordonnée d'événements**. Le viewer JavaScript reconstruit l'état en rejouant les events. Des **keyframes** (snapshots) toutes les 200 ticks permettent le scrub rapide.

## Format JSON

```json
{
  "header": {
    "grid_size": 128,
    "initial_nutrients": [[...], ...],
    "water_points": [{"x":42,"y":67,"r":5}],
    "rocks": [{"x":10,"y":20,"r":2}],
    "seeds": [{"id":0,"species":3,"x":45,"y":72,"brain":[0.12,-0.34,...]}]
  },
  "events": [
    {"t":1,"e":"grow","p":0,"x":46,"y":72},
    {"t":12,"e":"born","p":1,"sp":3,"x":50,"y":68,"parent":0},
    {"t":45,"e":"invade","p":0,"x":60,"y":70,"victim":2},
    {"t":80,"e":"link","a":0,"b":3},
    {"t":200,"e":"exudate","p":0,"rate":0.7,"cells":[[44,71],[45,71]]},
    {"t":500,"e":"season","name":"summer"},
    {"t":1200,"e":"died","p":2},
    {"t":1201,"e":"decompose","cells":[[60,70],[61,70]]}
  ],
  "keyframes": {
    "200": {"owners": [[...]], "nutrients": [[...]]},
    "400": {...}
  }
}
```

## Types d'events

| Event | Champs | Fréquence estimée |
|---|---|---|
| grow | plant_id, x, y | ~10/tick (haute) |
| shrink | plant_id, x, y | ~2/tick |
| born | plant_id, species, x, y, parent_id | ~0.5/tick |
| died | plant_id | ~0.1/tick |
| decompose | cells[] | Suit died |
| invade | plant_id, x, y, victim_id | ~1/tick |
| link | plant_a, plant_b | ~0.2/tick |
| unlink | plant_a, plant_b | ~0.1/tick |
| exudate | plant_id, rate, cells[] | Échantillonné 1/10 ticks |
| season | name | 4 par simulation |

Estimation : ~20 events/tick × 3000 ticks = 60K events × ~30 octets = **~1.8 Mo brut**. Après gzip : **~300-400 Ko**. Parfait pour GitHub Pages.

## Scrub avec keyframes

Pour naviguer au tick T, le viewer : (1) charge le keyframe le plus proche ≤ T, (2) rejoue les events jusqu'à T. Keyframe tous les 200 ticks = max 200 events à rejouer pour n'importe quel scrub. Rapide en JS.
