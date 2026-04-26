#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
exec "${CARGO:-$HOME/.cargo/bin/cargo}" run
