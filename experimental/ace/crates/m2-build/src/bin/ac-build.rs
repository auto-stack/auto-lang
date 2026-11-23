//! `ac-build` — the PLAN-978 engineering build CLI.
//!
//! Subcommands (the T-01 frozen surface):
//! ```text
//! ac-build build   --project <dir> --target <t>[,<t>...] --root <dir>
//! ac-build sdk     --project <dir> --target <t>[,<t>...] --out <dir> --root <dir> [--grant <g>] [--provider <file>]
//! ac-build run     --sdk <dir> --target <t> [--grant <g>]
//! ac-build verify  --sdk <dir>
//! ac-build restore --root <dir> --project <name>
//! ```
//! A project directory is `project.json`
//! (`{"name","entry","caps":[],"stdlib":[]}`) plus one `src/*.at` public
//! source. Every command prints a JSON receipt to stdout and exits
//! 0 (ok) / 1 (named rejection) / 2 (usage).
//! An AV Rust project declares `"rust_provider":{"source":"provider.rs"}`
//! and may supply `"cargo_lock":"Cargo.lock"` for an annotated dependency.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ace_m2_build::driver::{M2Build, ProjectInputs, RustProjectInputs};
use ace_m2_build::identity::TargetKind;

const EXIT_OK: u8 = 0;
const EXIT_REJECT: u8 = 1;
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(String::as_str) else {
        usage();
        return ExitCode::from(EXIT_USAGE);
    };
    let flags = parse_flags(&args[1..]);
    let result = match cmd {
        "build" => cmd_build(&flags),
        "sdk" => cmd_sdk(&flags),
        "run" => cmd_run(&flags),
        "verify" => cmd_verify(&flags),
        "restore" => cmd_restore(&flags),
        "help" | "--help" | "-h" => {
            usage();
            return ExitCode::from(EXIT_USAGE);
        }
        other => {
            eprintln!("unknown command `{other}`");
            usage();
            return ExitCode::from(EXIT_USAGE);
        }
    };
    match result {
        Ok(json) => {
            println!("{json}");
            ExitCode::from(EXIT_OK)
        }
        Err(reject) => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "code": reject.code,
                    "detail": reject.detail,
                })
            );
            ExitCode::from(EXIT_REJECT)
        }
    }
}

fn usage() {
    eprintln!("ac-build — PLAN-978 engineering incremental multi-target build + candidate SDK");
    eprintln!("  build   --project <dir> --target av,native,a2r,a2c,a2py --root <dir>");
    eprintln!("  sdk     --project <dir> --target <t> --out <dir> --root <dir> [--grant <g>] [--provider <id>:<kind>:<file>]");
    eprintln!("  run     --sdk <dir> --target <t> [--grant <g>]");
    eprintln!("  verify  --sdk <dir>");
    eprintln!("  restore --root <dir> --project <name>");
}

fn parse_flags(args: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if let Some(name) = args[i].strip_prefix("--") {
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                out.push((name.to_string(), args[i + 1].clone()));
                i += 2;
            } else {
                out.push((name.to_string(), String::new()));
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out
}

fn flag<'a>(flags: &'a [(String, String)], name: &str) -> Option<&'a str> {
    flags
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

fn parse_targets(raw: &str) -> Result<Vec<TargetKind>, ace_m2_build::M2Reject> {
    let mut out = Vec::new();
    for t in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        out.push(
            TargetKind::parse(t).map_err(|e| ace_m2_build::M2Reject::new("m2build.usage", e))?,
        );
    }
    if out.is_empty() {
        return Err(ace_m2_build::M2Reject::new(
            "m2build.usage",
            "no targets given",
        ));
    }
    Ok(out)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RustInputPaths {
    source: String,
    cargo_lock: Option<String>,
}

