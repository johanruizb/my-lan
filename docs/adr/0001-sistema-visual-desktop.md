# Sistema visual del desktop MyLAN

El frontend mezclaba tres lenguajes visuales sin regla clara (glassmorphism,
cyber-glow/radar, shadcn plano) con un primary de hue distinto por modo
(azul en claro, cyan en oscuro), tipografía declarada pero no cargada
(Outfit/Inter fantasma) y radius 0.75rem. Decidimos unificar a un sistema
**cyber-utility** plano: base shadcn con tokens, **sin glass-panel** (todo
`bg-card` sólido), **cyan como primary único** en ambos modos, **system-ui**
como tipografía honesta (sin fuentes no cargadas), **radius 0.5rem** con
densidad media (padding `p-4` base, `p-5` solo en hero) y **dark como tema por
defecto** en primera ejecución. El único efecto visual de marca que sobrevive
es el radar/ping, reservado para indicar scan activo o red activa, y desactivado
bajo `prefers-reduced-motion`.

## Considered Options

- **Mezcla intencional** (glass + glow + shadcn): rechazada por identidad
  visual inestable y drift futuro.
- **Glassmorphism como rector**: rechazada por ruido en tablas densas y peso
  visual.
- **Flat shadcn puro sin animaciones**: conservaba claridad pero perdía la
  única señal visual de "scan en curso" (radar).

## Consequences

- `glass-panel` y `hover-glow` se eliminan del CSS; las cards pasan a `bg-card`
  sólido. El radar (`radar-pulse`, `animate-ping`) se mantiene pero queda
  envuelto en `@media (prefers-reduced-motion: reduce)`.
- Cambiar el primary a cyan único exige revisar contraste del botón primario
  sobre fondo claro (saturación a ajustar en `:root`).
- Eliminar Outfit/Inter del `font-family` de `body` y dejar solo `system-ui`
  quita la mentira del CSS actual; no añade dependencias de fuentes.
- Forzar dark en primera ejecución rompe la preferencia del SO: el usuario
  puede cambiar a light desde el toggle; la elección persiste en settings.
- El nav item activo usa estilo sutil (`bg-accent` + `text-foreground` + barra
  `border-l-2 border-primary`), no `bg-primary` sólido, para distinguir
  ubicación (navegación) de acción (botones CTA primarios) en la jerarquía
  visual.
- Se añaden tokens `--success` (verde) y `--warning` (ámbar) a `:root` y `.dark`
  en `index.css`, junto al `--destructive` existente. Toast, Stat cards,
  OnlineBadge y `badge.tsx` migran sus `bg-green-50`/`bg-red-50`/`bg-green-500`
  hardcoded a esos tokens. Una paleta semántica, un punto de cambio.