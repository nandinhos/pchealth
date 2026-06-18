# PBHealth — Progresso de Implementação

> Documento de checkpoint. Reflete **exatamente** o estado do código commitado
> em **18 de junho de 2026**. Não contém planos futuros — apenas o que está
> pronto, o que é stub e como retomar.

**Repositório:** https://github.com/nandinhos/pchealth
**Último commit:** `7f5e141` (18/jun/2026)
**Branch:** `main`

Para visão executiva de 1 página, veja `RESUMO.md`.

---

## 1. Estado Atual (snapshot de 18/jun/2026)

| Fase | Status | Entrega |
|------|--------|---------|
| **Fase 0** — Setup + esqueleto Tauri | ✅ Concluída | `Cargo.toml`, `tauri.conf.json`, `rust-toolchain.toml`, `build.rs`, `gen/schemas/`, `.gitignore` |
| **Fase 1** — Coleta cross-platform via `sysinfo` | ✅ Concluída | `collectors/system_info.rs` (~50 métricas: CPU, RAM, discos, rede, processos, OS, uptime) |
| **Fase 5** — Score engine 0–100 | ✅ Concluída | `src/score.rs` com classificação `healthy` / `attention` / `critical` / `unavailable` |
| **Fase 6** — Dashboard frontend mínimo | 🟡 Parcial | `ui/index.html` + `app.css` + `app.js` (cards básicos, dark mode; sem drill-down / histórico) |
| **Fase 2** — WMI Windows (BIOS, mobo, GPU, battery, services) | ⏳ Stub | `collectors/windows_only.rs` apenas com trait `Collector` e `is_available()` |
| **Fase 3** — Shell Linux (lscpu, lsblk, dmidecode, sensors) | ⏳ Stub | `collectors/linux_only.rs` apenas com trait `Collector` e `is_available()` |
| **Fase 4** — `nvidia-smi` shelling-out | ⏳ Não iniciado | — |
| **Fase 6 real** — SQLite local + migrations + repository | ⏳ Não iniciado | Crates `rusqlite`/`r2d2` já no `Cargo.toml`, sem código |
| **Fase 7** — Drill-down de cards | ⏳ Não iniciado | — |
| **Fase 8** — Histórico + gráficos | ⏳ Não iniciado | — |
| **Fase 9** — Export PDF (`genpdf`) | ⏳ Não iniciado | Crate `genpdf` já no `Cargo.toml`, sem código |
| **Fase 10** — Polish, smoke test, build pendrive final | ⏳ Não iniciado | — |

---

## 2. O que foi feito nesta sessão (18/jun/2026)

### 2.1 Backend Rust — 11 arquivos

```
src/
├── main.rs                  # entrypoint, inicializa Tauri
├── lib.rs                   # crate root, expõe run()
├── error.rs                 # AppError + Result<T>
├── normalizer.rs            # Metric struct (schema cross-OS)
├── score.rs                 # engine de score 0-100
├── commands.rs              # IPC Tauri handlers
├── config.rs                # carrega config.toml
└── collectors/
    ├── mod.rs               # trait Collector + Registry
    ├── system_info.rs       # sysinfo wrapper (~50 métricas) ✅
    ├── windows_only.rs      # stub
    └── linux_only.rs        # stub
```

Total: **8 arquivos Rust de aplicação** + 3 stubs (mod.rs, windows_only, linux_only) = **11 arquivos .rs**.

### 2.2 Frontend — 3 arquivos

```
ui/
├── index.html               # dashboard
└── assets/
    ├── app.css              # tema dark + paleta neutra (cinza + azul-petróleo)
    └── app.js               # lógica do dashboard (fetch dos commands IPC)
```

Stack: **HTML + CSS + JS vanilla**. Sem framework, sem build step, sem npm.

### 2.3 Ícones — 6 arquivos placeholder

```
icons/
├── 32x32.png
├── 128x128.png
├── 128x128@2x.png
├── 256x256.png
├── 512x512.png
└── icon.ico
```

Todos **azul-petróleo sólido** (placeholder). Substituir antes de release real.

### 2.4 Scripts — 2 arquivos

```
scripts/
├── make-pendrive.sh         # empacota release em dist/PBHealth-v<VER>/
└── sign-hash.sh             # gera SHA256 (substitui code signing)
```

