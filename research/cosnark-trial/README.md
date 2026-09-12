# Private coCode / malicious-MPC trial

Status: **full financial smoke executed; not a production proof backend**.

The subsequent opt-in [canonical DeFMI integration](INTEGRATION.md) binds a
larger 64-bit four-account relation to authoritative state and chunked
settlement/recovery. The remainder of this page describes the original
isolated `cosnark-rough-012` experiment, not that later integration.

The final source completed `cosnark-rough-012` on SoftBank L40S in
**25.933 seconds**: one freshly generated private financial witness, a
standalone accepted public proof, all seven declared negative cases rejected,
and all seven owner-local query-replay guards rejected reuse. The earlier
successful run, `cosnark-rough-011`, took 35.423 seconds. Both are
`smoke_only`, not statistical confirmation or cryptographic security proofs.
The final public proof is **96,175,154 bytes**.

The selected reference remains
[Code-based Scalable Collaborative SNARKs](https://eprint.iacr.org/2026/729).
This is an independently written, explicitly adapted research implementation,
not an execution of the authors' implementation and not a claim that the
published paper has been broken. No unlicensed author proof-core code was
copied. The existing release backend and application deployments are unchanged.

## What actually executes

- Seven per-party Rust workers generate their own OS-random input contributions
  and masking entropy. The coordinator has no original witness or raw
  persistence-share input/output API. Each worker reads only its own native
  persistence file and encodes its own committed rows.
- Actual pinned MP-SPDZ `malicious-shamir-party.x` processes perform the
  scalar private computation with `N=7`, `T=2`, security setting 128,
  checked reveals, and the exact BN254 scalar-field modulus. The field is
  used only for arithmetic: no curve commitment, pairing, discrete-log proof,
  or outer signature quorum provides proof soundness.
- The fixed 64-row R1CS checks private amount times price equals notional,
  notional plus remainder equals reserve 65535, 8-bit amount/price bounds,
  16-bit remainder bounds, and a separately bound constant witness column.
  Inputs are fresh synthetic distributed fixtures, not operational orders.
- Seven masked constraint-sumcheck rounds are followed by two code-based
  openings: the committed separable mask polynomial and the joint
  witness/padding-mask linear relation. The verifier performs real FFT-based
  degree checks, checks all seven rows against degree two in the sharing
  dimension, authenticates the Merkle queries, and checks final evaluations.
- Per-owner authorization recomputes the Fiat-Shamir transcript, binds the
  statement and commitment roots, and permits only the correct one-shot fold
  and transcript-derived query set. A second query request is rejected by
  every owner with the expected consumed-budget error.
- A separate verifier process reads only a public statement and proof.
  Additional verification passed in a network-disabled, read-only container
  with only the verifier executable and public proof directory mounted.
  No run-specific private inputs or persisted shares were mounted.

## Explicit adaptations and limits

The implementation uses the normalized row fold
`(1-r) E(Y) + r O(Y)` so that folding and ordinary coefficient-table MLE
restriction agree, including `r=0` and `r=1`. The original necessary-equation
probe remains in `fold_binding_probe.rs`, with its historical blocked receipt
under `artifacts/receipt.json` and executed source archive. That probe alone
was never a full proof result; its interpretation is not a demonstrated
vulnerability in the paper.

This research variant uses one fold and a fully revealed **masked** terminal
oracle, a 16,384-element message domain, a 65,536-point multiplicative-coset
codeword domain, and 512 queries per opening. The coset excludes the original
secret-message interpolation points. SHA-512 domains and random 64-byte leaf
salts bind commitments and transcripts. A joint commitment and an O(nnz)
public linear-functional verifier replace the optimized second
sumcheck/sparse-matrix machinery. This explains the large proof; no succinctness
or performance superiority is claimed.

The complete normalized/masking/Fiat-Shamir composition, zero knowledge,
quantum soundness and concrete security parameters still require independent
analysis. Passing tamper tests does not prove any of those properties.
The native MPC engine is malicious-secure by its selected protocol, but this
single-host, same-UID lab is **not** an independent-operator isolation or
malicious-controller deployment demonstration. Native program distribution
and the local controller remain trusted laboratory infrastructure.

This crate is not registered as a release backend. The original trial did not
integrate authoritative financial commitments or settlement/recovery. The
later [research integration](INTEGRATION.md) adds a bounded canonical
four-account book, settlement, no-fill conservation, replay protection and
persisted recovery. Application-wide on/off policy, operational asset migration
and full venue-specific refund integration remain outside that result. The
full PQC-switch objective remains incomplete.

## Negative evidence

The final receipt records rejection of all seven predeclared cases:

1. Changed public statement.
2. Changed commitment root.
3. Changed authenticated codeword opening.
4. Changed final evaluation claim.
5. Inconsistent challenge views supplied by two of the seven parties.
6. Invalid private notional witness.
7. Changed deployment mode or identifier.

Cases 5 and 6 execute actual seven-party native computations and require the
specific `challenge view mismatch` or `invalid financial witness` marker;
all seven processes exit 1 in each negative run. They are not successful
because of a compiler error or an unrelated process crash. The other five
mutate the real accepted proof. These targeted cases are smoke coverage,
not an exhaustive adversarial proof.

## Runtime and corrected adapter failures

The native image is pinned to
`sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4`,
with MP-SPDZ revision
`9d809599ea6ce627216a389ca7d984fbb75d0cb9`.
The existing PQ TLS build is retained. A supported
`NO_MIXED_CIRCUITS` build removes an unused 64-bit auxiliary mixed-circuit
type; it does not reduce native security 128 or change the malicious-Shamir
arithmetic core. Unsupported mixed-circuit operations fail closed.

The compiler uses `-M -F 64 -P <exact-prime>` and explicitly records security
128 in every schedule. Here `-F 64` describes usable integer bits, not field
size. The official compiler's pinned `gmpy2==2.2.1` wheel is installed in an
isolated compiler dependency directory.

Without `-M`, native mask state passed its pre-persistence check but failed
after reload. Enabling the official
[memory-order preservation option](https://mp-spdz.readthedocs.io/en/latest/troubleshooting.html#order-of-memory-instructions-not-preserved)
fixed that failure. Exact nonzero entropy guards remain before persistence
and after reload.

Materializing 16,384 public weights per reply exhausted the 32 GiB compiler
cap. Each actual mask-opening reply had only 44 nonzero coefficients.
The adapter now omits **only public exactly-zero products**, preserving the
identical dot product and every cryptographic dimension and query count.
Both failed runs and their genuine causal changes remain in the ledger.
No failed result was relabeled as successful.

## Verification and reproduction

All builds and native runs are remote-only on `softbank-l40s`.

```sh
cargo fmt --package zkfmi-cosnark-trial -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked
cargo run --release --locked -- full <fresh-engine> <new-output> experiment-012.json
cargo run --release --locked -- verify <public-proof.json> <public-statement.json>
```

Formatting, all-target Clippy with warnings denied, and **11 unit tests**
passed. Unit tests use public fixtures and are not substitutes for the actual
private native runs.

The full runner atomically validates the SHA-bound contract and closed
result-first manifest, claims a fresh output, runs the candidate, writes a
public receipt, and appends exactly one verdict. A fresh engine must contain
the pinned runtime, matching compiler, supported arithmetic-only binary and
library, and lab TLS fixtures, but **no previous private inputs, persistence
or owner logs**. Reproduction is a new preregistered run, not permission to
overwrite an old output. Never delete `run.claim` to bypass the guard.

Evidence:

- [Final manifest](experiment-012.json).
- [Final public proof](artifacts/rough-012/proof.json) and
  [public statement](artifacts/rough-012/statement.json).
- [Final receipt](artifacts/rough-012/receipt.json), including native program
  and bytecode hashes, all process exits and the one-shot owner guards.
- [Dependency identities](artifacts/rough-012-dependency-identities.json),
  binding 74 native/compiler/local-path dependency artifacts.
- [Final verification](artifacts/verification-012.json).
- [Append-only ledger](artifacts/ledger.jsonl), preserving all 12 outcomes.
- Earlier executed-source snapshots are retained under
  `artifacts/rough-NNN-executed-source/` where subsequent edits changed hashes.

Contract: `zkfmi-private-pq-cosnark-trial-v1`.
Contract SHA-256:
`45dc526721b4f174c794557d15f1bc171a8bf4ef41fe210e5901bdd5402e26fb`.
Final manifest SHA-256:
`a76065a4d26c8e9bdb564df4b3609b14a630f3ef0146e5536908bcaa2475c5ad`.
Final receipt SHA-256:
`54cf0735bd25cb529e065d0851e31514e811680dc0167617b5a402cd8b9522bf`.

The earliest unresolved gate is independent validation of the adapted
cryptographic composition and parameters, followed by operational adoption
beyond the later bounded DeFMI research integration. Neither smoke result
promotes the candidate to production.

## Dependency provenance

Arithmetic uses arkworks 0.5.0:
[algebra](https://github.com/arkworks-rs/algebra) and
[scalar field](https://github.com/arkworks-rs/curves), MIT/Apache-2.0.
Cargo.lock pins registry packages and checksums. The existing
`qomm-mpc` compiler/persistence/engine-policy adapters are reused through a
path dependency; their executed source hashes are recorded separately.
MP-SPDZ is the existing pinned official implementation; the trial writes the
financial/program adapter, not a replacement malicious-MPC core.
