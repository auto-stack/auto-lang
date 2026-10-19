//! `validate_project` — I-PROJECT metadata / path / request-constraint
//! checks (PLAN-815 §5.5). Read-only: existence probes via `std::fs`
//! metadata, no writes, no evaluation, no side effects.
//!
//! Honesty contract: a passing report says the *request* is well-formed and
//! its declared paths exist — nothing more. [`NOT_CHECKED_SCOPES`] is echoed
//! into every report so no consumer can mistake this for a compile or
//! generation success.

use auto_project_model::generation_request::RequestOrigin;
use auto_project_model::GenerationRequest;
use auto_tool_protocol::report::{Finding, RequestSummary, Severity, ValidateProjectReport};
use std::path::Path;

/// What validation actually covers.
pub const CHECKED_SCOPES: &[&str] = &["metadata", "paths", "request-constraints"];
/// What it explicitly does NOT cover — echoed into every report.
pub const NOT_CHECKED_SCOPES: &[&str] = &[
    "language-parse",
    "dynamic-evaluation",
    "generation",
    "build",
];

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
    if let Some(output_root) = &request.output_root {
        if output_root.is_file() {
            findings.push(
                Finding::new(
                    Severity::Error,
                    "output-root-is-file",
                    "output_root exists but is a regular file, not a directory",
                )
                .with_field("output_root"),
            );
        }
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

    // --- path existence (read-only probes) -----------------------------------
    if let Some(root) = &request.project_root {
        if !root.exists() {
            findings.push(
                Finding::new(
                    Severity::Error,
                    "project-root-missing",
                    format!("declared project root does not exist: {}", root.display()),
                )
                .with_field("project_root"),
            );
        }
    }
    for source in &request.input_sources {
        if !source.exists() {
            findings.push(
                Finding::new(
                    Severity::Error,
                    "input-source-missing",
                    format!("declared evaluated input source does not exist: {}", source.display()),
                )
                .with_field("input_sources"),
            );
        }
    }
    for dep in &request.dep_roots {
        if let Some(path) = &dep.path {
            let resolved = request
                .resolve_against_root(path)
                .unwrap_or_else(|| Path::new(path).to_path_buf());
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
            }
        }
    }
    let front = request
        .front_node_form
        .as_deref()
        .or(request.front_decl.as_deref());
    if let Some(front) = front {
        if let Some(resolved) = request.resolve_against_root(front) {
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
            }
        }
    }

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
