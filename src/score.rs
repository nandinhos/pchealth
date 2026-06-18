// PBHealth — score engine
//
// Recebe a lista de métricas coletadas e produz um score 0-100 mais
// o status geral (`healthy` | `attention` | `critical`).
//
// Regras:
// - métricas `unavailable` NÃO penalizam
// - métricas `attention` subtraem 5 pontos
// - métricas `critical` subtraem 15 pontos
// - score clampado em [0, 100]
// - thresholds ficam em `crate::thresholds` (futuro)

use serde::{Deserialize, Serialize};

use crate::normalizer::{Metric, MetricStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreReport {
    pub score: u8,
    pub status: MetricStatus,
    pub counts: ScoreCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreCounts {
    pub total: usize,
    pub healthy: usize,
    pub attention: usize,
    pub critical: usize,
    pub unavailable: usize,
}

pub struct ScoreEngine;

impl ScoreEngine {
    pub fn compute(metrics: &[Metric]) -> ScoreReport {
        let mut counts = ScoreCounts {
            total: metrics.len(),
            ..Default::default()
        };

        let mut score: i32 = 100;

        for m in metrics {
            match m.status {
                MetricStatus::Healthy => counts.healthy += 1,
                MetricStatus::Attention => {
                    counts.attention += 1;
                    score -= 5;
                }
                MetricStatus::Critical => {
                    counts.critical += 1;
                    score -= 15;
                }
                MetricStatus::Unavailable => counts.unavailable += 1,
            }
        }

        let score = score.clamp(0, 100) as u8;

        let status = if counts.critical > 0 {
            MetricStatus::Critical
        } else if score >= 80 {
            MetricStatus::Healthy
        } else if score >= 50 {
            MetricStatus::Attention
        } else {
            MetricStatus::Critical
        };

        ScoreReport {
            score,
            status,
            counts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::Category;

    #[test]
    fn all_healthy_gives_full_score() {
        let m = vec![Metric::healthy(Category::Cpu, "load_pct", 10.0f64, Some("%"), "sysinfo")];
        let r = ScoreEngine::compute(&m);
        assert_eq!(r.score, 100);
        assert_eq!(r.status, MetricStatus::Healthy);
    }

    #[test]
    fn critical_dominates_attention() {
        let mut metrics = Vec::new();
        for _ in 0..5 {
            metrics.push(Metric::healthy(Category::Cpu, "x", 0i64, None, "sysinfo"));
        }
        metrics.push(Metric::healthy(Category::Cpu, "y", 0i64, None, "sysinfo").with_status(MetricStatus::Critical));
        let r = ScoreEngine::compute(&metrics);
        assert!(r.score < 100);
        assert_eq!(r.status, MetricStatus::Critical);
        assert_eq!(r.counts.critical, 1);
    }

    #[test]
    fn unavailable_does_not_penalize() {
        let metrics = vec![
            Metric::healthy(Category::Cpu, "x", 0i64, None, "sysinfo"),
            Metric::unavailable(Category::Cpu, "temp", "wmi", "no admin", true),
        ];
        let r = ScoreEngine::compute(&metrics);
        assert_eq!(r.score, 100);
        assert_eq!(r.counts.unavailable, 1);
    }
}