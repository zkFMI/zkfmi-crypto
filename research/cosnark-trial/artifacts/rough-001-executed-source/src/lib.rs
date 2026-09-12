//! Research-only collaborative proof experiment.
//!
//! Neither importing this crate nor passing arithmetic checks registers a
//! post-quantum backend in any zkFMI application. The end-to-end verifier and
//! malicious-security composition must be evaluated separately.
#![forbid(unsafe_code)]

pub mod field;
pub mod fold_binding_probe;
