// PBHealth — coletor específico para Windows
//
// Stub. Implementação completa virá nas Fases 2-4 do roadmap (ver
// `~/projects/pchealth/SETUP_SPEC.md` §9).
//
// O que vai entrar aqui:
// - WMI/CIM queries via crate `wmi` (BIOS, mobo, RAM módulos, GPU,
//   bateria, services, eventos do Event Viewer)
// - shelling-out para `nvidia-smi.exe`
// - Performance Counters via PowerShell (CPU% tempo-real, GPU engine)
//
// Por enquanto produz apenas um marcador de "presente" para sabermos
// na UI que o coletor foi registrado.

use crate::normalizer::{Category, Metric};

pub struct WindowsWmiCollector {
    name: &'static str,
}

impl WindowsWmiCollector {
    pub const fn new() -> Self {
        Self { name: "wmi" }
    }
}

impl Default for WindowsWmiCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Collector for WindowsWmiCollector {
    fn name(&self) -> &'static str {
        self.name
    }

    fn category(&self) -> Category {
        Category::Machine
    }

    fn collect(&self) -> Vec<Metric> {
        log::debug!("WindowsWmiCollector: stub — implementação virá na Fase 2");
        vec![Metric::unavailable(
            Category::Bios,
            "bios_vendor",
            self.name,
            "Coletor WMI ainda não implementado (Fase 2 do roadmap)",
            false,
        )]
    }
}