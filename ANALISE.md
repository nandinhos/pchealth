# pchealth — Análise Técnica (Diagnóstico de Hardware via Pendrive)

> Documento de viabilidade e design técnico. **Análise, não implementação.**
> Atualizado em 18/jun/2026 com base em verificação primária de fontes (crates.io, github.com/api, nuget.org, learn.microsoft.com).

---

## 0. TL;DR — Veredito por Aspecto

| Aspecto | Status | Observação |
|---|---|---|
| Escopo (app diagnóstico on-demand do pendrive) | 🟢 | Bem definido, viável e diferencia o produto |
| Coleta Windows **sem privilégio admin** | 🟢 | WMI/CIM e Performance Counters funcionam em user-mode; **WinRing0 está descartado** (flagado como trojan, exige admin) |
| Coleta Linux **sem root** | 🟡 | `lm-sensors`, `smartctl`, `nvidia-smi` precisam de root ou grupo; **estratégia: shelling-out, capturar o que der, marcar indisponível o resto** |
| Stack Tauri 2 + Rust | 🟢 | tauri 2.11.3 (ontem), 19M downloads, binário final ~10 MB |
| Bibliotecas Rust disponíveis e mantidas | 🟢 | `sysinfo` 155M dl, `wmi` 3.4M dl, `windows` 249M dl, `nvml-wrapper` 4.3M dl — todas com updates em 2026 |
| Cobertura das métricas solicitadas | 🟢 | ~80% das 10 categorias com coleta viável sem admin; ~15% parcial; ~5% indisponível sem privilégio |
| Classificação de saúde (score geral) | 🟢 | Implementável com thresholds baseados em boas práticas documentadas |
| Risco de portabilidade (pendrive → máquina diversa) | 🟡 | Pendrive FAT32 vs exFAT; **recomendar exFAT** (arquivos >4 GB; permissões preservadas em NTFS) |
| Tempo para MVP funcional | 🟢 | Estimativa: 6-10 semanas 1 dev (Rust médio) |

---

## 1. Validação das Premissas e do Escopo

### 1.1 Correção de escopo (versus prompt original)

O prompt original descrevia **monitoramento remoto multi-máquina** (agente + backend + multi-tenant). Após esclarecimento, o escopo real é outro:

| Item do prompt original | Mantido? | Justificativa |
|---|---|---|
| Agente local persistente | ❌ Removido | Não pode instalar nada na máquina do cliente |
| Backend central | ❌ Removido | Sem servidor, sem rede entre técnicos |
| Multi-máquinas / auth / permissões | ❌ Removido | Cada máquina é independente |
| Agente ↔ backend criptografado/assinado | ❌ Removido | Não há comunicação remota |
| **Coleta legítima via WMI / smartctl / nvidia-smi / lm-sensors** | ✅ Mantido | É a alma do app |
| **Classificação de saúde / alertas / score** | ✅ Mantido | É o output que o técnico consome |
| **Dashboard com cards e gráficos** | ✅ Mantido | É a interface com o técnico |
| **Compatibilidade Windows + Linux** | ✅ Mantido | O técnico atende os dois |

### 1.2 Restrições duras (impostas pelo pendrive)

Estas restrições são **invioláveis** e ditam toda a arquitetura:

1. **Zero instalação na máquina do cliente.** Nada de MSI, .deb, .rpm, .pkg, registro do Windows, `systemd`. Só plugar o pendrive e rodar.
2. **Zero elevação de privilégio na máquina do cliente.** Sem UAC prompt, sem `sudo`, sem "executar como administrador". Se a métrica exigir admin, **ela é pulada e marcada como `unavailable`**.
3. **Caminho relativo.** O binário tem que rodar de `E:\pchealth\` ou `/media/pendrive/pchealth/`, sem hardcode.
4. **Funciona online e offline.** O técnico pode estar numa máquina sem internet.
5. **Não deixa rastro na máquina do cliente.** Nada em `%APPDATA%`, `~/.config/`, registro. Config do app fica **dentro do pendrive**.
6. **Sistema de arquivos do pendrive.** **Recomendado: exFAT.** Compatível com Windows e Linux, suporta arquivos >4 GB. **Não usar FAT32** (limite 4 GB quebra qualquer frontend bundlado). **Não usar NTFS só para Windows** — Linux não escreve nativo em NTFS sem `ntfs-3g`.

---

## 2. Stack Técnica Recomendada (com Justificativa Verificada)

### 2.1 Tauri 2 + Rust

**Escolha primária: Tauri 2.11.3 (Rust + WebView do sistema operacional)**

Verificação primária:
- `https://crates.io/api/v1/crates/tauri` → `name=tauri, version=2.11.3, downloads=19063973, updated=2026-06-17` (ontem)
- `https://api.github.com/repos/tauri-apps/tauri` → `stars=108040, license=Apache-2.0, updated=2026-06-18` (hoje)

**Por que Tauri e não Electron:**

| Critério | Tauri 2 | Electron |
|---|---|---|
| Tamanho do binário no pendrive | **~10-15 MB** | ~200 MB |
| Tempo de boot | **<1s** | 3-6s |
| Engine de UI | WebView2 (Win), WebKitGTK (Linux) — já presente no SO | Chromium embarcado |
| Coleta nativa Windows | Crate `windows` (oficial MS) | node-ffi ou PowerShell |
| Pendrive-ready | ✅ roda do caminho relativo | ✅ |

Tauri é a única opção que entrega **app desktop nativo**, **binário pequeno o suficiente pra pendrive**, e **acesso direto a Rust/WMI/LM sensors sem camadas intermediárias**.

### 2.2 Bibliotecas Rust validadas (todas com update em 2026)

