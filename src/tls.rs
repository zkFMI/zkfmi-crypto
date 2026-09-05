//! Shared, mandatory hybrid TLS key agreement for internal service connections.
//!
//! OpenSSL implements the TLS hybrid construction; the standalone KEM wire
//! format is deliberately not reimplemented inside TLS. Certificate trust,
//! hostname verification and application authorization remain the caller's
//! responsibility. Classical certificates do not become PQ authentication.
use openssl::error::ErrorStack;
use openssl::ssl::{SslContextBuilder, SslOptions, SslSessionCacheMode, SslVersion};

pub const HYBRID_GROUP: &str = "X25519MLKEM768";
pub const TRANSPORT_POLICY_VERSION: u16 = 1;

/// Apply after creating either a connector or acceptor, before building it.
/// An unsupported implementation fails during configuration; a classical-only
/// peer fails during the handshake. Never retry with another group list.
pub fn require_hybrid_key_exchange(builder: &mut SslContextBuilder) -> Result<(), ErrorStack> {
    builder.set_min_proto_version(Some(SslVersion::TLS1_3))?;
    builder.set_max_proto_version(Some(SslVersion::TLS1_3))?;
    builder.set_groups_list(HYBRID_GROUP)?;
    // Reconnects must perform a fresh hybrid exchange. Application connections
    // themselves are persistent, so this is not an exchange per order.
    builder.set_options(SslOptions::NO_TICKET);
    builder.set_session_cache_mode(SslSessionCacheMode::OFF);
    Ok(())
}
