# Naming: Certeza y Confiable

La UI usaba "Confianza" (score automático 0-100 de la clasificación del tipo de
dispositivo) y "Confiable" (etiqueta manual del usuario) con la misma raíz, lo
que confundía dos conceptos distintos: uno es una medición automática, el otro
una decisión del usuario. Decidimos renombrar el score a **Certeza** (decisión
automática, 0-100) y reservar **Confiable** para la etiqueta manual binaria del
usuario. El lenguaje ubícuo vive en `CONTEXT.md`.

## Considered Options

- **Mantener Confianza/Confiable**: raíz compartida confunde al usuario sobre
  si está viendo una medición o su propia etiqueta.
- **Unificar ambos bajo "Confianza" con sub-label auto/manual**: funde los
  conceptos y pierde la distinción que sí importa al usuario.

## Consequences

- `ConfidenceBadge` pasa a llamarse y mostrarse como "Certeza" (barra + número
  0-100, un solo componente integrado).
- `TrustBadge` / `is_trusted` se muestran como "Confiable" (etiqueta manual).
- `CONTEXT.md` registra ambos términos como entradas separadas bajo
  "Señales de estado"; el glosario de usuario en
  `apps/desktop-tauri/src/lib/glossary.ts` actualiza la clave `confianza` a
  "Certeza" y añade "Confiable".
- Es un rename de API y UI: tocar `ConfidenceBadge`, `TrustBadge`, headers de
  tabla, tooltips y los textos de Ajustes/edición. No cambia lógica de negocio.