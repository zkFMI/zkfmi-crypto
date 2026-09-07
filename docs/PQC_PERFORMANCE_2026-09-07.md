# Post-quantum migration: what it costs, predicted and measured

Date: 2026-09-07. Scope: the primitive boundary that `zkfmi-crypto` exports
and every repository now consumes. The question is narrow: for each object the
stack signs and each key exchange it performs, what did the hybrid suite add in
bytes and in time, relative to the classical suite it replaced. The method is
the one this project uses for every measurement: predict from the mechanism
first, then run once, then explain any gap.

## 1. What changed

| boundary | before | now (mandatory, fail-closed) |
|---|---|---|
| signatures on quotes, settlement instructions, key rotation, governance, attestations, orders, audit checkpoints | Ed25519 | Ed25519 **and** ML-DSA-65, both verified, either failure rejects |
| key exchange for recipient envelopes and node transport | X25519 | X25519 **and** ML-KEM-768, secrets combined with HKDF-SHA256 over both shared secrets and both ciphertexts |
| TLS between nodes | X25519 groups | OpenSSL 3.5.5 with the `X25519MLKEM768` group required (`require-hybrid-tls.patch`) |
| commitments, sigma protocols, Bulletproofs, FROST | ristretto255 | unchanged (see "what stays classical" on the post-quantum page) |

The hybrid signature is the concatenation of the two component signatures; the
hybrid public key is the concatenation of the two public keys. There is no
compression and no shared randomness, so the byte cost is exactly additive.

## 2. Prediction, written before running anything

### 2.1 Bytes (exact, from the standards; no measurement needed)

| object | Ed25519 / X25519 | ML-DSA-65 / ML-KEM-768 | hybrid | ratio hybrid : classical |
|---|---:|---:|---:|---:|
| signature public key | 32 | 1952 | 1984 | 62.0× |
| signature | 64 | 3309 | 3373 | 52.7× |
| KEM public key (encapsulation key) | 32 | 1184 | 1216 | 38.0× |
| KEM ciphertext | 32 | 1088 | 1120 | 35.0× |

These are FIPS 204 and FIPS 203 parameter-set sizes. The bench prints the
lengths of the encoded objects it actually produced; if they differ from this
table the encoder is wrong, not the table.

Consequence for wire objects: a settlement instruction whose body is a few
hundred bytes with one Ed25519 signature grows by about 3.3 KB per signature
and by about 1.9 KB per embedded public key. Quorum receipts that carry one
signature per FROST participant do not change, because FROST stays classical.
The 96-byte reconciliation message does not change either; it is not signed
per message.

### 2.2 Time (predicted ranges, one x86-64 core, the pinned pure-Rust crates)

The crates are `ml-dsa 0.1.1` and `ml-kem 0.3.2` from RustCrypto, pure Rust
without platform intrinsics, against `ed25519-dalek 2.2.0` and
`x25519-dalek 2.0.1`, which carry hand-tuned field arithmetic. Reference
figures for optimised AVX2 implementations of ML-DSA-65 are about 60 µs
keygen, 150 to 250 µs sign, 60 µs verify; for ML-KEM-768 about 20 µs each
operation. A pure-Rust implementation without intrinsics is typically two to
four times slower than that. ML-DSA signing is a rejection-sampling loop with
an expected 5.1 attempts at this parameter set, so its distribution has a long
right tail; the median and the mean will differ visibly, and p90 will be
roughly twice the median.

| operation | Ed25519 / X25519 | ML-DSA-65 / ML-KEM-768 | hybrid (sum plus context hashing) |
|---|---:|---:|---:|
| signature keygen | 15 to 25 µs | 100 to 250 µs | 120 to 280 µs |
| sign | 15 to 30 µs | 300 to 900 µs median | 320 to 950 µs |
| verify | 45 to 70 µs | 100 to 250 µs | 150 to 320 µs |
| KEM keygen | 40 to 60 µs | 40 to 120 µs | 80 to 180 µs |
| encapsulate | 40 to 60 µs | 50 to 130 µs | 90 to 200 µs |
| decapsulate | 40 to 60 µs | 60 to 150 µs | 100 to 220 µs |

What decides whether any of this matters: the slowest operation the stack
performs per settlement is already the MPC round trip and the 7-node consensus,
measured in milliseconds to seconds. A hybrid sign at one millisecond and a
hybrid verify at a few hundred microseconds are one to two orders of magnitude
below that. The prediction, stated so it can be wrong: **the migration is not
visible in the end-to-end latency of a settlement; it is visible only in
bandwidth and storage, where each signed object grows by about fifty times the
signature it carried.** Verification throughput on a single core falls from
roughly 15 to 20 thousand Ed25519 verifications per second to roughly 3 to 6
thousand hybrid verifications per second, which is still above any load this
stack has been run at.

