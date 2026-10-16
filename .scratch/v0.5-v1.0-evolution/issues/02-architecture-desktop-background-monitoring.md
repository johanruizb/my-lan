# Arquitectura del Monitoreo Continuo en Desktop (Tauri 2 + System Tray)

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cuál es la arquitectura de hilos, ciclo de vida y gestión de recursos para el monitoreo continuo en segundo plano dentro de `apps/desktop-tauri`?

Se debe definir:
1. Ciclo de vida y System Tray: comportamiento de la ventana principal al cerrar (minimizar a bandeja del sistema en vez de salir), menú contextual del tray (Estado, Pausar/Reanudar monitoreo, Abrir ventana, Salir) e integración con `tauri-plugin-autostart`.
2. Hilo / Tarea de fondo en Rust: cómo se orquesta la tarea asíncrona de Tokio en el proceso de Tauri para invocar el pipeline de descubrimiento de forma periódica (`interval_secs`) sin congelar la UI ni degradar la red.
3. Skip guards y supresión de falsos positivos: adaptación de la lógica de `NetworkRunner` y `cold_start` que existía en `mylan-agent` para evitar escaneos superpuestos y suprimir ráfagas de eventos `device_online`/`device_offline` cuando la máquina despierta de suspensión (sleep/wake).

## Answer

### 1. Ciclo de Vida de la Ventana y System Tray
- **Interceptación de Cierre**: Se intercepta el evento `WindowEvent::CloseRequested` en Tauri 2. Si la opción de minimizar está activa (activada por defecto en Ajustes), la ventana invoca `window.hide()` en lugar de destruir el proceso.
- **Salida Definitiva**: La aplicación sólo termina completamente cuando el usuario selecciona explícitamente "Salir" en el menú contextual del System Tray o mediante combinación de teclas de cierre forzado.
- **Menú Contextual del System Tray**:
  1. Estado informativo (no interactivo): `Monitoreando: <SSID o CIDR>` o `Monitoreo en pausa`.
  2. Separador.
  3. `Abrir MyLAN`: invoca `window.show()`, `window.unminimize()` y `window.set_focus()`.
  4. `Escanear ahora`: dispara un barrido inmediato fuera del ciclo programado.
  5. `Pausar monitoreo` / `Reanudar monitoreo`: conmuta el estado del loop sin cerrar la app.
  6. Separador.
  7. `Salir`: finaliza Tokio tasks, cierra SQLite limpiamente y sale con código 0.
- **Autoarranque en el Sistema Operativo**:
  - Integración del plugin `tauri-plugin-autostart = "2"`.
  - Configurable desde la pantalla de Ajustes: cuando está habilitado, MyLAN arranca minimizado directamente en el System Tray al iniciar sesión en Linux o Windows.

### 2. Tarea de Fondo (Tokio Task) y Gestión de Recursos
- **Orquestación en Rust**: Se instancia una tarea asíncrona de Tokio dentro de `tauri::Builder::setup` que posee un `tokio_util::sync::CancellationToken` para apagado limpio.
- **Frecuencia y Perfil**:
  - Intervalo configurable por el usuario (selector: 1m, 2m, 5m, 10m, 15m, 30m; por defecto **5 minutos**).
  - Perfil **`quick`**: únicamente barrido de presencia de hosts (ARP sweep y ping ICMP/TCP no privilegiado), evitando sondeo agresivo de puertos en cada ciclo para no saturar enlaces Wi-Fi ni consumir CPU.
- **Desacoplamiento de Base de Datos**: Cada ejecución abre una conexión SQLite en modo WAL, corre `run_scan_pipeline_with_diff` y la cierra de inmediato, permitiendo concurrencia fluida con las lecturas/escrituras de la interfaz gráfica.

### 3. Skip Guards y Tolerancia a Suspensión (Sleep / Wake)
- **Skip Guard (`NetworkRunner`)**: Si un barrido previo aún no ha finalizado (por ejemplo, por una interfaz lenta o timeout alto), el siguiente tick del temporizador omite la ejecución para evitar acumulaciones de tareas concurrentes sobre la misma red.
- **Manejo de Suspensión (Sleep/Wake)**:
  - Cuando el equipo entra en suspensión, los temporizadores se congelan. Al despertar, se detecta un salto temporal anormal ($\Delta t > 2 \times \text{interval}$) o se escucha el evento de reconexión de red del SO.
  - Se aplica un **período de gracia de 20 segundos** para permitir que la interfaz Wi-Fi/Ethernet obtenga IP y default gateway de DHCP de forma estable.
  - El primer escaneo post-reanudación se ejecuta con bandera **`cold_start = true`**, suprimiendo la emisión de eventos masivos `device_offline` y `device_online`.
- **Histéresis de Presencia**:
  - Para evitar falsas alertas por caídas transitorias de paquetes Wi-Fi o ahorro de energía en teléfonos móviles, un dispositivo sólo se declara como `offline` y se notifica tras **3 escaneos fallidos consecutivos** (15 minutos por defecto). Al primer escaneo fallido simplemente se incrementa un contador interno de omisión en memoria.
