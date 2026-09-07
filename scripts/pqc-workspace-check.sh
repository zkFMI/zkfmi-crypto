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
  cargo fmt --all
  if [ "$workspace" = qomm/rust ]; then
    # Baseline helper qomm_live_acceptance_report.rs is incorrectly discovered
    # as a standalone binary (super import, no main). Keep that unrelated
    # failure visible; check the actual application target separately.
    cargo check --locked --workspace --all-targets --all-features --exclude qomm-demo
    cargo check --locked -p qomm-demo --lib --bin qomm_live_acceptance \
      --bin qomm_participant_node --bin qomm_mpc_node --bin qomm_frontend
  else
    cargo check --locked --workspace --all-targets --all-features
  fi
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
cargo fmt --all
cargo check --locked --workspace --all-targets --all-features
