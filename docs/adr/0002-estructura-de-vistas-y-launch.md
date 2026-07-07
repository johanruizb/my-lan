# Estructura de vistas y puntos de launch

Dashboard y Devices duplicaban función (ambos lanzaban descubrimiento y
mostraban inventario), DeviceDetail y Scans duplicaban el lanzamiento del
escaneo de puertos, y "Acerca de" ocupaba un slot de navegación permanente
para contenido que se visita una sola vez. Decidimos dividir las vistas por
**propósito**, no por granularity: el Dashboard es la puerta de entrada
(resumen + stats + el único CTA de descubrimiento, sin listado de
dispositivos), Devices es el dueño del inventario (filtros + export + refrescar,
sin botón de escaneo), DeviceDetail es el único dueño del lanzamiento del
escaneo de puertos del dispositivo abierto, y Scans queda solo como historial
con enlaces a re-abrir el detalle. "Acerca de" deja de ser ruta de navegación
y pasa a un acceso en el SidebarFooter.

## Considered Options

- **Fusionar Dashboard en Devices**: concentraba el resumen arriba del
  inventario, pero perdía la "puerta de entrada" con CTA destacado.
- **Mantener launch duplicado en ambos extremos**: cómodo para el usuario
  pero confunde sobre dónde vive cada operación y propaga estado de scan en
  dos puntos.

## Consequences

- El botón "Escanear" se quita de Devices; Devices solo conserva "Refrescar",
  filtros y export. Quien quiera re-escanear va al Dashboard.
- Scans pierde su botón de lanzar escaneo de puertos; su rol es solo mostrar
  el historial y enlazar al DeviceDetail correspondiente.
- La navegación principal queda en 4 items (Dashboard, Dispositivos, Escaneo
  de puertos, Ajustes); "Acerca de" se abre desde el footer de la sidebar.
- `ScanContext` sigue siendo global y único: la navegación entre vistas
  durante un scan sigue sin interrumpirlo, pero ahora solo hay un punto que
  lo inicia desde la UI de cada operación.