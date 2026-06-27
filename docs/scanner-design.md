# Diseño del descubrimiento y el scanner

## Descubrimiento (mylan-discovery)

Estrategia híbrida **sin root por defecto** con capacidades adicionales bajo `sudo`.

### Fase liveness (base, sin privilegios)

- Detección de interfaz por defecto, IP, MAC, gateway y CIDR (`netdev`).
- Lectura de `/proc/net/arp` (parseo de texto; vecinos ya conocidos).
- Barrido TCP-connect concurrente sobre toda la /24 (puertos sonda: 80, 443, 22, 445,
  53, 8080) con `Semaphore` (rate limiting) + timeouts. Tras el barrido se relee
  `/proc/net/arp` para capturar las MAC recién pobladas.
- ICMP no-root *best-effort* (`SOCK_DGRAM` + `IPPROTO_ICMP`, sujeto a
  `net.ipv4.ping_group_range`).
- mDNS (`mdns-sd`, limitado a la interfaz vía `enable_interface`) y SSDP
  (UDP multicast con `IP_MULTICAST_IF` apuntando a la interfaz LAN).

### Capacidades con privilegios (sudo)

- ARP sweep activo (`pnet_datalink`, requiere `CAP_NET_RAW`); el `recv` bloqueante se
  ejecuta en `spawn_blocking`.
- ICMP echo raw.

Cada técnica produce `Observation`s que un agregador deduplica por identidad
(MAC normalizada > IP). **Degradación elegante:** si falta una capacidad (p. ej. sin
`CAP_NET_RAW`), se omite sin romper el flujo base.

### Gate de cobertura (AC-7)

La cobertura sin sudo debe alcanzar **≥90%** de los hosts activos, medida contra un
ground-truth conocido (tabla DHCP del router / inventario manual / `ip neigh show` tras
un barrido activo). Es un gate duro al final del Paso 4 antes de construir capas encima.

## Scanner (mylan-scanner)

Port scan TCP-connect, perfiles:

- `quick`: top 32 puertos, timeout bajo.
- `normal`: top 100/200, banner básico.
- `deep`: top 1000 + probes específicos.

Concurrencia con `Semaphore`, timeouts configurables y cancelación de jobs. Opera solo
sobre hosts vivos (no se escanean puertos de toda la /24 en `mylan scan`, para respetar
el presupuesto de tiempo; el port scan es bajo demanda vía `mylan ports`).
