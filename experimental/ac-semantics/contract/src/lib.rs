//! PLAN-796 semantic identity / metadata contract library.
//!
//! This crate owns the semantic reference envelope, the complete meta
//! category/coverage matrix, and the versioned compact page codec that
//! PLAN-797 (ABI probe) consumes read-only. It deliberately has no
//! dependency on any product crate (auto-lang / UI / VM / Cranelift /
//! auto-man): semantic identity is logical, not host-shaped.
//!
//! The frozen contracts live in
//! `docs/plans/attachments/796/contract-decision.md` (T-01); every constant
//! below that participates in cross-plan identity is annotated with its
//! decision anchor.

pub mod codec;
pub mod identity;
pub mod metadata;

pub use codec::{
    decode_page, encode_page, CachedResult, CodecError, DepFingerprint, DepManifest, FactDiag,
    FieldDesc, FnMeta, InferFacts, Mode, ModuleIndex, NegDep, PageBody, PageHeader, PageKind,
    PageView, RecGroup, RecordMeta, RecordShape, ResultFacts, SemaPage, TypeDesc,
    MAX_NESTING_DEPTH, MAX_PAGE_BODY, SEMA_CODEC_VERSION, SEMA_PAGE_SCHEMA_V1,
};
pub use identity::{
    logical_id, ContentVersion, DeclKind, DeclRef, LogicalId, PackageId, RefError,
    SEMA_LOGICAL_DOMAIN, SEMA_PRODUCER_ID, SEMA_REF_SCHEMA_V1, SemaRef,
};