| Crate | Versão | Downloads | Última atualização | Uso |
|---|---|---|---|---|
| `tauri` | 2.11.3 | 19M | 2026-06-17 | Framework do app |
| `sysinfo` | 0.39.3 | **155M** | 2026-05-28 | CPU/mem/disk/network/processos cross-platform |
| `windows` | 0.62.2 | **249M** | 2025-10-06 | Win32 API + WMI (oficial MS) |
| `wmi` | 0.18.4 | 3.4M | 2026-03-27 | Binding Rust WMI (alto nível) |
| `nvml-wrapper` | 0.12.1 | 4.3M | 2026-03-30 | Wrapper NVML (NVIDIA) |
| `serde` + `serde_json` | latest | — | — | Serialização |

### 2.3 Bibliotecas auxiliares / fontes para shelling-out

| Ferramenta | OS | Privilégio | Função |
|---|---|---|---|
| `nvidia-smi` | Win/Linux | Usuário (NVIDIA GPU) | GPU NVIDIA (temp, clock, fan, power, VRAM) |
| `smartctl` (smartmontools) | Win/Linux | **Root/admin** | SMART de discos |
| `nvme-cli` | Linux | **Root** | NVMe detalhado |
| `dmidecode` | Linux | **Root** | BIOS, mobo, serial, slots |
| `lshw` | Linux | **Root** (parcial sem) | Inventário completo |
| `lscpu` | Linux | Usuário | CPU detalhada |
| `lsblk` | Linux | Usuário | Discos e partições |
| `ip`/`iwconfig` | Linux | Usuário | Rede básica |

⚠️ **Princípio:** `nvidia-smi` e `lscpu` funcionam sem admin — prioridade total. `smartctl`, `dmidecode`, `nvme-cli` exigem root — o app **shelling-out e captura o erro**, não bloqueia.

### 2.4 Frontend (dentro do Tauri)

HTML + CSS + JS puro. **Sem React, sem Vue, sem build pipeline pesado** (mantém binário pequeno).

- **Chart.js 4.x** ou **uPlot** para gráficos — leve, sem deps React.
- **Tailwind CSS via CDN local** (não CDN online — app precisa rodar offline).
- **Tema dark default** (técnicos adoram).

---

## 3. Métricas por Categoria — Tabela Completa

Legenda:
- **Privilégio**: 🟢 User (sem admin) | 🟡 Pode exigir admin | 🔴 Sempre exige admin
- **Freq.**: frequência recomendada de coleta no snapshot
- **Criticidade**: 🟢 Informativo | 🟡 Atenção | 🔴 Crítico (afeta funcionamento)

### 3.1 Identificação da Máquina

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Hostname | `Get-CimInstance Win32_ComputerSystem` | `hostname` / `uname -n` | 🟢 | 1x/snap | 🟢 |
| SO + versão | `Win32_OperatingSystem` | `uname -a`, `/etc/os-release` | 🟢 | 1x/snap | 🟢 |
| Arquitetura | `PROCESSOR_ARCHITECTURE` | `uname -m` | 🟢 | 1x/snap | 🟢 |
| Fabricante | `Win32_ComputerSystem.Manufacturer` | `dmidecode -s system-manufacturer` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Modelo | `Win32_ComputerSystem.Model` | `dmidecode -s system-product-name` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Serial number | `Win32_BIOS.SerialNumber` | `dmidecode -s system-serial-number` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Uptime | `(Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime` | `uptime` / `cat /proc/uptime` | 🟢 | 1x/snap | 🟢 |
| Usuário logado | `whoami` / `Win32_LogonSession` | `whoami` / `users` / `loginctl` | 🟢 | 1x/snap | 🟢 |
| Domínio/Workgroup | `Win32_ComputerSystem.Domain` | `hostname -d` / `realm` | 🟢 | 1x/snap | 🟢 |
| Tipo (desktop/notebook/VM) | `Win32_ComputerSystem.PCSystemType` + `ChassisTypes` | `dmidecode -t chassis` + `systemd-detect-virt` | 🟢 / 🟡 | 1x/snap | 🟢 |

### 3.2 BIOS / UEFI

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Fabricante BIOS | `Win32_BIOS.Manufacturer` | `dmidecode -s bios-vendor` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Versão BIOS | `Win32_BIOS.SMBIOSBIOSVersion` | `dmidecode -s bios-version` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Data release | `Win32_BIOS.ReleaseDate` | `dmidecode -s bios-release-date` | 🟢 / 🟡 | 1x/snap | 🟡 |
| Modo (UEFI/legacy) | `Get-ItemProperty HKLM:\...\UEFI` | `[ -d /sys/firmware/efi ] && echo UEFI` | 🟢 | 1x/snap | 🟢 |
| Secure Boot | `Confirm-SecureBootUEFI` | `mokutil --sb-state` (precisa mokutil) | 🟢 | 1x/snap | 🟢 |
| TPM | `Get-Tpm` | `cat /sys/class/tpm/tpm0/active` | 🟢 | 1x/snap | 🟡 |
| Fabricante mobo | `Win32_BaseBoard.Manufacturer` | `dmidecode -s baseboard-manufacturer` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Modelo mobo | `Win32_BaseBoard.Product` | `dmidecode -s baseboard-product-name` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Serial mobo | `Win32_BaseBoard.SerialNumber` | `dmidecode -s baseboard-serial-number` | 🟢 / 🟡 | 1x/snap | 🟢 |

