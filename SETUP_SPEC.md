# PBHealth — Setup Spec (Briefing Pronto pra "Vai")

> Documento gerado a partir da análise em `ANALISE.md`. Define **exatamente** o que será criado quando o usuário disser "vai".
> **Nenhum arquivo de código foi criado.** Este é o plano.

---

## 0. Resumo das Decisões Travadas

| Decisão | Escolha |
|---|---|
| Nome do app | **PBHealth** |
| Binário Windows | `PBHealth.exe` |
| Binário Linux | `pbhealth` |
| Marca / brand | PB Informática (crédito no rodapé) |
| Identidade visual | Minimalista (paleta neutra: cinza + azul-petróleo, system-ui) |
| Idioma | pt-BR only (sem framework i18n) |
| Code signing | Não (uso interno) |
| Exportação PDF | Sim (relatório ao cliente final) |
| Pendrive FS | exFAT (não FAT32, não NTFS-only) |

---

## 1. Pré-requisitos do Ambiente de Desenvolvimento

### 1.1 Ferramentas (instalar uma vez)

**Todas as plataformas:**
- **Rust** (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Tauri CLI 2.x**: `cargo install tauri-cli --version "^2.0"`
- **Node.js 20+** (apenas para bundlar assets frontend, opcional): `nvm install 20`

**Linux (Ubuntu/Debian) — para build:**
```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows — para build:**
- Visual Studio Build Tools 2022 (com "Desktop development with C++")
- WebView2 Runtime (já vem em Win10+; em Win7 é pré-requisito documentado)
- Rust via `rustup-init.exe`

### 1.2 Verificação do ambiente

```bash
rustc --version          # esperado: rustc 1.85+
cargo --version          # esperado: cargo 1.85+
cargo tauri --version    # esperado: tauri-cli 2.x
node --version           # esperado: v20+ (opcional)
```

---

## 2. Estrutura de Pastas

```
pbhealth/
├── README.md                    # instruções de uso (pt-BR)
├── LICENSE                      # MIT (PB Informática)
├── Cargo.toml                   # workspace
├── rust-toolchain.toml          # fixa versão Rust
├── .gitignore
├── .github/
│   └── workflows/
│       └── release.yml          # build cross-platform no push de tag
├── docs/
│   ├── ANALISE.md               # (já existe — análise técnica)
│   └── SETUP_SPEC.md            # (este arquivo)
├── src/                         # ── BACKEND RUST ──────────────────────
│   ├── main.rs                  # entrypoint, inicializa Tauri
│   ├── lib.rs                   # crate root, expõe run()
│   ├── error.rs                 # AppError + Result<T>
│   ├── collectors/
│   │   ├── mod.rs               # trait Collector + Registry
│   │   ├── system_info.rs       # sysinfo wrapper cross-platform
│   │   ├── cpu.rs               # CPU: cores, freq, uso
│   │   ├── memory.rs            # RAM: capacidade, uso, módulos
│   │   ├── gpu.rs               # GPU: detect + nvidia-smi
│   │   ├── storage.rs           # discos, partições, smart (best-effort)
│   │   ├── sensors.rs           # temp/fans via lm-sensors (Linux) / WMI (Win)
│   │   ├── network.rs           # interfaces, IP, MAC, velocidade
│   │   ├── battery.rs           # bateria (notebook)
│   │   ├── bios.rs              # BIOS/mobo via WMI/dmidecode
│   │   ├── os_info.rs           # SO, uptime, kernel, serviços
│   │   └── processes.rs         # top processos, eventos críticos
│   ├── normalizer.rs            # struct Metric unificado
│   ├── score.rs                 # engine de score 0-100 + status
│   ├── thresholds.rs            # tabelas de thresholds por métrica
│   ├── db/
│   │   ├── mod.rs
│   │   ├── schema.sql           # CREATE TABLEs (vem da análise)
│   │   ├── migrations.rs        # rusqlite + r2d2
│   │   └── repository.rs        # CRUD: machines, snapshots, metrics
│   ├── exporters/
│   │   ├── mod.rs
│   │   ├── json.rs              # snapshot → JSON file
│   │   └── pdf.rs               # snapshot → PDF (genpdf)
│   ├── commands.rs              # #[tauri::command] handlers (IPC)
│   └── config.rs                # carrega config.toml do pendrive
├── ui/                          # ── FRONTEND (HTML+JS+CSS) ─────────────
│   ├── index.html               # dashboard principal
│   ├── assets/
│   │   ├── app.css              # tema minimalista dark+light
│   │   ├── app.js               # lógica do dashboard, IPC Tauri
│   │   ├── chart.min.js         # Chart.js bundled local (offline)
│   │   └── fonts/               # se necessário
│   └── pages/
│       ├── dashboard.html       # cards por categoria
│       ├── detail.html          # drill-down por componente
│       └── history.html         # histórico de snapshots
├── dist/                        # output dos builds (gitignored)
├── icons/                       # ícones do Tauri (.png, .ico, .icns)
└── scripts/
    ├── build-windows.sh         # cross-compile ou roda no Windows
    ├── build-linux.sh           # build Linux
    ├── make-pendrive.sh         # empacota binário + assets em pasta pronta
    └── sign-hash.sh             # gera SHA256 (substitui code signing)
```

---

## 3. `Cargo.toml` — Dependências (versões travadas em jun/2026)

```toml
[package]
name = "pbhealth"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"

[lib]
name = "pbhealth_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
# --- Tauri ---
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"      # diálogo "Salvar PDF/JSON"
tauri-plugin-fs = "2"          # filesystem local
tauri-plugin-shell = "2"       # shelling-out (nvidia-smi, etc)

# --- Coleta ---
sysinfo = "0.39"
wmi = "0.18"
nvml-wrapper = "0.12"
cfg-if = "1"
once_cell = "1"

# --- DB local ---
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"

# --- Serialização ---
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# --- Async / paralelismo ---
tokio = { version = "1", features = ["full"] }
rayon = "1"
futures = "0.3"

# --- Utilitários ---
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"
env_logger = "0.11"

# --- PDF (relatório) ---
genpdf = "0.2"     # mantido, sem Chromium embedded

[target.'cfg(windows)'.dependencies]
windows = { version = "0.62", features = [
    "Win32_Foundation",
    "Win32_System_SystemInformation",
] }

[target.'cfg(target_os = "linux")'.dependencies]
libc = "0.2"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

---

## 4. `tauri.conf.json` (essencial)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "PBHealth",
  "version": "0.1.0",
  "identifier": "com.pbinformatica.pbhealth",
  "build": {
    "frontendDist": "../ui",
    "devUrl": null,
    "beforeDevCommand": null,
    "beforeBuildCommand": null
  },
  "app": {
    "windows": [
      {
        "title": "PBHealth — Diagnóstico de Hardware",
        "width": 1280,
        "height": 800,
        "minWidth": 960,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; script-src 'self';"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "category": "Utility",
    "shortDescription": "Diagnóstico de saúde de hardware",
    "longDescription": "PBHealth coleta métricas de hardware (CPU, GPU, RAM, disco, rede, bateria) e apresenta score de saúde. Roda do pendrive sem instalação.",
    "publisher": "PB Informática",
    "copyright": "© 2026 PB Informática",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico",
      "icons/icon.icns"
    ],
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper" },
      "wix": { "language": "pt-BR" }
    }
  }
}
```

---

## 5. Pipeline de Coleta (arquitetura interna)

### 5.1 Trait `Collector` (todos os coletores implementam)

```rust
// src/collectors/mod.rs
pub trait Collector: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;  // Cpu, Gpu, Memory, ...
    fn collect(&self) -> Vec<Metric>;
}

