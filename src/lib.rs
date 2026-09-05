//! Shared versioned cryptographic boundaries for ZKFMI services.
#![forbid(unsafe_code)]

pub mod backend;
pub mod canonical;
pub mod error;
pub mod hybrid;
pub mod key;
pub mod quorum;
pub mod suite;
#[cfg(feature = "test-support")]
pub mod test_support;
#[cfg(feature = "tls")]
pub mod tls;
pub mod traits;

/// The supported wire-format version. Unknown versions must be rejected.
pub const FORMAT_VERSION: u16 = 1;
