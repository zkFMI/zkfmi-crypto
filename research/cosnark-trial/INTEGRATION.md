# Canonical DeFMI research integration

Status: **opt-in research implementation; not a production PQC migration**.

The integration binds the private proof to a canonical DeFMI four-account
book. It is separate from the earlier isolated 64-row financial trial in
[README.md](README.md). Existing deployments and the default VM path remain
unchanged. No commit, push, asset migration, or deployment is implied.

## Implemented path

The `research-cocode` feature in `defmi-avalanche-vm` enables an immutable
research deployment policy, governance-approved book genesis, and canonical
proof begin/chunk/commit transactions. `State.apply` calls the actual
`zkfmi_cosnark_trial::integration::verify_serialized` verifier before changing
the authoritative sequence and book commitment. A successful commit consumes
the upload and records its operation ID; stale sequences and reused operations
are rejected. The same governance authorization used by the VM protects these
transactions; it does not replace the mathematical proof.

The four ordered positions are seller traded asset, buyer traded asset,
buyer cash asset, and seller cash asset. Genesis enforces matching asset rails
within each pair, distinct traded/cash rails, and distinct account/asset
handles. The private relation checks 64-bit before/after balances, quantity,
price and product; exact multiplication; and conservation of both rails.
The no-fill relation keeps all four balances unchanged. It is not a claim
that an operational venue reservation/refund adapter has been implemented.

Seven native workers retain private inputs, Shamir shares and commitment salts
under owner-local interfaces. Actual pinned malicious-Shamir MPC uses `N=7`,
`T=2`, native security 128, and the exact scalar field. The coordinator has no
private witness reconstruction API. The previous output commitment is recovered
exactly as the next authoritative input commitment, without reconstructing its
balances. The initial balances are distributed synthetic research fixtures,
not issued operational assets.

The integration uses a 1,024-row relation, 11 sumcheck rounds and four masked
code-based openings. The unchanged PCS dimensions are a 16,384-element message
domain, a 65,536-element codeword domain and 512 queries per opening. SHA-512
commitments/transcripts and fresh salts are used without curve commitments or
a pairing proof wrapper. BN254 denotes the arithmetic scalar field only.

## Transport and recovery

Proofs are uploaded in 512 KiB raw chunks. Canonical transaction and block
limits remain 1 MiB and 1.5 MiB. Each proof and the aggregate reserved bytes
across pending uploads are bounded at 256 MiB. The opt-in research snapshot
limit is 512 MiB to fit base64-encoded uploads plus existing state; the
default-off snapshot limit stays 66 MiB. SHA-512 chunk and whole-proof digests
bind the upload. These are transport checks, not substitutes for verification.

Recovery covers canonical state and state-sync snapshots, including a fully
uploaded public proof before commit and completed settlement state. The next
native transition recovers the prior private book through per-owner persistence
and salt custody. One-shot owner query/capability consumption fails closed.
This does not establish arbitrary crash-safe regeneration of an interrupted
private proof or availability under operator faults.

## Verification record

The first completed whole run, `cocode-defmi-integration-002`, passed in
640.745 seconds with two real 192,349,020/192,349,021-byte proofs and 740
canonical block/transaction roundtrips. Both standalone proof verifications,
the declared negative cases, owner replay guards and post-settlement recovery
passed. The earlier `001` stopped before MPC because clean-image lab TLS key
permissions were unreadable by the isolated user; its blocked receipt is
preserved. The fix copied the clean engine into a task-local owner-readable
directory without changing global permissions or proof parameters.

The final preregistered replication, `cocode-defmi-integration-003`, passed in
**645.231 seconds** (receipt duration), matching the predeclared binary
prediction: primary metric 1, prediction error 0, verdict `smoke_only`.
Its two fresh proofs are again 192,349,020 and 192,349,021 bytes. All 367 fill
proof chunks survive State JSON and state-sync recovery before commit. The
actual pending snapshot is 257,194,340 bytes across 491 state-sync chunks.
The recovered upload is then committed, completed state is recovered again,
and the no-fill proof is accepted against that authoritative book. The full
path applies 740 canonical blocks.

All eight canonical negatives and four proof/native negatives pass, including
a reused operation ID newly authorized against the recovered current root and
a changed output commitment supplied with a matching altered external
statement. Each proof's seven owner-local repeated-query requests are rejected.
The two positive native executions run 23 programs each: 322 party processes
exit zero. The invalid-product program makes all seven parties exit one with
the exact `invalid financial witness` marker, not an unrelated runtime error.

Final public evidence:

- [Manifest](integration-experiment-003.json),
  [whole receipt](artifacts/integration-003/receipt.json), and
  [canonical receipt](artifacts/integration-003/canonical-receipt.json).
