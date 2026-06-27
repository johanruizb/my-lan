# Base de datos OUI

Este directorio contiene el snapshot de prefijos OUI/MAC → vendor usado por
`mylan-fingerprint` para resolver el fabricante de un dispositivo a partir de su MAC.

## Fuente y licencia

- **Fuente:** registro público IEEE (MA-L / OUI), descargado desde
  `https://standards-oui.ieee.org/oui/oui.csv`.
- **Licencia:** el registro IEEE es público y redistribuible **con atribución a
  IEEE**. NO está etiquetado como CC BY-SA (eso aplicaría solo a un derivado
  curado con esa licencia, p. ej. el archivo `manuf` de Wireshark, que tiene su
  propia licencia). Esta snapshot se redistribuye bajo los términos de uso del
  registro IEEE.
- **Actualización:** refrescar el snapshot completo volviendo a descargar desde
  la URL anterior y normalizando al formato de dos columnas (ver más abajo).

## Estado

- `oui.csv` (snapshot **completo**, ~39 635 prefijos MA-L de 24-bit) — añadido
  en el Paso 6 (fingerprinting). Match por prefijo de 24 bits.

## Formato (`oui.csv`)

```csv
mac_prefix,vendor
286fb9,Nokia Shanghai Bell Co., Ltd.
e80ab9,Cisco Systems, Inc
```

- `mac_prefix`: prefijo OUI de 24-bit en **hex minúsculas sin separadores**
  (6 nibbles, p. ej. `286fb9`), extraído del campo `Assignment` del CSV IEEE.
  Solo se incluyen asignaciones MA-L (6 hex chars = 24-bit); las MA-M/MA-S
  (28/36-bit) se descartan porque el match de `mylan-fingerprint` es por prefijo
  de 24 bits (`MacAddr::oui_hex`).
- `vendor`: nombre de la organización (campo `Organization Name` del CSV IEEE),
  sin recortar.

El loader (`OuiDatabase::load_csv`) normaliza el prefijo a hex minúsculas sin
separadores para casar con `MacAddr::oui_hex`, por lo que también acepta
variantes con separadores (`AA:BB:CC`, `AA-BB-CC`) o mayúsculas.