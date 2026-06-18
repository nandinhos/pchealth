// PBHealth — tipos de erro
//
// Erros sempre conversíveis para string (necessário para Tauri IPC,
// que aceita `Result<T, String>` nos comandos).
//
// `thiserror::Error` já deriva `Display` automaticamente — não
// precisamos implementar manualmente.

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Coleta falhou: {0}")]
    Collect(String),

    #[error("Métrica indisponível: {0}")]
    Unavailable(String),

    #[error("Comando externo falhou ({cmd}): {msg}")]
    External { cmd: String, msg: String },

    #[error("Banco de dados: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Pool do banco: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialização: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Configuração inválida: {0}")]
    Config(String),

    #[error("PDF: {0}")]
    Pdf(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn collect<S: Into<String>>(msg: S) -> Self {
        AppError::Collect(msg.into())
    }

    pub fn external<S: Into<String>>(cmd: S, msg: S) -> Self {
        AppError::External {
            cmd: cmd.into(),
            msg: msg.into(),
        }
    }

    pub fn unavailable<S: Into<String>>(metric: S) -> Self {
        AppError::Unavailable(metric.into())
    }

    pub fn other<S: Into<String>>(msg: S) -> Self {
        AppError::Other(msg.into())
    }
}

/// Converte erro para string (necessário para Tauri IPC).
impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

/// Converte anyhow -> AppError (helper).
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(format!("{:#}", e))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;