pub struct Registry {
    collectors: Vec<Box<dyn Collector>>,
}

impl Registry {
    pub fn new() -> Self { /* detecta OS, registra coletores */ }
    pub fn collect_all(&self) -> Vec<Metric> { /* rayon par_iter */ }
}
```

### 5.2 Schema da `Metric` (normalizado cross-platform)

```rust
// src/normalizer.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metric {
    pub category: Category,
    pub key: String,
    pub value: MetricValue,
    pub unit: Option<String>,
    pub status: MetricStatus,
    pub source: String,
    pub requires_admin: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MetricValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Bool(bool),
    Null,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum MetricStatus {
    Healthy,
    Attention,
    Critical,
    Unavailable,
}
```

### 5.3 Comandos IPC Tauri (frontend ↔ backend)

```rust
// src/commands.rs
#[tauri::command]
async fn run_diagnostic(state: State<AppState>) -> Result<DiagnosticReport, String>;

#[tauri::command]
async fn get_history(state: State<AppState>) -> Result<Vec<SnapshotSummary>, String>;

#[tauri::command]
async fn export_json(report_id: i64, path: String) -> Result<(), String>;

#[tauri::command]
async fn export_pdf(report_id: i64, path: String) -> Result<(), String>;

#[tauri::command]
async fn get_config() -> Result<Config, String>;

