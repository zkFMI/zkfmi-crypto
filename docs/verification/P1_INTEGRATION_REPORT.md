# P1-P6 integration acceptance report

Status: **scoped integration acceptance passed; release pinning pending**.

The synchronized source snapshot passed deterministic crate gates and a native,
single-host P1-P6 acceptance run on 2026-09-06. This report supersedes the old
partial wording in this file. It does not rewrite the independent
[P0 report](P0_REPORT.md) or the intermediate stage receipts; those remain
historical records of what was known at each checkpoint.

Final immutable repository revisions are intentionally absent. The user-managed
GitHub organization re-push was not complete when this report was written. Every
remote HEAD must be fetched and reconciled before release pins, locks, or a
release verdict can be finalized.

## Accepted integration boundaries

| Stage | Observed integrated behavior |
| --- | --- |
| P1 | QOMM, OCLOB and DeFMI connections require TLS 1.3 with X25519MLKEM768 and ML-DSA-65 certificate keys and chain signatures. Application services separately enforce pinned certificate-fingerprint ACLs. Hybrid recipient KEM/AEAD envelopes bind recipient, context, suite and version. Durable retry and restore paths reject legacy or malformed envelopes. |
| P2 | Execution wires are typed and versioned. Winner envelopes require roster-pinned Ed25519 and ML-DSA signatures over the same unsigned body. Native settlement requires registered per-node ML-DSA approvals, exact quorum/roster checks, and lifecycle validation at authoritative host time. |
| P3 | DeKYX issuer, status and holder boundaries use registered hybrid authorization with encrypted holder custody, independent holder possession keys, lifecycle enforcement, and fail-closed legacy snapshots. |
| P4 | Governance, participant, provider, guarantor/CSD, DeCCP, cross-domain and Aethel authority paths use registered hybrid keys. OCLOB participants issue and retain independent one-time claim keys; signed public commitments cross the narrowly scoped mTLS boundary, while encrypted seeds remain in the corporate journal. |
| P5 | Public-root, settled-batch, receipt and zkPI relation evidence is typed and bounded. Public books carry current hybrid attestations and finality receipts; fixed file and network bounds reject oversized and legacy-signature fixtures. |
| P6 | Historical checkpoint provenance, key retirement, archived recovery, exact randomized-receipt replay and fail-closed restoration were exercised by deterministic and native recovery gates. |

Aethel is an application consumer of the shared foundation. It is not a
dependency of the foundation or another protocol repository: the Cargo
manifests and resolved metadata for zkfmi-crypto, QOMM, zkPI, DeFMI, OCLOB,
DeKYX and DeCCP do not resolve an Aethel package. That boundary must be checked
again after final remote revisions are pinned.

The integrated services bind keys to protocol identities and configured purpose,
suite, generation, validity and revocation state. They do not accept an arbitrary
public key supplied inside the signed wire. Randomized ML-DSA responses that must
be idempotent are persisted before being returned and replayed byte-for-byte on
same-process retry and after restart.

## Deterministic gate evidence

All Rust builds and tests ran remotely in the recorded Rust 1.97/native build
environment. Relevant final gates include:

| Run | Result |
| --- | --- |
| `20260905T235518Z-qomm-rust` | Broad QOMM checks, transport tests, demo tests and strict Clippy passed; log SHA-256 `102a61a460470677e4775dc07922c2649fb94c2e39bb69f2e659c0598ea5229c`. |
| `20260905T235848Z-oclob` | Broad OCLOB native gate passed 125 tests and workspace all-target strict Clippy; log SHA-256 `62456e8aab955702eb1d3b66f753f08d8699969bba182d7e31322f636559a8f9`. |
| `20260906T050355Z-aethel` | Obligation-wallet registered-hybrid-key tests 10/10, umbrella E2E 1/1 and all-target strict Clippy passed; log SHA-256 `a1c1798d8eefc3c67cf2e28b5a511744d21470ade35cbc9ac4e60a05b45b652d`. |
| `20260906T062917Z-defmi-rust` | Note-chain 7/7, note-settlement 8/8, VM state/snapshot/lifecycle/recovery gates and strict Clippy passed; log SHA-256 `2ed199d537ff3625eb8ecc72bc63ae01daf98d2a64916acdc39ea6dbd738043f`. |
| `20260906T075758Z-oclob` | OCLOB claim-custody standard gate passed settlement 6/6, node 74/74, checks and strict Clippy; log SHA-256 `9bdc59114b73dd0ee29365074b45efbdc69957238aa53dec1d15d1014ac7b135`. |
| `20260906T095423Z-oclob` | Current-hybrid public-depth fixture and cap tests passed 8/8, followed by node checks and strict Clippy; log SHA-256 `e49d050799d92d0597954ebb6cdac1c3f528959908aea3b9ebd1ccb035477ff9`. |
| `20260906T102435Z-oclob` | Exact private-state receipt retry/reopen/tamper regressions, node 75/75, checks and strict Clippy passed; log SHA-256 `e93fbef01ddc9a3350d1776fa77b682e4bdceac3fc61ac95aa0e18a9e0110c25`. |

