#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Installing hooks..."
if [[ -f .git/hooks/pre-commit ]]; then chmod +x .git/hooks/pre-commit; fi

echo "🧪 Reference + validate..."
bash scripts/check-references.sh
just validate

echo "🛠️ Generate scripts..."
just generate-scripts

echo "📚 Build cookbook..."
just docs

echo "✅ Bootstrap complete."
