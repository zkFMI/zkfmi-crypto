# PQC greenfield proof integration — implementation checkpoint, 2026-09-09

Status: **research E2E verified on isolated five-validator networks; full greenfield PQC integration incomplete, existing deployments unchanged**.

The user requested one overall PQC on/off switch and migration of proofs,
commitments, private distributed proving, and the DeFMI verifier. That full
acceptance target is unchanged. The earlier hybrid-authorization release does
not meet it, and the code below must not be presented as doing so.

The user clarified on 2026-09-09 that PQC may start on a new network and that
existing-ledger migration is not required. The active implementation target is
therefore greenfield PQC startup and application integration, with old ledgers
and deployments left untouched. This supersedes the earlier requirement to
design or implement migration of existing balances/state; it does not waive
cryptographic security, operator custody or end-to-end verification gates.

### Greenfield execution, verified research slice

The additive Genesis v3 implementation commits the shared deployment policy to
authority and state/restart behavior. It does not import an older ledger.
Greenfield live 001 reached five healthy validators but was blocked before proof
transactions: the newly generated JSON fixture rounded every committee key's
`not_after` integer from `9223372036854775807` to `9223372036854776000`.
The VM correctly refused the exact committee mismatch; this was a fixture bug,
not a reason to relax admission checks. The original manifest and malformed
fixture remain only as reproducibility inputs for that failed run.

`greenfield-on-genesis-v2.json` preserves the exact original public committee
integers (verified with an integer-preserving JSON parser). Experiment 002 used
the same frozen VM and driver, a new empty runtime, and the corrected fixture.
It completed in **1131.302 seconds**, primary metric **1**, verdict
**smoke_only**: both fill/no-fill proofs finalized through **740 actual accepted
transitions**, with exact On policy readback and equal roots on all five nodes.
Node3 recovered both a pending 256,567,624-byte proof-upload state (769 ms) and
the completed state (704 ms); all eight negative gates passed. The final root
was `0d14f6c910106635f027ad0f7d588ae10367985c83eb76e344dfbbe87f4704b9`.

Current stage: `RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC` completed for
the greenfield On slice only; verdict `smoke_only`. The earliest unresolved
overall gate is actual owner-local venue input and canonical-state proof linkage.

Contract `zkfmi-greenfield-pqc-live-v1` SHA-256:
`c2dcd5c1c2b2f4cc53b91f61bcfae54dd2efedaf38cb2717616b5ce3b81408ce`.
Manifest `greenfield-on-experiment-002.json` SHA-256:
`09b9c4ad61e6cf887002026b2d960773d92d8e6869944a17d9fecd9a30f3dc48`.
The [live receipt](../research/cosnark-trial/artifacts/greenfield-live-002/live-receipt.json)
has SHA-256 `e6a38200e23bcd014189e1a418b1c1eddebd5c04c2d0d3b60a2295c3d6a4073f`.
The append-only live ledger preserves all earlier bytes and adds the blocked
001 and smoke-only 002 outcomes separately. Source/binary/input hashes are in
the receipt; no private owner inputs or keys were mounted in the network run.

Native runner verification on Softbank passed 27 all-target tests and all-target
Clippy with warnings denied. The live VM/driver use canonical freeze002
(`584d5644...`, `3858f410...`). Current-source DeFMI tests/Clippy use freeze003:
three added tests, formatting and restored legacy invalid-key diagnostics differ;
valid policy/genesis/restart semantics do not. Binary identity is not claimed.
DeFMI feature verification passed 72 unit, 2 network, 4 application-boundary
and 2 recovery tests; default verification passed 67, 2, 4 and 2 respectively.
Both all-target Clippy configurations and touched-file formatting passed.
Whole-workspace `cargo fmt --check` still reports unrelated pre-existing
bench/source formatting differences, which were not changed.

Actual QOMM/OCLOB venue integration remains separate: their persisted MPC shares
use the Ed25519 scalar field and different wire semantics, while the research
coCode prover uses BN254. Fresh owner-local input ingestion and venue/canonical
state constraints are still required; existing MPC persistence is not imported
or centrally reconstructed. A new Genesis alone does not complete this work.