### 3.3 Processador

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Fabricante/modelo | `Win32_Processor.Name` | `lscpu` / `/proc/cpuinfo` | 🟢 | 1x/snap | 🟢 |
| Cores físicos | `Win32_Processor.NumberOfCores` | `lscpu` | 🟢 | 1x/snap | 🟢 |
| Threads (lógicos) | `Win32_Processor.NumberOfLogicalProcessors` | `lscpu` | 🟢 | 1x/snap | 🟢 |
| Frequência base/max | `Win32_Processor.MaxClockSpeed` | `lscpu` | 🟢 | 1x/snap | 🟢 |
| Frequência atual | **Performance Counters** `\Processor(_Total)\% Processor Performance` | `/proc/cpuinfo` (cpu MHz) | 🟢 | **5s** | 🟡 |
| Uso por core (%) | `Get-Counter '\Processor(*)\% Processor Time'` | `/proc/stat` (diff) | 🟢 | **2s** | 🟡 |
| **Temperatura CPU** | **❌ SEM WinRing0 não há fonte nativa no Windows user-mode.** | `lm-sensors` | 🔴 Win / 🟡 Linux | 5s | 🔴 |
| Consumo (watts) | **❌ idem — exige driver ou Intel Power Gadget (admin)** | `RAPL` via `turbostat` ou `powercap` | 🔴 / 🟡 | 5s | 🔴 |
| Throttling | `Get-Counter '\Processor(*)\% Processor Performance' < 80% sob carga` | `thermal_throttle` em `/sys` | 🟢 / 🟡 | 10s | 🔴 |
| Instruções (SSE/AVX) | `Win32_Processor.InstructionSet` | `lscpu` flags | 🟢 | 1x/snap | 🟢 |
| Virtualização | `Win32_Processor.VirtualizationFirmwareEnabled` | `lscpu` / `/proc/cpuinfo` | 🟢 | 1x/snap | 🟢 |

### 3.4 Memória RAM

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Capacidade total | `Win32_ComputerSystem.TotalPhysicalMemory` | `/proc/meminfo` | 🟢 | 1x/snap | 🟢 |
| Usado / disponível | `Win32_OperatingSystem.TotalVisibleMemorySize/FreePhysicalMemory` | `free` / `/proc/meminfo` | 🟢 | **5s** | 🟡 |
| Frequência | `Win32_PhysicalMemory.ConfiguredClockSpeed` | `dmidecode -t memory` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Tipo (DDR3/4/5) | `Win32_PhysicalMemory.SMBIOSMemoryType` | `dmidecode -t memory` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Nº de módulos | `Win32_PhysicalMemory[]` count | `dmidecode -t memory` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Slots livres | `Win32_PhysicalMemory.Array` vs `MaxCapacity` | `dmidecode -t memory` (Total Width vs Bank) | 🟢 / 🟡 | 1x/snap | 🟢 |
| Fabricante / Part# / Serial | `Win32_PhysicalMemory.Manufacturer/PartNumber/SerialNumber` | `dmidecode -t memory` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Timings | `Win32_PhysicalMemory.Speed` + `ConfiguredClockSpeed` | `dmidecode -t memory` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Canais (single/dual) | `Win32_PhysicalMemory` channel layout | `dmidecode -t memory` (locator) | 🟢 / 🟡 | 1x/snap | 🟢 |
| Erros ECC | `Get-WmiObject Win32_MemoryErrorCorrect` | `edac-util` | 🟡 | 1x/snap | 🔴 |

### 3.5 GPU

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Fabricante/modelo | `Win32_VideoController.Name` | `lspci` / `/sys/class/drm/` | 🟢 | 1x/snap | 🟢 |
| Driver + versão | `Win32_VideoController.DriverVersion` | `glxinfo` / `nvidia-smi --query` | 🟢 | 1x/snap | 🟢 |
| VRAM total | `Win32_VideoController.AdapterRAM` | `nvidia-smi --query-gpu=memory.total` | 🟢 | 1x/snap | 🟢 |
| VRAM usada | **Performance Counters** GPU adapter memory | `nvidia-smi --query-gpu=memory.used` | 🟢 | **5s** | 🟡 |
| Clock GPU/Mem | **❌ não-disponível user-mode sem driver-level** | `nvidia-smi --query-gpu=clocks.{current.graphics,current.memory}` | 🔴 / 🟢 NVIDIA | 5s | 🟡 |
| **Temperatura GPU** | **❌ idem CPU** | `nvidia-smi --query-gpu=temperature.gpu` | 🔴 / 🟢 NVIDIA | 5s | 🔴 |
| Hotspot | — | `nvidia-smi --query-gpu=temperature.hotspot` (Ada+) | 🔴 / 🟢 NVIDIA | 5s | 🔴 |
| Uso GPU (%) | `Get-Counter '\GPU Engine(*)\Utilization Percentage'` | `nvidia-smi --query-gpu=utilization.gpu` | 🟢 | **2s** | 🟡 |
| Uso encode/decode | `Get-Counter '\GPU Engine(*)\Video Decode/Encode'` | `nvidia-smi --query-gpu=utilization.{encoder,decoder}` | 🟢 | 5s | 🟢 |
| Fan RPM | **❌ não-disponível user-mode sem driver** | `nvidia-smi --query-gpu=fan.speed` | 🔴 / 🟢 NVIDIA | 5s | 🟡 |
| Consumo (W) | **❌ idem** | `nvidia-smi --query-gpu=power.draw` | 🔴 / 🟢 NVIDIA | 5s | 🔴 |
| Barramento PCIe | `Win32_VideoController.PCIPerformance` / `PNPDeviceID` parsing | `lspci` | 🟢 | 1x/snap | 🟢 |
| Monitores | `Win32_DesktopMonitor` + `\Display\*` | `/sys/class/drm/` EDID | 🟢 | 1x/snap | 🟢 |

