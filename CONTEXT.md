# MyLAN

Aplicación de escritorio (Tauri) para inspeccionar la red local: descubre los
dispositivos conectados, identifica sus servicios abiertos y ayuda a entender
el estado de la red. El lenguaje ubícuo distingue **descubrimiento** (barrido
de hosts) de **escaneo de puertos** (sondeo por dispositivo) porque son
operaciones distintas con resultados distintos.

## Language

### Operaciones de red

**Descubrimiento**:
Barrido de la red local para encontrar hosts activos y deducir su tipo.
_Avoid_: escaneo (genérico), sondeo, scan de red.

**Escaneo de puertos**:
Sondeo puerto a puerto de un **dispositivo** concreto para ver qué
**servicios** responde. Vive en el detalle del dispositivo.
_Avoid_: escaneo a secas (ambiguo con descubrimiento), port scan.

**Perfil**:
Configuración predefinida del barrido (quick/normal/deep/iot/router) que fija
rango de puertos y profundidad.
_Avoid_: preset, modo de escaneo.

### Entidades

**Dispositivo**:
Un host detectado en la red local, identificado por IP primaria y, como
fallback, MAC primaria.
_Avoid_: host (técnico), equipo, máquina.

**Red activa**:
La interfaz de red que la app está inspeccionando ahora mismo, con su CIDR,
gateway, DNS y SSID o etiqueta de usuario.
_Avoid_: red actual, conexión.

**Servicio**:
Un programa escuchando en un puerto abierto de un dispositivo (web, SSH,
impresora, etc.).
_Avoid_: puerto abierto (es el estado, no la entidad), aplicación.

**Vendor**:
El fabricante de la tarjeta de red, deducido del OUI de la MAC.
_Avoid_: fabricante (en código se usa vendor), marca.

### Señales de estado

**Certeza**:
Score 0-100 que mide qué tan seguro estamos de la clasificación del tipo de un
dispositivo. Lo calcula la app; más alto = clasificación más segura.
_Avoid_: confianza (reservado como raíz coloquial), score, rating, confidence.

**Confiable**:
Etiqueta **manual** del usuario que marca un dispositivo como seguro. Binaria,
no la calcula la app. Vive en la edición del dispositivo y como columna/badge.
_Avoid_: trusted, marcado, verificado.

**Online**:
El dispositivo respondió en el último descubrimiento. Es opt-in como filtro:
sin filtro activo se ven todos, incluyendo offline.
_Avoid_: activo (ambiguo con "activo en último scan"), conectado.

**Banner**:
Texto que un servicio abierto devuelve al conectarse; suele incluir nombre y
versión del software.
_Avoid_: cabecera, respuesta.

### Modos

**Censura**:
Modo de la app que enmascara identificadores sensibles (IP, MAC, hostname,
SSID) en la UI y en los exports para evitar compartirlos por accidente. Por
defecto activada; conmutable desde Ajustes. No altera los datos en disco, solo
la presentación.
_Avoid_: ofuscación, redacción, modo privado.

## Notas

- **Descubrimiento vs escaneo de puertos** es la distinción más importante del
  dominio. La UI la respeta: el descubrimiento se lanza desde el Dashboard; el
  escaneo de puertos se lanza desde el detalle de un dispositivo.
- Los términos técnicos de red (CIDR, MAC, IP, Gateway, DNS, Hostname,
  prefix_len, default route, SSID) conservan su forma canónica; el catálogo de
  tooltips explicativos para el usuario vive en `apps/desktop-tauri/src/lib/glossary.ts`,
  no aquí. Este documento es el lenguaje ubícuo, no un glosario de usuario.