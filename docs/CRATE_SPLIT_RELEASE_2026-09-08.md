# Crate split and QOMM integration release — 2026-09-08

## Verified implementation

The venue-specific QOMM proof and transport code now belongs to QOMM.
Shared proof, committee and measurement code remains in zkPI under
`zkpi-proofs`, `zkpi-committee` and `zkfmi-measure`. DeFMI, OCLOB and Aethel
consume the shared crates through immutable git revisions. Aethel's resolved
dependency graph contains no QOMM venue crate.

The migration preserves the underlying proof implementations. It is an ownership
split and integration repair, not a claim that the mathematical proofs are
post-quantum secure.

## Integration repairs

- Verify both components of an entity's hybrid approval before deriving its
  stable identity from the deterministic classical signature component.
- Persist the first complete policy-mandate signature and verify it on reuse.
  Re-signing an unchanged policy must not create a new standing-pool identity.
- Bound packed quote sentinels by the actual proof span without weakening the
  63-bit execution/masking configuration.
- Enroll intended recipients before constructing the actual note settlement.
- Compare no-fill refunds against the exact canonical facility transition,
  including outstanding usage from earlier transactions.
- Query spent notes using DeFMI's current note-nullifier implementation.
  Do not confuse the one-time ownership key with the spent-note nullifier.

The deployment README records the enrollment trust boundary, the persisted
signature cache, and the explicit old-state migration boundary.

## Verification

All Rust builds, checks and test runs were performed on the authorized remote
host, not on the Mac.

| Gate | Result |
| --- | --- |
| Immutable-pin metadata and all-target/all-feature checks | All eight workspaces plus standalone batch-audit workspace passed |
| QOMM demo, venue proofs and venue transport tests | 138 passed; one explicitly native-only test ignored |
| OCLOB workspace tests | 148 passed; one explicitly native-only test ignored |
| QOMM real lifecycle | Four settled trades, two canonical no-fill refunds; all six queues finalized; no local execution fallback |
| QOMM restart | Participants, all seven MPC nodes, the five-validator ledger and gateway restarted; subsequent trade settled |
| Native market, finality and lifecycle | All three passed; recorded as smoke_only |
| Actual VM snapshot recovery | Two tests passed |

The ignored unit-suite native tests are not cited as executed. Real native
execution is evidenced separately by the QOMM and OCLOB cluster acceptances.

QOMM's official Dockerfile was built from the standalone QOMM checkout, immutable
dependency pins and `--locked`, without sibling-source overlays. The actual
MP-SPDZ engine was rebuilt from the pinned source with the existing hybrid
transport configuration.

## Receipts and limits

- [QOMM and test receipts](verification/QOMM_CRATE_SPLIT_ACCEPTANCE_2026-09-08.json)
- [Native acceptance, ninth_run](verification/PQC_INTEGRATION_ACCEPTANCE_2026-09-07.json)

The native contract IDs are `oclob-native-market-v2`,
`oclob-native-finality-v2` and `oclob-native-lifecycle-v2`; their exact hashes,
bound manifests, result hashes and append-only ledger receipt are in
`ninth_run`. The completed stage is
`RUN_ROUGH_END_TO_END_AND_OBSERVE_FINAL_METRIC`, with verdict `smoke_only`.
No functional scenario gate remains unresolved in this run. The contracts do
not authorize production promotion; independent-operator confirmation remains
outside these single-host smoke contracts.

Seven MPC nodes and five real Avalanche validators on one host are not seven
or five independent operators. External audit, physical HSM/FIPS validation,
independent-organization/WAN operation and post-quantum mathematical proofs
are not established. Enterprise browser/keychain CA enrollment requires the
user's authorization and is not silently installed.
