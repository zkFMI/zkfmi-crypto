//! Classical versus hybrid post-quantum cost at the primitive boundary.
//!
//! Measures, on one core, the three operations the stack performs for every
//! signed object (key generation, sign, verify) and for every key exchange
//! (key generation, encapsulate, decapsulate), for the classical suite the
//! stack used before the migration, the post-quantum component on its own,
//! and the hybrid suite that is now mandatory. Sizes are reported from the
//! encoded objects, not from constants. Output is a JSON document on stdout;
//! `docs/PQC_PERFORMANCE_2026-09-07.md` records the prediction made before
//! this was run and the numbers it produced.
//!
//! Run with `cargo run --locked --release --example pqc_bench`.

use std::time::{Duration, Instant};

use zkfmi_crypto::{
    backend::{
        Ed25519Signer, Ed25519Verifier, MlDsa65Signer, MlDsa65Verifier, MlKem768Encapsulator,
        MlKem768Key,
    },
    hybrid::{
        kem::{HybridKemEncapsulator, HybridKemKey},
        signature::{HybridSigner, HybridVerifier},
    },
    key::KeyPurpose,
    traits::{KemDecapsulator, KemEncapsulator, Signer, Verifier},
};

const ITERATIONS: usize = 300;
const MESSAGE_BYTES: usize = 256;

struct Stat {
    median_us: f64,
    p90_us: f64,
    mean_us: f64,
}

fn stat(mut samples: Vec<Duration>) -> Stat {
    samples.sort();
    let n = samples.len();
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    Stat {
        median_us: us(samples[n / 2]),
        p90_us: us(samples[(n * 9) / 10]),
        mean_us: samples.iter().map(|d| us(*d)).sum::<f64>() / n as f64,
    }
}

fn time<T>(f: impl FnMut() -> T) -> Stat {
    let mut f = f;
    // Warm-up so page faults and lazy tables do not land in the first sample.
    for _ in 0..10 {
        std::hint::black_box(f());
    }
    let mut samples = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        std::hint::black_box(f());
        samples.push(start.elapsed());
    }
    stat(samples)
}

fn emit(name: &str, op: &str, s: &Stat, first: &mut bool) {
    if !*first {
        println!(",");
    }
    *first = false;
    print!(
        "    {{\"suite\": \"{name}\", \"op\": \"{op}\", \"median_us\": {:.1}, \"p90_us\": {:.1}, \"mean_us\": {:.1}}}",
        s.median_us, s.p90_us, s.mean_us
    );
}

fn signature_suite<S, V>(
    name: &str,
    generate: impl Fn() -> S,
    verifier: V,
    message: &[u8],
    first: &mut bool,
) -> (usize, usize)
where
    S: Signer,
    V: Verifier,
{
    let purpose = KeyPurpose::SettlementInstruction;
    let signer = generate();
    let public_key = signer.public_key();
    let signature = signer.sign(purpose, message).expect("sign");
    verifier
        .verify(purpose, &public_key, message, &signature)
        .expect("verify");
    emit(name, "keygen", &time(|| generate().public_key()), first);
    emit(
        name,
        "sign",
        &time(|| signer.sign(purpose, message).expect("sign")),
        first,
    );
    emit(
        name,
        "verify",
        &time(|| {
            verifier
                .verify(purpose, &public_key, message, &signature)
                .expect("verify")
        }),
        first,
    );
    (public_key.len(), signature.len())
}

fn kem_suite<K, E>(
    name: &str,
    generate: impl Fn() -> K,
    encapsulator: E,
    first: &mut bool,
) -> (usize, usize)
where
    K: KemDecapsulator,
    E: KemEncapsulator,
{
    let key = generate();
    let public_key = key.public_key();
    let encapsulation = encapsulator.encapsulate(&public_key).expect("encapsulate");
    let shared = key
        .decapsulate(&encapsulation.ciphertext)
        .expect("decapsulate");
    assert_eq!(
        shared.as_slice(),
        encapsulation.shared_secret.as_slice(),
        "shared secrets differ"
    );
    emit(name, "keygen", &time(|| generate().public_key()), first);
    emit(
        name,
        "encapsulate",
        &time(|| encapsulator.encapsulate(&public_key).expect("encapsulate")),
        first,
    );
    emit(
        name,
        "decapsulate",
        &time(|| {
            key.decapsulate(&encapsulation.ciphertext)
                .expect("decapsulate")
        }),
        first,
    );
    (public_key.len(), encapsulation.ciphertext.len())
}

