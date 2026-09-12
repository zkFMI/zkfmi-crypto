# coCode / Spartan integration security-gate review

Date: 2026-09-09

Review mode: bounded, read-only source and evidence review

Decision: **FAIL — the cryptographic security gate remains red**

## Executive decision

The `cocode-defmi-integration-003` result is valid evidence that the frozen
research path executed end to end and rejected its declared negative cases. It
is not evidence of a formal zero-knowledge, knowledge-soundness, QROM, or
malicious-composition theorem. The recorded `smoke_only` verdict is therefore
the highest justified verdict today.

Continued opt-in research and additional smoke runs are reasonable if they keep
the same explicit limitations. Production promotion, a claim of 128-bit proof
security, use with real confidential value, or description of this artifact as
a proved coSNARK must be rejected.

Two independent reasons are sufficient to keep the gate red:

1. The implementation is a substantial, useful but unproved variant of the
   protocol in ePrint 2026/729: it uses one normalized fold, reveals and exactly
   decodes the complete terminal oracle, and composes a custom R1CS sumcheck and
   batched linear opening across four PCS openings. The paper's theorems do not
   directly instantiate this construction.
2. The worker authorization path does not establish the malicious-prover
   challenge-agreement premise required by the paper's collaborative BCS
   transformation. A coordinator can give different complete `final_rows`
   rosters to different honest workers. Each worker checks only its own row and
   can therefore derive and answer a different Fiat-Shamir query set.

The second item is a concrete protocol deviation, but this review did **not**
demonstrate witness extraction. The existing one-shot controls and large random
padding may bound the additional leakage in the exercised two-transition path.
That observation does not restore the missing theorem premise and is not a gate
pass.

## First blocking obligations

### COCODE-SEC-001 — authenticated all-party query-view agreement

In a new protocol version, before any worker releases a Merkle opening, all
seven workers must authenticate and agree on one digest that binds at least:

- the protocol-suite identifier and deployment/session identifier;
- the statement and complete ordered commitment-root roster;
- the PCS kind, claim, mask claim, sumcheck reply, and fold challenge;
- the complete ordered seven-row `final_rows` roster; and
- the resulting ordered query-index list.

Agreement must be among the workers, not an assertion supplied by the
coordinator. Any equivocation, missing participant, duplicate session, restart,
or digest mismatch must fail before an opening is returned, with durable
one-shot consumption. This must be implemented under a new version; frozen
`003` source, proofs, receipts, and historical identities must not be rewritten.

This is necessary but not sufficient for promotion.

### COCODE-SEC-002 — exact-construction proof obligation

Freeze a byte-level specification of the corrected protocol and prove, or map
line by line to an applicable proved construction, all of the following for the
exact bytes and adversary model:

- round-by-round knowledge soundness and extraction for the one-fold,
  fully-revealed-terminal PCS;
- `t`-zero knowledge for its original and masking tensor codes, including all
  public terminal rows and adaptive query answers;
- sound binding of the custom constraint sumcheck, endpoint identities, book
  claims, and final batched witness opening to the stated R1CS relation;
- sequential composition of the mask, prior-book, next-book, and witness
  openings, including the same book's permitted output/input reuse; and
- preservation of those properties by the corrected collaborative
  Fiat-Shamir/BCS transform against a malicious coordinator and up to two
  corrupted parties.

Tests, source hashes, honest executions, and a model review cannot discharge
this proof obligation.

## Scope and evidence identity

