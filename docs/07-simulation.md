# Boucle de Simulation

## Modèle continu

La simulation tourne en continu. Il n'y a pas de "fin de partie" ni de cycle par vagues. L'île vit, les plantes naissent, poussent, meurent, et la banque de graines alimente en continu la population.

## Game loop (1 tick)

### Phase 1 — Environnement

1. **Saison** : mettre à jour les conditions saisonnières (lumière, pluie, multiplicateur de croissance).
2. **Pluie** : ajouter de l'humidité uniformément sur l'île, modulée par la saison.
3. **Ruissellement** (tous les N ticks, pas chaque tick) : l'eau coule vers les cellules plus basses via les bassins versants pré-calculés. Les creux s'accumulent. Les mares se forment ou s'assèchent.
4. **Évaporation** : l'humidité décroît. Les zones sous couvert végétal (canopée) s'évaporent moins vite.
5. **Régénération sol** : le carbone et l'azote se régénèrent lentement (taux saisonnier).
6. **Diffuser/décroître les exsudats** dans le sol.
7. **Ombrage** : recalculer la lumière par cellule (canopée bloque, pentes et exposition).

### Phase 2 — Perception et décision

8. **Pour chaque plante** : calculer les 18 inputs (4 état interne + 4 sol local + 10 gradients sur la zone d'influence).
9. **Forward pass** du réseau de neurones → 8 outputs.

### Phase 3 — Actions

10. **Croissance** : résoudre `grow_dir`, `grow_intensity`, `canopy_vs_roots`. Ajouter des cellules si énergie + carbone + azote suffisants. Si la cellule cible est dans la zone de place d'une autre plante → tentative d'invasion.
11. **Défense** : appliquer le coût de 3 énergie/tick pour les plantes en mode défensif (`defense > 0.5`).
12. **Exsudats** : injecter carbone ou azote (selon `exudate_type`) dans le sol des cellules racinaires. Les fixatrices d'azote bénéficient en plus d'une fixation atmosphérique automatique.
13. **Absorption** : absorber carbone + azote + humidité via la zone d'influence. Compétition passive dans les zones de chevauchement.
14. **Photosynthèse** : lumière reçue → énergie.
15. **Échanges mycorhiziens** : transférer les surplus de C↔N entre plantes connectées, proportionnellement à `connect_generosity`.
16. **Maintenance** : appliquer les coûts proportionnels à la biomasse.

### Phase 4 — Vie et mort

17. **Reproduction vivante** : les plantes éligibles (énergie > 60, biomasse > 8) lancent une graine. Clone + mutations à la dormance.
18. **Vérifier morts** : vitalité = 0 → mort. Déclencher décomposition (carbone ∝ biomasse, azote ∝ âge). Évaluer la fitness. Si fitness > pire de la banque → remplacer.
19. **Pluie de graines** (tous les M ticks) : la banque de graines injecte une graine (crossover + mutation) à une position aléatoire libre sur l'île.
20. **Dormance** : les graines en attente vérifient les conditions de germination (carbone + azote au-dessus des seuils). Germination ou mort par timeout.

### Phase 5 — Événements

21. **Émettre les events** pour le replay (grow, shrink, born, died, invade, link, etc.).
22. **Incrémenter le compteur de génération** pour chaque graine plantée (étapes 17 et 19).

## Performance

### Estimation (Mac M-series, simulation continue)

| Composante | Coût par tick | Notes |
|---|---|---|
| Environnement (phases 1) | Léger | Ruissellement amorti (tous les N ticks). Ombrage = parcours de grille. |
| Perception (phase 2) | Modéré | 18 inputs × nb plantes vivantes. Le gradient scanne la zone d'influence. |
| Actions (phase 3) | Modéré | Forward pass + résolution des conflits (invasion). |
| Vie/mort (phase 4) | Léger | Itération sur les plantes, banque de graines. |
| Events (phase 5) | Léger | Sérialisation JSON incrémentale. |

Avec ~20-50 plantes vivantes sur une grille 128×128 : **~1ms par tick** estimé. À 30 ticks/seconde = ~30ms/s de simulation. Large marge pour le rendu TUI ou l'accumulation rapide.

### Levier de parallélisme

- **Rayon** : la perception (étape 8) est parallélisable par plante (lecture seule sur la grille).
- Le ruissellement et la diffusion sont des opérations sur la grille entière, parallélisables par lignes/colonnes.
