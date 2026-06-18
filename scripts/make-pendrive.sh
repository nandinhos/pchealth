#!/usr/bin/env bash
# PBHealth — empacota o build release numa pasta pronta para gravar
# no pendrive exFAT.
#
# Uso:
#   ./scripts/make-pendrive.sh                # usa versão do Cargo.toml
#   ./scripts/make-pendrive.sh 0.2.0          # versão custom
#
# Saída:
#   dist/PBHealth-v<VER>/PBHealth.exe (Windows)
#   dist/PBHealth-v<VER>/pbhealth     (Linux)
#   dist/PBHealth-v<VER>/ui/          (assets do frontend)
#   dist/PBHealth-v<VER>/config.toml  (criado em runtime se ausente)
#   dist/PBHealth-v<VER>/README.txt   (instruções de uso)
#   dist/PBHealth-v<VER>/SHA256.txt   (hashes para verificação)

set -euo pipefail

cd "$(dirname "$0")/.."

VER="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')}"
OUT="dist/PBHealth-v${VER}"
BIN_WIN="target/release/pbhealth.exe"
BIN_LINUX="target/release/pbhealth"

echo "==> Empacotando PBHealth v${VER} para pendrive"

# Detecta plataforma pelo binário disponível
HAS_WIN=false
HAS_LINUX=false
[[ -f "$BIN_WIN" ]] && HAS_WIN=true
[[ -f "$BIN_LINUX" ]] && HAS_LINUX=true

if [[ "$HAS_WIN" == false && "$HAS_LINUX" == false ]]; then
    echo "❌ Nenhum binário encontrado em target/release/."
    echo "   Rode 'cargo tauri build' (Linux/Windows nativo) primeiro."
    exit 1
fi

mkdir -p "$OUT"

# Copia binários
if [[ "$HAS_WIN" == true ]]; then
    cp "$BIN_WIN" "$OUT/PBHealth.exe"
    echo "  ✓ PBHealth.exe copiado"
fi
if [[ "$HAS_LINUX" == true ]]; then
    cp "$BIN_LINUX" "$OUT/pbhealth"
    chmod +x "$OUT/pbhealth"
    echo "  ✓ pbhealth copiado"
fi

# Copia assets do frontend
if [[ -d "ui" ]]; then
    cp -r ui "$OUT/ui"
    echo "  ✓ ui/ copiado"
fi

# Copia config.toml se existir (senão será criado em runtime)
if [[ -f "config.toml" ]]; then
    cp "config.toml" "$OUT/config.toml"
fi

# README de uso
cat > "$OUT/README.txt" <<EOF
PBHealth v${VER} — Diagnóstico de Hardware
==============================================

USO:
  Windows: clique duplo em PBHealth.exe
  Linux  : abra terminal, ./pbhealth

NÃO PRECISA INSTALAR. NÃO PRECISA DE ADMINISTRADOR.

Se o Windows Defender bloquear na primeira execução:
  → "Mais informações" → "Executar mesmo assim"

CONFIGURAÇÃO:
  Edite config.toml antes de executar (opcional).
  Defaults estão em [technician] company = "PB Informática".

FORMATAÇÃO DO PENDRIVE:
  Recomendado: exFAT (compatível com Windows e Linux, suporta >4GB).
  Não recomendado: FAT32 (limite 4GB), NTFS-only (Linux não escreve nativo).

SUPORTE:
  PB Informática
EOF

# SHA256 (substitui code signing — uso interno)
echo "# SHA256 dos binários PBHealth v${VER}" > "$OUT/SHA256.txt"
echo "# Compare com o esperado antes de executar em máquinas de clientes." >> "$OUT/SHA256.txt"
if [[ -f "$OUT/PBHealth.exe" ]]; then
    sha256sum "$OUT/PBHealth.exe" >> "$OUT/SHA256.txt"
fi
if [[ -f "$OUT/pbhealth" ]]; then
    sha256sum "$OUT/pbhealth" >> "$OUT/SHA256.txt"
fi
echo "  ✓ SHA256.txt gerado"

echo ""
echo "✅ Pronto: $OUT/"
echo ""
echo "Tamanho:"
du -sh "$OUT"
echo ""
echo "Próximo passo: copie o conteúdo de $OUT/ para a raiz do pendrive (exFAT)."