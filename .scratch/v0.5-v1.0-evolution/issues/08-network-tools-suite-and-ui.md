# Suite de Herramientas de Red Comunes y UI Interactiva

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cómo se estructura la suite de "Herramientas de Red" (Ping continuo/ráfagas con gráfico, Traceroute visual con saltos geolocalizados, DNS Lookup multidominio, Wake-on-LAN y Port Checker rápido) en el backend de Tauri 2 y cómo se expone en una nueva pantalla (`Tools.tsx`) de forma simple, visual y accesible?

Se debe definir:
1. Catálogo de utilidades de red a integrar:
   - **Ping Interactivo**: Envío de paquetes con visualización en tiempo real de RTT, jitter, pérdida y gráfica de latencia en vivo.
   - **Traceroute Visual**: Traza de saltos (hops) mostrando IP, latencia por salto, hostname resuelto y ASN/país aproximado.
   - **DNS Lookup**: Resolución de registros DNS comunes (`A`, `AAAA`, `MX`, `TXT`, `CNAME`, `NS`, `PTR`) permitiendo alternar servidores DNS (sistema, Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9).
   - **Wake-on-LAN (WoL)**: Envío de magic packets UDP (puerto 9) a la dirección MAC seleccionada (integrado con la lista de dispositivos descubiertos o ingreso manual).
   - **Port Checker rápido**: Verificación de conectividad a puertos específicos sin requerir un barrido completo de perfil.
   - **Speed Test**: Acceso directo al test de velocidad (vía Cloudflare / NDT7) desde la misma suite de herramientas.
2. Contratos IPC en Tauri 2:
   - Comandos streaming y eventos (`app.emit("tool-output", payload)`) para que Ping y Traceroute reporten datos progresivamente a medida que cada paquete/hop responde, en lugar de bloquear la UI hasta que termine el comando completo.
3. Diseño de la interfaz (`Tools.tsx`):
   - Navegación: pestaña en el menú principal lateral/superior junto a Dispositivos, Escaneos y Ajustes.
   - Selección por tarjetas o pestañas simples con inputs intuitivos (ej. "Ingresa IP o dominio"), pre-poblado rápido desde el detalle de cualquier dispositivo ("Hacer ping a este dispositivo", "Traceroute", "Enviar WoL").
   - Respeto estricto del **Modo Censura** en los resultados mostrados en pantalla.

## Answer

### 1. Catálogo de Utilidades y Navegación en `Tools.tsx`
- **Acceso Global**: Nueva opción "Herramientas" en la barra de navegación principal de la aplicación.
- **Pestañas Horizontales Superiores**:
  1. **Ping**: Latencia continua o ráfagas (10, 50, infinito) con selector de intervalo y tamaño de buffer.
  2. **Traceroute**: Trazado de ruta hacia un host con resolución inversa DNS de cada salto intermedio.
  3. **DNS Lookup**: Consulta de registros (`A`, `AAAA`, `MX`, `TXT`, `CNAME`, `NS`, `PTR`, `SOA`) con opción de elegir el resolver (DNS del sistema, Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9).
  4. **Wake-on-LAN (WoL)**: Emisión de paquete mágico UDP (puerto 9) a una dirección MAC con broadcast a la red activa (`255.255.255.255` o broadcast de subred).
  5. **Port Checker**: Verificación rápida de puertos individuales o listas cortas (ej. 22, 80, 443, 8080) hacia una IP o dominio sin lanzar un escaneo masivo.
  6. **Speed Test**: Acceso integrado al motor Cloudflare (`speed.cloudflare.com`) con inicio a un clic.

### 2. Streaming IPC Reactivo y Cancelación en Tauri 2
- **Contratos de Comandos**:
  - `start_tool_ping(target: String, count: Option<u32>, interval_ms: u64, session_id: String)`
  - `start_tool_traceroute(target: String, max_hops: u8, timeout_ms: u64, session_id: String)`
  - `cancel_tool(session_id: String)`
- **Flujo de Eventos**:
  - Rust registra un `tokio_util::sync::CancellationToken` asociado al `session_id`.
  - Cada respuesta de ping o salto de traceroute emite un evento `app.emit("tool_stream", ToolEventPacket)`.
  - La interfaz en React escucha este canal y actualiza la gráfica y la tabla de resultados paquete a paquete sin bloquear la ventana.
  - Al pulsar "Detener" o cambiar de pantalla, se invoca `cancel_tool(session_id)` abortando inmediatamente el socket en Rust.

### 3. Puntos de Integración Rápida ("1 Clic")
- **Desde la Ficha de Dispositivo (`DeviceDetail.tsx`)**:
  - Barra de acciones rápidas con botones: `Ping`, `Traceroute`, `Comprobar Puertos`, `Wake-on-LAN`.
  - Al pulsar cualquiera de ellos, la aplicación navega a `Tools.tsx`, selecciona la pestaña correspondiente, pre-carga la IP o MAC del dispositivo y arranca la herramienta automáticamente.
- **Desde la Tabla General de Inventario (`Devices.tsx`)**:
  - Menú contextual mediante clic derecho sobre cualquier fila de dispositivo con las mismas acciones rápidas.

### 4. Visualización Gráfica y Modo Censura
- **Visualización de Ping**:
  - Gráfica sparkline / línea temporal en tiempo real de los últimos N paquetes con tooltip interactivo de RTT.
  - Tarjetas de resumen en la parte superior: `RTT Actual`, `RTT Mínimo`, `RTT Máximo`, `Jitter RFC 3550` y `% Pérdida de paquetes`.
- **Visualización de Traceroute**:
  - Diagrama visual vertical interactivo con nodos numerados por salto (*hop*).
  - Semáforo de latencia por nodo: Verde ($< 30$ ms), Amarillo ($30-100$ ms), Rojo ($> 100$ ms o Timeout `*`).
- **Respeto Estricto del Modo Censura**:
  - Si el Modo Censura está habilitado en la app, todas las IPs, hostnames y direcciones MAC ingresadas o mostradas en los resultados de `Tools.tsx` se enmascaran automáticamente en pantalla (ej. `192.168.1.***`, `a4:83:e7:**:**:**`).
