# P1 service integration checkpoint

Status: **partial implementation candidate; the full PQC migration remains open**.
All eight repositories use isolated `codex/pqc-full-integration` worktrees. No
shared-main source, deployment or existing application/node state was changed.

## Implemented paths

| Boundary | Candidate behavior |
| --- | --- |
| QOMM/OCLOB service connections and DeFMI HTTPS | TLS 1.3 with only X25519MLKEM768; incompatible peers fail the handshake; fresh sessions cannot resume classical parameters |
| Native MP-SPDZ | The upstream TLS adapter enforces the same group; the Rust build and runtime verify the pinned adapter and actual library/executable against a local build receipt |
| OCLOB stored party/capability shares | Explicit X25519+ML-KEM-768 suite, authenticated ciphertext and recipient/context binding; old envelope version 5 is refused |
| QOMM winner-only delivery | Version 2 fixed-size hybrid KEM/AEAD envelope; both ciphertext components, suite, context and quote are bound; no recipient key-privacy claim |
| FROST DKG private round-two exchange | Hybrid envelopes bind sender, recipient, session and roster; the sender signature must match the pinned roster; replay/restart behavior is preserved |
| Encrypted key stores | Independent 96-byte hybrid seeds, public/secret consistency on restore, validity and expiry checks, no hybrid-to-classical rotation |
| Common code ownership | Fifteen duplicate crate directories retired; unique native proof/settlement fixes retained in their canonical owners, recorded in INTEGRATION_SOURCES.json |

The proof-party durable schema is version 4. Versions 2 and 3 are refused with
an explicit migration error, leaving their bytes unchanged. No old proof state
was silently initialized, deleted, or treated as a new identity. OCLOB node
configuration and public cluster configuration also have explicit new versions.

## Verification

All builds and deterministic tests ran on softbank-l40s, using official Rust
1.97.1 in the recorded image. OCLOB runs used `make remote-test` with the
isolated integration profile. Logs are retained under this worktree's
`.artifacts/`, with remote copies in `~/work/pqc-full-integration-20260905`.

| Log prefix | Observed result |
| --- | --- |
| 20260905T141242Z-qomm-rust | Eight node-service tests passed |
| 20260905T141531Z-qomm-rust | Three real TCP/mTLS tests passed: hybrid reconnect and rejection of either classical-only peer; all-target Clippy passed |
| 20260905T142708Z-oclob | Eight edge-envelope tests and six real node network/proof-network tests passed |
| 20260905T145437Z-zkfmi-crypto | All-feature/all-target workspace checks passed across the dependency matrix, with the existing QOMM helper-autobin exception described below |
| 20260905T150556Z-qomm-rust | Seven key-store tests, five winner-envelope tests and two distributed-assembly tests passed |
| 20260905T151223Z-qomm-rust | 48 library, five DKG and five winner-envelope tests passed; all-target transport Clippy passed |
| 20260905T152721Z-oclob | Native-linked OCLOB workspace check and six network tests passed; this run did not execute matching |
| 20260905T155940Z-oclob | Seven real native malicious-Shamir parties executed the canonical matching circuit twice and agreed; all-target node/MPC Clippy with warnings denied passed |

The native regression uses the existing Rust adapter to invoke the upstream
compiler, real random Shamir sharing, real native processes and their actual
network channels. It is a deterministic single-host regression, with no mock
matching engine. It is not a live seven-validator DeFMI settlement run or a
comparison of latency, throughput, economic results or independent operators.

The QOMM workspace already places a helper file
`qomm_live_acceptance_report.rs` in a directory that Cargo auto-discovers as a
binary, although it has no entry point. The matrix excludes `qomm-demo` from
the blanket all-target check and checks its library and all four actual
application binaries explicitly. This existing packaging issue was preserved.
The 141203 log printed Cargo help because of an initial runner quoting error;
it is not verification evidence. Later failed intermediate runs remain in the
log directory and are not counted as passes.

The final source snapshot and log hashes are recorded in
[P1_INTEGRATION_2026-09-06.json](P1_INTEGRATION_2026-09-06.json). The
160739 QOMM run additionally passed two explicit engine-rejection tests and
all-target QOMM MPC/transport Clippy with warnings denied. All 570 source
inputs match the recorded remote bytes after importing formatting changes.

## Remaining gates

- **P1:** Remaining classical note-opening envelopes; seven-node WAN baseline
  comparison, persistent-session/rekey behavior across actual native lifecycle
  and operator configurations. Existing explicit localhost HTTP test modes do
  not constitute secure WAN deployment evidence.
- **P2:** Actual per-node ML-DSA settlement approvals, authenticated DeKYX-bound
  committee enrollment, trusted same-committee quorum verification, canonical
  payment/typed wire versions, stale-key/committee and cross-DeFMI rejection.
- **P3/P4:** Credentials, revocation, corporate key rotation, guarantees,
  collateral, clearing and cross-DeFMI authorizations at their real boundaries.
- **P5/P6:** Public audit proof migration, historical checkpoint provenance,
  classical-only retirement and recovery acceptance.
- **Release:** Current immutable baseline Git pins are supplemented by local
  integration path patches. Publish reviewed owner commits, update pins and
  regenerate locks without those patches before independent-consumer release.

TLS certificate authentication and DKG identity signatures remain classical.
FROST, Pedersen, Bulletproofs, Triptych and the existing proof circuits have not
become quantum-resistant merely because their transport is hybrid. In
particular, this checkpoint does not satisfy the P2 committee threshold claim
against a quantum attacker, nor the P5 public-proof claim.
