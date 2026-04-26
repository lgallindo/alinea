#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

script/build_app.sh
mkdir -p "$HOME/Applications"
rsync -a --delete "dist/Alinea.app" "$HOME/Applications/"

echo "$HOME/Applications/Alinea.app"
