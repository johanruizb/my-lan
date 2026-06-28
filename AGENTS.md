# AGENTS.md

## Project Overview

MyLAN is a local-first, open-source network discovery, diagnostics, and security utility written in Rust. It aims to discover devices connected to a local area network (LAN) by IP, MAC, hostname, vendor, and type, persisting findings in SQLite and exposing them via CLI subcommands.

## Repository Structure

- `apps/cli/` — CLI application (`mylan` binary) containing CLI subcommands and orchestrating the discovery pipeline.
- `apps/desktop-tauri/` — Desktop application (Tauri 2 + React + TS) for cross-platform network management and visualization.
- `crates/mylan-core/` — Domain models (Device, Observation, etc.) and pure domain logic (identities, merges, confidences).
- `crates/mylan-db/` — Local SQLite persistence utilizing `rusqlite` (bundled), versioned migrations, and repository queries.
- `crates/mylan-discovery/` — Discovery network techniques (ARP cache, ICMP, mDNS, SSDP, TCP connect sweeps, and sudo ARP sweeps).
- `crates/mylan-fingerprint/` — Device fingerprinting utilizing OUI database lookup and YAML rule matching.
- `crates/mylan-scanner/` — Port scanning and banner grabbing on discovered active hosts.
- `signatures/` — MAC prefix OUI database (`oui.csv`) and community/device identification rules in YAML format.
- `tests/integration/` — Integration tests validating multi-crate interactions and the scan-to-export pipelines.

## Where To Look

| Task | Location | Notes |
|---|---|---|
| CLI subcommands | [apps/cli/src/commands/](file:///home/jr0237/Documentos/my-lan/apps/cli/src/commands/) | Dispatch logic for scan, status, ports, etc. |
| CLI arguments | [apps/cli/src/cli.rs](file:///home/jr0237/Documentos/my-lan/apps/cli/src/cli.rs) | Subcommands and arguments definition using `clap` |
| Desktop views | [apps/desktop-tauri/src/screens/](file:///home/jr0237/Documentos/my-lan/apps/desktop-tauri/src/screens/) | Dashboard, Devices, Settings layouts (React) |
| Desktop commands | [apps/desktop-tauri/src-tauri/src/commands.rs](file:///home/jr0237/Documentos/my-lan/apps/desktop-tauri/src-tauri/src/commands.rs) | Tauri IPC command handlers |
| Domain models | [crates/mylan-core/src/models.rs](file:///home/jr0237/Documentos/my-lan/crates/mylan-core/src/models.rs) | Device, Network, Scan, and Service schema mappings |
| DB Migrations | [crates/mylan-db/src/migrations.rs](file:///home/jr0237/Documentos/my-lan/crates/mylan-db/src/migrations.rs) | Embedded SQL scripts versioned by PRAGMA user_version |
| DB Connection | [crates/mylan-db/src/connection.rs](file:///home/jr0237/Documentos/my-lan/crates/mylan-db/src/connection.rs) | Connection initialization & foreign key constraint flags |
| Discovery logic | [crates/mylan-discovery/src/](file:///home/jr0237/Documentos/my-lan/crates/mylan-discovery/src/) | Gateway detection, ping probes, mDNS, and SSDP listeners |
| YAML Rules | [crates/mylan-fingerprint/src/rules.rs](file:///home/jr0237/Documentos/my-lan/crates/mylan-fingerprint/src/rules.rs) | Fingerprinting logic mapping patterns to device profiles |
| Port Scanner | [crates/mylan-scanner/src/ports.rs](file:///home/jr0237/Documentos/my-lan/crates/mylan-scanner/src/ports.rs) | Throttled TCP port sweeps and service matching |

## Commands

```bash
# Build the workspace
cargo build --workspace

# Run unit and integration tests
cargo test --workspace

# Run code linter
cargo clippy --all-targets -- -D warnings

# Check code formatting style
cargo fmt --all -- --check

# Run local pre-push safety check for real MAC addresses and secrets
./scripts/pre-push-safety.sh

# Run/Build desktop-tauri application (from apps/desktop-tauri)
npm run tauri dev
npm run tauri build
```

## Architecture Notes

- **P3 — Pure Domain**: `mylan-core` contains domain models and algorithms (e.g. merge/aggregation logic). It must not perform any platform I/O or contain hardware/OS dependencies.
- **Pipeline Architecture**: Discovery is organized as a pipeline: Discover hosts (liveness in `mylan-discovery`) -> Enrich details (fingerprinting in `mylan-fingerprint` injected as a concrete function) -> Persist (upsert in `mylan-db`).
- **Graceful Privilege Degradation**: Core discovery operates standard user-space queries (best-effort ICMP ping, reading `/proc/net/arp`, TCP connect scans). Privileged techniques (raw ICMP, active ARP sweeps) are executed only when run as sudo/root, downgrading gracefully without errors if permissions are missing.
- **Platform Isolation**: Linux is the primary target. Place target-specific implementations behind `#[cfg(target_os = "linux")]` or construct portable fallback modules.

## Coding Conventions

- **Unsafe Code**: Always forbid unsafe code by placing `#![forbid(unsafe_code)]` at the root of all crates.
- **Error Handling**: Use custom errors in library crates (`DbError`, `DiscoveryError`, `FingerprintError`) and `anyhow` for top-level application logic.
- **Database Safety**: Enable SQLite referential integrity (`PRAGMA foreign_keys = ON;`) on every connection.
- **Resource Constraints**: Throttle concurrent network connections using semaphores (`tokio::sync::Semaphore`).

## Testing Guidelines

- Write unit tests in inline modules (`mod tests`) or submodules located adjacent to source files.
- Place integration tests inside the dedicated [tests/integration/](file:///home/jr0237/Documentos/my-lan/tests/integration/) package.
- Ensure all tests that mutate storage or access files use temporary directories (`tempfile`) to remain isolated and idempotent.

## Generated Code and External Assets

- Do not manually edit the OUI MAC prefix lookup file (`signatures/oui/oui.csv`).
- Add device fingerprinting rules as YAML files in the `signatures/device-rules/` directory.

## Agent Workflow

- Read the nearest `AGENTS.md` before editing files in a directory.
- Run targeted checks (e.g. `cargo test -p mylan-core`) before running full suite validation commands.
- Do not refactor public APIs or modify migrations without verifying all tests and dependent call sites.
- Follow existing formatting patterns and project style choices.

## Do Not

- Do not introduce raw unsafe blocks.
- Do not execute active network attacks (ARP spoofing, traffic manipulation, Wi-Fi deauth).
- Do not create or modify `CLAUDE.md`.
- Do not add external crates to the workspace unless required and pinned to a specific version in `Cargo.toml`.
- Do not commit real hardware MAC addresses or secrets to files outside tests/fixtures or `#[cfg(test)]` modules (which bypass `./scripts/pre-push-safety.sh`).
