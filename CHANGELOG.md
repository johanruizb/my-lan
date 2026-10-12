# Changelog

All notable changes to MyLAN are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2026-10-12 (873ad28)

### Added
- **Identidad de marca oficial (Discovery Scope)**: identidad visual técnica vectorial táctica con símbolo de retícula sonar/radar, corte óptico para tamaños pequeños (16–32 px), paleta Electric Cyan (`#06B6D4`) sobre fondo Deep Void Navy (`#090D16`), y despliegue del set completo de iconos nativos en Tauri (`.icns` para macOS, `.ico` para Windows, `.png` para Linux).
- **Auto-actualización in-app**: comprobación, descarga e instalación desatendida de nuevas versiones directamente desde la aplicación de escritorio usando GitHub Releases con verificación criptográfica Ed25519 (`tauri-plugin-updater`), interfaz de usuario interactiva en Ajustes y Acerca de, y notificación en la barra lateral.
- **Banner responsivo en documentación**: integración de cabecera con elemento `<picture>` de conmutación automática entre modo claro y modo oscuro en `README.md`.

### Changed
- **Actualización por lotes en SQLite**: optimización en `mylan-db` para consolidar actualizaciones de estado online/offline de dispositivos durante el diff de escaneo en una sola transacción por lotes, reduciendo escrituras y bloqueos en disco.
- **Persistencia asíncrona de ajustes**: migración del guardado de configuración en Tauri a operaciones asíncronas con `tokio::fs` en lugar de bloqueos síncronos de E/S.
- **Mejoras de robustez y resiliencia**: eliminación sistemática de `unwrap` en el parseo de direcciones MAC e IP, verificación segura de truncamiento de ficheros de tokens y cobertura extendida de pruebas unitarias para utilidades de CLI, DB y frontend.
- **Experiencia de usuario en filtros**: mejora en estados vacíos (*EmptyState*) cuando los filtros no arrojan resultados, con acción directa para restaurarlos.

**Full changelog:** https://github.com/johanruizb/my-lan/compare/v0.5.9...v0.6.0

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
