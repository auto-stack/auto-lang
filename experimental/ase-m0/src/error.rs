//! ASE M0 error taxonomy (PLAN-1021 AC-01/AC-05): resolution, source,
//! compile, execution and resource failures are distinguished by name.
//! `dispatches` counts entries into the public ACE boundary
//! (`compile_package_product`); every pre-boundary rejection carries 0.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// URI parsing / identity / version / capability admission.
    Resolve,
    /// Reading the registered source closure inside the allowed root.
    Source,
    /// The public packages compile chain (bind/resolve/typecheck/verify).
    Compile,
    /// AV cold link over the checked product.
    Link,
    /// Entry execution on the AV target.
    Execute,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Resolve => "resolve",
            Phase::Source => "source",
            Phase::Compile => "compile",
            Phase::Link => "link",
            Phase::Execute => "execute",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Malformed or unsupported URI (scheme/route/userinfo/port/query/fragment).
    Uri,
    /// Registered-domain refusal: unknown identity, unsupported version or
    /// capability (named, before any source read or dispatch).
    Authz,
    /// Host metadata is structurally invalid.
    Registry,
    /// The registered closure escapes the allowed root (`..`, absolute).
    SourceEscape,
    /// A reparse point (junction/symlink) inside the allowed root.
    SourceReparse,
    /// A registered source file is absent or not a regular file.
    SourceMissing,
    /// Filesystem read failure on a contained source.
    SourceIo,
    /// Package diagnostics from the public compile chain (non-resource).
    Compile,
    /// Resource refusal through the public entry (budget `source.resource`,
    /// artifact-store IO). Never reclassified as a syntax error or success.
    Resource,
    /// AV cold link failure over the checked product.
    Link,
    /// Entry execution failure or a non-i32 entry value.
    Execute,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Uri => "uri",
            Kind::Authz => "authz",
            Kind::Registry => "registry",
            Kind::SourceEscape => "source-escape",
            Kind::SourceReparse => "source-reparse",
            Kind::SourceMissing => "source-missing",
            Kind::SourceIo => "source-io",
            Kind::Compile => "compile",
            Kind::Resource => "resource",
            Kind::Link => "link",
            Kind::Execute => "execute",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AseError {
    pub phase: Phase,
    pub kind: Kind,
    pub message: String,
    /// ACE-boundary dispatches observed before this failure (0 for every
    /// resolution/authorization/source refusal).
    pub dispatches: u64,
}

impl AseError {
    pub fn new(phase: Phase, kind: Kind, message: impl Into<String>) -> Self {
        AseError {
            phase,
            kind,
            message: message.into(),
            dispatches: 0,
        }
    }

    /// Bind the dispatch count observed at the ACE boundary.
    pub fn with_dispatches(mut self, dispatches: u64) -> Self {
        self.dispatches = dispatches;
        self
    }
}

impl std::fmt::Display for AseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "phase={} kind={} dispatch={}: {}",
            self.phase.as_str(),
            self.kind.as_str(),
            self.dispatches,
            self.message
        )
    }
}

impl std::error::Error for AseError {}