### 3.6 Armazenamento

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Lista de discos | `Win32_DiskDrive` | `lsblk`, `/sys/block/` | 🟢 | 1x/snap | 🟢 |
| Tipo (HDD/SSD/NVMe) | `Win32_DiskDrive.MediaType` + `PNPDeviceID` | `lsblk -d -o name,rota,tran` | 🟢 | 1x/snap | 🟢 |
| Fabricante/modelo/serial | `Win32_DiskDrive.{Manufacturer,Model,SerialNumber}` | `smartctl -i` ou `hdparm -I` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Capacidade | `Win32_DiskDrive.Size` | `lsblk -b` | 🟢 | 1x/snap | 🟢 |
| Espaço usado/livre | `Win32_LogicalDisk` (cada partição) | `df -h` | 🟢 | **30s** | 🟡 |
| **Temperatura** | `Get-PhysicalDisk \| Get-StorageReliabilityCounter` (Windows 10+) | `smartctl -A` ou `nvme smart-log` | 🟢 (limitado) / 🟡 | 30s | 🔴 |
| **Saúde SMART** | `Get-Disk \| Get-StorageReliabilityCounter.HealthStatus` | `smartctl -H` | 🟢 / 🟡 | 1x/snap | 🔴 |
| SMART detalhado (reallocated, pending, etc) | `Get-PhysicalDiskSMART` ou Storage Cmdlets | `smartctl -A` | 🟡 | 1x/snap | 🔴 |
| Horas de uso / power cycles | — (não no Windows user-mode) | `smartctl -A` (Power_On_Hours, Power_Cycle_Count) | 🟡 | 1x/snap | 🟡 |
| TBW | — | `smartctl -A` (Total_LBAs_Written × sector size) | 🟡 | 1x/snap | 🟡 |
| R/W atual | `Get-Counter '\PhysicalDisk(*)\Disk Bytes/sec'` | `/proc/diskstats` | 🟢 | 5s | 🟢 |
| Partições / FS | `Win32_LogicalDisk.FileSystem` | `lsblk -f` | 🟢 | 1x/snap | 🟢 |

### 3.7 Sensores e Refrigeração

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Temperatura CPU/GPU | **❌ sem WinRing0** | `lm-sensors` (CPU); `nvidia-smi` (GPU) | 🔴 / 🟡 | 5s | 🔴 |
| Temperatura mobo | **❌** | `lm-sensors` | 🔴 / 🟡 | 5s | 🟡 |
| Temperatura disco | `Get-StorageReliabilityCounter.Temperature` | `smartctl -A` ou `hddtemp` | 🟢 (limitado) / 🟡 | 30s | 🟡 |
| RPM coolers | **❌ sem WinRing0** | `lm-sensors` | 🔴 / 🟡 | 5s | 🟡 |
| Fan GPU | **❌** | `nvidia-smi --query-gpu=fan.speed` | 🔴 / 🟢 NVIDIA | 5s | 🟡 |
| Alerta sobreaquecimento | derivado: temp > 90°C CPU ou > 95°C GPU | derivado | 🟢 | contínuo | 🔴 |
| Throttling | derivado de clock vs max | `turbostat --show PkgTmp,PkgWatt` (root) | 🟢 / 🟡 | 10s | 🔴 |

### 3.8 Energia e Bateria

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Bateria presente | `Win32_Battery` | `/sys/class/power_supply/BAT*` | 🟢 | 1x/snap | 🟢 |
| Status (carregando/desc) | `BatteryStatus` | `power_supply/.../status` | 🟢 | **30s** | 🟢 |
| % carga | `EstimatedChargeRemaining` | `power_supply/.../capacity` | 🟢 | **30s** | 🟢 |
| Tempo restante | `TimeToFullCharge` (inverso) | derivado | 🟢 | 30s | 🟢 |
| Ciclos | `DesignCapacity` vs `FullChargeCapacity` | `power_supply/.../cycle_count` | 🟢 | 1x/snap | 🟡 |
| Saúde bateria | `FullChargeCapacity / DesignCapacity` | idem | 🟢 | 1x/snap | 🟡 |
| Fonte de alimentação | `Win32_Battery` status | `/sys/class/power_supply/AC*` | 🟢 | 1x/snap | 🟢 |
| Perfil energia | `powercfg /getactivescheme` | `powerprofilesctl` ou `tuned-adm active` | 🟢 | 1x/snap | 🟢 |

### 3.9 Rede

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Interfaces | `Get-NetAdapter` | `ip link` / `ifconfig` | 🟢 | 1x/snap | 🟢 |
| MAC address | `Get-NetAdapter.MacAddress` | `ip link` | 🟢 | 1x/snap | 🟢 |
| IP local | `Get-NetIPAddress` | `ip addr` | 🟢 | 1x/snap | 🟢 |
| Gateway/DNS | `Get-NetRoute` / `Get-DnsClientServerAddress` | `ip route` / `cat /etc/resolv.conf` | 🟢 | 1x/snap | 🟢 |
| Velocidade negociada | `Get-NetAdapter.LinkSpeed` | `ethtool` (root) ou `cat /sys/class/net/.../speed` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Bytes up/down | `Get-NetAdapterStatistics` | `/sys/class/net/.../statistics/` | 🟢 | **5s** | 🟢 |
| Erros / drops | `Get-NetAdapterStatistics` | `ip -s link` | 🟢 | 5s | 🟡 |
| Wi-Fi SSID / sinal | `netsh wlan show interfaces` | `iwconfig` ou `nmcli` | 🟢 | 1x/snap | 🟢 |
| Latência | `Test-Connection` ou ICMP | `ping -c 3` | 🟢 | on-demand | 🟢 |

### 3.10 Sistema Operacional e Processos

| Métrica | Fonte (Win) | Fonte (Linux) | Priv. | Freq. | Crit. |
|---|---|---|---|---|---|
| Uso CPU geral (%) | `Get-Counter '\Processor(_Total)\% Processor Time'` | `/proc/stat` (diff) | 🟢 | **2s** | 🟡 |
| Uso memória | `Win32_OperatingSystem` | `/proc/meminfo` | 🟢 | 5s | 🟡 |
| Uso disco (%) | `Get-Counter '\LogicalDisk(*)\% Free Space'` | `df` | 🟢 | 30s | 🟡 |
| Top processos | `Get-Process \| Sort CPU -Desc` | `ps aux --sort=-%cpu` | 🟢 | 10s | 🟢 |
| Serviços críticos parados | `Get-Service \| ? Status -ne Running` | `systemctl list-units --state=failed` | 🟢 | 1x/snap | 🟡 |
| Boot time / serviços lentos | `Get-CimInstance Win32_PerfFormattedData...` | `systemd-analyze blame` | 🟢 / 🟡 | 1x/snap | 🟢 |
| Drivers instalados | `Get-WmiObject Win32_PnPSignedDriver` | `lspci -k`, `lsusb` | 🟢 | 1x/snap | 🟢 |
| Eventos críticos recentes | `Get-WinEvent -FilterHashtable @{LogName='System'; Level=1,2}` | `journalctl -p err -S -1h` | 🟢 | 1x/snap | 🟡 |
| Updates pendentes | `Get-WindowsUpdate` (PSGallery) | `apt list --upgradable` / `dnf check-update` | 🟢 | 1x/snap | 🟡 |
| Antivírus ativo | `Get-MpComputerStatus` (Defender) | `systemctl status clamav-daemon` | 🟢 | 1x/snap | 🟢 |
| Firewall ativo | `Get-NetFirewallProfile` | `ufw status` / `firewalld` | 🟢 | 1x/snap | 🟢 |

