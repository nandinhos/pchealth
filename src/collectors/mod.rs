// PBHealth — registry de coletores
//
// Cada categoria do dashboard tem 1+ coletores. O `Registry` detecta o OS
// em runtime e registra apenas os coletores aplicáveis.
//
// Princípio: **best-effort**. Coletores individuais nunca abortam o
// diagnóstico. Falha vira `Metric::unavailable(...)` e segue.

use std::time::Duration;

use rayon::prelude::*;

use crate::normalizer::Metric;

pub mod system_info;

#[cfg(target_os = "windows")]
pub mod windows_only;

#[cfg(target_os = "linux")]
pub mod linux_only;

/// Trait comum a todos os coletores.
///
/// Cada coletor declara sua categoria e implementa `collect`, que retorna
/// 0+ métricas. Em caso de falha total, deve retornar métricas com
/// `status: Unavailable` em vez de `Err` — para que o score engine
// possa seguir.
#[allow(dead_code)] // timeout() é API futura
pub trait Collector: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> crate::normalizer::Category;

    /// Coleta métricas. Erros são engolidos e viram `Metric::unavailable`.
    fn collect(&self) -> Vec<Metric>;

    /// Timeout sugerido por coletor (segurança contra hang).
    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}

/// Registry de coletores ativos.
pub struct Registry {
    collectors: Vec<Box<dyn Collector>>,
}

impl Registry {
    /// Constrói o registry adequado ao SO atual.
    pub fn new_for_host_os() -> Self {
        let mut collectors: Vec<Box<dyn Collector>> = Vec::new();

        // Coletor cross-platform via sysinfo — sempre presente
        collectors.push(Box::new(system_info::SystemInfoCollector::new()));

        #[cfg(target_os = "windows")]
        {
            collectors.push(Box::new(windows_only::WindowsWmiCollector::new()));
        }

        #[cfg(target_os = "linux")]
        {
            collectors.push(Box::new(linux_only::LinuxShellCollector::new()));
        }

        Self { collectors }
    }

    /// Lista categorias cobertas.
    pub fn categories(&self) -> Vec<crate::normalizer::Category> {
        let mut cats: Vec<_> = self.collectors.iter().map(|c| c.category()).collect();
        cats.sort_by_key(|c| c.as_str());
        cats.dedup();
        cats
    }

    /// Roda todos os coletores em paralelo e devolve todas as métricas.
    ///
    /// Coletores que demorarem mais que `timeout()` são descartados
    /// (timeout duro não implementado nesta fase; mantido como log warning).
    pub fn collect_all(&self) -> Vec<Metric> {
        log::info!(
            "Iniciando coleta: {} coletores ativos",
            self.collectors.len()
        );

        let results: Vec<Vec<Metric>> = self
            .collectors
            .par_iter()
            .map(|collector| {
                log::debug!("Coletando: {} ({})", collector.name(), collector.category().as_str());
                let started = std::time::Instant::now();
                let metrics = collector.collect();
                let elapsed = started.elapsed();
                log::debug!(
                    "{}: {} métricas em {:?}",
                    collector.name(),
                    metrics.len(),
                    elapsed
                );
                metrics
            })
            .collect();

        let total: Vec<Metric> = results.into_iter().flatten().collect();
        log::info!("Coleta concluída: {} métricas no total", total.len());
        total
    }
}