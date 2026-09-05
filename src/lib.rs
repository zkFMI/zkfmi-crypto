//! Independent P0 cryptographic boundaries. No integration with existing services.
#![forbid(unsafe_code)]

/// The supported wire-format version. Unknown versions must be rejected.
pub const FORMAT_VERSION: u16 = 1;