### 2.3 TLS

`X25519MLKEM768` adds one ML-KEM-768 encapsulation key (1184 B) to the
ClientHello and one ciphertext (1088 B) to the ServerHello, plus one
encapsulate and one decapsulate per handshake. Against a LAN round trip of
about 0.2 ms the added computation is comparable to one round trip; against a
WAN round trip it is noise. The handshake bytes grow by about 2.3 KB, which is
the figure that matters for the ClientHello fitting in one initial flight. Not
measured here; the native acceptance run exercises the handshake and its
receipts are in `docs/verification/`.

## 3. Measurement

Bench: `examples/pqc_bench.rs` in this repository, run with
`cargo run --locked --release --example pqc_bench` inside the pinned lane
container on softbank-l40s (`scripts/pqc-remote.sh zkfmi-crypto`). One core,
300 samples per operation after 10 warm-up calls, 256-byte message. The bench
goes through the same `Signer` / `Verifier` / `KemEncapsulator` /
`KemDecapsulator` traits the stack uses, so the classical rows include the
same context hashing as the hybrid rows.

Run 2026-09-07T11:16:54Z, host ngi-external022-vm1 (softbank-l40s), container
image `sha256:0e2bcaef…`, rustc 1.97.1, release profile. Receipt:
`.artifacts/20260907T111649Z-zkfmi-crypto.log`. One run, as the method
requires; the numbers below are medians over 300 samples with p90 in brackets
where the distribution is not tight.

### 3.1 Measured time (µs, one core)

| operation | Ed25519 / X25519 | ML-DSA-65 / ML-KEM-768 | hybrid | hybrid : classical |
|---|---:|---:|---:|---:|
| signature keygen | 12.2 | 221.9 | 238.1 | 19.5× |
| sign | 13.6 | 340.2 (p90 951.0) | 359.3 (p90 972.2) | 26.4× |
| verify | 38.4 | 126.8 | 173.7 | 4.5× |
| KEM keygen | 11.9 | 38.6 | 51.0 | 4.3× |
| encapsulate | 48.9 | 37.0 | 88.1 | 1.8× |
| decapsulate | 37.0 | 40.4 | 79.6 | 2.2× |

### 3.2 Measured bytes

Identical to the table in 2.1 in every row: 32 / 64, 1952 / 3309, 1984 / 3373
for signatures; 32 / 32, 1184 / 1088, 1216 / 1120 for the KEM. The encoders
produce exactly the standard sizes with no framing overhead.

### 3.3 Prediction against measurement

Everything that was predicted from the standards (bytes) and from the
structure of the hybrid (additivity: every hybrid row is the sum of its two
components to within 5 µs, the context hashing) held. Three timing predictions
missed, all in the same direction:

- **ML-DSA-65 sign, keygen and verify landed inside the predicted ranges**
  (340 in 300 to 900, 222 in 100 to 250, 127 in 100 to 250). The rejection
  tail is heavier than the "p90 about twice the median" guess: p90 is 2.8× the
  median. That is the expected 5.1 attempts showing as a distribution rather
  than a mean; nothing to fix.
- **ML-KEM-768 is faster than predicted** (37 to 40 µs against 50 to 150). The
  pure-Rust `ml-kem` crate is within about 2× of optimised AVX2 figures, not
  the 2 to 4× assumed. The prediction used the wrong prior for this crate.
- **X25519 key generation was over-predicted by 4×** (11.9 against 40 to 60).
  Key generation is a fixed-base scalar multiplication, which dalek does from
  precomputed tables; only the variable-base Diffie-Hellman costs about 37 µs.
  The hybrid KEM keygen therefore came in at 51 µs, below the 80 to 180
  predicted, for the same reason.

The conclusion in 2.2 stands and is now measured rather than argued: **a
hybrid sign costs about 0.36 ms and a hybrid verify about 0.17 ms on one core,
against an end-to-end settlement measured in tens of milliseconds to seconds.**
The migration is invisible in latency and visible only in bytes, where each
signed object grows by 3309 bytes per signature and 1952 bytes per embedded
public key. Single-core verification throughput falls from about 26,000 to
about 5,800 per second. Encapsulation and decapsulation, which are on the path
of every recipient envelope, less than double.

### 3.4 What this does not measure

Proof systems (Bulletproofs, sigma protocols, FROST) are unchanged and not
re-measured here. The TLS handshake is exercised by the native acceptance run,
not timed by this bench. Multi-core throughput scales linearly for all six
operations; none of them share state.
