# AGENTS.md

## Scope

This directory contains the desktop interface for MyLAN. It is built using Tauri 2 with a React/TS frontend (Vite, Tailwind, and Radix UI) and a Rust backend. The backend manages the SQLite database connection, coordinates background network scans, handles IPC commands, and supports importing database records from the CLI.

## Local Structure

- `src/` — React frontend codebase (screens: Dashboard, Devices, DeviceDetail, Scans, Settings).
- `src/App.tsx` — Main routing and application Shell.
- `src-tauri/` — Tauri 2 backend (independent Cargo workspace).
- `src-tauri/src/main.rs` & `lib.rs` — App builder, database initialization, and IPC handler registration.
- `src-tauri/src/commands.rs` — Implementation of `#[tauri::command]` IPC handlers invoking parent crates.
- `src-tauri/src/dto.rs` — Data Transfer Objects mapping domain entities for IPC transport.
- `src-tauri/src/state.rs` — Tauri application state managing database connection and active scan tracking.

## Local Commands

All commands below should be run from the `apps/desktop-tauri/` directory:

```bash
# Install frontend dependencies
npm ci

# Run Vite dev server for frontend prototyping (http://localhost:1420)
npm run dev

# Run the complete Tauri application (frontend + native window) in dev mode
npm run tauri dev

# Lint the TypeScript frontend
npm run lint

# Build the frontend assets only
npm run build

# Run formatting, clippy, and build checks for the backend Tauri crate
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo build --manifest-path src-tauri/Cargo.toml

# Package the desktop application into installation bundles
npm run tauri build
```

## Local Conventions

- **Tauri Detached Workspace**: `src-tauri` has its own `Cargo.lock` and is detached from the root workspace. Do not expect `cargo build --workspace` from the repository root to compile it.
- **IPC Boundaries**: All communication between React and Rust must use defined Tauri command wrappers in `src-tauri/src/commands.rs` and matching client hooks/invocations.
- **DTOs**: Domain models from `mylan-core` must be mapped to serializable DTOs in `src-tauri/src/dto.rs` before being returned to the frontend.
- **Accessibilities & Auditing**: Maintain a11y standards by running the audit script `node scripts/a11y-audit.mjs` as required.

## Testing & Quality

- Perform static analysis of backend using the clippy manifest-path command.
- Frontend styles are configured with Tailwind CSS; ensure UI elements are keyboard-accessible (governed by Radix/shadcn-ui).

## Do Not

- Do not share database connection files directly between the CLI and Desktop while both are running to avoid SQLite lock contention. Use the built-in brownfield import logic instead.
- Do not add standard HTTP routes; the desktop app relies solely on local Tauri IPC commands.
