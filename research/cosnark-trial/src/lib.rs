//! Research-only collaborative proof experiment.
//!
//! Neither importing this crate nor passing arithmetic checks registers a
//! post-quantum backend in any zkFMI application. The end-to-end verifier and
//! malicious-security composition must be evaluated separately.
#![forbid(unsafe_code)]

pub mod algebra;
pub mod circuit;
pub mod field;
pub mod fold_binding_probe;
#[cfg(feature = "native-prover")]
pub mod full_trial;
pub mod integration;
#[cfg(feature = "native-prover")]
pub mod mpc_program;
#[cfg(feature = "native-prover")]
pub mod native;
#[cfg(feature = "pqc-note-circuit")]
pub mod note_circuit;
#[cfg(feature = "pqc-note-circuit")]
pub mod note_mpc;
#[cfg(all(feature = "pqc-note-circuit", feature = "native-prover"))]
pub mod note_native;
#[cfg(feature = "pqc-note-circuit")]
pub mod note_piop;
#[cfg(feature = "native-prover")]
pub mod party;
pub mod pcs;
#[cfg(feature = "native-prover")]
pub mod prover;
#[cfg(feature = "pqc-note-circuit")]
pub mod recursive_pcs;
pub mod transcript;
pub mod verifier;
