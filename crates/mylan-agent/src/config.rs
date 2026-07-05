//! Configuración del agent (`mylan-agent.toml`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use mylan_core::ScanProfile;

/// Configuración del agent daemon.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    /// Intervalo entre scans en segundos.
    pub interval_secs: u64,
    /// Redes a escanear, cada una con su CIDR y perfil.
    pub networks: Vec<NetworkSchedule>,
    /// Puerto del API embebido (ADR-4).
    pub api_port: u16,
    /// Override de la ruta de la DB; `None` = `mylan_db::connection::default_db_path()`.
    #[serde(default)]
    pub db_path: Option<PathBuf>,
}

/// Una red programada para escaneo periódico.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkSchedule {
    /// CIDR de la red (p.ej. `192.168.1.0/24`).
    pub cidr: String,
    /// Perfil de profundidad del escaneo.
    pub profile: ScanProfile,
}

impl AgentConfig {
    /// Parsea la config desde un string TOML.
    ///
    /// # Errors
    /// Propaga errores de `toml::from_str` (formato inválido, campos faltantes).
    pub fn parse(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).context("parseando mylan-agent.toml")
    }

    /// Carga la config desde un fichero.
    ///
    /// # Errors
    /// Propaga errores de lectura de disco o de parseo TOML.
    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("leyendo config {}", path.display()))?;
        Self::parse(&s)
    }

    /// Path por defecto: `$XDG_CONFIG_HOME/mylan/mylan-agent.toml` o
    /// `~/.config/mylan/mylan-agent.toml`. `None` si no se puede resolver `$HOME`.
    #[must_use]
    pub fn default_config_path() -> Option<PathBuf> {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return Some(PathBuf::from(xdg).join("mylan").join("mylan-agent.toml"));
            }
        }
        let home = std::env::var("HOME").ok().filter(|h| !h.is_empty())?;
        Some(
            PathBuf::from(home)
                .join(".config")
                .join("mylan")
                .join("mylan-agent.toml"),
        )
    }

    /// Resuelve la ruta de la DB: override explícito o `default_db_path()`.
    ///
    /// # Errors
    /// `anyhow` si ni el override ni `default_db_path()` resuelven (p.ej. sin `$HOME`).
    pub fn db_path(&self) -> Result<PathBuf> {
        self.db_path
            .clone()
            .or_else(mylan_db::connection::default_db_path)
            .ok_or_else(|| anyhow!("no se pudo resolver db_path (sin $HOME ni override)"))
    }
}