//! PLAN-815 T-05 work-operation implementations: `generate_ui` and `run_ui`
//! over the real (migrated) generation/run paths. Every gate — served
//! profile, validation findings, project root — is answered with a named
//! error BEFORE any side effect; the execution itself is the real path, not
//! a projection of one.

use std::path::{Path, PathBuf};

use auto_project_model::generation_request::GenerationRequest;
use auto_tool_protocol::report::{BuildReport, GenerationReport, RunOutcome, RunReport};
use auto_tool_protocol::ToolError;

use crate::host::{BUILD_UI_SERVED_PROFILES, GENERATE_UI_SERVED_PROFILES, RUN_UI_SERVED_PROFILES};
use crate::proc_host;

/// `generate_ui`: real Vue generation for the request into the project's
/// generation root (the same step-1 semantics the vue run path applies:
/// full generate for fresh projects, self-heal for existing ones).
pub fn generate_report(request: &GenerationRequest) -> Result<GenerationReport, ToolError> {
    // ---- Gates (before any side effect) ----
    let frontends = request.frontend_names();
    let unserved: Vec<&String> = frontends
        .iter()
        .filter(|name| !GENERATE_UI_SERVED_PROFILES.contains(&name.as_str()))
        .collect();
    if frontends.is_empty() || !unserved.is_empty() {
        return Err(ToolError::invalid_field(
            "request",
            format!(
                "generate_ui serves {GENERATE_UI_SERVED_PROFILES:?} in this build; request declares {frontends:?}"
            ),
        ));
    }
    gate_on_validation(request)?;
    let root = project_root(request)?;
    let profile = frontends[0].clone();

    // ---- Real generation (dispatch by profile) ----
    proc_host::progress(1, Some(3), "incremental compile");
    match profile.as_str() {
        "vue" => {
            crate::vue::incremental_compile_changed(&root, Some(request))
                .map_err(|e| ToolError::internal(format!("incremental compile failed: {e}")))?;
            proc_host::progress(2, Some(3), "generate project");
            let project = crate::vue::VueProject::from_workspace_with_request(&root, request)
                .map_err(|e| ToolError::internal(format!("project context failed: {e}")))?;
            if !project.exists() {
                project
                    .generate()
                    .map_err(|e| ToolError::internal(format!("generation failed: {e}")))?;
            } else {
                project
                    .ensure_router_file()
                    .map_err(|e| ToolError::internal(format!("self-heal failed: {e}")))?;
                project
                    .ensure_code_editor_component()
                    .map_err(|e| ToolError::internal(format!("self-heal failed: {e}")))?;
            }
        }
        "rust" => {
            crate::rust_ui::generate_rust_ui_with_request(&root, None, true, request)
                .map_err(|e| ToolError::internal(format!("rust_ui generation failed: {e}")))?;
        }
        "jet" => {
            crate::jet::generate_jet_project_with_request(&root, None, true, request)
                .map_err(|e| ToolError::internal(format!("jet generation failed: {e}")))?;
        }
        "ark" => {
            crate::ark::generate_ark_project_with_request(&root, None, true, request)
                .map_err(|e| ToolError::internal(format!("ark generation failed: {e}")))?;
        }
        "tauri" => {
            // Tauri frontends scaffold via the Vue generator; the tauri
            // backend config lands via generate_tauri_backend (same crate).
            crate::vue::incremental_compile_changed(&root, Some(request))
                .map_err(|e| ToolError::internal(format!("incremental compile failed: {e}")))?;
            let project = crate::vue::VueProject::from_workspace_with_request(&root, request)
                .map_err(|e| ToolError::internal(format!("project context failed: {e}")))?;
            if !project.exists() {
                project.generate().map_err(|e| {
                    ToolError::internal(format!("tauri frontend generation failed: {e}"))
                })?;
            }
            crate::tauri_backend::generate_tauri_backend(&root).map_err(|e| {
                ToolError::internal(format!("tauri backend generation failed: {e}"))
            })?;
        }
        "vscode" => {
            crate::vscode::generate_vscode_project(&root, None, true)
                .map_err(|e| ToolError::internal(format!("vscode generation failed: {e}")))?;
        }
        other => {
            return Err(ToolError::invalid_field(
                "request",
                format!("render '{other}' is listed as served but has no dispatch arm"),
            ))
        }
    }

    proc_host::progress(3, Some(3), "count artifacts");
    if proc_host::cancelled() {
        return Err(ToolError::internal("operation aborted before completion"));
    }
    // The generator reports a project-relative output dir; the walk anchor
    // is the request's project_root (never the tool's cwd — R1-F04 rule).
    let output_root = generation_output_root(&root, &profile);
    let files_written = count_files(&output_root);
    Ok(GenerationReport {
        output_root: output_root.to_string_lossy().to_string(),
        frontends,
        files_written,
        duration_ms: 0, // filled by the worker
    })
}