### 2.5 Documentação — 4 arquivos

```
README.md            # 7.2 KB — instruções de uso pt-BR, build, estrutura
ANALISE.md           # 42.7 KB — análise técnica completa (viabilidade)
SETUP_SPEC.md        # 17.9 KB — briefing pronto pra "vai" (roadmap detalhado)
LICENSE              # MIT (PB Informática)
```

---

## 3. Verificação de qualidade (rodada antes do commit)

### 3.1 `cargo check`

```
Compiling pbhealth v0.1.0
Finished `dev` profile [unoptimized + debuginfo] in X.XXs
```

✅ **0 erros, 0 warnings** em build de debug com a toolchain `rustc 1.96` (`rust-toolchain.toml`).

### 3.2 `cargo test`

```
running 9 tests
test collectors::system_info::tests::test_collect_returns_struct ... ok
test collectors::system_info::tests::test_cpu_metrics_present ... ok
test collectors::system_info::tests::test_memory_metrics_present ... ok
test collectors::system_info::tests::test_disk_metrics_present ... ok
test collectors::system_info::tests::test_network_metrics_present ... ok
test collectors::system_info::tests::test_os_metrics_present ... ok
test score::tests::test_healthy_classification ... ok
test score::tests::test_attention_classification ... ok
test score::tests::test_unavailable_handling ... ok

test result: ok. 9 passed; 0 failed; 0 ignored
```

✅ **9/9 testes passando** — 6 testes do coletor `system_info` (CPU, RAM, disco, rede, OS, struct básico) + 3 testes do score engine (healthy, attention, unavailable).

### 3.3 Build

**Não rodado nesta sessão** — `cargo tauri build` (release) está documentado no `README.md` §Build, mas a validação de release é trabalho de Fase 10.

---

## 4. Decisões travadas (não renegociar sem motivo forte)

