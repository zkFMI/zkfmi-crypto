# P0 cryptographic boundaries

## Implementation scope

The implementation pins `ed25519-dalek =2.2.0`, `ml-dsa =0.1.1`, `ml-kem =0.3.2`,
and `x25519-dalek =2.0.1`. The resolved curve dependency is `curve25519-dalek 4.1.3`.
`Cargo.lock` also pins transitive dependencies. No cryptographic core is implemented
from scratch: this crate supplies purpose-specific byte APIs, key metadata,
canonicalization, and the specified hybrid compositions.

Registered operations are signing and verification with Ed25519, ML-DSA-65, and
Ed25519+ML-DSA-65, plus encapsulation and decapsulation with ML-KEM-768 and
X25519+ML-KEM-768. SuiteIds for FROST, Pedersen, Bulletproofs, ML-DSA-44, SLH-DSA,
classical TLS, and hashes identify algorithms only. Unregistered cryptographic
operations fail with `UnsupportedSuite`. Existing services are not yet integrated.

## Backend selection and provenance

RustCrypto ml-dsa 0.1.1 and ml-kem 0.3.2 are the initial P0 backends. They are
pure Rust, require no C toolchain, use Apache-2.0 OR MIT licenses, and declare
MSRV 1.85. The crate's own traits use bytes so a future backend can use aws-lc-rs.
This selection does not imply production adoption, an external audit, or FIPS
validation. Both upstream crates state that they have not been independently
audited. Adoption of aws-lc-rs remains a separate decision.

| Upstream | Published package VCS commit | crates.io tarball SHA-256 |
| --- | --- | --- |
| [RustCrypto/signatures ml-dsa](https://github.com/RustCrypto/signatures/tree/f75d5b829948988f18d9463f286805fb9410bcdd/ml-dsa) | `f75d5b829948988f18d9463f286805fb9410bcdd` | `add6b9d92e496f16f4526d68ff29da1483aba4b119baeab8bed3b9e3544a6f3d` |
| [RustCrypto/KEMs ml-kem](https://github.com/RustCrypto/KEMs/tree/440768245bba59784b504269cb3087a6c21af45c/ml-kem) | `440768245bba59784b504269cb3087a6c21af45c` | `5e15f3e5b957493873e396a66914e83e616b6afe335cdef7efe5c6e1216aba66` |

These implementations are consumed through Cargo dependencies, without copying
or modifying upstream code. Classical backends use rand_core 0.6; PQ backends
use separate OS randomness acquisition or rand_core 0.10. The Ed25519 backend's
signature 2 and the ML-DSA backend's signature 3 are not treated as a common trait.
The dependencies' zeroize features are enabled for secret keys. Owned seeds and
shared secrets are held in `Zeroizing`. Callers importing seeds must erase their
own original buffers. No persistence serde is provided for secret keys, Signers,
or shared secrets.

## Signatures and canonicalization

`SigningPreimage` begins with the fixed domain `ZKFMI:CANONICAL:v1`. Variable-length
values use a u32 byte-length prefix; arrays use a u32 element count; integers use
fixed-width big-endian encoding. Strings are exact UTF-8 byte sequences, with no
implicit Unicode normalization or sorting. `object_ids` follow the order defined
by the protocol. The body hash is a fixed 32-byte SHA-256 value; the calling
protocol defines canonicalization of the body itself.

The signature context is `ZKFMI:SIGNATURE:v1 || KeyPurpose(u16) || Suite(u16,u16)`.
ML-DSA receives this as its FIPS 204 context. Ed25519 signs a separate
`ZKFMI:ED25519-CONTEXT:v1` domain followed by length-prefixed context and message.
Both hybrid components bind the hybrid suite, preventing reuse of standalone
signatures. Acceptance requires both verifications to succeed. A failed, missing,
or swapped component causes rejection.

## KEM composition

The public key consists of the 32-byte X25519 key and the 1,184-byte ML-KEM key.
The ciphertext concatenates the sender's 32-byte ephemeral X25519 public key and
the 1,088-byte ML-KEM ciphertext in a fixed order. HKDF-SHA256 uses
`ZKFMI:HYBRID-KEM:v1` as salt and `ZKFMI:HYBRID-KEM:SESSION:v1` as info, producing
32 output bytes. The input keying material is
`ss_x25519 || ss_mlkem || ct_x25519 || ct_mlkem || suite_id || suite_version`.
Both suite_id and suite_version are u16 big-endian values. Tests check that
changing either component secret or ciphertext changes the output. This does not
replace a formal security proof.

Noncontributory X25519 shared values and invalid lengths are rejected. ML-KEM
follows FIPS 203 implicit rejection: a tampered ciphertext of the correct length
produces a different shared secret. There is no classical-only fallback. This KEM
does not itself authenticate the peer or confirm possession of the resulting key.
It does not implement a TLS handshake or P1 configuration changes.

## Key-management responsibilities

Participant IDs are independent of public keys and expose no mutation API. A key
is valid during `[not_before, not_after)` and revoked from `revoked_at` onward.
Timestamps use Unix seconds. `key_version` identifies successive key generations
starting at 1; use of a generation that differs from the known generation is
rejected. Protocol, suite, and rotation-proof versions are separate closed types
that currently accept V1 only.

Initial registration and revocation must be invoked as authorized administrative
operations. A DeKYX binding is an opaque reference; this crate does not establish
the validity of the underlying qualification. The caller manages the clock,
expected network/deployment/contract/protocol, nonce-reuse checks, and business
authorization for registration.

Rotation requires both old and new keys to sign the transition, including the
participant, key IDs, suite, generation, purpose, public key, validity interval,
and DeKYX reference. Approval and acknowledgement use separate domains; the
acknowledgement also binds the old key ID. State changes only after both
directions verify, and successful rotation revokes the old key. Changes of
participant or purpose, skipped generations, reuse of the same key, and rollback
from PQ to classical-only cryptography are rejected. Rotation proofs apply to
signing-capable keys. The P1 integration layer defines authorization for KEM key
rotation. Transport purpose includes signing keys used for communication
authentication. KEM keys are restricted to Transport, but authentication signing
keys such as Ed25519 are also permitted for Transport.

Public DTOs support serde and reject unknown fields. `RegistrySnapshot` provides
a storage representation for public records; deserialization does not
automatically turn it into a trusted registry. This crate does not replace the
authentication and persistence responsibilities of the existing `PublicManifest`
or `EncryptedKeyStore`.

The `post_quantum` metadata flag classifies an algorithm; it does not certify an
implementation or assert security for every use. Reduced hash security margins
under Grover and related algorithms are considered separately. Pedersen's
perfect hiding and its binding property, which quantum attacks can break, are
also treated as distinct guarantees.
