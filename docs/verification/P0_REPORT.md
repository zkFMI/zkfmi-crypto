# Independent P0 Implementation and Acceptance Report

> **Translator and publication note — 2026-09-05:** This report preserves the original P0 acceptance-time account of invalid `gh` authentication and no repository creation or push at that time. Publication is now complete: [shukob/zkfmi-crypto](https://github.com/zkFMI/zkfmi-crypto) is public, its default branch is `main`, and its initial published implementation/documentation HEAD is `25e69708b978b7d1a79258c3318b720f0ead952e`. The user's later decisions to use English documentation and public visibility superseded the original Japanese-README and private-visibility instructions. The work-order file was a byte-for-byte copy of the attachment at the original S1 commit `7bf7a69bb45b0ce37cd7838518d498308a8f7dae`; its present version is a later, user-requested English translation, and the original remains in Git history.

The commissioned S1–S5 slices were completed in local Git. The cryptographic implementation is not yet wired into existing services.
On softbank-l40s, the official Rust Docker image successfully ran fmt, clippy,
45 tests, and inventory reconciliation against the committed implementation. Because GitHub authentication was invalid, remote repository creation and
push were not performed. This was the local-only completion path permitted by the work order.

## Commits

| Slice | Repository | Commit | Deliverable |
| --- | --- | --- | --- |
| S1 | zkfmi-crypto | `7bf7a69bb45b0ce37cd7838518d498308a8f7dae` | MIT, Rust workspace, Japanese README, complete work order, project-memory, remote-gate |
| S2 | zkfmi-crypto | `bd43f598a9a78b7f74d16eafb02e10be90d9c95c` | Suites, key management, canonicalization, byte-sequence traits, real backends, hybrid signatures/KEM |
| S3 | zkfmi-crypto | `7efd7ffa6eadc89495ed801813c4cb02fb088c6c` | 8 pinned NIST ACVP cases, 1 RFC 8032 case, sources and hashes |
| S2 supplement | zkfmi-crypto | `c6ece07210a7f0f85ae4e548bd296fc35e630a7e` | Allowed authentication signing keys for the Transport purpose and regression-tested their purpose separation from KEM keys |
| S4 | zkfmi-crypto | `3dfb937b966601e01e39602306f91ece609140b6` | Frozen inventory of 7 repositories, independent grep reconciliation, generated Markdown, count reconciliation |
| S5 | qomm | `61596e523a4249031ae2471c78bd2249fea8f49b` | Only the single permitted plan file, committed with the `docs:` prefix |

The implementation acceptance checks below were performed after synchronizing the clean working tree at the S4 commit above.
The same gate will also be run after the commit that adds this report and its acceptance evidence, and that final HEAD and
log will be included in the task's final response. HEAD values in earlier slice logs
are the starting points for those slices; distinguish them from logs that verified candidates that were uncommitted at the time.

## Execution environment and results

| Item | Measurement / evidence |
| --- | --- |
| Invocation | `RUN_INVENTORY_CHECK=1 make remote-gate` |
| SSH host / hostname | `softbank-l40s` / `ngi-external022-vm1` |
| Remote working path | `/home/ubuntu/work/zkfmi-crypto/` |
| Docker | Official `rust:1.97-bookworm` image, CPU limit 4 |
| Image digest | `sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97` |
| rustc | `1.97.1 (8bab26f4f 2026-07-14)` |
| cargo | `1.97.1 (c980f4866 2026-06-30)` |
| Verified commit | `3dfb937b966601e01e39602306f91ece609140b6`, `worktree_dirty=0` |
| Execution time | 2026-09-05 05:58:33–05:58:37 UTC (14:58:33–14:58:37 JST) |
| Synchronized input manifest SHA-256 | `b07d223ff64d342dc1adf4c54a98caab5c7ac69128c9960ee8b742abd396387c` |

Execution order and results:

1. `cargo fmt --all -- --check`: Passed.
2. `cargo clippy --all-targets -- -D warnings`: Passed.
3. `cargo test --release`: **45 passed, 0 failed, 0 ignored**.
4. `cargo tree -d`: Passed. `rand_core` 0.6.4 / 0.10.1, `signature` 2.2.0 / 3.0.0.
5. `scripts/inventory_check.sh`: Exit 0, with the exact agreement below.

```text
grep_files=507 inventory_files=507 primitive_entries=1032 missing=0 stale=0
source_hashes=825 markdown_from_json=PASS inventory_check=PASS
```

The full execution log is [final-code-gate.log](final-code-gate.log); the list and hashes of synchronized inputs are in
[final-code-input.sha256](final-code-input.sha256). The duration in the log is the time taken by a cached
verification gate, not a signature/KEM performance measurement or the initial build time.
`rust-version = "1.85"` is an MSRV declaration; execution with rustc 1.85 was not verified.

### Coverage of the 45 tests

| Category | Count | What was exercised |
| --- | ---: | --- |
| Backend known-answer tests | 9 | 4 NIST ML-DSA-65 cases, 4 ML-KEM-768 cases, 1 RFC 8032 Ed25519 case |
| canonical / suite / memory | 9 | 3 fixed-hex golden cases, FIPS sizes, purpose and reuse boundaries, length ambiguity, rejection of unknown fields/schemes/versions, project-memory format |
| hybrid / provider | 14 | 4 signature quadrants, missing/swapped/incorrect-length components, purpose and suite binding, real KEM round trips, replacement of either component's secret/ciphertext, rejection of non-contributory X25519, rejection of unregistered operations |
| Key lifecycle | 13 | Bidirectional rotation, atomicity, unknown key generations, revocation/expiration, purpose, rejection of migration from PQ back to classical-only, separation of Transport signing keys from KEM keys |

Cryptographic operations use the actual specified crates; the expected known-answer values were not
substituted with the upstream crates' built-in tests. The pinned NIST ACVP commit is
`975de31eb83d87039ec88934fdc47d8c312b892d`.
ML-DSA tcId values are 31/33/42/43, and ML-KEM tcId values are 26/27/86/88; the cases also cover
tampering rejection and implicit rejection. Original source URLs, SHA-256 hashes of the retrieved originals and JSON excerpts, and licenses are
stored in [SOURCES.md](../../tests/vectors/SOURCES.md).

Secret-key zeroization with zeroize, signature contexts, HKDF input order, time endpoints, and responsibility for authorizing administrative operations are
defined in [CRYPTO_CONTRACT.md](../CRYPTO_CONTRACT.md). Hybrid signatures are
accepted only when both components succeed. For tampered ML-KEM ciphertexts of the correct length,
the standard implicit rejection behavior returns a different secret; it does not switch to acceptance based only on classical cryptography.

The absence of the new canonical domain `ZKFMI:CANONICAL:v1` from the frozen Rust sources of the 7 repositories
was also verified remotely ([canonical-domain-check.log](canonical-domain-check.log)).

## Inventory and count discrepancies

For the frozen snapshot whose capture began at 2026-09-05T05:22:20Z, Rust files containing the specified cryptographic crate
identifiers were counted. The originals were accessed read-only. SHA-256 hashes of the 825
Rust/Cargo files actually captured, along with each repository's HEAD and dirty paths, were stored in
[source_manifest.json](../../inventory/source_manifest.json).
Because the snapshot includes working trees under concurrent modification, no claim is made that the originals' HEAD values remain the same afterward.

| Repository | Files using cryptography |
| --- | ---: |
| qomm | 201 |
| zkpi | 59 |
| defmi | 206 |
| oclob | 25 |
| dekyx | 3 |
| deccp | 4 |
| aethel | 9 |
| Total | 507 |

The 1,032 primitive-level records were saved as the authoritative JSON, and the Markdown was
generated from that JSON. owner denotes the responsible component in `repo/crate` form; no unverified names of individual owners
were invented. quantum_status uses the 3 specified categories, and migration phases range from P1 to P6.

Reconciliation with the work-order counts is recorded for all 22 crates in
[count_reconciliation.json](../../inventory/count_reconciliation.json).
The set used here includes tests, hash-only files, and existing examples/benches, among other files. For example,
qomm-transport has 58 files in the all-Rust/src+tests set, 42 in src, and 34 in the non-hash src set;
the last set matches the work-order count of 34. Similarly, qomm-defmi has 54 in all Rust, 46 in src+tests, 28 in src, and
25 in non-hash src, reproducing the work-order count of 25.

qomm-zkpi has 8 non-hash src files (13 in src+tests) against the work-order count of 7; oclob-mpc has
2 in src+tests against 1; and oclob-settlement has 5 in src+tests against 3. The causes of these 3 remaining discrepancies
cannot be established because the file lists used for the work order were not provided. The completeness check does not exclude files
to match the work order's aggregate counts; it verifies set equality with an independent grep over
the files actually captured, regeneration of primitive records, original-source hashes, and Markdown generated from JSON.

As a negative check of the checker itself, removing the record for the existing file
`aethel/crates/aethel/tests/end_to_end.rs` from a temporary copy produced
`grep_files=507 inventory_files=506 primitive_entries=1031 missing=1 stale=0` and
**exit 1, as expected**. That temporary copy has been deleted. The evidence is
[s4-negative.log](s4-negative.log). This is not included in the failure count for the 45 tests.

## Plan

The only change in qomm was to `/Users/shukob/Research/DeFMI/qomm/doc/ja/PQC_MIGRATION_PLAN.md`.
There were 43 added lines and 1 deleted line (replacement of 1 existing size-criterion line). Comparing the order and text of all existing headings
(levels 1–3) confirmed an exact match.
The commit's file list and `git diff --check` were also checked. The evidence is
[qomm-plan-check.log](qomm-plan-check.log).

The additions cover the unmet start condition and the rationale for proceeding with independent P0, P1 wiring after duplicated-crate consolidation, the candidate list and
the same selection rationale as decisions.jsonl, P1's OpenSSL conditions, and pending TLS authentication decisions.
16KB/32KB was revised to "to be reset after confirming the definition of the current package_bytes value of 57,971 B,"
and a table of fixed FIPS sizes and the arithmetic for 3 signatures was added. The current package/proof sizes are
values carried over from the work order, not new measurements made in this task.

## Changed paths

The new repository root is `/Users/shukob/Research/DeFMI/zkfmi-crypto/`.
The following paths are relative to that root; all were newly created files in this task.

| Path | Content |
| --- | --- |
| `.gitignore`, `Cargo.toml`, `Cargo.lock`, `LICENSE`, `Makefile`, `README.md` | Scaffold, pinned dependencies, MIT, documentation, remote gate |
| `.codex/project-memory/project.toml`, `facts.jsonl`, `decisions.jsonl`, `worklog.jsonl` | Boundaries, verified facts, selection decisions, slice records |
| `docs/orders/2026-09-05-p0-handoff.md` | Complete, byte-for-byte copy of the attached work order at the original S1 commit `7bf7a69bb45b0ce37cd7838518d498308a8f7dae`; now a later, user-requested English translation, with the original retained in Git history |
| `docs/CRYPTO_CONTRACT.md` | Specification, upstream sources, storage/authorization responsibilities, implementation limits |
| `src/lib.rs`, `error.rs`, `suite.rs`, `canonical.rs`, `key.rs`, `traits.rs`, `backend.rs`, `backend_tests.rs` | Cryptographic APIs, DTOs, lifecycle, real backends, known-answer tests |
| `src/hybrid/mod.rs`, `signature.rs`, `kem.rs` | Hybrid signatures/KEM |
| `src/bin/crypto-inventory.rs` | Rust inventory generation and checking |
| `tests/canonical_and_suite.rs`, `hybrid.rs`, `key_lifecycle.rs` | 36 integration tests |
| `tests/vectors/ml-dsa-65-sigver.json`, `ml-kem-768-encapdecap.json`, `rfc8032-ed25519-1.json`, `SOURCES.md`, `NIST-NOTICE.md` | Independent vectors, URL/commit/hash, original-source licenses |
| `scripts/remote-gate.sh`, `capture-inventory.sh`, `inventory_check.sh` | Isolated synchronization, read-only capture, independent grep reconciliation |
| `inventory/crypto_inventory.json`, `source_manifest.json`, `count_reconciliation.json`, `CRYPTO_INVENTORY.md` | Authoritative JSON, input evidence, count discrepancies, generated document |
| `docs/verification/s1-gate.log`, `s2-gate.log`, `s3-gate.log`, `s3-input.sha256`, `s4-gate.log`, `s4-input.sha256`, `s4-negative.log` | Verification evidence for each slice |
| `docs/verification/P0_REPORT.md`, `final-code-gate.log`, `final-code-input.sha256`, `qomm-plan-check.log` | This report and acceptance evidence for the committed state |
| `docs/verification/canonical-domain-check.log` | Confirmation that the canonical domain does not collide with frozen existing sources |

In addition, Git metadata, verification logs, captured snapshots, and dependency/toolchain/build caches were created in
`.git/`, the ignored `.artifacts/` and `.cache/` directories within the new repository, and the remote
`/home/ubuntu/work/zkfmi-crypto/` directory. No tracked files were deleted.
In qomm, only the single plan file above was edited, and Git metadata was updated as part of the commit.

**Writes in this task were limited to this new repository, the dedicated remote working area, the permitted
qomm plan, and its commit operation.** No files in defmi, oclob, zkpi, dekyx, deccp, aethel, or
zkfmi-site, and no existing Cargo.lock, pins, Dockerfile, or compose files were edited.
Other remote working areas and existing Docker images were not changed. No builds/tests on the local Mac,
Python implementation, or temporary-file creation in `/tmp` were performed.

## Work not performed or left pending

- GitHub creation/push: `gh auth status` at the start indicated invalid authentication. None of `gh auth login`, GitHub write APIs,
  repository creation, or push was executed. All local slices were committed.
- Adding dependencies to existing crates, consolidating common crates, P1 TLS changes, and P2 storage changes: explicitly out of scope.
- Performance benchmarks and finalization of the overall plan's P0 performance criteria: not included in the slices commissioned for this task.
  Cryptographic processing times and production performance were not measured. Completion of independent S1–S5 is not treated as completion of P0 in the overall migration plan.
- Actual operations for ML-DSA-44, SLH-DSA, FROST, proof systems, and similar schemes: this task provides only SuiteId and the required size definitions.
  Unregistered operations are rejected. Secret-key storage, authorization of KEM key rotation, and TLS key confirmation are the responsibility of the existing infrastructure/P1.
- External audit, production adoption, and FIPS certification: no claim is made that these were performed or obtained.

## Matters requiring a decision

- Repository visibility: The initial work order defaulted to private, but the additional instruction on 2026-09-05 had already decided on public visibility. At the time of this original P0 acceptance report, creation on GitHub and push remained unperformed because GitHub authentication was invalid.
- aws-lc-rs: P0 has adopted RustCrypto. Whether to adopt aws-lc-rs as a production candidate requires a separate decision.
- TLS authentication: The choice between an ML-DSA certificate alone and dual certificates has not been made. Decide before implementing P1.
