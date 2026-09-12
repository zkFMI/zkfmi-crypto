# coCode feasibility trial - blocked, not a proof backend

The user-selected target remains an independent implementation of
[Code-based Scalable Collaborative SNARKs](https://eprint.iacr.org/2026/729)
composed with malicious-secure MPC, preserving seven parties, at most two
corrupted parties, no coordinator witness reconstruction, and a fully
post-quantum proof path. The existing release has not been modified by this
trial. This standalone crate is not registered as a release backend.

## What actually ran

An exact finite-field necessary-completeness check on SoftBank L40S, using
seven **public fixture rows**, eight coefficients per row, and 32 actual FFT
evaluation points per row. It executes aggregation, the sumcheck equality,
Reed-Solomon folding, bounded decoding, and the final evaluation equality.
It does **not** execute a complete coPCS, financial R1CS, secret-sharing
protocol, malicious MPC, Merkle/Fiat-Shamir compiler, or quantum security gate.
The small fixture is not a secure parameter set. Seven rows are not seven
independent operators. None of the seven declared full-proof tamper cases ran.

Observed first-run result: the initial sumcheck identity passed; all seven
literal folds satisfied the degree bound and the Appendix B formula; the
folded-oracle evaluation did not match the sumcheck claim. The normalized
arithmetic control matched, including its zero/one endpoint controls.
The contract's full-candidate prediction was 1, but its primary metric remains
**unobserved**, not 0 or 1. The receipt verdict is `blocked`.

## Exact boundary found

Appendix B (printed page 40) folds a row `p(Y) = E(Y^2) + Y O(Y^2)` into
`E(Y) + r O(Y)`. Construction 4 (printed page 22, steps 1c/1f/3) uses the same
`r` to restrict the multilinear extension of the coefficient vector. Under
the ordinary Boolean-table MLE convention, that restriction instead gives
`(1-r) E(Y) + r O(Y)`. For example, a public coefficient pair `(2,3)` at
`r=5` yields 17 under the Appendix B fold and 7 under MLE restriction.

The probe transcribes this interpretation independently. The normalized
control changes the even-part multiplier to `1-r`; it does not silently
change the selected production protocol. PDF pages 22 and 40 were visually
checked, and Appendix E's completeness argument was inspected. This is an
implementation/specification question, **not a demonstrated break of the
paper or the authors' implementation**. Resolving the convention or
normalization still requires carrying it through the complete folding,
soundness, masking, and transcript definitions. A successful arithmetic
control is not a malicious-security or zero-knowledge proof.

## Reproduction and evidence

Builds run remotely only. Pinned build image:
`sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97`.

Run `cargo run --release --locked` inside a **fresh isolated copy** of this
directory on the allowed remote host. The executable verifies the SHA-bound
contract and closed manifest, atomically claims the run, and immediately
executes the one registered probe. It refuses a second run in the same
artifact directory. Do not delete `run.claim` to bypass this guard.

The generated `artifacts/observation.json`, `artifacts/receipt.json`, and
`artifacts/ledger.jsonl` contain only public fixtures and metadata. The receipt
binds the exact executed source and Cargo lock bytes; the ledger records one
blocked result. Deterministic arithmetic unit tests are verification, not
additional empirical candidate experiments.

Contract: `zkfmi-private-pq-cosnark-trial-v1`.
SHA-256: `45dc526721b4f174c794557d15f1bc171a8bf4ef41fe210e5901bdd5402e26fb`.
Manifest: `experiment.json`, experiment `cosnark-rough-001`.
Earliest unresolved gate: fold-to-MLE binding, then the full financial-R1CS
private proving flow with seven-party malicious MPC and negative tests.

## Dependency provenance

Arithmetic uses arkworks 0.5.0 (`ark-ff`, `ark-poly`, `ark-bn254` scalar-field
feature), licensed MIT/Apache-2.0. Sources:
<https://github.com/arkworks-rs/algebra> and
<https://github.com/arkworks-rs/curves>. Cargo.lock pins registry packages and
checksums. Only the scalar **field** is used, not a curve commitment, pairing,
or discrete-log assumption. Transitive crate availability is not a claim that
its elliptic-curve algorithms ran. No unlicensed author proof-core code was
copied. This is a public arithmetic probe, not a completed independent SNARK.
