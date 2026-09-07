#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
integration_root=$(cd .. && pwd)
test "$integration_root" = /Users/shukob/Research/DeFMI/zkfmi-crypto/.worktrees/pqc-astra-high

host=softbank-l40s
remote_base=work/pqc-astra-high-20260906
stamp=$(date -u +%Y%m%dT%H%M%SZ | tr '[:upper:]' '[:lower:]')
run="$stamp-$$-pqc-native-acceptance"
task_rel="$remote_base/.cache/native-runs/$run"
base_id=sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4
crypto_pin=26ebfe96563a05c3379219f7fdacea137ab5fb4f
market_contract_sha=0e01378323b129f6fdd92aa7a59d353b2a034fbc1cf404a895bbd6a4e619a185
finality_contract_sha=308d47bfc84c19784dc6606bf17db783f4d7579a4b48b5c97a12654599cf4886
lifecycle_contract_sha=74093488ce6e3accd983a8b2ad025fd0ed3226bb8cd2165a2e5024edf3904743
base_tag="pqc-native-acceptance-base:$run"
cluster_image="oclob-pqc-native-cluster:$run"
avalanche_image="oclob-pqc-native-avalanche:$run"
ssh_args=(-S /Users/shukob/Research/DeFMI/zkfmi-crypto/.cache/ssh-direct -o BatchMode=yes -o ProxyJump=none)
ssh_options="${ssh_args[*]}"
local_receipts="$PWD/.artifacts/$run"
log="$PWD/.artifacts/$run.log"
repos=(zkfmi-crypto qomm zkpi defmi oclob dekyx deccp aethel)

