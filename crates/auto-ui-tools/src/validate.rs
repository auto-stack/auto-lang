//! `validate_project` — I-PROJECT metadata / path / request-constraint
//! checks (PLAN-815 §5.5). Read-only: existence/role probes via `std::fs`
//! metadata, no writes, no evaluation, no side effects.
//!
//! Path discipline (review R1 F-04): relative paths in the request resolve
//! against the request's own `project_root` — never the tool process's
//! cwd. A relative path that cannot be anchored (no project root, or the
//! root itself is relative) is reported as unverifiable instead of being
//! silently probed against whatever directory the tool happens to start
//! in; role mismatches (file where a directory is declared, and vice
//! versa) are findings, not accidents.
//!
//! Honesty contract: a passing report says the *request* is well-formed and
//! its declared paths exist — nothing more. [`NOT_CHECKED_SCOPES`] is echoed
//! into every report so no consumer can mistake this for a compile or
//! generation success.

use auto_project_model::generation_request::RequestOrigin;
use auto_project_model::GenerationRequest;
use auto_tool_protocol::report::{Finding, RequestSummary, Severity, ValidateProjectReport};
use std::path::{Path, PathBuf};

/// What validation actually covers.
pub const CHECKED_SCOPES: &[&str] = &["metadata", "paths", "request-constraints"];
/// What it explicitly does NOT cover — echoed into every report.
pub const NOT_CHECKED_SCOPES: &[&str] = &[
    "language-parse",
    "dynamic-evaluation",
    "generation",
    "build",
];

/// How a declared path was anchored for probing.
enum Anchored {
    /// Absolute, or a purely relative path joined onto the request's
    /// absolute project root (the join result is verified absolute).
    Resolved(PathBuf),
    /// Cannot be anchored against request data. Never probed against the
    /// tool process's working directory. `cwd_qualified` marks Windows
    /// drive-relative (`C:dir`) / rooted-without-prefix (`\dir`) forms:
    /// `Path::join` keeps them drive-relative, so any probe would use the
    /// process's per-drive current directory (review R2, F-815-R1-04).
    NoAnchor { raw: PathBuf, cwd_qualified: bool },
}

/// True when the path is relative in the *pure* sense: no Windows prefix
/// and no root component — the only form `root.join(p)` anchors correctly
/// on every platform.
fn is_purely_relative(path: &Path) -> bool {
    !path.components().any(|c| {
        matches!(
            c,
            std::path::Component::Prefix(_) | std::path::Component::RootDir
        )
    })
}

fn anchor(request: &GenerationRequest, raw: &Path) -> Anchored {
    if raw.is_absolute() {
        return Anchored::Resolved(raw.to_path_buf());
    }
    if !is_purely_relative(raw) {
        return Anchored::NoAnchor {
            raw: raw.to_path_buf(),
            cwd_qualified: true,
        };
    }
    match &request.project_root {
        Some(root) if root.is_absolute() => {
            let joined = root.join(raw);
            // belt and braces: a join that somehow stays relative can
            // never be probed cwd-free
            if joined.is_absolute() {
                Anchored::Resolved(joined)
            } else {
                Anchored::NoAnchor {
                    raw: raw.to_path_buf(),
                    cwd_qualified: true,
                }
            }
        }
        _ => Anchored::NoAnchor {
            raw: raw.to_path_buf(),
            cwd_qualified: false,
        },
    }
}

