# Ética y uso responsable

MyLAN es una herramienta de **visibilidad de red defensiva**. Estas reglas son parte del
diseño del producto, no recomendaciones opcionales.

## Reglas

- Usar **solo** en redes propias o con autorización explícita del responsable.
- **No** incluir funciones ofensivas por defecto.
- **No** automatizar la explotación de vulnerabilidades.
- **No** realizar ataques de desautenticación (deauth) Wi‑Fi.
- **No** interceptar tráfico privado sin permiso explícito.
- **No** usar ARP spoofing para bloquear dispositivos en modo normal.
- Preferir el control mediante **APIs de router/firewall**, no técnicas que rompan la red.

## Diseño seguro por defecto

- Rate limiting en los escaneos; confirmación antes de escaneos profundos.
- No se escanean redes públicas automáticamente.
- La API local escucha solo en `localhost` por defecto.
- Las credenciales de router se cifran en disco.
- La base de datos local (MACs/IPs reales) nunca se publica (está en `.gitignore`).

## Por qué importa

El escaneo de red puede ser malinterpretado o abusado. MyLAN existe para que las personas
entiendan y protejan **su propia** red. Las contribuciones que añadan capacidades
ofensivas por defecto serán rechazadas.
