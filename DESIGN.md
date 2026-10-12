---
name: MyLAN
description: Tu red, bajo control. Sistema de diseño para el banco de trabajo táctico de descubrimiento y auditoría de redes locales.
colors:
  primary: "#0891b2"
  primary-dark: "#06b6d4"
  sky-signal: "#38bdf8"
  deep-void-navy: "#090d16"
  background-light: "#f8fafc"
  background-dark: "#080c14"
  card-light: "#ffffff"
  card-dark: "#0c1322"
  border-light: "#e2e8f0"
  border-dark: "#1b253b"
  text-light: "#020817"
  text-dark: "#f8fafc"
  muted-light: "#64748b"
  muted-dark: "#94a3b8"
  success: "#22c55e"
  warning: "#f59e0b"
  destructive: "#ef4444"
typography:
  display:
    fontFamily: '"Geist Variable", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif'
    fontSize: "1.5rem"
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: "-0.02em"
  headline:
    fontFamily: '"Geist Variable", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif'
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.35
    letterSpacing: "-0.01em"
  title:
    fontFamily: '"Geist Variable", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif'
    fontSize: "1rem"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "normal"
  body:
    fontFamily: '"Geist Variable", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif'
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: '"Geist Mono Variable", ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, monospace'
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: "0.02em"
rounded:
  sm: "4px"
  md: "8px"
  lg: "12px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  xl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  button-secondary:
    backgroundColor: "{colors.background-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  card:
    backgroundColor: "{colors.card-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.md}"
    padding: "20px"
---

# Design System: MyLAN

## Overview

**Creative North Star: "Discovery Scope — El Banco de Trabajo Táctico de Red"**

MyLAN es una herramienta de escritorio local-first y de alta fidelidad técnica creada para homelabbers, administradores de sistemas y usuarios avanzados. La interfaz rechaza deliberadamente el exceso de decoración, los degradados superfluos y el software inflado de los SaaS modernos; en su lugar, abraza la estética sobria, densa y orientada al propósito de los analizadores de sistemas de precisión (como Wireshark, htop o consolas de diagnóstico de red).

La experiencia visual se estructura en torno a una disposición horizontal amplia, pensada para pantallas de escritorio (1080p, 1440p y ultrawide), donde la visibilidad instantánea de hosts, subredes, direcciones MAC y puertos abiertos prevalece sobre el espacio vacío desaprovechado. Los identificadores sensibles y de telemetría conviven con el Modo Censura, permitiendo compartir diagnósticos y capturas con total privacidad sin romper el ritmo de trabajo.

**Key Characteristics:**
- **Claridad de instrumentación:** Densidad técnica alta con alineación numérica tabular (`tabular-nums`) para IPs, máscaras CIDR, puertos y tiempos de respuesta.
- **Soberanía y privacidad:** Tokens visuales integrados de censura para enmascarar direcciones y hostnames en un clic sin alterar la base de datos local SQLite.
- **Señalética sobria:** Paleta cromática restringida a tonos cian/teal tácticos sobre fondos oscuros profundos (*Deep Void Navy*) o claros neutros (*Pure Slate*), donde el color actúa como señal de telemetría activa y nunca como adorno.
- **Micro-interacciones táctiles:** Transiciones directas (150–200ms) con aceleradores de teclado, copiado rápido con feedback visual y ondas de sonar focalizadas en operaciones de descubrimiento.

## Colors

La paleta cromática se inspira en las pantallas de radar y analizadores de espectro electromagnético: superficies neutras de contraste calibrado sobre las que destacan emisiones luminosas en cian y señales de estado unívocas.

### Primary
- **Deep Teal / Signal Cyan** (`#0891B2` en modo claro / `#06B6D4` en modo oscuro): Utilizado para la identidad de marca, estados activos de navegación, enlaces interactivos y anillos de sonar durante el descubrimiento de la red local. Calibrado para cumplir WCAG AA (≥4.5:1) contra texto y fondos en ambos temas.

### Secondary
- **Sky Signal (Accent)** (`#38BDF8`): Utilizado como color de telemetría táctica: hosts detectados en vivo, blips de radar y marcadores de pulso en el Dashboard.

