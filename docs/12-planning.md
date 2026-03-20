# Planning et Roadmap

## Phases

### Phase 1 — Domain pur (garden-core/domain/)

Le cœur métier sans aucune dépendance externe. Tout se teste unitairement.

| Tâche | Critère de done |
|---|---|
| Value objects : Vitality, Energy, Biomass, Lineage | Types créés, tests unitaires passent |
| Entity Plant : zone de place, stats vitales, traits génétiques | Cycle de vie complet (graine → croissance → maturité → mort) testable |
| Entity Brain : forward pass 18→H→H→8, hidden_size variable | Forward pass correct pour inputs aléatoires, tailles 6 à 14 |
| Entity World : grille 128×128, cellules (altitude, carbone, azote, humidité, lumière, exsudats) | Grille manipulable, accesseurs testés |
| Island : topographie, masque île/mer, bassins versants | Génération d'île valide (via trait Rng injecté) |
| Symbiosis : liens mycorhiziens, échanges C↔N | Création/rupture de lien, échange bidirectionnel testé |
| Domain events : Grew, Died, Born, Invaded, Linked, Flood, LineageFork... | Events émis correctement par les entities |

**Done quand** : `cargo test` passe, zéro dépendance externe dans domain/, couverture des entities et value objects.

### Phase 2 — Application (garden-core/application/)

Les use cases et services. Orchestration du domain.

| Tâche | Critère de done |
|---|---|
| Perception : calcul des 18 inputs (gradients sur zone d'influence) | Gradients corrects pour des configurations de grille connues |
| Simulation : game loop complète (1 tick, 5 phases) | Un tick produit les bons domain events |
| Évolution : banque de graines, fitness, pluie de graines, crossover | La banque se remplit, les meilleurs remplacent les pires |
| Highlights : détection des moments clés (heuristiques) | Les triggers se déclenchent sur des scénarios de test |

**Done quand** : simulation continue fonctionnelle, l'évolution progresse (fitness moyenne monte), tests d'intégration Gherkin passent.

### Phase 3 — Infra + CLI (garden-core/infra/ + garden-cli/)

Sérialisation, persistence, configuration, interface en ligne de commande.

| Tâche | Critère de done |
|---|---|
| Config TOML : tous les paramètres configurables | Chargement + validation config |
| Persistence : sauvegarde/chargement banque de graines + état île | Round-trip save → load → état identique |
| Replay : émission events → clips JSON, montage | Clips lisibles, format JSON valide |
| Implémentation Rng (rand) + Perlin noise | Île générée, simulation reproductible avec seed |
| CLI (clap) : commandes garden run, garden replay, garden live | Commandes fonctionnelles |

**Done quand** : on peut lancer `garden run`, l'entraînement tourne, et on peut sauvegarder/reprendre.

### Phase 4 — TUI Dashboard

Dashboard de monitoring ratatui.

| Tâche | Critère de done |
|---|---|
| Layout 5 panneaux : fitness, diversité, coopération, santé île, alertes | Affichage correct des données en temps réel |
| Courbes : fitness, population, lignées dans le temps | Courbes lisibles, mise à jour fluide |
| Contrôles : pause, reprendre, arrêter, vitesse, sauvegarde | Toutes les hotkeys fonctionnelles |

**Done quand** : on peut surveiller un entraînement complet via la TUI et prendre des décisions informées.

### Phase 5 — Web Viewer 3D

Le "documentaire nature" en Three.js.

| Tâche | Critère de done |
|---|---|
| Terrain voxel : île, altitude, sol coloré, eau (mares + mer) | Île rendue en 3D, mares visibles |
| Plantes voxel : troncs, canopées, graines, dépérissement | Plantes qui poussent et meurent visuellement |
| Interactions : liens mycorhiziens (filaments), exsudats (halos), invasion (flash) | Interactions visibles |
| Éclairage saisonnier : 4 ambiances, ombres | Transitions fluides entre saisons |
| Caméra Dieu : ortho iso, zoom, scroll, sélection plante | Navigation fluide |
| Caméra Exploration : perspective, WASD, souris | Balade sur l'île fonctionnelle |
| Timeline : clips, scrub, navigation entre clips | Lecture du montage complète |
| Panneau latéral : stats, plante sélectionnée, cerveau | Infos visibles et à jour |
| Mode live : WebSocket depuis le moteur | Connexion temps réel fonctionnelle |

**Done quand** : on peut regarder un montage de clips en 3D et se balader sur l'île en live. Tests E2E Playwright passent.

### Phase 6 — Polish et déploiement

| Tâche | Critère de done |
|---|---|
| Calibrage : équilibre des paramètres (coûts, seuils, fitness) | La coopération émerge, la diversité se maintient |
| Performance : parallélisme rayon, greedy meshing | Simulation fluide, viewer à 60fps |
| Déploiement GitHub Pages | Viewer accessible en ligne |
| README + démo | Documentation et vidéo/GIF de démonstration |

## Commandes CLI

| Commande | Description | Exemple |
|---|---|---|
| garden run | Lancer la simulation continue | `garden run --config garden.toml` |
| garden run --resume | Reprendre une simulation sauvegardée | `garden run --resume saves/state_001.json` |
| garden replay | Servir le viewer + un fichier de montage | `garden replay replays/montage_001.json` |
| garden live | Servir le viewer en mode live (WebSocket) | `garden live --port 8080` |
| garden inspect | Détails d'un génome de la banque | `garden inspect saves/state_001.json --genome 12` |
| garden benchmark | Test de performance | `garden benchmark --ticks 10000` |
| garden config init | Générer un fichier config par défaut | `garden config init` |

## Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| La symbiose n'émerge jamais | Élevé | Réduire coût exsudats, augmenter poids fitness coopération. Les deux ressources C/N forcent la complémentarité. |
| L'invasion domine toujours | Moyen | Augmenter coût invasion, renforcer défense. La pénalité monoculture + appauvrissement du sol pénalisent les envahisseurs. |
| Banque de graines stagne (pas de diversité) | Moyen | Augmenter le taux de pluie de graines, élargir la banque, forcer des mutations plus fortes. |
| L'île se vide (toutes les plantes meurent) | Élevé | La pluie de graines continue garantit le repeuplement. Biais de survie dans les cerveaux initiaux. |
| Ruissellement trop coûteux en calcul | Moyen | Pré-calcul statique des bassins versants + ruissellement tous les N ticks seulement. |
| Rendu Three.js trop lent (128×128 voxels) | Moyen | Greedy meshing, LOD, frustum culling. Instanced meshes pour les plantes. |
| Monde voxel pas assez beau | Moyen | Éclairage saisonnier, ombres, eau avec reflets. Le volume compense le détail. |
| Scope creep | Élevé | Pas de son, pas de pollinisation, pas d'herbivores en v1. |

## Évolutions post-v1

- **Toxicité (allélopathie)** : plantes qui empoisonnent le sol. Nouvel output toxic_rate.
- **Pollinisation** : insectes attirés par les fleurs, boostent la reproduction.
- **Herbivores** : agents externes qui mangent les canopées (pression de sélection).
- **NEAT** : topologie dynamique (ajout/suppression de neurones par mutation). Complète le hidden_size mutable actuel.
- **Dispersion active** : output pour contrôler la direction de lancer de graine.
- **Climat évolutif** : sécheresses, inondations, changement climatique progressif.
- **Types de sol** : roche, sable, argile avec des propriétés différentes.
- **Son procédural** : ambiance forêt (vent, pluie, oiseaux) dans le viewer.
- **Commentateur IA** : LLM local qui narre les moments forts du replay.
