#!/usr/bin/env bash
set -euo pipefail
# Import only formatter/lockfile changes from a completed isolated remote run.
# Every local source must still equal that run's before hash, and every fetched
# byte must equal the after hash. Never import keys, runtime state or receipts
# from another workspace.
cd "$(dirname "$0")/.."
run=${1:?expected run identifier}
[[ "$run" =~ ^[0-9]{8}T[0-9]{6}Z-[a-z0-9-]+$ ]] || exit 2
root=$(cd .. && pwd)
destination="$PWD/.artifacts/import-$run"
mkdir -p "$destination"
remote=work/pqc-astra-high-20260906
args=(-o ControlPath=/Users/shukob/Research/DeFMI/zkfmi-crypto/.cache/ssh-direct -o BatchMode=yes -o ProxyJump=none)
for phase in before after; do
  scp -q "${args[@]}" "softbank-l40s:$remote/.artifacts/$run-input-$phase.sha256" "$destination/$phase.sha256"
done
while read -r after path; do
  [[ "$path" =~ ^src/(zkfmi-crypto|qomm|zkpi|defmi|oclob|dekyx|deccp|aethel)/ ]] || exit 2
  [[ "$path" != *..* ]] || exit 2
  relative=${path#src/}
  before=$(awk -v path="$path" '$2 == path {print $1}' "$destination/before.sha256")
  [[ "$before" =~ ^[0-9a-f]{64}$ && "$after" =~ ^[0-9a-f]{64}$ ]] || exit 2
  if [ "$before" = "$after" ]; then continue; fi
  local_path="$root/$relative"
  [ -f "$local_path" ] && [ ! -L "$local_path" ] || exit 2
  current=$(shasum -a 256 "$local_path" | cut -d ' ' -f 1)
  if [ "$current" != "$before" ]; then
    printf 'source changed since run: %s\n' "$relative" >&2
    exit 1
  fi
  mkdir -p "$destination/$(dirname "$relative")"
  fetched="$destination/$relative"
  scp -q "${args[@]}" "softbank-l40s:$remote/$path" "$fetched"
  actual=$(shasum -a 256 "$fetched" | cut -d ' ' -f 1)
  [ "$actual" = "$after" ] || { printf 'remote source changed: %s\n' "$relative" >&2; exit 1; }
  cp "$fetched" "$local_path"
  printf 'imported=%s\n' "$relative"
done < "$destination/after.sha256"