The subsequent implementation adds `integration/owner_input.rs` and an explicit
`integration-worker-v2-owned` launch. Fresh participant-edge degree-two BN254
evaluations use the already pinned arkworks polynomial implementation. Workers
load only their own private contribution file, check the deployment/book/
operation/sequence binding, and initialize the existing financial relation
without synthetic balances or quantity/price. The MPC checks degree-two
consistency and reveals only a Boolean rejection, not residuals or inputs.
The `owner-input` feature permits a participant client to use this adapter
without linking the native MP-SPDZ compiler/runtime feature.

`seal_for_worker` / `open_at_worker` now adapt the existing hybrid KEM/AES-GCM
delivery implementation and policy-bound hybrid signatures to the six-field
input. A dedicated `PrivateMpcInput` purpose preserves the old purpose codes.
The signed/encrypted context binds the expected party, deployment/settlement
binding, pinned participant/key ID and version, and both public keys. The worker
verifies the externally selected registration record and signature before
decryption; the packet cannot select its own trusted sender key. Registration
authenticity and account ownership remain application obligations. The sealed
worker path below now supplies actual-clock validation and durable one-shot
consumption for its pinned laboratory registrations.

This is **not yet verified venue acceptance**: actual participant enrollment,
order constraints and the native venue/canonical-state linkage remain to be
connected. The sealed worker path below closes the laboratory encrypted-input
delivery gap, not those operational authority gaps. In particular,
the original file-input launcher does not acquire authenticated provenance just
because the separate sealed-input adapter exists.

The initial SSH observation failure was resolved using nested SSH from OmenX
to Softbank; direct laptop access is excluded by the user's IP restriction.
On reconnection the `owner-input-001` build log was absent and no running
container mounted that source. Current source was copied to the separate
`owner-input-002/src` directory and container `pqc-owner-input-002-gate` was
started for remote build/tests/Clippy. It completed with exit 0: release build,
23 tests with the client-only `owner-input` feature, 30 native all-target tests,
and warnings-denied all-target Clippy passed. The normalised Rust-source hash
matches the local tree; the copied log hash also matches the remote original.
See the [source-gate receipt](../research/cosnark-trial/artifacts/owner-input-002/source-gate-receipt.json)
and [exact gate log](../research/cosnark-trial/artifacts/owner-input-002/gate.log).
Those deterministic checks do not by themselves exercise the owned initializer
or establish an operational venue connection. The subsequent whole-path run
below now supplies the former, not the latter.
The seven touched owner-input/native integration Rust files pass local
`rustfmt --edition 2021 --config skip_children=true --check`; this is parsing
and formatting evidence only, not a type check, test or proof execution.
No local build/test replaced the remote gate, and the earlier job was not
restarted based only on an observation timeout.

### Owned-input native-to-canonical execution

`integration-full-owned` now runs the existing atomic result-first harness with
manifest-pinned public bindings and seven private input paths for each operation.
The separate `integration-lab-owner-inputs` process reads laboratory values from
a private file and emits per-party degree-two evaluations. Values are not
command-line arguments or coordinator messages. This same-host file workflow
does not implement participant enrollment or encrypted worker transport.

The first whole-path run completed on Softbank through OmenX in **639.496 seconds**:
prediction **1**, observed **1**, verdict **smoke_only**. Actual owned-input fill
and persisted-book recovered no-fill proofs passed separate public verification
and **740 canonical Block/State transitions**. The owner-supplied insufficient
balance was rejected by all seven native parties. All six proof/native and eight
canonical negative checks passed; 367 uploaded proof chunks survived canonical
recovery. Public-program readback matches the native receipt and confirms that
the new six-field owner initializer actually ran, without the old fixture path.

Both proofs were then accepted in a separate network-disabled, read-only
container mounting only the frozen runner and public artifacts, with no keys,
private inputs or shares. Current-source verification passed 23 client-feature
tests, 30 native all-target tests and warnings-denied Clippy. The 28 hashed
source/manifests excluding seven noncompiled AppleDouble metadata entries match
the local tree. The main ledger preserves its previous bytes and appends the
new verified receipt; the isolated remote snapshot's older ledger was not used
to overwrite current history.

