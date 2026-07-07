# Edición híbrida en DeviceDetail

DeviceDetail exponía los tres campos editables (`display_name`, `is_trusted`,
`notes`) siempre visibles como inputs con un botón "Guardar cambios" global,
mezclando una acción binaria inmediata (marcar confiable) con una edición
deliberada con commit (nombre, notas). Decidimos un patrón **híbrido por campo**:
`is_trusted` pasa a ser un switch/toggle siempre visible de cambio inmediato, y
`display_name` + `notes` pasan a un modo edición (lectura por defecto, botón
"Editar" revela los inputs + "Guardar"/"Cancelar").

## Considered Options

- **Todo siempre editable** (patrón actual): simple, pero ruido permanente de
  inputs y edición sin intent explícito.
- **Todo en modo edición** (un botón "Editar" revela los tres campos): consistente,
  pero el toggle de confianza pierde inmediatez y obliga dos clicks para una
  acción binaria.

## Consequences

- El switch de `is_trusted` hace el cambio inmediato (persiste al toggle), sin
  botón Guardar intermedio. Alinea el toggle de censura en Ajustes al mismo
  patrón de switch (decisión de sistema visual).
- `display_name` y `notes` mantienen commit explícito (Guardar/Cancelar) porque
  son texto libre; el modo lectura evita editar por accidente.
- El botón "Guardar cambios" global desaparece; cada sub-patrón gestiona su
  propio commit. El estado `saving` se reparte entre el switch (cambio rápido)
  y el Guardar del modo edición.