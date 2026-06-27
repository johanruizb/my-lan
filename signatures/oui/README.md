# Base de datos OUI

Este directorio contiene el snapshot de prefijos OUI/MAC → vendor usado por
`mylan-fingerprint` para resolver el fabricante de un dispositivo a partir de su MAC.

## Fuente y licencia

- **Fuente:** registro público IEEE (MA-L / OUI). El registro IEEE es público y
  redistribuible **con atribución a IEEE**.
- **Importante:** NO etiquetar este dato como CC BY-SA salvo que se use un derivado
  curado con esa licencia (p. ej. el archivo `manuf` de Wireshark, que tiene su propia
  licencia). Documenta aquí la fuente exacta del `oui.csv` que se incluya.

## Estado

- `oui.csv` (snapshot **completo**) se añade en el **Paso 6** (fingerprinting).
- En este push (Paso 1) solo existe esta nota; aún no hay datos.

## Formato esperado (`oui.csv`)

```csv
mac_prefix,vendor
AA:BB:CC,Example Vendor Inc.
```

El match se hace por prefijo de 24 bits (OUI). Refrescar el snapshot completo desde la
fuente IEEE según se documente en el script de actualización.
