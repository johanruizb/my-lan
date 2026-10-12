---
target: apps/desktop-tauri
total_score: 20
max_score: 40
na_heuristics:
p0_count: 1
p1_count: 1
target_identity: "file:/home/srnoob/Documentos/my-lan/apps/desktop-tauri"
timestamp: 2026-10-12T15-43-14Z
slug: apps-desktop-tauri
---
## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|:-----:|-----------|
| 1 | Visibility of System Status | 2 | Falla en actualización de fondo en Ajustes ("Error al comprobar") sin detalle ni recuperación |
| 2 | Match System / Real World | 1 | Violación severa de términos: "Escaneo de puertos" para historial, puertos cerrados como "servicios activos", "Fabricante" en vez de "Vendor" |
| 3 | User Control and Freedom | 2 | Modal "¿Descartar cambios?" agresivo en db_path; sin atajos rápidos de copiado al portapapeles |
| 4 | Consistency and Standards | 2 | IPs duplicadas como título y subtítulo; patrones de guardado dispares (auto-save vs botón); selector de tema duplicado |
| 5 | Error Prevention | 3 | Buen bloqueo de navegación sucia (`useBlocker`), pero nula prevención de conflictos entre escaneos concurrentes |
| 6 | Recognition Rather Than Recall | 2 | Truncamiento agresivo de vendors OUI y banners de servicio ("SSH-2.0-OpenSSH_10...") sin popovers ni expansión |
| 7 | Flexibility and Efficiency | 1 | Cero aceleradores desktop: sin atajos de teclado, sin acciones en lote, sin filtros avanzados en inventario |
| 8 | Aesthetic and Minimalist Design | 2 | Polución de badges `(?)` en cada etiqueta básica; 7 puertos cerrados saturan la tabla principal; enorme espacio en blanco desaprovechado |
| 9 | Error Recovery | 2 | Mensajes de error en actualizaciones y scans sin diagnóstico de red accionable |
| 10 | Help and Documentation | 3 | Glosario y onboarding presentes, pero tooltips redundantes en términos elementales de red |
| **Total** | | **20/40** | **Acceptable (50%)** |

---

## Design Specificity Verdict

- **Evaluación Heurística / LLM**: La interfaz sufre del síndrome de *"SaaS Web App enlatada en WebView"*. En lugar de sentirse como un instrumento de escritorio de precisión para homelabbers y administradores de red (con densidad de datos, inspección de subredes, visualización master-detail y atajos rápidos), la UI se constriñe artificialmente a un contenedor centrado `max-w-5xl mx-auto` rodeado de desiertos de espacio en blanco. Además, presenta violaciones directas del Lenguaje Ubícuo definido en el proyecto: la barra lateral titula "Escaneo de puertos" a una vista de sólo lectura que en realidad es un log de historial de descubrimientos, se rotula "Fabricante" en vez de "Vendor", y la tabla de detalle afirma tener "Servicios activos (8)" cuando 7 de ellos están cerrados o filtrados.
- **Escaneo Determinista**: El detector automatizado detectó **1 antipatrón** (`overused-font`, severidad `warning`, categoría `slop`) en `apps/desktop-tauri/src/index.css`: uso de `Geist Variable` como fuente principal. Geist refuerza la estética de plantilla web SaaS genérica y diluye la identidad técnica que un workbench local de red debe proyectar.
- **Overlays Visuales**: Omitidos al tratarse de una aplicación de escritorio nativa Tauri sin servidor de desarrollo web activo ni navegador headless inyectable.

---

## Overall Impression

MyLAN tiene una base técnica muy sólida y una identidad visual de marca atractiva (isologotipo Discovery Scope y radar de pulsos en el Dashboard). Sin embargo, la interfaz actual se comporta como una web móvil ampliada en desktop: desaprovecha el ancho de pantalla de monitores 1080p/1440p, dispersa la información en tarjetas con escasos datos visibles, comete errores conceptuales graves al reportar puertos cerrados como "servicios activos", y carece por completo de la ergonomía y densidad que exige un administrador de sistemas o entusiasta de homelab.

---

## What's Working

1. **Identidad de Marca Táctica y Feedback en Escaneo:** El radar animado y la estética basada en tonos cyan/teal transmiten propósito e intención técnica de monitorización.
2. **Arquitectura Defensiva de Privacidad (Modo Censura):** El `CensorshipProvider` enmascara de manera consistente IPs, MACs y hostnames en la interfaz, facilitando compartir capturas y reportes sin exponer datos sensibles.
3. **Control de Formularios y Estados Sucios:** La integración de `useBlocker` previene eficazmente la pérdida accidental de notas y cambios manuales al navegar entre vistas.

---

## Priority Issues

### [P0] Conteo Engañoso de Servicios Activos y Polución de Puertos Cerrados
- **Qué:** En `DeviceDetail.tsx`, la tarjeta se titula `"Servicios activos (8)"`, pero 7 de las 8 filas corresponden a puertos cerrados (`Cerrado`) o filtrados (`Filtrado`).
- **Por qué importa:** Presentar puertos cerrados como "servicios activos" es un error conceptual crítico que genera falsa alarma de vulnerabilidades y socava la confianza técnica en la herramienta.
- **Solución:** Filtrar la tabla principal exclusivamente a puertos con estado `open` y renombrar a `"Servicios abiertos (X)"`. Mover los puertos cerrados/filtrados a un panel colapsable secundario (`"Puertos cerrados o filtrados (Y)"`).
- **Comando sugerido:** `$impeccable distill`

