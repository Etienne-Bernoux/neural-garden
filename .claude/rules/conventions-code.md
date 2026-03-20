## Rust

- `cargo fmt` avant chaque commit.
- `cargo clippy -- -D warnings` : zéro warning.
- Nommage : snake_case partout, types PascalCase.
- Pas de `unwrap()` dans le code de production (sauf `unreachable!`).
- Commentaires en anglais dans le code, docs en français.

## JavaScript (web viewer)

- ES modules, pas de bundler.
- Vanilla JS, pas de framework.
- Fichiers dans `web/js/`, un module par responsabilité.

## Git

- Commits atomiques, message en anglais préfixé : `feat:`, `fix:`, `test:`, `docs:`, `refactor:`.
- Branche `main` toujours verte (tests passent).
- Feature branches pour les phases de travail.