#[tauri::command]
async fn set_config(config: Config) -> Result<(), String>;
```

---

## 6. Configuração Persistente (`config.toml` no pendrive)

```toml
# PBHealth/config.toml — configurações do técnico
[technician]
name = ""                       # nome do técnico (opcional)
company = "PB Informática"      # exibido no rodapé do dashboard/PDF
cnpj = ""                       # opcional, exibido no PDF
website = ""                    # opcional

[ui]
theme = "auto"                  # "auto" | "dark" | "light"
auto_refresh_seconds = 0        # 0 = desligado; senão, atualiza métricas dinâmicas
language = "pt-BR"              # fixo por enquanto

[collection]
parallel_collectors = true      # rayon
timeout_seconds = 10            # timeout por coletor
include_unavailable = true      # exibe métricas indisponíveis com aviso

[export]
pdf_include_json_appendix = true
pdf_company_logo = false        # futuro (logo no PDF)
```

**Onde fica:** `<diretório do binário>/config.toml`. Carregado na inicialização; criado com defaults se não existir.

---

## 7. Identidade Visual (CSS variables)

```css
/* ui/assets/app.css — minimalista dark default */
:root {
  --bg-primary: #0f1419;
  --bg-secondary: #1a1f26;
  --bg-card: #1e252e;
  --border: #2a323d;
  --text-primary: #e4e7eb;
  --text-secondary: #8a94a3;
  --accent: #14b8a6;            /* azul-petróleo */
  --accent-hover: #0f9488;
  --healthy: #22c55e;
  --attention: #f59e0b;
  --critical: #ef4444;
  --unavailable: #6b7280;
  --font: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  --radius: 8px;
  --shadow: 0 1px 3px rgba(0,0,0,0.3);
}

[data-theme="light"] {
  --bg-primary: #fafafa;
  --bg-secondary: #ffffff;
  --bg-card: #ffffff;
  --border: #e5e7eb;
  --text-primary: #1f2937;
  --text-secondary: #6b7280;
  --shadow: 0 1px 3px rgba(0,0,0,0.08);
}
```

**Rodapé do dashboard:**
```html
<footer>PBHealth · PB Informática · {ano} · {versão}</footer>
```

---

## 8. Comandos de Build

### 8.1 Dev (desenvolvimento, hot-reload)

```bash
cd pbhealth
cargo tauri dev
```

### 8.2 Build release (Windows)

```bash
cd pbhealth
cargo tauri build
# Output: target/release/pbhealth.exe
```

**Para binário portable (sem instalador), usamos o `target/release/pbhealth.exe` diretamente.**

### 8.3 Build release (Linux)

```bash
cd pbhealth
cargo tauri build
# Output: target/release/pbhealth
#         target/release/bundle/appimage/PBHealth_0.1.0_amd64.AppImage
```

### 8.4 Empacotar para o pendrive (sem instalador)

```bash
# scripts/make-pendrive.sh
#!/usr/bin/env bash
set -euo pipefail

VER="${1:-0.1.0}"
OUT="dist/PBHealth-v${VER}"

mkdir -p "$OUT"
cp "target/release/pbhealth.exe" "$OUT/PBHealth.exe" 2>/dev/null || \
cp "target/release/pbhealth"     "$OUT/pbhealth"
cp -r ui "$OUT/ui"
cp config.toml "$OUT/config.toml" 2>/dev/null || echo "(config.toml não existe — será criado em runtime)"

# README de uso
cat > "$OUT/README.txt" <<EOF
PBHealth v${VER} — Diagnóstico de Hardware
==============================================

USO:
  Windows: clique duplo em PBHealth.exe
  Linux  : abra terminal, ./pbhealth

NÃO PRECISA INSTALAR. NÃO PRECISA DE ADMIN.

Se o Windows Defender bloquear:
  → "Mais informações" → "Executar mesmo assim"

Configuração: edite config.toml antes de executar (opcional).
EOF

# SHA256
sha256sum "$OUT/PBHealth.exe" 2>/dev/null >> "$OUT/SHA256.txt" || true
sha256sum "$OUT/pbhealth"     2>/dev/null >> "$OUT/SHA256.txt" || true

