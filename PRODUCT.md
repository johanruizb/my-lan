# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Homelabbers, usuarios técnicos avanzados, administradores de pequeñas oficinas (SOHO) y entusiastas de redes locales.

- **Situación:** Gestionan redes locales domésticas o de trabajo con decenas de dispositivos heterogéneos (servidores, IoT, routers, estaciones de trabajo).
- **Trabajo principal (Job to be done):** Descubrir hosts activos en su LAN, identificar servicios expuestos, detectar cambios o anomalías y auditar la seguridad de su red de forma rápida, fiable y sin depender de servicios en la nube ni pagar suscripciones abusivas.

## Product Purpose

Proporcionar una herramienta integral, local-first y de código abierto para el descubrimiento, inventario, diagnóstico y monitoreo de redes locales. El éxito significa que el usuario tiene visibilidad total e instantánea de cada dispositivo y servicio en su red, con control absoluto sobre sus propios datos y sin fricción.

## Positioning

Reemplazo open-source, local-first y sin paywalls de herramientas propietarias como Fing. A diferencia de las alternativas comerciales que bloquean funciones tras muros de pago o exigen cuentas en la nube con telemetría obligatoria, MyLAN corre localmente sobre SQLite, ofrece escaneos y dispositivos ilimitados de forma gratuita, y protege la privacidad con modo censura integrado.

## Operating Context

- **Entorno de ejecución:** Aplicación de escritorio sobre Tauri 2 (Linux y Windows; en el futuro macOS) con backend Rust e interfaz React + TypeScript + Tailwind CSS.
- **Flujos clave:**
    1. _Descubrimiento:_ barrido de hosts en la red local desde el Dashboard/inventario general (ARP, TCP ping, mDNS, SSDP).
    2. _Escaneo de puertos:_ sondeo detallado de puertos y servicios por dispositivo individual según perfiles (`quick`, `normal`, `deep`, `iot`, `router`).
    3. _Inventario y etiquetado:_ clasificación, marcado manual de dispositivos confiables y edición de metadatos persistentes en SQLite.
    4. _Diagnóstico:_ ping, traceroute y resolución DNS para resolución de problemas de conectividad.
- **Modo Censura:** Enmascaramiento activo de IPs, MACs, hostnames y SSIDs en UI y exportaciones para compartir capturas o reportes con seguridad.

## Capabilities and Constraints

- **Capacidades confirmadas:**
  - Descubrimiento de red con resolución de vendor por OUI de MAC y deducción de tipo con nivel de certeza (0-100%).
  - Escaneo de puertos por perfiles configurables y detección de banners de servicios.
  - Almacenamiento local persistente en SQLite con WAL.
  - Exportación de inventario y servicios a JSON y CSV.
  - Funcionamiento sin privilegios de administrador (`sudo`) por defecto.
  - Temas Claro y Oscuro persistidos en configuración.
- **Terminología canónica (Lenguaje Ubícuo):**
  - _Descubrimiento_ (barrido de hosts) vs _Escaneo de puertos_ (sondeo por dispositivo).
  - _Dispositivo_, _Red activa_, _Servicio_, _Vendor_, _Certeza_ (métrica calculada), _Confiable_ (etiqueta manual), _Online_ (estado de presencia), _Banner_ y _Censura_.
- **Restricciones técnicas:**
  - Frontend empaquetado en Tauri 2 con IPC asíncrono hacia comandos Rust.
  - Respetar separación estricta: descubrimiento desde Dashboard/vista general; escaneo de puertos exclusivamente desde la ficha del dispositivo.
  - Ética estricta: herramienta de diagnóstico y auditoría defensiva; sin ataques de desautenticación Wi-Fi ni explotación ofensiva automatizada.

## Brand Commitments

- **Nombre:** MyLAN.
- **Lema:** "Tu red, bajo control."
- **Voz y tono:** Técnico, preciso, transparente y profesional; sobrio y directo, sin jerga corporativa vacía.
- **Identidad visual:** Identidad y kit de marca en `design/branding/kit/` (`final-horizontal-color.svg`, `final-horizontal-light.svg`, isotipos hexagonales y tonos cyan/teal identificativos).

## Evidence on Hand

- Código fuente completo y tests en Rust (`crates/mylan-core`, `crates/mylan-db`, `crates/mylan-discovery`, `crates/mylan-fingerprint`, `crates/mylan-scanner`).
- Aplicación de escritorio funcional en `apps/desktop-tauri` (React 18, Vite, Tailwind CSS, Lucide Icons).
- Capturas de pantalla reales de la versión alpha: Ajustes, Historial de escaneo, Listado de dispositivos y Detalle de dispositivo.
- Documento de lenguaje ubicuo en `CONTEXT.md` y especificación técnica en `MyLAN_plan_open_source.md`.

## Product Principles

1. **Local-first y soberanía de datos:** La aplicación funciona completamente offline y en red local; sin cuentas obligatorias, sin telemetría forzada y con base de datos SQLite en el equipo del usuario.
2. **Cero paywalls y funciones completas:** Todas las capacidades esenciales (escaneos, historial, alertas, exportación) son y serán 100% gratuitas y sin limitaciones artificiales.
3. **Densidad y precisión técnica:** Diseñada para que los usuarios avanzados y homelabbers tengan acceso claro a métricas de red, banners de servicios y detalles exactos con alta legibilidad y rapidez de escaneo visual.
4. **Seguridad defensiva por defecto:** Funciona sin requerir privilegios de superusuario en la mayoría de operaciones y protege la privacidad con modo censura a un clic.

## Accessibility & Inclusion

- Cumplimiento de pautas WCAG 2.1 AA (contraste de texto y badges, tamaños de fuente legibles, navegación por teclado y etiquetas claras).
- Compatibilidad visual idéntica y con alto contraste tanto en modo claro como en modo oscuro.
- Soporte completo del Modo Censura en pantalla para presentaciones, streaming o capturas compartidas.
