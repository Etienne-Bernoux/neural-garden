## Rust

- `cargo fmt` avant chaque commit.
- `cargo clippy -- -D warnings` : zéro warning.
- Nommage : snake_case partout, types PascalCase.
- Pas de `unwrap()` dans le code de production (sauf `unreachable!`).
- Commentaires en français dans le code, docs en français.
- Noms de tests en français (snake_case) : `#[test] fn la_plante_meurt_quand_vitalite_atteint_zero()`.
- Un fichier = une responsabilité. Si un fichier dépasse ~300 lignes, le découper en modules. Le fichier d'orchestration ne garde que les appels, la logique va dans des modules dédiés.

## JavaScript (web viewer)

- ES modules, pas de bundler.
- Vanilla JS, pas de framework.
- DDD : `web/js/domain/` (état pur, zéro Three.js/DOM), `web/js/application/` (orchestration), `web/js/infra/` (Three.js), `web/js/ui/` (DOM).
- Tests unitaires colocalisés avec le code : `state.test.js` à côté de `state.js`.
- Vitest pour les tests unitaires : `describe`, `it`, `expect`. Lancer avec `pnpm test` depuis `web/`.
- Gestionnaire de paquets : pnpm.
- Tests E2E (Playwright) dans `web/tests/`.
- Un module par responsabilité.

## Git

- Commits atomiques, message en anglais préfixé : `feat:`, `fix:`, `test:`, `docs:`, `refactor:`.
- Branche `main` toujours verte (tests passent).
- Feature branches pour les phases de travail.
