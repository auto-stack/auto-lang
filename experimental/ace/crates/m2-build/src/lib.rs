//! PLAN-978: the engineering incremental multi-target build + precompiled
//! candidate SDK layer (M2-A).
//!
//! Identity layering (plan §5 ID-01..06, frozen by T-01):
//! * semantic keys stay the delivered AXS* layers (untouched);
//! * target fragments stay AXR*/AXC*/AXPY* (untouched);
//! * provider builds stay the 967 S967B `BuildIdentity` (consumed as-is);
//! * THIS crate adds the project-level engineering build identity
//!   (`ace-m2-build/project-build/1`), the runtime-compatibility record
//!   (`ace-m2-build/runtime-compat/1` — rustc identity is NOT an admission
//!   axis), the provider artifact selection
//!   (`ace-m2-build/provider-selection/1`) and the candidate-SDK recipe
//!   (`ace-m2-build/sdk-recipe/1`).
//!
//! Two working modes (plan §5 ID-02): the SOURCE-BUILD lane may run real
//! producers (in-process cranelift/rust-lld for native, fileset emit for
//! a2r/a2c/a2py); the PRECOMPILED-CONSUMPTION lane (`run` on a candidate
//! SDK) never requires a Rust/C toolchain, bindgen or the development
//! tree — AV executes in-process, native/precompiled artifacts are run
//! directly and the a2py lane runs through its DECLARED interpreter.
//!
//! Linux legs (2026-11-15 r14 ruling): this package implements and keeps
//! the full Linux fixture surface but runs NO Linux validation on this
//! machine; every Linux run/validation leg is `deferred-required/NOT_RUN`
//! with owner PLAN-1063.

pub mod compat;
pub mod driver;
pub mod identity;
pub mod load;
pub mod probe;
pub mod recipe;
#[cfg(windows)]
mod rust_project;
pub mod selection;

pub use compat::{
    check_compatibility, CompatRejection, ConsumerRequirements, ProviderOffer, TargetAxis,
};
pub use driver::{M2Build, ProjectInputs, RunObservation};
pub use identity::{ProjectBuildIdentity, SourceFile, TargetKind, M978B_LAYER};
pub use load::{select_layers, LoadSelection};
pub use recipe::{SdkRecipe, SDK_RECIPE_SCHEMA};
pub use selection::{
    admit_selection, AdmissionReceipt, ProviderArtifact, ProviderSelection, SelectionRejection,
};

/// A named m2-build rejection (every boundary in this crate rejects with a
/// stable `m2build.*` code BEFORE any dispatch — plan §5 ID-03/V-ID-REJECT).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M2Reject {
    pub code: &'static str,
    pub detail: String,
}

impl M2Reject {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for M2Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for M2Reject {}
