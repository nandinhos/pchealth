// PBHealth — crate root
//
// Aqui ficam: configuração do Tauri, registro de comandos IPC,
// estado da aplicação, e entrypoint da biblioteca.

use std::sync::Arc;

mod error;
mod collectors;
mod normalizer;
mod score;
mod commands;
mod config;

use collectors::Registry;

/// Estado global da aplicação passado para todos os comandos Tauri.
pub struct AppState {
    pub registry: Arc<Registry>,
    pub config: Arc<tokio::sync::RwLock<config::Config>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Logger básico — só inicializa se ainda não foi
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    log::info!("PBHealth iniciando...");

    // Carrega config.toml (ou cria com defaults)
    let cfg = match config::Config::load() {
        Ok(c) => Arc::new(tokio::sync::RwLock::new(c)),
        Err(e) => {
            log::warn!("Falha ao carregar config.toml: {e}. Usando defaults.");
            Arc::new(tokio::sync::RwLock::new(config::Config::default()))
        }
    };

    // Detecta OS e monta o registry de coletores
    let registry = Arc::new(collectors::Registry::new_for_host_os());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            log::info!("PBHealth pronto. OS: {}", std::env::consts::OS);
            Ok(())
        })
        .manage(AppState {
            registry,
            config: cfg,
        })
        .invoke_handler(tauri::generate_handler![
            commands::run_diagnostic,
            commands::get_history,
            commands::export_json,
            commands::export_pdf,
            commands::get_config,
            commands::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("erro fatal ao iniciar PBHealth");
}

// Re-exports úteis para crates externos (útil em testes)
pub use error::Result;
pub use normalizer::{Metric, MetricStatus, MetricValue, Category};
pub use score::{ScoreEngine, ScoreReport};