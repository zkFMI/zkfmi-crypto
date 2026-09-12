# v2 hybrid query-agreement adapter

Status: **research-only implementation of COCODE-SEC-001; not a security-gate pass**

Protocol identifier:

```text
cocode-defmi-private-dvp-research-v2-hybrid-query-agreement
```

The v2 adapter prevents a coordinator from obtaining Merkle openings from
honest owners under divergent final-row or Fiat-Shamir query views. It does not
prove the custom coPCS/Spartan construction, establish concrete QROM security,
provide independent operator custody, or promote the integration beyond
research use. Frozen v1/`003` proofs remain explicit legacy artifacts and are
not reinterpreted as v2.

## Trust and custody boundary

The public roster and its canonical SHA-512 pin are deployment inputs. They must
be established and delivered to every owner and the trusted launcher before an
untrusted coordinator can interact with a worker. Accepting a roster or pin
from the coordinator being constrained is not supported.

Each private key is generated and loaded only by its owner process. Requests
and responses contain public enrollments, digests, and signatures, never key
seeds. A real deployment additionally needs separate operator accounts/hosts,
protected storage, authenticated channels, and an external enrollment
authority.

The bundled `Native` launcher starts all seven workers under one host identity.
Even if it generates seven distinct keys, that is a **laboratory-only ceremony**:
the launcher/host controls the filesystem and is not evidence of isolated or
independent operator custody.

## Exact public roster schema

The roster is strict JSON (`deny_unknown_fields`) with exactly seven ordered
owners. `party` must equal the zero-based array position `0..6`; every public
key must be distinct and lowercase canonical hexadecimal.

```json
{
  "protocol": "cocode-defmi-private-dvp-research-v2-hybrid-query-agreement",
  "suite": "ed25519-ml-dsa-65-and-v1",
  "roster_id": "deployment-authority-assigned-roster-id",
  "owners": [
    { "party": 0, "public_key": "<3968 lowercase hex characters>" },
    { "party": 1, "public_key": "<3968 lowercase hex characters>" },
    { "party": 2, "public_key": "<3968 lowercase hex characters>" },
    { "party": 3, "public_key": "<3968 lowercase hex characters>" },
    { "party": 4, "public_key": "<3968 lowercase hex characters>" },
    { "party": 5, "public_key": "<3968 lowercase hex characters>" },
    { "party": 6, "public_key": "<3968 lowercase hex characters>" }
  ]
}
```

Each 1,984-byte public key is the exact concatenation produced by
`HybridSigner::public_key`: 32-byte Ed25519 followed by 1,952-byte ML-DSA-65.
Each approval signature is AND-verified and contains a 64-byte Ed25519
signature followed by a 3,309-byte ML-DSA-65 signature. This adapter uses the
repository's `HybridSigner` and `HybridVerifier`; it does not implement either
primitive itself.

`roster_id` must be 1–128 ASCII characters from letters, digits, `-`, `_`, `.`,
`:`, or `/`.

## Trusted enrollment and pinning CLI

Run key generation separately in each owner's private directory. The directory
must already be private (`0700`, or be freshly created by the command), and the
key file is created once with mode `0600`:

```sh
zkfmi-cosnark-trial integration-query-owner-keygen \
  /operator-private/cocode-query-v2/owner.key 0
```

The command fails if the key file already exists. It emits one public-only JSON
object suitable for the matching `owners` array position:

```json
{"party":0,"public_key":"<3968 lowercase hex characters>"}
```

After a trusted enrollment administrator has assembled and independently
checked all seven entries, compute the canonical roster digest:

```sh
zkfmi-cosnark-trial integration-query-roster-digest \
  /trusted-enrollment/query-roster-v2.json
```

The output is 128 lowercase hexadecimal characters. It is **not** the SHA-512
of the JSON file bytes: the canonical calculation domain-separates and encodes
the protocol, hybrid suite label and numeric suite ID, roster ID, ordered party
numbers, and decoded public-key bytes. JSON whitespace therefore does not
change the pin. The roster file and printed pin must reach runtime through an
authenticated deployment/configuration path independent of the coordinator.

## Frozen Rust API

The same-host research launcher constructor is:

```rust
Native::new_query_agreement(
    root: &Path,
    port: u16,
    resumed: bool,
    roster_path: &Path,
    expected_roster_sha512: &str,
    owner_key_paths: &[PathBuf],
) -> Result<Native, String>
```

It validates the strict roster and independently supplied canonical pin before
spawning any worker, and requires exactly seven key paths. Each worker then
independently rereads the roster and pin and refuses to enter its coordinator
request loop unless its owner-local key derives the public key pinned at its
own ordered roster position.

The internal worker command, normally launched through that constructor, is:

```text
integration-worker-v2 NATIVE_ROOT PARTY ROSTER_JSON EXPECTED_ROSTER_SHA512 OWNER_KEY
```

The root-owned same-host laboratory runner consumes a strict launch-config JSON:

```json
{
  "roster_path": "/trusted-launch/query-roster-v2.json",
  "roster_sha512": "<128 lowercase hex characters from the canonical digest CLI>",
  "owner_key_paths": [
    "/owner-0-private/cocode-query-v2/owner.key",
    "/owner-1-private/cocode-query-v2/owner.key",
    "/owner-2-private/cocode-query-v2/owner.key",
    "/owner-3-private/cocode-query-v2/owner.key",
    "/owner-4-private/cocode-query-v2/owner.key",
    "/owner-5-private/cocode-query-v2/owner.key",
    "/owner-6-private/cocode-query-v2/owner.key"
  ]
}
```