/// Validate one request.
pub fn validate_request(request: &GenerationRequest) -> ValidateProjectReport {
    let mut findings = Vec::new();

    // --- request constraints ------------------------------------------------
    if request.mode == auto_project_model::RequestMode::Run && request.output_root.is_none() {
        findings.push(
            Finding::new(
                Severity::Error,
                "run-without-output-root",
                "mode 'run' carries no output_root — a runner would have nowhere to deliver",
            )
            .with_field("output_root"),
        );
    }

    // --- origin honesty ------------------------------------------------------
    if let RequestOrigin::StaticAdmitted { unsupported, .. } = &request.origin {
        if !unsupported.is_empty() {
            findings.push(
                Finding::new(
                    Severity::Warning,
                    "static-unsupported-constructs",
                    format!(
                        "request was statically admitted with {} construct(s) only the config \
                         VM can evaluate; validate_project does not evaluate them",
                        unsupported.len()
                    ),
                )
                .with_field("origin"),
            );
        }
    }

    // --- project root: the anchor for every other relative path ---------------
    let root_usable = match &request.project_root {
        None => None,
        Some(root) if !root.is_absolute() => {
            findings.push(
                Finding::new(
                    Severity::Error,
                    "project-root-not-absolute",
                    format!(
                        "project_root '{}' is relative; relative paths resolve against the \
                         request's project root, so the root itself must be absolute",
                        root.display()
                    ),
                )
                .with_field("project_root"),
            );
            None
        }
        Some(root) => {
            if !root.exists() {
                findings.push(
                    Finding::new(
                        Severity::Error,
                        "project-root-missing",
                        format!("declared project root does not exist: {}", root.display()),
                    )
                    .with_field("project_root"),
                );
                None
            } else if root.is_file() {
                findings.push(
                    Finding::new(
                        Severity::Error,
                        "project-root-is-file",
                        format!(
                            "declared project root exists but is a regular file: {}",
                            root.display()
                        ),
                    )
                    .with_field("project_root"),
                );
                None
            } else {
                Some(root.as_path())
            }
        }
    };

    // --- evaluated input sources (load-bearing: error severity) ---------------
    for source in &request.input_sources {
        match anchor(request, source) {
            Anchored::Resolved(path) => {
                if !path.exists() {
                    findings.push(
                        Finding::new(
                            Severity::Error,
                            "input-source-missing",
                            format!(
                                "declared evaluated input source does not exist: {}",
                                path.display()
                            ),
                        )
                        .with_field("input_sources"),
                    );
                } else if path.is_dir() {
                    findings.push(
                        Finding::new(
                            Severity::Error,
                            "input-source-is-directory",
                            format!(
                                "declared evaluated input source is a directory, expected a file: {}",
                                path.display()
                            ),
                        )
                        .with_field("input_sources"),
                    );
                }
            }
            Anchored::NoAnchor { raw, cwd_qualified } => findings.push(
                Finding::new(
                    Severity::Warning,
                    if cwd_qualified {
                        "cwd-qualified-path"
                    } else {
                        "relative-path-without-root"
                    },
                    format!(
                        "relative input source '{}' cannot be verified: the request carries no \
                         project root (never probed against the tool's working directory)",
                        raw.display()
                    ),
                )
                .with_field("input_sources"),
            ),
        }
    }

    // --- output root (delivery target; only role errors are findings) ---------
    if let Some(output_root) = &request.output_root {
        match anchor(request, output_root) {
            Anchored::Resolved(path) => {
                if path.is_file() {
                    findings.push(
                        Finding::new(
                            Severity::Error,
                            "output-root-is-file",
                            format!(
                                "output_root exists but is a regular file, not a directory: {}",
                                path.display()
                            ),
                        )
                        .with_field("output_root"),
                    );
                }
            }
            Anchored::NoAnchor { raw, cwd_qualified } => findings.push(
                Finding::new(
                    Severity::Warning,
                    if cwd_qualified {
                        "cwd-qualified-path"
                    } else {
                        "relative-path-without-root"
                    },
                    format!(
                        "relative output_root '{}' cannot be verified: the request carries no \
                         project root (never probed against the tool's working directory)",
                        raw.display()
                    ),
                )
                .with_field("output_root"),
            ),
        }
    }

    // --- dependency roots / front directory (advisory: warning severity) ------
    for dep in &request.dep_roots {
        if let Some(path) = &dep.path {
            match anchor(request, Path::new(path)) {
                Anchored::Resolved(resolved) => {
                    if !resolved.exists() {
                        findings.push(
                            Finding::new(
                                Severity::Warning,
                                "dep-root-missing",
                                format!(
                                    "declared dependency root '{}' resolves to a missing path: {}",
                                    dep.name,
                                    resolved.display()
                                ),
                            )
                            .with_field("dep_roots"),
                        );
                    } else if resolved.is_file() {
                        findings.push(
                            Finding::new(
                                Severity::Warning,
                                "dep-root-is-file",
                                format!(
                                    "declared dependency root '{}' resolves to a regular file: {}",
                                    dep.name,
                                    resolved.display()
                                ),
                            )
                            .with_field("dep_roots"),
                        );
                    }
                }
                Anchored::NoAnchor { raw, cwd_qualified } => findings.push(
                    Finding::new(
                        Severity::Warning,
                        if cwd_qualified {
                            "cwd-qualified-path"
                        } else {
                            "relative-path-without-root"
                        },
                        format!(
                            "dependency root '{}' cannot be verified against request data \
                             (cwd-qualified path forms are never probed against the tool's \
                             working directory)",
                            raw.display()
                        ),
                    )
                    .with_field("dep_roots"),
                ),
            }
        }
    }
    let front = request
        .front_node_form
        .as_deref()
        .or(request.front_decl.as_deref());
    if let Some(front) = front {
        match anchor(request, Path::new(front)) {
            Anchored::Resolved(resolved) => {
                if !resolved.exists() {
                    findings.push(
                        Finding::new(
                            Severity::Warning,
                            "front-directory-missing",
                            format!(
                                "declared front directory does not exist: {}",
                                resolved.display()
                            ),
                        )
                        .with_field("front"),
                    );
                } else if resolved.is_file() {
                    findings.push(
                        Finding::new(
                            Severity::Warning,
                            "front-is-file",
                            format!(
                                "declared front directory exists but is a regular file: {}",
                                resolved.display()
                            ),
                        )
                        .with_field("front"),
                    );
                }
            }
            Anchored::NoAnchor { raw, cwd_qualified } => findings.push(
                Finding::new(
                    Severity::Warning,
                    if cwd_qualified {
                        "cwd-qualified-path"
                    } else {
                        "relative-path-without-root"
                    },
                    format!(
                        "relative front directory '{}' cannot be verified: the request carries \
                         no project root",
                        raw.display()
                    ),
                )
                .with_field("front"),
            ),
        }
    }
    let _ = root_usable; // root usability is folded into the findings above

    let ok = !findings.iter().any(|f| f.severity == Severity::Error);
    ValidateProjectReport {
        ok,
        findings,
        summary: summarize(request),
    }
}

