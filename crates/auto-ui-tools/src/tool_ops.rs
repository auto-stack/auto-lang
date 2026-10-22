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

/// Normalize a declared frontend name onto the generator/profile name it
/// dispatches to (PLAN-815 R4-07 支持校验补全)：descriptor 解析把 `ark`
/// 规范化为 canonical `arkts`（from_scan 双名同源），而生成器/服务集的
/// profile 名是 `ark`——在工具边界统一到生成器名，真实 pac 声明
/// （`render: "ark"`）与 wire 直带 `ark` 两条路都能服务。
fn normalize_profile(name: &str) -> &str {
    match name {
        "arkts" => "ark",
        other => other,
    }
}

/// `generate_ui`: real generation for the request into the project's
/// generation root — EVERY declared (served) frontend, in declared order
/// (PLAN-815 R4-07 / F-815-T05-R4-07: a multi-profile request generates
/// all targets; the report never claims a frontend that did not run).
pub fn generate_report(request: &GenerationRequest) -> Result<GenerationReport, ToolError> {
    // ---- Gates (before any side effect) ----
    let frontends: Vec<String> = request
        .frontend_names()
        .iter()
        .map(|name| normalize_profile(name).to_string())
        .collect();
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
    // PLAN-815 R5（F-815-T05-R4-04）：本操作的包管理器单源=当前请求
    // （raw `pkg:` 字段）——guard 在操作结束/早退时恢复 legacy 形态。
    let _pm_guard = crate::pkg::set_tool_pm_for_request(request);

    // Unique profiles in declared order (a repeated frontend generates once;
    // `frontends` on the report echoes the declaration verbatim).
    let mut ordered: Vec<String> = Vec::new();
    for name in &frontends {
        if !ordered.contains(name) {
            ordered.push(name.clone());
        }
    }
    let total = ordered.len() as u64;

    // ---- Real generation (dispatch by profile) ----
    let mut output_dirs: Vec<PathBuf> = Vec::new();
    for (idx, profile) in ordered.iter().enumerate() {
        proc_host::progress(
            idx as u64 + 1,
            Some(total + 1),
            &format!("generate {profile}"),
        );
        generate_profile(&root, request, profile)?;
        // The walk anchor is each profile's REAL layout root (the same
        // constants the generators write to — never a guessed dir,
        // PLAN-815 R4-03 / F-815-T05-R4-03), deduped across profiles
        // (tauri shares gen/front/vue with a plain vue target).
        for dir in profile_output_dirs(&root, profile) {
            if !output_dirs.contains(&dir) {
                output_dirs.push(dir);
            }
        }
        if proc_host::cancelled() {
            return Err(ToolError::internal("operation aborted before completion"));
        }
    }

    proc_host::progress(total + 1, Some(total + 1), "count artifacts");
    let (output_root, files_written) = aggregate_output_root(&root, &output_dirs);
    Ok(GenerationReport {
        output_root: output_root.to_string_lossy().to_string(),
        frontends,
        files_written,
        duration_ms: 0, // filled by the worker
    })
}

/// The real generation arm for ONE profile (shared by the multi-target
/// loop above).
fn generate_profile(
    root: &Path,
    request: &GenerationRequest,
    profile: &str,
) -> Result<(), ToolError> {
    match profile {
        "vue" => {
            crate::vue::incremental_compile_changed(root, Some(request))
                .map_err(|e| ToolError::internal(format!("incremental compile failed: {e}")))?;
            let project = crate::vue::VueProject::from_workspace_with_request(root, request)
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
            crate::rust_ui::generate_rust_ui_with_request(root, None, true, request)
                .map_err(|e| ToolError::internal(format!("rust_ui generation failed: {e}")))?;
        }
        "jet" => {
            crate::jet::generate_jet_project_with_request(root, None, true, request)
                .map_err(|e| ToolError::internal(format!("jet generation failed: {e}")))?;
        }
        "ark" => {
            crate::ark::generate_ark_project_with_request(root, None, true, request)
                .map_err(|e| ToolError::internal(format!("ark generation failed: {e}")))?;
        }
        "tauri" => {
            // Tauri frontends scaffold via the Vue generator; the tauri
            // backend config lands via generate_tauri_backend (same crate).
            crate::vue::incremental_compile_changed(root, Some(request))
                .map_err(|e| ToolError::internal(format!("incremental compile failed: {e}")))?;
            let project = crate::vue::VueProject::from_workspace_with_request(root, request)
                .map_err(|e| ToolError::internal(format!("project context failed: {e}")))?;
            if !project.exists() {
                project.generate().map_err(|e| {
                    ToolError::internal(format!("tauri frontend generation failed: {e}"))
                })?;
            }
            crate::tauri_backend::generate_tauri_backend(root).map_err(|e| {
                ToolError::internal(format!("tauri backend generation failed: {e}"))
            })?;
        }
        "vscode" => {
            // PLAN-815 R4-04（F-815-T05-R4-04）：消费已求值请求——不再经
            // legacy 主 pac 自扫（vscode 第二真源反例的根修）。
            crate::vscode::generate_vscode_project_with_request(root, None, true, request)
                .map_err(|e| ToolError::internal(format!("vscode generation failed: {e}")))?;
        }
        other => {
            return Err(ToolError::invalid_field(
                "request",
                format!("render '{other}' is listed as served but has no dispatch arm"),
            ))
        }
    }
    Ok(())
}

