# Research: Sincronización Serverless con Google Drive (Zero-Knowledge)

Type: research
Status: resolved
Blocked by: none

## Question

¿Cómo estructurar la autenticación OAuth 2.0 PKCE (RFC 8252) sin backend en Tauri 2, el acceso al scope privado `drive.appdata` de Google Drive y el esquema criptográfico Zero-Knowledge (Argon2id + AEAD XChaCha20-Poly1305 / AES-256-GCM) para sincronizar snapshots/deltas de SQLite de forma segura?

Se debe investigar y responder:
1. Flujo OAuth 2.0 PKCE en Tauri 2: cómo manejar el loopback local (`http://127.0.0.1:<port>/callback`) o custom protocol scheme (`mylan://oauth-callback`) en Windows y Linux sin exponer secretos.
2. Uso del scope `https://www.googleapis.com/auth/drive.appdata`: operaciones de API necesarias (listar archivos, subir archivo cifrado, descargar archivo, control de versiones vía metadatos o hashes).
3. Stack criptográfico en Rust: crates recomendadas (ej. `argon2`, `chacha20poly1305` o `aes-gcm`, `zeroize`) para derivación de clave desde passphrase y cifrado autenticado de los datos antes de la llamada de red.
4. Gestión de cuotas y token refresh en cliente: cómo almacenar tokens de refresco de forma segura en el sistema (ej. keyring del SO o SQLite cifrada).

## Answer

### 1. Autenticación OAuth 2.0 PKCE (RFC 8252)
- **Servidor HTTP Loopback Efímero (`http://127.0.0.1:0/callback`)**: Es la arquitectura recomendada por Google y el RFC 8252 §7.3 para apps nativas de escritorio. El kernel asigna dinámicamente un puerto libre disponible (`:0`), evitando conflictos de puertos y registro de esquemas de URL en el registro de Windows o `.desktop` en Linux.
- Se genera un `code_verifier` criptográfico (32 bytes aleatorios base64url) y se deriva el `code_challenge = Base64Url(SHA256(verifier))` con `code_challenge_method=S256`.
- El listener loopback local recibe la redirección GET de Google con `code` y `state` CSRF, responde con HTML amigable (`HTTP 200 OK`) y se cierra de inmediato. El cliente Rust intercambia el código mediante POST directo a `https://oauth2.googleapis.com/token`.

### 2. Uso del scope privado `drive.appdata`
- Ámbito: `https://www.googleapis.com/auth/drive.appdata`.
- Permite almacenar archivos en la carpeta especial `appDataFolder`. Esta carpeta está totalmente aislada y oculta de la interfaz web estándar del usuario en Google Drive (no contamina sus carpetas ni búsquedas). MyLAN solo puede leer y escribir sus propios archivos.
- Operaciones REST:
  - Búsqueda: `GET /drive/v3/files?spaces=appDataFolder&q=name='mylan_backup.enc' and trashed=false`.
  - Creación: `POST /upload/drive/v3/files?uploadType=multipart` con `parents: ["appDataFolder"]`.
  - Actualización con control de concurrencia: `PATCH /upload/drive/v3/files/{fileId}?uploadType=multipart` enviando cabecera `If-Match: "{etag}"` (devuelve `412 Precondition Failed` si otro dispositivo subió un snapshot más reciente mientras este estaba desconectado).
  - Descarga: `GET /drive/v3/files/{fileId}?alt=media` con `If-None-Match`.

### 3. Esquema Criptográfico Zero-Knowledge
- **Derivación de clave (KDF)**: `argon2 = "0.5"` (Argon2id con salt aleatorio de 16 bytes, `m_cost=65536` [64 MiB], `t_cost=3`, `p_cost=1`, derivando 32 bytes de clave simétrica protegida con `zeroize`).
- **Cifrador AEAD**: `chacha20poly1305 = "0.10"` (XChaCha20-Poly1305). Elegido sobre AES-GCM porque su nonce extendido de 192 bits (24 bytes) garantiza una probabilidad de colisión prácticamente nula ($2^{-96}$) al generarlo aleatoriamente en múltiples clientes desconectados, sin necesidad de coordinar contadores secuenciales de nonces.
- **Formato del binario cifrado**:
  - Encabezado (48 bytes): Magic bytes `MYLN` (4B) + versión/algoritmos (4B) + salt Argon2id (16B) + nonce XChaCha20 (24B).
  - Payload: Datos SQLite comprimidos cifrados + Auth tag Poly1305 (16B).

### 4. Almacenamiento seguro del Refresh Token
- Estrategia híbrida:
  - Primario: Crate `keyring = "3"` (Windows Credential Manager en Windows; Secret Service / D-Bus en Linux).
  - Fallback: Si el usuario corre en entornos Linux minimalistas sin Secret Service (ej. i3/Sway), se persiste en SQLite cifrado localmente.

### 5. Snapshot SQLite en caliente
- Uso de la API de backup en caliente de `rusqlite` (`features = ["bundled", "backup"]`) para generar snapshots consistentes en caliente (WAL-consistent) sin bloquear operaciones concurrentes de lectura/escritura de la UI.
