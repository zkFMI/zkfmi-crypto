#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
host=softbank-l40s
mkdir -p .cache
ssh_args=(-o BatchMode=yes -o ConnectTimeout=15 -o ProxyJump=none
  -o ControlMaster=auto -o ControlPersist=600 -o "ControlPath=$PWD/.cache/ssh-direct")
printf -v RSYNC_RSH '%q ' ssh "${ssh_args[@]}"
export RSYNC_RSH
stamp=$(date -u +%Y%m%dT%H%M%SZ)
commit=$(git rev-parse --verify HEAD 2>/dev/null || printf uncommitted)
log="gate-${stamp}-${commit:0:12}.log"
mkdir -p .artifacts
for attempt in 1 2 3; do
  if ssh "${ssh_args[@]}" "$host" 'mkdir -p ~/work/zkfmi-crypto'; then break; fi
  if [ "$attempt" -eq 3 ]; then exit 1; fi
done
rsync -az --exclude=.git --exclude=.artifacts --exclude=.cache --exclude=target ./ "$host:work/zkfmi-crypto/"
set +e
ssh "${ssh_args[@]}" "$host" bash -s -- "$commit" "$log" <<'REMOTE' | tee ".artifacts/$log"
set -euo pipefail
cd ~/work/zkfmi-crypto
mkdir -p .artifacts .cache/cargo .cache/tmp
exec > >(tee ".artifacts/$2") 2>&1
printf 'host=%s\ncommit=%s\nstarted_at=%s\n' "$(hostname)" "$1" "$(date -u +%FT%TZ)"
docker run --rm --network=host --cpus=4 --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/work/.cache/cargo -e CARGO_BUILD_JOBS=4 -e TMPDIR=/work/.cache/tmp \
  -e RUSTUP_HOME=/work/.cache/rustup \
  -v "$PWD:/work" -w /work rust:1.97-bookworm bash -euc '
    if [ ! -f "$RUSTUP_HOME/settings.toml" ]; then
      mkdir -p "$RUSTUP_HOME"
      cp -R /usr/local/rustup/. "$RUSTUP_HOME/"
    fi
    rustup component add rustfmt clippy
    rustc --version
    cargo --version
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --release
    cargo tree -d
  '
docker image inspect rust:1.97-bookworm --format 'image_id={{.Id}} digests={{json .RepoDigests}}'
printf 'completed_at=%s\nresult=PASS\n' "$(date -u +%FT%TZ)"
REMOTE
result=${PIPESTATUS[0]}
set -e
if [ "$result" -eq 0 ]; then
  rsync -az "$host:work/zkfmi-crypto/Cargo.lock" ./Cargo.lock
fi
printf 'gate_exit=%s log=%s\n' "$result" "$PWD/.artifacts/$log"
exit "$result"
