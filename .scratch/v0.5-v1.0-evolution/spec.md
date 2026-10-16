# Especificación Técnica Integral: Evolución de Arquitectura MyLAN (v0.5 – v1.0)

**Estado:** Especificación Aprobada (Vía Wayfinding Map `v0.5-v1.0-evolution`)  
**Fecha:** Octubre 2026  
**Plataforma Objetivo:** Desktop Tauri 2 (Linux & Windows), Rust 1.96, SQLite WAL (`rusqlite`), React 18 + TypeScript + Tailwind CSS.  
**Licencia:** AGPL-3.0-or-later  

---

## 1. Visión y Principios Rectores

Esta especificación consolida las decisiones de arquitectura, contratos de datos y diseño de interfaz para los 6 pilares de la evolución de MyLAN:
1. **Monitoreo Continuo en Desktop** (Loop en segundo plano + System Tray + Autoarranque).
2. **Alertas y Notificaciones Nativas del SO** (Agrupación inteligente + Modo Censura + Deep Linking).
3. **Suite Interactiva de Herramientas de Red** (Ping, Traceroute, DNS Lookup, Wake-on-LAN, Port Checker, Speed Test).
4. **Motor de Evaluación de Riesgos y Auditoría Pasiva** (Severidad de 4 niveles, Score de seguridad 0-100, ciclo de vida de hallazgos).
5. **Speed Test y Salud de Red** (Cloudflare Anycast primario, fallback NDT7, salud local unprivileged y retención SQLite).
6. **Workspaces y Generador de Informes PDF** (Partición relacional, detección por red, reportes React `@react-pdf/renderer` con censura).
7. **Sincronización Serverless con Google Drive (Zero-Knowledge)** (OAuth 2.0 PKCE loopback, `drive.appdata`, Argon2id + XChaCha20-Poly1305, frase mnemónica BIP-39 y almacenamiento en OS Keyring).

---

## 2. Pilar 1: Monitoreo Continuo en Desktop (Tauri 2 + System Tray)

### 2.1. Topología del Ciclo de Vida
- **Desktop-Exclusive:** El monitoreo continuo vive dentro del proceso principal de `apps/desktop-tauri`. No requiere un daemon independiente ni servicios systemd.
- **Interceptación de Cierre:** Se captura `WindowEvent::CloseRequested`. Si la opción está habilitada (por defecto), la ventana ejecuta `window.hide()` en lugar de destruir el proceso.
- **System Tray:**
  - Item de estado: `Monitoreando: <SSID o CIDR>` o `Monitoreo en pausa`.
  - Acciones: `Abrir MyLAN` (restaura y enfoca ventana), `Escanear ahora` (barrido inmediato), `Pausar/Reanudar monitoreo` y `Salir` (cierre definitivo con código 0 y limpieza de SQLite/Tokio).
- **Autoarranque del Sistema:** Integración de `tauri-plugin-autostart = "2"` configurable desde Ajustes (`Settings.tsx`) para arrancar minimizado a la bandeja al iniciar el sistema operativo.

### 2.2. Tarea en Segundo Plano (Tokio Task)
- Orquestada en `tauri::Builder::setup` gobernada por un `tokio_util::sync::CancellationToken`.
- **Frecuencia:** Configurable por el usuario (1m, 2m, 5m, 10m, 15m, 30m; **5 minutos por defecto**).
- **Perfil:** Perfil `quick` (únicamente barrido ARP/ping de presencia) para minimizar consumo de CPU y no saturar enlaces Wi-Fi residenciales.
- **Skip Guard (`NetworkRunner`):** Si un barrido previo aún está en ejecución, el tick se omite para evitar solapamientos concurrentes.
- **Tolerancia a Suspensión (Sleep/Wake):**
  - Período de gracia de 20 segundos tras reanudar el equipo antes del primer escaneo.
  - Marcado de `cold_start = true` en el primer escaneo post-reanudación para suprimir falsas alarmas masivas.
  - **Histéresis:** Se exigen **3 escaneos fallidos consecutivos** (15 min por defecto) antes de marcar un dispositivo como `offline`.

---

## 3. Pilar 2: Alertas y Notificaciones Nativas del SO

### 3.1. Catálogo de Disparadores
- `device_new_untrusted`: Dispositivo nuevo detectado que no está en la base de datos o no tiene la etiqueta manual `Confiable`.
- `device_identity_tampered`: Cambio de IP o MAC en un dispositivo que el usuario marcó previamente como `Confiable` (alerta de suplantación/ARP spoofing o cambio DHCP).
- `gateway_down` / `internet_down`: Pérdida de conectividad al default gateway o resolución DNS externa.
- `port_critical_opened`: Detección de puertos de alto riesgo recién abiertos (Telnet 23, SMB 445 expuesto, FTP 21).

