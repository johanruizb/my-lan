# Release Conventions

## Why
- `versionFile` is `Cargo.toml` because `[workspace.package].version` is the single source of truth for the entire workspace. Synchronization with `package.json`, `apps/desktop-tauri/package.json`, and `tauri.conf.json` is handled by `npm run version:sync`.
- `branchPrefix` is `release/` to follow standard Git Flow release branch separation.
- `changelogLinks` is `inline` with compare URLs placed at the end of each version block.

## Gotchas
- Always run `npm run version:sync` after editing `Cargo.toml`, and verify with `npm run version:check`.
- The CI release workflow (`release.yml`) builds Linux and Windows binaries on `workflow_dispatch` with input `version` (e.g. `0.6.0`), and verifies that the tag does NOT yet exist on origin before building assets and creating the GitHub Release. Push `main` and `release/v<version>` first, trigger the workflow, and let it create/tag the release.
- Desktop tests require passing `npm test` and `npm run build` prior to cutting the release.

## Preferences
- User approved minor bump to `0.6.0` for Discovery Scope branding and auto-updater features.
- Ship gate: prompt Go gate summary before push and triggering the GitHub Actions release workflow.
