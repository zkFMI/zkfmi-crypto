#!/usr/bin/env bash
set -euo pipefail
# Invoke only inside the isolated remote container via pqc-remote.sh. Metadata
# and checks are locked so this gate verifies the committed dependency graph.
test "${PWD}" = /integration/src/zkfmi-crypto
for workspace in zkfmi-crypto qomm/rust zkpi/rust defmi/rust oclob dekyx deccp aethel; do
  cd "/integration/src/$workspace"
  label=${workspace//\//-}
  printf 'workspace_gate=%s\n' "$workspace"
  metadata="/integration/.artifacts/$label-metadata.json"
  cargo metadata --locked --format-version=1 --all-features > "$metadata"
  if [ "$workspace" != aethel ] && grep -Eq '"name":"aethel[^"]*"' "$metadata"; then
    printf 'foundation workspace unexpectedly resolves an Aethel package: %s\n' "$workspace" >&2
    exit 1
  fi
  cargo fmt --all -- --check
  cargo check --locked --workspace --all-targets --all-features
done

# qomm-batch-audit declares its own workspace and is therefore intentionally
# outside zkpi/rust's parent workspace. Gate it explicitly with its lockfile so
# its independently pinned crypto dependency cannot escape the portable check.
workspace=zkpi/rust/qomm-batch-audit
cd "/integration/src/$workspace"
label=${workspace//\//-}
printf 'workspace_gate=%s\n' "$workspace"
metadata="/integration/.artifacts/$label-metadata.json"
cargo metadata --locked --format-version=1 --all-features > "$metadata"
if grep -Eq '"name":"aethel[^"]*"' "$metadata"; then
  printf 'foundation workspace unexpectedly resolves an Aethel package: %s\n' "$workspace" >&2
  exit 1
fi
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets --all-features
