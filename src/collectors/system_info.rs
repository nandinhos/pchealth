// PBHealth — coletor cross-platform via `sysinfo`
//
// Cobre o conjunto básico de métricas que funciona em Windows e Linux
// sem privilégio administrativo:
//   - identificação da máquina (hostname, OS, kernel)
//   - CPU (modelo, cores, freq, uso)
//   - memória (total, usado, disponível)
//   - discos (lista, espaço, tipo)
//   - rede (interfaces, tráfego)
//   - processos (top por CPU)

use sysinfo::{
    CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, RefreshKind,
    System,
};

use crate::normalizer::{Category, Metric};

pub struct SystemInfoCollector {
    name: &'static str,
}

impl SystemInfoCollector {
    pub const fn new() -> Self {
        Self { name: "sysinfo" }
    }
}

impl Default for SystemInfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Collector for SystemInfoCollector {
    fn name(&self) -> &'static str {
        self.name
    }

    fn category(&self) -> Category {
        // Categoria primária é `Machine`, mas o coletor produz métricas
        // de múltiplas categorias. O trait exige 1 categoria — usamos
        // Machine como "primária" e as outras vêm como side-effect.
        Category::Machine
    }

    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::with_capacity(64);

        // sysinfo 0.39 mudou a API: Disks virou tipo separado
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // Primeiro refresh — popula uso de CPU corretamente
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // ── Machine ──────────────────────────────────────────────────
        metrics.push(Metric::healthy(
            Category::Machine,
            "hostname",
            System::host_name().unwrap_or_else(|| "(desconhecido)".into()),
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Machine,
            "os_name",
            System::name().unwrap_or_else(|| "(desconhecido)".into()),
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Machine,
            "os_version",
            System::os_version().unwrap_or_else(|| "(desconhecido)".into()),
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Machine,
            "kernel_version",
            System::kernel_version().unwrap_or_else(|| "(desconhecido)".into()),
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Machine,
            "arch",
            std::env::consts::ARCH,
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Machine,
            "uptime_seconds",
            System::uptime() as i64,
            Some("s"),
            self.name,
        ));

        // ── CPU ──────────────────────────────────────────────────────
        let cpus = sys.cpus();

        if let Some(cpu) = cpus.first() {
            metrics.push(Metric::healthy(
                Category::Cpu,
                "brand",
                cpu.brand().to_string(),
                None,
                self.name,
            ));

            metrics.push(Metric::healthy(
                Category::Cpu,
                "vendor",
                cpu.vendor_id().to_string(),
                None,
                self.name,
            ));

            metrics.push(Metric::healthy(
                Category::Cpu,
                "frequency_mhz",
                cpu.frequency() as f64,
                Some("MHz"),
                self.name,
            ));
        } else {
            metrics.push(Metric::unavailable(
                Category::Cpu,
                "brand",
                self.name,
                "sysinfo não detectou CPU",
                false,
            ));
        }

        // physical_core_count é função ASSOCIADA em sysinfo 0.39 (não método)
        metrics.push(Metric::healthy(
            Category::Cpu,
            "physical_cores",
            System::physical_core_count().unwrap_or(0) as i64,
            None,
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Cpu,
            "logical_cores",
            cpus.len() as i64,
            None,
            self.name,
        ));

        // Uso global
        let global_cpu = sys.global_cpu_usage() as f64;
        metrics.push(Metric::healthy(
            Category::Cpu,
            "usage_global_pct",
            global_cpu,
            Some("%"),
            self.name,
        ));

        // Uso por core
        for (idx, cpu) in cpus.iter().enumerate() {
            metrics.push(Metric::healthy(
                Category::Cpu,
                format!("core_{}_usage_pct", idx),
                cpu.cpu_usage() as f64,
                Some("%"),
                self.name,
            ));
        }

        // ── Memória ──────────────────────────────────────────────────
        let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0; // MB
        let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0;
        let avail_mem = sys.available_memory() as f64 / 1024.0 / 1024.0;

        metrics.push(Metric::healthy(
            Category::Memory,
            "total_mb",
            total_mem,
            Some("MB"),
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Memory,
            "used_mb",
            used_mem,
            Some("MB"),
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Memory,
            "available_mb",
            avail_mem,
            Some("MB"),
            self.name,
        ));

        metrics.push(Metric::healthy(
            Category::Memory,
            "usage_pct",
            if total_mem > 0.0 { (used_mem / total_mem) * 100.0 } else { 0.0 },
            Some("%"),
            self.name,
        ));

        let total_swap = sys.total_swap() as f64 / 1024.0 / 1024.0;
        let used_swap = sys.used_swap() as f64 / 1024.0 / 1024.0;

        if total_swap > 0.0 {
            metrics.push(Metric::healthy(
                Category::Memory,
                "swap_total_mb",
                total_swap,
                Some("MB"),
                self.name,
            ));
            metrics.push(Metric::healthy(
                Category::Memory,
                "swap_used_mb",
                used_swap,
                Some("MB"),
                self.name,
            ));
        }

        // ── Discos ───────────────────────────────────────────────────
        // sysinfo 0.39: Disks é tipo separado, não parte de System
        let disks = Disks::new_with_refreshed_list();
        metrics.push(Metric::healthy(
            Category::Storage,
            "disk_count",
            disks.iter().count() as i64,
            None,
            self.name,
        ));

        for (idx, disk) in disks.iter().enumerate() {
            let kind = match disk.kind() {
                DiskKind::HDD => "HDD",
                DiskKind::SSD => "SSD",
                DiskKind::Unknown(_) => "?",
            };

            metrics.push(Metric::healthy(
                Category::Storage,
                format!("disk_{}_name", idx),
                disk.name().to_string_lossy().to_string(),
                None,
                self.name,
            ));

            metrics.push(Metric::healthy(
                Category::Storage,
                format!("disk_{}_kind", idx),
                kind,
                None,
                self.name,
            ));

            let total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let avail_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;

            metrics.push(Metric::healthy(
                Category::Storage,
                format!("disk_{}_total_gb", idx),
                total_gb,
                Some("GB"),
                self.name,
            ));

            metrics.push(Metric::healthy(
                Category::Storage,
                format!("disk_{}_available_gb", idx),
                avail_gb,
                Some("GB"),
                self.name,
            ));

            let used_gb = total_gb - avail_gb;
            let usage_pct = if total_gb > 0.0 { (used_gb / total_gb) * 100.0 } else { 0.0 };
            metrics.push(Metric::healthy(
                Category::Storage,
                format!("disk_{}_usage_pct", idx),
                usage_pct,
                Some("%"),
                self.name,
            ));
        }

        // ── Rede ─────────────────────────────────────────────────────
        let networks = Networks::new_with_refreshed_list();
        let iface_count = networks.iter().count();
        metrics.push(Metric::healthy(
            Category::Network,
            "interface_count",
            iface_count as i64,
            None,
            self.name,
        ));

        for (name, net) in networks.iter() {
            let key = sanitize_key(name);
            metrics.push(Metric::healthy(
                Category::Network,
                format!("iface_{}_name", key),
                name.clone(),
                None,
                self.name,
            ));
            metrics.push(Metric::healthy(
                Category::Network,
                format!("iface_{}_received_mb", key),
                net.received() as f64 / 1024.0 / 1024.0,
                Some("MB"),
                self.name,
            ));
            metrics.push(Metric::healthy(
                Category::Network,
                format!("iface_{}_transmitted_mb", key),
                net.transmitted() as f64 / 1024.0 / 1024.0,
                Some("MB"),
                self.name,
            ));
        }

        // ── Top processos ────────────────────────────────────────────
        // Refresh de processos sob demanda — pode custar ~50-200ms em
        // máquinas com muitos processos.
        let mut sys_proc = System::new();
        sys_proc.refresh_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );

        let mut procs: Vec<_> = sys_proc.processes().values().collect();
        procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(5);

        for (rank, proc) in procs.iter().enumerate() {
            let name = proc.name().to_string_lossy().to_string();
            let cpu = proc.cpu_usage() as f64;
            let mem_mb = proc.memory() as f64 / 1024.0 / 1024.0;

            metrics.push(Metric::healthy(
                Category::Os,
                format!("top_proc_{}_name", rank),
                name,
                None,
                self.name,
            ));
            metrics.push(Metric::healthy(
                Category::Os,
                format!("top_proc_{}_cpu_pct", rank),
                cpu,
                Some("%"),
                self.name,
            ));
            metrics.push(Metric::healthy(
                Category::Os,
                format!("top_proc_{}_mem_mb", rank),
                mem_mb,
                Some("MB"),
                self.name,
            ));
        }

        metrics
    }
}

/// Sanitiza nome de interface de rede para uso como chave de métrica.
fn sanitize_key(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::Collector;

    #[test]
    fn collect_smoke() {
        let c = SystemInfoCollector::new();
        let m = c.collect();
        assert!(!m.is_empty(), "coletor deve produzir ao menos 1 métrica");

        // Deve ter hostname
        assert!(m.iter().any(|m| m.key == "hostname"));
        // Deve ter CPU brand
        assert!(m.iter().any(|m| m.key == "brand"));
        // Deve ter memória total
        assert!(m.iter().any(|m| m.key == "total_mb"));
    }

    #[test]
    fn sanitize_key_works() {
        assert_eq!(sanitize_key("eth0"), "eth0");
        assert_eq!(sanitize_key("Wi-Fi"), "Wi_Fi");
        assert_eq!(sanitize_key("Ethernet 2"), "Ethernet_2");
    }
}