// PBHealth — coletor específico para Linux
//
// Stub. Implementação completa virá nas Fases 3-4 do roadmap.
//
// O que vai entrar aqui:
// - shelling-out para `lscpu`, `lsblk -J`, `df -h`, `dmidecode`,
//   `lshw -json`, `sensors`, `nvidia-smi`, `ip`, `iwconfig`
// - leitura de `/sys/class/...` (temperaturas via hwmon)
// - leitura de `/proc/stat`, `/proc/meminfo`
// - best-effort com smartctl, nvme-cli (se root; senão, unavailable)

use crate::normalizer::{Category, Metric};

pub struct LinuxShellCollector {
    name: &'static str,
}

impl LinuxShellCollector {
    pub const fn new() -> Self {
        Self { name: "linux-shell" }
    }
}

impl Default for LinuxShellCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Collector for LinuxShellCollector {
    fn name(&self) -> &'static str {
        self.name
    }

    fn category(&self) -> Category {
        Category::Machine
    }

    fn collect(&self) -> Vec<Metric> {
        log::debug!("LinuxShellCollector: stub — implementação virá na Fase 3");
        vec![Metric::unavailable(
            Category::Bios,
            "bios_vendor",
            self.name,
            "Coletor shell Linux ainda não implementado (Fase 3 do roadmap)",
            false,
        )]
    }
}