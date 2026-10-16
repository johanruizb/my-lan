# MyLAN — Guía de Marca e Identidad (Discovery Scope)

> Guía compacta para desarrolladores, diseñadores y colaboradores de MyLAN.  
> Regla fundamental: **el logotipo identifica, no decora ni explica**.

---

## 1. El Logotipo

- **Concepto:** *Discovery Scope* es una retícula táctica de escaneo de red con 4 aberturas cardinales de 18°, ondas de radio/sonar concéntricas en el cuadrante superior derecho y un host local detectado en el cuadrante inferior izquierdo. Simboliza la visibilidad en tiempo real, el descubrimiento continuo de dispositivos y la soberanía de la red local (*Tu red, bajo control*).
- **Versiones oficiales:**
  - **Horizontal (Lockup principal):** `final-horizontal.svg` / `final-horizontal-color.svg` (modo oscuro) / `final-horizontal-light.svg` (modo claro) (cabeceras, web, README).
  - **Apilado (Vertical):** `final-stacked.svg` (badges, documentación, pegatinas).
  - **Símbolo único:** `final-symbol.svg` (avatares, app docks, terminales).
  - **Corte óptico pequeño (16–32 px):** `final-symbol-small.svg` (favicon de navegador, bandeja del sistema/tray).
  - **Marca denominativa (Wordmark):** `final-wordmark.svg`.

---

## 2. Área de Seguridad y Protección (Clear Space)

Mantener siempre una zona de respeto libre de texto y otros elementos gráficos de **$1 \times X$** alrededor del logotipo en los cuatro lados:
- Para el símbolo: $X$ equivale al radio del nodo central ($r = 22\text{ px}$ en lienzo de 256 px).
- Para el lockup horizontal: $X$ equivale a la altura de las mayúsculas de *LAN* ($110\text{ px}$ en lienzo de 680 px).

---

## 3. Tamaños Mínimos de Reproducción

| Versión | Pantalla (Digital) | Impresión / Físico | Archivo recomendado |
|---|---|---|---|
| **Lockup Horizontal** | $\ge 120\text{ px}$ ancho | $\ge 32\text{ mm}$ | `final-horizontal.svg` |
| **Lockup Apilado** | $\ge 80\text{ px}$ ancho | $\ge 24\text{ mm}$ | `final-stacked.svg` |
| **Símbolo Estándar** | $\ge 32\text{ px}$ | $\ge 10\text{ mm}$ | `final-symbol.svg` |
| **Favicon / Dock / Tray** | $16\text{ px} - 32\text{ px}$ | N/A | `final-symbol-small.svg` / `favicon.ico` |

*Nota técnica:* Por debajo de 24 px, debe emplearse obligatoriamente `final-symbol-small.svg`, el cual omite el punto menor secundario y refuerza la onda de escaneo para evitar empastado de trazos.

---

## 4. Paleta Cromática

| Tono | Rol | HEX | RGB | CMYK | Pantone |
|---|---|---|---|---|---|
| **Electric Cyan** | Primario / Señal | `#06B6D4` | `6, 182, 212` | `74, 0, 14, 0` | Pantone 3115 C |
| **Deep Void Navy** | Fondo oscuro / UI | `#090D16` | `9, 13, 22` | `85, 75, 55, 75` | Pantone Black 6 C |
| **Deep Cyan (Teal)** | Primario modo claro | `#0891B2` | `8, 145, 178` | `85, 20, 20, 10` | Pantone 3145 C |
| **Sky Signal (Accent)**| Host detectado / Blip | `#38BDF8` | `56, 189, 248` | `60, 0, 0, 0` | Pantone 298 C |
| **Pure White** | Monocromo negativo | `#FFFFFF` | `255, 255, 255`| `0, 0, 0, 0` | Blanco |
| **Obsidian Black** | Monocromo positivo | `#000000` | `0, 0, 0` | `0, 0, 0, 100` | Process Black |

### Combinaciones de fondo aprobadas:
1. **Modo Oscuro (Principal de MyLAN):** Símbolo en *Electric Cyan* (`#06B6D4`) y texto en blanco sobre `#090D16`.
2. **Modo Claro:** Símbolo en *Deep Cyan* (`#0891B2`) y texto en negro sobre blanco o fondo neutro (`#F8FAFC`).
3. **Monocromo Puro:** Negro sobre fondo blanco (`final-symbol-black.svg`) o Blanco sobre fondo negro (`final-symbol-white.svg`).

---

## 5. Tipografía de Acompañamiento

- **Títulos y Display:** Inter, Geist Sans o sistema grotesco geométrico en peso *Bold* / *SemiBold*.
- **Cuerpo de texto e interfaces:** Inter, Roboto o pila de fuentes del sistema (`system-ui, sans-serif`).
- **Terminal y Datos de Red (IP, MAC, Puertos):** JetBrains Mono, Fira Code o Geist Mono.

---

## 6. Usos Incorrectos (Don'ts)

- **No deformar:** No comprimir ni estirar las proporciones horizontales/verticales.
- **No rotar:** La orientación táctica con las ondas en el cuadrante superior derecho es fija e inmutable.
- **No añadir efectos ajenos:** Prohibido el uso de sombras paralelas gruesas, brillos externos o biseles tridimensionales.
- **No alterar el espaciado interno:** No separar arbitrariamente el símbolo del texto en los lockups oficiales.
- **No recrear con texto vivo:** El texto `MyLAN` en los assets oficiales está vectorizado en curvas matemáticas puras; no escribirlo encima con fuentes no alineadas.

---

## 7. Inventario de Archivos Entregados

- **Directorio de Maestros:** `design/branding/kit/`
  - `final-symbol.svg` (Símbolo estándar)
  - `final-symbol-small.svg` (Corte óptico 16/32 px)
  - `final-symbol-color.svg` (Símbolo con color en tile oscuro `#090D16`)
  - `final-symbol-light.svg` (Símbolo con color en tile claro `#F1F5F9` con borde `#E2E8F0`)
  - `final-symbol-dark.svg` (Símbolo transparente para UI oscura)
  - `final-horizontal.svg` / `final-horizontal-color.svg` / `final-horizontal-light.svg` (Lockup horizontal)
  - `final-stacked.svg` (Lockup apilado)
  - `final-wordmark.svg` (Marca denominativa)
- **Directorio de Exportaciones:** `design/branding/kit/exports/`
  - Variantes SVG: `*-black.svg`, `*-white.svg`, `*-mono-06b6d4.svg`, `*-square.svg`, `*-app-icon.svg`
  - Renders PNG: Tamaños 32, 64, 128, 256, 512, 1024, 1200 px.
  - Set de iconos Web / PWA: `favicon.ico`, `favicon-16.png`, `favicon-32.png`, `apple-touch-icon.png`, `icon-192.png`, `icon-512.png`, `site.webmanifest`, `head-snippet.html`.
- **Tablero de Presentación y Mockups:** `design/branding/kit/presentation.html` y láminas PNG en `design/branding/kit/slides/`.
