# Planning et Roadmap

## Phases

### Phase 1 — Domain pur (garden-core/domain/) ✅ Terminée

> 57 tests unitaires, zéro dépendance externe, clippy strict.

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

### Phase 2 — Application (garden-core/application/) ✅ Terminée

> 101 tests, game loop complet (5 phases), neuroévolution, perception par gradients, saisons.

Les use cases et services. Orchestration du domain.

| Tâche | Critère de done |
|---|---|
| Perception : calcul des 18 inputs (gradients sur zone d'influence) | Gradients corrects pour des configurations de grille connues |
| Simulation : game loop complète (1 tick, 5 phases) | Un tick produit les bons domain events |
| Évolution : banque de graines, fitness, pluie de graines, crossover | La banque se remplit, les meilleurs remplacent les pires |
| Highlights : détection des moments clés (heuristiques) | Les triggers se déclenchent sur des scénarios de test |

**Done quand** : simulation continue fonctionnelle, l'évolution progresse (fitness moyenne monte), tests d'intégration Gherkin passent.

### Phase 3 — Infra + CLI (garden-core/infra/ + garden-cli/) ✅ Terminée

> 130 tests, sérialisation DTO, Perlin noise, config TOML, persistence JSON, replay filtré, CLI fonctionnel (garden run/config init).

Sérialisation, persistence, configuration, interface en ligne de commande.

| Tâche | Critère de done |
|---|---|
| Config TOML : tous les paramètres configurables | Chargement + validation config |
| Persistence : sauvegarde/chargement banque de graines + état île | Round-trip save → load → état identique |
| Replay : émission events → clips JSON, montage | Clips lisibles, format JSON valide |
| Implémentation Rng (rand) + Perlin noise | Île générée, simulation reproductible avec seed |
| CLI (clap) : commandes garden run, garden replay, garden live | Commandes fonctionnelles |

**Done quand** : on peut lancer `garden run`, l'entraînement tourne, et on peut sauvegarder/reprendre.

### Phase 4 — TUI Dashboard ✅ Terminée

> Dashboard ratatui 5 panneaux + mini-carte Unicode. Architecture multi-thread (simulation + UI). Perception parallélisée avec rayon. Mode --no-tui pour les sessions headless. Signal handler Ctrl+C.

Dashboard de monitoring ratatui.

| Tâche | Critère de done |
|---|---|
| Layout 5 panneaux : fitness, diversité, coopération, santé île, alertes | Affichage correct des données en temps réel |
| Courbes : fitness, population, lignées dans le temps | Courbes lisibles, mise à jour fluide |
| Contrôles : pause, reprendre, arrêter, vitesse, sauvegarde | Toutes les hotkeys fonctionnelles |

**Done quand** : on peut surveiller un entraînement complet via la TUI et prendre des décisions informées.

### Phase 5 — Web Viewer 3D ✅ Terminée (V1)

> Viewer Three.js avec terrain voxel, plantes, interactions visuelles, éclairage saisonnier, caméra Dieu iso, timeline replay, mode live WebSocket. Structure DDD JS. 26 tests Vitest.

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

### Phase 6a — Calibrage (itératif) ✅ Itérations 1-4

Boucle : Brainstorm → Plan → Work → Review → Observer → Ajuster → Repeat

Implémenté : banque compartimentée 100 slots, placement intelligent (80% près des plantes), graines fraîches 10%, échange C↔N bidirectionnel, LineageExtinction, chimiotaxie racinaire, racines gratuites, bonus croissance, biais connect_signal. Fitness rééquilibrée (symbiose ×8, lifetime ×0.3).

Résultats : fitness jusqu'à 1941, symbiose sporadique (0-14 liens), pop régulée.

### Phase 6a-R1 — Refonte modèle 3 couches ✅

| Couche | Partage | Rôle |
|---|---|---|
| Footprint (emprise au sol) | Exclusive | Invasion possible, biomass = footprint.len() |
| Canopy (canopée aérienne) | Partagée avec priorité | Photosynthèse, ombre dynamique (max = footprint × 4) |
| Roots (racines sous-sol) | Partagée | Absorption, symbiose, chimiotaxie (max = footprint × 5) |

Graines inertes (0 consommation, invasion gratuite, timeout 360). Saisons allongées (360 ticks/saison). Ombre dynamique par taille d'emprise.

### Phase 6a-R2 — Recalibrage : dépendance vitale C/N ⏸

Modèle de dépendance vitale implémenté :
- **Fixation atmosphérique** : les plantes Nitrogen créent du N à partir de rien (coût 0.5 énergie/tick)
- **Rareté N** : nitrogen_regen_rate quasi nul (0.00005), growth_nitrogen_cost doublé (0.1)
- **Échange énergie** : via symbiose, la grande plante (énergie) nourrit la fixatrice (azote)
- **Fitness par ordres de grandeur** : cn_exchanges ×5000, symbiotic_connections ×500

📊 Résultats : fitness 610 588 (✅ >1000), pop régulée ~620, symbiose sporadique (0-19 liens par vagues, pas encore stable en permanence).

⚠️ À reprendre :
- La symbiose reste sporadique — les liens se forment puis se rompent. Piste : les plantes meurent trop vite ou les racines se décalent.
- Le critère "liens stables > 5 en permanence" n'est pas encore atteint.
- Les defaults DTO divergent des defaults config pour les anciennes sauvegardes (à aligner si besoin).

### Phase 6b — Viewer V2 ✅

Caméra FPS (ZQSD + WASD, pointer lock, saut, taille humaine 1.7m). 4 archétypes procéduraux (herbe/buisson/arbre/conifère). Système de particules décomposition. Brain-viz Canvas 2D. Textures procédurales (terrain/eau/plantes/biomes). Soleil + nuages. Contrôle vitesse. Graines et racines invisibles.

**Done** : on peut se balader entre les plantes et voir leur cerveau.

### Phase 6c — Qualité et performance ✅

7 tests unitaires actions critiques, lib.rs façade (19 ré-exports), benchmark (15K ticks/s @30 plantes, 6K @200), 8 docs de design alignées.

### Phase 6d — Déploiement (reportée)

| Tâche | Critère de done |
|---|---|
| GitHub Pages : viewer déployé statiquement | Accessible en ligne avec un montage de démo |
| README + GIF/vidéo de démonstration | Le projet est présentable |

### Phase 7 — Refonte TUI ✅ Terminée

| Tâche | Critère de done |
|---|---|
| Panneau Fitness : fitness max banque + stats accumulées vivantes (biomasse, connexions, exsudats, état croissance) | On voit la santé des plantes vivantes |
| Panneau Diversité : tableau lignées trié par fitness moyenne, graines exclues | On voit quelles lignées dominent |
| Panneau Banque de graines (nouveau) : compartiments actifs, top 5 génomes, spread | On voit le réservoir évolutif |
| Panneau Coopération : liens actifs + ressources transférées (fenêtre 2 ans = 2880 ticks) | On voit si la coopération est durable |
| Panneau Île : ressources moyennes sol (C/N), couverture végétale, cases vides | On voit la santé de l'île |
| Minimap plein écran (toggle M) | Vue d'ensemble de l'île |
| Toggle panneaux (touches 1-5) | Chaque panneau peut être affiché/caché |
| Barre de raccourcis en bas : espace, q, s, 1-5, m | L'utilisateur voit les commandes dispo |

**Done quand** : le TUI raconte l'histoire de la simulation en un coup d'œil, les panneaux sont toggleables, les raccourcis sont visibles.

**Done** ✅ — Dashboard compact + 5 deep dives plein écran (Évolution, Population, Coopération, Île, Logs). Barres d'état et raccourcis permanentes. Toggle 1-5 + Esc. Nouvelles métriques (naissances/morts, âges, C/N, coopérateurs, sol, échanges, banque).

### Phase 9 — Pépinière (pre-training)

Algorithme génétique en bacs isolés. Chaque génome est testé individuellement dans 10 environnements. Les meilleurs sont sélectionnés, mutés, et re-testés. Multi-threadé via rayon.

**Boucle** : 100 génomes → évaluer chacun dans 10 envs → trier → top 10 → 100 graines mutées → itérer.

**Environnements** (cibles théoriques — l'évolution trouvera peut-être mieux) :

| Env | Setup | Fixture | Cible potentielle |
|---|---|---|---|
| Solo riche | C+N abondant | Aucune | Généraliste robuste ("chêne") |
| Carence N | Zéro N | Aucune | Fixatrice pionnière ("trèfle") |
| Carence C | C faible | Aucune | Optimiseur solaire ("fougère") |
| Avec fixatrice | Pauvre N | Fixatrice artificielle | Coopérateur ("bouleau") |
| Avec arbre | Normal | Arbre ombrageur | Plante d'ombre ("mousse") |
| Hiver permanent | Hivernal | Aucune | Résistant froid ("pin") |
| Sécheresse | Humidité basse | Aucune | Xérophyte ("cactus") |
| Compétiteur | Normal | Plante agressive | Défenseur ("ronce") |
| Exsudation | Normal | Partenaire exsudateur | Opportuniste ("lierre") |
| Mixte | Normal | Fixatrice + compétiteur | Adaptable ("hêtre") |

#### Phase 9a-R0 — Refactoring Plant en traits ✅

5 sous-traits (PlantIdentity, PlantVitals, PlantSpatial, PlantGenetics, PlantReproduction) + super-trait PlantEntity. SimState migré vers Vec<Box<dyn PlantEntity>>. FixturePlant (immortelle, déterministe) pour la pépinière.

#### Phase 9a — Infra pépinière

| Tâche | Critère de done |
|---|---|
| Micro-grille isolée (8×8), sol configurable | Bac fonctionnel |
| Plantes artificielles (fixtures) déterministes | Fixtures placables dans un bac |
| Fonction d'évaluation mono-génome (place, tourne, score) | Évaluer un génome dans un bac |

#### Phase 9b — 10 environnements + scoring

| Tâche | Critère de done |
|---|---|
| 10 configs d'environnement avec fixtures | Chaque env sélectionne une stratégie |
| Score multi-env pondéré | Score global pour un génome |

**Done** ✅ — 10 environnements avec régénération sol variable. Scoring multi-env. 6 tests nursery.

#### Phase 9c — Boucle génétique + multi-thread ✅

| Tâche | Critère de done |
|---|---|
| Boucle évaluer → trier → top 10 → muter × 10 → répéter | La boucle tourne |
| Parallélisation rayon (génome × env indépendants) | Multi-cœurs utilisés |
| Sauvegarde auto : `nursery/gen_042_best.json` à chaque génération | Persistance + reprise possible |
| `garden nursery --resume nursery/` | Reprise d'entraînement |

**Done** ✅ — `run_nursery_env` (boucle génétique mono-env), `run_nursery_all` (parallèle rayon sur tous les envs), `save_generation` / `export_champions` / `load_champions` (persistance + roundtrip). 161 tests, 0 clippy warning.

#### Phase 9d — CLI headless + commit (TUI reportée)

| Tâche | Critère de done | Statut |
|---|---|---|
| `garden nursery --no-tui` — logs texte (gen, best, avg, worst, temps) + mode verbose | Mode agent/scripting | ✅ |
| `garden nursery commit --output seeds/v1.json` — figer les meilleurs | Banque versionnable dans git | ✅ |
| `garden nursery --bank seeds/v1.json` — reprise d'entraînement avec banque pré-entraînée | Intégration complète | ✅ |
| `garden nursery` — TUI ratatui (gen, scores, distribution, contrôles) | Dashboard interactif | ⏳ reportée |

**Done partiel** ✅ — Mode headless (`--no-tui` normal + verbose), commit (`nursery commit --output`), et reprise (`nursery --bank`) fonctionnels. TUI ratatui reportée.

### Phase 10 — Stades de croissance (8 stades)

Refonte du cycle de vie : 8 stades avec avantages/inconvénients distincts. Le max_size du génome détermine le stade plafond. La maturité (reproduction) commence au stade Arbuste.

| Stade | Biomasse | Avantage | Inconvénient |
|---|---|---|---|
| Germe | 1 | Invisible, 0 maintenance | Fragile, 0 photosynthèse |
| Pousse | 2-3 | Croissance rapide, faible coût | Pas de reproduction |
| Plantule | 4-6 | Début racines, résistant | Encore petit |
| Arbuste | 7-10 | Première reproduction, couvert | Maintenance croissante |
| Jeune arbre | 11-15 | Canopée, ombre | Coût énergie élevé |
| Arbre | 16-22 | Photosynthèse forte, graines | Gros drain N |
| Arbre mature | 23-30 | Peak reproduction, réseau racinaire | Vieillissement accéléré |
| Vénérable | 31+ | Résilient, enrichit le sol | Croissance arrêtée, sénescence |

Chaque stade influence : taux photosynthèse, coût maintenance, capacité reproduction, résistance, rendu visuel (archétypes).
À brainstormer en profondeur avant implémentation.

## Commandes CLI

| Commande | Description | Exemple |
|---|---|---|
| garden run | Lancer la simulation continue | `garden run --config garden.toml` |
| garden run --resume | Reprendre une simulation sauvegardée | `garden run --resume saves/state_001.json` |
| garden nursery --no-tui | Pépinière en mode headless (logs texte) | `garden nursery --no-tui --verbose` |
| garden nursery commit | Exporter les meilleurs génomes | `garden nursery commit --output seeds/v1.json` |
| garden nursery --bank | Reprendre un entraînement avec banque existante | `garden nursery --bank seeds/v1.json` |
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
