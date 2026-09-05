//! Independent P0 cryptographic boundaries. No integration with existing services.
#![forbid(unsafe_code)]

pub mod backend;
pub mod canonical;
pub mod error;
pub mod hybrid;
pub mod key;
pub mod suite;
pub mod traits;

/// The supported wire-format version. Unknown versions must be rejected.
pub const FORMAT_VERSION: u16 = 1;
