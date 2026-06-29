# MyLAN

**Tu red, bajo control.**

MyLAN es una app open-source, gratuita y *local-first* para descubrir, monitorear y proteger tu red local, sin paywalls y sin nube obligatoria. Es una alternativa libre y extensible a herramientas comerciales de monitoreo de red.

> **Estado:** desarrollo activo. El repositorio ya completó el scaffolding del workspace Rust, el CLI funcional (`mylan`), la base de datos local SQLite, el descubrimiento de red y la **Desktop UI alpha** con Tauri 2. El plan completo está en [`MyLAN_plan_open_source.md`](MyLAN_plan_open_source.md) y el orden de entrega en [`ROADMAP.md`](ROADMAP.md).

## Características actuales

- Descubrir dispositivos conectados a tu LAN (IP, MAC, hostname, vendor, tipo).
- Escanear puertos por perfil (`quick`, `normal`, `deep`, `iot`, `router`).
- Detectar servicios de red y filtrarlos por dispositivo, puerto o protocolo.
- Diagnóstico de red: `ping`, `traceroute` y resolución `dns`.
- Inventario persistente en SQLite local.
- Exportar dispositivos y servicios a **JSON** y **CSV**.
- Funcionar **sin privilegios** por defecto (sudo opcional amplía la cobertura).
- Desktop UI alpha con React + TypeScript + Tauri 2.

## Requisitos

- Rust (toolchain pinneada en [`rust-toolchain.toml`](rust-toolchain.toml), instalada por `rustup`).
- Un compilador de C (`cc`/`gcc`) para `rusqlite` (feature `bundled`).
- Linux (objetivo principal, verificado).
- Windows: soporte de compilación cruzada en CI (`x86_64-pc-windows-gnu`), funcionalidad en validación.
- Para la Desktop UI: Node.js 20+, `npm` y dependencias de sistema Tauri 2 (ver [tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/)).

## Versionado

La versión pública de MyLAN tiene una única fuente de verdad:
`[workspace.package].version` en [`Cargo.toml`](Cargo.toml). Después de cambiarla,
sincroniza los manifests obligatorios de npm/Tauri con:

```bash
npm run version:sync
```

Para validar que no haya drift entre Rust, npm y Tauri:

```bash
npm run version:check
```

## Build

### CLI y workspace Rust

```bash
# Build de todo el workspace (binarios mylan + tests de integración)
cargo build --workspace

# Build optimizado del CLI
cargo build --release -p mylan-cli
```

### Desktop UI

```bash
cd apps/desktop-tauri
npm install

# Modo desarrollo
npm run dev

# Build de producción (frontend + binario Tauri)
npm run build

# Lint del frontend
npm run lint

# Auditoría de accesibilidad (requiere servidor dev o preview)
npm run lint:a11y
```

### Compatibilidad cruzada

```bash
# Windows (requiere target mingw y gcc-mingw-w64-x86-64)
rustup target add x86_64-pc-windows-gnu
cargo check --workspace --target x86_64-pc-windows-gnu
```

## Uso (CLI)

```bash
# Estado general y de la interfaz activa
mylan status

# Escanear la red actual (perfil quick por defecto)
mylan scan
mylan scan --profile normal --interface enp37s0

# Listar inventario de dispositivos
mylan devices

# Detalle de un dispositivo por IP
mylan device 192.168.1.20

# Escanear puertos de un host
mylan ports 192.168.1.1 --top 100
mylan ports 192.168.1.1 --profile iot

# Listar servicios detectados
mylan services
mylan services --device 192.168.1.1 --port 80

# Exportar datos
mylan export devices --format json --output devices.json
mylan export services --format csv --output services.csv

# Diagnóstico de red
mylan ping 1.1.1.1 --count 4
mylan traceroute 1.1.1.1 --max-hops 30
mylan dns example.com --rtype A
```

### Perfiles de escaneo

| Perfil | Uso |
|--------|-----|
| `quick` | Barrido rápido, top 100 puertos |
| `normal` | Cobertura media |
| `deep` | Cobertura extendida |
| `iot` | Selección de puertos comunes en dispositivos IoT |
| `router` | Selección de puertos comunes en routers/cámaras |

## Ética y uso responsable

MyLAN está diseñado **únicamente** para redes propias o con autorización explícita. No incluye funciones ofensivas por defecto: sin deauth Wi‑Fi, sin ARP spoofing, sin MITM, sin interceptación de tráfico. El control de acceso se hace mediante integraciones con router/firewall, no con técnicas agresivas. Ver [`docs/ethics.md`](docs/ethics.md).

## Matriz de compatibilidad (estado actual)

| Función | Linux (sin root) | Linux (sudo) | Windows |
|---|---|---|---|
| Detección de interfaz/gateway/CIDR | Verificado | Verificado | En validación |
| ARP cache (`/proc/net/arp`) | Verificado | Verificado | N/A (otro backend) |
| Barrido TCP-connect | Verificado | Verificado | En validación |
| ARP sweep / ICMP raw | — | Verificado | En validación |
| mDNS / SSDP | Verificado | Verificado | En validación |
| Port scan TCP | Verificado | Verificado | En validación |
| Ping / Traceroute / DNS | Verificado | Verificado | En validación |
| Desktop UI (Tauri) | Verificado | Verificado | En CI |

## Estructura del repositorio

```
crates/           core, db, discovery, fingerprint, scanner
apps/cli          binario `mylan`
apps/desktop-tauri  Desktop UI con Tauri 2 + React + TypeScript
signatures/       OUI + reglas de dispositivos
docs/             arquitectura, ética, diseño del scanner
tests/            integración
.github/workflows/  CI/CD para workspace y desktop
```

## Licencia

[AGPL-3.0-or-later](LICENSE). Las firmas/reglas pueden tener su propia licencia (ver `signatures/oui/README.md`).

## Roadmap

Ver [`ROADMAP.md`](ROADMAP.md) y el [plan completo](MyLAN_plan_open_source.md).
