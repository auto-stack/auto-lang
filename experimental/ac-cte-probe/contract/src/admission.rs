//! Typed admission failures of the CTE probe (contract-decision §4–§7).
//! Codes are stable strings; the stage projection uses ONLY already
//! supported runner stages (admission/frontend → Parse, binding/verify →
//! Verify, capability → Capability) — no unknown stages.

use std::fmt;

/// Stable diagnostic code for one admission failure class.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CteCode {
    WrongSchema,
    UnknownRecipe,
    InputTooLarge,
    TooManyQueries,
    QueryDepth,
    ConsumerFrontend,
    CalleeFrontend,
    UnknownCapability,
    MetaNotGranted,
    UnknownQueryRef,
    QueryCycle,
    MetaRecordNotFound,
    BindingDuplicate,
    BindingUnknownOwner,
    BindingAmbiguous,
    BindingPlaceholderMismatch,
    BindingTypeMismatch,
    EntryMissing,
    CapabilityDenied,
    Trap(String),
    Resource(String),
    Cancelled,
    Infra(String),
}

impl CteCode {
    pub fn as_str(&self) -> &str {
        match self {
            CteCode::WrongSchema => "cte.request.schema",
            CteCode::UnknownRecipe => "cte.request.recipe",
            CteCode::InputTooLarge => "cte.resource.input-size",
            CteCode::TooManyQueries => "cte.resource.query-nodes",
            CteCode::QueryDepth => "cte.resource.query-depth",
            CteCode::ConsumerFrontend => "cte.consumer.frontend",
            CteCode::CalleeFrontend => "cte.callee.frontend",
            CteCode::UnknownCapability => "cte.capability.unknown",
            CteCode::MetaNotGranted => "cte.capability.meta",
            CteCode::UnknownQueryRef => "cte.query.unknown-ref",
            CteCode::QueryCycle => "cte.schedule.query-cycle",
            CteCode::MetaRecordNotFound => "cte.meta.record-not-found",
            CteCode::BindingDuplicate => "cte.consumer.binding-duplicate",
            CteCode::BindingUnknownOwner => "cte.consumer.binding-owner",
            CteCode::BindingAmbiguous => "cte.consumer.binding-ambiguous",
            CteCode::BindingPlaceholderMismatch => "cte.consumer.binding-placeholder",
            CteCode::BindingTypeMismatch => "cte.consumer.type-mismatch",
            CteCode::EntryMissing => "cte.request.entry",
            CteCode::CapabilityDenied => "cte.capability.denied",
            CteCode::Trap(k) => match k.trim_start_matches("av.trap.") {
                "int-overflow" => "cte.trap.int-overflow",
                "div-by-zero" => "cte.trap.div-by-zero",
                "div-overflow" => "cte.trap.div-overflow",
                "rem-overflow" => "cte.trap.rem-overflow",
                _ => "cte.trap.unknown",
            },
            CteCode::Resource(k) => match k.as_str() {
                "request-deadline" => "cte.resource.deadline",
                _ => "cte.resource.exhausted",
            },
            CteCode::Cancelled => "cte.cancelled",
            CteCode::Infra(_) => "cte.infra",
        }
    }
}

/// One admission failure: code + human detail (never a panic).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CteAdmissionError {
    pub code: CteCode,
    pub message: String,
    /// The CT call chain / query chain when the failure is a cycle.
    pub chain: Vec<String>,
}

impl CteAdmissionError {
    pub fn new(code: CteCode, message: impl Into<String>) -> Self {
        CteAdmissionError {
            code,
            message: message.into(),
            chain: Vec::new(),
        }
    }
    pub fn with_chain(mut self, chain: Vec<String>) -> Self {
        self.chain = chain;
        self
    }
    /// Classification for the runner: language-diagnostic classes vs
    /// resource/cancel/infra (never language verdicts).
    pub fn is_language_class(&self) -> bool {
        matches!(
            self.code,
            CteCode::WrongSchema
                | CteCode::UnknownRecipe
                | CteCode::UnknownCapability
                | CteCode::MetaNotGranted
                | CteCode::UnknownQueryRef
                | CteCode::QueryCycle
                | CteCode::MetaRecordNotFound
                | CteCode::BindingDuplicate
                | CteCode::BindingUnknownOwner
                | CteCode::BindingAmbiguous
                | CteCode::BindingPlaceholderMismatch
                | CteCode::BindingTypeMismatch
                | CteCode::EntryMissing
                | CteCode::CapabilityDenied
                | CteCode::ConsumerFrontend
                | CteCode::CalleeFrontend
                | CteCode::Trap(_)
        )
    }
}

impl fmt::Display for CteAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.chain.is_empty() {
            write!(f, "{}: {}", self.code.as_str(), self.message)
        } else {
            write!(
                f,
                "{}: {} (chain: {})",
                self.code.as_str(),
                self.message,
                self.chain.join(" -> ")
            )
        }
    }
}

impl std::error::Error for CteAdmissionError {}