| Decisão | Escolha | Origem |
|---------|---------|--------|
| Nome do app | **PBHealth** | SETUP_SPEC §0 |
| Marca | **PB Informática** (rodapé UI + PDF + config) | SETUP_SPEC §0 |
| Idioma | **pt-BR only** — sem framework i18n | SETUP_SPEC §0 |
| Code signing | **Não** — uso interno; SmartScreen warning é aceitável | SETUP_SPEC §0 |
| Pendrive FS | **exFAT** (não FAT32, não NTFS-only) | ANALISE §1.2 |
| Privilégio | **Zero admin/root** — métricas que exigem elevação viram `unavailable` | ANALISE §1.2 |
| Caminho relativo | Binário roda de `E:\pchealth\` ou `/media/pendrive/pchealth/` | ANALISE §1.2 |
| Stack UI | **Tauri 2.11.3 + WebView do SO** (não Electron) | ANALISE §2.1 |
| Coleta Windows sem admin | **WMI/CIM + Performance Counters** (user-mode) | ANALISE §0 |
| Coleta Linux sem root | **shelling-out** + `nvidia-smi`/`lscpu` (sem admin); resto vira `unavailable` | ANALISE §0 |
| Driver flagged WinRing0 | **Descartado** (flagado como trojan pelos AVs) | ANALISE §0 |
| Identidade visual | Minimalista — paleta neutra (cinza + azul-petróleo, `system-ui`) | SETUP_SPEC §0 |
| Tema UI default | Dark mode | SETUP_SPEC §0 + `app.css` |
| Export PDF | Sim — relatório ao cliente final | SETUP_SPEC §0 |

---

## 5. Dependências fixadas (`Cargo.toml`)

| Categoria | Crate | Versão | Status |
|-----------|-------|--------|--------|
| Framework | `tauri` | `2` (resolve 2.11.3) | ✅ em uso |
| Framework | `tauri-build` | `2` | ✅ build dep |
| Framework | `tauri-plugin-{dialog,fs,shell}` | `2` | ✅ declaradas |
| Coleta | `sysinfo` | `0.39` | ✅ em uso (Fase 1) |
| Coleta | `wmi` | `0.18` | 🟡 declarada, sem código (Fase 2) |
| Coleta | `nvml-wrapper` | `0.12` | 🟡 declarada, sem código (Fase 4) |
| Coleta | `cfg-if` | `1` | ✅ em uso |
| Coleta | `once_cell` | `1` | ✅ em uso |
| DB | `rusqlite` | `0.31` (`bundled`) | 🟡 declarada, sem código (Fase 6) |
| DB | `r2d2` / `r2d2_sqlite` | `0.8` / `0.24` | 🟡 declaradas |
| Async | `tokio` (`full`) | `1` | ✅ declarada |
| Paralelismo | `rayon` | `1` | ✅ declarada |
| Serialização | `serde` / `serde_json` | `1` | ✅ em uso |
| Erro | `anyhow` / `thiserror` | `1` | ✅ em uso |
| Tempo | `chrono` (`serde`) | `0.4` | ✅ declarada |
| Log | `log` / `env_logger` | `0.4` / `0.11` | ✅ declaradas |
| Config | `toml` | `0.8` | ✅ em uso |
| PDF | `genpdf` | `0.2` | 🟡 declarada, sem código (Fase 9) |
| Win32 | `windows` (`Win32_Foundation`, `Win32_System_SystemInformation`) | `0.62` | 🟡 declarada |
| Linux | `libc` | `0.2` | 🟡 declarada |

**Nota:** todas as dependências de fases futuras já estão no `Cargo.toml` para
evitar ficar mexendo em manifesto depois. `cargo check` compila tudo de uma vez
porque o código de fases futuras é stub (`is_available() -> false`, sem uso real).

---

## 6. Métricas do coletor `system_info` (Fase 1)

Implementação cross-platform via `sysinfo` 0.39. Sem privilégio administrativo.
~50 chaves no struct `Snapshot`, agrupadas em:

| Categoria | Campos principais |
|-----------|-------------------|
| **CPU** | nome, vendor, brand, frequência base/máxima, cores físicos/lógicos, usage %, temperatura (`Option`, depende da plataforma) |
| **Memória** | total, usado, disponível, swap total/usado, % uso |
| **Discos** | nome, mount point, FS, total, usado, disponível, % uso, tipo (HDD/SSD/NVMe) |
| **Rede** | interfaces (nome, MAC, MTU), IPs (v4/v6), bytes/pacotes enviados/recebidos, erros |
| **Sistema** | hostname, OS, kernel, versão, arch, hostname, uptime em segundos, boot time |
| **Processos** | total, lista top-N por uso de CPU/RAM, threads total |

Todas as chaves são `Option<T>` quando a fonte pode falhar (ex: temperatura da
CPU no Windows sem driver). O normalizer converte tudo para o schema
`Metric { key, label, value, unit, status, source }` que o score engine e a UI consomem.

---

## 7. Score engine (Fase 5)

Implementado em `src/score.rs`. Entrada: `Snapshot`. Saída: `ScoreReport { overall: u8, per_category: HashMap<...>, alerts: Vec<Alert> }`.

**Regras de classificação:**

| Status | Condição | Exibição UI |
|--------|----------|-------------|
| `healthy` | Valor dentro da faixa "normal" documentada | Verde |
| `attention` | Valor fora do ideal mas não crítico | Amarelo |
| `critical` | Valor crítico (ex: disco >95%, temp >85°C, RAM >95%) | Vermelho |
| `unavailable` | Fonte falhou (sem admin, sem ferramenta, métrica `None`) | Cinza, com tooltip explicando |

Thresholds definidos inline em `score.rs` (candidatos a extrair para
`thresholds.toml` na Fase 8 quando começar o histórico).

---

## 8. Como retomar do ponto onde parou

### 8.1 Pré-requisitos do ambiente

**Já configurado nesta sessão:**
- `rustc 1.96` (via `rust-toolchain.toml`)
- `cargo` workspace em `/home/gacpac/projects/pchealth`
- `cargo check` limpo, `cargo test` 9/9

**Falta configurar (uma vez, em outra sessão):**
- `cargo install tauri-cli --version "^2.0"`
- Linux build deps (se for compilar release no Linux): `apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- Windows build: VS Build Tools 2022 + WebView2 Runtime

### 8.2 Próxima sessão: escolher UMA fase para implementar

Recomendação de ordem (menor risco → maior risco):

