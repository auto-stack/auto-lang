//! Actual production generation consumer. No UI, Man or Tauri dependency.
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let project = std::path::Path::new(&args[2]);
    if args[1] == "generate" {
        if let Err(error) = auto_api_gen::api_gen::generate_api(project, "rust") {
            eprintln!("production generation failed: {error}");
            std::process::exit(1);
        }
    } else if args[1] == "freshness" {
        let fresh = auto_api_gen::api_gen::backend_generation_is_fresh(project);
        println!("{}", serde_json::json!({"fresh": fresh}));
        if !fresh {
            std::process::exit(1);
        }
        return;
    } else {
        panic!("generate or freshness required");
    }
    let workspace = auto_api_gen::rust_workspace::resolve_rust_workspace_dir(project);
    let name = auto_api_gen::rust_workspace::back_member_name(project);
    println!(
        "{}",
        serde_json::json!({
            "workspace": workspace, "member": workspace.join(&name), "name": name,
        })
    );
}
