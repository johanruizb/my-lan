# MyLAN — Plan de proyecto open-source para un reemplazo gratuito de Fing

**Nombre del proyecto:** MyLAN  
**Tipo:** aplicación open-source, gratuita y local-first para descubrimiento, monitoreo, diagnóstico y control de redes LAN/Wi‑Fi.  
**Fecha del plan:** 2026-06-27  
**Autor inicial:** Johan Ruíz  
**Estado:** borrador técnico para iniciar repositorio, arquitectura y roadmap.

---

## 1. Visión

**MyLAN** será una alternativa libre y sin paywalls a herramientas como Fing, enfocada en dar visibilidad completa de una red doméstica, pequeña oficina o red de laboratorio.

El objetivo no es copiar una marca, interfaz ni implementación propietaria, sino construir una herramienta open-source con funcionalidades equivalentes o superiores:

- Descubrir todos los dispositivos conectados a una red.
- Identificar IP, MAC, fabricante, hostname, tipo de dispositivo y servicios.
- Monitorear cambios 24/7.
- Alertar cuando aparezcan dispositivos nuevos o sospechosos.
- Ejecutar diagnósticos de red.
- Detectar puertos abiertos y riesgos comunes.
- Medir velocidad y salud de conexión.
- Permitir control de acceso mediante integraciones seguras con router/firewall.
- Exportar datos sin límites.
- Funcionar sin nube obligatoria.
- Permitir sincronización self-hosted opcional.
- Mantener todas las funciones gratuitas.

---

## 2. Principios del producto

### 2.1 Sin paywalls

Todo lo esencial debe estar disponible gratis:

- Escaneos ilimitados.
- Dispositivos ilimitados.
- Redes ilimitadas.
- Monitoreo 24/7.
- Alertas.
- Exportación.
- API local.
- Plugins.
- Agent.
- Reportes.

Si en el futuro existe algún servicio hospedado por terceros, debe ser opcional. La versión self-hosted debe tener las mismas funciones.

### 2.2 Local-first

MyLAN debe funcionar completamente en red local:

- Sin cuenta obligatoria.
- Sin telemetría obligatoria.
- Sin dependencia de servidores externos para escanear la red.
- Base de datos local.
- API local.
- Sync opcional.

### 2.3 Multiplataforma

Prioridad inicial:

1. Linux
2. Windows
3. Android

Prioridad secundaria:

4. macOS
5. iOS
6. Raspberry Pi
7. Docker
8. NAS compatibles con Docker

### 2.4 Seguridad y ética

MyLAN debe incluir reglas claras:

- Usar solo en redes propias o con autorización.
- No incluir funciones ofensivas por defecto.
- No automatizar explotación de vulnerabilidades.
- No hacer ataques de desautenticación Wi‑Fi.
- No interceptar tráfico privado sin permiso explícito.
- No usar ARP spoofing para bloquear dispositivos en modo normal.
- Preferir control mediante APIs de router/firewall.

---

## 3. Funcionalidades de referencia tipo Fing

A partir de documentación pública de Fing, tiendas de apps y páginas de producto, las áreas funcionales actuales que MyLAN debería cubrir son:

### 3.1 Descubrimiento e inventario

- Escanear redes Wi‑Fi y LAN.
- Descubrir dispositivos conectados.
- Reconocer IP, MAC, nombre, modelo, fabricante y vendor.
- Mostrar tipo de dispositivo.
- Analizar NetBIOS, UPnP, SNMP y Bonjour/mDNS.
- Inventario de redes y dispositivos.

### 3.2 Herramientas de diagnóstico

- Ping.
- Traceroute.
- DNS lookup.
- Port scan.
- Speed test.
- Latencia.
- Upload/download.
- Análisis de proveedor de internet.
- Detección de outages.

### 3.3 Monitoreo y alertas

- Monitoreo continuo.
- Escaneos automáticos.
- Cronología de eventos.
- Alertas cuando un dispositivo aparece.
- Alertas cuando un dispositivo se desconecta.
- Notificaciones móviles.
- Notificaciones por email.
- Notificaciones por webhooks.

### 3.4 Seguridad

- Detección de puertos abiertos.
- Análisis de vulnerabilidad de router.
- Detección de dispositivos desconocidos.
- Detección de posibles cámaras IP.
- Cambios sospechosos en red.
- Reporte de riesgos.

### 3.5 Control de red

- Bloquear dispositivos.
- Auto-bloquear dispositivos desconocidos.
- Pausar internet.
- Horarios de downtime.
- Límites o restricciones por dispositivo.
- Perfiles familiares o grupos.

### 3.6 Agent 24/7

- Agente instalable en Linux.
- Agente para Raspberry Pi.
- Agente para Docker.
- Posible despliegue en NAS compatible con Docker.
- Monitoreo permanente sin depender del teléfono.

### 3.7 Reporting y funciones profesionales

- Exportar CSV.
- Exportar JSON.
- Exportar PDF.
- API local.
- Historial completo.
- Múltiples redes.
- Workspaces.
- Colaboración.
- Búsqueda avanzada.
- Filtros.

---

## 4. Stack recomendado

### 4.1 Recomendación principal

```text
Core de red:      Rust
Agent 24/7:       Rust
CLI:              Rust
Desktop UI:       Tauri 2 + React/Svelte
Android UI:       Flutter o Tauri Mobile
Base de datos:    SQLite
API local:        REST + WebSocket, opcional gRPC
Plugins:          Rust + WebAssembly opcional
Empaquetado:      Cargo, Docker, Snap/AppImage/MSIX
```

