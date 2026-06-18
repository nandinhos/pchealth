# PBHealth — Resumo Executivo (1 página)

> **Para retomar em outra sessão, leia só este arquivo.**
> Detalhes técnicos estão em `PROGRESSO.md`, `ANALISE.md` e `SETUP_SPEC.md`.

---

## 🔗 Links

- **Repo:** https://github.com/nandinhos/pchealth
- **Diretório local:** `/home/gacpac/projects/pchealth` (WSL)
- **Branch:** `main`
- **Último commit:** `7f5e141` (18/jun/2026)

---

## ✅ O que está pronto e validado

| Componente | Status | Validação |
|---|---|---|
| Compilação | ✅ | `cargo check` limpo, 0 warnings |
| Testes | ✅ | 9/9 passando |
| Coleta de hardware | ✅ | `sysinfo` no WSL coleta ~50 métricas reais |
| Score engine | ✅ | 0-100, status healthy/attention/critical/unavailable |
| Frontend básico | ✅ | HTML+CSS+JS vanilla, dark mode, 10 cards por categoria |
| Config TOML | ✅ | Carrega/cria com defaults, persiste em disco |
| Build pipeline | ✅ | `tauri.conf.json` + `build.rs` + scripts bash |

## 🟡 O que é stub (não funciona de verdade ainda)

| Componente | Estado |
|---|---|
| `WindowsWmiCollector` | Trait pronto, retorna 1 métrica `unavailable` |
| `LinuxShellCollector` | Idem |
| `export_json` / `export_pdf` | Retornam erro "não implementado" |
| Persistência SQLite | Crates no `Cargo.toml`, sem código ainda |
| Histórico no dashboard | UI tem placeholder, sem dados |

---

## 🚀 Como retomar (em outra sessão)

```bash
cd ~/projects/pchealth

# 1. Atualizar (se mudou de máquina)
git pull

# 2. Validar estado
cargo check                    # esperado: Finished, 0 erros
cargo test --lib               # esperado: 9 passed; 0 failed

# 3. Próxima frente (escolher UMA):
#    "Fase 2: vai WMI"        → implementação Windows real
#    "Fase 4: vai nvidia-smi"  → shelling-out cross-OS
#    "Fase 6: vai SQLite"      → persistência local
#    "Fase 9: vai PDF"         → relatório ao cliente
```

---

## 🎯 Próximas frentes (em ordem de valor)

1. **Fase 4: `nvidia-smi` shelling-out** — funciona Win+Linux, cobre GPU NVIDIA inteira (temp, clock, fan, watts). Crate `nvml-wrapper` já nas deps. Estimativa: 2-3 dias.
2. **Fase 2: WMI Windows real** — BIOS, mobo, RAM módulos, GPU, battery, services. Crate `wmi` já nas deps. Estimativa: 1-2 semanas.
3. **Fase 6: SQLite local** — habilitar `db/` module com `rusqlite` + `r2d2`, gravar snapshots, listar histórico. Estimativa: 3-5 dias.
4. **Fase 9: PDF** — `genpdf` já nas deps. Template com logo PB Informática + score + tabelas. Estimativa: 1 semana.

**Total para MVP "mostrável":** ~3-5 semanas.

---

## 📦 Stack travada

- **App:** Tauri 2.11.3 + Rust 1.85+ (instalado 1.96.0)
- **Coleta:** `sysinfo` 0.39, `wmi` 0.18, `nvml-wrapper` 0.12, `windows` 0.62
- **DB:** `rusqlite` 0.31 + `r2d2` 0.8 (já no Cargo.toml)
- **PDF:** `genpdf` 0.2 (já no Cargo.toml)
- **Frontend:** HTML+CSS+JS puro (sem React/Vue), tema dark minimalista
- **Pendrive:** exFAT, sem instalador

---

## ⚠️ Gotchas descobertos (pra não cair de novo)

| Gotcha | Sintoma | Fix |
|---|---|---|
| `sysinfo 0.39` API mudou | `with_disks_list not found`, `disks() not found` | Usar `Disks::new_with_refreshed_list()` (tipo separado) |
| `physical_core_count` | `not a method` | É função ASSOCIADA: `System::physical_core_count()` |
| `tauri::generate_context!` panic | `frontendDist path doesn't exist` | Layout flat precisa `"frontendDist": "ui"` (não `"../ui"`) |
| `thiserror::Error` + Display manual | `E0782: expected a type, found a trait` | Remover `impl Display` — `thiserror` já gera |
| `#[from] rusqlite::Error` em enum com Serialize | `E0277: Serialize not satisfied` | Não derivar Serialize no AppError |
| `unwrap_or(&PathBuf::from("."))` | `E0716: temporary dropped while borrowed` | Bind em variável antes: `let f = ...; exe.parent().unwrap_or(&f)` |
| `read -r` em sandbox Hermes | Captura EOF imediato | Não-interativo: passar token em variável de env ou arquivo |

---

## 🔐 Segurança & decisões travadas

- **Sem code signing** (uso interno, gratuito) — SmartScreen pode bloquear na 1ª execução em máquinas novas
- **Pendrive exFAT** (não FAT32 limite 4GB, não NTFS-only que trava escrita no Linux)
- **Zero admin na máquina cliente** — métricas que exigem privilégio vêm como `unavailable` na UI
- **Sem rede** — app não envia nada pra lugar nenhum, fica offline
- **Marca PB Informática** no rodapé do dashboard e do PDF (configurável em `config.toml`)

---

## 📂 Estrutura do projeto

```
pbhealth/
├── RESUMO.md               ← você está aqui
├── PROGRESSO.md            ← detalhe técnico do checkpoint
├── ANALISE.md              ← análise técnica completa (42KB)
├── SETUP_SPEC.md           ← briefing de implementação (18KB)
├── README.md               ← manual de uso
├── LICENSE                 ← MIT
├── Cargo.toml              ← deps travadas
├── tauri.conf.json
├── build.rs
├── src/                    ← backend Rust
│   ├── main.rs / lib.rs / error.rs
│   ├── normalizer.rs       ← struct Metric
│   ├── score.rs            ← engine 0-100
│   ├── commands.rs         ← IPC Tauri
│   ├── config.rs           ← TOML loader
│   └── collectors/
│       ├── mod.rs          ← trait + Registry
│       ├── system_info.rs  ← ✅ funciona
│       ├── windows_only.rs ← 🟡 stub
│       └── linux_only.rs   ← 🟡 stub
├── ui/                     ← frontend
├── icons/                  ← PNG/ICO placeholders
└── scripts/
    ├── make-pendrive.sh    ← empacota release
    ├── sign-hash.sh        ← SHA256 (substitui code signing)
    └── push-to-github.sh   ← helper de autenticação
```

---

**Última atualização:** 18/jun/2026
**Próximo marco sugerido:** Fase 4 (nvidia-smi) — mais rápida e funciona nos 2 SOs.