mkdir -p "$PWD/.artifacts" "$local_receipts"
mkdir "$PWD/.artifacts/remote-run.lock" || {
  echo 'another isolated remote run owns the source snapshot' >&2
  exit 2
}
remote_lock_owned=0
cleanup() {
  if [ "$remote_lock_owned" = 1 ]; then
    ssh "${ssh_args[@]}" "$host" "rmdir '$remote_base/.artifacts/remote-run.lock'" \
      >/dev/null 2>&1 || true
  fi
  rmdir "$PWD/.artifacts/remote-run.lock" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM
exec > >(tee "$log") 2>&1

printf -v RSYNC_RSH '%q ' ssh "${ssh_args[@]}"
export RSYNC_RSH

record_local_source() {
  (
    cd "$integration_root"
    find "${repos[@]}" \
      -name .git -prune \
      -o -type d \( -name .codex -o -name .cache -o -name .worktrees \
        -o -name .artifacts -o -name target -o -name node_modules -o -name .runtime \) -prune \
      -o -type f -print \
      | LC_ALL=C sort \
      | while IFS= read -r file; do sha256sum "$file"; done
  ) > "$local_receipts/source-local-$1.sha256"
}

record_local_source before
git -C "$integration_root/zkfmi-crypto" diff --quiet "$crypto_pin" -- \
  src Cargo.toml Cargo.lock
test -z "$(git -C "$integration_root/zkfmi-crypto" ls-files --others --exclude-standard -- \
  src Cargo.toml Cargo.lock)"
printf 'dependency=zkfmi-crypto\npinned_revision=%s\nlocal_rust_and_cargo_bytes_identical=true\n' \
  "$crypto_pin" > "$local_receipts/crypto-dependency-pin.txt"
ssh "${ssh_args[@]}" "$host" \
  "mkdir -p '$remote_base/.artifacts' '$remote_base/.cache/native-runs'; mkdir '$remote_base/.artifacts/remote-run.lock'"
remote_lock_owned=1
ssh "${ssh_args[@]}" "$host" \
  "umask 077; mkdir -p '$task_rel/src' '$task_rel/receipts' '$task_rel/scenarios' '$task_rel/.cache/tmp' '$task_rel/.cache/cargo' '$task_rel/.cache/target-p6'"

for repo in "${repos[@]}"; do
  rsync -az --checksum --no-times --delete \
    --exclude=.git --exclude=.codex --exclude=.cache --exclude=.worktrees \
    --exclude=.artifacts --exclude=target --exclude=node_modules --exclude=.runtime \
    "$integration_root/$repo/" "$host:$task_rel/src/$repo/"
done
rsync -a "$local_receipts/crypto-dependency-pin.txt" \
  "$host:$task_rel/receipts/crypto-dependency-pin.txt"

ssh "${ssh_args[@]}" "$host" bash -s -- \
  "$remote_base" "$run" "$base_id" "$base_tag" "$cluster_image" "$avalanche_image" \
  "$market_contract_sha" "$finality_contract_sha" "$lifecycle_contract_sha" <<'REMOTE'
set -euo pipefail
[ "$#" -eq 9 ]
remote_base=$1
run=$2
expected_base=$3
base_tag=$4
cluster_image=$5
avalanche_image=$6
market_contract_sha=$7
finality_contract_sha=$8
lifecycle_contract_sha=$9
run_root="$HOME/$remote_base/.cache/native-runs/$run"
cd "$run_root"

record_source() {
  (
    cd src
    find zkfmi-crypto qomm zkpi defmi oclob dekyx deccp aethel \
      -name .git -prune \
      -o -type d \( -name .codex -o -name .cache -o -name .worktrees \
        -o -name .artifacts -o -name target -o -name node_modules -o -name .runtime \) -prune \
      -o -type f -print \
      | LC_ALL=C sort \
      | while IFS= read -r file; do sha256sum "$file"; done
  ) > "receipts/source-$1.sha256"
}
record_protocol_inputs() {
  sha256sum \
    src/oclob/research/oclob_native_market_v2_contract.json \
    src/oclob/research/manifests/oclob_native_market_003.json \
    src/oclob/research/oclob_native_finality_v2_contract.json \
    src/oclob/research/manifests/oclob_native_finality_003.json \
    src/oclob/research/oclob_native_lifecycle_v2_contract.json \
    src/oclob/research/manifests/oclob_native_lifecycle_005.json \
    > "receipts/protocol-inputs-$1.sha256"
}
record_images() {
  receipt="receipts/images-$1.txt"
  : > "$receipt"
  for entry in "base:$base_tag" "cluster:$cluster_image" "avalanche:$avalanche_image"; do
    role=${entry%%:*}
    tag=${entry#*:}
    id=$(docker image inspect "$tag" --format '{{.Id}}')
    digests=$(docker image inspect "$tag" --format '{{json .RepoDigests}}')
    labels=$(docker image inspect "$tag" --format '{{json .Config.Labels}}')
    printf '%s_tag=%s\n%s_id=%s\n%s_repo_digests=%s\n%s_config_labels=%s\n' \
      "$role" "$tag" "$role" "$id" "$role" "$digests" "$role" "$labels" >> "$receipt"
  done
}

record_source before
record_protocol_inputs before
printf '%s  %s\n' \
  "$market_contract_sha" \
  src/oclob/research/oclob_native_market_v2_contract.json \
  "$finality_contract_sha" \
  src/oclob/research/oclob_native_finality_v2_contract.json \
  "$lifecycle_contract_sha" \
  src/oclob/research/oclob_native_lifecycle_v2_contract.json \
  | sha256sum -c -

actual_base=$(docker image inspect "$expected_base" --format '{{.Id}}')
[ "$actual_base" = "$expected_base" ]
docker tag "$expected_base" "$base_tag"
source_before_sha=$(sha256sum receipts/source-before.sha256 | cut -d ' ' -f 1)
printf 'started_at=%s\nhost=%s\nrun_root=%s\nverified_base_id=%s\n' \
  "$(date -u +%FT%TZ)" "$(hostname)" "$run_root" "$actual_base" \
  > receipts/run.txt
printf 'source_before_manifest_sha256=%s\n' "$source_before_sha" >> receipts/run.txt

docker build --progress=plain --network host \
  --file src/zkfmi-crypto/scripts/PQC-Native-Acceptance.Dockerfile \
  --build-arg "PQC_NATIVE_BASE=$base_tag" \
  --label "org.zkfmi.pqc-native.run=$run" \
  --label "org.zkfmi.pqc-native.source-manifest-sha256=$source_before_sha" \
  --label "org.zkfmi.pqc-native.base-image-id=$actual_base" \
  --target oclob-cluster --tag "$cluster_image" src
docker build --progress=plain --network host \
  --file src/zkfmi-crypto/scripts/PQC-Native-Acceptance.Dockerfile \
  --build-arg "PQC_NATIVE_BASE=$base_tag" \
  --label "org.zkfmi.pqc-native.run=$run" \
  --label "org.zkfmi.pqc-native.source-manifest-sha256=$source_before_sha" \
  --label "org.zkfmi.pqc-native.base-image-id=$actual_base" \
  --target oclob-avalanche-acceptance --tag "$avalanche_image" src
for built_image in "$cluster_image" "$avalanche_image"; do
  test "$(docker image inspect "$built_image" --format '{{ index .Config.Labels "org.zkfmi.pqc-native.run" }}')" = "$run"
  test "$(docker image inspect "$built_image" --format '{{ index .Config.Labels "org.zkfmi.pqc-native.source-manifest-sha256" }}')" = "$source_before_sha"
  test "$(docker image inspect "$built_image" --format '{{ index .Config.Labels "org.zkfmi.pqc-native.base-image-id" }}')" = "$actual_base"
  docker image inspect "$built_image" --format '{{range .Config.Env}}{{println .}}{{end}}' \
    | grep -Fx 'MP_SPDZ_ROOT=/opt/MP-SPDZ'
  docker image inspect "$built_image" --format '{{range .Config.Env}}{{println .}}{{end}}' \
    | grep -Fx 'LD_LIBRARY_PATH=/opt/pqc-openssl/lib64:/opt/MP-SPDZ'
done
record_images before
for entry in "cluster:$cluster_image" "avalanche:$avalanche_image"; do
  role=${entry%%:*}
  tag=${entry#*:}
  docker run --rm --entrypoint sh "$tag" -euc '
    cd /opt/MP-SPDZ
    sha256sum \
      Programs/Source/oclob_match_v1.mpc \
      Programs/Schedules/oclob_match_v1.sch \
      Programs/Bytecode/oclob_match_v1-*.bc
  ' > "receipts/mp-spdz-$role.sha256"
done
cmp receipts/mp-spdz-cluster.sha256 receipts/mp-spdz-avalanche.sha256

for scenario in market finality lifecycle; do
  mkdir -p "scenarios/$scenario"
  ln -s ../../src/oclob "scenarios/$scenario/oclob"
done
REMOTE

rsync -a "$host:$task_rel/receipts/source-before.sha256" "$local_receipts/"
cmp "$local_receipts/source-local-before.sha256" "$local_receipts/source-before.sha256"

run_scenario() {
  local scenario=$1
  local target=$2
  local contract=$3
  local manifest=$4
  local artifact="$local_receipts/$scenario.json"
  local compose_project="pqc-$run-$scenario"
  make -C "$integration_root/oclob" "$target" \
    REMOTE_TEST_HOST="$host" \
    REMOTE_TEST_SSH_OPTIONS="$ssh_options" \
    PQC_NATIVE_RUN_ROOT="$task_rel/scenarios/$scenario" \
    PQC_NATIVE_SKIP_SYNC=1 \
    PQC_NATIVE_REUSE_IMAGES=1 \
    PQC_NATIVE_CLUSTER_IMAGE="$cluster_image" \
    PQC_NATIVE_AVALANCHE_IMAGE="$avalanche_image" \
    PQC_NATIVE_COMPOSE_PROJECT="$compose_project" \
    PQC_NATIVE_CONTRACT="$contract" \
    PQC_NATIVE_MANIFEST="$manifest" \
    PQC_NATIVE_ARTIFACT="$artifact"
  test -s "$artifact"
  (cd "$local_receipts" && sha256sum "$scenario.json") >> "$local_receipts/scenario-artifacts.sha256"
}

run_scenario market remote-native-market-e2e \
  /research/oclob_native_market_v2_contract.json \
  /research/manifests/oclob_native_market_003.json
run_scenario finality remote-native-finality-e2e \
  /research/oclob_native_finality_v2_contract.json \
  /research/manifests/oclob_native_finality_003.json
run_scenario lifecycle remote-native-lifecycle-e2e \
  /research/oclob_native_lifecycle_v2_contract.json \
  /research/manifests/oclob_native_lifecycle_005.json

ssh "${ssh_args[@]}" "$host" bash -s -- "$remote_base" "$run" "$base_id" <<'REMOTE'
set -euo pipefail
[ "$#" -eq 3 ]
run_root="$HOME/$1/.cache/native-runs/$2"
base_id=$3
cd "$run_root"
printf 'p6_started_at=%s\n' "$(date -u +%FT%TZ)" >> receipts/run.txt
docker run --rm --init --network=host --cpus=8 \
  --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/integration/.cache/cargo \
  -e CARGO_TARGET_DIR=/integration/.cache/target-p6 \
  -e TMPDIR=/integration/.cache/tmp \
  -e RUSTUP_HOME=/usr/local/rustup \
  -v "$run_root:/integration" \
  -w /integration/src/defmi/rust \
  "$base_id" bash -euc \
  'env -u MP_SPDZ_ROOT cargo test --locked --release -p defmi-avalanche-vm --test recovery' \
  2>&1 | tee receipts/p6-recovery.log
sha256sum receipts/p6-recovery.log > receipts/p6-recovery.sha256
printf 'p6_completed_at=%s\np6_result=PASS\n' "$(date -u +%FT%TZ)" >> receipts/run.txt

(
  cd src
  find zkfmi-crypto qomm zkpi defmi oclob dekyx deccp aethel \
    -name .git -prune \
    -o -type d \( -name .codex -o -name .cache -o -name .worktrees \
      -o -name .artifacts -o -name target -o -name node_modules -o -name .runtime \) -prune \
    -o -type f -print \
    | LC_ALL=C sort \
    | while IFS= read -r file; do sha256sum "$file"; done
) > receipts/source-after.sha256
sha256sum \
  src/oclob/research/oclob_native_market_v2_contract.json \
  src/oclob/research/manifests/oclob_native_market_003.json \
  src/oclob/research/oclob_native_finality_v2_contract.json \
  src/oclob/research/manifests/oclob_native_finality_003.json \
  src/oclob/research/oclob_native_lifecycle_v2_contract.json \
  src/oclob/research/manifests/oclob_native_lifecycle_005.json \
  > receipts/protocol-inputs-after.sha256

: > receipts/images-after.txt
for entry in \
  "base:pqc-native-acceptance-base:$2" \
  "cluster:oclob-pqc-native-cluster:$2" \
  "avalanche:oclob-pqc-native-avalanche:$2"; do
  role=${entry%%:*}
  tag=${entry#*:}
  id=$(docker image inspect "$tag" --format '{{.Id}}')
  digests=$(docker image inspect "$tag" --format '{{json .RepoDigests}}')
  labels=$(docker image inspect "$tag" --format '{{json .Config.Labels}}')
  printf '%s_tag=%s\n%s_id=%s\n%s_repo_digests=%s\n%s_config_labels=%s\n' \
    "$role" "$tag" "$role" "$id" "$role" "$digests" "$role" "$labels" >> receipts/images-after.txt
done
cmp receipts/source-before.sha256 receipts/source-after.sha256
cmp receipts/protocol-inputs-before.sha256 receipts/protocol-inputs-after.sha256
cmp receipts/images-before.txt receipts/images-after.txt
printf 'completed_at=%s\nresult=PASS\n' "$(date -u +%FT%TZ)" >> receipts/run.txt
REMOTE

rsync -a "$host:$task_rel/receipts/" "$local_receipts/remote/"
record_local_source after
cmp "$local_receipts/source-local-before.sha256" "$local_receipts/source-local-after.sha256"
cmp "$local_receipts/source-local-after.sha256" "$local_receipts/remote/source-after.sha256"

ledger="$local_receipts/experiment-ledger.pending.jsonl"
: > "$ledger"
record_ledger() {
  local scenario=$1
  local contract_id=$2
  local contract_hash=$3
  local manifest=$4
  local observation=$5
  local artifact_hash manifest_hash recorded
  artifact_hash=$(sha256sum "$local_receipts/$scenario.json" | cut -d ' ' -f 1)
  manifest_hash=$(sha256sum "$integration_root/oclob/$manifest" | cut -d ' ' -f 1)
  recorded=$(date -u +%FT%TZ)
  printf '{"recorded_at":"%s","experiment_id":"%s-%s","contract_id":"%s","contract_sha256":"%s","manifest":"%s","manifest_sha256":"%s","stage":"RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC","verdict":"smoke_only","observed":"%s","evidence":["%s.json SHA256 %s","OCLOB deterministic custody gate 20260906T075758Z-oclob log SHA256 9bdc59114b73dd0ee29365074b45efbdc69957238aa53dec1d15d1014ac7b135 proves byte-identical randomized response retry and process restart","source before and after manifests equal","verified native base and final image IDs and labels retained in runner receipt"],"limitations":["Single-host functional acceptance; exact response restart is combined deterministic-gate plus native-path evidence, not a corporate-process restart performed by every native scenario; no WAN, independent operators, physical HSM, performance, economic or production claim."]}\n' \
    "$recorded" "$run" "$scenario" "$contract_id" "$contract_hash" "$manifest" \
    "$manifest_hash" "$observation" "$scenario" "$artifact_hash" >> "$ledger"
}
record_ledger market oclob-native-market-v2 \
  "$market_contract_sha" \
  research/manifests/oclob_native_market_003.json \
  'Resident market crash and restart acceptance passed with four claim response signatures, eight fresh claim keys and zero financial approval signatures.'
record_ledger finality oclob-native-finality-v2 \
  "$finality_contract_sha" \
  research/manifests/oclob_native_finality_003.json \
  'Single-fill finality and custody acceptance passed with two claim response signatures, four fresh claim keys and zero financial approval signatures.'
record_ledger lifecycle oclob-native-lifecycle-v2 \
  "$lifecycle_contract_sha" \
  research/manifests/oclob_native_lifecycle_005.json \
  'Three-fill lifecycle acceptance passed with six claim response signatures, twelve fresh claim keys and zero financial approval signatures.'

printf 'Native acceptance PASS; retained remote root: %s\n' "$task_rel"
printf 'Receipts: %s\n' "$local_receipts"
printf 'Pending append-only ledger records: %s\n' "$ledger"
