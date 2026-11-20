//! The AAC bridge admits IR data again on the consumer side.
//!
//! This crate does not perform source compilation. The process protocol carries
//! a closure's ABC bytes and the typed closure facts absent from standalone ABC.

mod execution;
mod protocol;
mod worker;

pub use protocol::{
    CtFacts, Domains, Failure, FailureKind, InputIdentity, Mode, Object, Options, Outcome, Request,
    RequestId, Response, SourceSpan, Value, WireGlobal, BUILD_BUDGET, CT_BUDGET, REQUEST_CAP,
    RESPONSE_CAP, TARGET,
};
pub use worker::{serve, Client};
