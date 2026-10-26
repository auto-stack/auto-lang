//! Build the single production source against delivered public core APIs.
pub use auto_av_core::{abc, abc_verify, contract, function_code, mir, mir_verify};
#[path = "../../../core/src/loaded_program.rs"]
pub mod loaded_program;