### 3.2. Agrupación Inteligente y Throttling Antispam
- Si un escaneo genera **1 o 2 alertas**: se emiten avisos individuales específicos.
- Si un escaneo genera **3 o más alertas**: se emite una única **notificación resumen** consolidada (ej. `MyLAN: Se detectaron 5 nuevos dispositivos en la red`).
- Supresión absoluta de alertas durante el primer descubrimiento de una red (`cold_start = true`).

### 3.3. Modo Censura y Deep Linking
- **Censura Obligatoria:** Si el Modo Censura está habilitado en la app, las notificaciones nativas del SO enmascaran rigurosamente IPs, MACs y hostnames (ej. `Nuevo dispositivo: 192.168.1.*** (Apple Inc.)`).
- **Deep Linking:** Al hacer clic en la notificación del SO, MyLAN se enfoca y navega directamente a la ficha `DeviceDetail` del dispositivo involucrado o al historial de `Scans`.

---

## 4. Pilar 3: Suite Interactiva de Herramientas de Red (`Tools.tsx`)

### 4.1. Catálogo de Utilidades en `Tools.tsx`
1. **Ping Interactivo:** Latencia continua o ráfagas con visualización gráfica sparkline en tiempo real, RTT actual/mín/máx, Jitter RFC 3550 y pérdida de paquetes.
2. **Traceroute Visual:** Diagrama interactivo vertical de saltos (*hops*) con hostname resuelto, RTT y semáforo visual de latencia (Verde $<30$ms, Amarillo $30-100$ms, Rojo $>100$ms o timeout `*`).
3. **DNS Lookup Multidominio:** Resolución de registros (`A`, `AAAA`, `MX`, `TXT`, `CNAME`, `NS`, `PTR`, `SOA`) alternando entre DNS local, Cloudflare 1.1.1.1, Google 8.8.8.8 o Quad9 9.9.9.9.
4. **Wake-on-LAN (WoL):** Envío de paquete mágico UDP (puerto 9) a una dirección MAC con broadcast a la subred activa.
5. **Port Checker Rápido:** Comprobación puntual de puertos específicos (ej. 22, 80, 443, 8080) hacia una IP o dominio sin escaneo masivo de perfil.
6. **Speed Test Integrado:** Acceso directo a la prueba de velocidad de Internet.

### 4.2. Streaming IPC y Cancelación en Tauri 2
- Comandos: `start_tool_ping`, `start_tool_traceroute`, `cancel_tool(session_id)`.
- Cada paquete o salto emite un evento `app.emit("tool_stream", ToolPacket)` en tiempo real.
- Botón "Detener": cancela inmediatamente la tarea en Rust con `CancellationToken` sin dejar sockets zombis.
- **Puntos de Integración Contextual ("1 Clic"):** Botones directos en `DeviceDetail.tsx` y menú de clic derecho en la tabla de `Devices.tsx` para lanzar Ping, Traceroute, WoL o Port Checker pre-cargando la IP/MAC del host.

---

## 5. Pilar 4: Motor de Evaluación de Riesgos y Auditoría de Seguridad Pasiva

### 5.1. Taxonomía de Severidad y Score Global de Seguridad (0–100)
- **4 Niveles:**
  - `Critical` (-25 pts): Telnet 23 expuesto, panel de router sin TLS, dispositivos de control no confiables.
  - `High` (-15 pts): Servicios de texto plano sin cifrar (FTP 21, HTTP admin en IoT, SMB 445 en hosts no confiables).
  - `Medium` (-5 pts): UPnP activo en router, SNMP v1/v2 community `public`, RTSP sin autenticación.
  - `Low` (-1 pt): Configuraciones subóptimas, cambios frecuentes de IP en hosts estáticos, hostnames genéricos de fábrica.
- **Score Ponderado:** $\text{Score} = \max(0, 100 - \sum \text{penalizaciones})$. Hallazgos en estado `acknowledged` o `ignored` no restan puntos.

### 5.2. Persistencia en SQLite (`security_findings`)
- Estados:
  - `open`: Detectado y pendiente.
  - `acknowledged`: El usuario reconoce y acepta el riesgo intencionalmente.
  - `ignored`: Silenciado por el usuario.
  - `resolved`: Auto-resuelto automáticamente cuando la condición o puerto vulnerable desaparece en escaneos futuros.
- Timestamps de auditoría: `first_seen_at`, `last_seen_at`, `resolved_at`.

