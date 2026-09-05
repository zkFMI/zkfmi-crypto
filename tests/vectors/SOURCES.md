# Independent known-answer vector sources

Retrieved on 2026-09-05. The required public test data were extracted from NIST
and RFC originals, not reused from tests bundled with the cryptographic crates.
These seeds and decapsulation keys are public test values, not operational keys.
Using them does not imply ACVP service certification or FIPS implementation validation.

## NIST ACVP

Repository: [usnistgov/ACVP-Server](https://github.com/usnistgov/ACVP-Server)

Pinned commit: `975de31eb83d87039ec88934fdc47d8c312b892d`

| Local file | Original | Selected cases |
| --- | --- | --- |
| `ml-dsa-65-sigver.json` | [ML-DSA-sigVer-FIPS204/internalProjection.json](https://github.com/usnistgov/ACVP-Server/blob/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files/ML-DSA-sigVer-FIPS204/internalProjection.json) | tgId 3, external/pure, tcId 31/33/42/43. Two valid and two tampered cases, including a 255-byte context. |
| `ml-kem-768-encapdecap.json` | [ML-KEM-encapDecap-FIPS203/internalProjection.json](https://github.com/usnistgov/ACVP-Server/blob/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files/ML-KEM-encapDecap-FIPS203/internalProjection.json) | Encapsulation tcId 26/27 from tgId 2 and decapsulation tcId 86/88 from tgId 5. Case 88 exercises implicit rejection of a tampered ciphertext. |

Modifications: on 2026-09-05, the cases above were extracted for the specified
parameter sets, and fields unnecessary for verification were removed.
Cryptographic inputs and expected outputs were not changed. Cases without `m`
or `reason` in the original use null in the extracted JSON. The source is the
US National Institute of Standards and Technology. The original README's full
permission and disclaimer notice is retained in [NIST-NOTICE.md](NIST-NOTICE.md).

SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| Original ML-DSA source | `47cdd6314c7f746d02421ffcba89d4dbc7bb875ac49e07a029fdfc26fba55437` |
| Original ML-KEM source | `a556952ce869bb89c3a3196a701dad89647c193a34c86eafb61a9d710d5b810f` |
| Extracted ML-DSA JSON | `4f189e858d6c5959a41296fd1c49c5395a15bef49dd4625e548c36921efb4c21` |
| Extracted ML-KEM JSON | `23497b3f39b4cd83b3a50c458b21abff3adea309563dacb1d1d1c616cbd32b2b` |

## Ed25519

Original: [RFC 8032 section 7.1, TEST 1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1).
The local file is `rfc8032-ed25519-1.json`. It retains the public key, signature,
and seed for an empty message and checks both generation equality and verification.
These RFC inputs have no purpose context and must not be confused with P0's
purpose-specific signatures. Verification uses the same raw Ed25519 verification
path inside the P0 backend.
