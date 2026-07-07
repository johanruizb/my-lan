# Onboarding y feedback en background

La app no explicaba el modo censura (activado por defecto) ni la diferencia entre
descubrimiento y escaneo de puertos al usuario nuevo, y un escaneo de puertos largo
no avisaba al terminar si la ventana estaba minimizada. Decidimos que el
onboarding sea **un diálogo conciso de 3 cards** (descubrimiento, escaneo de
puertos, censura) **re-abrible desde Ajustes**, sin multi-step, y que el fin de
un escaneo de puertos emita una **notificación del SO además del toast** solo
cuando la ventana de la app no está enfocada (con fallback a toast si el permiso
de notificación está denegado).

## Considered Options

- **Wizard multi-step**: rechazado porque la app es simple y la fricción no se
  justifica; tres conceptos caben en un diálogo.
- **Notificación del SO siempre**: rechazada porque duplica al toast cuando la
  app ya está a la vista y resulta ruidosa.

## Consequences

- El diálogo de onboarding añade una tercera card explicando censura (por qué
  todo se ve borroso, cómo revelar con hover/focus, cómo desactivar) y un
  botón "Ver de nuevo" en Ajustes que limpia `localStorage["onboarding_shown"]`
  y re-abre el diálogo.
- El fin de escaneo de puertos consulta `document.hidden` / foco de la ventana:
  si la app no está enfocada, lanza una notificación OS nativa vía Tauri
  ("MyLAN: escaneo de puertos completado en <dispositivo>"); si lo está, solo
  toast. Si el permiso de notificación está denegado, cae silenciosamente al
  toast existente sin error visible.
- El descubrimiento (Dashboard) sigue avisando solo con toast, porque es una
  operación corta y el usuario normalmente la lanza estando a la vista.
- `CensuraUpgradeDialog` migra del `div` custom con `role="dialog"` manual al
  mismo `DialogContent` de Radix Dialog que usa el onboarding, heredando focus
  trap, scroll lock, Escape y restore focus automáticos. El patrón custom
  reimplementado a mano queda fuera; la app tiene un solo patrón de modal.