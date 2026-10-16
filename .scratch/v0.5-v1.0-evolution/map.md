## Destination

Especificación técnica integral de arquitectura (v0.5 a v1.0) con contratos de módulos, modelos de datos y ADRs para los pilares: Monitoreo continuo en Desktop, Alertas de escritorio, Suite interactiva de Herramientas de red, Evaluación de riesgos, Speed test y salud de red, Workspaces & Reporting, y Sincronización Serverless con Google Drive Zero-Knowledge.

## Notes

- Dominio: Inspección, diagnóstico y monitoreo de redes locales con Tauri 2 (Desktop), backend en Rust y frontend en React + TypeScript + Tailwind CSS. Persistencia local-first sobre SQLite con WAL.
- Decisiones rectoras confirmadas:
  - Monitoreo continuo exclusivo de Desktop (Tauri background runner con soporte de System Tray y autoarranque configurable en el SO).
  - Alertas exclusivamente locales vía notificaciones nativas del sistema operativo (`tauri-plugin-notification`).
  - Suite de Herramientas de red interactiva integrada (Ping continuo, Traceroute visual con hops, DNS Lookup multidominio, Wake-on-LAN y Port Checker rápido con streaming IPC).
  - Evaluación de riesgos de red pasiva/defensiva basada en reglas sobre datos ya recolectados (sin tráfico agresivo).
  - Salud de red continua (latencia/jitter/pérdida hacia gateway y DNS) combinada con Speed test de ancho de banda bajo demanda con motor primario Cloudflare (`speed.cloudflare.com`) y fallback NDT7.
  - Workspaces como particiones lógicas aisladas en la misma base de datos; reportes en PDF y JSON con Modo Censura estricto.
  - Sincronización serverless con Google Drive utilizando el ámbito privado `drive.appdata`, OAuth 2.0 PKCE y cifrado Zero-Knowledge local en el cliente con contraseña maestra.
- Lenguaje ubícuo: consultado en `CONTEXT.md` y `GLOSSARY.md`.
- Skills relevantes: `domain-modeling`, `rust-best-practices`, `research`.

## Decisions so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [01-research-google-drive-serverless-sync.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/01-research-google-drive-serverless-sync.md): Loopback HTTP efímero en 127.0.0.1:0 para OAuth PKCE, Google Drive appDataFolder privado y cifrado Zero-Knowledge con Argon2id + XChaCha20-Poly1305.
- [02-architecture-desktop-background-monitoring.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/02-architecture-desktop-background-monitoring.md): Interceptación de cierre a System Tray, autoarranque en OS, Tokio runner cada 5m (perfil quick), período de gracia de 20s en wake e histéresis de 3 fallos para offline.
- [03-desktop-notifications-contract.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/03-desktop-notifications-contract.md): Notificaciones nativas OS con tauri-plugin-notification, agrupación inteligente (resumen si >=3 eventos), respeto estricto del Modo Censura y deep-linking a DeviceDetail/Scans.
- [04-risk-evaluation-rules-engine.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/04-risk-evaluation-rules-engine.md): Taxonomía de 4 niveles, Score de seguridad 0-100 ponderado, ciclo de vida en tabla security_findings con estados (open/acknowledged/ignored/resolved) y respeto de Modo Censura.
- [05-speedtest-and-network-health-design.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/05-speedtest-and-network-health-design.md): Salud local unprivileged (ICMP DGRAM en Linux, IcmpSendEcho en Windows, UDP 53 DNS), Jitter RFC 3550, Speed test Anycast HTTP con Cloudflare (speed.cloudflare.com) y fallback NDT7, con retención de 30 días en SQLite.
- [06-workspaces-schema-and-reporting.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/06-workspaces-schema-and-reporting.md): Partición por workspaces en SQLite con migración brownfield automática, selector en UI, y reportes ejecutivos en PDF generados en cliente (@react-pdf/renderer) con Modo Censura estricto.
- [07-sync-data-contract-and-conflict-handling.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/07-sync-data-contract-and-conflict-handling.md): Paquetes cifrados por workspace (mylan_workspace_{id}.enc), resolución de conflictos con backup preventivo local y modal, recuperación mnemónica de 12 palabras (BIP-39) y persistencia de clave en Keyring OS.
- [08-network-tools-suite-and-ui.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/issues/08-network-tools-suite-and-ui.md): Suite de utilidades en pantalla Tools.tsx (Ping, Traceroute, DNS, WoL, Port Checker, Speed Test) con streaming IPC en tiempo real, cancelación CancellationToken, accesos directos desde DeviceDetail y Modo Censura.

**DESTINO ALCANZADO:** La ruta está completamente despejada y todas las decisiones han sido resueltas en la especificación técnica integral: [.scratch/v0.5-v1.0-evolution/spec.md](file:///home/srnoob/.antigravity-profiles/srnoob2570/Documentos/my-lan/.scratch/v0.5-v1.0-evolution/spec.md).

## Not yet specified

<!-- Fog of war: temas dentro del alcance que se afinarán una vez resueltos los tickets base -->

- Algoritmo de backoff dinámico para el sondeo de salud de red cuando se detecta alta saturación o juego online (se abordará durante la implementación de Fase 9).

## Out of scope

<!-- Trabajo explícitamente descartado de este esfuerzo -->

- Daemon independiente headless 24/7 (`mylan-agent` como servicio systemd/Docker independiente) — Se priorizó Desktop-exclusive con system tray.
- Servidor central de sincronización (`mylan-server` / backend en la nube alojado) — Se optó por sincronización serverless Zero-Knowledge con Google Drive.
- Webhooks externos, bots de Telegram, Discord y pasarelas de email SMTP — Se limitó el alcance a notificaciones nativas del sistema operativo.
- Técnicas ofensivas de auditoría (fuerza bruta, escaneos intrusivos, explotación de vulnerabilidades).
