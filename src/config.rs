// PBHealth — configuração persistente
//
// Carrega `config.toml` do diretório do executável (ou cria com defaults
// na primeira execução). Editável manualmente pelo técnico antes de rodar.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technician {
    pub name: String,
    pub company: String,
    pub cnpj: String,
    pub website: String,
}

impl Default for Technician {
    fn default() -> Self {
        Self {
            name: String::new(),
            company: "PB Informática".to_string(),
            cnpj: String::new(),
            website: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ui {
    pub theme: String,                // "auto" | "dark" | "light"
    pub auto_refresh_seconds: u32,    // 0 = desligado
    pub language: String,             // fixo em "pt-BR"
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            auto_refresh_seconds: 0,
            language: "pt-BR".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub parallel_collectors: bool,
    pub timeout_seconds: u32,
    pub include_unavailable: bool,
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            parallel_collectors: true,
            timeout_seconds: 10,
            include_unavailable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub pdf_include_json_appendix: bool,
    pub pdf_company_logo: bool,
}

impl Default for Export {
    fn default() -> Self {
        Self {
            pdf_include_json_appendix: true,
            pdf_company_logo: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub technician: Technician,
    pub ui: Ui,
    pub collection: Collection,
    pub export: Export,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            technician: Technician::default(),
            ui: Ui::default(),
            collection: Collection::default(),
            export: Export::default(),
        }
    }
}

impl Config {
    /// Caminho do arquivo `config.toml` — ao lado do binário.
    pub fn config_path() -> PathBuf {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let fallback = PathBuf::from(".");
        let dir = exe.parent().unwrap_or(&fallback);
        dir.join("config.toml")
    }

    /// Carrega config do disco. Se não existir, cria com defaults.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            log::info!("config.toml não existe — criando com defaults em {:?}", path);
            let cfg = Self::default();
            cfg.save()?;
            return Ok(cfg);
        }

        let content = fs::read_to_string(&path)?;
        let cfg: Self = toml::from_str(&content).map_err(|e| {
            AppError::Config(format!("Falha ao parsear config.toml: {e}"))
        })?;
        Ok(cfg)
    }

    /// Salva config no disco (formato TOML).
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self).map_err(|e| {
            AppError::Config(format!("Falha ao serializar config: {e}"))
        })?;
        fs::write(&path, content)?;
        Ok(())
    }
}