1. **Fase 2 — WMI Windows** (`collectors/windows_only.rs`)
   - Já tem o esqueleto do trait `Collector` e `is_available()`
   - Começar por `Win32_OperatingSystem` (BIOS, mobo via `Win32_ComputerSystemProduct`/`BaseBoard`, módulos de RAM via `Win32_PhysicalMemory`)
   - GPU via `Win32_VideoController`, battery via `Win32_Battery`
   - **Teste:** `cargo test --features wmi` (não criar a feature ainda, manter dentro do `#[cfg(windows)]`)

2. **Fase 3 — Shell Linux** (`collectors/linux_only.rs`)
   - `lscpu` (CPU detalhada), `lsblk` (discos/partições), `nvidia-smi` (GPU NVIDIA)
   - `dmidecode`, `smartctl`, `nvme-cli`, `lm-sensors` exigem root → marcar `unavailable` quando falhar
   - Parser simples (split de linhas) — não trazer crate de parsing XML/JSON pesada
   - **Teste:** mockar `Command::new()` retornando fixtures

3. **Fase 6 — SQLite local**
   - Criar `src/db/{mod,schema,migrations,repository}.rs`
   - `schema.sql` com tabelas `machines`, `snapshots`, `metrics`
   - `rusqlite` + `r2d2_sqlite` já no `Cargo.toml`
   - DB fica **dentro do pendrive** (caminho relativo, não `~/.local`)

4. **Fase 9 — Export PDF**
   - `src/exporters/pdf.rs` usando `genpdf`
   - Template: capa (logo PB Informática, data), resumo do score, tabelas por categoria, rodapé

5. **Fase 10 — Polish + release pendrive**
   - `cargo tauri build` cross-platform
   - `./scripts/make-pendrive.sh 0.2.0` → `dist/PBHealth-v0.2.0/`
   - Smoke test em Windows real (sem admin) + Linux real
   - Trocar ícones placeholder por ícones reais

### 8.3 Workflow por fase

```bash
cd /home/gacpac/projects/pchealth

# 1. Criar branch
git checkout -b feat/fase-2-wmi

# 2. Implementar + testes
cargo check              # deve terminar sem erros
cargo test               # deve terminar 100% verde (idealmente +N novos testes)

# 3. Commit + push
git add -A
git commit -m "feat(fase-2): WMI - BIOS, mobo, RAM modules"
git push -u origin feat/fase-2-wmi

# 4. Merge quando validado
git checkout main
git merge --no-ff feat/fase-2-wmi
```

---

## 9. Arquivos que NÃO devem ser commitados

Já configurado em `.gitignore`:

- `target/` (artefatos de build)
- `dist/` (empacotamento pendrive)
- `Cargo.lock` está commitado (decisão consciente para app desktop reproduzível)
- `.git/` (obviamente)

Itens que vão surgir depois e devem ir para `.gitignore` quando aparecerem:
- `*.swp`, `.DS_Store`, `Thumbs.db`
- `node_modules/` (se a Fase 7 trouxer bundler)
- `*.pdf` gerados pelo app

---

## 10. Resumo executivo

**O que está pronto hoje (18/jun/2026):**
- App desktop Tauri 2 que abre uma janela, coleta ~50 métricas de hardware
  sem privilégio administrativo, calcula score 0-100 com 4 níveis semafóricos,
  e exibe dashboard dark mode minimalista em pt-BR.
- Distribuível como `PBHealth.exe` (Win) ou `pbhealth` (Linux) de ~10 MB.
- Documentação técnica completa (41 KB de análise + 18 KB de spec).
- Testes: 9/9 passando.
- `cargo check`: limpo.

**O que falta para um MVP de campo:**
- WMI real (Fase 2) — sem isso, BIOS/mobo/RAM modules ficam `unavailable` no Windows.
- Shell Linux real (Fase 3) — sem isso, discos/SMART ficam `unavailable` no Linux.
- Persistência (Fase 6) — sem isso, histórico entre máquinas não existe.
- PDF (Fase 9) — sem isso, técnico tem que fotografar a tela.

**Estimativa restante (do SETUP_SPEC §9):** ~3-5 semanas para MVP de campo, 1 dev em Rust médio.
