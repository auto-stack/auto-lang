//! Static project-model DTO for pac.at declarations (PLAN-780 T-02).
//!
//! The DTO mirrors the *frozen static subset* (T-01 report §3) with
//! normalizations aligned point-by-point to `auto_man::Pac::new` — the
//! pilot's comparison oracle — while depending only on auto-atom/auto-val:
//! no compiler, VM, builder, index-store, or UI types cross this boundary.
//! Unknown fields and unmodeled nodes are catalogued verbatim (with source
//! lines); unsupported constructs make the file non-static-equivalent
//! without failing the parse.

use auto_val::Value;
use std::path::PathBuf;

/// Where a document came from. Relative paths declared inside the document
/// (e.g. `dep x { path: "../y" }`, `back: { project }`) resolve against the
/// pac.at's own directory — consumers join against
/// `source.path.parent()`, matching every production read point.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceInfo {
    pub path: Option<PathBuf>,
}

impl SourceInfo {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
        }
    }

    pub fn inline() -> Self {
        Self { path: None }
    }
}

/// A construct only the config VM can produce. Files with a non-empty
/// `unsupported` list are NOT static-equivalent — never silently treated as
/// equal (PLAN-780 AC-02/V-03).
#[derive(Debug, Clone, PartialEq)]
pub struct UnsupportedConstruct {
    pub kind: UnsupportedKind,
    pub line: usize,
    pub column: usize,
    /// Verbatim text of the construct (or its unescaped content for
    /// strings) — preserved so no field is lost to classification.
    pub raw: String,
    /// Declaring prop key when known.
    pub key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedKind {
    /// `${expr}` / `$ident` interpolation.
    Interpolation,
    /// Backtick or `f"…"` f-string.
    FString,
    /// Bare identifier value outside {true,false,null,nil,on,off} —
    /// a variable reference in the config VM.
    BareIdentValue,
    /// `key = value` assignment statement.
    Assignment,
    /// `ident(...)` call in value position.
    Call,
    /// Bare `on`/`off` outside the toggle fields (VM evaluates Bool; no
    /// static equivalent for the affected field's normalization).
    OnOffOutsideToggle,
    /// Single-quoted string value (outside the static pac.at grammar).
    SingleQuotedString,
    /// A `dep` node whose name cannot be represented statically.
    DepNameUnrepresentable,
    /// A modeled field's value has the wrong static shape (e.g. `members`
    /// declared as a scalar string).
    TypeMismatch { field: &'static str },
}

impl std::fmt::Display for UnsupportedKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UnsupportedKind::Interpolation => "interpolation",
            UnsupportedKind::FString => "f-string",
            UnsupportedKind::BareIdentValue => "bare-identifier-value",
            UnsupportedKind::Assignment => "assignment",
            UnsupportedKind::Call => "call",
            UnsupportedKind::OnOffOutsideToggle => "on/off outside toggle fields",
            UnsupportedKind::SingleQuotedString => "single-quoted string",
            UnsupportedKind::DepNameUnrepresentable => "dep name unrepresentable",
            UnsupportedKind::TypeMismatch { field } => {
                return write!(f, "type mismatch on '{field}'")
            }
        };
        f.write_str(s)
    }
}

/// Scene kind — mirrors `auto_man::pac::Scene::from_str` (unknown → Default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneKind {
    Workspace,
    Ui,
    C,
    #[default]
    Default,
}

impl SceneKind {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "workspace" => Self::Workspace,
            "ui" => Self::Ui,
            "c" | "core" => Self::C,
            _ => Self::Default,
        }
    }
}

/// Render declaration — mirrors `auto_lang::config::BackendConfig::from_value`
/// shapes (single / split / multi), plus the declared-but-unknown form the
/// oracle maps to `backend_config = None` (e.g. `backend: "rust-tauri"`).
#[derive(Debug, Clone, PartialEq)]
pub enum RenderSpec {
    /// No render/backend declaration survived parsing.
    Absent,
    Single(String),
    Multi(Vec<String>),
    Split { front: Vec<String>, back: String },
    /// Declared, but not a statically-known backend name/shape. The verbatim
    /// repr is preserved; the oracle keeps it only in the legacy
    /// `backend` string.
    Unknown(String),
}

/// The render/backend declaration pair, with the effective-backend fallback
/// chain of `Pac::new` (render wins over backend; lang rust/ts infers the
/// legacy string when both are absent).
#[derive(Debug, Clone, PartialEq)]
pub struct RenderDecl {
    pub render: RenderSpec,
    /// Legacy `backend:` field as declared (to_astr form), when present.
    pub legacy_backend: Option<String>,
    /// Pac.backend fallback: render astr → backend astr → lang inference
    /// ("rust"→rust, "ts"/"typescript"→ts, else "c"); None only when nothing
    /// at all is declared *and* the caller did not supply a lang.
    pub effective_backend: Option<String>,
}