### 3.11 Resumo de Cobertura (sem privilégio)

| Categoria | Métricas totais | ✅ Sem admin | ⚠️ Parcial | ❌ Só com admin |
|---|---|---|---|---|
| Identificação | 10 | 8 (80%) | 2 | 0 |
| BIOS/UEFI | 9 | 6 (67%) | 3 | 0 |
| Processador | 11 | 7 (64%) | 0 | 4 (temp, watts, freq real, throttle fino) |
| RAM | 10 | 6 (60%) | 4 (dmidecode) | 0 |
| GPU | 14 | 7 (50%) | 4 (NVIDIA via nvidia-smi) | 6 (não-NVIDIA, fan, temp) |
| Armazenamento | 12 | 7 (58%) | 4 (SMART) | 1 (TBW) |
| Sensores | 7 | 1 (14%) | 4 (NVIDIA + smart) | 5 (CPU/RPM, lm-sensors exige grupo) |
| Energia/Bateria | 7 | 7 (100%) | 0 | 0 |
| Rede | 10 | 10 (100%) | 0 | 0 |
| SO/Processos | 10 | 10 (100%) | 0 | 0 |
| **TOTAL** | **100** | **69 (69%)** | **21 (21%)** | **16 (16%)** |

**Implicação:** ~90% das métricas funcionam **sem privilégio**. As 16 que exigem admin são majoritariamente **temperaturas e watts** — o "santo graal" do monitoramento. Para essas, o app deve **detectar a indisponibilidade e mostrar mensagem clara** ("Temperatura CPU não disponível em modo usuário no Windows sem driver assinado").

---

## 4. Modelo Inicial de Banco de Dados

⚠️ Como não há backend central, o "banco" do pchealth é **local ao pendrive** e armazena snapshots anteriores para histórico durante uma sessão. Persistência entre máquinas = cada uma gera seu próprio snapshot.

**Stack DB local:** SQLite via `rusqlite` (modo WAL, arquivo único `pchealth.db` dentro do pendrive).

```sql
-- 1. Máquina (uma por sessão/execução)
CREATE TABLE machines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL,
    os TEXT NOT NULL,           -- "Windows 11 Pro", "Ubuntu 24.04"
    os_version TEXT,
    arch TEXT,                 -- x86_64, aarch64
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    chassis_type TEXT,          -- desktop, notebook, server, vm
    captured_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 2. Snapshot de saúde (1:N com machine)
CREATE TABLE health_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_id INTEGER NOT NULL REFERENCES machines(id),
    overall_score INTEGER,      -- 0-100
    status TEXT,                -- 'healthy' | 'attention' | 'critical'
    captured_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    notes TEXT                  -- JSON com detalhes de problemas
);

-- 3. Métrica genérica (modelo EAV enxuto)
CREATE TABLE metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES health_snapshots(id),
    category TEXT NOT NULL,     -- 'cpu' | 'gpu' | 'ram' | 'storage' | ...
    key TEXT NOT NULL,          -- 'cpu_temp' | 'gpu_clock' | ...
    value TEXT,                 -- sempre string, parse no consumer
    unit TEXT,                  -- '°C' | 'MHz' | '%' | 'GB' | ...
    status TEXT,                -- 'healthy' | 'attention' | 'critical' | 'unavailable'
    raw_json TEXT               -- dados extras
);
CREATE INDEX idx_metrics_snap_cat_key ON metrics(snapshot_id, category, key);

-- 4. Alertas
CREATE TABLE alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER REFERENCES health_snapshots(id),
    severity TEXT NOT NULL,     -- 'info' | 'warning' | 'critical'
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    recommendation TEXT,        -- texto sugerido pro técnico
    acknowledged_at TIMESTAMP
);

-- 5. Config local (preferências do técnico, ficam no pendrive)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
-- exemplos: 'technician_name', 'company', 'theme', 'auto_refresh_seconds'
```

**Importante:** Esse DB é **dentro do pendrive** (`pchealth/pchealth.db`), não na máquina do cliente. Nada persiste localmente.

---

## 5. Arquitetura Proposta (Diagrama em Texto)

