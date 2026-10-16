# Contrato y Reglas de Notificaciones Nativas de Escritorio

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cómo se modela el contrato de eventos, el filtrado y la máquina de reglas de notificaciones de escritorio nativas (`tauri-plugin-notification`)?

Se debe definir:
1. Catálogo de disparadores de alerta:
   - Dispositivo nuevo detectado que no está marcado como `Confiable`.
   - Cambio de IP o MAC en un dispositivo marcado como `Confiable` (posible spoofing o reasignación DHCP).
   - Caída o recuperación del Gateway o conectividad a Internet.
   - Apertura de puertos sensibles/críticos en dispositivos de la red.
2. Throttling y antispam: cómo evitar saturar al usuario con decenas de notificaciones simultáneas cuando se conecta a una red grande o cuando varios dispositivos cambian de estado al mismo tiempo.
3. Permisos y configuración en UI: panel de ajustes en React para activar/desactivar tipos específicos de notificaciones y niveles de severidad sonora/visual.

## Answer

### 1. Catálogo de Disparadores de Alerta
- **`device_new_untrusted`**: Dispositivo nuevo detectado que no está en la base de datos o no tiene la etiqueta manual `Confiable`.
  - Título: `Nuevo dispositivo detectado`
  - Cuerpo: `<IP> • <Vendor o Hostname>`
- **`device_identity_tampered`**: Cambio de IP o MAC en un dispositivo que el usuario marcó previamente como `Confiable` (alerta de suplantación/ARP spoofing o cambio de arrendamiento DHCP).
  - Título: `Advertencia: Cambio de identidad en dispositivo confiable`
  - Cuerpo: `<Nombre/Etiqueta> cambió de IP a <Nueva IP>`
- **`gateway_down` / `internet_down`**: Caída del default gateway local o pérdida de resolución/conectividad hacia Internet.
  - Título: `Alerta de Red: Conectividad interrumpida`
  - Cuerpo: `El gateway o la conexión a Internet no responden`
- **`port_critical_opened`**: Detección de puertos de alto riesgo recién abiertos (ej. Telnet 23, SMB 445 expuesto, FTP 21) tras un escaneo de puertos.

### 2. Agrupación Inteligente y Throttling Antispam
- **Ventana de Consolidación por Escaneo**:
  - Si un ciclo de monitoreo genera **1 o 2 alertas**: se emiten notificaciones nativas individuales e informativas.
  - Si un ciclo de monitoreo genera **3 o más alertas simultáneas**: el motor suprime los avisos individuales y emite una única **notificación resumen** consolidada (ej. `MyLAN: Se detectaron 5 nuevos dispositivos en la red` o `3 dispositivos confiables cambiaron de estado`).
- **Supresión en Cold Start**: Durante el primer barrido de una red recién conectada (`cold_start = true`), las alertas de dispositivos nuevos se omiten por completo para no inundar el centro de notificaciones con decenas de ventanas emergentes.

### 3. Respeto Estricto del Modo Censura
- Si el **Modo Censura** está activado en la aplicación, el formateador de notificaciones enmascara obligatoriamente todas las IPs, MACs, hostnames y SSIDs en el texto enviado al sistema operativo (ej. `Nuevo dispositivo: 192.168.1.*** (Apple Inc.)`).
- Esto previene fugas de información accidental en grabaciones de pantalla, transmisiones, reuniones compartidas o historiales del Centro de Notificaciones de Windows/Linux.

### 4. Deep Linking al Clic
- Al hacer clic sobre cualquier notificación del SO:
  1. Se invoca `window.show()`, `window.unminimize()` y `window.set_focus()`.
  2. Si la notificación pertenece a un dispositivo (`device_id`), la interfaz navega directamente a la pantalla `DeviceDetail` de dicho dispositivo.
  3. Si la notificación es un resumen o alerta general de red/gateway, la interfaz navega a la pantalla de Historial de Escaneos (`Scans`).

### 5. Configuración en la Interfaz de Usuario (`Settings.tsx`)
- Integración de `tauri-plugin-notification = "2"`.
- Interruptor Maestro: `Permitir notificaciones de escritorio` (con verificación y solicitud de permisos al SO).
- Toggles individuales por categoría:
  - `Notificar cuando aparezcan dispositivos desconocidos` (Activado por defecto).
  - `Notificar cambios de IP/MAC en dispositivos confiables` (Activado por defecto).
  - `Notificar caída o recuperación de Internet y Gateway` (Activado por defecto).
  - `Notificar puertos críticos abiertos` (Activado por defecto).
