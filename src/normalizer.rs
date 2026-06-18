// PBHealth — modelo normalizado de métrica
//
// Toda coleta vira um `Metric`. Independente da fonte (WMI, sysinfo,
// nvidia-smi, lm-sensors), o consumidor (score engine, dashboard, PDF)
// só conhece este tipo.

use serde::{Deserialize, Serialize};

/// Categoria principal da métrica (alinhada com dashboard cards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Machine,     // identificação da máquina
    Bios,        // BIOS/UEFI + mobo
    Cpu,
    Memory,
    Gpu,
    Storage,
    Sensors,     // temp/fans
    Power,       // bateria
    Network,
    Os,          // SO + processos + serviços
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Machine => "machine",
            Category::Bios => "bios",
            Category::Cpu => "cpu",
            Category::Memory => "memory",
            Category::Gpu => "gpu",
            Category::Storage => "storage",
            Category::Sensors => "sensors",
            Category::Power => "power",
            Category::Network => "network",
            Category::Os => "os",
        }
    }

    pub fn label_pt(&self) -> &'static str {
        match self {
            Category::Machine => "Máquina",
            Category::Bios => "BIOS / Placa-mãe",
            Category::Cpu => "Processador",
            Category::Memory => "Memória RAM",
            Category::Gpu => "Placa de vídeo",
            Category::Storage => "Armazenamento",
            Category::Sensors => "Sensores",
            Category::Power => "Energia / Bateria",
            Category::Network => "Rede",
            Category::Os => "Sistema operacional",
        }
    }
}

/// Valor da métrica — variants cobrem todos os casos reais.
///
/// `Null` é para `unavailable`. Use `status: MetricStatus::Unavailable`
/// em conjunto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Bool(bool),
    Null,
}

/// Status semafórico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    /// Dentro dos thresholds normais.
    Healthy,
    /// Atenção — fora do ideal mas funcional.
    Attention,
    /// Crítico — problema sério, ação recomendada.
    Critical,
    /// Não foi possível coletar (sem admin, ferramenta ausente, etc).
    Unavailable,
}

impl MetricStatus {
    pub fn from_optional<S: Into<String>>(msg: S) -> MetricStatus {
        let _ = msg.into(); // reservado para futuro
        MetricStatus::Unavailable
    }
}

/// Uma métrica coletada.
///
/// Invariantes:
/// - `value == MetricValue::Null` sse `status == MetricStatus::Unavailable`
/// - `requires_admin` indica se o coletor precisaria de privilégio
/// - `source` é o nome da fonte real (`wmi`, `sysinfo`, `nvidia-smi`, ...)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub category: Category,
    pub key: String,
    pub value: MetricValue,
    pub unit: Option<String>,
    pub status: MetricStatus,
    pub source: String,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Metric {
    /// Construtor para métrica saudável.
    pub fn healthy<T: Into<MetricValue>>(
        category: Category,
        key: impl Into<String>,
        value: T,
        unit: Option<&str>,
        source: &str,
    ) -> Self {
        Self {
            category,
            key: key.into(),
            value: value.into(),
            unit: unit.map(String::from),
            status: MetricStatus::Healthy,
            source: source.to_string(),
            requires_admin: false,
            error: None,
        }
    }

    /// Construtor para métrica indisponível.
    pub fn unavailable(
        category: Category,
        key: impl Into<String>,
        source: &str,
        error: impl Into<String>,
        requires_admin: bool,
    ) -> Self {
        Self {
            category,
            key: key.into(),
            value: MetricValue::Null,
            unit: None,
            status: MetricStatus::Unavailable,
            source: source.to_string(),
            requires_admin,
            error: Some(error.into()),
        }
    }

    /// Helper para atualizar status baseado em threshold.
    pub fn with_status(mut self, status: MetricStatus) -> Self {
        self.status = status;
        self
    }

    /// Serializa valor para string amigável ao usuário (pt-BR).
    pub fn display_value(&self) -> String {
        match &self.value {
            MetricValue::Text(s) => s.clone(),
            MetricValue::Number(n) => format_number(*n),
            MetricValue::Integer(i) => i.to_string(),
            MetricValue::Bool(b) => if *b { "sim" } else { "não" }.to_string(),
            MetricValue::Null => "indisponível".to_string(),
        }
    }
}

/// Formata número com separador pt-BR e até 2 casas decimais.
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        // Inteiro disfarçado — sem casas decimais
        let i = n as i64;
        format_with_thousands_separator(i)
    } else if n.abs() >= 100.0 {
        format!("{:.0}", n)
    } else if n.abs() >= 10.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.2}", n)
    }
}

fn format_with_thousands_separator(i: i64) -> String {
    let s = i.abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    for (idx, c) in bytes.iter().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.insert(0, '.');
        }
        out.insert(0, *c as char);
    }
    if i < 0 {
        out.insert(0, '-');
    }
    out
}

// Conversões From para MetricValue
impl From<String> for MetricValue {
    fn from(s: String) -> Self {
        MetricValue::Text(s)
    }
}

impl From<&str> for MetricValue {
    fn from(s: &str) -> Self {
        MetricValue::Text(s.to_string())
    }
}

impl From<f64> for MetricValue {
    fn from(n: f64) -> Self {
        MetricValue::Number(n)
    }
}

impl From<i64> for MetricValue {
    fn from(i: i64) -> Self {
        MetricValue::Integer(i)
    }
}

impl From<u32> for MetricValue {
    fn from(i: u32) -> Self {
        MetricValue::Integer(i as i64)
    }
}

impl From<u64> for MetricValue {
    fn from(i: u64) -> Self {
        MetricValue::Integer(i as i64)
    }
}

impl From<usize> for MetricValue {
    fn from(i: usize) -> Self {
        MetricValue::Integer(i as i64)
    }
}

impl From<bool> for MetricValue {
    fn from(b: bool) -> Self {
        MetricValue::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_metric_display() {
        let m = Metric::healthy(Category::Cpu, "cores", 16i64, None, "sysinfo");
        assert_eq!(m.display_value(), "16");
        assert_eq!(m.status, MetricStatus::Healthy);
    }

    #[test]
    fn unavailable_metric_display() {
        let m = Metric::unavailable(
            Category::Cpu,
            "temperature",
            "wmi",
            "Requer privilégio administrativo",
            true,
        );
        assert_eq!(m.display_value(), "indisponível");
        assert_eq!(m.status, MetricStatus::Unavailable);
        assert!(m.requires_admin);
    }

    #[test]
    fn float_formatting() {
        let m = Metric::healthy(Category::Cpu, "load_pct", 23.45f64, Some("%"), "sysinfo");
        // display_value usa format!("{:.1}", n) para |n| >= 10.
        // Rust usa banker's rounding: 23.45 -> "23.4" (mais próximo par).
        assert_eq!(m.display_value(), "23.4");
    }

    #[test]
    fn category_label_pt() {
        assert_eq!(Category::Cpu.label_pt(), "Processador");
        assert_eq!(Category::Storage.label_pt(), "Armazenamento");
    }
}