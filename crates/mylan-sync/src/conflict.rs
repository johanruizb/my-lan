//! Deteccion y resolucion de conflictos de sincronizacion.
//!
//! Cuando un Workspace ha sido modificado tanto remotamente como localmente:
//! 1. Genera un snapshot preventivo local en el directorio de backups.
//! 2. Notifica el conflicto con metadatos de ambas versiones.
//! 3. Permite al usuario adoptar la version de la nube o conservar y sobrescribir con la local.

use crate::package::WorkspaceSnapshot;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConflictError {
    #[error("Error de serializacion/I/O creando backup preventivo: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error de serializacion JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Metadatos de un conflicto detectado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncConflict {
    pub workspace_id: String,
    pub remote_modified_at: Option<String>,
    pub remote_snapshot_epoch: Option<i64>,
    pub local_snapshot_epoch: i64,
    pub backup_path: Option<String>,
}

/// Opcion de resolucion elegida por el usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// Descarga y aplica el estado remoto de Google Drive (el local queda en backup preventivo).
    AdoptCloud,
    /// Conserva el estado local y sobrescribe el archivo en Google Drive.
    KeepLocalAndOverwrite,
}

/// Crea una copia de seguridad preventiva local antes de resolver cualquier conflicto.
pub fn create_preventive_backup(
    snapshot: &WorkspaceSnapshot,
    backups_dir: impl AsRef<Path>,
) -> Result<PathBuf, ConflictError> {
    let dir = backups_dir.as_ref();
    std::fs::create_dir_all(dir)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let workspace = safe_workspace_component(&snapshot.metadata.workspace_id);

    let bytes = snapshot.to_json_bytes().map_err(|e| match e {
        crate::package::PackageError::JsonError(je) => ConflictError::JsonError(je),
        _ => ConflictError::IoError(std::io::Error::other("Fallo de empaquetado")),
    })?;

    // `create_new` evita sobrescribir un backup previo; el nanosegundo hace que
    // una colisión sea prácticamente imposible, pero reintentamos por si acaso.
    for attempt in 0..10u32 {
        let suffix = if attempt == 0 {
            String::new()
        } else {
            format!("_{attempt}")
        };
        let path = dir.join(format!("conflict_{workspace}_{now}{suffix}.json"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(&bytes)?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(ConflictError::IoError(e)),
        }
    }
    Err(ConflictError::IoError(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "no se pudo crear un nombre de backup único",
    )))
}

/// Reduce un `workspace_id` a un componente de nombre de fichero seguro.
///
/// Sustituye cualquier carácter fuera de `[A-Za-z0-9_-]` y añade un hash FNV-1a
/// cuando hubo sustitución, de modo que `../x` y `.._x` no colisionen y nunca
/// se pueda escapar del directorio de backups.
fn safe_workspace_component(workspace_id: &str) -> String {
    let mut out = String::with_capacity(workspace_id.len());
    let mut changed = false;
    for ch in workspace_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
            changed = true;
        }
    }
    if out.is_empty() {
        out.push_str("workspace");
        changed = true;
    }
    if changed {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in workspace_id.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        out.push_str(&format!("_{hash:016x}"));
    }
    out
}

/// Evalua si existe conflicto entre el estado local y los metadatos remotos de Google Drive.
pub fn detect_conflict(
    last_synced_epoch: Option<i64>,
    local_current_epoch: i64,
    remote_epoch: Option<i64>,
) -> bool {
    match (last_synced_epoch, remote_epoch) {
        (Some(last_synced), Some(remote)) => {
            // Si la nube tiene cambios posteriores a la ultima sincronizacion
            // Y nosotros tenemos cambios locales posteriores a la ultima sincronizacion
            remote > last_synced && local_current_epoch > last_synced
        }
        (None, Some(remote)) => {
            // Primer sync en este dispositivo, pero ya existe un archivo en la nube
            // y localmente hay dispositivos/datos generados
            remote > 0 && local_current_epoch > 0
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::SnapshotMetadata;
    use mylan_core::Workspace;

    #[test]
    fn test_conflict_detection_logic() {
        // Caso 1: Sin cambios en ninguno -> false
        assert!(!detect_conflict(Some(100), 100, Some(100)));

        // Caso 2: Solo cambio en remoto -> false (se puede adoptar directamente sin conflicto)
        assert!(!detect_conflict(Some(100), 100, Some(150)));

        // Caso 3: Solo cambio en local -> false (se puede subir directamente)
        assert!(!detect_conflict(Some(100), 150, Some(100)));

        // Caso 4: Ambos cambiaron de forma divergente -> true (CONFLICCTO)
        assert!(detect_conflict(Some(100), 150, Some(160)));
    }

    #[test]
    fn test_preventive_backup_creation() {
        let temp_dir = tempfile::tempdir().unwrap();

        let snapshot = WorkspaceSnapshot {
            metadata: SnapshotMetadata {
                workspace_id: "default".to_string(),
                workspace_name: "Mi Red".to_string(),
                device_origin: "laptop".to_string(),
                app_version: "1.0.0".to_string(),
                schema_version: 8,
                snapshot_epoch: 123456789,
            },
            workspace: Workspace {
                id: "default".to_string(),
                name: "Mi Red".to_string(),
                description: None,
                icon: "home".to_string(),
                color: "#06b6d4".to_string(),
                created_at: "2026-10-16T10:00:00Z".to_string(),
                updated_at: "2026-10-16T10:00:00Z".to_string(),
            },
            networks: vec![],
            devices: vec![],
            security_findings: vec![],
            network_health_samples: vec![],
            speed_tests: vec![],
        };

        let backup_path = create_preventive_backup(&snapshot, temp_dir.path()).unwrap();
        assert!(backup_path.exists());
        let content = std::fs::read_to_string(backup_path).unwrap();
        assert!(content.contains("\"workspace_id\":\"default\""));
    }

    #[test]
    fn preventive_backup_sanitizes_workspace_id_and_never_overwrites() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut snapshot = WorkspaceSnapshot {
            metadata: SnapshotMetadata {
                workspace_id: "../../escape/evil".to_string(),
                workspace_name: "Mi Red".to_string(),
                device_origin: "laptop".to_string(),
                app_version: "1.0.0".to_string(),
                schema_version: 8,
                snapshot_epoch: 123456789,
            },
            workspace: Workspace {
                id: "../../escape/evil".to_string(),
                name: "Mi Red".to_string(),
                description: None,
                icon: "home".to_string(),
                color: "#06b6d4".to_string(),
                created_at: "2026-10-16T10:00:00Z".to_string(),
                updated_at: "2026-10-16T10:00:00Z".to_string(),
            },
            networks: vec![],
            devices: vec![],
            security_findings: vec![],
            network_health_samples: vec![],
            speed_tests: vec![],
        };

        let first = create_preventive_backup(&snapshot, temp_dir.path()).unwrap();
        assert_eq!(first.parent(), Some(temp_dir.path()));
        assert!(!first.to_string_lossy().contains(".."));

        let second = create_preventive_backup(&snapshot, temp_dir.path()).unwrap();
        assert_ne!(first, second);
        assert!(second.exists());
        assert_eq!(second.parent(), Some(temp_dir.path()));

        // Mutar el id no debe permitir salir del directorio.
        snapshot.metadata.workspace_id = "/etc/passwd".to_string();
        let third = create_preventive_backup(&snapshot, temp_dir.path()).unwrap();
        assert_eq!(third.parent(), Some(temp_dir.path()));
    }
}
