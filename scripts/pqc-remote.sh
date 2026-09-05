#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-}" in zkfmi-crypto|qomm/rust|zkpi/rust|defmi/rust|oclob|aethel|dekyx|deccp) project=$1;; *) echo 'expected integration workspace' >&2; exit 2;; esac
command=${2:?expected verification command}
native=${PQC_NATIVE_ENGINE:-0}
case "$native" in 0|1) ;; *) echo 'PQC_NATIVE_ENGINE must be 0 or 1' >&2; exit 2;; esac
root=$(cd .. && pwd)
test "$root" = /Users/shukob/Research/DeFMI/zkfmi-crypto/.worktrees/pqc-astra-high
host=softbank-l40s
remote=work/pqc-astra-high-20260906
stamp=$(date -u +%Y%m%dT%H%M%SZ)
run="${stamp}-${project//\//-}"
ssh_args=(-S /Users/shukob/Research/DeFMI/zkfmi-crypto/.cache/ssh-direct -o BatchMode=yes -o ProxyJump=none)
printf -v RSYNC_RSH '%q ' ssh "${ssh_args[@]}"
export RSYNC_RSH
mkdir -p .artifacts
mkdir .artifacts/remote-run.lock || { echo 'another isolated remote run owns the source snapshot' >&2; exit 2; }
trap 'rmdir .artifacts/remote-run.lock' EXIT
ssh "${ssh_args[@]}" "$host" "mkdir -p ~/$remote/{src,.cache/tmp,.cache/cargo,.cache/target,.artifacts}"
for repo in zkfmi-crypto qomm zkpi defmi oclob dekyx deccp aethel; do
  rsync -az --delete --exclude=.git --exclude=.cache --exclude=.worktrees --exclude=.artifacts --exclude=target --exclude=node_modules --exclude=.runtime \
    "$root/$repo/" "$host:$remote/src/$repo/"
done
printf -v remote_command 'bash -s -- %q %q %q %q %q' "$remote" "$project" "$command" "$run" "$native"
ssh "${ssh_args[@]}" "$host" "$remote_command" <<'REMOTE' | tee ".artifacts/$run.log"
set -euo pipefail
[ "$#" -eq 5 ] || { echo 'invalid remote verification arguments' >&2; exit 2; }
test "$1" = work/pqc-astra-high-20260906
umask 077
cd "$HOME/$1"
exec > >(tee ".artifacts/$4.log") 2>&1
run=$4
record_inputs() {
  find src -type f \( -name '*.rs' -o -name Cargo.toml -o -name Cargo.lock \
    -o -name '*.cpp' -o -name '*.h' -o -name '*.patch' -o -name '*.sh' \
    -o -name '*Dockerfile*' -o -name Makefile \) -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > ".artifacts/$run-input-$1.sha256"
  printf 'input_%s=' "$1"
  sha256sum ".artifacts/$run-input-$1.sha256"
}
image=rust:1.97-bookworm
target=.cache/target
native_mount=()
if [ "$5" = 1 ]; then
  # Existing verified image content; another task cannot replace a mutable tag.
  image=sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4
  target=.cache/target-native
  image_id=$(docker image inspect "$image" --format '{{.Id}}')
  engine_dir=".cache/native-engine-${image_id#sha256:}"
  # The compiler writes generated circuits. Keep that writable checkout and
  # lab-only certificates inside this task, with the invoking user's ownership.
  if [ ! -f "$engine_dir/.integration-image" ]; then
    docker run --rm --user 0:0 -v "$PWD:/integration" "$image" bash -euc '
      destination=/integration/$1
      mkdir -p "$destination"
      cp -a /opt/MP-SPDZ/. "$destination/"
      chown -R "$2:$3" "$destination"
      chmod 0700 "$destination"
      printf "%s\n" "$4" > "$destination/.integration-image"
      chown "$2:$3" "$destination/.integration-image"
    ' bash "$engine_dir" "$(id -u)" "$(id -g)" "$image_id"
  fi
  [ "$(cat "$engine_dir/.integration-image")" = "$image_id" ]
  native_mount=(-v "$PWD/$engine_dir:/opt/MP-SPDZ")
fi
mkdir -p "$target"
printf 'started_at=%s\nhost=%s\nworkspace=%s\n' "$(date -u +%FT%TZ)" "$(hostname)" "$2"
printf 'command=%s\n' "$3"
docker image inspect "$image" --format 'image_id={{.Id}}'
record_inputs before
if docker run --rm --init --network=host --cpus=8 --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/integration/.cache/cargo -e CARGO_BUILD_JOBS=8 \
  -e CARGO_TARGET_DIR="/integration/$target" -e TMPDIR=/integration/.cache/tmp \
  -e RUSTUP_HOME=/integration/.cache/rustup \
  -v "$PWD:/integration" "${native_mount[@]}" -w "/integration/src/$2" "$image" bash -euc '
    umask 077
    if [ ! -f "$RUSTUP_HOME/settings.toml" ]; then
      mkdir -p "$RUSTUP_HOME"
      cp -R /usr/local/rustup/. "$RUSTUP_HOME/"
    fi
    rustup component add rustfmt clippy
    rustc --version
    exec bash -euc "$1"
  ' bash "$3"; then code=0; result=PASS; else code=$?; result=FAIL; fi
record_inputs after
printf 'completed_at=%s\nresult=%s\n' "$(date -u +%FT%TZ)" "$result"
exit "$code"
REMOTE
printf 'log=%s/.artifacts/%s.log\n' "$PWD" "$run"