fn read_rust_input(dir: &Path, rel: &str) -> Result<String, ace_m2_build::M2Reject> {
    ace_m2_build::SdkRecipe::safe_rel(rel)?;
    let io_error = |e: std::io::Error| {
        ace_m2_build::M2Reject::new("m2build.rust-input", format!("read {rel}: {e}"))
    };
    let root = dir.canonicalize().map_err(io_error)?;
    let path = dir.join(rel).canonicalize().map_err(io_error)?;
    if !path.starts_with(root) {
        return Err(ace_m2_build::M2Reject::new(
            "m2build.rust-input",
            format!("{rel} resolves outside the declared project"),
        ));
    }
    std::fs::read_to_string(path).map_err(io_error)
}

fn load_project(dir: &Path) -> Result<ProjectInputs, ace_m2_build::M2Reject> {
    let doc = std::fs::read_to_string(dir.join("project.json")).map_err(|e| {
        ace_m2_build::M2Reject::new(
            "m2build.usage",
            format!("read {}: {e}", dir.join("project.json").display()),
        )
    })?;
    let spec: serde_json::Value = serde_json::from_str(&doc)
        .map_err(|e| ace_m2_build::M2Reject::new("m2build.usage", format!("project.json: {e}")))?;
    let name = spec["name"].as_str().unwrap_or("project").to_string();
    let entry = spec["entry"].as_str().unwrap_or("main").to_string();
    let caps = spec["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut sources = Vec::new();
    let src_dir = dir.join("src");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&src_dir)
        .map_err(|e| {
            ace_m2_build::M2Reject::new("m2build.usage", format!("read {}: {e}", src_dir.display()))
        })?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("at"))
        .collect();
    entries.sort();
    for p in entries {
        let text = std::fs::read_to_string(&p).map_err(|e| {
            ace_m2_build::M2Reject::new("m2build.usage", format!("read {}: {e}", p.display()))
        })?;
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        sources.push((name, text));
    }
    if sources.len() != 1 {
        return Err(ace_m2_build::M2Reject::new(
            "m2build.usage",
            format!(
                "expected exactly one src/*.at public source, found {}",
                sources.len()
            ),
        ));
    }
    let mut stdlib = Vec::new();
    if let Some(list) = spec["stdlib"].as_array() {
        for item in list {
            if let Some(rel) = item.as_str() {
                // relative to the project dir; packed under its declared
                // stdlib/auto path
                let full = dir.join(rel);
                let text = std::fs::read_to_string(&full).map_err(|e| {
                    ace_m2_build::M2Reject::new("m2build.usage", format!("read stdlib {rel}: {e}"))
                })?;
                stdlib.push((rel.replace('\\', "/"), text));
            }
        }
    }
    let rust_provider = if let Some(value) = spec.get("rust_provider") {
        let paths: RustInputPaths = serde_json::from_value(value.clone()).map_err(|e| {
            ace_m2_build::M2Reject::new("m2build.rust-input", format!("rust_provider: {e}"))
        })?;
        Some(RustProjectInputs {
            source: read_rust_input(dir, &paths.source)?,
            cargo_lock: paths
                .cargo_lock
                .as_deref()
                .map(|path| read_rust_input(dir, path))
                .transpose()?,
            prepared: None,
        })
    } else {
        None
    };
    Ok(ProjectInputs {
        name,
        entry,
        caps,
        sources,
        stdlib,
        rust_provider,
    })
}

fn cmd_build(flags: &[(String, String)]) -> Result<String, ace_m2_build::M2Reject> {
    let project = flag(flags, "project").ok_or_else(|| usage_err("--project <dir>"))?;
    let root = flag(flags, "root").ok_or_else(|| usage_err("--root <dir>"))?;
    let targets = parse_targets(flag(flags, "target").unwrap_or("av"))?;
    let inputs = load_project(Path::new(project))?;
    let mut driver = M2Build::new(Path::new(root));
    let mut rows = Vec::new();
    for target in targets {
        let outcome = driver.build(target, &inputs)?;
        rows.push(serde_json::json!({
            "target": target.name(),
            "rebuilt": outcome.rebuilt,
            "identity": outcome.identity.key(),
            "run": outcome.run,
            "counts": outcome.counts,
            "products": outcome.products.iter().map(|(p, b)| {
                serde_json::json!({"path": p, "bytes": b.len()})
            }).collect::<Vec<_>>(),
        }));
    }
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "command": "build",
        "project": inputs.name,
        "builds": rows,
        "tool_spawns": driver.spawn_counts(),
    }))
    .unwrap())
}

