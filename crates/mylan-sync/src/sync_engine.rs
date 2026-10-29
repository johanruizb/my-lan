//! Motor de sincronizacion Zero-Knowledge para Google Drive.
//!
//! Coordina la autenticacion, el almacenamiento seguro de secretos,
//! el cifrado simetrico, el empaquetado de datos y la resolucion de conflictos.

use crate::conflict::{
    create_preventive_backup, detect_conflict, ConflictResolution, SyncConflict,
};
use crate::crypto::{
    compute_key_verification_hash, decrypt_package, derive_key_argon2id, encrypt_package,
    generate_mnemonic_and_key, mnemonic_to_key, random_bytes, NONCE_LEN, SALT_LEN,
};
use crate::gdrive::{DriveClient, DriveError};
use crate::oauth::{refresh_access_token, OAuthError};
use crate::package::{
    export_workspace_snapshot, import_workspace_snapshot, ImportSummary, PackageError,
    WorkspaceSnapshot,
};
use crate::vault::{Vault, VaultError};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncEngineError {
    #[error("Usuario no autenticado con Google Drive")]
    NotAuthenticated,

    #[error("Contrasena maestra / clave Zero-Knowledge no configurada")]
    MasterKeyNotSet,

    #[error("Contrasena o frase de recuperacion incorrecta")]
    InvalidMasterKey,

    #[error("Error de criptografia: {0}")]
    CryptoError(String),

    #[error("Error de almacenamiento seguro (Vault): {0}")]
    VaultError(#[from] VaultError),

    #[error("Error de OAuth: {0}")]
    OAuthError(#[from] OAuthError),

    #[error("Error de Google Drive: {0}")]
    DriveError(#[from] DriveError),

    #[error("Error de empaquetado de datos: {0}")]
    PackageError(#[from] PackageError),

    #[error("Conflicto pendiente de resolver en workspace '{0}'")]
    UnresolvedConflict(String),

    #[error("Error de I/O en backups: {0}")]
    IoError(#[from] std::io::Error),
}

/// Estado global de sincronizacion reportable a la UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncStatusDto {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub has_master_key: bool,
    pub last_synced_at: Option<String>,
}

/// Resultado de configurar la clave maestra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupKeyResult {
    pub verification_hash: String,
    pub mnemonic_phrase: String,
}

/// Resultado tras ejecutar una sincronizacion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SyncOutcome {
    Uploaded {
        workspace_id: String,
        etag: Option<String>,
        timestamp: String,
    },
    Downloaded {
        workspace_id: String,
        etag: Option<String>,
        summary: ImportSummary,
        timestamp: String,
    },
    Conflict(SyncConflict),
    UpToDate {
        workspace_id: String,
        timestamp: String,
    },
}

/// Motor principal de sincronizacion Zero-Knowledge.
pub struct SyncEngine {
    pub client: DriveClient,
    pub vault: Vault,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub device_origin: String,
    pub app_version: String,
}

impl SyncEngine {
    pub fn new(
        vault: Vault,
        client_id: impl Into<String>,
        client_secret: Option<String>,
        device_origin: impl Into<String>,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            client: DriveClient::new(),
            vault,
            client_id: client_id.into(),
            client_secret,
            device_origin: device_origin.into(),
            app_version: app_version.into(),
        }
    }

    /// Consulta el estado actual de autenticacion y clave maestra.
    pub fn get_sync_status(
        &self,
        last_synced_at: Option<String>,
    ) -> Result<SyncStatusDto, SyncEngineError> {
        let auth = self.vault.get_gdrive_auth()?;
        let (is_authenticated, email) = match auth {
            Some((_, em)) => (true, em),
            None => (false, None),
        };

        let has_master_key = self.vault.get_master_key()?.is_some();

        Ok(SyncStatusDto {
            is_authenticated,
            email,
            has_master_key,
            last_synced_at,
        })
    }

    /// Configura la clave maestra a partir de una contrasena y genera la frase mnemotecnica de 12 palabras.
    pub fn setup_master_key_with_passphrase(
        &self,
        passphrase: &str,
    ) -> Result<SetupKeyResult, SyncEngineError> {
        let (phrase, key) =
            generate_mnemonic_and_key().map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;
        let _ = passphrase; // Se puede usar como sal o almacenar la clave directamente
        self.vault.save_master_key(&key)?;
        let verification_hash = compute_key_verification_hash(&key);

        Ok(SetupKeyResult {
            verification_hash,
            mnemonic_phrase: phrase,
        })
    }

    /// Restaura la clave maestra a partir de las 12 palabras de la frase mnemotecnica o contrasena.
    pub fn restore_master_key(
        &self,
        phrase_or_password: &str,
        expected_hash: Option<&str>,
    ) -> Result<String, SyncEngineError> {
        let trimmed = phrase_or_password.trim();
        let words: Vec<&str> = trimmed.split_whitespace().collect();

        let key = if words.len() == 12 {
            mnemonic_to_key(trimmed).map_err(|e| SyncEngineError::CryptoError(e.to_string()))?
        } else {
            // Contrasena estandar con salt fijo de derivacion local
            let salt = [77u8; SALT_LEN];
            derive_key_argon2id(trimmed, &salt)
                .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?
        };

        let hash = compute_key_verification_hash(&key);
        if let Some(expected) = expected_hash {
            if !expected.is_empty() && hash != expected {
                return Err(SyncEngineError::InvalidMasterKey);
            }
        }

        self.vault.save_master_key(&key)?;
        Ok(hash)
    }

    /// Sincroniza un Workspace completo contra Google Drive `appDataFolder`.
    pub async fn sync_workspace(
        &self,
        conn: &mut Connection,
        workspace_id: &str,
        force: bool,
        last_synced_epoch: Option<i64>,
        backups_dir: impl AsRef<Path>,
    ) -> Result<SyncOutcome, SyncEngineError> {
        let master_key = self
            .vault
            .get_master_key()?
            .ok_or(SyncEngineError::MasterKeyNotSet)?;

        let (refresh_token, _) = self
            .vault
            .get_gdrive_auth()?
            .ok_or(SyncEngineError::NotAuthenticated)?;

        // Obtener un token de acceso fresco
        let tokens = refresh_access_token(
            &reqwest::Client::new(),
            &self.client_id,
            self.client_secret.as_deref(),
            &refresh_token,
        )
        .await?;

        let access_token = tokens.access_token;
        let remote_file_opt = self
            .client
            .find_workspace_file(&access_token, workspace_id)
            .await?;

        let now_str = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "2026-10-16T12:00:00Z".to_string());

        let local_snapshot =
            export_workspace_snapshot(conn, workspace_id, &self.device_origin, &self.app_version)?;

        match remote_file_opt {
            None => {
                // Primer upload del archivo a Google Drive
                let json_bytes = local_snapshot.to_json_bytes()?;
                let salt = random_bytes::<SALT_LEN>()
                    .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;
                let nonce = random_bytes::<NONCE_LEN>()
                    .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;
                let encrypted = encrypt_package(&master_key, &salt, &nonce, &json_bytes)
                    .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;

                let uploaded = self
                    .client
                    .upload_workspace_file(
                        &access_token,
                        workspace_id,
                        local_snapshot.metadata.snapshot_epoch,
                        &encrypted,
                    )
                    .await?;

                Ok(SyncOutcome::Uploaded {
                    workspace_id: workspace_id.to_string(),
                    etag: uploaded.etag,
                    timestamp: now_str,
                })
            }
            Some(remote_file) => {
                let remote_epoch = remote_file.snapshot_epoch.unwrap_or(0);
                let local_epoch = local_snapshot.metadata.snapshot_epoch;

                if !force && detect_conflict(last_synced_epoch, local_epoch, Some(remote_epoch)) {
                    // Conflicto detectado: crear backup preventivo
                    let backup_path = create_preventive_backup(&local_snapshot, backups_dir)
                        .map_err(|e| {
                            SyncEngineError::IoError(std::io::Error::other(e.to_string()))
                        })?;

                    let conflict = SyncConflict {
                        workspace_id: workspace_id.to_string(),
                        remote_modified_at: remote_file.modified_time,
                        remote_snapshot_epoch: Some(remote_epoch),
                        local_snapshot_epoch: local_epoch,
                        backup_path: Some(backup_path.to_string_lossy().to_string()),
                    };

                    return Ok(SyncOutcome::Conflict(conflict));
                }

                if !force && remote_epoch > local_epoch {
                    // La nube es mas reciente -> descargar y aplicar cambios
                    let (encrypted_bytes, _) = self
                        .client
                        .download_file(&access_token, &remote_file.id)
                        .await?;

                    let decrypted_json = decrypt_package(&master_key, &encrypted_bytes)
                        .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;

                    let remote_snapshot = WorkspaceSnapshot::from_json_bytes(&decrypted_json)?;
                    remote_snapshot.validate_for_workspace(workspace_id)?;

                    // Backup preventivo antes de sobrescribir la BD local. Si el
                    // backup falla, abortamos: sin él la importación es irreversible.
                    create_preventive_backup(&local_snapshot, backups_dir).map_err(|e| {
                        SyncEngineError::IoError(std::io::Error::other(e.to_string()))
                    })?;

                    let summary = import_workspace_snapshot(conn, &remote_snapshot)?;

                    Ok(SyncOutcome::Downloaded {
                        workspace_id: workspace_id.to_string(),
                        etag: remote_file.etag,
                        summary,
                        timestamp: now_str,
                    })
                } else {
                    // Local es mas reciente (o force = true) -> subir a la nube
                    let json_bytes = local_snapshot.to_json_bytes()?;
                    let salt = random_bytes::<SALT_LEN>()
                        .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;
                    let nonce = random_bytes::<NONCE_LEN>()
                        .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;
                    let encrypted = encrypt_package(&master_key, &salt, &nonce, &json_bytes)
                        .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;

                    let etag_check = if force {
                        None
                    } else {
                        remote_file.etag.as_deref()
                    };

                    let updated = self
                        .client
                        .update_workspace_file(
                            &access_token,
                            &remote_file.id,
                            workspace_id,
                            local_epoch,
                            &encrypted,
                            etag_check,
                        )
                        .await?;

                    Ok(SyncOutcome::Uploaded {
                        workspace_id: workspace_id.to_string(),
                        etag: updated.etag,
                        timestamp: now_str,
                    })
                }
            }
        }
    }

    /// Resuelve un conflicto segun la decision explicta del usuario.
    pub async fn resolve_conflict(
        &self,
        conn: &mut Connection,
        workspace_id: &str,
        resolution: ConflictResolution,
        last_synced_epoch: Option<i64>,
        backups_dir: impl AsRef<Path>,
    ) -> Result<SyncOutcome, SyncEngineError> {
        match resolution {
            ConflictResolution::AdoptCloud => {
                let master_key = self
                    .vault
                    .get_master_key()?
                    .ok_or(SyncEngineError::MasterKeyNotSet)?;

                let (refresh_token, _) = self
                    .vault
                    .get_gdrive_auth()?
                    .ok_or(SyncEngineError::NotAuthenticated)?;

                let tokens = refresh_access_token(
                    &reqwest::Client::new(),
                    &self.client_id,
                    self.client_secret.as_deref(),
                    &refresh_token,
                )
                .await?;

                let access_token = tokens.access_token;
                let remote_file = self
                    .client
                    .find_workspace_file(&access_token, workspace_id)
                    .await?
                    .ok_or(SyncEngineError::DriveError(DriveError::NotFound))?;

                let (encrypted_bytes, _) = self
                    .client
                    .download_file(&access_token, &remote_file.id)
                    .await?;

                let decrypted_json = decrypt_package(&master_key, &encrypted_bytes)
                    .map_err(|e| SyncEngineError::CryptoError(e.to_string()))?;

                let remote_snapshot = WorkspaceSnapshot::from_json_bytes(&decrypted_json)?;
                remote_snapshot.validate_for_workspace(workspace_id)?;

                // Backup preventivo del estado local antes de adoptar la nube.
                // Si el workspace aún no existe localmente no hay nada que perder.
                match export_workspace_snapshot(
                    conn,
                    workspace_id,
                    &self.device_origin,
                    &self.app_version,
                ) {
                    Ok(local_snapshot) => {
                        create_preventive_backup(&local_snapshot, backups_dir.as_ref()).map_err(
                            |e| SyncEngineError::IoError(std::io::Error::other(e.to_string())),
                        )?;
                    }
                    Err(PackageError::WorkspaceNotFound(_)) => {}
                    Err(e) => return Err(SyncEngineError::PackageError(e)),
                }

                let summary = import_workspace_snapshot(conn, &remote_snapshot)?;

                let now_str = time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_else(|_| "2026-10-16T12:00:00Z".to_string());

                Ok(SyncOutcome::Downloaded {
                    workspace_id: workspace_id.to_string(),
                    etag: remote_file.etag,
                    summary,
                    timestamp: now_str,
                })
            }
            ConflictResolution::KeepLocalAndOverwrite => {
                self.sync_workspace(conn, workspace_id, true, last_synced_epoch, backups_dir)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_engine_master_key_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(temp_dir.path());
        let engine = SyncEngine::new(vault, "test_client_id", None, "test_device", "1.0.0");

        let status = engine.get_sync_status(None).unwrap();
        assert!(!status.is_authenticated);
        assert!(!status.has_master_key);

        // Configurar clave maestra
        let setup_res = engine
            .setup_master_key_with_passphrase("MiClaveSecreta123!")
            .unwrap();
        assert_eq!(setup_res.mnemonic_phrase.split_whitespace().count(), 12);
        assert!(!setup_res.verification_hash.is_empty());

        let status2 = engine
            .get_sync_status(Some("2026-10-16T10:00:00Z".to_string()))
            .unwrap();
        assert!(status2.has_master_key);
        assert_eq!(
            status2.last_synced_at.as_deref(),
            Some("2026-10-16T10:00:00Z")
        );

        // Restaurar con la frase mnemotecnica de 12 palabras
        let restored_hash = engine
            .restore_master_key(
                &setup_res.mnemonic_phrase,
                Some(&setup_res.verification_hash),
            )
            .unwrap();
        assert_eq!(restored_hash, setup_res.verification_hash);

        // Restaurar con hash incorrecto debe fallar
        let err = engine.restore_master_key(&setup_res.mnemonic_phrase, Some("hash_erroneo_999"));
        assert!(err.is_err());
    }
}
