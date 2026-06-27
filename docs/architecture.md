# Arquitectura de MyLAN

MyLAN es un sistema modular: un *core* de dominio puro reutilizable, crates de
librería para cada capacidad, y una app CLI que las orquesta. (Diagrama y detalle
completos en el [plan](../MyLAN_plan_open_source.md) §5.)

```text
        ┌──────────────────────────────────────────┐
        │                 apps/cli (mylan)          │  ← orquesta el pipeline
        └───────────────────┬──────────────────────┘
                            │
   ┌─────────────┬──────────┼───────────┬──────────────┐
   ▼             ▼          ▼           ▼              ▼
mylan-       mylan-      mylan-      mylan-          mylan-
discovery    scanner     fingerprint   db            core
(liveness +  (puertos +  (OUI/reglas/  (SQLite +     (modelos,
 mDNS/SSDP)  servicios)  tipo)         migraciones)  Observation,
                                                     identidad/merge)
```

## Principios

- **P3 — Dominio puro:** `mylan-core` no hace I/O de plataforma. No se añaden traits de
  estrategia sin un segundo consumidor real (los consumidores futuros —agent, desktop,
  móvil— son non-goals de este push).
- **Pipeline de dos fases:** `liveness` (descubrir hosts vivos → `Observation`s) →
  `enrichment` (vendor/tipo/hostname vía `mylan-fingerprint`) → `persist` (upsert en DB).
  La fase de enrichment se inyecta como **función concreta** definida en `mylan-core`,
  de modo que añadir fingerprint es aditivo, no una reescritura del pipeline.
- **Sin privilegios por defecto:** el flujo base no requiere root; `sudo` solo amplía la
  cobertura (ARP sweep + ICMP raw) con degradación elegante.
- **Aislamiento de plataforma:** lo específico de SO va tras `#[cfg(...)]` con impls
  portables por defecto (sin `todo!()`), para que el build se mantenga verde en cualquier
  target mientras Windows queda documentado-no-compilado.

## Crates

| Crate | Responsabilidad |
|---|---|
| `mylan-core` | Modelos de dominio, `Observation`, identidad/merge de dispositivos, firma de enrichment. |
| `mylan-db` | SQLite (`rusqlite` bundled), migraciones (`PRAGMA user_version`), repos. |
| `mylan-discovery` | Interfaz/gateway/CIDR, `/proc/net/arp`, TCP-ping, mDNS, SSDP, ICMP (+sudo: ARP/ICMP raw). |
| `mylan-fingerprint` | OUI, reverse-DNS, reglas YAML → `device_type` + `confidence`. |
| `mylan-scanner` | Port scan TCP-connect + detección de servicios. |
| `apps/cli` | Binario `mylan`; orquesta scan/devices/ports/export. |
