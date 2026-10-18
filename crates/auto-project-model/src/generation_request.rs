//! Generation request data contract (PLAN-809 §5.1 / SD-02).
//!
//! One request carries everything a UI/project generator needs from the
//! project side: origin (static-admitted vs dynamically evaluated), project
//! root, display/toggle/runtime fields, selected backend(s) and mode, output
//! root, package-asset keys (`npm_deps` / `styles` / `i18n`), local dep
//! roots, the light `theme{}` declaration, and sidecar presence — with
//! unknown keys preserved verbatim and no fake evaluation claims.
//!
//! The contract is pure data: constructing or reading a request performs no
//! network access, no directory creation, no compilation. Two entry paths:
//!
//! - [`GenerationRequest::from_descriptor`] — static admission: project the
//!   frozen static subset of a scanned pac.at. A descriptor that is not
//!   static-equivalent keeps its fields but the request reports
//!   [`GenerationRequest::is_evaluated`] == false: statically
//!   non-representable values are NOT silently treated as evaluated
//!   (mirrors the crate's "no fake static equivalence" invariant).
//! - direct construction / [`GenerationRequest::dynamic`] — the config-VM
//!   adapter (auto-man, PLAN-809 T-06) fills final evaluated values.
//!
//! Override precedence is applied by the adapter at construction time
//! (CLI explicit > request-carried declaration > built-in default) — the
//! request itself carries the resulting values plus their origin.
//! Relative paths resolve against the pac.at's own directory
//! ([`GenerationRequest::resolve_against_root`]), never against the
//! process cwd.

use crate::descriptor::{
    DependencyDecl, DisplayFields, Identity, RawNodeDecl, RawProp, RenderDecl, RenderSpec,
    RuntimeFields, SceneKind, SourceInfo, ToggleFields, UnsupportedKind, WindowSpec,
};
use crate::ProjectDescriptor;
use std::path::{Path, PathBuf};

/// Where the request's data came from.
#[derive(Debug, Clone, PartialEq)]
pub enum RequestOrigin {
    /// Projected from a statically scanned pac.at — no evaluation happened.
    /// `unsupported` lists the constructs only the config VM can produce
    /// (empty ⇔ the descriptor was static-equivalent).
    StaticAdmitted {
        source: SourceInfo,
        unsupported: Vec<UnsupportedKind>,
    },
    /// Produced by the config-VM evaluation adapter — fields are final
    /// values (the VM's own accumulation semantics already applied).
    DynamicEvaluated { source: SourceInfo },
}

impl RequestOrigin {
    pub fn source(&self) -> &SourceInfo {
        match self {
            RequestOrigin::StaticAdmitted { source, .. } => source,
            RequestOrigin::DynamicEvaluated { source } => source,
        }
    }
}

/// Selected generation mode (what the consumer is asked to do with the
/// request). Delivery roots are carried by `output_root`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RequestMode {
    /// Generate code/artifacts only (`auto gen`, `--gen-only`).
    #[default]
    Generate,
    /// Generate and run.
    Run,
}

/// Light `theme{}` block declaration — mirrors the shape `Pac::new` extracts
/// from `theme: { … }` (PLAN-601 T-03): extends/mode scalars plus colors as
/// key/value string pairs. Composition (`design_tokens::decl::compose`) is a
/// language-side concern and stays in the adapter.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeDeclLite {
    pub name: Option<String>,
    pub extends: Option<String>,
    pub mode: Option<String>,
    pub colors: Vec<(String, String)>,
}

/// One local dependency root (`dep name { path: "…" }`).
#[derive(Debug, Clone, PartialEq)]
pub struct DepRoot {
    pub name: String,
    /// Declared `path` when present (relative to the pac.at directory).
    pub path: Option<String>,
}

/// Package-asset keys the generators scan from pac.at text today
/// (vue.rs `parse_npm_deps` / `parse_style_files` / `parse_i18n`). The
/// static value is carried as-is (`None` when the declaration is not in the
/// frozen static subset — never faked); the declared-but-unparsed form is
/// preserved on `raw`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackageAssets {
    pub npm_deps: Option<auto_val::Value>,
    pub styles: Option<auto_val::Value>,
    pub i18n: Option<auto_val::Value>,
}

