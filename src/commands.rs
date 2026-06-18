// PBHealth — comandos IPC (frontend ↔ backend)
//
// Os comandos abaixo são invocados pelo JavaScript do frontend via
// `invoke('nome_do_comando', { args })`. Retornam `Result<T, String>`
// (Tauri serializa/deserializa automaticamente).

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::normalizer::Metric;
use crate::score::ScoreEngine;
use crate::AppState;

/// Reporte completo de uma rodada de diagnóstico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub os: String,
    pub metrics: Vec<Metric>,
    pub score: crate::score::ScoreReport,
}

/// Resumo de um snapshot gravado (para histórico).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSummary {
    pub id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub score: u8,
    pub status: String,
    pub metric_count: usize,
}

/// Comando principal — roda diagnóstico completo.
#[tauri::command]
pub async fn run_diagnostic(state: State<'_, AppState>) -> Result<DiagnosticReport, String> {
    log::info!("Comando run_diagnostic invocado");

    // Coleta todas as métricas (rayon par_iter)
    let metrics = state.registry.collect_all();

    // Calcula score
    let score = ScoreEngine::compute(&metrics);

    // Persistência no SQLite virá na Fase 6 — por enquanto só retornamos.
    // TODO(fase-6): gravar snapshot na tabela `health_snapshots`.

    Ok(DiagnosticReport {
        timestamp: chrono::Utc::now(),
        os: std::env::consts::OS.to_string(),
        metrics,
        score,
    })
}

/// Histórico de snapshots. Stub — virá na Fase 6 com SQLite.
#[tauri::command]
pub async fn get_history(_state: State<'_, AppState>) -> Result<Vec<SnapshotSummary>, String> {
    log::info!("Comando get_history invocado (stub)");
    Ok(Vec::new())
}

/// Exporta snapshot como JSON. Stub — virá na Fase 8.
#[tauri::command]
pub async fn export_json(_report_id: i64, _path: String) -> Result<(), String> {
    Err("export_json ainda não implementado (Fase 8)".into())
}

/// Exporta snapshot como PDF. Stub — virá na Fase 9.
#[tauri::command]
pub async fn export_pdf(_report_id: i64, _path: String) -> Result<(), String> {
    Err("export_pdf ainda não implementado (Fase 9)".into())
}

/// Lê configuração atual.
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<crate::config::Config, String> {
    let cfg = state.config.read().await;
    Ok(cfg.clone())
}

/// Atualiza e persiste configuração.
#[tauri::command]
pub async fn set_config(
    state: State<'_, AppState>,
    new_config: crate::config::Config,
) -> Result<(), String> {
    let mut cfg = state.config.write().await;
    *cfg = new_config;
    cfg.save().map_err(|e: AppError| e.to_string())?;
    Ok(())
}