#!/usr/bin/env bash
# PBHealth — script one-shot pra fazer push manual
# Uso: ./scripts/push-to-github.sh
#     (vai pedir o token interativamente, sem armazenar em disco)

set -euo pipefail
cd "$(dirname "$0")/.."

echo "╔════════════════════════════════════════════════════════╗"
echo "║  PBHealth — push inicial para github.com/nandinhos   ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "Status local:"
git log --oneline -1
echo ""
echo "Remote:"
git remote -v
echo ""

# Verifica se já tem credencial armazenada (não persistir)
if git -c credential.helper= ls-remote origin HEAD >/dev/null 2>&1; then
    echo "✅ Já autenticado! Tentando push..."
    git push -u origin main
    echo ""
    echo "✅ Push concluído!"
    echo "Repo: https://github.com/nandinhos/pchealth"
    exit 0
fi

echo "=== Como autenticar (escolha um método) ==="
echo ""
echo "MÉTODO 1 (recomendado, NÃO precisa colar token aqui):"
echo "  Use um credential helper persistente:"
echo "    git config --global credential.helper store"
echo "    git push -u origin main     # vai pedir user/senha 1x, salva"
echo ""
echo "MÉTODO 2 (token inline, manual):"
echo "  git push -u origin main"
echo "  Username: x-access-token"
echo "  Password: <cole seu PAT>"
echo ""
echo "MÉTODO 3 (SSH — definitivo, sem token):"
echo "  ssh-keygen -t ed25519 -C 'nandinhos@gmail.com'"
echo "  cat ~/.ssh/id_ed25519.pub   # adicione em https://github.com/settings/keys"
echo "  git remote set-url origin git@github.com:nandinhos/pchealth.git"
echo "  git push -u origin main"
echo ""
echo "Após o push, delete /tmp/git-creds-* se existir:"
echo "  rm -f /tmp/git-creds-*"
echo ""
echo "Para verificar depois:"
echo "  curl -s https://api.github.com/repos/nandinhos/pchealth | head"