# PBHealth

**Diagnóstico de saúde de hardware rodando direto do pendrive.**
PB Informática — sem instalação, sem administrador, sem servidor.

![Status](https://img.shields.io/badge/status-MVP%20em%20desenvolvimento-yellow)
![Plataforma](https://img.shields.io/badge/plataforma-Windows%20%7C%20Linux-blue)
![Licença](https://img.shields.io/badge/licen%C3%A7a-MIT-green)

---

## O que é

PBHealth é um **app desktop portátil** que o técnico carrega num pendrive exFAT. Ao plugar numa máquina de cliente e dar duplo clique, o app abre uma janela nativa e coleta métricas de hardware (CPU, GPU, RAM, disco, rede, bateria, sensores) **sem exigir privilégio administrativo**, exibindo um **score de saúde 0-100** com alertas semafóricos.

Inspirado conceitualmente em **HWiNFO**, **CPU-Z**, **GPU-Z** e **LibreHardwareMonitor** — mas pensado para uso em campo por técnicos de TI.

## O que **NÃO** é

- ❌ Não é monitoramento remoto multi-máquina
- ❌ Não é agente persistente instalado
- ❌ Não substitui HWiNFO para diagnóstico profissional ultra-detalhado
- ❌ Não tem backend / servidor / login / nuvem

---

## Uso

### Requisitos

- **Pendrive** formatado em **exFAT** (compatível com Windows e Linux, suporta arquivos >4GB)
- **Windows 10/11** com WebView2 Runtime (já vem por padrão em Win10+) **OU** Linux moderno (Ubuntu 22.04+)
- **NÃO precisa de administrador**
- **NÃO precisa instalar nada**

### Executar

1. Copie o conteúdo de `PBHealth-vX.Y.Z/` para a raiz do pendrive
2. Plug o pendrive na máquina do cliente
3. Execute o binário:
   - **Windows**: duplo clique em `PBHealth.exe`
   - **Linux**: terminal → `./pbhealth`
4. Se o Windows Defender bloquear: **"Mais informações" → "Executar mesmo assim"**
5. Clique em **"Diagnosticar agora"** no app

### Configuração (opcional)

Edite `config.toml` antes de executar. Defaults:

```toml
[technician]
company = "PB Informática"   # exibido no rodapé do dashboard e no PDF

[ui]
theme = "auto"               # "auto" | "dark" | "light"
auto_refresh_seconds = 0     # 0 = desligado

[collection]
parallel_collectors = true
timeout_seconds = 10
```

---

## Limitações conhecidas

| Limitação | Por quê | Mitigação |
|---|---|---|
| Temperatura CPU no Windows | Exige driver assinado (WinRing0) ou admin | Marcada como `indisponível` na UI; tutorial explica |
| RPM de fans no Windows | Mesmo motivo | Marcada como `indisponível` |
| SMART detalhado no Linux | Exige root ou grupo `disk` | App NÃO pede senha — métricas marcadas `indisponível` |
| GPU AMD/Intel | Sem ferramenta CLI equivalente ao `nvidia-smi` | Dados pobres, marcados `indisponível` quando ausentes |
| Sem persistência entre máquinas | Cada uma é independente | Cada máquina gera seu próprio snapshot dentro do pendrive |
| Sem histórico de longo prazo | Snapshots só durante uso do pendrive | SQLite local no pendrive |

---

## Build (para desenvolvedores)

### Pré-requisitos

- Rust 1.85+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Tauri CLI 2.x (`cargo install tauri-cli --version "^2.0"`)
- Linux: `apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- Windows: Visual Studio Build Tools 2022 (com "Desktop development with C++") + WebView2 Runtime

### Desenvolvimento (hot-reload)

```bash
cargo tauri dev
```

### Build release

```bash
# Linux (roda no Linux)
cargo tauri build

# Windows (roda no Windows, ou cross-compile com cargo-xwin)
cargo tauri build
```

Saída:
- `target/release/pbhealth.exe` (Windows)
- `target/release/pbhealth` (Linux)

### Empacotar para o pendrive

```bash
./scripts/make-pendrive.sh
# ou com versão custom:
./scripts/make-pendrive.sh 0.2.0
```

Gera `dist/PBHealth-v<VER>/` com binário, frontend, config.toml, README.txt e SHA256.txt.

---

## Estrutura do projeto

```
pbhealth/
├── Cargo.toml                   # deps travadas
├── tauri.conf.json              # config do app
├── src/
│   ├── main.rs                  # entrypoint
│   ├── lib.rs                   # crate root, Tauri setup
│   ├── error.rs                 # AppError
│   ├── normalizer.rs            # Metric struct (schema cross-OS)
│   ├── score.rs                 # score engine 0-100
│   ├── commands.rs              # IPC Tauri
│   ├── config.rs                # config.toml loader
│   └── collectors/
│       ├── mod.rs               # trait Collector + Registry
│       ├── system_info.rs       # cross-platform via sysinfo
│       ├── windows_only.rs      # WMI stub (Fase 2)
│       └── linux_only.rs        # shelling stub (Fase 3)
├── ui/
│   ├── index.html               # dashboard
│   └── assets/
│       ├── app.css              # tema minimalista
│       └── app.js               # lógica do dashboard
├── icons/                       # PNG + ICO (placeholders sólidos)
├── scripts/
│   ├── make-pendrive.sh         # empacota release
│   └── sign-hash.sh             # SHA256 (substitui code signing)
└── docs/
    ├── ANALISE.md               # análise técnica completa (41KB)
    └── SETUP_SPEC.md            # briefing de implementação (18KB)
```

---

## Roadmap

| Fase | Status | Descrição |
|---|---|---|
| 0 | ✅ | Setup + esqueleto Tauri |
| 1 | ✅ | Coleta cross-platform via `sysinfo` |
| 2 | ⏳ | WMI completo (BIOS, mobo, RAM módulos, GPU, battery, services) |
| 3 | ⏳ | Shell Linux completo (lscpu, lsblk, dmidecode, sensors) |
| 4 | ⏳ | `nvidia-smi` shelling-out |
| 5 | ✅ | Score engine |
| 6 | ⏳ | SQLite local + migrations + repositório |
| 7 | 🟡 | Dashboard frontend (cards básicos OK; falta drill-down e histórico) |
| 8 | ⏳ | Histórico + gráficos |
| 9 | ⏳ | Export PDF (relatório ao cliente) |
| 10 | ⏳ | Polish, smoke test, build pendrive |

Veja `docs/SETUP_SPEC.md` §9 para roadmap completo com estimativas.

---

## Créditos

**PB Informática** — autoria e manutenção.

Bibliotecas de terceiros (todas MIT/Apache-2.0/BSD):
- [Tauri 2](https://tauri.app/) — framework desktop
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — coleta cross-platform
- [wmi](https://github.com/ohyoject/wmi-rs) — WMI Windows
- [nvml-wrapper](https://github.com/Coolnesss/nvml-wrapper) — NVIDIA
- [rusqlite](https://github.com/rusqlite/rusqlite) — SQLite local
- [genpdf](https://github.com/sfstoolbox/genpdf) — geração PDF
- [rayon](https://github.com/rayon-rs/rayon) — paralelismo
- [serde](https://serde.rs/) — serialização

Inspirado conceitualmente em HWiNFO, CPU-Z, GPU-Z e LibreHardwareMonitor.

---

## Licença

MIT — ver [LICENSE](LICENSE).

---

## Aviso de segurança

Este binário NÃO é assinado digitalmente (decisão de uso interno + gratuito).
Windows SmartScreen pode bloquear na primeira execução em máquinas novas.
Workaround documentado: **"Mais informações" → "Executar mesmo assim"**.

Para distribuição externa em larga escala, considere adquirir um certificado
de assinatura de código (~US$ 200-400/ano).