### 5.3. Interfaz y UX
- Widget resumen en `Dashboard.tsx` con semáforo y desglose por severidad.
- Pestaña "Seguridad y Riesgos" en `DeviceDetail.tsx` con tarjetas de evidencia técnica, panel colapsable de recomendaciones prácticas y botones de acción ("Aceptar riesgo", "Ignorar").
- Respeto estricto del Modo Censura en todas las evidencias.

---

## 6. Pilar 5: Speed Test y Salud de Red

### 6.1. Salud Local Unprivileged (Sin permisos root/admin)
- **Linux:** Datagram sockets ICMP (`SOCK_DGRAM` + `IPPROTO_ICMP` vía `socket2`) permitidos por `net.ipv4.ping_group_range`.
- **Windows:** API Win32 nativa `IcmpSendEcho` (`iphlpapi.dll`) vía `windows-sys` en `tokio::task::spawn_blocking` (sin necesidad de elevación de Administrador).
- **Sondeo DNS rápido UDP (Cross-Platform):** Paquetes compactos de 28 bytes al puerto 53 vía `tokio::net::UdpSocket`. RTT real a nivel de paquete sin depender de ICMP.
- **Gateway Fallback:** ICMP Echo $\rightarrow$ DNS UDP port 53 $\rightarrow$ TCP connect port 80/443.
- **Métricas:** Algoritmo de Jitter RFC 3550 continuo ($\alpha = 1/16$) + Desviación Estándar ($\sigma$) en ráfagas de 3 a 5 pings cada 200 ms.

### 6.2. Speed Test con Cloudflare (`speed.cloudflare.com`)
- **Motor Primario:** Cloudflare Speed Test Anycast (>330 ciudades globales) sobre HTTP/2 estándar vía `reqwest`.
  - `/cdn-cgi/trace`: Obtención de IP pública, país y código de PoP (`colo`).
  - `/__down?bytes=0`: Latencia pura RTT y jitter.
  - `/__down?bytes={N}`: Descarga en chunks progresivos (1MB, 10MB, 25MB, 50MB).
  - `/__up`: Subida en POST con payload generado en memoria.
- **Motor Fallback:** NDT7 (Measurement Lab) vía WebSockets para telemetría profunda de kernel TCP (`TCP_INFO`, bufferbloat).
- **Persistencia en SQLite:** Tablas `network_health_samples` y `speed_tests` con retención rodante automática de 30 días (~18 MB máx.).

---

## 7. Pilar 6: Workspaces y Generador de Informes PDF

### 7.1. Modelo Relacional de Workspaces
- Nueva tabla `workspaces` (`id`, `name`, `description`, `icon`, `color`, `created_at`).
- Clave foránea `workspace_id` en todas las tablas (`networks`, `scans`, `devices`, `events`, `security_findings`, `network_health_samples`).
- **Migración Brownfield:** Las filas existentes se asignan sin rotura al workspace `'default'` (*"Mi Red Local"*).
- **Detección Automática:** MyLAN detecta el Workspace correspondiente según el SSID o CIDR/Gateway de la red activa y conmuta automáticamente.
- **Selector UI:** Desplegable en la barra de navegación superior izquierda con colores e iconos.

### 7.2. Compilación de Informes PDF en React (`@react-pdf/renderer`)
- Generación en cliente en React con paridad visual exacta del sistema de diseño (paleta Tailwind, badges de Certeza/Confiable e isotipos hexagonales oficiales).
- **Secciones del Informe:**
  1. *Portada / Encabezado:* Logo oficial de MyLAN, nombre del Workspace, fecha/hora, red y estado de censura.
  2. *Resumen Ejecutivo:* Métricas globales (hosts online/offline, confiables vs desconocidos, Score de Seguridad 0-100 y velocidad reciente).
  3. *Matriz de Riesgos de Seguridad:* Hallazgos abiertos con severidad, evidencia y recomendaciones de remediación.
  4. *Inventario Técnico Detallado:* Tabla formateada de dispositivos con IP, MAC, Vendor, Tipo con % de Certeza, etiqueta Confiable y puertos/servicios abiertos.
  5. *Pie de Página:* Numeración de páginas y metadatos de auditoría defensiva.
- **Modo Censura:** Toggle explícito en el diálogo de exportación. Si está activo, enmascara todas las IPs, MACs, hostnames y SSIDs con aviso legal en el pie.

---

## 8. Pilar 7: Sincronización Serverless con Google Drive (Zero-Knowledge)