fn main() {
    let message = vec![0x5a_u8; MESSAGE_BYTES];
    let mut first = true;
    println!("{{");
    println!("  \"iterations\": {ITERATIONS},");
    println!("  \"message_bytes\": {MESSAGE_BYTES},");
    println!("  \"timings\": [");

    let ed = signature_suite(
        "ed25519",
        || Ed25519Signer::generate().expect("ed25519 keygen"),
        Ed25519Verifier,
        &message,
        &mut first,
    );
    let mldsa = signature_suite(
        "ml-dsa-65",
        || MlDsa65Signer::generate().expect("ml-dsa keygen"),
        MlDsa65Verifier,
        &message,
        &mut first,
    );
    let hybrid_sig = signature_suite(
        "ed25519+ml-dsa-65",
        || HybridSigner::generate().expect("hybrid keygen"),
        HybridVerifier,
        &message,
        &mut first,
    );
    let x = kem_suite(
        "x25519",
        || X25519Key::generate(),
        X25519Encapsulator,
        &mut first,
    );
    let mlkem = kem_suite(
        "ml-kem-768",
        || MlKem768Key::generate().expect("ml-kem keygen"),
        MlKem768Encapsulator,
        &mut first,
    );
    let hybrid_kem = kem_suite(
        "x25519+ml-kem-768",
        || HybridKemKey::generate().expect("hybrid kem keygen"),
        HybridKemEncapsulator,
        &mut first,
    );

    println!();
    println!("  ],");
    println!("  \"sizes\": [");
    let rows = [
        ("ed25519", "public_key", ed.0),
        ("ed25519", "signature", ed.1),
        ("ml-dsa-65", "public_key", mldsa.0),
        ("ml-dsa-65", "signature", mldsa.1),
        ("ed25519+ml-dsa-65", "public_key", hybrid_sig.0),
        ("ed25519+ml-dsa-65", "signature", hybrid_sig.1),
        ("x25519", "public_key", x.0),
        ("x25519", "ciphertext", x.1),
        ("ml-kem-768", "public_key", mlkem.0),
        ("ml-kem-768", "ciphertext", mlkem.1),
        ("x25519+ml-kem-768", "public_key", hybrid_kem.0),
        ("x25519+ml-kem-768", "ciphertext", hybrid_kem.1),
    ];
    for (i, (suite, object, bytes)) in rows.iter().enumerate() {
        let sep = if i + 1 == rows.len() { "" } else { "," };
        println!("    {{\"suite\": \"{suite}\", \"object\": \"{object}\", \"bytes\": {bytes}}}{sep}");
    }
    println!("  ]");
    println!("}}");
}

// The classical key exchange on its own, expressed through the same traits so
// the three KEM rows are measured by identical code. The hybrid suite's X25519
// half is exactly this: an ephemeral Diffie-Hellman whose public key is the
// ciphertext, so the "ciphertext" row is 32 bytes by construction.
struct X25519Key {
    secret: x25519_dalek::StaticSecret,
}

impl X25519Key {
    fn generate() -> Self {
        Self {
            secret: x25519_dalek::StaticSecret::random_from_rng(rand_core_06::OsRng),
        }
    }
}

impl KemDecapsulator for X25519Key {
    fn suite(&self) -> zkfmi_crypto::suite::Suite {
        zkfmi_crypto::suite::Suite::new(zkfmi_crypto::suite::SuiteId::X25519Tls13)
    }
    fn public_key(&self) -> Vec<u8> {
        x25519_dalek::PublicKey::from(&self.secret).as_bytes().to_vec()
    }
    fn decapsulate(&self, ciphertext: &[u8]) -> zkfmi_crypto::error::Result<zkfmi_crypto::traits::SecretBytes> {
        let bytes: [u8; 32] = ciphertext
            .try_into()
            .map_err(|_| zkfmi_crypto::error::CryptoError::InvalidCiphertext)?;
        let shared = self.secret.diffie_hellman(&x25519_dalek::PublicKey::from(bytes));
        Ok(zeroize::Zeroizing::new(shared.as_bytes().to_vec()))
    }
}

struct X25519Encapsulator;

impl KemEncapsulator for X25519Encapsulator {
    fn suite(&self) -> zkfmi_crypto::suite::Suite {
        zkfmi_crypto::suite::Suite::new(zkfmi_crypto::suite::SuiteId::X25519Tls13)
    }
    fn encapsulate(
        &self,
        public_key: &[u8],
    ) -> zkfmi_crypto::error::Result<zkfmi_crypto::traits::Encapsulation> {
        let bytes: [u8; 32] = public_key
            .try_into()
            .map_err(|_| zkfmi_crypto::error::CryptoError::InvalidKey)?;
        let ephemeral = x25519_dalek::StaticSecret::random_from_rng(rand_core_06::OsRng);
        let shared = ephemeral.diffie_hellman(&x25519_dalek::PublicKey::from(bytes));
        Ok(zkfmi_crypto::traits::Encapsulation {
            ciphertext: x25519_dalek::PublicKey::from(&ephemeral).as_bytes().to_vec(),
            shared_secret: zeroize::Zeroizing::new(shared.as_bytes().to_vec()),
        })
    }
}
