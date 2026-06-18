#!/usr/bin/env bash
# PBHealth — gera SHA256 dos binários (substitui code signing para uso interno).
#
# Uso: ./scripts/sign-hash.sh
# Saída: ./SHA256.txt (na raiz do projeto)

set -euo pipefail
cd "$(dirname "$0")/.."

OUT="SHA256.txt"
echo "# SHA256 dos binários PBHealth" > "$OUT"
echo "# Gerado em $(date -u +'%Y-%m-%dT%H:%M:%SZ')" >> "$OUT"
echo "# Compare antes de distribuir em máquinas de clientes." >> "$OUT"

if [[ -f "target/release/pbhealth.exe" ]]; then
    sha256sum "target/release/pbhealth.exe" >> "$OUT"
fi
if [[ -f "target/release/pbhealth" ]]; then
    sha256sum "target/release/pbhealth" >> "$OUT"
fi

cat "$OUT"
echo ""
echo "✓ Hashes salvos em $OUT"