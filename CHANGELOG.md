# Changelog

All notable changes to MyLAN are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] — 2026-07-03

### Added

- **Agent daemon** (`mylan-agent` crate): scheduler loop con escaneo periódico,
  degradación de privilegios (ARP sweep sudo opcional, fallback
  ICMP/TCP-ping/mDNS/SSDP — nunca crash), graceful shutdown vía
  `CancellationToken` (ctrl_c + SIGTERM), y API REST+WS embebida in-process.
  - `mylan agent start|run|stop` CLI subcommands (daemon con pidfile, foreground
    debug, SIGTERM al pidfile).
  - `mylan-agent.toml` config (`interval_secs`, `networks` [cidr + profile],
    `api_port`, `db_path`).
- **Diff events** (motor de diff entre scans): `device_new`, `device_ip_changed`,
  `device_offline`, `device_online`, `port_opened` (vía service snapshot
  before/after). Cold-start suppression (no event storm tras restart).
  Persistencia atómica con el scan en una transacción (ADR-5).
  - `events` table (migración V4) + `is_online` column en `devices` (backfill
    one-shot derivado del último scan, no blanket `DEFAULT 1`).
  - `Event`/`EventType`/`Severity` domain models en `mylan-core`.
- **API local** (`mylan-api` crate, axum 0.8): REST read-mostly + WS
  `/api/v1/events/live` (timeline en vivo + backfill `?since=<ISO8601>`).
  Token bearer auth (32 bytes `getrandom` + base64 URL-safe, ADR-7), bind
  `127.0.0.1` only (localhost security model).
  - `GET /api/v1/{status,interfaces,networks,devices,devices/{id},events,scans}`,
    `POST /api/v1/scans` (discovery in-process).
- **Packaging** (4 targets): systemd unit (`agent/systemd/`), Dockerfile
  multiarch amd64 + arm64 (`agent/docker/`), Raspberry Pi native guide
  (`agent/raspberry-pi/`), Windows service wrappers (`agent/windows/`). CI smoke
  en `.github/workflows/packaging-smoke.yml` (systemd + docker + rpi cross-build
  + windows).
- **CLI**: `mylan agent` subcommand + `mylan serve` (debug alias de
  `mylan agent run`, foreground agent + API en un proceso).

### Changed

- Workspace version bumped to `0.5.0` (all crates) + `apps/desktop-tauri`
  (`src-tauri/Cargo.toml` + `package.json`).

### Architecture (ADRs)

- **ADR-4**: single-process agent-embeds-API (broadcast channel in-process).
- **ADR-5**: txn-composable pipeline (`run_scan_pipeline_at_in_tx` +
  `run_scan_pipeline_with_diff`).
- **ADR-6**: `port_opened` via service snapshot (no `mylan-scanner` en agent
  deps — port scan stays on-demand).
- **ADR-7**: token via `getrandom` (no `uuid`).

### Notes

- Desktop app (`apps/desktop-tauri`) sin cambios funcionales (AC-14: `lib.rs` y
  `commands.rs` intactos); solo version bump.