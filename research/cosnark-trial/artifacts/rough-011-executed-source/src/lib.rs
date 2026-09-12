//! Research-only collaborative proof experiment.
//!
//! Neither importing this crate nor passing arithmetic checks registers a
//! post-quantum backend in any zkFMI application. The end-to-end verifier and
//! malicious-security composition must be evaluated separately.
#![forbid(unsafe_code)]

pub mod field;
pub mod fold_binding_probe;
pub mod algebra;
pub mod circuit;
pub mod transcript;
pub mod pcs;
pub mod verifier;
pub mod mpc_program;
pub mod party;
pub mod native;
pub mod prover;
pub mod full_trial;
