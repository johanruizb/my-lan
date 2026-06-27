# MyLAN

**Tu red, bajo control.**

MyLAN es una app open-source, gratuita y *local-first* para descubrir, monitorear y proteger tu red local, sin paywalls y sin nube obligatoria. Es una alternativa libre y extensible a herramientas comerciales de monitoreo de red.

> **Estado:** desarrollo temprano. Este repositorio está en el **Paso 1** del [plan de implementación](MyLAN_plan_open_source.md): scaffolding del workspace Rust. Los comandos de la CLI existen como esqueleto y aún no realizan trabajo real.

## Características objetivo (MVP v0.1 + v0.2)

- Descubrir los dispositivos conectados a tu LAN (IP, MAC, hostname, vendor, tipo).
- Funcionar **sin privilegios** por defecto (sudo opcional amplía la cobertura).
- Persistir el inventario en SQLite local y exportarlo a JSON/CSV.
- Identificar dispositivos por OUI y reglas de fingerprinting comunitarias.
- Escanear puertos y detectar servicios (perfil rápido).

## Requisitos

- Rust (toolchain pinneada en [`rust-toolchain.toml`](rust-toolchain.toml), instalada por `rustup`).
- Un compilador de C (`cc`/`gcc`) para `rusqlite` (feature `bundled`).
- Linux (objetivo verificado). Windows: planeado, **no probado** todavía.

## Build

```bash
cargo build --workspace
```

## Uso (CLI)

```bash
mylan status                       # estado general
mylan scan [--profile quick] [--interface enp37s0]
mylan devices                      # listar inventario
mylan device 192.168.1.20          # detalle por IP
mylan ports 192.168.1.1 --top 100  # puertos de un host
mylan export devices --format json # exportar (json | csv)
```

## Ética y uso responsable

MyLAN está diseñado **únicamente** para redes propias o con autorización explícita. No incluye funciones ofensivas por defecto: sin deauth Wi‑Fi, sin ARP spoofing, sin MITM, sin interceptación de tráfico. El control de acceso se hace mediante integraciones con router/firewall, no con técnicas agresivas. Ver [`docs/ethics.md`](docs/ethics.md).

## Matriz de compatibilidad (estado actual)

| Función | Linux (sin root) | Linux (sudo) | Windows |
|---|---|---|---|
| Detección de interfaz/gateway/CIDR | Verificado* | Verificado* | Planeado / no probado |
| ARP cache (`/proc/net/arp`) | Verificado* | Verificado* | N/A (otro backend) |
| Barrido TCP-connect | Verificado* | Verificado* | Planeado / no probado |
| ARP sweep / ICMP raw | — | Verificado* | Planeado / no probado |
| mDNS / SSDP | Verificado* | Verificado* | Planeado / no probado |
| Port scan TCP | Verificado* | Verificado* | Planeado / no probado |

\* "Verificado" = objetivo de aceptación de este push; se confirma al completar los pasos correspondientes.

## Estructura del repositorio

```
crates/      mylan-core, mylan-db, mylan-discovery, mylan-fingerprint, mylan-scanner
apps/cli     binario `mylan`
signatures/  OUI + reglas de dispositivos
docs/        arquitectura, ética, diseño del scanner
tests/       integración
```

## Licencia

[AGPL-3.0-or-later](LICENSE). Las firmas/reglas pueden tener su propia licencia (ver `signatures/oui/README.md`).

## Roadmap

Ver [`ROADMAP.md`](ROADMAP.md) y el [plan completo](MyLAN_plan_open_source.md).
