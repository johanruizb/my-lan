//! Cliente REST para la API de Google Drive (`drive.appdata` / `appDataFolder`).
//!
//! Almacena y recupera paquetes cifrados por Workspace (`mylan_workspace_{id}.enc`)
//! de forma aislada e invisible para el usuario en su Google Drive personal.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub const GDRIVE_FILES_API: &str = "https://www.googleapis.com/drive/v3/files";
pub const GDRIVE_UPLOAD_API: &str = "https://www.googleapis.com/upload/drive/v3/files";

#[derive(Error, Debug)]
pub enum DriveError {
    #[error("Error de conexion de red con Google Drive: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Token no autorizado o expirado (401 Unauthorized)")]
    Unauthorized,

    #[error("Conflicto de concurrencia detectado (412 Precondition Failed / ETag desactualizado)")]
    ConflictPreconditionFailed,

    #[error("Archivo no encontrado en Google Drive (404 Not Found)")]
    NotFound,

    #[error("Error de API de Google Drive: codigo {status}, mensaje: {body}")]
    ApiError { status: u16, body: String },

    #[error("Error de serializacion/deserializacion JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Metadatos de un archivo en `appDataFolder`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub modified_time: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub snapshot_epoch: Option<i64>,
}

#[derive(Deserialize)]
struct FileListResponse {
    #[serde(default)]
    files: Vec<RawDriveFile>,
}

#[derive(Deserialize)]
struct RawDriveFile {
    id: String,
    name: String,
    #[serde(rename = "modifiedTime")]
    modified_time: Option<String>,
    size: Option<String>,
    #[serde(default)]
    etag: Option<String>,
    #[serde(rename = "appProperties", default)]
    app_properties: Option<HashMap<String, String>>,
}

/// Escapa un literal para la gramática de consultas de Google Drive.
///
/// Drive usa comillas simples para strings; una comilla o barra invertida sin
/// escapar permite ampliar la consulta (`or name contains ...`) y seleccionar
/// el fichero de otro workspace.
fn escape_query_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Cliente de Google Drive para almacenamiento de paquetes de sincronizacion.
#[derive(Clone, Default)]
pub struct DriveClient {
    client: reqwest::Client,
}