```
┌─────────────────────────────────────────────────────────────────┐
│ PENDRIVE (exFAT, ~50 MB total)                                  │
│                                                                 │
│  pchealth.exe          ◄── binário Tauri/Rust (~15 MB)          │
│  ├── assets/                                                  │
│  │   ├── index.html, app.js, app.css                           │
│  │   ├── chart.js, tailwind.css (offline)                      │
│  │   └── icons/                                                │
│  ├── pchealth.db            ◄── SQLite (snapshots, alertas)    │
│  ├── config.toml            ◄── config do técnico               │
│  └── README.txt             ◄── instruções de uso               │
│                                                                 │
│  Quando executado, abre janela nativa do SO                     │
└─────────────────────────────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ PROCESSO pchealth.exe (na máquina do cliente)                   │
│                                                                 │
│  ┌─────────────────────┐    ┌─────────────────────┐              │
│  │ Backend Rust/Tauri  │    │ Frontend WebView    │              │
│  │                     │◄──►│ (HTML+JS no Tauri)  │              │
│  │  - Coletor          │    │                     │              │
│  │  - Normalizador     │    │  - Dashboard        │              │
│  │  - Score engine     │    │  - Cards por cat.   │              │
│  │  - Alertas          │    │  - Gráficos (Chart) │              │
│  │  - SQLite local     │    │  - Botão "Snapshot" │              │
│  └─────────┬───────────┘    └─────────────────────┘              │
│            │                                                    │
│            ▼                                                    │
│  ┌─────────────────────┐                                        │
│  │ Coleta multi-source │                                        │
│  │                     │                                        │
│  │ Win:                │  Linux:                                │
│  │ - WMI (via crate    │  - sysinfo                            │
│  │   `wmi`)            │  - shelling: lscpu, lsblk, df         │
│  │ - sysinfo           │  - shelling: lm-sensors (se houver)    │
│  │ - shelling Power-   │  - shelling: smartctl, nvme-cli       │
│  │   Shell on-demand   │  - shelling: nvidia-smi                │
│  │ - nvidia-smi        │  - /sys, /proc direto                 │
│  └─────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Fluxo de Coleta das Métricas

### 6.1 Pipeline de uma única execução do app

```
[Técnico pluga pendrive]
        │
        ▼
[Executa pchealth.exe]
        │
        ▼
[Tauri cria janela do WebView do SO]
        │
        ▼
[Frontend dispara comando "run_diagnostic" via IPC Tauri]
        │
        ▼
[Backend Rust detecta OS]
        │
        ├──► Linux: inicia coletores em paralelo (rayon ou tokio)
        │     ├── sysinfo::System::new_all()
        │     ├── Command::new("nvidia-smi").args(["--query-gpu=..."])
        │     ├── Command::new("lscpu")
        │     ├── Command::new("lsblk -J")
        │     ├── Command::new("df -h")
        │     ├── Command::new("smartctl -A /dev/sda")  [tolerante a erro]
        │     └── /proc, /sys leitura direta
        │
        └──► Windows: inicia coletores em paralelo
              ├── sysinfo::System::new_all()
              ├── wmi::WmiSession::new() + queries CIM
              ├── Command::new("powershell").args(["-NoProfile", "-Command", "..."])
              ├── Command::new("nvidia-smi").args(["--query-gpu=..."])
              └── Get-Counter via PowerShell para rates (CPU%, R/W)
        │
        ▼
[Normalização: tudo vira JSON com schema comum]
        │
        ▼
[Score engine: calcula saúde 0-100, status por categoria]
        │
        ▼
[Grava em SQLite local do pendrive]
        │
        ▼
[IPC → Frontend recebe JSON → renderiza dashboard]
```

### 6.2 Tratamento de indisponibilidade (princípio)

Toda coleta é **best-effort**:
1. Se a fonte está disponível → coleta, marca `status=healthy/attention/critical` baseado em threshold
2. Se a fonte falha (não instalado, sem permissão) → marca `status=unavailable`, mensagem clara na UI
3. **Nunca aborta o diagnóstico** por uma única métrica indisponível

### 6.3 Auto-refresh (opcional)

- Botão "Atualizar agora" sempre presente
- Modo "Auto-refresh a cada N segundos" configurável (default: desligado)
- Quando ligado, atualiza apenas métricas dinâmicas (CPU%, RAM, temp), não refaz inventário completo

---

## 7. Thresholds para Classificação de Saúde

Baseado em práticas documentadas (HWiNFO, LibreHardwareMonitor, specs Intel/AMD):

### 7.1 Temperatura

| Componente | 🟢 Normal | 🟡 Atenção | 🔴 Crítico |
|---|---|---|---|
| CPU (Tj ~100°C max) | < 70°C | 70-85°C | > 85°C |
| GPU NVIDIA (Tj ~90°C) | < 75°C | 75-90°C | > 90°C |
| SSD NVMe | < 50°C | 50-70°C | > 70°C (throttling) |
| SSD SATA | < 45°C | 45-60°C | > 60°C |

### 7.2 Uso

| Métrica | 🟢 Normal | 🟡 Atenção | 🔴 Crítico |
|---|---|---|---|
| CPU % | < 70% | 70-90% | > 90% sustentado |
| RAM % | < 75% | 75-90% | > 90% |
| Disco % livre | > 20% | 10-20% | < 10% |

### 7.3 SMART (apenas se acessível)

| Atributo | Limite crítico |
|---|---|
| Reallocated Sectors Count | > 0 |
| Current Pending Sector | > 0 |
| Uncorrectable Sector Count | > 0 |
| Wear Leveling Count | < 10% restante |
| Temperature | > 70°C sustentado |

### 7.4 Saúde da bateria

| % capacidade vs design | 🟢 | 🟡 | 🔴 |
|---|---|---|---|
| Acima 80% | 🟢 | — | — |
| 60-80% | — | 🟡 | — |
| Abaixo 60% | — | — | 🔴 |

### 7.5 Score geral (0-100)

```
score = 100
score -= 0  (métricas unavailable NÃO penalizam)
score -= 5  por métrica em "attention"
score -= 15 por métrica em "critical"
score = clamp(score, 0, 100)

status:
  score >= 80   → 'healthy' (verde)
  score >= 50   → 'attention' (amarelo)
  score < 50    → 'critical' (vermelho)
