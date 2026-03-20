# Boucle de Simulation

## Game loop (1 tick)

1. Mettre à jour les conditions saisonnières
2. Régénérer les nutriments du sol (+ décomposition, + appauvrissement monoculture)
3. Diffuser humidité + diffuser/décroître les exsudats
4. Calculer l'ombrage (canopées bloquent la lumière)
5. Pour chaque plante : calculer les 13 inputs (sol local + gradients)
6. Forward pass du réseau de neurones → 8 outputs
7. Résoudre les actions : croissance (+ invasion si cellule occupée), défense
8. Injecter les exsudats dans le sol
9. Absorber nutriments + eau via les racines
10. Photosynthèse (lumière → énergie)
11. Transférer nutriments entre symbiontes (liens directs)
12. Appliquer coûts de maintenance (proportionnels à la biomasse)
13. Vérifier morts, déclencher décomposition
14. Émettre les events pour le replay

## Performance

200 cerveaux × 3 rounds × 12 plantes = ~50 simulations/génération. Chaque simulation : 3000 ticks sur une grille 128×128. Estimation sur Mac M-series : **~45s par génération** (single thread). Avec rayon (parallélisme data) : ~12s. 500 générations ≈ **~1h40**.
