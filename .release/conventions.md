# Release Conventions

## Why
- `versionFile` is `Cargo.toml` because `[workspace.package].version` is the single source of truth for the entire workspace. Synchronization with `package.json`, `apps/desktop-tauri/package.json`, and `tauri.conf.json` is handled by `npm run version:sync`.
- `branchPrefix` is `release/` to follow standard Git Flow release branch separation.
- `changelogLinks` is `inline` with compare URLs placed at the end of each version block.

## Gotchas
- Always run `node scripts/sync-version.mjs` after editing `Cargo.toml`, and verify with `node scripts/sync-version.mjs --check`.
- The CI release workflow (`release.yml`) builds Linux and Windows binaries on `workflow_dispatch` with input `version` (e.g. `0.6.0`), extracts release notes from `CHANGELOG.md`, verifies that the tag does NOT yet exist on origin before building assets, and publishes the GitHub Release directly as latest (non-draft). Push `main` and `release/v<version>` first, trigger the workflow, and let it create/tag the release.
- Desktop tests require passing `npx vitest run` and `npm run build` in `apps/desktop-tauri` prior to cutting the release.

## Preferences
- User approved minor bump to `0.6.0` for Discovery Scope branding and auto-updater features.
- User approved patch bump to `0.6.1` for desktop layout elevation, Discovery Scope sidebar AppMark icon sync, and Linux updater manifest support.
- User approved patch bump to `0.6.2` for unified Red view, live NetworkHeader telemetry, light Discovery Scope branding symbol variant, and quick device detail actions.
- Ship gate: prompt Go gate summary before push and triggering the GitHub Actions release workflow.
