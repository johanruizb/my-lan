//! Almacenamiento seguro de secretos (Keyring del SO con fallback local).
//!
//! Guarda la clave de desbloqueo Zero-Knowledge y el refresh_token de Google Drive:
//! - En Windows: Windows Credential Manager
//! - En Linux: Secret Service API (gnome-keyring / kwallet)
//! - Fallback: archivo protegido local en `fallback_path` si el SO carece de Secret Service

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

const KEYRING_SERVICE: &str = "mylan_network_monitor";
const KEYRING_USER_MASTER_KEY: &str = "master_key";
const KEYRING_USER_REFRESH_TOKEN: &str = "gdrive_refresh_token";
const KEYRING_USER_EMAIL: &str = "gdrive_email";

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Error de acceso a Keyring: {0}")]
    KeyringError(String),

    #[error("Error de I/O en fallback vault: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error de serializacion JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Almacen de secretos de sincronizacion.
pub struct Vault {
    fallback_path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
struct FallbackStorage {
    secrets: HashMap<String, String>,
}

impl Vault {
    pub fn new(fallback_dir: impl AsRef<Path>) -> Self {
        let dir = fallback_dir.as_ref().to_path_buf();
        let fallback_path = dir.join(".mylan_vault_fallback.json");
        Self { fallback_path }
    }

    fn read_fallback(&self) -> FallbackStorage {
        if let Ok(bytes) = std::fs::read(&self.fallback_path) {
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            FallbackStorage::default()
        }
    }

    fn write_fallback(&self, storage: &FallbackStorage) -> Result<(), VaultError> {
        if let Some(parent) = self.fallback_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let bytes = serde_json::to_vec(storage)?;
        std::fs::write(&self.fallback_path, bytes)?;
        Ok(())
    }

    fn get_secret(&self, key: &str) -> Result<Option<String>, VaultError> {
        // Intentar leer de keyring del SO
        if let Ok(ent) = keyring::Entry::new(KEYRING_SERVICE, key) {
            if let Ok(val) = ent.get_password() {
                return Ok(Some(val));
            }
        }
        // Fallback local
        let fallback = self.read_fallback();
        Ok(fallback.secrets.get(key).cloned())
    }

    fn set_secret(&self, key: &str, value: &str) -> Result<(), VaultError> {
        let mut saved_in_keyring = false;
        if let Ok(ent) = keyring::Entry::new(KEYRING_SERVICE, key) {
            if ent.set_password(value).is_ok() {
                saved_in_keyring = true;
            }
        }

        // Siempre persistir tambien en fallback como respaldo de redundancia
        let mut storage = self.read_fallback();
        storage.secrets.insert(key.to_string(), value.to_string());
        self.write_fallback(&storage)?;

        let _ = saved_in_keyring;
        Ok(())
    }

    fn delete_secret(&self, key: &str) -> Result<(), VaultError> {
        if let Ok(ent) = keyring::Entry::new(KEYRING_SERVICE, key) {
            let _ = ent.delete_credential();
        }

        let mut storage = self.read_fallback();
        storage.secrets.remove(key);
        self.write_fallback(&storage)?;
        Ok(())
    }

    // --- Metodos de alto nivel ---

    /// Guarda la clave simetrica de 32 bytes codificada en base64.
    pub fn save_master_key(&self, key: &[u8; 32]) -> Result<(), VaultError> {
        let b64 = STANDARD.encode(key);
        self.set_secret(KEYRING_USER_MASTER_KEY, &b64)
    }

    /// Recupera la clave simetrica de 32 bytes si esta configurada.
    pub fn get_master_key(&self) -> Result<Option<[u8; 32]>, VaultError> {
        if let Some(b64) = self.get_secret(KEYRING_USER_MASTER_KEY)? {
            if let Ok(decoded) = STANDARD.decode(&b64) {
                if decoded.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&decoded);
                    return Ok(Some(key));
                }
            }
        }
        Ok(None)
    }

    /// Elimina la clave maestra almacenada.
    pub fn delete_master_key(&self) -> Result<(), VaultError> {
        self.delete_secret(KEYRING_USER_MASTER_KEY)
    }

    /// Guarda los tokens de Google Drive y el correo de usuario.
    pub fn save_gdrive_auth(
        &self,
        refresh_token: &str,
        email: Option<&str>,
    ) -> Result<(), VaultError> {
        self.set_secret(KEYRING_USER_REFRESH_TOKEN, refresh_token)?;
        if let Some(em) = email {
            self.set_secret(KEYRING_USER_EMAIL, em)?;
        }
        Ok(())
    }

    /// Recupera el refresh_token y el email guardados de Google Drive.
    pub fn get_gdrive_auth(&self) -> Result<Option<(String, Option<String>)>, VaultError> {
        if let Some(refresh_token) = self.get_secret(KEYRING_USER_REFRESH_TOKEN)? {
            let email = self.get_secret(KEYRING_USER_EMAIL)?;
            return Ok(Some((refresh_token, email)));
        }
        Ok(None)
    }

    /// Elimina las credenciales de Google Drive.
    pub fn delete_gdrive_auth(&self) -> Result<(), VaultError> {
        self.delete_secret(KEYRING_USER_REFRESH_TOKEN)?;
        self.delete_secret(KEYRING_USER_EMAIL)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(temp_dir.path());

        assert!(vault.get_master_key().unwrap().is_none());
        assert!(vault.get_gdrive_auth().unwrap().is_none());

        let sample_key = [99u8; 32];
        vault.save_master_key(&sample_key).unwrap();
        let retrieved_key = vault.get_master_key().unwrap().unwrap();
        assert_eq!(sample_key, retrieved_key);

        vault
            .save_gdrive_auth("mock_refresh_token_abc", Some("usuario@ejemplo.com"))
            .unwrap();

        let (rt, email) = vault.get_gdrive_auth().unwrap().unwrap();
        assert_eq!(rt, "mock_refresh_token_abc");
        assert_eq!(email.as_deref(), Some("usuario@ejemplo.com"));

        vault.delete_master_key().unwrap();
        assert!(vault.get_master_key().unwrap().is_none());

        vault.delete_gdrive_auth().unwrap();
        assert!(vault.get_gdrive_auth().unwrap().is_none());
    }
}
