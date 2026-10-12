# Commit Conventions

## Why
- `CHANGELOG.md` tracks user-facing changes following Keep a Changelog.
- Conventional Commits (`type(scope): summary`) is used across the repository.

## Gotchas
- The workspace includes both Rust crates and the Tauri desktop frontend (`apps/desktop-tauri`).
- The project documentation and code comments are largely in Spanish, while commit messages follow standard Conventional Commits.
- Unstaged directories like `design/` contain local design explorations and must not be staged automatically.

## Preferences
- User-facing features (`feat`, `fix`, etc.) extend `CHANGELOG.md` under `## [Unreleased]`.