echo "Pronto: $OUT/"
echo "Tamanho:"
du -sh "$OUT"
```

**Tamanho esperado da pasta:** ~50 MB (binário 10-15 MB + assets frontend ~2 MB + extras).

---

## 9. Roadmap Refinado por Fase (com Semanas)

| Fase | Descrição | Saída concreta | Semanas |
|---|---|---|---|
| **0** | Setup do projeto, `cargo init`, `tauri init`, hello world | Binário que abre janela vazia com "PBHealth" no título | 0.5 |
| **1** | Coleta cross-platform básica via `sysinfo` (CPU/RAM/disk/network/processos) | Botão "Diagnosticar" mostra JSON em textarea | 1.5 |
| **2** | Coleta Windows via `wmi` (BIOS, mobo, GPU, RAM módulos, battery) | Categorias completas no JSON | 2 |
| **3** | Coleta Linux via shelling (lscpu, lsblk, df, dmidecode, sensors) | Categorias completas no JSON | 1.5 |
| **4** | `nvidia-smi` shelling-out (Win+Linux) | GPU NVIDIA com temp/clock/fan/watt | 0.5 |
| **5** | Score engine + thresholds + status | Score 0-100 calculado, status por categoria | 1 |
| **6** | SQLite local + migrations + repositório | Snapshots persistidos em `pbhealth.db` | 1 |
| **7** | Dashboard frontend (cards, score, dark mode) | UI bonita substituindo JSON | 2 |
| **8** | Histórico de snapshots + gráficos (Chart.js) | Linha do tempo na sessão | 1 |
| **9** | Export PDF (genpdf) | Relatório imprimível com logo PB Informática | 1 |
| **10** | Polish, README, build pendrive, smoke test em 3 máquinas | Pasta `dist/PBHealth-v0.1.0/` pronta | 1 |
| **TOTAL** | | | **~12 semanas** |

**Justificativa do prazo:** documento tem 100+ métricas, 10 categorias, 2 SOs, PDF, score engine, dashboard. Subestimar isso é a forma mais comum de projeto desses morrer. **12 semanas com folga é melhor que 6 semanas atoladas.**

---

## 10. Validação no Final (Smoke Test Antes de "Pronto")

Quando você rodar `scripts/make-pendrive.sh` e levar o pendrive a uma máquina, o teste é:

1. **Windows 11 limpo:** plugar pendrive → duplo clique → janela abre em <2s → score aparece em <10s
2. **Windows 10 com Defender agressivo:** confirmar se aparece aviso; documentar workaround
3. **Ubuntu 24.04 (sem root):** mesma operação. Confirmar que métricas sem root aparecem, e as que exigem root vêm com `unavailable`
4. **Pendrive exFAT:** rodar a partir de `D:\PBHealth\` e de `/media/user/PENDRIVE/PBHealth/` — ambos funcionam
5. **Export PDF:** relatório gerado, abre no leitor padrão, contém "PB Informática" + score + tabelas
6. **Desconectar pendrive com app aberto:** SQLite em WAL mode não corrompe (confirmar)

---

## 11. Arquivos Criados Quando Disser "Vai"

Quando você disser "vai", **na próxima mensagem** eu criarei, **nessa ordem**:

1. `Cargo.toml`, `rust-toolchain.toml`, `.gitignore`
2. `src/main.rs`, `src/lib.rs`, `src/error.rs`
3. `src/collectors/mod.rs` + trait `Collector` + Registry
4. `src/normalizer.rs` com `Metric` struct
5. `src/collectors/system_info.rs` (sysinfo wrapper)
6. `src/commands.rs` com `run_diagnostic`
7. `tauri.conf.json`
8. `ui/index.html`, `ui/assets/app.js`, `ui/assets/app.css`
9. `icons/` (4 tamanhos mínimos — gerados por placeholder programático)
10. `scripts/make-pendrive.sh`
11. `README.md`

**Estimativa de output:** ~15-20 arquivos, ~1500-2500 linhas totais (Rust + HTML/JS/CSS + config).

---

## 12. O que NÃO vou fazer mesmo depois do "vai"

- ❌ Não vou rodar `cargo build` (você roda, valida, me reporta)
- ❌ Não vou instalar Rust/Tauri CLI no seu sistema (você roda `rustup` quando quiser)
- ❌ Não vou criar logo ou imagem de marca (minimalista — sem logo por design)
- ❌ Não vou publicar no GitHub (decisão sua)
- ❌ Não vou criar instalador MSI/NSIS (descartado pelo escopo pendrive)
- ❌ Não vou comprar certificado de code signing (decisão "uso interno")

---

## 13. Próximo Passo

Leia `ANALISE.md` (análise técnica completa) e este `SETUP_SPEC.md` (briefing de implementação).

Se concordar, responda:

> **"vai"**

E eu começo a criar os arquivos na ordem da seção 11.

Se quiser ajustar algo, fale agora (ex: "trocar `genpdf` por `printpdf`", "MVP sem PDF primeiro", "fazer versão web em vez de desktop").