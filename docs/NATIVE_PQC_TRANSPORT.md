# Native MP-SPDZ hybrid transport

This integration adds a small TLS configuration adapter to the existing MP-SPDZ
engine. OpenSSL supplies the TLS, ML-DSA and KEM implementations. The adapter
requires TLS 1.3 and X25519MLKEM768 on both connection ends, sets the local and
client handshake-signature allowlists to `mldsa65`, requires a peer certificate,
and verifies that every certificate in the validated chain has an ML-DSA-65
public key and ML-DSA-65 signature OID. It retains CA and hostname verification,
disables session tickets and caching, and fails when any required policy cannot
be configured or verified. TLS certificate authentication is ML-DSA-65 alone;
it is not a composite TLS signature. Separately, the surrounding application
services apply pinned certificate-fingerprint ACLs, and hybrid application
signatures bind the financial and execution statements carried over the
channel.

## Exact upstream inputs

| Input | Version or commit | SHA-256 |
| --- | --- | --- |
| MP-SPDZ ssl_sockets.h | 9d809599ea6ce627216a389ca7d984fbb75d0cb9 | 640a71dd4cf012c8864010bf9690b2a6a8cd758807c90650aa03cd0599054832 |
| MP-SPDZ License.txt | same commit | 058bfbbd7da65ab998620441cddc59caf7593e92caa8a0d9a5c06b1f85356b06 |
| OpenSSL source archive | openssl-3.5.5 | b28c91532a8b65a1f983b4c28b7488174e4a01008e29ce8e69bd789f28bc2a89 |
| Patched ssl_sockets.h | accepted adapter | ccc7d6a496423ead58a3fc44f66a17f7fc692928395821bc546add38cefe58aa |

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

Run `scripts/pqc-native-build.sh` only on the authorized remote host through its
existing SSH wrapper. The dedicated acceptance runner verifies the native base
image ID before building synchronized source:

`sha256:91d284e9e0f0ba3fed0a005997edff224182eb949a465f54a6c287c249e5c3a4`.

Invoke `scripts/pqc-native-acceptance.sh` from this repository to create a
unique task root, source manifest, protocol receipt, image labels, image receipt,
and scenario artifacts. It uses the shared `remote-run.lock`, stores all runtime
state below the remote project's `.cache/native-runs`, and never uses host
`/tmp`. For accepted run
`20260906t110145z-44291-pqc-native-acceptance`, the synchronized-source images
were:

| Image | Tag | Image ID and repository digest |
| --- | --- | --- |
| OCLOB cluster | `oclob-pqc-native-cluster:20260906t110145z-44291-pqc-native-acceptance` | `sha256:a23c98e6dd2cfbd079235ab869e0528b7ae0effe0569f125176a3a5d87ca87c1` |
| DeFMI Avalanche VM | `oclob-pqc-native-avalanche:20260906t110145z-44291-pqc-native-acceptance` | `sha256:c249b407e4d63dc76f85ea809dc16438b7496152294dd75e51ec511f1a6b8814` |

Both images label the run ID, source-manifest SHA-256, and verified base image
ID. The runner compares source, protocol, and image receipts before and after
execution and fails closed on any change.

The build writes .pqc-tls.sha256 over the patched header, libSPDZ.so and
malicious-shamir-party.x. The Rust build and service startup check the receipt
against the fixed adapter hash and actual artifacts. This is local build
integrity evidence, not an attestation from independent operators. It does not
protect a host whose operator can replace both artifacts and its receipt.

`PQC_NATIVE_ENGINE=1` selects the native image in `scripts/pqc-remote.sh`. The
dedicated runner builds the OCLOB cluster and DeFMI VM from the synchronized
sources, copies the compiled MP-SPDZ tree into the final cluster image, and
compares its compiled receipt across the cluster and VM images. The engine and
all generated circuits live in a private writable task directory. Lab
certificate ownership stays within that task.

The configured runtime library path selects OpenSSL 3.5.5. QOMM resident
execution preserves that library path when clearing unrelated environment
variables. A native engine without the required receipt is refused, including
an explicitly configured classical checkout; an absent MP_SPDZ_ROOT remains
the explicit library-only build mode.

## Accepted execution evidence

Run `20260906t110145z-44291-pqc-native-acceptance` completed from
2026-09-06T11:02:02Z through 11:17:09Z. Its 1,119-entry local and remote source
manifests were byte-identical before and after execution; the manifest-file SHA
is `3460765104640f2efd0d6e0be693b1a87cf5fd83c5c14a2347982caa0e51f51f`.
This document and the consolidated receipt were written after the successful run
and are therefore not bytes covered by that run manifest.
The MP-SPDZ program source SHA is
`809eced81d8ffd8369d33f532c1dbab5171c6b7a4aa7a8f5a17fc3f576073121`,
and the compiled receipt SHA is
`d3a44bb5444415a45d398323c50e0325b8d80537b5a3b638ddd08d00accc4e51`.

The market scenario executed three rounds and two fills with seven MPC nodes,
then survived the canonical exit-75 crash, restart, and response-loss recovery.
The finality scenario proved exact private-state receipt replay and restart. The
lifecycle scenario executed three fills, cancellation and expiry releases,
participant-owned claim custody/redemption, and recovery. Each scenario reached
five equal validator roots before and after its required restart. The separate
P6 recovery/retirement gate passed 2/2.

This is `smoke_only` evidence from one host. It is not a seven-operator WAN
deployment, a latency or throughput comparison, independent-operator evidence,
or proof that FROST, Pedersen, anonymous credentials, or existing zero-knowledge
relations are post-quantum. See the
[consolidated report](verification/P1_INTEGRATION_REPORT.md) and
[machine-readable receipt](verification/PQC_INTEGRATION_ACCEPTANCE_2026-09-06.json)
for the exact scenario and hash boundaries.
