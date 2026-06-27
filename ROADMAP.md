# ROADMAP de MyLAN

El plan completo (visión, arquitectura, fases) está en [`MyLAN_plan_open_source.md`](MyLAN_plan_open_source.md). Resumen del orden de entrega:

| Versión | Nombre | Objetivo |
|---|---|---|
| 0.1 | Discovery | CLI + inventario básico |
| 0.2 | Identity | fingerprinting v1 |
| 0.3 | Inspect | port scan + diagnóstico |
| 0.4 | Desktop | UI alpha |
| 0.5 | Watch | Agent + monitoreo |
| 0.6 | Alerts | notificaciones |
| 0.7 | Mobile | Android companion |
| 0.8 | Secure | seguridad v1 |
| 0.9 | Control | plugins router |
| 1.0 | Home | release estable para hogar |

## Push actual: MVP v0.1 + v0.2

Componentes en desarrollo y su paso del plan:

- [x] **Paso 0** — Toolchain Rust (rustup, pin 1.96.0).
- [~] **Paso 1** — Scaffolding del workspace (este commit).
- [ ] **Paso 2** — `mylan-core`: modelos + `Observation` + identidad/merge.
- [ ] **Paso 3** — `mylan-db`: SQLite + migraciones.
- [ ] **Paso 4** — `mylan-discovery`: interfaz/ARP/TCP-ping/mDNS/SSDP + gate AC-7 (≥90%).
- [ ] **Paso 5** — `apps/cli`: pipeline de dos fases (scan/devices/export).
- [ ] **Paso 6** — `mylan-fingerprint`: OUI + reglas + tipo/confidence.
- [ ] **Paso 7** — `mylan-scanner`: port scan quick + servicios.

Criterios de aceptación detallados (AC-1..AC-13) en el plan de implementación.