Its explicit research-run command is:

```text
integration-full-v2 PRIVATE_PARENT PUBLIC_OUTPUT MANIFEST CANONICAL_VERIFIER QUERY_CONFIG_JSON
```

It uses the separately versioned `integration-query-contract.json`; the manifest
must bind the canonical `trusted_query_roster_sha512` before the atomic run
claim. This command is an activation surface only. It must not be invoked until
the root task freezes a new contract/manifest and separately authorizes a v2
research run.

The verifier-only surface keeps versions explicit:

- v1: `verify_serialized(bytes, expected)` / `verifier::verify(...)`
- v2: `verify_serialized_query_agreement(bytes, expected, trusted_roster_sha512)` /
  `verifier::verify_query_agreement(...)`

The corresponding explicit file CLI is:

```text
integration-verify-v2 PROOF_JSON STATEMENT_JSON TRUSTED_ROSTER_SHA512
```

The v1 verifier rejects a v2 statement. The v2 verifier requires the trusted
roster pin and rejects v1 or a different pin. No default verifier silently
accepts both versions.

## Canonical policy and actual-network activation

The opt-in DeFMI research VM uses
`DeploymentPolicy::native_query_agreement(deployment_id, trusted_roster_sha512)`.
The canonical 128-character lowercase pin is part of the immutable policy and
its authorization digest. `State.apply` selects the explicit v2 verifier using
that policy pin, never a trusted pin chosen from proof bytes. Legacy policies
omit the optional pin field entirely and retain their prior serialization.
Missing, malformed, uppercase, changed or protocol-mismatched pins fail closed.

The canonical acceptance driver takes `--query-roster-sha512 PIN`; actual
network mode additionally takes `--live-config FILE`. The manifest-bound
research entry point is:

```text
integration-live-v2 NEW_OUTPUT MANIFEST NETWORK_RUNNER VM CANONICAL_DRIVER GENESIS_CONFIG
```

It uses `live-query-contract.json`, validates the completed native receipt and
both public proof hashes under the same roster, and launches a fresh network
only after the atomic run claim. The network runner receives the validated pin
from the parent entry point; legacy mode clears that environment input.
Postflight requires the expected v2 protocol and canonical roster pin as well
as five equal validator roots and actual pending-upload/final restart recovery.
Neither this live network nor its verifier needs the owner keys or witnesses.

For the same-host lab launcher, keep the enrollment/key mount read-only and
provide a narrowly scoped writable `query-agreement-v2` subdirectory for
durable markers. A wholly read-only key-parent mount correctly prevents
execution rather than silently disabling the query budget. This filesystem
arrangement is not independent custody or rollback-resistant storage.

## Two-phase release protocol

For every PCS kind (`0`, `2`, `4`, or `6`):

1. Each worker reconstructs the authorized transcript locally and checks its
   own committed roots, claim, masked sumcheck identity, fold challenge, own
   final row, all seven final-row encodings, and all 512 derived indices.
2. `PrepareQuery` computes one canonical digest binding the protocol and suite,
   trusted roster pin, complete statement and external session identity, all
   ordered roots, PCS kind, claim, mask claim, sumcheck response, aggregation
   and fold challenges, pre-row transcript digest, all seven ordered final rows,
   all ordered query indices, and post-query transcript digest.
3. Before signing, the owner atomically creates and `fsync`s a `.prepared`
   marker under its owner-local key directory. A partial or failed attempt burns
   that logical session/kind budget; recovery requires a newly authorized
   external operation/session.
4. The owner returns an Ed25519 + ML-DSA-65 AND-signature over that exact digest.
5. `QueryWithAgreement` accepts only seven ordered approvals whose party IDs,
   roster pins, digests, public keys, and both signature components all verify.
6. The owner atomically creates and `fsync`s its `.consumed` marker before any
   call to `Merkle::open`, then returns only the already-authorized openings.

Markers live below the parent directory of the owner key:

```text
<owner-key-directory>/query-agreement-v2/P<PARTY>/<session-kind-id>.prepared
<owner-key-directory>/query-agreement-v2/P<PARTY>/<session-kind-id>.consumed
```

The marker ID is tied to the roster, protocol, deployment ID, book ID,
operation ID, sequence, and PCS kind. It deliberately does not use mutable
commitment roots as the session key. Recreating the coordinator run root or
recommitting under the same externally assigned logical session therefore does
not reset the owner's budget.

Missing or reordered approvals, duplicate owners/keys, either invalid signature
component, a changed non-own row, changed roots/claim/challenges/indices, replay,
or process restart all fail before openings are returned.

## Deliberately unresolved

- COCODE-SEC-002: exact-construction knowledge-soundness, ZK, and composition
  proof.
- Concrete classical and QROM parameter/security calculation.
- Real seven-operator enrollment authority and independent key/share custody.
- Production-grade owner service, authenticated transport, HSM/keystore use,
  rollback-resistant state, backup/restore policy, and secure erasure.
- Root-owned v2 contract/manifest, canonical whole-MPC runner, and actual v2
  research experiment.

No v2 proof experiment or production deployment is authorized by this adapter
or document.
