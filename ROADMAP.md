# ROADMAP de MyLAN

El plan completo (visión, arquitectura, fases) está en [`MyLAN_plan_open_source.md`](MyLAN_plan_open_source.md). Resumen del orden de entrega:

| Versión | Nombre | Objetivo | Estado |
|---|---|---|---|
| 0.1 | Discovery | CLI + inventario básico | ✅ Entregado |
| 0.2 | Identity | Fingerprinting v1 (OUI + reglas) | ✅ Entregado |
| 0.3 | Inspect | Port scan + diagnóstico de red | ✅ Entregado |
| 0.4 | Desktop | UI alpha (Tauri 2 + React) | ✅ Entregado |
| 0.5 | Watch | Agente 24/7 + monitoreo | Planeado |
| 0.6 | Alerts | Notificaciones locales y webhook | Planeado |
| 0.7 | Mobile | Companion Android (Tauri Mobile/Flutter) | Planeado |
| 0.8 | Secure | Seguridad v1 (riesgos, alertas de intrusos) | Planeado |
| 0.9 | Control | Plugins router/firewall | Planeado |
| 1.0 | Home | Release estable para hogar | Planeado |

## Push actual: v0.4 Desktop Alpha

Componentes entregados en este ciclo:

- [x] **Paso 0** — Toolchain Rust (rustup, pin 1.96.0).
- [x] **Paso 1** — Scaffolding del workspace (`crates/`, `apps/cli`, `tests/`).
- [x] **Paso 2** — `mylan-core`: modelos + `Observation` + identidad/merge.
- [x] **Paso 3** — `mylan-db`: SQLite + migraciones + WAL + importación brownfield.
- [x] **Paso 4** — `mylan-discovery`: interfaz/ARP/TCP-ping/mDNS/SSDP + diagnóstico ICMP.
- [x] **Paso 5** — `apps/cli`: pipeline de dos fases (`scan`, `devices`, `export`, `ports`, `services`, `ping`, `traceroute`, `dns`, `status`).
- [x] **Paso 6** — `mylan-fingerprint`: OUI + reglas + tipo/confidence.
- [x] **Paso 7** — `mylan-scanner`: port scan quick + servicios.
- [x] **Paso 8** — `apps/desktop-tauri`: Desktop alpha con IPC directo, eventos en tiempo real, inventario, escaneo, detalle de dispositivo, historial de escaneos y modo oscuro.

Criterios de aceptación detallados (AC-1..AC-16) en el plan de implementación.

## Siguientes pasos (v0.5 Watch)

- [ ] Diseñar el agente 24/7 (headless o daemon).
- [ ] Escaneos periódicos programados desde la base de datos.
- [ ] Detección de cambios en el inventario (nuevo/dispositivo ausente).
- [ ] API local (`mylan serve`) como backend para el agente y UI.
- [ ] Packaging del agente (systemd, Docker, Windows service).
