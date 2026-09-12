#!/usr/bin/env bash
set -euo pipefail

# Invoked only by the Rust integration-live atomic preflight, inside a fresh
# network-none container. Every validator and ANR socket is loopback-local.
[[ "$#" == 5 ]] || { echo "expected output proofs VM driver genesis" >&2; exit 2; }
output=$1
proofs=$2
vm=$3
driver=$4
genesis_config=$5
runtime=${COCODE_LIVE_RUNTIME_ROOT:?explicit fresh runtime directory required}
runner=/usr/local/bin/avalanche-network-runner
avalanchego=/usr/local/bin/avalanchego
endpoint=localhost:18080

test -s "$output/run.claim"
test -x "$vm"
test -x "$driver"
test -x "$runner"
test -x "$avalanchego"
umask 077
mkdir "$runtime"
mkdir "$runtime/plugins" "$runtime/data" "$runtime/logs" "$runtime/tmp"
runner_pid=""
cleanup() {
  if [[ -n "$runner_pid" ]]; then
    "$runner" control stop --endpoint="$endpoint" --request-timeout=3m \
      >> "$runtime/logs/stop.log" 2>&1 || true
    kill -TERM "$runner_pid" >/dev/null 2>&1 || true
    wait "$runner_pid" 2>/dev/null || true
  fi
  # Preserve this exact run's data/logs for inspection. Do not delete keys,
  # chain state, other containers or an earlier execution's artifacts.
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

vm_id=$("$vm" vmid)
[[ "$vm_id" =~ ^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$ ]]
install -m 0755 "$vm" "$runtime/plugins/$vm_id"
"$vm" genesis --config "$genesis_config" --out "$runtime/genesis.bin"
"$avalanchego" --version > "$output/avalanchego-version.txt"
"$runner" --version > "$output/network-runner-version.txt"
sha256sum "$vm" "$driver" "$avalanchego" "$runner" "$genesis_config" \
  "$runtime/genesis.bin" > "$output/live-binary-inputs.sha256"

"$runner" server --port=:18080 --grpc-gateway-port=:18081 \
  --log-dir="$runtime/logs" > "$runtime/logs/server.log" 2>&1 &
runner_pid=$!
ready=0
for _ in {1..100}; do
  if "$runner" control rpc_version --endpoint="$endpoint" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 0.1
done
[[ "$ready" == 1 ]] || { echo "live network runner did not start" >&2; exit 3; }

spec="[{\"vm_name\":\"defmivm\",\"genesis\":\"$runtime/genesis.bin\"}]"
"$runner" control start --endpoint="$endpoint" --request-timeout=5m \
  --avalanchego-path="$avalanchego" --plugin-dir="$runtime/plugins" \
  --root-data-dir="$runtime/data" --network-id=1337 --num-nodes=5 \
  --dynamic-ports --reassign-ports-if-used --blockchain-specs="$spec" \
  > "$runtime/logs/start.log" 2>&1
"$runner" control wait-for-healthy --endpoint="$endpoint" --request-timeout=5m \
  > "$runtime/logs/healthy.log" 2>&1

strip_ansi() { sed $'s/\033\\[[0-9;]*m//g'; }
chain_id=$("$runner" control list-blockchains --endpoint="$endpoint" 2>&1 \
  | strip_ansi | sed -n 's/.*Blockchain ID: //p' | head -n 1)
uri_line=$("$runner" control uris --endpoint="$endpoint" 2>&1 \
  | strip_ansi | sed -n 's/.*URIs: \[\(.*\)\]/\1/p' | head -n 1)
[[ "$chain_id" =~ ^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$ ]]
node_uris=()
for uri in $uri_line; do
  [[ "$uri" =~ ^http://127\.0\.0\.1:[0-9]+$ ]] \
    || { echo "non-loopback validator URI rejected" >&2; exit 4; }
  node_uris+=("$uri")
done
[[ "${#node_uris[@]}" == 5 ]] || { echo "exactly five validators required" >&2; exit 4; }

# All interpolated values have fixed or validated alphabets. The runtime path
# is supplied by the root's fixed isolated-container launch, never a RPC peer.
[[ "$runtime" =~ ^/[A-Za-z0-9_/-]+$ ]]
config="$output/live-config.json"
{
  printf '{"chainId":"%s","nodeUris":[' "$chain_id"
  separator=""
  for uri in "${node_uris[@]}"; do
    printf '%s"%s"' "$separator" "$uri"
    separator=,
  done
  printf '],"acceptanceTimeoutSeconds":300,"pollIntervalMillis":200,'
  printf '"restart":{"runnerPath":"%s","runnerEndpoint":"%s","nodeName":"node3","pluginDir":"%s/plugins","timeoutSeconds":300}}\n' \
    "$runner" "$endpoint" "$runtime"
} > "$config"

for operation in fill nofill; do
  mkdir "$output/$operation"
  cp "$proofs/$operation/proof.json" "$output/$operation/proof.json"
  cp "$proofs/$operation/statement.json" "$output/$operation/statement.json"
done
printf 'five real AvalancheGo validators healthy; chain=%s\n' "$chain_id"
driver_args=("$output" --live-config "$config")
if [[ -n "${COCODE_LIVE_QUERY_ROSTER_SHA512:-}" ]]; then
  [[ "$COCODE_LIVE_QUERY_ROSTER_SHA512" =~ ^[0-9a-f]{128}$ ]] \
    || { echo "invalid pinned query roster digest" >&2; exit 4; }
  driver_args+=(--query-roster-sha512 "$COCODE_LIVE_QUERY_ROSTER_SHA512")
fi
if [[ -n "${COCODE_LIVE_DEPLOYMENT_POLICY:-}" ]]; then
  test -f "$COCODE_LIVE_DEPLOYMENT_POLICY"
  driver_args+=(--deployment-policy "$COCODE_LIVE_DEPLOYMENT_POLICY")
fi
"$driver" "${driver_args[@]}"
