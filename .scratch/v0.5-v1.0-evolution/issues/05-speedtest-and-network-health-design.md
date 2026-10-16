# Research: Crate y Protocolos para Speed Test y Salud de Red

Type: research
Status: resolved
Blocked by: none

## Question

¿Cuáles son las opciones técnicas y crates en el ecosistema Rust más ligeras, estables y compatibles con Tokio para: (1) medición pasiva continua de salud de red (latencia, jitter y pérdida de paquetes hacia gateway y DNS sin requerir privilegios root), y (2) cliente de speed test bajo demanda contra servidores públicos abiertos (NDT7 / Measurement Lab o Ookla compatible)?

Se debe investigar y responder:
1. Métricas de salud local: cómo medir latencia precisa y jitter en Rust en Linux y Windows sin raw sockets (ej. ICMP no privilegiado vía socket `IPPROTO_ICMP` en Linux / `IcmpSendEcho` en Windows, o probes UDP/TCP rápidos hacia DNS/Gateway).
2. Protocolo de Speed Test bajo demanda: viabilidad de implementar un cliente NDT7 (protocolo abierto de Measurement Lab basado en WebSockets y BBR) vs alternativas HTTP chunked o crates existentes de Speedtest en Rust.
3. Evaluación de crates existentes: revisar crates como `speedtest-rs`, clientes NDT7 o implementaciones directas sobre `reqwest`/`tokio-tungstenite`.
4. Contrato de datos en SQLite: tabla `network_health_samples` (timestamp, rtt_ms, jitter_ms, loss_pct) y `speed_tests` (download_mbps, upload_mbps, latency_ms, server_name).

## Answer

### 1. Medición de Salud Local sin Privilegios Root
- **Linux**: Datagram sockets ICMP (`SOCK_DGRAM` con `IPPROTO_ICMP` vía `socket2`), permitidos sin root por `net.ipv4.ping_group_range`.
- **Windows**: API Win32 `IcmpSendEcho` en `iphlpapi.dll` invocada vía `windows-sys` (que ya está en el árbol de dependencias). No requiere permisos de Administrador a diferencia de `SOCK_RAW`. Se ejecuta en `tokio::task::spawn_blocking`.
- **Sondeo DNS rápido UDP (Cross-platform)**: Sondas DNS compactas de 28 bytes enviadas al puerto 53 vía `tokio::net::UdpSocket`. Todo servidor DNS responde de inmediato (`NOERROR`/`NXDOMAIN`), midiendo RTT real a nivel de paquete sin depender de ICMP.
- **Gateway Fallback**: Cadena ICMP echo -> probe DNS UDP puerto 53 -> TCP connect puerto 80/443.

### 2. Algoritmo de Jitter
- **RFC 3550 (RTP Interarrival Jitter)**: Filtro IIR exponencial continuo con factor de suavizado $\alpha = 1/16$ ($J_i = J_{i-1} + (|RTT_i - RTT_{i-1}| - J_{i-1})/16$). Estándar de la industria para tráfico VoIP y streaming.
- **Desviación Estándar ($\sigma$)**: Calculada en ráfagas de 3 a 5 pings espaciados cada 200 ms por ciclo de monitoreo.

### 3. Protocolo de Speed Test bajo demanda: Cloudflare (`speed.cloudflare.com`) como Primario
- **Motor Primario Seleccionado**: **Cloudflare Speed Test (`speed.cloudflare.com`)**.
  - Operación 100% sobre HTTP/2 estándar vía `reqwest` (sin dependencias adicionales de WebSockets).
  - Infraestructura Anycast global en más de 330 ciudades en 120+ países: peering directo con casi todos los proveedores de internet residenciales.
  - Endpoints utilizados:
    - `/cdn-cgi/trace`: Obtiene IP pública, país y código de PoP/datacenter (`colo`).
    - `/__down?bytes=0`: Mide latencia RTT y jitter sin transferencia de datos.
    - `/__down?bytes={N}`: Mide velocidad de bajada con chunks progresivos (1MB, 10MB, 25MB, 50MB).
    - `/__up`: Mide velocidad de subida vía POST con payload generado en memoria.
- **Motor Secundario / Fallback**: **NDT7 (Measurement Lab)** vía WebSockets para entornos con telemetría de kernel requerida o donde Cloudflare esté bloqueado.
- **Descarte de Ookla**: Términos de Servicio prohíben clientes no autorizados y `speedtest-rs` está desactualizado.

### 4. Modelo de Datos y Persistencia en SQLite
- Tablas creadas en migración:
  - `network_health_samples`: `id`, `network_id`, `target_type` (gateway/dns/internet), `target_ip`, `rtt_ms`, `jitter_ms`, `loss_pct`, `sample_count`, `probes_failed`, `method`, `created_at`.
  - `speed_tests`: `id`, `network_id`, `server_name`, `server_host`, `server_city`, `server_country`, `download_mbps`, `upload_mbps`, `latency_ms`, `jitter_ms`, `bufferbloat_ms`, `retransmits`, `protocol` ('cloudflare' | 'ndt7'), `created_at`.
- **Estrategia de retención**: Ventana rodante de 30 días con purga periódica automática para mantener la base de datos local ligera (~18 MB máx. en 30 días con sondeo cada 30s).