### Neutral
- **Deep Void Navy (Fondo Oscuro)** (`#090D16` / `#080C14`): Lienzo base de la interfaz en modo oscuro. Absorbe el brillo para reducir la fatiga visual en sesiones prolongadas de auditoría.
- **Dark Surface Card** (`#0C1322`): Superficie de tarjetas y contenedores en modo oscuro, con bordes nítidos de 1px en `#1B253B`.
- **Pure Slate Canvas (Fondo Claro)** (`#F8FAFC`): Lienzo base de la interfaz en modo claro, limpio y sobrio.
- **Light Surface Card** (`#FFFFFF`): Fondo de tarjetas y tablas en modo claro, con bordes de 1px en `#E2E8F0`.
- **Muted Foreground** (`#64748B` en claro / `#94A3B8` en oscuro): Metadatos secundarios, fechas relativas, etiquetas técnicas y banners inactivos.

### Semantic
- **Online / Success** (`#22C55E`): Indicador de host online o escaneo completado con éxito.
- **Warning / Unverified** (`#F59E0B`): Puertos filtrados, advertencias de configuración y hosts nuevos no categorizados.
- **Offline / Destructive** (`#EF4444`): Puertos cerrados, fallos de conectividad, errores de actualización o acciones destructivas.

### Named Rules
**The Telemetry Rarity Rule.** El color primario cian y el color de acento se reservan para ≤10% de la superficie visual en pantalla. El resto de la interfaz permanece en tonalidades neutras estructuradas para que cualquier señal activa destaque al instante.

**The Semantic State Rule.** Los badges de estado en tablas usan invariablemente variantes outline con fondos tintados al 10% (`bg-emerald-500/10 border-emerald-500/30`), reservando los fondos sólidos saturados únicamente para botones primarios o alertas críticas.

## Typography

El sistema tipográfico combina la legibilidad contemporánea de una fuente geométrica neutral para textos y títulos con una pila monoespaciada estricta optimizada para datos de red.

**Display & Body Font:** `Geist Variable`, con respaldo nativo en `system-ui`, `-apple-system`, `Segoe UI`, `Roboto`, `sans-serif`.  
**Data & Monospace Font:** `Geist Mono Variable`, con respaldo en `ui-monospace`, `SFMono-Regular`, `Menlo`, `Monaco`, `Consolas`, `monospace`.

**Character:** Tipografía austera, precisa y auto-hospedada sin dependencias externas en red, garantizando funcionamiento 100% offline. Los números en tablas y datos de red activan obligatoriamente `tabular-nums` para mantener columnas alineadas al píxel.

### Hierarchy
- **Display** (Bold 700, `1.5rem` / 24px, line-height 1.25, tracking `-0.02em`): Encabezados principales de pantalla (Dashboard, Dispositivos, Historial, Ajustes).
- **Headline** (SemiBold 600, `1.125rem` / 18px, line-height 1.35, tracking `-0.01em`): Títulos de tarjetas principales, detalle de host e información de red activa.
- **Title** (SemiBold 600, `1rem` / 16px, line-height 1.4, tracking normal): Subsecciones, cabeceras de tablas y grupos de ajustes.
- **Body** (Regular 400, `0.875rem` / 14px, line-height 1.5, tracking normal): Texto general de interfaz, descripciones, mensajes de estado y diálogos.
- **Label / Data** (Medium 500, `0.75rem` / 12px, tracking `0.02em`, `font-mono`): Direcciones IP, MACs, puertos, banners de protocolo y marcas temporales.

### Named Rules
**The Single-IP Hierarchy Rule.** En tarjetas y encabezados de detalle, cuando un dispositivo carece de nombre personalizado o hostname, la dirección IP se muestra una sola vez como título prominente; nunca debe repetirse en subtítulos o badges redundantes adyacentes.

## Layout

La interfaz abandona los contenedores estrechos centrados tipo SaaS móvil y adopta una disposición fluida para escritorio con tope en `max-w-7xl` (con escalado responsivo hasta `2xl:max-w-screen-2xl`).

- **Barra lateral izquierda (Sidebar):** Ancho fijo de 240px (en pantallas de escritorio) con acceso a Dashboard, Dispositivos, Historial y Ajustes. En el pie incluye estado de interfaz activa y versión.
- **Área principal de trabajo:** Padding de 24px (`px-6 py-6 md:px-8`), organizada en secciones verticales conectadas por un ritmo consistente de 24px (`gap-6`).
- **Cuadrículas de inventario:**
  - *Vista de tarjetas:* Grid responsivo de 1 a 3 columnas (`grid-cols-1 md:grid-cols-2 lg:grid-cols-3`) con tarjetas compactas.
  - *Vista de tabla (Matriz homelab):* Tabla densa con cabecera fija, columnas ordenables y botones de copiado rápido al paso del cursor (`hover`) o foco de teclado (`focus-visible`).

## Elevation & Depth

