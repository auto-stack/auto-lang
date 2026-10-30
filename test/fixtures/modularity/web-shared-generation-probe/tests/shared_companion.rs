//! Actual language helper consumer, with compiler rejection controls. No HTTP
//! runtime, UI/Man/Tauri dependency or hand-edited generated Rust output.
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use web_shared_generation_probe::shared_generation as shared;

fn cargo(project: &Path, target: &Path, action: &str, token: &str) -> (bool, String) {
    let log_path = project.join(format!("{action}.log"));
    let log = std::fs::File::create(&log_path).unwrap();
    let mut child = Command::new(env!("CARGO"))
        .args([action, "--offline", "--quiet", "-j", "4", "--manifest-path"])
        .arg(project.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target)
        .env("PLAN890_HELPER_ROOT", token)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return (status.success(), std::fs::read_to_string(log_path).unwrap());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "owned generated consumer exceeded deadline: {}",
                std::fs::read_to_string(log_path).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn shared_helpers_preserve_import_signatures_and_public_type_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root.join("target/shared-companion");
    std::fs::create_dir_all(&scratch).unwrap();
    let project = tempfile::tempdir_in(scratch).unwrap();
    let back = project.path().join("back");
    let src = project.path().join("src");
    std::fs::create_dir_all(&back).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    let works = r#"pub type Work = { path str }
pub fn join_path(root str, leaf str) str { return root + "/" + leaf }
pub fn make(root str, leaf str) Work { return Work { path: join_path(root, leaf) } }
"#;
    let repository = r#"use works: Work, make
pub fn load() Work { return make(Env.get("PLAN890_HELPER_ROOT"), "leaf") }
"#;
    std::fs::write(back.join("works.at"), works).unwrap();
    std::fs::write(back.join("repository.at"), repository).unwrap();
    let api_types = shared::try_full_parse("pub type Work = { path str }")
        .expect("real type extraction")
        .types;
    let (works_rs, _) =
        shared::transpile_back_module_to_rs_with_proofs("works", works, Some(back.clone()))
            .unwrap();
    let unpromoted = shared::post_process_companion_rs(works_rs);
    let promoted = shared::promote_api_types_in_companion(&unpromoted, &api_types);
    let (repository_rs, _proofs) = shared::transpile_back_module_to_rs_with_proofs(
        "repository",
        repository,
        Some(back.clone()),
    )
    .unwrap();
    let repository_rs = shared::post_process_companion_rs(repository_rs);
    std::fs::write(src.join("works.rs"), &promoted).unwrap();
    std::fs::write(src.join("repository.rs"), &repository_rs).unwrap();
    std::fs::write(
        src.join("main.rs"),
        r#"mod types {
    #[derive(Clone, Debug)] pub struct Work { pub path: String }
}
mod works;
mod repository;
fn main() {
    let value: types::Work = repository::load();
    assert_eq!(value.path, format!("{}/leaf", std::env::var("PLAN890_HELPER_ROOT").unwrap()));
    println!("owned={}", value.path);
}
"#,
    )
    .unwrap();
    let core = root
        .join("../../../../crates/auto-lang-core")
        .canonicalize()
        .unwrap();
    let manifest = format!(
        r#"[package]
name = "plan890-shared-companion-consumer"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
auto-lang-core = {{ path = {} }}
[profile.dev]
debug = "line-tables-only"
"#,
        serde_json::to_string(&core.to_string_lossy()).unwrap()
    );
    std::fs::write(project.path().join("Cargo.toml"), manifest).unwrap();
    let exe = std::env::current_exe().unwrap();
    let target = exe.parent().unwrap().parent().unwrap().parent().unwrap();
    let token = format!(
        "owned-{}-{}",
        std::process::id(),
        project.path().file_name().unwrap().to_string_lossy()
    );
    let (success, output) = cargo(project.path(), target, "run", &token);
    assert!(success, "positive generated consumer failed: {output}");
    assert!(
        output
            .lines()
            .any(|line| line == format!("owned={token}/leaf")),
        "{output}"
    );
    // Removing just the type promotion must be observable to the real compiler.
    std::fs::write(src.join("works.rs"), unpromoted).unwrap();
    let (success, output) = cargo(project.path(), target, "check", &token);
    assert!(
        !success
            && output.contains("mismatched types")
            && output.contains("types::Work")
            && output.contains("works::Work"),
        "unpromoted sibling falsely accepted: {output}"
    );
    println!("type identity negative: expected types::Work / found works::Work (actual E0308)");
    std::fs::write(src.join("works.rs"), &promoted).unwrap();
    // Removing source_dir reproduces the imported &str-signature loss.
    let (without_imports, _) =
        shared::transpile_back_module_to_rs_with_proofs("repository", repository, None).unwrap();
    std::fs::write(
        src.join("repository.rs"),
        shared::post_process_companion_rs(without_imports),
    )
    .unwrap();
    let (success, output) = cargo(project.path(), target, "check", &token);
    assert!(
        !success
            && output.contains("mismatched types")
            && output.contains("repository.rs")
            && output.contains("expected `&str`, found `String`"),
        "missing signature source falsely accepted: {output}"
    );
    println!(
        "import signature negative: repository.rs expected &str / found String (actual E0308)"
    );
    // Restore the exact good generated sources, then consume again.
    std::fs::write(src.join("repository.rs"), repository_rs).unwrap();
    let (success, output) = cargo(project.path(), target, "run", &token);
    assert!(
        success
            && output
                .lines()
                .any(|line| line == format!("owned={token}/leaf")),
        "restored generated consumer failed: {output}"
    );
    println!(
        "shared helpers: actual compile/run, both compiler rejection controls, restored run passed"
    );
}
