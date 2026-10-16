# Motor de Evaluación de Riesgos y Auditoría de Seguridad Pasiva

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cómo se estructura el motor de evaluación de riesgos (`mylan-security`), la taxonomía de severidades y el catálogo de reglas de auditoría pasiva?

Se debe definir:
1. Modelo de datos del riesgo: entidad `RiskFinding` (`id`, `device_id`, `risk_code`, `title`, `description`, `severity` [Bajo, Medio, Alto, Crítico], `evidence`, `remediation_advice`, `status` [open, acknowledged, ignored]).
2. Catálogo inicial de reglas pasivas:
   - Servicios en texto plano obsoletos (Telnet 23, FTP 21, rsh/rlogin).
   - Paneles HTTP sin TLS en Gateway/Router o dispositivos de administración.
   - UPnP activo detectado en routers domésticos.
   - Puertos SMB v1/antiguos (445/139) expuestos.
   - Dispositivos de tipo cámara IP / IoT no marcados como `Confiable` con puertos RTSP (554) o HTTP abiertos.
3. Persistencia y ciclo de vida en SQLite: cuándo se recalcula la evaluación (al finalizar descubrimiento o escaneo de puertos), cómo se conservan los estados modificados por el usuario (ej. marcar un riesgo como ignorado o mitigado).

## Answer

### 1. Taxonomía de Severidad y Score Global de Seguridad
- **Taxonomía en 4 Niveles**:
  - `Critical`: Riesgos graves de compromiso inmediato (ej. Telnet 23 expuesto en Internet o dispositivo desconocido con control de acceso; interfaz de administración de router expuesta sin TLS).
  - `High`: Servicios inseguros en texto plano o puertos de gestión sin cifrado (ej. FTP 21, HTTP admin en IoT, SMB 445 en hosts no confiables).
  - `Medium`: Servicios potencialmente vulnerables o expuestos innecesariamente (ej. UPnP activo en router, SNMP v1/v2 público con community `public`, RTSP sin autenticación).
  - `Low` (Informativo): Configuraciones subóptimas o anómalas (ej. cambios frecuentes de IP en dispositivos estáticos, hostnames genéricos de fábrica sin clasificar).
- **Score Global de Seguridad de Red (0–100)**:
  - Fórmula ponderada: $\text{Score} = \max\left(0, 100 - \sum \text{penalizaciones}\right)$, donde:
    - Hallazgo Crítico abierto: $-25$ pts
    - Hallazgo Alto abierto: $-15$ pts
    - Hallazgo Medio abierto: $-5$ pts
    - Hallazgo Bajo abierto: $-1$ pt
  - Los hallazgos en estado `acknowledged` (aceptados por el usuario) o `ignored` no penalizan el score global de la red.

### 2. Disparadores de Ejecución (Triggers)
- **Modo Incremental Automático**:
  - Tras cada barrido de hosts (Descubrimiento): se evalúan reglas a nivel de red (UPnP activo en gateway, aparición de dispositivos no confiables, alteración de DNS).
  - Tras cada Escaneo de puertos: se evalúan las reglas específicas de servicios y banners para el dispositivo escaneado.
- **Auditoría Global Bajo Demanda**:
  - Botón "Re-auditar seguridad" en la interfaz que reevalúa toda la base de reglas sobre la totalidad del inventario de la red activa.

### 3. Ciclo de Vida y Persistencia en SQLite (`security_findings`)
- **Esquema de la Tabla**:
  - `id`: UUID v4 primario.
  - `device_id`: Clave foránea al dispositivo (o `NULL` si es un riesgo a nivel de Gateway/Red).
  - `network_id`: Clave foránea a la red activa.
  - `rule_id`: Código único de regla (ej. `SEC-001-TELNET-OPEN`, `SEC-005-UPNP-ROUTER`).
  - `severity`: `'low' | 'medium' | 'high' | 'critical'`.
  - `title`: Título corto descriptivo del riesgo.
  - `description`: Explicación clara en lenguaje comprensible sin jerga innecesaria.
  - `evidence`: Detalle técnico observado (puerto, banner, IP).
  - `remediation_advice`: Guía paso a paso sobre cómo mitigar o corregir el problema.
  - `status`: `'open' | 'acknowledged' | 'ignored' | 'resolved'`.
  - `first_seen_at`, `last_seen_at`, `resolved_at`: Marcas de tiempo ISO 8601.
- **Auto-Resolución**: Si en un nuevo escaneo el puerto se cerró o la condición vulnerable desapareció, el motor conmuta automáticamente el estado a `resolved` registrando `resolved_at`, conservando el registro histórico para auditoría.

### 4. Visualización en Interfaz de Usuario
- **Widget Resumen en Dashboard (`Dashboard.tsx`)**:
  - Indicador visual del Score de Seguridad (0 a 100) con código de color (Verde $\ge 85$, Amarillo $60-84$, Rojo $< 60$).
  - Desglose de hallazgos activos por severidad y acceso rápido con un clic a los problemas críticos.
- **Pestaña "Seguridad y Riesgos" en Detalle de Dispositivo (`DeviceDetail.tsx`)**:
  - Lista de tarjetas de riesgo del dispositivo con badge de severidad, evidencia técnica y panel colapsable de remediación.
  - Botones de acción rápida por hallazgo: "Aceptar riesgo" (pasa a `acknowledged`), "Ignorar" (pasa a `ignored`) o "Re-escanear dispositivo".
- **Respeto Estricto del Modo Censura**: Si está activo, las evidencias técnicas (IPs, hostnames, MACs) se enmascaran automáticamente en todas las tarjetas de riesgo y reportes.