/// `build_ui` (T-06): dispatch the request's render profile onto the real
/// build path.
pub fn build_report(request: &GenerationRequest) -> Result<BuildReport, ToolError> {
    // ---- Gates ----
    let frontends = request.frontend_names();
    let profile = frontends
        .iter()
        .find(|name| BUILD_UI_SERVED_PROFILES.contains(&name.as_str()))
        .cloned();
    let Some(profile) = profile else {
        return Err(ToolError::invalid_field(
            "request",
            format!(
                "build_ui serves {BUILD_UI_SERVED_PROFILES:?} in this build (vm is interpreted — no build step); request declares {frontends:?}"
            ),
        ));
    };
    gate_on_validation(request)?;
    let root = project_root(request)?;

    // ---- Real build ----
    proc_host::progress(1, Some(2), "build project");
    let flow: Result<(), auto_val::AutoError> = match profile.as_str() {
        "vue" => crate::vue::build_vue_project(&root).map_err(|e| e.into()),
        "rust" => crate::rust_ui::build_rust_ui_with_request(&root, request).map_err(|e| e.into()),
        "jet" => crate::jet::build_jet_project_with_request(&root, request).map_err(|e| e.into()),
        "ark" => crate::ark::build_ark_project_with_request(&root, request).map_err(|e| e.into()),
        "vscode" => crate::vscode::build_vscode_project(&root).map_err(|e| e.into()),
        other => {
            return Err(ToolError::invalid_field(
                "request",
                format!("render '{other}' is listed as served but has no build arm"),
            ))
        }
    };

    let (outcome, error) = map_flow(flow.map(|_| crate::proc_host::FlowEnd::Completed));
    Ok(BuildReport {
        profile,
        outcome,
        duration_ms: 0,
        launch: None,
        error,
    })
}

