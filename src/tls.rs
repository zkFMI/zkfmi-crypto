//! Mandatory hybrid key agreement and ML-DSA-65 authentication for internal TLS.
//!
//! OpenSSL implements both protocol algorithms. The TLS signature is ML-DSA-65,
//! not a composite Ed25519/ML-DSA signature. Application approvals retain their
//! separate hybrid signatures. Trust anchors, hostname verification and
//! application authorization remain the caller's responsibility.
use openssl::error::ErrorStack;
use openssl::pkey::{KeyType, PKey, Private};
use openssl::ssl::{SslContextBuilder, SslOptions, SslSessionCacheMode, SslVerifyMode, SslVersion};
use openssl::x509::X509Ref;
use zeroize::Zeroizing;

pub const HYBRID_GROUP: &str = "X25519MLKEM768";
pub const AUTHENTICATION_SCHEME: &str = "mldsa65";
pub const AUTHENTICATION_OID: &str = "2.16.840.1.101.3.4.3.18";
pub const TRANSPORT_POLICY_VERSION: u16 = 2;

/// Apply before building a connector or acceptor. The entire verified chain,
/// including its configured trust anchor, must use ML-DSA-65 keys and signatures.
/// Unsupported libraries and classical-only peers fail closed without retry.
pub fn require_pqc_transport(
    builder: &mut SslContextBuilder,
    verify_mode: SslVerifyMode,
) -> Result<(), ErrorStack> {
    builder.set_min_proto_version(Some(SslVersion::TLS1_3))?;
    builder.set_max_proto_version(Some(SslVersion::TLS1_3))?;
    builder.set_groups_list(HYBRID_GROUP)?;
    builder.set_sigalgs_list(AUTHENTICATION_SCHEME)?;
    builder.set_verify_callback(verify_mode | SslVerifyMode::PEER, |valid, context| {
        valid
            && context
                .current_cert()
                .is_some_and(certificate_uses_pqc_authentication)
    });
    // Persistent connections amortize setup. Reconnects require fresh agreement
    // and authentication rather than resurrecting retired certificate sessions.
    builder.set_options(SslOptions::NO_TICKET);
    builder.set_session_cache_mode(SslSessionCacheMode::OFF);
    Ok(())
}

pub fn certificate_uses_pqc_authentication(certificate: &X509Ref) -> bool {
    certificate
        .public_key()
        .is_ok_and(|key| key.is_a(KeyType::ML_DSA_65))
        && openssl::asn1::Asn1Object::from_str(AUTHENTICATION_OID).is_ok_and(|oid| {
            oid.nid() != openssl::nid::Nid::UNDEF
                && certificate.signature_algorithm().object().nid() == oid.nid()
        })
}

/// Generate independently random node-local or CA key material. Only OpenSSL
/// performs key construction; the seed buffer is erased when this function exits.
pub fn generate_authentication_key() -> Result<PKey<Private>, String> {
    let mut seed = Zeroizing::new([0_u8; 32]);
    getrandom_04::fill(seed.as_mut()).map_err(|_| "TLS key randomness failed".to_string())?;
    PKey::private_key_from_seed(None, KeyType::ML_DSA_65, None, seed.as_ref())
        .map_err(|error| error.to_string())
}