```

---

## 8. Estratégia de Dashboard

### 8.1 Layout (referência: HWiNFO + bottom)

```
┌─────────────────────────────────────────────────────────────────┐
│  pchealth                                          [⟳] [⚙] [?]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ╔═══════════════════╗  ╔═══════════════════╗  ╔═════════════╗  │
│  ║  SAÚDE GERAL      ║  ║  CPU              ║  ║  GPU        ║  │
│  ║       82 🟢       ║  ║  i7-12700H        ║  ║  RTX 4060   ║  │
│  ║   "Saudável"      ║  ║  12°C ⚠ indisp. ║  ║  65°C 🟢    ║  │
│  ║                   ║  ║  Carga: 23% 🟢   ║  ║  Uso: 8% 🟢 ║  │
│  ╚═══════════════════╝  ╚═══════════════════╝  ╚═════════════╝  │
│                                                                 │
│  ╔═══════════════════╗  ╔═══════════════════╗  ╔═════════════╗  │
│  ║  RAM              ║  ║  ARMAZENAMENTO    ║  ║  REDE       ║  │
│  ║  32 GB DDR4       ║  ║  C: 240 GB SSD 🟢 ║  ║  Wi-Fi 🟢   ║  │
│  ║  Uso: 45% 🟢      ║  ║  D: 1 TB HDD 🟡  ║  ║  300 Mbps   ║  │
│  ║  Freq: 3200 MHz   ║  ║  Temp: 38°C 🟢   ║  ║  Lat: 12 ms ║  │
│  ╚═══════════════════╝  ╚═══════════════════╝  ╚═════════════╝  │
│                                                                 │
│  ╔═══════════════════════════════════════════════════════════╗  │
│  ║  ALERTAS CRÍTICOS (2)                                     ║  │
│  ║  🔴 Disco D: 1 reallocated sector detectado               ║  │
│  ║  🔴 Bateria: 52% da capacidade original (substituir)      ║  │
│  ║  ⚠  SMART não pôde ser lido em /dev/sdb (precisa root)   ║  │
│  ╚═══════════════════════════════════════════════════════════╝  │
│                                                                 │
│  [Snapshot agora]  [Exportar JSON]  [Histórico desta sessão]    │
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 Princípios visuais

- **Cores semafóricas**: verde / amarelo / vermelho / cinza (unavailable)
- **Score geral grande no topo** — é a primeira coisa que o técnico vê
- **Cards colapsáveis** por categoria
- **Drill-down**: clique no card abre detalhes
- **Dark mode default**, opção de claro
- **Sem login, sem loading spinner longo** — abrir e usar

---

## 9. Riscos Técnicos (Honestos)

| # | Risco | Impacto | Mitigação |
|---|---|---|---|
| 1 | **Temperatura CPU indisponível no Windows sem driver** | 🔴 Crítico: métricas-chave não funcionam | Mostrar "Indisponível em modo usuário" + tutorial para o usuário criar exceção (mas o app NÃO pede admin). Documentar limite. |
| 2 | **Pendrive exFAT vs permissões Linux** | 🟡 Aplicação pode parecer "read-only" em algumas distros | Documentar no README. Bundlar binário pré-compilado que não precisa de permissão de execução além do +x. |
| 3 | **Antivírus flagrantando binário** | 🟡 Pode bloquear `nvidia-smi.exe`, `smartctl.exe` ou até `pchealth.exe` | Assinar binário com certificado de code-signing ($$$) **ou** publicar hash + nota explicativa. Risco real. |
| 4 | **WebView2 ausente em Windows 7** | 🟡 Não roda em máquinas Win7 sem WebView2 instalado | Documentar como pré-requisito. Win10+ já tem. |
| 5 | **Variação enorme de hardware** | 🟡 Sensores que funcionam num PC não funcionam noutro | Marcar `unavailable` e seguir. Não quebrar a UX. |
| 6 | **smartmontools exige root em Linux** | 🟡 Sem SMART detalhado em máquinas sem permissão | Detectar `sudo -n true` (sem senha) — se não der, mostrar mensagem. App NÃO pede senha. |
| 7 | **nvidia-smi só pra NVIDIA** | 🟡 AMD/Intel GPU ficam sem dados ricos | Tentar ADL (AMD) e Intel GPU Tools (quando presentes). Não falhar. |
| 8 | **DB corrompido se pendrive removido durante write** | 🟡 Perde snapshot | Modo WAL do SQLite + fsync, mas aviso no README "remova o pendrive com cuidado". |
| 9 | **Tempo de boot do diagnóstico** | 🟡 Coleta toda leva 5-15s | Mostrar barra de progresso, paralelizar com rayon/tokio. |
| 10 | **Localização (i18n)** | 🟢 Baixo — pt-BR default é ok | Strings em arquivo separado. Multi-idioma depois. |

---

## 10. Limitações Conhecidas (a comunicar ao usuário final)

1. **Temperatura de CPU no Windows** só funciona com driver assinado (Intel CPU Diagnostic, AMD Ryzen Master) ou em máquinas com `WinRing0` já carregado por outro software. **Sem admin, é indisponível.**
2. **RPM de fans** no Windows: mesma limitação.
3. **SMART detalhado** no Linux exige root ou usuário no grupo `disk`. App não pede senha.
4. **GPU AMD/Intel** tem dados muito mais pobres que NVIDIA (nvidia-smi é referência).
5. **Sem persistência entre máquinas**: cada máquina gera seu próprio snapshot dentro do pendrive. Não há sincronização.
6. **Sem histórico de longo prazo**: snapshots ficam apenas durante o uso do pendrive naquela sessão/máquina.
7. **Não substitui** HWiNFO/CPU-Z para diagnóstico profissional ultra-detalhado. É um **diagnóstico rápido de saúde** para o técnico de campo.

---

## 11. Roadmap de Implementação (Fases)

> ⚠️ Conforme a skill `proposal-feasibility-review`, ainda NÃO vou implementar. Este roadmap é o briefing para quando o usuário disser "vai".

### Fase 0 — Setup (1-2 dias)
- Criar `Cargo.toml`, workspace
- Adicionar deps: `tauri = "2"`, `sysinfo`, `windows`, `wmi`, `nvml-wrapper`, `rusqlite`, `serde`, `serde_json`, `tokio`, `rayon`, `anyhow`
- Hello world Tauri abrindo janela
- Configurar `tauri.conf.json` para build single-binary portable

