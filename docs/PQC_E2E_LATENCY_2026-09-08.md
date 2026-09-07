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

Run 2026-09-07T20:15:29Z to 20:18:56Z UTC (2026-09-08 05:15 JST) on
`softbank-l40s` (64 vCPU x86-64, load 0.6 at start, 2.0 at end). Images by
identity: `oclob-server:main-20260907` sha256:dd56d4e0…, `oclob-server:20260907`
sha256:6ab5d588…, each started on a fresh volume with the same queue
passphrase. Scripts, raw records and the run log are in
`docs/verification/pqc-e2e-2026-09-08/`.

### 3.1 Order legs, 20 measured rounds per image after 2 warm-up rounds, alternating image every round

One round is a resting maker sell (good till cancelled, price 100, quantity 10)
followed by a taker buy (immediate or cancel, same price and quantity) that
fills it completely. Every measured round produced exactly one fill. Wall time
is the client-side duration of the `POST /api/order` request.

| leg | classical median | classical p95 | hybrid median | hybrid p95 | hybrid − classical, median | at p95 | relative, median |
|---|---:|---:|---:|---:|---:|---:|---:|
| maker (rests, no fill, no settlement) | 1,211.6 ms | 1,298.2 ms | 1,268.0 ms | 1,314.5 ms | **+56.4 ms** | +16.3 ms | +4.7 % |
| taker (fill, settlement) | 1,707.2 ms | 1,788.7 ms | 1,781.0 ms | 1,865.0 ms | **+73.8 ms** | +76.3 ms | +4.3 % |

Standard deviation within each series 35 to 45 ms. Three of the four series
drift upward by 3 to 5 ms per round (both images, both legs except the
classical taker), so the growth of on-disk state affects both images alike and
is not a migration effect.

### 3.2 Where inside the leg: six more alternating rounds, reading `mpc_execution_ms` from the receipt after each leg

| leg | window | classical median | hybrid median | difference |
|---|---|---:|---:|---:|
| maker | MPC round (`mpc_execution_ms`: spawn to last exit) | 725.5 ms | 797.1 ms | +71.6 ms |
| maker | everything else in the request | 471.2 ms | 554.8 ms | +83.6 ms |
| taker | MPC round | 719.4 ms | 745.5 ms | +26.1 ms |
| taker | everything else in the request | 972.2 ms | 1,060.6 ms | +88.4 ms |

Six rounds per cell; treat the split as indicative and the twenty-round totals
above as the measurement.

### 3.3 One TLS 1.3 handshake, loopback, one `s_server` and one `s_time` client, OpenSSL 3.5.5 for every row

`openssl s_time -new -time 10`, ten seconds of fresh handshakes, connections
divided by the 11 real seconds the tool reports (rounded up by the tool, so the
per-handshake figures carry about 5 % uncertainty; the ratios do not).

| certificate and key exchange | server authentication only | mutual authentication (what MP-SPDZ does) | certificate size (DER) |
|---|---:|---:|---:|
| RSA-2048 + X25519 (deployed classical) | 13,817 handshakes, 0.80 ms each | 10,259 handshakes, **1.07 ms** each | 783 B |
| Ed25519 + X25519 (best classical, not deployed) | 21,703, 0.51 ms | 19,697, 0.56 ms | 320 B |
| ML-DSA-65 + X25519MLKEM768 (deployed hybrid) | 8,371, 1.31 ms | 5,027, **2.19 ms** | 5,511 B |

The hybrid rows negotiated `X25519MLKEM768` and presented the ML-DSA-65
party certificate `CN=P0`; the classical rows negotiated X25519.

### 3.4 Component speeds in the two OpenSSL builds the images actually link (`openssl speed`, 3 s each, one core)

| primitive | OpenSSL 3.5.5 (hybrid image) | OpenSSL 3.0.20 (classical image) |
|---|---:|---:|
| RSA-2048 sign / verify | 266 µs / 15 µs | 268 µs / 15 µs |
| Ed25519 sign / verify | 25 µs / 81 µs | 31 µs / 84 µs |
| X25519 | 28 µs | 28 µs |
| ML-DSA-65 keygen / sign / verify | 142 µs / **692 µs** / 132 µs | n/a |
| ML-KEM-768 keygen / encapsulate / decapsulate | 29 µs / 16 µs / 26 µs | n/a |
| X25519MLKEM768 keygen / encapsulate / decapsulate | 55 µs / 70 µs / 55 µs | n/a |
| AES-256-GCM, 16 KiB blocks | 14.6 GB/s | 5.5 GB/s |

### 3.5 The engine receipt check

`oclob-mpc::MpcRunner::run` calls `qomm_mpc::engine_policy::verify` on every
round, before the timer that produces `mpc_execution_ms` starts. The check
re-reads and SHA-256-hashes `libSPDZ.so` (14.9 MB), `malicious-shamir-party.x`
(30.5 MB) and `Networking/ssl_sockets.h` and compares them with
`.pqc-tls.sha256`. Measured inside the running hybrid container, files in the
page cache: 33 ms per pass with a SHA-NI implementation (`openssl dgst`),
124 ms with a portable one (`sha256sum`); seven concurrent passes cost the
same wall time as one. The `sha2 0.10` crate the check uses selects SHA-NI at
run time on this CPU, so the expected cost is about 35 ms per round, all of it
new in the hybrid image and all of it outside the MPC window.

