#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source_root=${1:?usage: capture-inventory.sh SOURCE_ROOT [SNAPSHOT_DIRECTORY]}
snapshot=${2:-"$PWD/.cache/inventory-source-$(date -u +%Y%m%dT%H%M%SZ)"}
if [ -e "$snapshot" ]; then printf 'Refusing to overwrite snapshot: %s\n' "$snapshot" >&2; exit 1; fi
mkdir -p "$snapshot"
for repo in qomm zkpi defmi oclob dekyx deccp aethel; do
  mkdir -p "$snapshot/$repo"
  (cd "$source_root/$repo" && rg --files -g '*.rs' -g Cargo.toml) > "$snapshot/$repo.files"
  rsync -a --files-from="$snapshot/$repo.files" "$source_root/$repo/" "$snapshot/$repo/"
  git -C "$source_root/$repo" rev-parse HEAD > "$snapshot/$repo.head"
  git -C "$source_root/$repo" status --porcelain=v1 > "$snapshot/$repo.status"
done
date -u +%FT%TZ > "$snapshot/captured_at.txt"
printf '%s\n' "$snapshot"
