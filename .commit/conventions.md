# Commit Conventions

## Why
- `CHANGELOG.md` tracks user-facing changes following Keep a Changelog.
- Conventional Commits (`type(scope): summary`) is used across the repository.

## Gotchas
- The workspace includes both Rust crates and the Tauri desktop frontend (`apps/desktop-tauri`).
- The project documentation and code comments are largely in Spanish, while commit messages follow standard Conventional Commits.
- `design/branding/kit/` contains the official brand identity assets and guidelines.
- In pre-commit, config `args` REPLACE the hook manifest's `args` (no concatenation). The doublify `fmt` hook therefore needs `args: [--, --check]`: the leading `--` passes staged files to rustfmt, and `--check` makes it validate read-only. Never let hooks rewrite staged files mid-commit — that triggered `VCS_STATE_INVALID` in Antigravity IDE and froze agent conversations (2026-10-16). To format: `cargo fmt` before `git add`.
- The `Desktop` CI job runs only `npx eslint .` for the frontend, despite its comment saying «eslint + tsc». `npx tsc --noEmit` keeps pre-existing errors (RiskSeverity casing in `executive-report-pdf.tsx`, null/undefined in `export-report-modal.tsx`, `Function` in `speed-test-panel.tsx`); these are not introduced by lint fixes.

## Preferences
- User-facing features (`feat`, `fix`, etc.) extend `CHANGELOG.md` under `## [Unreleased]`.