Contract `zkfmi-cocode-owned-native-integration-v1`, SHA-256
`5352072bd2e087f2f8f3483bcb1f71612ce9c88a97002a8a94cbda2a7003ac9c`.
Manifest SHA-256 `62e3c2dee7a4dd21019a93a896056f93e56c83fe54fe5456b9e9e9241b91c5a7`.
See the [whole-path receipt](../research/cosnark-trial/artifacts/owned-native-001/receipt.json)
and [verification/readback](../research/cosnark-trial/artifacts/owned-native-001/verification.json).
The stage `RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC` is observed for this
laboratory slice. The earliest unresolved overall gate remains actual enrolled,
role-bound owner input and venue/order-to-canonical linkage. This is not actual
QOMM/OCLOB matching, a newly deployed validator network, independent operator
custody or security promotion. The overall On/Off objective is still incomplete.

### Authenticated encrypted worker input — whole-path execution

`integration-worker-v2-sealed` now authenticates the pinned owner registration,
recipient key, deployment/operation binding, purpose and actual clock before
decrypting its own packet. Successful admission durably consumes a semantic
operation identity before MPC initialization; ciphertext changes or key rotation
do not reset the budget. The shared sealed-message API accepts a resident
decapsulation capability and rejects a non-hybrid suite before calling it.
The existing raw-input runner remains separate; sealed runs cannot fall back.

Separate laboratory key-owner processes generated one sender and seven recipient
keys. A separate sender process produced 21 signed hybrid-encrypted packets.
The main runtime did not mount the sender signing seed, clear values or source
fixtures. It did mount all seven laboratory recipient key paths on one host;
this is not independent custody, a production keystore or real venue enrollment.

Experiment `cocode-sealed-native-001` completed in **635.727122121 seconds**:
prediction **1**, observed **1**, `smoke_only`. Both native proofs, recovered
no-fill, **740 canonical Block/State transitions**, six proof/native negatives
and eight canonical negatives passed. Public-only verification accepted both
proofs in a separate network-disabled read-only container with no private mounts.
A newly encrypted fill packet was rejected by a new worker process using the
durable consumption journal, before any query key or native engine was available.

Postflight verification passed **26 client-feature tests**, **33 native tests**,
all-target warnings-denied Clippy and **2 shared sealed-message unit tests**.
The latter used the root library's locked dependency set with network access
after the offline cache lacked an optional dependency; it did not change the
lockfile. The first added test version had compile errors, corrected solely in
test code. The executed production source prefix is unchanged.

Contract `zkfmi-cocode-sealed-input-integration-v1`, SHA-256
`5b2446b4da86b91531de18ec783eb6090b19febb6c3230b116480a59fa44c839`.
Manifest SHA-256 `1257aaea12198b02a0605c4af18fa05e99a33c44a8bf87d8ade3cbd1355638ac`.
The [whole-path receipt](../research/cosnark-trial/artifacts/sealed-input-001/receipt.json)
has SHA-256 `93759de8749c8c079c3813d61b7520d005665cef7ead09cbb3d62db41d0e7a60`;
[postflight verification](../research/cosnark-trial/artifacts/sealed-input-001/verification.json)
records exact source, container and replay evidence.

The overall objective remains incomplete. This packet has one owner for six
fields; OCLOB has separate buyer/seller authority and agreed matching outputs.
Its authoritative state consumes escrow notes, not this laboratory four-account
book. Neither a reservation entity commitment nor the coCode book commitment is
an owner account address or interchangeable canonical state root. Operational
role/asset/reservation constraints and canonical linkage, meaningful Off,
independent custody, consumed-input crash recovery and formal PQ composition
remain unresolved. Existing ledgers and deployments are unchanged.

### OCLOB greenfield deployment-policy integration

The common policy is now mandatory across OCLOB provisioning, all seven node
configs, corporate/market configs, startup checks, stores and journals.
Canonical policy bytes bind restart and the native DKG session; missing or
different policies cannot retrofit existing state. The three remote E2E Make
entry points require an explicit deployment ID and `on|off`, with no default.
Existing ledgers and canonical dependency pins remain unchanged.

