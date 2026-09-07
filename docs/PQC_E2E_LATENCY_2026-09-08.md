# Post-quantum migration: what it costs in wall-clock time on the deployed OCLOB path

Date: 2026-09-08. Companion to `PQC_PERFORMANCE_2026-09-07.md`, which measured
the primitives. This document asks the question the plan actually set
(`qomm/doc/ja/PQC_MIGRATION_PLAN.md`: throughput at least 95% of the classical
stack, added latency on the online path at most 5 ms at p95, 10 ms as the
ceiling under discussion): on the path a user of the public OCLOB demo exercises,
how much real time did the migration add, and where.

Method, as always: derive the mechanism and write the prediction first (section
2, committed before anything ran), run once (section 3), explain the gaps
(section 4).

## 1. What runs when an order is placed

`POST /api/order` on `oclob-demo` (`rust/oclob-demo/src/server.rs`,
`place_order` then `pump_role`) does, synchronously and under one mutex:

1. builds the secret order, its Ed25519 authorisation and the DeKYX
   possession evidence, and appends it to the encrypted durable queue;
2. pumps the queue: the node executor (`rust/oclob-node/src/executor.rs`,
   `execute`) writes the seven parties' inputs, then **spawns seven
   `malicious-shamir-party.x` processes** for this round and waits for all
   of them;
3. each MP-SPDZ party opens **two TLS sockets to every other party**
   (`Networking/CryptoPlayer.cpp`: `sockets[i]` and `other_sockets[i]`), so a
   round performs 7 × 6 = 42 TLS 1.3 handshakes, all with **mutual
   authentication** (`ssl_sockets.h` sets `verify_peer`), on the loopback
   interface;
4. the matching circuit runs, the parties agree on the public output, the node
   signs its execution receipt (Ed25519) and the settlement path issues its
   post-quantum response signatures and one-time keys;
5. the response carries the receipt, including `execution_ms`, the wall time
   of step 2 to 4 measured inside the executor.

The migration changed step 3 (TLS parameters) and step 4 (hybrid signatures
and keys). It did not change the process model: the parties were spawned per
round before the migration too, so the 42 handshakes were already paid on
every order. This matters for the comparison below.

### What each image is

| variant | image | TLS between parties | application signatures |
|---|---|---|---|
| classical (deployed until 2026-09-07) | `oclob-server:20260907` (oclob main 1a97383 with the reworked UI) | MP-SPDZ default: system OpenSSL 3.0.20, `Scripts/setup-ssl.sh` certificates, **RSA-2048 with SHA-256**, X25519 key share, TLS 1.3 | Ed25519 only |
| hybrid (deployed since 2026-09-07) | `oclob-server:main-20260907` (oclob main 1a4af96) | OpenSSL 3.5.5 in `/opt/pqc-openssl`, `require-hybrid-tls.patch`: TLS 1.3 only, group list `X25519MLKEM768` only, signature algorithms `mldsa65` only, self-signed **ML-DSA-65** party certificates, `verify_peer` | Ed25519 + ML-DSA-65 hybrid, ML-KEM-768 recipient envelopes, ML-DSA one-time keys |

The two images are the ones that ran on the public port, copied byte for byte
(`docker save | docker load`) to the build host, which is otherwise idle
(64 vCPU, load under 2 when the run started). The Omen box that serves the
demo carries a load average above 20 from unrelated processes and is not a
measurement host; a number taken there would measure the neighbours.

Confounder, stated up front: the classical image is the old main, so the diff
between the two images is the whole integration branch, not only the
cryptography. Anything the run attributes to "the migration" is an upper bound
on the cryptographic cost.

## 2. Prediction

### 2.1 One mutual TLS 1.3 handshake, loopback, one core each side

Compute per handshake, from the primitive figures (OpenSSL's C
implementations; the Rust bench measured RustCrypto, which is slower for
ML-DSA and about equal for ML-KEM):

| step | classical RSA-2048 + X25519 | hybrid ML-DSA-65 + X25519MLKEM768 | Ed25519 + X25519 (best classical, not what was deployed) |
|---|---:|---:|---:|
| signing of CertificateVerify, both sides | 2 × ~1.0 ms | 2 × ~0.25 ms | 2 × ~0.02 ms |
| verification of the peer certificate and CertificateVerify, both sides | 4 × ~0.03 ms | 4 × ~0.09 ms | 4 × ~0.05 ms |
| key exchange (keygen, encapsulate, decapsulate) | 3 × ~0.05 ms | X25519 part 3 × 0.05 ms plus ML-KEM 3 × ~0.03 ms | 3 × ~0.05 ms |
| **predicted compute per handshake** | **~2.3 ms** | **~1.1 ms** | **~0.4 ms** |
| bytes on the wire (two certificates, two CertificateVerify, key shares) | ~3 KB | ~20 KB (two ~5.5 KB certificates, two 3,309 B signatures, 1,216 + 1,120 B key shares) | ~1.2 KB |

So against what was actually deployed, the hybrid handshake is predicted to be
**about 1 ms faster**, because RSA-2048 signing is slower than ML-DSA-65
signing. Against the best classical option (Ed25519 certificates) it is
predicted to be about 0.7 ms slower. With `openssl s_time` on one client and
one server thread the predicted rates are roughly 400 to 500 handshakes/s
(RSA), 800 to 1,000 (hybrid), 2,000 to 3,000 (Ed25519). Server-authentication
only removes one signature and two verifications per handshake; the ordering
does not change.

### 2.2 One MPC round: 42 handshakes

Each party performs 12 handshakes, largely sequentially, while the seven
processes run in parallel on separate cores. The wall time of the handshake
phase is therefore about 12 × (one side's share):

| variant | per party | expected effect on `execution_ms` |
|---|---:|---:|
| classical RSA | 12 × ~1.15 ms ≈ 14 ms | baseline |
| hybrid | 12 × ~0.55 ms ≈ 7 ms | **about 7 ms less** than the baseline |
| Ed25519 (hypothetical) | 12 × ~0.2 ms ≈ 2.5 ms | 12 ms less than the baseline |

### 2.3 Application-side hybrid operations per fill

From the acceptance record (twelve new post-quantum keys and six response
signatures for three fills) and the measured primitives (238 µs keygen,
359 µs sign, 174 µs verify, 88 µs encapsulate): about 4 keygens, 2 signatures
and a handful of verifications and envelopes per fill, **2 to 8 ms** added on
the taker leg, less on the maker leg (no fill, no settlement).

### 2.4 The round itself and the resulting prediction

The round is dominated by spawning seven processes, loading the compiled
program and running malicious Shamir over a 253-bit field on the loopback. I
predict `execution_ms` between 500 ms and 3,000 ms per pump, with
round-to-round scatter of the order of 100 ms.

Net prediction for the difference hybrid minus classical, per order leg:
**between −10 ms and +5 ms**, that is, not resolvable against the scatter
of a multi-second round at 20 samples. The plan's 5 ms p95 target is
therefore predicted to be met on this path, with the honest footnote that
part of the reason is the RSA baseline the old image used. The cost the
migration does impose on this path is measurable only in isolation, which
is why section 3 measures the handshake separately, and it appears in
bytes, not time.

What would falsify this: a difference of more than 50 ms per leg in either
direction that persists after the warm-up rounds. If it appears, the cause is
in the non-cryptographic diff between the images and section 4 must find it.

## 3. Measurement

(filled in after the run)

## 4. Prediction against measurement

(filled in after the run)