/// One `dep` declaration. `name_form` records the syntactic form
/// (Bare is the only corpus-proven one; Quoted/Arg are grammar-supported).
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyDecl {
    pub name: String,
    pub name_form: DepNameForm,
    /// `path` prop (to_astr) when declared.
    pub path: Option<String>,
    /// All body props in declaration order (including `path`) with their
    /// parsed values — version-plane keys are preserved for the PLAN-647
    /// guard plane, not enforced here.
    pub props: Vec<(String, Value)>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepNameForm {
    Bare,
    Quoted,
    Arg,
}

/// Node-name syntactic forms (scanner output).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeName {
    Bare(String),
    Quoted(String),
    Arg {
        name: Option<String>,
        raw: Option<String>,
    },
    None,
}

/// Display/desktop-facing fields (PLAN-015 four-name contract surface and
/// registry keys). String fields are trimmed exactly like `Pac::new`;
/// theme/accent/desktop_render additionally lowercase (Pac does; the
/// registry's line reader does not — divergence recorded in the T-04 owner
/// table, the DTO follows Pac).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DisplayFields {
    pub title: Option<String>,
    pub title_zh: Option<String>,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub opens: Option<String>,
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub desktop_render: Option<String>,
    pub daemon: Option<String>,
    pub desktop: Option<String>,
    pub desktop_exe: Option<String>,
    pub ui_config: Option<String>,
}

/// Runtime-facing fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RuntimeFields {
    pub api: Option<String>,
    pub scene: Option<SceneKind>,
    pub front_port: Option<u16>,
    pub back_port: Option<u16>,
    pub media_root: Option<String>,
    pub photo_root: Option<String>,
    pub window: Option<WindowSpec>,
}

/// `window:` declaration (Plan 411/504) — WxH bounds mirror `Pac::new`
/// (200..=7680 × 200..=4320); `"fit"` selects content-fit mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowSpec {
    Size(f32, f32),
    Fit,
}

/// Toggle fields: the config VM evaluates bare `on/off` to Bool while the
/// static parse yields the string form; `normalized()` applies the Pac::new
/// rule set (Bool wins; strings "off"/"false"/"no"/"0" → false; anything
/// else, including absent, → true) so both arms compare equal.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ToggleValue {
    #[default]
    Unset,
    Bool(bool),
    Str(String),
}

impl ToggleValue {
    pub fn normalized(&self) -> bool {
        match self {
            ToggleValue::Unset => true,
            ToggleValue::Bool(b) => *b,
            ToggleValue::Str(s) => {
                !matches!(s.trim().to_lowercase().as_str(), "off" | "false" | "no" | "0")
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToggleFields {
    pub shadcn: ToggleValue,
    pub default_classes: ToggleValue,
}

/// An unmodeled top-level prop — preserved verbatim (key, parsed value when
/// static, source line). Nothing is dropped for being unknown.
#[derive(Debug, Clone, PartialEq)]
pub struct RawProp {
    pub key: String,
    pub value: Option<Value>,
    pub line: usize,
}

/// An unmodeled node (targets: app/lib/bag/device/test, and any unknown
/// head) — head/name/props preserved for the T-04 owner decision; not
/// materialized into build targets by this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct RawNodeDecl {
    pub head: String,
    pub name: Option<String>,
    pub props: Vec<(String, Value)>,
    pub children: Vec<RawNodeDecl>,
    pub line: usize,
}

/// A duplicate top-level key occurrence (later declaration wins for the
/// modeled value, mirroring the VM accumulation and `parse_pac_fields`;
/// the override is recorded, not hidden).
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateKey {
    pub key: String,
    pub line: usize,
}

/// The parsed project descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDescriptor {
    pub source: SourceInfo,
    pub identity: Identity,
    pub render: RenderDecl,
    pub members: Option<Vec<String>>,
    pub defines: Option<Vec<String>>,
    pub deps: Vec<DependencyDecl>,
    pub display: DisplayFields,
    pub runtime: RuntimeFields,
    pub toggles: ToggleFields,
    pub external_backend: Option<String>,
    pub raw_props: Vec<RawProp>,
    pub raw_nodes: Vec<RawNodeDecl>,
    pub duplicates: Vec<DuplicateKey>,
    pub unsupported: Vec<UnsupportedConstruct>,
    /// Source line of each modeled field declaration (field name → line).
    pub provenance: Vec<(&'static str, usize)>,
}

impl ProjectDescriptor {
    /// A file is static-equivalent only when nothing was classified
    /// unsupported (AC-02: dynamic/unknown never fake green).
    pub fn is_static_equivalent(&self) -> bool {
        self.unsupported.is_empty()
    }
}

/// Package identity (name/version mirror `Pac::new` to_astr — no trim).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Identity {
    pub name: Option<String>,
    pub version: Option<String>,
    pub lang: Option<String>,
}

pub(crate) fn parse_window(raw: &str) -> (Option<WindowSpec>, bool) {
    let raw_trimmed = raw.trim();
    if raw_trimmed.eq_ignore_ascii_case("fit") {
        return (Some(WindowSpec::Fit), true);
    }
    let spec = raw_trimmed
        .replace(['x', 'X', '\u{d7}'], "x")
        .split_once('x')
        .and_then(|(w, h)| {
            let w: f32 = w.trim().parse().ok()?;
            let h: f32 = h.trim().parse().ok()?;
            (w >= 200.0 && h >= 200.0 && w <= 7680.0 && h <= 4320.0).then_some((w, h))
        })
        .map(|(w, h)| WindowSpec::Size(w, h));
    (spec, false)
}