/// `build_ui` (T-06): dispatch the request's render profile onto the real
/// build path.
pub fn build_report(request: &GenerationRequest) -> Result<BuildReport, ToolError> {
    // ---- Gates ----
    let frontends = request.frontend_names();
    let profile = frontends
        .iter()
        .map(|name| normalize_profile(name))
        .find(|name| BUILD_UI_SERVED_PROFILES.contains(name))
        .map(str::to_string);
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
    // PLAN-815 R5（F-815-T05-R4-04）：包管理器单源=当前请求（见
    // set_tool_pm_for_request；guard 恢复 legacy 形态）。
    let _pm_guard = crate::pkg::set_tool_pm_for_request(request);

    // ---- Real build ----
    proc_host::progress(1, Some(2), "build project");
    let flow: Result<(), auto_val::AutoError> = match profile.as_str() {
        // PLAN-815 R4-04：build 链消费已求值请求（vue/vscode 此前走
        // legacy 主 pac 自扫=第二真源——F-815-T05-R4-04 根修）。
        "vue" => crate::vue::build_vue_project_with_request(&root, request).map_err(|e| e.into()),
        "rust" => crate::rust_ui::build_rust_ui_with_request(&root, request).map_err(|e| e.into()),
        "jet" => crate::jet::build_jet_project_with_request(&root, request).map_err(|e| e.into()),
        "ark" => crate::ark::build_ark_project_with_request(&root, request).map_err(|e| e.into()),
        "vscode" => {
            crate::vscode::build_vscode_project_with_request(&root, request).map_err(|e| e.into())
        }
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
        .map(|name| normalize_profile(name))
        .find(|name| RUN_UI_SERVED_PROFILES.contains(name))
        .map(str::to_string);
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
    // PLAN-815 R5（F-815-T05-R4-04）：包管理器单源=当前请求（见
    // set_tool_pm_for_request；guard 恢复 legacy 形态）。
    let _pm_guard = crate::pkg::set_tool_pm_for_request(request);

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
            // has a front source dir; plain rust projects are not UI
            // execution. PLAN-815 T-06 R1（F-815-T06-R1-01）：判定消费
            // 请求声明的前端目录（无声明=默认 src/front 正控保持）。
            let front_dir = crate::util::request_front_dir(&root, request)
                .unwrap_or_else(|| root.join("src").join("front"));
            if !front_dir.exists() {
                return Err(ToolError::invalid_field(
                    "request",
                    format!(
                        "render 'rust' without a front source dir ({}) is not a UI project",
                        front_dir.display()
                    ),
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
        "vscode" => crate::vscode::run_vscode_project_with_request(&root, Vec::new(), request)
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
        Ok(proc_host::FlowEnd::Completed) => {
            // PLAN-815 R5（F-815-T05-R4-05-P1）：取消/EOF 之后到达的 Ok
            // 不是 completed——操作的执行已被取消打断（例如捕获排水在
            // 停止后返回的 Ok 路径），终态必须诚实为 cancelled。
            if proc_host::cancelled() {
                (RunOutcome::Cancelled, None)
            } else {
                (RunOutcome::Completed, None)
            }
        }
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

/// Real per-profile generation output roots (PLAN-815 R4-03 /
/// F-815-T05-R4-03): the SAME layout constants the generators write to —
/// vue/jet/ark/vscode under `gen/front/<profile>`; **tauri** covers BOTH
/// halves (Vue frontend in `gen/front/vue` + tauri backend in
/// `gen/back/rust` — the multi-output boundary); rust lands in the shared
/// workspace member. The report walks these roots; a root that does not
/// exist counts zero files honestly.
pub fn profile_output_dirs(root: &Path, profile: &str) -> Vec<PathBuf> {
    let gen_front = |name: &str| root.join("gen").join("front").join(name);
    match profile {
        "vue" => vec![gen_front("vue")],
        "jet" => vec![gen_front("jet")],
        "ark" => vec![gen_front("ark")],
        "vscode" => vec![gen_front("vscode")],
        "tauri" => vec![gen_front("vue"), root.join("gen").join("back").join("rust")],
        "rust" => {
            // rust_ui generates into the shared workspace member.
            let ws = crate::rust_ui::resolve_rust_workspace_dir(root);
            vec![ws.join(crate::rust_ui::front_member_name(root))]
        }
        _ => Vec::new(),
    }
}

/// Aggregate the walk oracle: `files_written` counts the deduped union of
/// the real per-profile roots; the reported `output_root` is the single
/// root when one, else the deepest shared ancestor (tauri → `gen/`,
/// [vue, jet] → `gen/front`; mixed roots outside `gen/` fall back to the
/// project root).
pub fn aggregate_output_root(root: &Path, dirs: &[PathBuf]) -> (PathBuf, u64) {
    let files_written: u64 = dirs.iter().map(|dir| count_files(dir)).sum();
    let output_root = match dirs {
        [single, ..] if dirs.len() == 1 => single.clone(),
        many => {
            let mut shared: Vec<std::path::Component<'_>> =
                many[0].components().collect::<Vec<_>>();
            for dir in &many[1..] {
                let comps = dir.components().collect::<Vec<_>>();
                let common = shared
                    .iter()
                    .zip(comps.iter())
                    .take_while(|(a, b)| a == b)
                    .count();
                shared.truncate(common);
            }
            if shared.is_empty() {
                root.to_path_buf()
            } else {
                shared.iter().collect::<PathBuf>()
            }
        }
    };
    (output_root, files_written)
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