### 4.2 Por qué Rust

Rust encaja bien porque:

- Tiene soporte para múltiples plataformas mediante targets.
- Permite rendimiento alto.
- Es seguro en memoria.
- Funciona bien para networking low-level.
- Puede compilar para Linux, Windows y Android.
- Es adecuado para CLI, agent, librerías y backend embebido.
- Facilita compartir el core entre desktop, agent y móvil.

### 4.3 Por qué Tauri

Tauri 2 permite crear apps para Windows, Linux, macOS, Android e iOS desde una base común, con un backend Rust y frontend web. Es atractivo para MyLAN porque permite reutilizar el core Rust y crear una app desktop liviana.

### 4.4 Por qué considerar Flutter

Flutter tiene soporte oficial para Android, Windows, Linux, macOS, iOS y web. Si la prioridad visual y móvil es alta, Flutter puede dar mejor experiencia Android que Tauri Mobile en etapas tempranas.

### 4.5 Decisión recomendada

Para reducir riesgo:

```text
Fase inicial:
- Rust core
- Rust CLI
- Rust Agent
- Tauri Desktop para Linux/Windows

Fase móvil:
- Flutter Android conectado al Agent/API local
- Reutilizar el core Rust vía FFI solo cuando sea necesario
```

Esto evita depender demasiado pronto de permisos complejos de Android para escaneos low-level.

---

## 5. Arquitectura general

```text
┌─────────────────────────────────────────────────────────────┐
│                         MyLAN UI                            │
│  Desktop App │ Android App │ Web Local │ CLI                │
└──────────────────────────────┬──────────────────────────────┘
                               │
                         Local API
                    REST / WebSocket / gRPC
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                        MyLAN Core                           │
│  Discovery │ Scanner │ Fingerprint │ Monitor │ Security     │
│  Speedtest │ Events  │ Rules       │ Plugins │ Reports      │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                     Platform Adapters                       │
│  Linux raw sockets/netlink │ Windows Npcap/WMI/IP Helper    │
│  Android APIs/fallbacks    │ Docker/Linux Agent             │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                         Storage                             │
│  SQLite │ Event log │ Device inventory │ Rules │ Settings   │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Estructura sugerida del repositorio

```text
mylan/
├── README.md
├── LICENSE
├── SECURITY.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── ROADMAP.md
├── docs/
│   ├── architecture.md
│   ├── ethics.md
│   ├── scanner-design.md
│   ├── android-limitations.md
│   ├── plugin-system.md
│   └── api.md
├── crates/
│   ├── mylan-core/
│   ├── mylan-discovery/
│   ├── mylan-scanner/
│   ├── mylan-fingerprint/
│   ├── mylan-monitor/
│   ├── mylan-security/
│   ├── mylan-speedtest/
│   ├── mylan-db/
│   ├── mylan-api/
│   ├── mylan-platform/
│   └── mylan-plugin-sdk/
├── apps/
│   ├── cli/
│   ├── desktop-tauri/
│   ├── android-flutter/
│   └── web-local/
├── agent/
│   ├── systemd/
│   ├── docker/
│   ├── snap/
│   └── raspberry-pi/
├── signatures/
│   ├── oui/
│   ├── device-rules/
│   ├── service-probes/
│   ├── router-checks/
│   └── risk-rules/
├── plugins/
│   ├── openwrt/
│   ├── pfsense/
│   ├── opnsense/
│   ├── mikrotik/
│   ├── unifi/
│   └── fritzbox/
└── tests/
    ├── integration/
    ├── fixtures/
    └── network-lab/
```

---

## 7. Módulos internos

### 7.1 mylan-core

Responsabilidades:

- Modelos base.
- Configuración.
- Sistema de jobs.
- Scheduler.
- Event bus.
- Permisos internos.
- Tipos comunes.

Entidades principales:

```text
Network
Interface
Device
DeviceIdentity
Scan
ScanResult
Service
Port
Event
Risk
Rule
Plugin
Workspace
```

### 7.2 mylan-discovery

Responsabilidades:

- Descubrir hosts activos.
- Detectar gateway.
- Detectar subnet.
- Leer ARP cache.
- ARP sweep.
- ICMP ping.
- TCP ping fallback.
- mDNS.
- SSDP/UPnP.
- NetBIOS.
- SNMP básico.

Estrategias:

```text
Fast discovery:
- ARP cache
- gateway
- TCP ping top ports

Normal discovery:
- ARP sweep
- ICMP
- mDNS
- SSDP

Deep discovery:
- SNMP
- NetBIOS
- UDP probes
```

### 7.3 mylan-scanner

Responsabilidades:

- Escaneo de puertos TCP.
- Escaneo UDP básico.
- Banner grabbing.
- Detección de servicios.
- Perfiles de escaneo.
- Rate limiting.
- Timeouts.
- Cancelación de jobs.

Perfiles:

```text
quick:
- Top 32 puertos
- Timeout bajo

normal:
- Top 100/200 puertos
- Banner básico

deep:
- Top 1000 puertos
- Probes HTTP/SSH/FTP/RTSP/SMB/MQTT

iot:
- RTSP, HTTP, ONVIF, MQTT, CoAP, UPnP