/// The generation request (PLAN-809 "I-PROJECT").
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationRequest {
    pub origin: RequestOrigin,
    /// pac.at's own directory — relative paths in the request resolve
    /// against this, not the process cwd.
    pub project_root: Option<PathBuf>,
    pub identity: Identity,
    pub display: DisplayFields,
    pub runtime: RuntimeFields,
    pub scene: SceneKind,
    pub members: Vec<String>,
    pub render: RenderDecl,
    pub external_backend: Option<String>,
    /// Raw toggle declarations (unknown string forms preserved);
    /// normalized decisions via [`GenerationRequest::shadcn`] /
    /// [`GenerationRequest::default_classes`].
    pub toggles: ToggleFields,
    pub theme_decl: Option<ThemeDeclLite>,
    pub dep_roots: Vec<DepRoot>,
    pub package_assets: PackageAssets,
    /// `exe_name:` declaration (rust_ui `[[bin]]` naming).
    pub exe_name: Option<String>,
    /// `rust_sidecar(…) { … }` node presence (verbatim children preserved
    /// on `raw_nodes`).
    pub has_rust_sidecar: bool,
    /// `front:` path declaration (`app("front")` node form stays on
    /// `raw_nodes` — the adapter resolves it).
    pub front_decl: Option<String>,
    /// Actual input source list (evaluated/discovered inputs; static
    /// admission leaves it empty — not guessable without evaluation).
    pub input_sources: Vec<PathBuf>,
    /// Output root for this generation (delivery root / generator output).
    pub output_root: Option<PathBuf>,
    pub mode: RequestMode,
    /// Unknown top-level keys, preserved verbatim (no silent field loss).
    pub raw: Vec<RawProp>,
    /// Unmodeled nodes, preserved verbatim.
    pub raw_nodes: Vec<RawNodeDecl>,
}

impl GenerationRequest {
    /// Project a statically scanned descriptor into a request. Package-asset
    /// keys, the theme block, exe_name, front and rust_sidecar presence are
    /// recovered from the descriptor's raw catalogue (they are not modeled
    /// fields of the static subset); everything unknown stays on `raw` /
    /// `raw_nodes`.
    pub fn from_descriptor(descriptor: &ProjectDescriptor) -> Self {
        let mut package_assets = PackageAssets::default();
        let mut theme_decl = None;
        let mut exe_name = None;
        let mut front_decl = None;
        let mut raw = Vec::new();
        for prop in &descriptor.raw_props {
            match prop.key.as_str() {
                "npm_deps" => package_assets.npm_deps = prop.value.clone(),
                "styles" => package_assets.styles = prop.value.clone(),
                "i18n" => package_assets.i18n = prop.value.clone(),
                "exe_name" => exe_name = prop.value.as_ref().map(value_to_string),
                "front" => front_decl = prop.value.as_ref().map(value_to_string),
                "theme" => theme_decl = prop.value.as_ref().and_then(theme_decl_from_value),
                _ => raw.push(prop.clone()),
            }
        }
        let has_rust_sidecar = descriptor
            .raw_nodes
            .iter()
            .any(|n| n.head == "rust_sidecar");
        GenerationRequest {
            origin: RequestOrigin::StaticAdmitted {
                source: descriptor.source.clone(),
                unsupported: descriptor.unsupported.iter().map(|u| u.kind).collect(),
            },
            project_root: descriptor
                .source
                .path
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
            identity: descriptor.identity.clone(),
            display: descriptor.display.clone(),
            runtime: descriptor.runtime.clone(),
            scene: descriptor.runtime.scene.unwrap_or_default(),
            members: descriptor.members.clone().unwrap_or_default(),
            render: descriptor.render.clone(),
            external_backend: descriptor.external_backend.clone(),
            toggles: descriptor.toggles.clone(),
            theme_decl,
            dep_roots: dep_roots_from(&descriptor.deps),
            package_assets,
            exe_name,
            has_rust_sidecar,
            front_decl,
            input_sources: Vec::new(),
            output_root: None,
            mode: RequestMode::Generate,
            raw,
            raw_nodes: descriptor.raw_nodes.clone(),
        }
    }

    /// Start a dynamically evaluated request (adapter fills final values).
    pub fn dynamic(source: SourceInfo) -> Self {
        Self {
            project_root: source
                .path
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
            origin: RequestOrigin::DynamicEvaluated { source },
            identity: Identity::default(),
            display: DisplayFields::default(),
            runtime: RuntimeFields::default(),
            scene: SceneKind::default(),
            members: Vec::new(),
            render: RenderDecl {
                render: RenderSpec::Absent,
                legacy_backend: None,
                effective_backend: None,
            },
            external_backend: None,
            toggles: ToggleFields::default(),
            theme_decl: None,
            dep_roots: Vec::new(),
            package_assets: PackageAssets::default(),
            exe_name: None,
            has_rust_sidecar: false,
            front_decl: None,
            input_sources: Vec::new(),
            output_root: None,
            mode: RequestMode::Generate,
            raw: Vec::new(),
            raw_nodes: Vec::new(),
        }
    }

