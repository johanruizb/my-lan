# Contrato de Datos de Sync y Manejo de Conflictos

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cómo se define el formato del payload binario cifrado en Google Drive y la estrategia de sincronización/resolución de conflictos por Workspace?

Se debe definir:
1. Formato del paquete cifrado:
   - Header en texto plano mínimo: versión del protocolo, ID aleatorio de la instalación, salt de Argon2id, nonce/IV del cifrado AEAD.
   - Payload cifrado: metadatos del Workspace, timestamp de última sincronización, snapshot comprimido de SQLite (o changelog de operaciones).
2. Detección y resolución de conflictos:
   - Al sincronizar con Google Drive, si el archivo remoto tiene un timestamp más reciente o hash diferente al último descargado localmente: ¿cuál es la política de fusión (last-write-wins por timestamp con confirmación del usuario, o branching con copia de seguridad local del estado anterior)?
3. Experiencia de usuario en Desktop:
   - Flujo de configuración inicial: botón "Conectar con Google Drive", ventana del navegador para autorizar la cuenta, ingreso de la Contraseña Maestra de Cifrado (con advertencia de que MyLAN no la almacena y no puede recuperarla).
   - Indicador de estado de sincronización en la barra de estado (Sincronizado, Sincronizando, Error, Desconectado).

## Answer

### 1. Granularidad y Nomenclatura en Google Drive (`appDataFolder`)
- **Organización por Workspace**: Se almacena un archivo independiente por cada Workspace: `mylan_workspace_{workspace_id}.enc`.
  - Permite sincronizar, restaurar o compartir datos de un entorno específico (ej. "Casa") sin transferir ni arriesgar los datos de otros workspaces (ej. "Oficina").
  - Menor consumo de ancho de banda y menor probabilidad de conflictos entre máquinas que solo operan sobre un workspace concreto.
- **Formato del Paquete Binario Cifrado (48 bytes header + payload + 16 bytes tag)**:
  - Header en claro (48B): Magic `MYLN` (4B) + Versión de formato `0x01` (1B) + KDF `0x01` (Argon2id) + Cifrador `0x01` (XChaCha20-Poly1305) + Reservado (1B) + Salt aleatorio (16B) + Nonce aleatorio (24B).
  - Payload Cifrado: Metadatos JSON (workspace name, id, epoch timestamp, device origin) + SQLite snapshot (o delta WAL) comprimido con `zstd`.
  - Tag Poly1305 (16B): Verificación de integridad y autenticación.

### 2. Detección y Resolución de Conflictos
- **Detección Optimista**: Uso de cabeceras HTTP `If-Match: "{etag}"` y el metadato `snapshot_epoch` en `appProperties` de Google Drive.
- **Flujo ante Conflicto (`412 Precondition Failed` o ETag no coincidente)**:
  1. **Backup Preventivo Local**: MyLAN guarda automáticamente una copia snapshot de seguridad en `.mylan/backups/conflict_{workspace_id}_{timestamp}.db` antes de cualquier acción.
  2. **Modal Interactivo de Decisión**:
     - Muestra información comparativa clara: "Se detectaron cambios remotos más recientes en Google Drive (sincronizados desde otra máquina a las HH:MM)".
     - Opción 1: *Adoptar versión de la nube* (descarga y aplica el estado remoto; los datos locales anteriores quedan a salvo en el backup preventivo).
     - Opción 2: *Conservar versión local y sobrescribir nube* (fuerza la subida del snapshot local reemplazando el archivo remoto).

### 3. Gestión de Clave Maestra y Respaldo Mnemónico (BIP-39)
- **Configuración Inicial**:
  - El usuario ingresa su Contraseña Maestra de Cifrado (Zero-Knowledge).
  - La aplicación genera y muestra una **Frase de Recuperación Mnemónica de 12 palabras (BIP-39)** con opción para copiar o imprimir. Si el usuario olvida su contraseña, puede rederivar la clave maestra usando estas 12 palabras.
- **Almacenamiento Seguro para Sincronización en Segundo Plano**:
  - La clave simétrica derivada (o token de desbloqueo) se persiste en el Keyring nativo del sistema operativo (`keyring = "3"` en Windows Credential Manager / Secret Service) para permitir que los ciclos periódicos de sync en segundo plano operen silenciosamente sin solicitar la contraseña continuamente.

### 4. Frecuencia y UX en Desktop
- **Disparadores de Sincronización Automática**:
  - Intervalo periódico en segundo plano: cada 30 minutos (si hay cambios pendientes en el inventario o nuevo escaneo).
  - Evento de minimizado/cierre a la bandeja del sistema (System Tray).
- **Indicador Visual en la Barra de Estado**:
  - Ícono y tooltip con estados:
    - 🟢 *Sincronizado* (última sync hace X min).
    - 🟡 *Sincronizando...* (subiendo o descargando snapshot).
    - 🔴 *Error de sincronización* (clic para ver detalle o resolver conflicto).
    - ⚪ *Desconectado* (Google Drive no vinculado).
  - Botón interactivo "Sincronizar ahora" para forzar sincronización manual inmediata.