/// The `run_ui` dispatch (T-06: full matrix): every profile's real run path.
pub fn run_report(request: &GenerationRequest) -> Result<RunReport, ToolError> {
    // ---- Gates (before any side effect) ----
    let frontends = request.frontend_names();
    let profile = frontends
        .iter()
        .find(|name| RUN_UI_SERVED_PROFILES.contains(&name.as_str()))
        .cloned();
    let Some(profile) = profile else {
        return Err(ToolError::invalid_field(
            "request",
            format!(
                "run_ui serves {RUN_UI_SERVED_PROFILES:?} in this build; request declares {frontends:?}"
            ),
        ));
    };
    gate_on_validation(request)?;
    let root = project_root(request)?;

    // ---- Real run (dispatch by profile) ----
    let flow: Result<proc_host::FlowEnd, auto_val::AutoError> = match profile.as_str() {
        "vue" => crate::run_vue::run_vue_flow(&root, Vec::new(), Some(request)),
        "vm" => crate::run_vm::run_vm_ui_flow(
            &root,
            Vec::new(),
            &crate::rust_ui::PacSource::from_request(request),
        ),
        "rust" => {
            // automan's rule: a rust-render project is a UI project iff it
            // has src/front; plain rust projects are not UI execution.
            if !root.join("src").join("front").exists() {
                return Err(ToolError::invalid_field(
                    "request",
                    "render 'rust' without src/front is not a UI project",
                ));
            }
            crate::rust_ui::run_rust_ui_with_request(&root, Vec::new(), request)
                .map(|_| proc_host::FlowEnd::Completed)
        }
        "jet" => crate::jet::run_jet_project_with_request(&root, Vec::new(), request)
            .map(|_| proc_host::FlowEnd::Completed),
        "ark" => crate::ark::run_ark_project_with_request(&root, Vec::new(), request)
            .map(|_| proc_host::FlowEnd::Completed),
        "tauri" => crate::tauri::run_tauri_project_with_request(&root, Vec::new(), request)
            .map(|_| proc_host::FlowEnd::Completed),
        "vscode" => crate::vscode::run_vscode_project(&root, Vec::new())
            .map(|_| proc_host::FlowEnd::Completed),
        other => {
            return Err(ToolError::invalid_field(
                "request",
                format!("render '{other}' is listed as served but has no dispatch arm"),
            ))
        }
    };

    let (outcome, error) = map_flow(flow);
    Ok(RunReport {
        profile,
        outcome,
        duration_ms: 0, // filled by the worker
        launch: None,   // filled by the worker from the launched cell
        error,
    })
}

/// Map a flow result to (outcome, error), recognizing cancellations.
fn map_flow(flow: Result<proc_host::FlowEnd, auto_val::AutoError>) -> (RunOutcome, Option<String>) {
    match flow {
        Ok(proc_host::FlowEnd::Completed) => (RunOutcome::Completed, None),
        Ok(proc_host::FlowEnd::Cancelled) => (RunOutcome::Cancelled, None),
        Err(e) => {
            let text = e.to_string();
            // F-815-T05-R1-05: a cancelled flow is Cancelled regardless of
            // which sentinel surfaced — a plain failure stays Failed.
            if proc_host::cancelled() || proc_host::cancelled_err(&text) {
                (RunOutcome::Cancelled, None)
            } else {
                (RunOutcome::Failed, Some(text))
            }
        }
    }
}

/// Resolve the generation output root for the walk oracle (profile-aware).
fn generation_output_root(root: &Path, profile: &str) -> PathBuf {
    match profile {
        "vue" | "tauri" => root.join("gen").join("front").join("vue"),
        "rust" => {
            // rust_ui generates into the shared workspace member.
            let ws = crate::rust_ui::resolve_rust_workspace_dir(root);
            let member = crate::rust_ui::front_member_name(root);
            ws.join(member)
        }
        "jet" => root.join("android"),
        "ark" => root.join("ohos"),
        "vscode" => root.join("gen").join("vscode"),
        _ => root.to_path_buf(),
    }
}

fn gate_on_validation(request: &GenerationRequest) -> Result<(), ToolError> {
    let report = crate::validate::validate_request(request);
    if report.ok {
        return Ok(());
    }
    let errors: Vec<String> = report
        .findings
        .iter()
        .filter(|f| f.severity == auto_tool_protocol::Severity::Error)
        .map(|f| format!("{}: {}", f.code, f.message))
        .collect();
    Err(ToolError::invalid_field(
        "request",
        format!("request failed validation: {errors:?}"),
    ))
}

/// Absolute project root (the I-PROJECT anchor; validate guarantees it when
/// present and existing — the gate ordering makes the unwrap honest).
fn project_root(request: &GenerationRequest) -> Result<PathBuf, ToolError> {
    match &request.project_root {
        Some(root) if root.is_absolute() => Ok(root.clone()),
        _ => Err(ToolError::invalid_field(
            "project_root",
            "work operations require an absolute project_root",
        )),
    }
}

/// Count files under `root` recursively (generation oracle: measured on
/// disk, not the generator's self-report).
fn count_files(root: &Path) -> u64 {
    let mut count = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                count += 1;
            }
        }
    }
    count
}
