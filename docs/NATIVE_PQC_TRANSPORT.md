# Native MP-SPDZ hybrid transport

This integration candidate adds a small TLS configuration adapter to the existing
MP-SPDZ engine. OpenSSL supplies the TLS and KEM implementations. The adapter
requires TLS 1.3 and X25519MLKEM768 on both connection ends, disables session
tickets and caching, and fails when the group cannot be configured. Existing
certificate authentication remains classical.

## Exact upstream inputs

| Input | Version or commit | SHA-256 |
| --- | --- | --- |
| MP-SPDZ ssl_sockets.h | 9d809599ea6ce627216a389ca7d984fbb75d0cb9 | 640a71dd4cf012c8864010bf9690b2a6a8cd758807c90650aa03cd0599054832 |
| MP-SPDZ License.txt | same commit | 058bfbbd7da65ab998620441cddc59caf7593e92caa8a0d9a5c06b1f85356b06 |
| OpenSSL source archive | openssl-3.5.5 | b28c91532a8b65a1f983b4c28b7488174e4a01008e29ce8e69bd789f28bc2a89 |
| Patched ssl_sockets.h | local adapter | 7ad764ba4abf0aa7031ee99260f8ae6f42254ef68d32864a2f74c36191b2e1ff |

The exact [MP-SPDZ source](https://github.com/data61/MP-SPDZ/blob/9d809599ea6ce627216a389ca7d984fbb75d0cb9/Networking/ssl_sockets.h)
uses a TLS 1.2 context before the adapter.
The [upstream license](https://github.com/data61/MP-SPDZ/blob/9d809599ea6ce627216a389ca7d984fbb75d0cb9/License.txt)
is retained in qomm/rust/qomm-mpc/patches/MP-SPDZ-LICENSE.txt.
It contains the CSIRO BSD 3-Clause notice and the bundled third-party notices.
The adapted file is Networking/ssl_sockets.h; the local patch is
qomm/rust/qomm-mpc/patches/require-hybrid-tls.patch.
The [OpenSSL release](https://github.com/openssl/openssl/releases/tag/openssl-3.5.5)
provides the full archive and its checksum. Its Apache-2.0 license is retained
in the native image under /opt/pqc-openssl/LICENSE.txt.

## Reproduction and runtime boundary

Run scripts/pqc-native-build.sh only on the authorized remote host through its
existing SSH wrapper. The parent image is checked against
sha256:8513c0e1c3792f61bd8806d9ef05040c1649f6145a11e7d79126a54ca73bdc9d.
The resulting candidate image is
pqc-full-integration:rust-1.97.1-mpspdz-9d809599-openssl-3.5.5,
observed image ID sha256:aa83bce9de257fc59cb3edb989b78844adc95b25783e52177c60ee5f64df1261.

The build writes .pqc-tls.sha256 over the patched header, libSPDZ.so and
malicious-shamir-party.x. The Rust build and service startup check the receipt
against the fixed adapter hash and actual artifacts. This is local build
integrity evidence, not an attestation from independent operators. It does not
protect a host whose operator can replace both artifacts and its receipt.

PQC_NATIVE_ENGINE=1 selects this image in scripts/pqc-remote.sh. OCLOB invokes
that wrapper through make remote-test PQC_INTEGRATION=1. The runner copies the
engine into a private, writable directory inside the dedicated task area, so
the official compiler can write generated circuits without modifying any shared
checkout. Lab certificate ownership stays within that task.

The configured runtime library path selects OpenSSL 3.5.5. QOMM resident
execution preserves that library path when clearing unrelated environment
variables. A native engine without the required receipt is refused, including
an explicitly configured classical checkout; an absent MP_SPDZ_ROOT remains
the explicit library-only build mode.

## Evidence limits

The 20260905T152721Z-oclob verification compiled and linked the native engine,
passed six real node network tests, and checked the OCLOB workspace targets.
It did not execute the matching circuit. The explicit ignored hybrid_native
regression executes seven real native processes twice on one host. It passed in 20260905T155940Z-oclob (two successful matching rounds), followed
by warnings-denied, all-target OCLOB node/MPC Clippy. The upstream honest-majority
Shamir machine enables encrypted channels by default, and the tested launch
passes no --unencrypted option. Runtime readback resolves libssl.so.3 and
libcrypto.so.3 from /opt/pqc-openssl/lib64 with OpenSSL 3.5.5.

Neither result is a seven-operator WAN deployment, a live Avalanche validator
network, a performance comparison, or evidence that FROST, Pedersen, anonymous
credentials or existing zero-knowledge proofs are post-quantum.
