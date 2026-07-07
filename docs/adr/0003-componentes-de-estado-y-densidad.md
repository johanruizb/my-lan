# Componentes de estado y densidad de información

El frontend trataba los estados loading/empty/error con tres patrones
distintos (EmptyState genérico, Loader2 inline, clases rojo hardcoded en el
error del Dashboard), las Stat cards usaban gradientes de color hardcodeados
fuera del sistema de tokens, y las cards de dispositivo apilaban tres badges
(tipo + online + confianza) además de un ícono grande que ya codificaba el
tipo. Decidimos unificar los tres estados en un componente **StateBanner** con
variantes por token semántico y `role="alert"`/`aria-live` consistentes,
reemplazar los gradientes de las Stat cards por **tokens semánticos**
(primary/success/warning) alineados con el design system, y dejar las cards de
dispositivo con **dos badges** (Online + Trust), ya que el ícono grande ya
comunica el tipo y ConfidenceBadge solo aparece en el detalle.

## Considered Options

- **Mantener los tres patrones de estado y alinear solo el color del error**:
  menor churn pero deja la inconsistencia estructural y de accesibilidad.
- **Cards con 3 badges o con 1 solo badge**: 3 era redundante con el ícono; 1
  (solo Trust) perdía la señal explícita de online que aporta el OnlineBadge.

## Consequences

- Las clases `border-red-300 bg-red-50 …` del error del Dashboard se eliminan;
  el color de error pasa al token `destructive` del design system.
- Las Stat cards pierden las clases `bg-gradient-to-br from-indigo-500 …` y
  pasan a fondos tono derivados de tokens; el mapeo semántico es
  detectados→primary, activos→success, nuevos→warning.
- El Badge de tipo de dispositivo (texto + ícono) se quita de la card de
  inventario; el tipo se sigue leyendo del ícono grande. ConfidenceBadge deja
  de usarse en el listado y queda reservado para DeviceDetail.