- [Fill proof](artifacts/integration-003/fill/proof.json) and
  [no-fill proof](artifacts/integration-003/nofill/proof.json), with adjacent
  public statements and native records. Large generated artifacts are ignored
  by Git but retained locally and in the isolated remote run directory.
- [Final verification](artifacts/verification-integration-003.json) and
  [371 artifact identities](artifacts/integration-003-identities.json).
  All 147 corresponding current local source/manifest files match the executed
  remote bytes; all 74 pinned native/compiler/path dependencies match the
  earlier recorded dependency identities. Every one of the 47 native schedules
  retains security 128.
- [Append-only integration ledger](integration-ledger.jsonl): `001` blocked,
  `002` and `003` smoke-only. Frozen sources and executables for all three
  attempts are retained remotely under `source-snapshots/NNN`.

Final manifest SHA-256:
`da99f346670c53e1616345a270914c96726674b4e30d3ffa288f83200a0e9ea9`.
Final receipt SHA-256:
`44db45723797a0df0877b3bd184585a659eb5cc1cc49330affa57cc83745d83a`.

All builds and executions are remote-only on `softbank-l40s`, using pinned
image `sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4`
and the same supported arithmetic-only MP-SPDZ engine as the original trial.
Native deterministic gates pass 19 public-fixture unit tests, warnings-denied
all-target Clippy, formatting and verifier-only compilation. These tests do
not replace private MPC or canonical acceptance.

DeFMI release `--locked --all-targets` tests pass 66 cases with the feature
off and 67 with it on. Both configurations pass warnings-denied all-target
Clippy. The current standalone verifier also accepts both final integration
proofs and the original `rough-012` public proof in separate read-only,
network-disabled containers with no run-specific private state mounted.
Static and execution logs are retained under
[`artifacts/integration-003-checks`](artifacts/integration-003-checks).

Run the closed manifest through the atomic result-first entry point, never
reuse an output or remove its claim:

```sh
cargo build --release --locked
zkfmi-cosnark-trial integration-full \
  <fresh-private-parent> <fresh-public-output> \
  integration-experiment-003.json <canonical-acceptance-binary>
zkfmi-cosnark-trial integration-verify <proof.json> <statement.json>
```

Build the canonical binary from the DeFMI Rust workspace with
`cargo build --release --locked -p defmi-avalanche-vm --features research-cocode --bin cocode-canonical-acceptance`.
The consumer uses the verifier-only dependency (`default-features = false`).
The separate native trial lock retains rand 0.8.5; DeFMI retains its existing
compatible rand 0.8.8 rather than downgrading the workspace lock.

Contract: [`zkfmi-cocode-defmi-integration-v1`](integration-contract.json).
Contract SHA-256:
`44b45868cadcefa44340f42bc7199762d7d1f8a9ebb78e86496667b06804944f`.
Stage: `RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC`.
Maximum verdict: `smoke_only`. Prediction for the binary final metric: 1.

## Subsequent live and v2 evidence

The v1 `live-006` follow-through used these frozen public proofs on five actual
AvalancheGo validators. Both proofs finalized through 740 accepted transitions;
the complete 367-chunk pending upload and final state survived actual node3
restarts with all five roots equal. Its
[live receipt](artifacts/live-006/live-receipt.json) remains `smoke_only`.
The RPC database limit was preserved by content-addressed immutable proof-blob
storage, and post-restart metadata readback follows bounded custom-chain root
recovery instead of assuming that network-runner health means route readiness.

The separately versioned [v2 adapter](QUERY_AGREEMENT_V2.md) adds trusted-roster
all-seven hybrid query-view agreement before owner opening release and an
immutable canonical roster pin. Its real native `query-v2-002` fill/no-fill
run and public-only standalone verification passed; see the
[native receipt](artifacts/integration-query-002/receipt.json).
Current follow-through status and exact input hashes are maintained in
[verification-followthrough.json](artifacts/verification-followthrough.json)
and the [overall status](../../docs/PQC_PROOF_SWITCH_STATUS.md).

## Remaining acceptance boundaries

Independent review of the adapted zero-knowledge, Fiat-Shamir/QROM and
malicious-MPC composition and concrete quantum parameters is still required.
In particular, the blinded book is opened once as an output and once as the
next input; test acceptance is not a security theorem for this two-use
composition or for broader reuse.

This single-host laboratory does not demonstrate seven independent
operators, malicious-controller isolation, production custody,
application-wide PQ startup/transport, or
venue-specific QOMM/OCLOB lifecycle adoption. Acceptance governance uses the
repository's explicitly public development keys, separately from the seven
MPC parties. Governance signatures already use Ed25519 AND ML-DSA-65 through
`defmi::governance::GovernanceSigner`; the unresolved boundary is public fixture
key custody/enrollment and application-wide mode selection, not an absence of
PQC governance signatures. No production backend is promoted.