### 8.1. Autenticación OAuth 2.0 PKCE sin Servidor (RFC 8252 §7.3)
- Servidor HTTP loopback efímero local (`http://127.0.0.1:0/callback`) levantado por el kernel en un puerto dinámico libre.
- `code_verifier` (32 bytes base64url) y `code_challenge = Base64Url(SHA256(verifier))` con `code_challenge_method=S256` y `state` CSRF.
- Intercambio directo a tokens en `https://oauth2.googleapis.com/token`.

### 8.2. Almacenamiento en Google Drive `drive.appdata`
- Ámbito privado: `https://www.googleapis.com/auth/drive.appdata` (carpeta `appDataFolder`).
- Totalmente oculta y aislada de la interfaz web estándar del usuario en Google Drive.
- Organización por Workspace: `mylan_workspace_{workspace_id}.enc`.

### 8.3. Esquema Criptográfico Zero-Knowledge
- **KDF:** `argon2 = "0.5"` (Argon2id con salt aleatorio de 16B, `m_cost=65536` [64 MiB], `t_cost=3`, `p_cost=1`, derivando clave simétrica de 256 bits protegida con `zeroize`).
- **Cifrador AEAD:** `chacha20poly1305 = "0.10"` (XChaCha20-Poly1305 con nonce extendido aleatorio de 24B y tag Poly1305 de 16B).
- **Wire Format (64 bytes de overhead criptográfico):**
  - Magic `MYLN` (4B) + Versión `0x01` (1B) + KDF `0x01` (1B) + Cifrador `0x01` (1B) + Reservado (1B) + Salt (16B) + Nonce (24B) = 48 bytes header en claro.
  - Payload cifrado: Metadatos JSON + snapshot SQLite WAL consistente comprimido con `zstd`.
  - Tag Poly1305 (16B).
- **Respaldo Mnemónico:** Generación de **Frase de Recuperación de 12 palabras (BIP-39)** para emergencias.
- **Persistencia de Sesión:** Clave derivada almacenada en el Keyring nativo del SO (`keyring = "3"` en Windows Credential Manager / Secret Service).

### 8.4. Resolución de Conflictos y Frecuencia
- Control de concurrencia optimista vía cabecera `If-Match: "{etag}"` y `snapshot_epoch`.
- En caso de colisión (`412 Precondition Failed`):
  1. Generación de **backup preventivo local automático** en `.mylan/backups/conflict_{workspace_id}_{timestamp}.db`.
  2. Modal interactivo de decisión: *Adoptar versión de la nube* o *Conservar versión local y sobrescribir nube*.
- Sincronización automática periódica cada 30 minutos, al minimizar/cerrar la app, e indicador visual de estado en la barra inferior (🟢 Sincronizado, 🟡 Sincronizando, 🔴 Error, ⚪ Desconectado).

---

## 9. Plan de Implementación por Fases

| Fase | Título | Componentes Principales | Entregables |
| :--- | :--- | :--- | :--- |
| **Fase 5** | **Desktop Watch** | Tokio background task, System Tray, `tauri-plugin-autostart`, histéresis de 3 scans, tolerancia sleep/wake. | Monitoreo 24/7 en desktop sin ventanas abiertas. |
| **Fase 6** | **Desktop Alerts** | `tauri-plugin-notification`, agregación antispam, Modo Censura en notificaciones, deep-linking. | Notificaciones nativas OS para dispositivos nuevos/alterados. |
| **Fase 7** | **Network Tools** | Comandos streaming IPC, pantalla `Tools.tsx` (Ping, Traceroute, DNS, WoL, Port Checker), gráficas de latencia. | Suite de utilidades de red simple a 1 clic. |
| **Fase 8** | **Security Audit** | Motor `mylan-security`, taxonomía 4 niveles, tabla `security_findings`, Score 0-100, pestaña en `DeviceDetail`. | Auditoría de seguridad pasiva y defensiva. |
| **Fase 9** | **Speed & Health** | Módulo `speed.cloudflare.com` vía `reqwest`, fallback NDT7, API `IcmpSendEcho` (Win) / socket2 (Linux), tablas de salud. | Monitoreo continuo de salud local + test de ancho de banda. |
| **Fase 10** | **Workspaces & PDF** | Migración SQLite workspaces, conmutación automática por red, `@react-pdf/renderer` con Modo Censura. | Soporte multi-red e informes ejecutivos descargables. |
| **Fase 11** | **Serverless Sync** | OAuth 2.0 PKCE loopback, API `drive.appdata`, Argon2id + XChaCha20-Poly1305, BIP-39 mnemonic, OS Keyring. | Respaldo y sincronización en la nube personal sin servidor. |

---

*Especificación compilada y validada mediante el mapa de Wayfinding de MyLAN.*