The primary reference was the current minor revision of
[ePrint 2026/729](https://eprint.iacr.org/2026/729), including its
[PDF](https://eprint.iacr.org/2026/729.pdf), inspected on 2026-09-09. The PDF
reviewed here has SHA-256
`aef6f55c549ffc006d46d7d1f5c68ec703abc3984e7e2940653110afda3f2802`.
The relevant paper material is Construction 2 and Theorem 1 (zero-knowledge
tensor encoding), Section 4 and Theorem 2 (collaborative BCS in the QROM),
Constructions 3 and 4 (coIOPP/coPCS), and Constructions 5 and 6 (coSumcheck and
Spartan composition).

The reviewed integration contract is
`zkfmi-cocode-defmi-integration-v1`, SHA-256
`44b45868cadcefa44340f42bc7199762d7d1f8a9ebb78e86496667b06804944f`.
Its final verification record explicitly reports `smoke_only` and excludes
formal ZK or quantum soundness, independent-operator isolation, and production
promotion (`research/cosnark-trial/artifacts/verification-integration-003.json:1-24,214-225`).

The following reviewed core hashes were rechecked on 2026-09-09 and match the
frozen `003` arithmetic/proof core identities:

| File | SHA-256 |
| --- | --- |
| `src/integration/circuit.rs` | `c08d009e254eb765aeab1dc28f2140dfae306dcf49456e2669c72739693cfe2c` |
| `src/integration/verifier.rs` | `50f4b594034884727c7bcaca441a48a1b9c71f1ff99412c6de1760ea33471410` |
| `src/integration/party.rs` | `09d8e0ac3b4a95ad6711366cf614369919223539ef2ee0379ba21a104889f18d` |
| `src/integration/prover.rs` | `77741340d94f35bf07e7df88f6e6552c6197ca0615dbd4f224e683d54953b5cf` |
| `src/integration/mpc_program.rs` | `b54d68658d76d53e2f88ec09ad09cd0add45d525c7efefe8e3beadecf130cccb` |
| `src/pcs.rs` | `be9b78c93225b5540f9798a00db56159a0170b1951220856abf709436dbcf7fa` |
| `src/algebra.rs` | `d7f3dfe255f66f955d3318de23a1b9dc5bf9288cc7a103a1969d8be3c5a75541` |
| `src/transcript.rs` | `bff883ff45a9367324a5815db994a06ed03a66a253c749eaef7ccac17465902f` |

This review did not run a new proof, mutate a receipt or snapshot, build a
binary, commit, push, or deploy.

## Findings

### 1. Gate blocker: no theorem currently covers the exact PCS and R1CS composition

Severity: **critical / promotion blocker**

The code describes its PCS as a "one-round, fully revealed final-oracle"
adapter and says the terminal reveal replaces an unimplemented recursive or
sparse-matrix verifier (`research/cosnark-trial/src/pcs.rs:1-2`). It commits to
length-65,536 codewords, performs one fold, reveals all seven length-32,768
terminal rows, exactly decodes each row to degree below 8,192, checks a
degree-two polynomial across all seven party rows, and makes 512 paired queries
back to the committed oracles (`src/pcs.rs:13-15,94-187`).

That may be a defensible code-based polynomial argument, but it is not the
paper's complete multi-round coIOPP/coPCS. Likewise, the integration performs
one 11-round custom constraint sumcheck (`src/integration/verifier.rs:60-99`),
then replaces the rest of the Spartan construction with randomized linear
book/witness binding and four sequential PCS openings
(`src/integration/verifier.rs:141-195`; `src/integration/prover.rs:206-251`).
The paper's coPCS and Spartan theorems therefore cannot be inherited by naming
the same ingredients.

No concrete accepting forgery was found. The failure is that the required
extractor, ZK simulator, and composition bound have not been supplied for what
the verifier actually accepts. An implementation error could consequently be
perfectly repeatable in honest tests while remaining outside the proved
security relation.

Promotion requirement: satisfy COCODE-SEC-002 for a frozen corrected protocol,
then obtain independent cryptographic review of the proof and its
implementation mapping.

### 2. Malicious coordinator can fork honest workers' Fiat-Shamir query views

Severity: **high / theorem-premise failure**

Section 4 of ePrint 2026/729 requires provers to broadcast and compare the
challenge they derive. It explains that omitting this step lets malicious
provers send inconsistent messages so honest provers answer different query
indexes, exposing more oracle values. This safeguard is part of the premises
used by Theorem 2.

The integration checks a common transcript view inside each native MPC program
(`src/integration/mpc_program.rs:17-23`), and each worker independently
reconstructs and verifies earlier proof context
(`src/integration/party.rs:56-89`). Those are useful checks, but they occur
before the final-row roster is supplied for the query phase.

In `Request::Query`, a worker requires seven rows but compares only
`final_rows[party]` to its own saved row. It then hashes every coordinator-
supplied row, derives 512 indices, and returns openings if those indices match
the same coordinator-supplied list (`src/integration/party.rs:298-350`). A
malicious coordinator can therefore:

1. collect the real row from each worker;
2. give each honest worker a different vector in the six non-own positions;
3. compute the corresponding query list for that worker; and
4. collect the union of the different valid query responses.

The final standalone proof contains only one roster and will not directly show
the extra private interactions. This is a privacy/composition concern, not just
a malformed-proof concern.

This review does not claim that the bounded fork recovers the witness. With
five honest workers, a single in-process request per worker, and 16,320 random
padding positions beyond the 64-position kind-2 message region, the current
honest-path padding may be sufficient for that bounded union. A proof must
establish the bound; it cannot assume the common-query premise that the code
does not enforce.

Promotion requirement: COCODE-SEC-001, an adversarial equivocation test that
covers every non-own row and restart boundary, and inclusion of the resulting
agreement protocol in COCODE-SEC-002.

### 3. No concrete QROM security level has been established

Severity: **high / PQ-security blocker**

The transcript is a custom streaming SHA-512 state. Field challenges reduce a
512-bit digest modulo the BN254 scalar field, while query sampling takes an
index in a 32,768-element set (`research/cosnark-trial/src/transcript.rs:14-58`).
The Merkle tree uses SHA-512 with independent 64-byte leaf salts
(`src/transcript.rs:61-69,94-127`). The proof uses `QUERIES = 512`
(`src/pcs.rs:13-15`).

Theorem 2 of the paper does not turn those constants into "128-bit PQ
security." Its classical Fiat-Shamir term depends on the adversary's random-
oracle query bound `Q`, the underlying round-by-round error, and a term of the
form `3(Q^2+1)/2^lambda`; its QROM result incurs an additional asymptotic loss
depending on `Q`. The ZK statement also includes the hiding error of the Merkle
commitment. All underlying errors must first be proved for the exact PCS, then
combined across four correlated openings, the two permitted views of a book,
and the full settlement lifecycle.

The paper's experimental choice of 520 queries and distance parameters for
approximately 100 bits is useful context, not a transferable estimate for this
different construction. The `program.set_security(128)` directive and runtime
`-S 128` selection (`src/integration/mpc_program.rs:17-23`;
`src/integration/native.rs:265-288`) configure the native MPC engine. They do
not set or prove the security level of the PCS, Fiat-Shamir transform, Merkle
commitment, composition, or whole system.

Promotion requirement: a reproducible parameter worksheet tied to the exact
protocol spec. It must state the target classical and quantum security levels,
field size, exact code distances and proximity bounds, round-by-round
soundness/knowledge errors, `Q`, hash and Merkle assumptions, multi-opening and
multi-session losses, corruption threshold, and all hidden constants. It must
fail closed if any input or bound is unknown.

### 4. The single-host coordinator boundary does not realize independent malicious-Shamir parties

Severity: **high for deployment; acknowledged laboratory limitation**

The selected MP-SPDZ `malicious-shamir-party.x` mode is an active-security,
honest-majority protocol, and `N=7`, `T=2` is within that threshold. Its theorem
still assumes distinct party state, authenticated encrypted channels, an agreed
computation, and the absence of an actor that can read every party's files.

The current driver accepts generated MPC source, writes it into the common
engine tree, compiles it, and launches all seven parties from the same root
(`research/cosnark-trial/src/integration/native.rs:181-205,220-298`). The
coordinator also creates/copies every owner's persistence and salt files in one
filesystem (`src/integration/run.rs:54-83`). File modes do not isolate processes
that share the same operating-system identity. A controller with that identity
can inspect all input contributions and shares or replace the computation. No
end-to-end privacy claim against that controller follows from malicious Shamir.

This is consistent with the recorded `single-host, same-UID` laboratory scope;
it does not invalidate the smoke result. It does block a claim that seven
independent operators or production custody have been exercised. The
[official MP-SPDZ repository](https://github.com/data61/MP-SPDZ) also states
that the software has not undergone the security review required for critical
production code and requires encrypted channels for these honest-majority
secret-sharing protocols.

Promotion requirement: independent operator custody; authenticated encrypted
party channels and identities; an immutable, independently approved
program/bytecode/schedule/field hash; no host, coordinator, or backup principal
able to read three party states; and a real seven-operator adversarial
acceptance run. Operational evidence remains additional to, not a substitute
for, the cryptographic proof.

### 5. Two-session book reuse currently assumes a static corruption set

Severity: **high if mobile/adaptive corruption is in scope; medium otherwise**

On resume, the exact previous kind-6 share vector is copied into kind 4
(`research/cosnark-trial/src/integration/mpc_program.rs:101-117`), and the
driver copies each owner's persistence plus the commitment salts needed to
recover the same root (`src/integration/run.rs:55-83`). There is no proactive
resharing between the output proof and the next-input proof.

For degree-two Shamir sharing, three distinct retained shares reconstruct the
secret. An adversary that corrupts two operators in one session and a different
operator in the next can therefore exceed the lifetime threshold if old shares
are retained. The two independent linear-claim blinds protect the two public
linear book claims; they do not refresh party custody shares.

The current claim is supportable only if `T=2` means one static corrupted set,
or at most two distinct parties cumulatively across the complete book lifetime,
with secure erasure assumptions stated explicitly. If mobile/adaptive
corruption is required, promotion needs a proactively secure refresh protocol
and a proof that refresh preserves the authoritative committed book relation.

### 6. The proof does not bind a complete protocol-suite identity

Severity: **medium**

The statement fixes a protocol string, party count, threshold, deployment
mode, and an R1CS fingerprint (`research/cosnark-trial/src/integration/circuit.rs:30-57`).
The fingerprint hashes the circuit rows (`src/integration/circuit.rs:102-115`),
but it does not cover the PCS dimensions, query count, fold definition,
transcript and serialization rules, Merkle construction, opening order,
two-use policy, native-engine identity, or verifier bytes. Those identities are
present in external research manifests and receipts, not in the proof's
cryptographic statement.

A later verifier could therefore retain the same protocol string while
changing security-relevant semantics. Promotion requires an immutable suite
identifier or digest covering the byte-level protocol specification and
verifier release, with explicit upgrade and downgrade rejection.

### 7. The current object is not succinct and should not inherit the paper's coSNARK label

Severity: **informational / claims control**

The terminal oracle is intentionally fully revealed (`research/cosnark-trial/src/pcs.rs:1-2`).
The final `003` proofs are approximately 192 MB each
(`research/cosnark-trial/artifacts/verification-integration-003.json:26-33,57-58`).
Verifier work includes decoding all seven terminal rows for every opening. This
does not satisfy the ordinary succinctness expectation of a SNARK.

Until COCODE-SEC-002 and the parameter gate pass, the accurate description is
"transparent code-based research argument/proof candidate using malicious MPC,"
not a production coSNARK and not a proved 128-bit post-quantum proof system.

## Security assumptions that must be explicit

Any future proof or security claim must identify, at minimum:

- the exact NP relation and extractor output accepted by the standalone
  verifier;
- a static or adaptive/mobile corruption model, including the coordinator,
  verifier, filesystem, backups, and cumulative corruptions across book reuse;
- no more than two actively corrupted parties out of seven under the selected
  malicious-Shamir theorem;
- independent, authenticated operator identities and confidential authenticated
  channels;
- identical independently approved MPC program, bytecode, schedule, scalar
  field, and runtime configuration at all parties;
- correct, independent `OsRng` entropy for Shamir input contributions, code
  padding, masks, Merkle salts, and book-claim blinds;
- a precise random-oracle/QROM model for the domain-separated SHA-512 transcript
  and a hiding/binding model for the salted SHA-512 Merkle tree;
- canonical statement, field, root, proof, and transcript encodings;
- a lifetime limit of exactly the proved number of openings and sessions, with
  durable fail-closed consumption and secure erasure; and
- correct binding by the external VM of the expected statement, sequence,
  book roots, protocol suite, and verifier release. Outer governance signatures
  authorize state transitions; they do not add mathematical proof soundness.

BN254 in this implementation is the arithmetic scalar field. The PCS does not
use a pairing or elliptic-curve hardness assumption. That removes one classical
assumption but does not by itself establish a concrete post-quantum security
level.

## Positive evidence retained

The following observations support continued research, although none closes
the gate:

- Transcript order binds the statement and root roster before derived
  challenges, and the prover locally runs the standalone PCS verification after
  each opening (`src/integration/verifier.rs:35-57`;
  `src/integration/prover.rs:37-104`).
- Field vectors are length checked and field elements are decoded canonically
  (`research/cosnark-trial/src/algebra.rs:103-115`).
- The PCS checks the folded coset, exact terminal degree bounds, a degree-two
  polynomial across all seven party rows, all 512 Merkle-query rosters, and
  disjoint query/message domains (`src/pcs.rs:120-187`).
- Initialization fills all eight 16,384-element message tables from distributed
  random contributions before overwriting relation data
  (`src/integration/mpc_program.rs:101-153`).
- The intended same-book lifecycle has two independent full-field linear
  challenges and two dedicated book-claim blinds, plus owner-local durable
  consumption markers (`src/integration/verifier.rs:132-138`;
  `src/integration/circuit.rs:147-155`;
  `src/integration/party.rs:328-341`).
- The frozen `003` evidence reports two accepted standalone proofs, 740
  canonical block applications, eight canonical negatives, four proof/native
  negatives, per-owner repeated-query rejection, and readback of frozen source
  and dependency identities. Those results support `smoke_only` exactly as
  recorded.

## Rejected candidate / not a bug

**Retracted claim:** "The kind-2 sumcheck-mask codeword contains only 34 random
values, so ordinary PCS queries recover its mask."

This is false and must not motivate an edit. Initialization first assigns
distributed randomness to all `8 * PADDED` state entries
(`research/cosnark-trial/src/integration/mpc_program.rs:107-108`). For kind 2,
the later code clears only the 64-position message prefix, restores the 34
intended mask values, and leaves positions 64 through 16,383 random
(`src/integration/mpc_program.rs:150-153`). Thus the kind-2 table retains 16,320
random padding positions beyond that prefix, in addition to the 34 intended
random mask values. The proposed sparse-mask reconstruction does not apply.

## Fail-closed promotion criteria

The security gate must remain red if any of the following is true:

- the corrected protocol lacks authenticated all-party final-row/challenge/query
  agreement;
- the accepted verifier bytes cannot be mapped to a frozen specification and
  the exact-construction proof;
- the proof omits extraction, terminal-oracle ZK, correlated four-opening
  composition, or the two-view book lifecycle;
- a concrete estimate omits `Q`, an underlying error term, a QROM loss, Merkle
  hiding/binding, or a union/composition loss;
- "128-bit" is inferred only from MP-SPDZ `-S 128`, SHA-512 output length, or
  the count of 512 PCS queries;
- any coordinator or host can read or rewrite three party states or choose an
  unapproved MPC program;
- corruption can move across operators without proactive refresh and secure
  erasure;
- the same book is queried beyond the proved two-view budget;
- a verifier accepts a proof under a different suite/parameter identity; or
- smoke tests, a five-validator/live-network run, signatures, governance, or a
  model review are presented as substitutes for the missing cryptographic
  theorem and independent review.

## Review limitation

This document is an independent model-assisted source review, not a formal
proof, exploit audit, side-channel review, or professional cryptographic
certification. Absence of a demonstrated attack is not evidence that the gate
can pass. Conversely, the red gate does not erase the reproducible research
value of the frozen `003` smoke result.