Current OCLOB proof dispatch is still classical. `On` therefore rejects before
provisioning keys or opening proof state; the owned-input/coCode backend has
not been connected to this dispatch. This is a startup boundary, not completed
PQC venue operation.

Through OmenX, Softbank `make remote-test` completed the workspace test phase:
**153 passed, 0 failed, 1 ignored**, including 78 node unit tests and two real
provisioning CLI tests. Those CLI tests generated actual Off keys/configs,
checked all 11 configs against one policy, and checked On/missing-policy
rejection without creating a cluster. All 82 Rust files in the frozen source
manifest match the local source. This uses a documented isolated source overlay
for the shared crypto crate; release-pinning is still pending.

The overall source gate is **partial**: Clippy stopped at the unchanged
`oclob-mpc/src/lib.rs:375` range loop. A separate remote whole-workspace format
check also reports pre-existing differences in unchanged files. Neither was
silently fixed or waived. Exact hash-verified logs and the limits are in the
[OCLOB source-gate receipt](../research/cosnark-trial/artifacts/oclob-policy-001/source-gate-receipt.json).
This does not cover a seven-party owned-input native proof, an actual financial
venue journey, seven independent operators or security promotion.

## Implemented in the shared library

- `src/mode.rs`: explicit `PqcMode::{Off, On}`, strict serialization, versioned
  deployment policy, policy equality check for future peer/state admission, and
  actual Ed25519 versus Ed25519+ML-DSA-65 signing/verification. Both the mode and
  deployment identifier are bound to each signed message. There is no default,
  negotiation, automatic key rotation, or invalid-value fallback.
- `src/commitment.rs`: salted, domain-separated SHA-512 commitments using
  RustCrypto, independently OS-random 64-byte salts, and zeroizing opening
  custody. Context, policy, value length, and value are committed. This is not
  homomorphic, is not a ZK proof, and is not yet checked inside a ZK relation.
- `tests/deployment_mode.rs` and `tests/hash_commitment.rs`: six added tests,
  including both real signature modes, cross-network/mode rejection, modified
  values, changed contexts, and independently randomized commitments.

The proof-security enum is backend-policy metadata only, not proof verification
or a wire-provided attestation. No PQ proof backend has been registered.

## Reference provenance and resolved implementation decision