## 4. Prediction against measurement

**The net prediction (−10 to +5 ms per leg) was wrong.** The migration adds
56 ms to a resting order and 74 ms to a filled order at the median, 4 to 5 %
of the leg. The 50 ms falsification line in section 2.4 was crossed on both
legs. Three mistakes in the prediction, in order of size:

1. **The engine receipt check was not in the model.** I assumed the
   fail-closed hash of the engine artifacts ran once at start-up (it does, in
   `MpcRunner::compile`) and missed that `run` repeats it every round. That is
   about 35 ms per order (section 3.5), the largest single new term, and it is
   not cryptography on the wire at all; it is the integrity check the
   migration added around the engine. It accounts for roughly 40 % of the
   "everything else" difference on both legs.

2. **The TLS handshake prediction had the sign wrong.** RSA-2048 signing was
   over-predicted 4× (1.0 ms predicted, 0.27 ms measured) and ML-DSA-65
   signing in OpenSSL 3.5.5 was under-predicted 2.8× (0.25 ms predicted,
   0.69 ms measured; OpenSSL 3.5 ships a portable C implementation, and it is
   also twice as slow as the RustCrypto figure of 0.34 ms from the primitive
   bench). Summing the measured components reproduces the handshake numbers:
   hybrid mutual 2 × 0.69 + 4 × 0.13 + 0.18 = 2.08 ms against 2.19 measured,
   Ed25519 mutual 0.45 against 0.56. The RSA row carries about 0.4 ms the
   components do not explain; it does not change the ordering. So the hybrid
   mutual handshake costs **+1.1 ms over the RSA baseline that was deployed
   and +1.6 ms over an Ed25519 baseline**, not −1 ms. With twelve handshakes
   per party per round the TLS term is about +13 ms per round, which is the
   right sign and a third to a half of the measured MPC-window difference
   (+26 to +72 ms). The remainder of the window difference is not attributed;
   the two party binaries differ in more than TLS parameters (patched
   sources, a different libssl), and the party logs that would time the
   connection phase are deleted with the round directory. The record layer is
   excluded as a cause: the new build's AES-256-GCM is 2.6× faster.

3. **The application-side estimate (2 to 8 ms) was probably low but is not
   the story.** Every OCLOB application signature on this path is the v2
   hybrid (`oclob-core::application_crypto`, 3,373-byte ML-DSA-65 part), and
   the round produces a coordinator plan, seven party receipts, proof-slot
   metadata, a node receipt, five transition attestations and a three-node
   quorum, so 10 to 20 ms of pure-Rust hybrid signing and verifying per leg
   is the better estimate. Together with the receipt check that leaves about
   35 ms of the "everything else" difference to the non-cryptographic diff
   between old and new main (618 changed lines in `oclob-node`, v2 receipt
   formats), which this run cannot separate further.

What held: the round is dominated by spawning seven processes and running
malicious Shamir (predicted 500 to 3,000 ms, measured 720 to 800 ms in the
window); the scatter is of the predicted order (sd 35 to 45 ms); the bytes
are where the migration shows on the wire (a 5,511-byte certificate against
783, about 20 KB per handshake against 3 KB); and the primitive-level
ordering from the earlier bench (ML-KEM cheap, ML-DSA signing the expensive
step with a long tail) is what the handshake numbers reflect.

### Against the plan's targets

| target (PQC_MIGRATION_PLAN.md) | measured on this path | met? |
|---|---|---|
| throughput at least 95 % of the classical stack | a sequential order path runs at 95.5 % (maker) and 95.9 % (taker) of the classical rate | yes, by half a percentage point, one run |
| added latency on the online path at most 5 ms at p95 (10 ms ceiling) | +16.3 ms (maker), +76.3 ms (taker) at p95; +56 ms and +74 ms at the median | **no** |

### What would bring it back under the target, not done here

- Verify the engine artifacts once per process and re-check identity (device,
  inode, size, mtime) per round instead of re-hashing 45 MB. Saves about
  35 ms per order. Any change here re-opens the native acceptance.
- The TLS term (about 13 ms per round) is a consequence of spawning the seven
  parties per round; a persistent party mesh would remove 42 handshakes per
  order from both variants. That is a design change to the engine adapter, not
  a cryptographic one.
- ML-DSA-65 signing at 0.69 ms in OpenSSL 3.5.5 is the dominant handshake
  cost; an optimised provider would roughly halve the hybrid handshake.

Scope of this evidence: one run, one idle host, loopback networking, the two
images that were deployed, and the OCLOB demo path only. The QOMM demo on the
public port is built from a pre-migration tree and was not measured; the
settlement paths inside QOMM, DeFMI and DeKYX have no end-to-end comparison
yet.