### [P1] Colisión de Navegación y Lenguaje Ubícuo en `/scans`
- **Qué:** En `App.tsx`, el menú lateral titula el acceso directo como `"Escaneo de puertos"`, pero conduce a `Scans.tsx`, una tabla de auditoría titulada `"Historial (4)"`.
- **Por qué importa:** Viola la regla central de `CONTEXT.md` (el escaneo de puertos vive exclusivamente en el detalle del host). El usuario espera un espacio de escaneo interactivo y recibe un log histórico de descubrimientos.
- **Solución:** Renombrar el ítem de navegación e historial a `"Historial"` con icono `History`. Corregir además las fugas de vocabulario: `"Vendor"` (no Fabricante), `"Online"` (no En línea) y `"Último descubrimiento"` (no Último scan).
- **Comando sugerido:** `$impeccable harden`

### [P2] Layout de Escritorio Desaprovechado y Baja Densidad de Información
- **Qué:** Todas las pantallas principales están restringidas a `max-w-5xl mx-auto`, dejando enormes bandas de espacio vacío a los costados en resoluciones desktop, mientras que las tarjetas de dispositivos muestran muy pocos datos útiles.
- **Por qué importa:** En redes con 30-80 hosts, el usuario debe hacer scroll interminable y no puede comparar equipos, ver MACs ni evaluar estados de un vistazo.
- **Solución:** Extender a un layout full-width con padding controlado; habilitar una vista tabular de alta densidad (tipo matriz técnica) con cabeceras ordenables y atajos de copiado rápido.
- **Comando sugerido:** `$impeccable layout`

### [P3] Redundancia Visual y Truncamiento de Datos Críticos
- **Qué:** Las tarjetas y encabezados de dispositivos repiten la IP dos veces consecutivas cuando no hay hostname, mientras que los Vendors OUI y los banners de versión (`SSH-2.0...`) se truncan sin posibilidad de expansión.
- **Por qué importa:** Se gasta espacio en texto duplicado mientras se oculta la información forense real necesaria para identificar hardware o versiones vulnerables.
- **Solución:** Mostrar la IP una sola vez de forma prominente si no hay nombre personalizado; incorporar chips completos con popover para vendors y visor expandible para banners de servicio.
- **Comando sugerido:** `$impeccable typeset`

---

## Persona Red Flags

### Alex (Power User / Homelabber)
- **Red Flag 1:** Imposible hacer triaje rápido: en un monitor 1440p con 40 hosts solo caben 6 tarjetas en pantalla. No hay vista de lista compacta ni agrupación por subredes.
- **Red Flag 2:** Cero atajos de teclado (`/` para buscar, `Ctrl+R` para escanear), sin botones de copiado de IP/MAC a un clic, y banners truncados sin vista de inspección cruda.

### Jordan (First-Timer / Administrador SOHO)
- **Red Flag 1:** Leer `"Servicios activos (8)"` con líneas en gris y rojo le genera pánico inmediato, creyendo que su router tiene 8 puertos abiertos hacia internet.
- **Red Flag 2:** La saturación de iconos de interrogación `(?)` en cada etiqueta crea sobrecarga cognitiva y sensación de complejidad innecesaria.

### Riley (Stress Tester / Cazador de Edge Cases)
- **Red Flag 1:** Al escanear un servidor con 50 servicios abiertos, la tabla carece de paginación, filtro por protocolo (TCP/UDP) o búsqueda en vivo.
- **Red Flag 2:** Una desconexión de red durante la comprobación de actualizaciones en Ajustes deja un texto rojo permanente `"Error al comprobar"` sin opción de reintento ni diagnóstico.

---

## Minor Observations & Questions to Consider

### Observaciones Menores
1. **Interruptor de tema duplicado:** Existe un botón de cambio de tema en la barra superior y otro idéntico dentro de `Ajustes > Apariencia`.
2. **Asequibilidad confusa en tarjetas:** El chevron `>` superior derecho parece un botón individual, pero toda la tarjeta es clickeable.
3. **Colisión de color en estado de red:** El punto verde del CIDR en la barra inferior (`192.168.1.4/24`) utiliza el mismo color esmeralda que el estado Online de los dispositivos.

### Preguntas Provocativas
- *¿Por qué MyLAN se presenta como una web SaaS centrada cuando su ventaja competitiva radica en ser un banco de trabajo desktop rápido y denso para administradores locales?*
- *Si el escaneo de puertos es una operación íntima y específica por dispositivo, ¿por qué la barra lateral publicita "Escaneo de puertos" para llevar a un log pasivo de descubrimientos?*
- *¿Por qué obligamos al usuario a abrir ficha por ficha para consultar direcciones MAC y vendors, en lugar de ofrecer una matriz de triaje densa capaz de auditar 50 hosts en segundos?*