The author implementation for [Code-based Scalable Collaborative SNARKs,
IEEE S&P 2026](https://eprint.iacr.org/2026/729) is available at
<https://github.com/ChristodoulosPappas/Code-Based-Scalable-coSNARKs>.
Inspected revision: `1c50a7410fa464ad325e0014d0d5fedd196d86c5`.
No top-level license or code-use grant covering the proof core was found in that snapshot; GitHub's
repository-license endpoint also returned 404. Some bundled hash helpers have
their own licenses; those are not assumed to cover the proof core. The paper's CC BY grant is not
treated as a license for the separately published software. Its code has not
been imported into a crate, built, executed, or redistributed by this task.

The user explicitly authorized a research-only independent implementation from
the paper on 2026-09-08. This resolves the implementation decision; it does not
grant rights to reuse the separately published proof core. The independent
implementation remains unreviewed, and no author has been contacted or external
message sent.

Plonky3 was also inspected at
`7230fc572870436e6651762f35c6c3f3f48960d2` (MIT/Apache-2.0). It includes a hiding
FRI PCS and single-prover ZK tests, but this alone is not a secret-shared
multi-party prover. Replacing the distributed witness with a central witness,
or replacing proof soundness with quorum signatures, is outside acceptance.

The user's requested supplementary paper search is recorded in
[PQC_COLLABORATIVE_PAPER_SURVEY_2026-09-08.md](PQC_COLLABORATIVE_PAPER_SURVEY_2026-09-08.md).
It distinguishes licensed distributed FRI/PCS references from genuinely private
collaborative proving and does not change the approved implementation target.

## Verification

Run on `softbank-l40s`, in the existing pinned Rust build image
`sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97`:

- `cargo test --locked`: **54 passed**, no ignored tests.
- `cargo clippy --locked --lib --tests -- -D warnings`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: fails on an unchanged
  redundant closure at `examples/pqc_bench.rs:184`. This task does not edit it.
- Only the four added Rust files were formatted; no bulk formatting of existing
  sources was applied.

Logs: `/home/ubuntu/work/pqc-proof-switch-20260908/artifacts/` on Softbank,
copied to `.artifacts/pqc-proof-switch/` locally. These are deterministic
library tests, not research promotion or transaction end-to-end evidence.

## Isolated private proof smoke

The independently written, explicitly normalized coCode/Spartan variant in
[`research/cosnark-trial`](../research/cosnark-trial/README.md) now executes the
complete small financial relation using actual seven-party, threshold-two
malicious-Shamir MPC. Final run `cosnark-rough-012` completed in 25.933 seconds:
standalone public proof accepted, all seven declared negative cases rejected,
and every owner rejected a repeated opening-query request. Its proof is
96,175,154 bytes. A read-only, network-disabled verifier also accepts the
public artifact without mounting run-specific private state. Final trial
formatting, all-target Clippy and 11 unit tests pass.

The [receipt](../research/cosnark-trial/artifacts/rough-012/receipt.json) records
`smoke_only`, not promotion. This is a single-host synthetic-input experiment,
not independent operators, production zero-knowledge/quantum security
validation, or operational DeFMI financial commitment verification. No release
proof backend or consumer application was changed by that original isolated
trial; the subsequent consumer change is described below.

## Canonical research integration

The separate [`integration` module](../research/cosnark-trial/INTEGRATION.md)
now binds an actual native N7/T2 private 64-bit DvP relation to a canonical
four-account DeFMI research book. The opt-in `research-cocode` VM feature adds
immutable deployment policy, authorized genesis, bounded proof chunks,
actual proof verification inside `State.apply`, authoritative before/after
commitments, operation replay protection, no-fill conservation and persisted
state recovery. The final whole run `003` passed in 645.231 seconds, including
all 367 uploaded fill-proof chunks surviving State/state-sync recovery before
commit, two real proofs, 740 canonical blocks, eight canonical negatives and
four proof/native negatives. Both final proofs also pass in separate public-only
read-only, network-disabled verifiers. See the
[verification receipt](../research/cosnark-trial/artifacts/verification-integration-003.json).

This is real canonical library execution using actual private MPC proof
artifacts, not mocked verifier responses. It is not live Avalanche consensus
or operational QOMM/OCLOB integration. Default-off VM behavior, transaction and
block limits and existing deployments are preserved. The research snapshot
envelope alone is enlarged, with an aggregate pending-proof byte budget.

## Still required

### Verified live v1 and native v2 follow-through

The frozen native `003` proofs completed a fresh, isolated five-validator
AvalancheGo execution in `live-006`. All manifest-bound attempts remain in the
[live ledger](../research/cosnark-trial/live-ledger.jsonl):

- `live-001`: blocked before startup by an unregistered container UID.
- `live-002`: blocked by the network runner's read-only service directory.
- `live-003`: blocked because VM subprocess startup did not preserve the
  dynamic-library environment. A derived runtime image now registers the
  already pinned OpenSSL and MP-SPDZ libraries without changing VM bytes.
- `live-004`: all five custom-chain validators became healthy and accepted
  real proof-chunk transactions, but a database request reached 67,159,447
  bytes, above the unchanged 67,108,864-byte RPC limit. The node shut down;
  the ensuing HTTP 500 is a failure, not a successful negative test. The
  receipt records `blocked` after 159.00981195 seconds.

- `live-005`: the immutable proof-blob/reference persistence adapter removed
  the database-size failure without raising limits. The complete pending
  upload was persisted, but metadata was read before the restarted custom-chain
  RPC route was ready. HTTP 404 correctly blocked the run.
- `live-006`: the existing bounded root-recovery wait now precedes strict
  metadata readback. Both proofs finalized in 1,087.156683091 seconds, with
  740 accepted transitions and all eight canonical negative gates passing.
  All 367 pending fill chunks survived an actual validator restart; pending
  State was 256,566,989 bytes and its state-sync snapshot was 257,201,197 bytes.
  All five roots matched before and after both the pending and final restarts.
  The final root is
  `93b232c805b0af810ead6ce3000770b14b22d8b0432a1f53a90350124198e7ed`.
  See the [actual live receipt](../research/cosnark-trial/artifacts/live-006/live-receipt.json).

The new [v2 query-agreement adapter](../research/cosnark-trial/QUERY_AGREEMENT_V2.md)
uses an explicitly pinned roster, seven Ed25519 AND ML-DSA-65 approvals and
durable owner-local prepare/consume markers before releasing Merkle openings.
Its native whole run `query-v2-002` passed in 658.098792165 seconds, including
both fresh proofs, six proof/native negative gates, eight canonical negatives
and the explicit canonical roster/protocol pin. Both proofs also passed in
separate read-only, network-disabled containers with public artifacts only;
neither owner keys nor witness state were mounted. See the
[v2 native receipt](../research/cosnark-trial/artifacts/integration-query-002/receipt.json).
The earlier `query-v2-001` remains blocked: its read-only enrollment mount
prevented durable marker creation. Run002 changed only the scoped writable
budget submount and used a fresh private witness execution.

The v2 actual-network run `live-query-007` completed under its
[separate contract](../research/cosnark-trial/live-query-contract.json) and
[sealed manifest](../research/cosnark-trial/live-query-experiment-007.json).
Its [actual live receipt](../research/cosnark-trial/artifacts/live-query-007/live-receipt.json)
records `smoke_only`, primary metric 1 and 1,087.020500307 seconds. All 740
transitions and eight canonical negative gates passed with the expected v2
protocol and immutable roster pin. The full 367-chunk pending proof survived
an actual node3 restart (256,567,283-byte State, 257,201,735-byte snapshot).
All five roots matched after the final restart:
`b7b03f26329691f51eedd67f6b624aabb210ee44b43c9e39f4be0a4eb15fc31e`.
The live receipt SHA-256 is
`1a4e0f58e2788676f8e167d9d850ab5ca28791dfaadbfd138eecd72a577c6880`.

Current native tests (27), VM feature-on tests (76), VM default-off tests (72),
format checks and strict Clippy pass. Content-checksum comparison finds no
differences between the current trial/VM source trees and executed live007
source. The research default-off tests explicitly preserve legacy persistence
without content-addressed proof writes. The compact
[follow-through verification](../research/cosnark-trial/artifacts/verification-followthrough.json)
links exact contracts, manifests, receipts, binary hashes and scope limits.

The independent model-assisted
[security-gate review](COCODE_SECURITY_GATE_REVIEW_2026-09-09.md) remains red.
Its immediate implementation requirement, authenticated agreement among all
seven owners on the complete final-row/challenge/query view before openings,
has a separately versioned research implementation and the native evidence
above. It does not supply the missing
exact-construction soundness/ZK/composition proof, concrete QROM parameters, or
independent operator custody. The earlier sparse-mask attack candidate was
retracted and is not a reason to alter the padding.

### Remaining acceptance gates

1. Independently validate the implemented normalized/masking composition and
   concrete security parameters before promoting the research-only proof core.
2. Extend the implemented committed four-account relation to the operational
   venue lifecycle without reconstructing private witnesses at a coordinator.
   Include quantum soundness and hiding parameter analysis; do not retain a
   pairing-based final proof wrapper.
3. Wire the common policy through application startup, transport/envelopes,
   enrollment/custody, public statement formats, consensus and persisted state.
   Bind the chosen mode to fresh genesis/startup and reject cross-policy peers
   and state. In-place mode changes and existing-ledger migration are out of scope.
4. Migrate operational consumer proofs and authoritative assets beyond the
   opt-in canonical research book and verifier on fresh networks only; do not
   import or alter existing ledger state.
5. Run both deployment modes through independent seven-party operators and
   live DeFMI/venue settlement, including negative cases, actual reservation
   refunds, replay and network/restart recovery. The bounded single-host
   proof-to-canonical path is not this full operational gate.

The changed scope includes the described research proof/owner-input adapter,
opt-in DeFMI consumer and in-progress OCLOB startup/persistence policy guards.
These source changes do not establish completed operational venue integration.
No public service, deployed state, remote branch or main branch was changed.
The work remains uncommitted.