MyLAN es un sistema plano por defecto (*flat-by-default*). La profundidad se comunica mediante capas tonales y bordes nítidos de 1px, evitando sombras voluminosas o difusas que restan precisión técnica.

### Shadow Vocabulary
- **Subtle Surface** (`box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05)`): Utilizado únicamente en tarjetas de contenido para elevarlas sutilmente sobre el fondo base.
- **Active Sonar Glow** (`box-shadow: 0 0 15px rgba(56, 189, 248, 0.25)`): Resplandor táctico alrededor del nodo central del radar animado durante el descubrimiento de la red.

### Named Rules
**The Crisp Border Doctrine.** Las tarjetas, modales y tablas definen su elevación mediante bordes exactos de 1px (`border-border`). Quedan prohibidas las sombras gruesas decorativas o los biseles emulados.

## Shapes

- **Esquinas redondeadas:**
  - Controles pequeños, badges y botones: radio suave de 4px a 8px (`rounded-md`, 8px).
  - Tarjetas y contenedores: radio contenido de 8px a 12px (`rounded-xl`).
  - Badges de presencia y píldoras de radar: forma completamente redondeada (`rounded-full`, 9999px).
- **Indicadores de estado:** Puntos circulares de 8px de diámetro (`h-2 w-2 rounded-full`) con soporte para ondas de pulso (`animate-ping`) en operaciones en curso.

## Components

### Buttons
- **Shape:** 8px de radio (`rounded-md`).
- **Primary:** Fondo cian primario (`#0891b2` en claro / `#06b6d4` en oscuro), texto blanco/oscuro de alto contraste, padding `8px 16px`. Hover con opacidad 90%.
- **Outline / Ghost:** Borde de 1px en `#e2e8f0` / `#1b253b`, fondo transparente, hover con `bg-muted/40`.
- **CopyButton:** Botón de acción rápida cuadrado (20x20px o 24x24px), transparente en reposo, visible al pasar el ratón o navegar por teclado (`focus-visible:opacity-100`).

### Cards / Containers
- **Corner Style:** 8px a 12px (`rounded-xl`).
- **Border:** Borde de 1px continuo.
- **Internal Padding:** 16px a 24px según la jerarquía del contenedor.

### Inputs / Search Fields
- **Style:** Fondo de tarjeta, borde sutil de 1px, tipografía en 14px, radio de 8px.
- **Focus:** Anillo exterior cian de 2px con desfase de 2px (`outline: 2px solid hsl(var(--ring)); outline-offset: 2px`).

### Tables (Matriz de Hosts y Servicios)
- **Cabeceras:** Altura 40px, texto muted en 12px con mayúsculas/semi-negrita y separación clara.
- **Filas:** Altura compacta (44–48px), separadores de 1px, resaltado sutil en hover (`hover:bg-muted/30`).
- **Banners forenses (`BannerCell`):** Texto monoespaciado en 12px con truncado dinámico, botón expandible (*"Más" / "Menos"*) y copiado instantáneo al portapapeles.

### Acordeones y Divisores de Estado (`Collapsible`)
- **Separación de puertos:** Los servicios abiertos se presentan directamente en la tabla principal; los puertos cerrados y filtrados se agrupan en un acordeón secundario colapsado por defecto para evitar contaminación visual y falsas alarmas.

## Do's and Don'ts

### Do:
- **Do** utilizar siempre el Lenguaje Ubícuo canónico: *Descubrimiento* para el barrido de hosts desde el Dashboard/inventario, y *Escaneo de puertos* exclusivamente para el sondeo de un host en su vista de detalle.
- **Do** formatear todas las direcciones IP, MACs, puertos y banners con la fuente monoespaciada y la propiedad `tabular-nums`.
- **Do** incorporar botones de copiado rápido accesibles (`CopyButton`) en cada identificador de red relevante.
- **Do** respetar la relación de contraste WCAG AA (≥4.5:1 para texto normal y ≥3:1 para controles y badges).
- **Do** aislar los puertos cerrados o filtrados en el panel colapsable secundario, manteniendo la tabla principal enfocada en servicios abiertos.

### Don't:
- **Don't** llamar "servicios activos" a puertos cerrados o filtrados.
- **Don't** duplicar la dirección IP como título y subtítulo en tarjetas o encabezados cuando no exista un hostname conocido.
- **Don't** usar sombras gruesas decorativas, gradientes de texto ni desenfoques de vidrio sin propósito funcional.
- **Don't** forzar barras laterales con nombres confusos; la vista de historial de barridos debe llamarse *Historial*, no *Escaneo de puertos*.
- **Don't** usar "Fabricante" en lugar del término técnico estándar *Vendor*.