router:
- HTTP/HTTPS admin, SSH, Telnet, UPnP, DNS, DHCP
```

### 7.4 mylan-fingerprint

Responsabilidades:

- Identificar tipo de dispositivo.
- Identificar fabricante.
- Inferir modelo.
- Inferir OS.
- Calcular confidence score.
- Permitir correcciones manuales.

Fuentes:

- OUI/MAC vendor.
- Hostname.
- mDNS/Bonjour.
- SSDP/UPnP.
- NetBIOS.
- SNMP.
- HTTP headers.
- HTTP title.
- SSH banner.
- Puertos abiertos.
- TTL.
- DHCP fingerprints si están disponibles.
- Reglas comunitarias.

Ejemplo de regla:

```yaml
id: camera_rtsp_generic
match:
  any:
    - port: 554
      service: rtsp
    - mdns_contains: "_rtsp"
    - upnp_device_type_contains: "MediaServer"
score:
  device_type: camera
  confidence: 75
```

### 7.5 mylan-monitor

Responsabilidades:

- Escaneos programados.
- Detección de cambios.
- Timeline.
- Eventos.
- Estado online/offline.
- Alertas.

Eventos:

```text
network_discovered
device_seen
device_new
device_online
device_offline
device_ip_changed
device_mac_changed
device_hostname_changed
device_vendor_changed
device_type_changed
port_opened
port_closed
service_changed
gateway_down
internet_down
dns_failure
speed_degraded
risk_detected
unknown_device_detected
```

### 7.6 mylan-security

Responsabilidades:

- Checks de seguridad no intrusivos.
- Clasificación de riesgo.
- Detección de puertos sensibles.
- Detección de router expuesto.
- Detección de UPnP activo.
- Detección de posibles cámaras IP.
- Recomendaciones.

Riesgos iniciales:

```text
HIGH:
- Telnet abierto
- Admin HTTP sin HTTPS en router
- Puerto remoto de administración en gateway
- Dispositivo desconocido nuevo
- RTSP expuesto en cámara no reconocida

MEDIUM:
- UPnP activo
- SMB abierto en red doméstica
- FTP abierto
- HTTP admin panel en IoT
- SNMP público

