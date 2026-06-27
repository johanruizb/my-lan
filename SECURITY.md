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

- La API local es solo-localhost por defecto; los tokens tienen scopes.
- Los secretos (credenciales de router) se cifran en disco.
- Los escaneos tienen rate limiting; no se escanean redes públicas automáticamente.
- La base de datos local (con MACs/IPs reales) **nunca** se publica: está en `.gitignore`.
