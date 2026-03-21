## Tests unitaires (Rust)

- Couvrent le `domain/` en priorité.
- Chaque entity/value object a ses tests dans le même fichier (`#[cfg(test)]`).
- Pas de mock — le domain n'a pas de dépendance.
- Conventions : `#[test] fn la_plante_meurt_quand_vitalite_atteint_zero()`

## Tests d'intégration (Gherkin, cucumber-rs)

- Vrais fichiers `.feature` dans `crates/garden-core/tests/features/`.
- Exécutés via [cucumber-rs](https://github.com/cucumber-rs/cucumber).
- Rédigés en français, proches du métier.
- Les step definitions vivent dans `crates/garden-core/tests/steps/`.
- Les scénarios couvrent les interactions métier : symbiose, invasion, décomposition, saisons.

## Tests unitaires (JavaScript)

- Colocalisés avec le code : `state.test.js` à côté de `state.js`.
- Couvrent le `domain/` JS en priorité (state, clips).
- Vitest pour les tests unitaires JS (`describe`, `it`, `expect`).

## Tests E2E — Playwright + Gherkin (web viewer uniquement)

- Vrais fichiers `.feature` dans `web/tests/features/`.
- Exécutés via Playwright + [playwright-bdd](https://github.com/vitalets/playwright-bdd).
- Rédigés en français, proches du métier.
- Step definitions dans `web/tests/steps/`.
- Stack : Playwright + TypeScript + playwright-bdd.
