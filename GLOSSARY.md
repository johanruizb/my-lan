# MyLAN

Glosario de términos del dominio de MyLAN (herramienta de inspección, diagnóstico y monitoreo de redes locales).

## Language

### Operaciones de red

**Descubrimiento**:
Barrido de la red local para encontrar hosts activos y deducir su tipo.
_Avoid_: escaneo (genérico), sondeo, scan de red.

**Escaneo de puertos**:
Sondeo puerto a puerto de un **dispositivo** concreto para ver qué **servicios** responde. Vive en el detalle del dispositivo.
_Avoid_: escaneo a secas (ambiguo con descubrimiento), port scan.

**Evaluación de riesgos**:
Análisis pasivo y defensivo de las observaciones de red para identificar configuraciones inseguras, servicios expuestos o anomalías de seguridad sin emitir tráfico agresivo.
_Avoid_: escaneo de seguridad, pentest, análisis de vulnerabilidades activo.

**Salud de red**:
Medición continua y pasiva de métricas de calidad de conexión local (latencia, jitter y pérdida de paquetes) hacia el gateway y los resolvedores DNS.
_Avoid_: speedtest local, ping test continuo.

**Speed test**:
Prueba bajo demanda de ancho de banda de subida y bajada hacia internet utilizando servidores públicos abiertos estandarizados.
_Avoid_: test de velocidad continuo, benchmark de red.

**Perfil**:
Configuración predefinida del barrido (quick/normal/deep/iot/router) que fija rango de puertos y profundidad.
_Avoid_: preset, modo de escaneo.

### Entidades y Organización

**Dispositivo**:
Un host detectado en la red local, identificado por IP primaria y, como fallback, MAC primaria.
_Avoid_: host (técnico), equipo, máquina.

**Red activa**:
La interfaz de red que la app está inspeccionando ahora mismo, con su CIDR, gateway, DNS y SSID o etiqueta de usuario.
_Avoid_: red actual, conexión.

**Workspace**:
Partición lógica de red o entorno (ej. Casa, Oficina, Lab) con su propio inventario, historial y configuración aislada.
_Avoid_: perfil de usuario, proyecto, red secundaria.

**Servicio**:
Un programa escuchando en un puerto abierto de un dispositivo (web, SSH, impresora, etc.).
_Avoid_: puerto abierto (es el estado, no la entidad), aplicación.

**Vendor**:
El fabricante de la tarjeta de red, deducido del OUI de la MAC.
_Avoid_: fabricante (en código se usa vendor), marca.

### Señales de estado

**Certeza**:
Score 0-100 que mide qué tan seguro estamos de la clasificación del tipo de un dispositivo. Lo calcula la app; más alto = clasificación más segura.
_Avoid_: confianza (reservado como raíz coloquial), score, rating, confidence.

**Confiable**:
Etiqueta **manual** del usuario que marca un dispositivo como seguro. Binaria, no la calcula la app. Vive en la edición del dispositivo y como columna/badge.
_Avoid_: trusted, marcado, verificado.

**Online**:
El dispositivo respondió en el último descubrimiento. Es opt-in como filtro: sin filtro activo se ven todos, incluyendo offline.
_Avoid_: activo (ambiguo con "activo en último scan"), conectado.

**Banner**:
Texto que un servicio abierto devuelve al conectarse; suele incluir nombre y versión del software.
_Avoid_: cabecera, respuesta.

### Modos y Seguridad

**Censura**:
Modo de la app que enmascara identificadores sensibles (IP, MAC, hostname, SSID) en la UI y en los exports para evitar compartirlos por accidente. Por defecto activada; conmutable desde Ajustes. No altera los datos en disco, solo la presentación.
_Avoid_: ofuscación, redacción, modo privado.

**Sincronización serverless**:
Mecanismo de sincronización y respaldo de datos hacia el almacenamiento en la nube del usuario (Google Drive AppData) sin requerir un servidor central de MyLAN.
_Avoid_: sincronización en la nube tradicional, backend sync.

**Cifrado Zero-Knowledge**:
Cifrado de extremo a extremo realizado localmente en el cliente con clave derivada de una contraseña maestra antes de enviar cualquier dato fuera del equipo.
_Avoid_: cifrado en tránsito, HTTPS a secas.