    /// True when the carried values are final (dynamically evaluated, or
    /// statically admitted with nothing unsupported). A static request with
    /// unsupported constructs must not be treated as fully evaluated.
    pub fn is_evaluated(&self) -> bool {
        match &self.origin {
            RequestOrigin::DynamicEvaluated { .. } => true,
            RequestOrigin::StaticAdmitted { unsupported, .. } => unsupported.is_empty(),
        }
    }

    /// Display title with the PLAN-015 locale chain: zh prefers a non-blank
    /// `title_zh`, otherwise falls back to `title`; no declaration → None.
    /// Mirrors `Pac::display_title_in` exactly.
    pub fn display_title(&self, prefers_zh: bool) -> Option<String> {
        let title = self.display.title.clone()?;
        if prefers_zh {
            if let Some(zh) = &self.display.title_zh {
                let zh = zh.trim();
                if !zh.is_empty() {
                    return Some(zh.to_string());
                }
            }
        }
        Some(title)
    }

    /// Frontend backend names in the `Pac::frontend_types` shape: render
    /// fronts when declared; otherwise the legacy single backend when it
    /// parses; empty otherwise (declared-but-unknown stays visible on
    /// `render` — never faked as a known frontend).
    pub fn frontend_names(&self) -> Vec<String> {
        match &self.render.render {
            RenderSpec::Single(name) => vec![name.clone()],
            RenderSpec::Multi(names) => names.clone(),
            RenderSpec::Split { front, .. } => front.clone(),
            RenderSpec::Absent | RenderSpec::Unknown(_) => Vec::new(),
        }
    }

    /// Split-mode backend name, when declared (`Pac::backend_type` shape).
    pub fn backend_name(&self) -> Option<&str> {
        match &self.render.render {
            RenderSpec::Split { back, .. } => Some(back.as_str()),
            _ => None,
        }
    }

    /// `shadcn` normalized decision (Pac::new rule set via ToggleValue).
    pub fn shadcn(&self) -> bool {
        self.toggles.shadcn.normalized()
    }

    /// `default_classes` normalized decision.
    pub fn default_classes(&self) -> bool {
        self.toggles.default_classes.normalized()
    }

    /// Startup window size (`Pac.window` shape; None for fit/absent/invalid).
    pub fn window_size(&self) -> Option<(f32, f32)> {
        match self.runtime.window {
            Some(WindowSpec::Size(w, h)) => Some((w, h)),
            _ => None,
        }
    }

    /// Content-fit window mode (`Pac.window_fit` shape).
    pub fn window_fit(&self) -> bool {
        matches!(self.runtime.window, Some(WindowSpec::Fit))
    }

    /// Resolve a request-carried relative path against the project root
    /// (absolute paths pass through) — pure path algebra, no FS access and
    /// no cwd dependence. None when the request carries no root.
    pub fn resolve_against_root(&self, rel: &str) -> Option<PathBuf> {
        let p = Path::new(rel);
        if p.is_absolute() {
            return Some(p.to_path_buf());
        }
        self.project_root.as_ref().map(|root| root.join(p))
    }
}

fn value_to_string(value: &auto_val::Value) -> String {
    value.to_astr().trim().to_string()
}

fn theme_decl_from_value(value: &auto_val::Value) -> Option<ThemeDeclLite> {
    let obj = match value {
        auto_val::Value::Obj(obj) => obj,
        _ => return None,
    };
    let scalar = |key: &str| {
        obj.get(key)
            .map(|v| v.to_astr().trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let colors = match obj.get("colors") {
        Some(auto_val::Value::Obj(colors)) => colors
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_astr().trim().to_string()))
            .filter(|(_, v)| !v.is_empty())
            .collect(),
        _ => Vec::new(),
    };
    Some(ThemeDeclLite {
        name: scalar("name"),
        extends: scalar("extends"),
        mode: scalar("mode").map(|m| m.to_lowercase()),
        colors,
    })
}

fn dep_roots_from(deps: &[DependencyDecl]) -> Vec<DepRoot> {
    deps.iter()
        .map(|d| DepRoot {
            name: d.name.clone(),
            path: d.path.clone(),
        })
        .collect()
}
