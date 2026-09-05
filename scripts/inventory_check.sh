#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "$(uname -s)" != Linux ]; then
  printf 'Run inventory verification in the softbank-l40s Rust Docker container.\n' >&2
  exit 1
fi
cargo run --release --locked --bin crypto-inventory -- check "${1:-.cache/inventory-source}" "${2:-inventory}"