fn summarize(request: &GenerationRequest) -> RequestSummary {
    RequestSummary {
        name: request.identity.name.clone(),
        origin: match &request.origin {
            RequestOrigin::StaticAdmitted { .. } => "static".into(),
            RequestOrigin::DynamicEvaluated { .. } => "dynamic".into(),
        },
        is_evaluated: request.is_evaluated(),
        frontends: request.frontend_names(),
        mode: match request.mode {
            auto_project_model::RequestMode::Generate => "generate".into(),
            auto_project_model::RequestMode::Run => "run".into(),
        },
        window: request.window_size(),
        members: request.members.len(),
        dep_roots: request.dep_roots.len(),
        raw_props: request.raw.len(),
        raw_nodes: request.raw_nodes.len(),
        input_sources: request.input_sources.len(),
        has_output_root: request.output_root.is_some(),
        has_theme_decl: request.theme_decl.is_some(),
        package_assets_present: [
            request.package_assets.npm_deps.is_some(),
            request.package_assets.styles.is_some(),
            request.package_assets.i18n.is_some(),
        ],
        checked: CHECKED_SCOPES.iter().map(|s| s.to_string()).collect(),
        not_checked: NOT_CHECKED_SCOPES.iter().map(|s| s.to_string()).collect(),
    }
}