Failed intermediate runs are retained in `.artifacts/` for diagnosis and are not
counted as passes. Formatting imports were guarded by per-file hashes; the final
native runner separately proved that its local and remote source manifests were
equal before and after execution.

## Native acceptance

Dedicated run `20260906t110145z-44291-pqc-native-acceptance` completed with exit
0 from 2026-09-06T11:02:02Z through 11:17:09Z on `ngi-external022-vm1`.

The top-level log is
`.artifacts/20260906t110145z-44291-pqc-native-acceptance.log`, SHA-256
`4ea73c1419e4e8a5419e54b7012c751c8a835a46c980b3799eb6d98c782fa8ce`.
Its receipt directory has the same basename.

The run verified the fixed native base image
`sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4`
and built unique synchronized-source cluster and Avalanche images with IDs
`sha256:a23c98e6dd2cfbd079235ab869e0528b7ae0effe0569f125176a3a5d87ca87c1`
and `sha256:c249b407e4d63dc76f85ea809dc16438b7496152294dd75e51ec511f1a6b8814`.
Image labels bound the run ID, base image ID and source-manifest SHA.

The 1,119-entry source manifests were equal locally and remotely, before and
after the run. The manifest-file SHA-256 was
`3460765104640f2efd0d6e0be693b1a87cf5fd83c5c14a2347982caa0e51f51f`.
This updated report and its consolidated JSON receipt were authored after the
successful run and are not bytes covered by that manifest. The manifest remains
the immutable record of the code and protocol inputs that actually executed.
Protocol inputs and image receipts were also unchanged before and after; their
receipt SHA-256 values were respectively
`ec30e876e53ddea2b2456cd4b203e6a5da43212f74ffac3bbac1241a6ca325b9`
and `dbf184876a5a1bb20d30e892e1676269e15ffb6c6693618f6f5b9080850a60ed`.

The native adapter `Networking/ssl_sockets.h` SHA-256 was
`ccc7d6a496423ead58a3fc44f66a17f7fc692928395821bc546add38cefe58aa`.
The build log verified the adapter, `libSPDZ`, and
`malicious-shamir-party.x`. The MP-SPDZ program source SHA-256 was
`809eced81d8ffd8369d33f532c1dbab5171c6b7a4aa7a8f5a17fc3f576073121`;
the compiled-program receipt SHA-256 was
`d3a44bb5444415a45d398323c50e0325b8d80537b5a3b638ddd08d00accc4e51`.

| Scenario | Observed result |
| --- | --- |
| Market | Three orders, three rounds, two fills and notional 9,030; 14 node-finality observations; four signed claim-authorization responses; eight fresh one-time claim keys; zero post-match financial approval signatures. The market exited with the canonical code 75, restarted, recovered an intentionally lost response exactly once, and retained five equal validator roots. Artifact SHA-256 `841f54d5f59a28d2acf8f4d5c58bf8b544fa1bc11fbf6c0f5a2004fa3e9c6596`. |
| Finality | One fill at quantity 40 and price 100; seven node observations; three claims redeemed; two signed claim responses and four fresh claim keys; zero post-match financial approval signatures. Exact private-state finalization receipts survived retry and restart, a recovered note funded the next order, and five validator roots remained equal. Measured scenario duration 21,899 ms. Artifact SHA-256 `913ca3afdbd4ba1f320128d30b3925ca59cdbba046dcd1457a48e4d880c01474`. |
| Lifecycle | Two rounds and three fills, including a two-fill atomic batch; cancellation and expiry released two remainders; nine claims were redeemed; six signed claim responses and twelve fresh claim keys; zero post-match financial approval signatures. Participant-owned custody survived restart, partial-fill cancellation recovered wallet sequence 6, expiry recovered sequence 8, and five validator roots remained equal. Measured scenario duration 81,052 ms. Artifact SHA-256 `25c7dd141f634d1d779df5df6f7b6350d65f0c619d58ffae69e9f27ed0af3bd8`. |
| P6 recovery/retirement | Two tests passed and none failed. Log SHA-256 `41a5fbbb33660295bba701b57b812567578cf6d818c8aea32f84d2e05351ad44`. |