LOW:
- Hostname genérico
- Vendor desconocido
- Cambios frecuentes de IP
```

### 7.7 mylan-control

Responsabilidades:

- Bloqueo de dispositivos.
- Pausa de internet.
- Horarios.
- Perfiles.
- Integración con firewalls/routers.

Regla clave:

> MyLAN no debe usar técnicas agresivas por defecto para bloquear tráfico. El control debe hacerse mediante integración con el gateway, router, firewall o DNS local.

Integraciones prioritarias:

1. OpenWrt
2. MikroTik
3. UniFi
4. pfSense
5. OPNsense
6. FRITZ!Box
7. dnsmasq
8. nftables/iptables en Linux gateway

### 7.8 mylan-speedtest

Responsabilidades:

- Medir download.
- Medir upload.
- Medir latencia.
- Medir jitter.
- Estimar packet loss.
- Guardar histórico.
- Programar pruebas.
- Detectar degradación.

Modo inicial:

- Prueba contra servidores configurables.
- Fallback con endpoints HTTP.
- Modo self-hosted para pruebas internas.

### 7.9 mylan-api

Responsabilidades:

- API local.
- WebSocket para eventos en tiempo real.
- Autenticación local.
- Tokens para integraciones.
- API estable para apps móviles.

Endpoints iniciales:

```http
GET  /api/v1/status
GET  /api/v1/interfaces
GET  /api/v1/networks
GET  /api/v1/devices
GET  /api/v1/devices/{id}
GET  /api/v1/events
GET  /api/v1/scans
POST /api/v1/scans
GET  /api/v1/risks
GET  /api/v1/reports/network
GET  /api/v1/reports/security
GET  /api/v1/export/devices.csv
GET  /api/v1/export/devices.json
WS   /api/v1/events/live
```

---

## 8. Base de datos inicial

### 8.1 Tablas principales

```sql
CREATE TABLE networks (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  cidr TEXT NOT NULL,
  gateway_ip TEXT,
  dns_servers TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE devices (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  primary_mac TEXT,
  primary_ip TEXT,
  hostname TEXT,
  display_name TEXT,
  vendor TEXT,
  manufacturer TEXT,
  model TEXT,
  device_type TEXT,
  os_family TEXT,
  confidence INTEGER DEFAULT 0,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  is_trusted INTEGER DEFAULT 0,
  is_hidden INTEGER DEFAULT 0,
  notes TEXT,
  FOREIGN KEY(network_id) REFERENCES networks(id)
);

CREATE TABLE device_addresses (
  id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL,
  ip TEXT,
  mac TEXT,
  interface_name TEXT,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE services (
  id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL,
  protocol TEXT NOT NULL,
  port INTEGER NOT NULL,
  service_name TEXT,
  product TEXT,
  version TEXT,
  banner TEXT,
  state TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE scans (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  scan_type TEXT NOT NULL,
  profile TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  summary_json TEXT,
  FOREIGN KEY(network_id) REFERENCES networks(id)
);

CREATE TABLE events (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  device_id TEXT,
  event_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  message TEXT NOT NULL,
  data_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(network_id) REFERENCES networks(id),
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE risks (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  device_id TEXT,
  severity TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  recommendation TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  resolved_at TEXT,
  FOREIGN KEY(network_id) REFERENCES networks(id),
  FOREIGN KEY(device_id) REFERENCES devices(id)
);
```

---

## 9. Comandos CLI propuestos

```bash
# Estado general
mylan status

# Escanear red actual
mylan scan

# Escaneo rápido
mylan scan --profile quick

# Escaneo profundo
mylan scan --profile deep

# Listar dispositivos
mylan devices

# Ver detalle de dispositivo
mylan device 192.168.1.20

# Escanear puertos
mylan ports 192.168.1.20 --top 100

# Ejecutar diagnóstico
mylan diagnose

# Ver eventos
mylan events

# Iniciar agent
mylan agent start

# Exportar inventario
mylan export devices --format csv
mylan export devices --format json

# Servir API local
mylan serve --port 43117
```

---

## 10. Roadmap paso a paso

## Fase 0 — Preparación del proyecto

**Objetivo:** dejar listo el proyecto para desarrollo público.

Duración sugerida: 1 semana.

Tareas:

- Crear repositorio `mylan`.
- Elegir licencia.
- Crear README inicial.
- Crear roadmap público.
- Crear guía ética.
- Crear CONTRIBUTING.
- Crear SECURITY.
- Definir arquitectura.
- Definir estándares de código.
- Configurar CI.
- Configurar formatos y linters.

Entregables:

- Repositorio público.
- Documentación inicial.
- Primer tablero de issues.
- Labels:
  - `good first issue`
  - `core`
  - `discovery`
  - `scanner`
  - `desktop`
  - `android`
  - `agent`
  - `security`
  - `plugin`
  - `docs`

Criterio de salida:

- El repo puede recibir contribuciones externas.

---

## Fase 1 — CLI e inventario básico

**Objetivo:** descubrir dispositivos de una LAN y mostrarlos por CLI.

Duración sugerida: 2–4 semanas.

Funcionalidades:

- Detectar interfaz activa.
- Detectar IP local.
- Detectar gateway.
- Calcular CIDR.
- Leer ARP cache.
- Ejecutar escaneo TCP fallback.
- Guardar inventario SQLite.
- Exportar JSON/CSV.
- Mostrar lista de dispositivos.

Tareas técnicas:

- Crear `mylan-core`.
- Crear `mylan-db`.
- Crear `mylan-discovery`.
- Crear `apps/cli`.
- Implementar modelos `Network`, `Device`, `Scan`.
- Implementar persistencia.
- Implementar `mylan scan`.
- Implementar `mylan devices`.

Criterio de salida:

- En Linux y Windows se puede ejecutar `mylan scan` y ver dispositivos básicos.

---

## Fase 2 — Fingerprinting v1

**Objetivo:** reconocer mejor cada dispositivo.

Duración sugerida: 3–5 semanas.

Funcionalidades:

- Vendor por MAC/OUI.
- Hostname.
- Reverse DNS.
- mDNS.
- SSDP/UPnP.
- NetBIOS.
- SNMP básico.
- Tipo de dispositivo por reglas.
- Confidence score.
- Alias manual.
- Dispositivo confiable/no confiable.

Tareas técnicas:

- Crear `mylan-fingerprint`.
- Crear `signatures/oui`.
- Crear `signatures/device-rules`.
- Implementar motor de reglas.
- Agregar importador de OUI.
- Agregar edición manual.

Criterio de salida:

- La mayoría de dispositivos comunes aparecen con tipo probable:
  - router
  - phone
  - laptop
  - desktop
  - tv
  - printer
  - camera
  - nas
  - console
  - iot
  - unknown

---

## Fase 3 — Port scanner y herramientas de diagnóstico

**Objetivo:** igualar las herramientas básicas de diagnóstico: port scan, ping, traceroute y DNS lookup.

Duración sugerida: 3–6 semanas.

Funcionalidades:

- TCP port scan.
- Perfiles quick/normal/deep.
- Banner grabbing básico.
- Service detection.
- Ping.
- Traceroute.
- DNS lookup.
- Reporte de puertos por dispositivo.

Tareas técnicas:

- Crear `mylan-scanner`.
- Crear `signatures/service-probes`.
- Implementar scanner asíncrono con rate limiting.
- Implementar cancelación de scans.
- Implementar timeout configurable.
- Guardar servicios en DB.
- Exponer resultados por CLI.

Criterio de salida:

- Se puede ver qué puertos y servicios tiene cada dispositivo.

---

## Fase 4 — Desktop Alpha

**Objetivo:** crear app visual para Linux y Windows.

Duración sugerida: 4–8 semanas.

Funcionalidades:

- Dashboard.
- Lista de dispositivos.
- Detalle de dispositivo.
- Escaneo manual.
- Puertos abiertos.
- Eventos recientes.
- Exportación.
- Configuración básica.

Tareas técnicas:

- Crear `apps/desktop-tauri`.
- Crear `mylan-api`.
- Exponer API local.
- Conectar frontend a API.
- Crear UI inicial.
- Empaquetar Linux.
- Empaquetar Windows.

Pantallas:

```text
Dashboard
Devices
Device Detail
Scans
Events
Security
Settings
```

Criterio de salida:

- Usuario no técnico puede abrir MyLAN y ver su red.

---

## Fase 5 — Agent 24/7

**Objetivo:** permitir monitoreo continuo sin depender de abrir la app.

Duración sugerida: 4–8 semanas.

Funcionalidades:

- Agent como daemon.
- Escaneos programados.
- Timeline completo.
- Detección de cambios.
- Notificaciones locales.
- Webhooks.
- Docker.
- systemd.
- Raspberry Pi.

Tareas técnicas:

- Crear `mylan-agent`.
- Crear scheduler.
- Crear event diff engine.
- Crear configuración YAML/TOML.
- Crear Dockerfile.
- Crear unidad systemd.
- Crear guía Raspberry Pi.
- Crear modo headless.

Eventos mínimos:

- Dispositivo nuevo.
- Dispositivo online.
- Dispositivo offline.
- IP cambió.
- Hostname cambió.
- Puerto abierto.
- Puerto cerrado.
- Gateway caído.
- Internet caído.

Criterio de salida:

- MyLAN puede monitorear una red 24/7 en Linux/Raspberry Pi/Docker.

---

## Fase 6 — Alertas y notificaciones

**Objetivo:** alertar eventos importantes.

Duración sugerida: 2–4 semanas.

Canales:

- Desktop notification.
- Webhook.
- Email SMTP.
- Gotify.
- ntfy.
- Telegram bot opcional.
- Matrix opcional.
- Discord webhook opcional.

Reglas:

```text
Alertar si:
- aparece dispositivo desconocido
- desaparece dispositivo crítico
- se abre puerto sensible
- se cae internet
- se cae gateway
- cambia DNS
- aparece posible cámara IP
```

Criterio de salida:

- Usuario recibe una alerta cuando aparece un dispositivo desconocido.

---

## Fase 7 — Android Companion

**Objetivo:** app Android para ver red, iniciar scans y recibir alertas.

Duración sugerida: 6–10 semanas.

Estrategia:

- Android debe poder funcionar solo con capacidades limitadas.
- Android debe funcionar mejor conectado a un Agent local.

Modo standalone:

- Escaneo TCP connect.
- mDNS/SSDP cuando sea posible.
- DNS lookup.
- Ping fallback.
- Speed test.
- Lista de red básica.

Modo companion:

- Conexión al Agent.
- Eventos en tiempo real.
- Notificaciones.
- Historial.
- Seguridad.
- Control vía router plugins.

Limitaciones:

- Android requiere permisos para funciones Wi‑Fi/local network.
- Algunas funciones low-level no estarán disponibles sin APIs especiales, VPNService o root.
- No se debe prometer paridad total con Linux/Windows en modo standalone.

Criterio de salida:

- Usuario Android puede ver su red y recibir alertas desde el Agent.

---

## Fase 8 — Seguridad v1

**Objetivo:** detectar riesgos comunes sin explotación ofensiva.

Duración sugerida: 4–8 semanas.

Checks:

- Telnet abierto.
- FTP abierto.
- SMB abierto en red doméstica.
- RTSP abierto.
- ONVIF probable.
- UPnP activo.
- SNMP público.
- Admin panel HTTP.
- Router con puertos sensibles.
- DNS sospechoso.
- Dispositivo desconocido.

Reportes:

- Riesgo.
- Severidad.
- Evidencia.
- Recomendación.
- Estado:
  - open
  - acknowledged
  - resolved
  - ignored

Criterio de salida:

- MyLAN genera un reporte de seguridad útil y entendible.

---

## Fase 9 — Speed test y salud de red

**Objetivo:** medir calidad de internet y red local.

Duración sugerida: 3–6 semanas.

Funcionalidades:

- Download.
- Upload.
- Latencia.
- Jitter.
- Packet loss aproximado.
- Histórico.
- Detección de degradación.
- Tests programados.
- Reportes ISP básicos.

Criterio de salida:

- Usuario puede saber si su internet empeoró y cuándo.

---

## Fase 10 — Control de acceso

**Objetivo:** bloquear o pausar internet de dispositivos mediante integraciones seguras.

Duración sugerida: 8–12 semanas.

No hacer por defecto:

- ARP spoofing.
- Deauth attacks.
- Interceptar tráfico.
- MITM.
- Técnicas que rompan la red o puedan ser abusadas.

Hacer:

- OpenWrt plugin.
- MikroTik plugin.
- UniFi plugin.
- pfSense/OPNsense plugin.
- dnsmasq plugin.
- nftables plugin para Linux gateway.
- Perfiles.
- Horarios.
- Logs de auditoría.

Criterio de salida:

- En un router compatible, MyLAN puede bloquear un dispositivo de forma reversible y auditable.

---

## Fase 11 — Reporting y workspaces

**Objetivo:** funcionalidades profesionales gratuitas.

Duración sugerida: 6–10 semanas.

Funcionalidades:

- Multi-red.
- Workspaces.
- Roles.
- Reportes PDF.
- Reportes CSV/JSON.
- Filtros avanzados.
- Búsqueda global.
- Comparación histórica.
- API tokens.
- Export programado.

Criterio de salida:

- MyLAN sirve para casa, pequeña oficina y consultores.

---

## Fase 12 — Sync self-hosted

**Objetivo:** sincronizar varias instalaciones sin nube obligatoria.

Duración sugerida: 8–16 semanas.

Componentes:

- `mylan-server`.
- Sync de eventos.
- Sync de inventario.
- Sync de reglas.
- Usuarios.
- Workspaces.
- Autenticación.
- Cifrado en tránsito.
- Opción de cifrado extremo a extremo.

Criterio de salida:

- Un usuario puede desplegar su propio servidor MyLAN y conectar agentes remotos.

---

## 11. Matriz de compatibilidad

| Función | Linux | Windows | Android standalone | Android + Agent | Docker/RPi |
|---|---:|---:|---:|---:|---:|
| Descubrimiento básico | Sí | Sí | Parcial | Sí | Sí |
| ARP scan | Sí | Sí, con backend adecuado | Limitado | Sí vía Agent | Sí |
| ICMP ping | Sí | Sí | Limitado | Sí vía Agent | Sí |
| TCP connect scan | Sí | Sí | Sí | Sí | Sí |
| mDNS/SSDP | Sí | Sí | Parcial | Sí | Sí |
| NetBIOS | Sí | Sí | Limitado | Sí | Sí |
| SNMP | Sí | Sí | Parcial | Sí | Sí |
| Port scan | Sí | Sí | Sí, limitado | Sí | Sí |
| Traceroute | Sí | Sí | Parcial | Sí | Sí |
| Speed test | Sí | Sí | Sí | Sí | Sí |
| Monitoreo 24/7 | Sí | Sí | No ideal | Sí | Sí |
| Alertas | Sí | Sí | Sí | Sí | Sí |
| Control router | Plugin | Plugin | Vía Agent | Sí | Sí |
| Reportes | Sí | Sí | Sí | Sí | Sí |

---

## 12. MVP recomendado

El MVP no debe intentar hacerlo todo. Debe probar que MyLAN puede descubrir, identificar y monitorear dispositivos.

### MVP v0.1

Incluye:

- CLI.
- Linux + Windows.
- Escaneo de red actual.
- Inventario local.
- IP/MAC/hostname/vendor.
- Export JSON/CSV.
- SQLite.

No incluye:

- UI.
- Android.
- Control parental.
- Speed test.
- Reportes PDF.
- Workspaces.

### MVP v0.2

Incluye:

- mDNS.
- SSDP.
- NetBIOS.
- Fingerprinting por reglas.
- Tipos de dispositivo.
- Port scan rápido.
- Detalle de dispositivo.

### MVP v0.3

Incluye:

- Agent.
- Timeline.
- Eventos.
- Notificaciones webhook.
- Docker.
- Raspberry Pi.

### MVP v0.4

Incluye:

- Desktop alpha.
- Dashboard.
- Lista de dispositivos.
- Eventos.
- Port scan visual.

---

## 13. Primeros issues para GitHub

1. Crear workspace Rust.
2. Crear modelos `Device`, `Network`, `Scan`.
3. Implementar detección de interfaz activa.
4. Implementar lectura de ARP cache en Linux.
5. Implementar lectura de ARP cache en Windows.
6. Implementar TCP ping fallback.
7. Implementar SQLite migrations.
8. Crear comando `mylan scan`.
9. Crear comando `mylan devices`.
10. Importar base OUI.
11. Crear motor simple de reglas.
12. Implementar mDNS discovery.
13. Implementar SSDP discovery.
14. Implementar port scanner TCP quick.
15. Implementar export JSON.
16. Implementar export CSV.
17. Crear Dockerfile para agent.
18. Crear unidad systemd.
19. Crear endpoint `GET /api/v1/devices`.
20. Crear dashboard desktop inicial.

---

## 14. Modelo de plugins

### 14.1 Objetivo

Permitir que la comunidad agregue:

- Routers.
- Firewalls.
- Firmwares.
- Nuevos detectores de dispositivos.
- Nuevos checks de seguridad.
- Nuevos canales de notificación.

### 14.2 Tipos de plugin

```text
discovery_plugin
fingerprint_plugin
security_check_plugin
notification_plugin
router_control_plugin
export_plugin
```

### 14.3 Ejemplo de router plugin

```yaml
id: openwrt
name: OpenWrt
capabilities:
  - list_clients
  - block_device
  - unblock_device
  - pause_device
  - schedule_downtime
auth:
  methods:
    - luci_rpc
    - ssh
```

---

## 15. API local inicial

### 15.1 Autenticación

- Token local generado en primera ejecución.
- Opción de permitir solo `localhost`.
- Pairing con QR para Android.
- Revocación de tokens.

### 15.2 Eventos en tiempo real

```json
{
  "type": "device_new",
  "severity": "warning",
  "device": {
    "ip": "192.168.1.45",
    "mac": "AA:BB:CC:DD:EE:FF",
    "vendor": "Example Vendor",
    "device_type": "unknown"
  },
  "created_at": "2026-06-27T12:00:00Z"
}
```

### 15.3 Ejemplo de respuesta de dispositivo

```json
{
  "id": "dev_01",
  "display_name": "Living Room TV",
  "ip": "192.168.1.32",
  "mac": "AA:BB:CC:DD:EE:FF",
  "vendor": "Samsung",
  "device_type": "tv",
  "confidence": 86,
  "is_trusted": true,
  "last_seen_at": "2026-06-27T12:00:00Z",
  "services": [
    {
      "protocol": "tcp",
      "port": 8008,
      "service_name": "http"
    }
  ]
}
```

---

## 16. Seguridad interna del proyecto

### 16.1 Diseño seguro por defecto

- Rate limiting para scans.
- Confirmación antes de escaneos profundos.
- No escanear redes públicas automáticamente.
- No guardar credenciales de router sin cifrado.
- Logs sin datos sensibles cuando sea posible.
- Permisos mínimos.
- Sandbox para plugins si se usa WASM.

### 16.2 Threat model inicial

Actores:

- Usuario legítimo.
- Plugin malicioso.
- Dispositivo comprometido en LAN.
- Usuario no autorizado en la máquina.
- Servicio web local expuesto accidentalmente.

Riesgos:

- Exposición de inventario de red.
- Abuso de API local.
- Ejecución de plugin malicioso.
- Captura de credenciales de router.
- Escaneo no autorizado.

Mitigaciones:

- API solo local por defecto.
- Tokens con scopes.
- UI de permisos por plugin.
- Firma o reputación de plugins.
- Cifrado de secretos en disco.
- Auditoría de acciones de control.
- Deshabilitar control de red por defecto.

---

## 17. Licencia recomendada

### Opción recomendada: AGPL-3.0-or-later

Razón:

- MyLAN podría tener servidor self-hosted.
- AGPL protege contra forks cerrados ofrecidos como servicio.
- Mantiene mejoras disponibles para la comunidad.

### Alternativa: GPL-3.0-or-later

Buena si el proyecto se enfoca en desktop/agent y no tanto en servidor.

### Alternativa: Apache-2.0/MIT

Buena para máxima adopción, pero permite forks cerrados.

### Recomendación práctica

```text
Core + Agent + Server: AGPL-3.0-or-later
Plugin SDK: Apache-2.0 o MIT
Signatures/rules: CC BY-SA 4.0 o similar
```

---

## 18. Branding de MyLAN

### 18.1 Nombre

**MyLAN**

Ventajas:

- Corto.
- Fácil de recordar.
- Indica red local.
- Amigable para usuarios no técnicos.
- Funciona bien para app doméstica.

### 18.2 Taglines

Opciones:

1. **MyLAN — Know your network.**
2. **MyLAN — Tu red, bajo control.**
3. **MyLAN — Open network visibility for everyone.**
4. **MyLAN — Descubre, monitorea y protege tu red.**
5. **MyLAN — Fing-like power, open-source freedom.**

Recomendado:

```text
MyLAN — Tu red, bajo control.
```

### 18.3 Descripción corta

> MyLAN es una app open-source y gratuita para descubrir, monitorear y proteger tu red local, sin paywalls y sin nube obligatoria.

### 18.4 Descripción larga

> MyLAN te ayuda a saber qué dispositivos están conectados a tu red, detectar cambios, recibir alertas, ejecutar diagnósticos, revisar puertos abiertos y controlar el acceso mediante integraciones seguras con routers y firewalls. Está diseñado como una alternativa libre, local-first y extensible a herramientas comerciales de monitoreo de red.

---

## 19. Diferenciadores frente a Fing

| Área | Fing | MyLAN |
|---|---|---|
| Código abierto | No | Sí |
| Paywalls | Sí | No |
| Escaneos ilimitados | Según plan | Sí |
| Agent 24/7 | Suscripción | Gratis |
| API local | Según plan | Gratis |
| Self-hosted | Limitado/no central | Sí |
| Plugins comunitarios | No central | Sí |
| Control router abierto | Limitado | Plugins |
| Reglas de fingerprinting | Cerradas | Abiertas |
| Export completo | Según plan | Gratis |

---

## 20. Riesgos técnicos

### 20.1 Android

Android puede limitar acceso a información Wi‑Fi y red local. Por eso se recomienda una app Android companion que use un Agent local para funciones avanzadas.

Mitigación:

- Permisos claros.
- Modo standalone limitado.
- Modo Agent recomendado.
- Documentar limitaciones por versión de Android.

### 20.2 Windows

Algunas funciones low-level pueden requerir Npcap o APIs específicas.

Mitigación:

- Fallback TCP connect.
- Integración opcional con Npcap.
- Documentar permisos de administrador.

### 20.3 Fingerprinting

Reconocer dispositivos con precisión es difícil.

Mitigación:

- Motor de reglas comunitario.
- Confidence score.
- Correcciones manuales.
- Dataset abierto de firmas.
- No prometer 100% de precisión.

### 20.4 Control de acceso

Bloquear dispositivos de forma universal es difícil sin controlar el gateway.

Mitigación:

- Plugins de router/firewall.
- DNS/firewall gateway mode.
- No usar técnicas agresivas por defecto.

---

## 21. Métricas de éxito

### Técnicas

- Tiempo de escaneo rápido menor a 10 segundos en una /24 típica.
- Detección de más del 90% de dispositivos activos en red doméstica común.
- Port scan quick menor a 30 segundos por red típica.
- Agent estable 7 días sin reinicios.
- Consumo bajo en Raspberry Pi.

### Producto

- Instalación en menos de 5 minutos.
- Primer escaneo en menos de 60 segundos.
- Usuario entiende qué dispositivos hay en su red.
- Usuario recibe alerta útil ante dispositivo nuevo.
- Export completo sin cuenta.

### Comunidad

- 10 contribuidores.
- 50 reglas de fingerprinting.
- 5 plugins de router.
- Documentación para Linux, Windows, Android y Docker.
- Roadmap público.

---

## 22. Plan de versiones

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
| 1.5 | Pro-Free | workspaces + reportes |
| 2.0 | Mesh | multi-site self-hosted |

---

## 23. Primer sprint sugerido

Duración: 2 semanas.

### Semana 1

- Crear repo.
- Crear workspace Rust.
- Crear CLI básica.
- Detectar interfaz local.
- Detectar IP local.
- Detectar gateway.
- Crear modelos.
- Configurar SQLite.

### Semana 2

- Leer ARP cache Linux.
- Leer ARP cache Windows.
- Implementar TCP ping fallback.
- Guardar dispositivos.
- Mostrar tabla.
- Exportar JSON.
- Crear primer README real.

Resultado esperado:

```bash
mylan scan
mylan devices
mylan export devices --format json
```

---

## 24. Backlog extendido

### Discovery

- ARP sweep.
- ICMP echo.
- TCP SYN opcional.
- TCP connect fallback.
- UDP probes.
- mDNS.
- SSDP.
- NetBIOS.
- SNMP.
- DHCP fingerprints.
- VLAN awareness.
- IPv6 discovery.

### Scanner

- Top ports.
- Custom ports.
- UDP scan limitado.
- Banner grabbing.
- HTTP probe.
- TLS cert probe.
- SSH banner.
- SMB info.
- RTSP detection.
- ONVIF detection.
- MQTT detection.

### UI

- Dashboard.
- Network map.
- Device detail.
- Risk detail.
- Event timeline.
- Scan progress.
- Reports.
- Settings.
- Plugin manager.
- Pairing with Agent.

### Agent

- systemd.
- Docker.
- Snap.
- Auto-update opcional.
- Config file.
- Local API.
- Headless mode.
- Remote pairing.
- Health check.
- Watchdog.

### Security

- Risk rules.
- Router checks.
- UPnP check.
- DNS check.
- Camera detection.
- Unknown device alert.
- Sensitive port alert.
- Baseline drift.
- Device trust model.

### Control

- OpenWrt.
- MikroTik.
- UniFi.
- pfSense.
- OPNsense.
- FRITZ!Box.
- dnsmasq.
- nftables.
- iptables.
- Schedules.
- Profiles.

### Reports

- CSV.
- JSON.
- PDF.
- HTML.
- Scheduled reports.
- ISP performance.
- Network inventory.
- Security posture.
- Change history.

---

## 25. Decisiones que faltan

Antes de empezar desarrollo intensivo:

1. ¿Licencia final?
2. ¿Tauri o Flutter para UI inicial?
3. ¿Android standalone temprano o después del Agent?
4. ¿Soporte macOS desde el inicio o después?
5. ¿Base de firmas propia desde cero o importadores?
6. ¿Plugins nativos Rust o WebAssembly?
7. ¿Sync self-hosted en 1.x o 2.x?
8. ¿Nombre del binario: `mylan`, `mylanctl` o `mylan-cli`?

Recomendación:

```text
Licencia: AGPL-3.0-or-later
UI inicial: Tauri Desktop
Android: después del Agent
Plugins iniciales: Rust nativo
Plugin sandbox futuro: WebAssembly
Binario: mylan
Agent: myland
Servidor self-hosted futuro: mylan-server
```

---

## 26. Referencias consultadas

Estas fuentes se usaron para contrastar funcionalidades actuales de Fing y decisiones técnicas del plan:

1. Fing — Network Scanner & Monitoring Tool  
   https://www.fing.com/

2. Fing pricing/features en español  
   https://www.fing.com/es/

3. Fing App Store — Network Scanner features  
   https://apps.apple.com/us/app/fing-network-scanner/id430921107

4. Fing Google Play — Escáner de red  
   https://play.google.com/store/apps/details?id=com.overlook.android.fing

5. Fing Agent  
   https://www.fing.com/agent/

6. Fing Agent en español  
   https://www.fing.com/es/agent/

7. Fing network monitoring features  
   https://www.fing.com/news/network-monitoring-features/

8. Fing network control  
   https://www.fing.com/control/

9. Fing Desktop en español  
   https://www.fing.com/es/desktop/

10. Nmap Reference Guide  
    https://nmap.org/book/man.html

11. Nmap Service and Version Detection  
    https://nmap.org/book/man-version-detection.html

12. Nmap OS Detection  
    https://nmap.org/book/man-os-detection.html

13. Tauri 2  
    https://v2.tauri.app/

14. Tauri 2 Stable Release  
    https://v2.tauri.app/blog/tauri-20/

15. Flutter Supported Platforms  
    https://docs.flutter.dev/reference/supported-platforms

16. Rust Platform Support  
    https://doc.rust-lang.org/rustc/platform-support.html

17. Android Nearby Wi‑Fi Devices permission  
    https://developer.android.com/develop/connectivity/wifi/wifi-permissions

18. Android Local Network Permission  
    https://developer.android.com/privacy-and-security/local-network-permission

19. Open Source Initiative — AGPL-3.0  
    https://opensource.org/license/agpl-3-0

20. Choose a License  
    https://choosealicense.com/licenses/

---

## 27. Resumen ejecutivo

**MyLAN** debería construirse como un sistema modular:

```text
Rust Core + CLI + Agent
Tauri Desktop
Flutter Android Companion
SQLite local
API local
Plugins de router/firewall
Sync self-hosted futuro
```

El orden correcto es:

1. Descubrir dispositivos.
2. Identificarlos.
3. Escanear servicios.
4. Mostrar UI.
5. Monitorear 24/7.
6. Alertar.
7. Crear Android companion.
8. Agregar seguridad.
9. Agregar control vía router.
10. Agregar reportes/workspaces.
11. Agregar sync self-hosted.

El proyecto debe enfocarse en ser:

- libre,
- local-first,
- auditable,
- extensible,
- seguro,
- multiplataforma,
- sin paywalls.