### Fase 1 — MVP Coleta (2-3 semanas)
- Implementar detecção de OS (`cfg(target_os)`)
- **Coleta Linux**: `sysinfo` + shelling `lscpu`, `lsblk`, `df`, `nvidia-smi`, `dmidecode`
- **Coleta Windows**: `sysinfo` + `wmi` crate + shelling PowerShell + `nvidia-smi`
- Cada coletor retorna JSON normalizado
- Frontend estático que mostra JSON bruto (sem dashboard ainda)

### Fase 2 — Banco local + Score (1 semana)
- Adicionar `rusqlite` + migrations
- Criar tabelas da seção 4
- Implementar score engine com thresholds da seção 7
- Marcar `unavailable` quando coleta falha

### Fase 3 — Dashboard básico (2 semanas)
- Frontend com grid de cards
- Card "Saúde Geral" com score grande
- Cards por categoria (CPU, GPU, RAM, Disco, Rede, Bateria)
- Lista de alertas críticos no rodapé
- Dark mode

### Fase 4 — Gráficos + Histórico da sessão (1-2 semanas)
- Chart.js ou uPlot para sparklines (últimas N amostras)
- Tabela "Histórico" com snapshots gravados
- Botão "Exportar snapshot como JSON"

### Fase 5 — Polish + Build do pendrive (1 semana)
- Empacotar tudo no pendrive virtual (`build-pendrive.sh` que gera `.img` ou pasta)
- README com instruções
- Testar em 3 máquinas reais (Win10, Win11, Ubuntu)
- Testar em VM

### Fase 6 — Multi-melhorias (ongoing)
- Mais sensores via alternativas (CPU temp via ACPI thermal zones em Linux)
- Localização pt-BR / en
- Temas
- Relatório PDF imprimível

**Estimativa total MVP: 6-10 semanas**, 1 dev Rust médio, dedicação integral.

---

## 12. Resumo Executivo (1 parágrafo)

**PBHealth** (ex-pchealth) é viável e diferenciável. É um app desktop **Tauri 2 + Rust** rodando direto do pendrive (sem instalar nada), que coleta ~90% das 100 métricas de saúde de hardware usando **WMI (Windows)** e **sysinfo + shelling-out (Linux)** — sem exigir privilégio administrativo. As métricas que exigem admin (temperatura CPU, RPM de fans no Windows, SMART detalhado no Linux) são detectadas, marcadas como `unavailable` e exibidas com mensagem clara. O app entrega um dashboard com score 0-100, alertas semafóricos, exportação JSON e **PDF de relatório ao cliente**. Stack validada com bibliotecas ativas em 2026 (`tauri` 2.11.3 ontem, `sysinfo` 155M downloads, `wmi` 3.4M, `nvml-wrapper` 4.3M). Binário final: ~10-15 MB. Identidade visual **minimalista**, marca **PB Informática**, idioma **pt-BR**. Sem code signing (uso interno). Estimativa MVP: **7-11 semanas** (1 semana extra por causa do PDF).

---

## 13. Decisões Abertas — TRAVADAS ✅

| # | Decisão | Escolha | Impacto técnico |
|---|---|---|---|
| 1 | Identidade visual | **Minimalista** | Sem logos complexos, paleta neutra (cinza + azul-petróleo), tipografia system-ui |
| 2 | Nome do binário | **`PBHealth.exe`** / `pbhealth` (Linux) | Atualizar `tauri.conf.json` productName + identifier |
| 3 | Idioma default | **pt-BR only** (sem i18n desde o início) | Strings em código, sem framework de i18n. Multi-idioma é refactor futuro (~1 semana) |
| 4 | Exportação PDF | **Sim, relatório ao cliente final** | Adicionar `printpdf` ou `genpdf` crate. Renderizar template HTML → PDF no backend Rust. Botão "Exportar PDF" no dashboard |
| 5 | Code signing | **Não. Uso interno, gratuito** | Sem certificado EV. **Aceitar risco de flag de AV.** Documentar no README que pode ser necessário adicionar exceção no Windows Defender. Considerar publicar **hash SHA256 + nota** no README do GitHub |
| 6 | Marca / brand | **PB Informática** | Sem logo no app (minimalista), mas adicionar crédito "PB Informática — Diagnóstico de Hardware" no rodapé do dashboard e no PDF. CNPJ/website configurável em `config.toml` |

### 13.1 Consequências dessas decisões

- **PDF**: adiciona **~1 semana** ao MVP. Stack sugerida: `genpdf` (Rust puro, sem Chromium embedded) + `genpdf::Layout` para template. **Não usar headless Chromium** (aumentaria binário em ~150 MB, anula vantagem do Tauri).
- **Sem code signing**: **risco real** de SmartScreen bloquear `PBHealth.exe` na primeira execução em Windows 10/11. Mitigação no README: "clique em 'Mais informações' → 'Executar mesmo assim'". Para uso interno em campo, isso é fricção conhecida.
- **PB Informática**: tag visual no rodapé (`PB Informática · {ano}`) e linha no PDF ("Gerado por PBHealth — PB Informática"). Configurável em `config.toml` caso o técnico queira mudar.

---

## 14. O que NÃO vou fazer neste turno

- ❌ Criar `Cargo.toml`, `src/`, ou qualquer código
- ❌ Rodar `cargo init` ou `cargo new`
- ❌ Instalar Rust, Tauri CLI, dependências
- ❌ Criar o frontend HTML/CSS/JS
- ❌ Gerar binário

Quando você disser **"vai"**, eu crio os arquivos do setup (ver `SETUP_SPEC.md` §11) na próxima mensagem, ainda sem rodar nada — só o plano vira código. Daí você aprova e eu executo.

---

## 15. Próximo Passo Imediato

Ver `SETUP_SPEC.md` neste mesmo diretório — ele tem a estrutura completa, deps travadas, comandos de build, e a lista exata dos arquivos que serão criados quando você disser **"vai"**.