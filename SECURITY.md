# Política de seguridad

## Reportar una vulnerabilidad

Si descubres una vulnerabilidad de seguridad en MyLAN, repórtala de forma privada
(no abras un issue público). Usa los **GitHub Security Advisories** del repositorio
(`Security > Report a vulnerability`) o contacta al mantenedor.

Incluye, si es posible:

- Versión/commit afectado.
- Pasos de reproducción.
- Impacto estimado.

Intentaremos responder en un plazo razonable y coordinar la divulgación.

## Alcance y uso responsable

MyLAN es una herramienta de **visibilidad de red defensiva**. Está pensada para usarse
únicamente en redes propias o con autorización explícita.

Por diseño, MyLAN **no** incluye:

- Ataques de desautenticación (deauth) Wi‑Fi.
- ARP spoofing / MITM / interceptación de tráfico.
- Explotación automatizada de vulnerabilidades.

El control de acceso a dispositivos se realiza mediante integraciones con
router/firewall, no con técnicas que rompan la red.

## Seguridad interna

Estado actual del MVP (v0.1+v0.2):

- Los escaneos tienen rate limiting (concurrencia acotada + timeouts); no se
  escanean redes públicas automáticamente.
- La base de datos local y las exportaciones (con MACs/IPs reales) **nunca** se
  publican: están en `.gitignore`.
- El descubrimiento es no intrusivo y degrada sin privilegios (no asume root).

Planeado (aún NO implementado en este MVP; `mylan serve` es un stub):

- API local solo-localhost con tokens con scopes.
- Cifrado en disco de secretos (p.ej. credenciales de router) para las
  integraciones de control de acceso.
