# Changelog

All notable changes to MyLAN are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Sincronización Serverless Zero-Knowledge con Google Drive**: respaldo y sincronización cifrada de extremo a extremo (Argon2id + XChaCha20-Poly1305) en la carpeta privada `appDataFolder` de Google Drive sin servidor intermedio, con frase de recuperación mnemónica de 12 palabras (BIP-39), resolución de conflictos con respaldo preventivo y gestión de credenciales en el llavero seguro del sistema operativo.
- **Espacios de trabajo multi-sitio y reportes ejecutivos en PDF**: partición lógica de inventarios por ubicación física o cliente en la misma base de datos, con selector rápido en la barra lateral y exportación de informes ejecutivos de red y seguridad en formato PDF y JSON con enmascaramiento estricto de Modo Censura.
- **Test de velocidad Cloudflare y monitor de salud de red**: medición continua de latencia, jitter y pérdida de paquetes hacia gateway y DNS con sondeo ICMP/UDP sin privilegios, combinado con test de ancho de banda y bufferbloat bajo demanda contra `speed.cloudflare.com`.
- **Motor de auditoría de seguridad pasiva y evaluación de riesgos**: evaluación automatizada de riesgos en la red basada en 8 reglas de seguridad (SEC-001..008), asignación de puntuación de seguridad ponderada (0-100) y panel interactivo con estados de hallazgos.
- **Suite interactiva de herramientas de red**: pantalla de utilidades integrada con Ping continuo, Traceroute visual con saltos interactivos, DNS Lookup multidominio, Wake-on-LAN y comprobador rápido de puertos (Port Checker) con streaming en tiempo real.
- **Alertas y notificaciones nativas de escritorio**: notificaciones integradas del sistema operativo para detección de dispositivos desconocidos, alteraciones de IP/MAC en equipos de confianza, caída de gateway o puertos críticos abiertos.
- **Monitoreo continuo 24/7 en segundo plano**: runner periódico configurable en Desktop con minimización a la bandeja del sistema (System Tray) y soporte de inicio automático con el sistema operativo.
- **Componente Alert de shadcn/ui**: integración oficial del componente `Alert` (`Alert`, `AlertTitle`, `AlertDescription`) en la interfaz de escritorio y porteo de avisos de error y advertencia en vistas de red, escaneos, ajustes y acerca de junto con iconos de Lucide.

### Changed
- **Rediseño y layout workbench de Herramientas de red**: interfaz en dos columnas para diagnóstico de red con osciloscopio SVG de latencia RTT en tiempo real, atajos de destino predefinidos, HUD interactivo en espera y cadena visual de saltos en Traceroute.
- **Robustez en migraciones y accesibilidad de componentes**: ejecución segura de migración v8 desactivando temporalmente foreign keys durante la migración y estandarización accesible de botones en diálogos y paneles.
- **Notificaciones Toast al estándar shadcn/ui**: reescritura del componente `Toast` conforme a la especificación oficial de shadcn/ui y sustitución del fondo sólido plano por superficies translúcidas con iconos contextuales de Lucide (`CheckCircle2`, `AlertCircle`, `AlertTriangle`, `Info`).

### Fixed
- **Accesibilidad, jerarquía y flujos de usuario (Auditoría UX)**: estandarización de encabezados `h1` semánticos en vistas principales, breadcrumbs jerárquicos con preservación de filtros en detalle de dispositivo, validación en línea de formularios, confirmaciones modales para acciones destructivas y optimización de objetivos táctiles en navegación móvil.
- **Alineación y continuidad en tarjetas de dispositivos**: estandarización de la altura de tarjetas en cuadrícula (`h-full`), alineación horizontal uniforme de badges e indicador de última vez visto en el pie de tarjeta, y preservación del renglón técnico con indicador para dispositivos sin MAC detectada.

## [0.6.2] - 2026-10-16 (441fd22)

### Added
- **Vista Red unificada y cabecera de telemetría**: consolidación de Dashboard y Dispositivos en una sola vista principal (`Red`) con lecturas técnicas siempre visibles, barra de progreso en vivo y control de lanzamiento de descubrimiento.
- **Variante clara de marca Discovery Scope**: isotipo `final-symbol-light.svg` integrado en `AppMark` con adaptación dinámica entre temas claro (`#F1F5F9` / `#0891B2`) y oscuro (`#090D16` / `#06B6D4`).
- **Acciones rápidas en detalle de dispositivo**: alternancia inmediata de confianza en la cabecera e inicio de escaneo de puertos integrado.

### Changed
- **Ajustes de escaneo y almacenamiento**: selección interactiva con persistencia inmediata del perfil predeterminado y visualización de la ruta de la base de datos SQLite.

**Full changelog:** https://github.com/johanruizb/my-lan/compare/v0.6.1...v0.6.2

## [0.6.1] - 2026-10-12 (fea6613)

### Added
- **Design System y especificación de producto**: formalización del sistema de diseño visual en `DESIGN.md`, `.impeccable/design.json` y del contexto de producto en `PRODUCT.md` bajo la identidad *Discovery Scope*.
- **Acción rápida CopyButton**: componente accesible para copiar IPs, MACs, vendors y banners al portapapeles con feedback visual y soporte completo para navegación por teclado.

### Changed
- **Separación de servicios y puertos en detalle**: la tabla principal de `DeviceDetail` muestra exclusivamente puertos abiertos (`Servicios abiertos`), mientras que los puertos cerrados o filtrados se agrupan en un panel colapsable secundario para evitar falsas alarmas.
- **Ergonomía y densidad en escritorio**: ampliación del contenedor principal a ancho de pantalla completo (`max-w-7xl 2xl:max-w-screen-2xl`) y vista tabular de alta densidad en el inventario de dispositivos.
- **Alineación con Lenguaje Ubícuo**: renombramiento del menú lateral a "Historial", reemplazo de "Fabricante" por "Vendor", unificación a "Online"/"Offline" y uso estricto de "Descubrimiento".
- **Inspección forense de banners**: celda interactiva expandible con preservación de saltos de línea y botón de copiado para firmas de servicios de red.
- **Tematización de superficies del navegador**: estilización de selección de texto (`::selection`), color de cursor (`caret-color`) y barras de desplazamiento sutiles integradas con la paleta visual.

### Fixed
- **Identidad en barra lateral**: sincronización del icono `AppMark` en la barra de navegación lateral con el isotipo oficial *Discovery Scope*.
- **Compatibilidad del actualizador en Linux**: inclusión de paquetes `.AppImage` y `.deb` en la generación de manifiestos `latest.json` para distribución en plataformas Linux.

**Full changelog:** https://github.com/johanruizb/my-lan/compare/v0.6.0...v0.6.1

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
