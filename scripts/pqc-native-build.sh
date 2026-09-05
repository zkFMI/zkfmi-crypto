#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
host=softbank-l40s
remote=work/pqc-full-integration-20260905
stamp=$(date -u +%Y%m%dT%H%M%SZ)
ssh_args=(-S /Users/shukob/Research/DeFMI/zkfmi-crypto/.cache/ssh-direct -o BatchMode=yes -o ProxyJump=none)
printf -v RSYNC_RSH '%q ' ssh "${ssh_args[@]}"
export RSYNC_RSH
mkdir -p .artifacts
ssh "${ssh_args[@]}" "$host" "mkdir -p ~/$remote/.cache/native-build"
rsync -az scripts/PQC-Native.Dockerfile "$host:$remote/.cache/native-build/Dockerfile"
rsync -az ../qomm/rust/qomm-mpc/patches/require-hybrid-tls.patch "$host:$remote/.cache/native-build/"
ssh "${ssh_args[@]}" "$host" 'bash -s' <<'REMOTE' 2>&1 | tee ".artifacts/$stamp-native-build.log"
set -euo pipefail
cd "$HOME/work/pqc-full-integration-20260905"
test "$(docker image inspect oclob-test:rust-1.97.1-mpspdz-9d809599 --format '{{.Id}}')" = sha256:8513c0e1c3792f61bd8806d9ef05040c1649f6145a11e7d79126a54ca73bdc9d
if [ ! -f .cache/native-build/openssl-3.5.5.tar.gz ]; then
  curl --fail --silent --show-error --location \
    https://github.com/openssl/openssl/releases/download/openssl-3.5.5/openssl-3.5.5.tar.gz \
    --output .cache/native-build/openssl-3.5.5.tar.gz
fi
printf '%s\n' 'b28c91532a8b65a1f983b4c28b7488174e4a01008e29ce8e69bd789f28bc2a89  .cache/native-build/openssl-3.5.5.tar.gz' | sha256sum -c -
printf 'started_at=%s\n' "$(date -u +%FT%TZ)"
docker build --progress=plain --file .cache/native-build/Dockerfile \
  --tag pqc-full-integration:rust-1.97.1-mpspdz-9d809599-openssl-3.5.5-pqc-auth-v2 .cache/native-build
docker image inspect pqc-full-integration:rust-1.97.1-mpspdz-9d809599-openssl-3.5.5-pqc-auth-v2 --format 'image_id={{.Id}}'
printf 'completed_at=%s\nresult=PASS\n' "$(date -u +%FT%TZ)"
REMOTE
