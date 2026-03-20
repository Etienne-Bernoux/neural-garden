## Règles DDD

- `domain/` ne dépend de RIEN d'externe (pas de serde, pas de fichiers, pas de rand).
- `application/` dépend de `domain/` uniquement.
- `infra/` dépend de `domain/` et `application/`, implémente les traits définis dans domain.
- Les dépendances externes (serde, rand, toml) vivent dans `infra/` exclusivement.
- `rand` est injecté via un trait `Rng` dans domain, implémenté dans infra.