The runner observed 34 unique persistent container starts across isolated market,
finality and lifecycle projects. Each native scenario used seven MPC nodes and
compared five validator roots. Cleanup removed the run's containers, networks,
volumes, locks and processes; it did not target unrelated compose projects.

## Protocol contracts

| Scenario | Contract | Contract SHA-256 | Manifest | Manifest SHA-256 |
| --- | --- | --- | --- | --- |
| Market | `oclob-native-market-v2` | `0e01378323b129f6fdd92aa7a59d353b2a034fbc1cf404a895bbd6a4e619a185` | `oclob-native-market-pqc-custody-003` | `bec05e500fe03d2134713bb549aeea5e8cc180c27f6f2a7350b844a628c9d950` |
| Finality | `oclob-native-finality-v2` | `308d47bfc84c19784dc6606bf17db783f4d7579a4b48b5c97a12654599cf4886` | `oclob-native-finality-pqc-custody-003` | `99b469ab5fba4fc075e9fc92fb9f56fc44934c8271e0f469d1fc86453e4d64f5` |
| Lifecycle | `oclob-native-lifecycle-v2` | `74093488ce6e3accd983a8b2ad025fd0ed3226bb8cd2165a2e5024edf3904743` | `oclob-native-lifecycle-pqc-custody-005` | `297f6cd960f528791d7155ed577217c7d81873afdac2628e24d9871edcc1644d` |

These versioned contracts distinguish post-match financial approvals from
participant claim-authorization responses. Exact API response retry after a
corporate-process restart is covered by deterministic journal tests; the native
scenarios exercise the real API and custody paths but do not independently
repeat that exact API crash window.

## Experiment ledger postflight

After all scenario receipts passed, the runner prepared and the integration
guard appended one record for each scenario to
`oclob/research/experiment-ledger.jsonl`. The append-only ledger grew from 62 to
65 records. All 65 lines parse, and all 65 experiment IDs are unique. Its final
SHA-256 is
`c7c742aa5a52018f38a04e22c92b72840d1d992b72da3fe2ed5fb3437f770f76`.

The three appended IDs are:

- `20260906t110145z-44291-pqc-native-acceptance-market`
- `20260906t110145z-44291-pqc-native-acceptance-finality`
- `20260906t110145z-44291-pqc-native-acceptance-lifecycle`

Each record uses stage `RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC`, verdict
`smoke_only`, the matching v2 contract and predeclared manifest, and the
postflight `market.json`, `finality.json`, or `lifecycle.json` receipt and hash.
The ledger append occurred after the runner's source before/after equality guard;
it is postflight evidence and is not represented as an input to the run.

## Evidence limits and release gate

The native scenarios are classified `smoke_only`. They ran on one host with lab
certificates and local container networks. `independent_operators=false` and
`wan_evidence=false`. There is no physical-HSM ceremony, production deployment,
external audit, FIPS validation, performance comparison, or economic outcome
claim.

FROST, Pedersen commitments, Bulletproof/range proofs, Triptych, OR-DLEQ and
anonymous relation proofs retain their existing mathematical assumptions. The
integration adds hybrid outer authorization to active financial paths; it does
not make those proof systems post-quantum. Scoped-wallet `ViewingGrant` and
`SpendDisclosure` remain non-consensus disclosure controls.

Release remains pending until the user-managed GitHub organization re-push is
complete and all eight remote HEADs are fetched, compared with this accepted
source snapshot, pinned to immutable revisions, and revalidated with regenerated
locks. The associated machine-readable record is
[PQC_INTEGRATION_ACCEPTANCE_2026-09-06.json](PQC_INTEGRATION_ACCEPTANCE_2026-09-06.json).