fn cmd_sdk(flags: &[(String, String)]) -> Result<String, ace_m2_build::M2Reject> {
    let project = flag(flags, "project").ok_or_else(|| usage_err("--project <dir>"))?;
    let root = flag(flags, "root").ok_or_else(|| usage_err("--root <dir>"))?;
    let out = flag(flags, "out").ok_or_else(|| usage_err("--out <dir>"))?;
    let grant = flag(flags, "grant").unwrap_or("run:default");
    let targets = parse_targets(flag(flags, "target").unwrap_or("av"))?;
    let inputs = load_project(Path::new(project))?;
    let mut providers = Vec::new();
    if let Some(spec) = flag(flags, "provider") {
        // <id>:<kind>:<file>
        let parts: Vec<&str> = spec.splitn(3, ':').collect();
        if parts.len() == 3 {
            let bytes = std::fs::read(parts[2]).map_err(|e| {
                ace_m2_build::M2Reject::new(
                    "m2build.usage",
                    format!("read provider {}: {e}", parts[2]),
                )
            })?;
            let file_name = Path::new(parts[2])
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            providers.push(ace_m2_build::driver::ProviderArtifactInput {
                provider_id: parts[0].into(),
                file_name,
                bytes,
                kind: parts[1].into(),
            });
        }
    }
    let mut driver = M2Build::new(Path::new(root));
    let recipe = driver.build_sdk(&inputs, &targets, Path::new(out), providers, grant)?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "command": "sdk",
        "project": recipe.project,
        "targets": recipe.targets,
        "artifacts": recipe.artifacts.len(),
        "run_dependencies": recipe.run_dependencies,
        "build_inputs": recipe.build_inputs,
        "provider_selection": recipe.provider_selection.is_some(),
    }))
    .unwrap())
}

fn cmd_run(flags: &[(String, String)]) -> Result<String, ace_m2_build::M2Reject> {
    let sdk = flag(flags, "sdk").ok_or_else(|| usage_err("--sdk <dir>"))?;
    let target = TargetKind::parse(flag(flags, "target").unwrap_or("av"))
        .map_err(|e| ace_m2_build::M2Reject::new("m2build.usage", e))?;
    let grant = flag(flags, "grant").unwrap_or("run:default");
    let mut driver = M2Build::new(&std::env::temp_dir().join("ac-build-run"));
    let run = driver.run_sdk(Path::new(sdk), target, grant)?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "command": "run",
        "target": target.name(),
        "run": run,
        "tool_spawns": driver.spawn_counts(),
    }))
    .unwrap())
}

fn cmd_verify(flags: &[(String, String)]) -> Result<String, ace_m2_build::M2Reject> {
    let sdk = flag(flags, "sdk").ok_or_else(|| usage_err("--sdk <dir>"))?;
    let checks = M2Build::verify_sdk(Path::new(sdk))?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "command": "verify",
        "checks": checks,
    }))
    .unwrap())
}

fn cmd_restore(flags: &[(String, String)]) -> Result<String, ace_m2_build::M2Reject> {
    let root = flag(flags, "root").ok_or_else(|| usage_err("--root <dir>"))?;
    let project = flag(flags, "project").ok_or_else(|| usage_err("--project <name>"))?;
    let mut driver = M2Build::new(Path::new(root));
    let lines = driver.restore(project)?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "ok": true,
        "command": "restore",
        "report": lines,
    }))
    .unwrap())
}

fn usage_err(what: &str) -> ace_m2_build::M2Reject {
    ace_m2_build::M2Reject::new("m2build.usage", format!("missing {what}"))
}