impl DriveClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Nombre del archivo en Google Drive para un Workspace dado.
    pub fn workspace_filename(workspace_id: &str) -> String {
        format!("mylan_workspace_{workspace_id}.enc")
    }

    /// Busca el archivo cifrado correspondiente a un Workspace en `appDataFolder`.
    pub async fn find_workspace_file(
        &self,
        access_token: &str,
        workspace_id: &str,
    ) -> Result<Option<DriveFile>, DriveError> {
        let filename = Self::workspace_filename(workspace_id);
        let q = format!(
            "name = '{}' and trashed = false",
            escape_query_literal(&filename)
        );

        let resp = self
            .client
            .get(GDRIVE_FILES_API)
            .bearer_auth(access_token)
            .query(&[
                ("spaces", "appDataFolder"),
                ("q", &q),
                (
                    "fields",
                    "files(id, name, modifiedTime, size, etag, appProperties)",
                ),
            ])
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriveError::Unauthorized);
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DriveError::ApiError { status, body });
        }

        let bytes = resp.bytes().await?;
        let list: FileListResponse = serde_json::from_slice(&bytes)?;

        if let Some(first) = list.files.into_iter().next() {
            let epoch = first
                .app_properties
                .as_ref()
                .and_then(|p| p.get("snapshot_epoch"))
                .and_then(|s| s.parse::<i64>().ok());

            let size = first.size.and_then(|s| s.parse::<u64>().ok());

            Ok(Some(DriveFile {
                id: first.id,
                name: first.name,
                modified_time: first.modified_time,
                size,
                etag: first.etag,
                snapshot_epoch: epoch,
            }))
        } else {
            Ok(None)
        }
    }

    /// Sube por primera vez el paquete cifrado de un Workspace a `appDataFolder` usando multipart upload.
    pub async fn upload_workspace_file(
        &self,
        access_token: &str,
        workspace_id: &str,
        snapshot_epoch: i64,
        data: &[u8],
    ) -> Result<DriveFile, DriveError> {
        let filename = Self::workspace_filename(workspace_id);
        let boundary = "MYLAN_MULTIPART_BOUNDARY_2026";

        let metadata_json = serde_json::json!({
            "name": filename,
            "parents": ["appDataFolder"],
            "appProperties": {
                "workspace_id": workspace_id,
                "snapshot_epoch": snapshot_epoch.to_string(),
            }
        });

        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(metadata_json.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");

        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let resp = self
            .client
            .post(format!("{GDRIVE_UPLOAD_API}?uploadType=multipart"))
            .bearer_auth(access_token)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriveError::Unauthorized);
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DriveError::ApiError { status, body });
        }

        let bytes = resp.bytes().await?;
        let raw: RawDriveFile = serde_json::from_slice(&bytes)?;

        Ok(DriveFile {
            id: raw.id,
            name: raw.name,
            modified_time: raw.modified_time,
            size: Some(data.len() as u64),
            etag: raw.etag,
            snapshot_epoch: Some(snapshot_epoch),
        })
    }

    /// Actualiza el paquete cifrado de un Workspace en Google Drive con soporte de concurrencia optimista (`If-Match`).
    pub async fn update_workspace_file(
        &self,
        access_token: &str,
        file_id: &str,
        workspace_id: &str,
        snapshot_epoch: i64,
        data: &[u8],
        if_match_etag: Option<&str>,
    ) -> Result<DriveFile, DriveError> {
        let filename = Self::workspace_filename(workspace_id);
        let boundary = "MYLAN_MULTIPART_BOUNDARY_2026";

        let metadata_json = serde_json::json!({
            "name": filename,
            "appProperties": {
                "workspace_id": workspace_id,
                "snapshot_epoch": snapshot_epoch.to_string(),
            }
        });

        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(metadata_json.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");

        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let mut req = self
            .client
            .patch(format!(
                "{GDRIVE_UPLOAD_API}/{file_id}?uploadType=multipart"
            ))
            .bearer_auth(access_token)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/related; boundary={boundary}"),
            );

        if let Some(etag) = if_match_etag {
            if !etag.is_empty() {
                req = req.header(reqwest::header::IF_MATCH, etag);
            }
        }

        let resp = req.body(body).send().await?;

        if resp.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            return Err(DriveError::ConflictPreconditionFailed);
        }

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriveError::Unauthorized);
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DriveError::ApiError { status, body });
        }

        let bytes = resp.bytes().await?;
        let raw: RawDriveFile = serde_json::from_slice(&bytes)?;

        Ok(DriveFile {
            id: raw.id,
            name: raw.name,
            modified_time: raw.modified_time,
            size: Some(data.len() as u64),
            etag: raw.etag,
            snapshot_epoch: Some(snapshot_epoch),
        })
    }

    /// Descarga el contenido binario del archivo desde Google Drive (`alt=media`).
    pub async fn download_file(
        &self,
        access_token: &str,
        file_id: &str,
    ) -> Result<(Vec<u8>, Option<String>), DriveError> {
        let resp = self
            .client
            .get(format!("{GDRIVE_FILES_API}/{file_id}"))
            .bearer_auth(access_token)
            .query(&[("alt", "media")])
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriveError::Unauthorized);
        }

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(DriveError::NotFound);
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DriveError::ApiError { status, body });
        }

        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let bytes = resp.bytes().await?.to_vec();
        Ok((bytes, etag))
    }

    /// Elimina un archivo de Google Drive.
    pub async fn delete_file(&self, access_token: &str, file_id: &str) -> Result<(), DriveError> {
        let resp = self
            .client
            .delete(format!("{GDRIVE_FILES_API}/{file_id}"))
            .bearer_auth(access_token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DriveError::Unauthorized);
        }

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DriveError::ApiError { status, body });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_filename_convention() {
        assert_eq!(
            DriveClient::workspace_filename("default"),
            "mylan_workspace_default.enc"
        );
        assert_eq!(
            DriveClient::workspace_filename("office-1"),
            "mylan_workspace_office-1.enc"
        );
    }
}
