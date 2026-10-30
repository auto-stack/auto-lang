//! PLAN-890 bounded dependency proof: the actual three shared generation
//! modules and restricted language helpers compile without the mixed
//! auto-api-gen implementation or its Tauri dev closure. There are no copied
//! SidecarDecl or workspace rules.
#[path = "../../../../../crates/auto-api-gen/src/rust_workspace.rs"]
pub mod rust_workspace;
#[path = "../../../../../crates/auto-api-gen/src/sidecar.rs"]
pub mod sidecar;
#[path = "../../../../../crates/auto-api-gen/src/source_owner.rs"]
pub mod source_owner;

#[path = "../../../../../crates/auto-api-gen/src/shared_generation.rs"]
pub mod shared_generation;

#[cfg(test)]
mod controls {
    use super::*;

    #[test]
    fn service_workspace_needs_no_ui_but_preserves_ui_members() {
        const CHILD: &str = "PLAN890_SERVICE_WORKSPACE_CHILD";
        if let Some(base) = std::env::var_os(CHILD) {
            let base = std::path::PathBuf::from(base);
            let project = base.join("project");
            let ws = rust_workspace::try_ensure_workspace_for(
                &project,
                rust_workspace::WorkspaceRequirements::Service,
            )
            .unwrap();
            let previous = std::fs::read(ws.join("Cargo.toml")).unwrap();
            let manifest: toml::Value = std::str::from_utf8(&previous).unwrap().parse().unwrap();
            let deps = &manifest["workspace"]["dependencies"];
            assert!(deps.get("auto-lang-core").is_some());
            assert!(deps.get("auto-ui-runtime").is_none());
            assert!(deps.get("iced").is_none());
            // Legacy UI entry must still reject a missing explicitly selected UI SDK.
            assert!(rust_workspace::try_ensure_shared_workspace(&project).is_err());
            assert_eq!(std::fs::read(ws.join("Cargo.toml")).unwrap(), previous);
            // A renamed dependency under a target table still requires UI.
            let member = ws.join("existing-ui");
            std::fs::create_dir_all(member.join("src")).unwrap();
            std::fs::write(member.join("src/lib.rs"), "").unwrap();
            std::fs::write(member.join("Cargo.toml"),
                "[package]\nname = \"existing-ui\"\nversion = \"0.1.0\"\n[target.'cfg(windows)'.dependencies]\nrenderer = { package = \"auto-ui-runtime\", workspace = true }\n").unwrap();
            assert!(rust_workspace::try_ensure_workspace_for(
                &project,
                rust_workspace::WorkspaceRequirements::Service,
            )
            .is_err());
            assert_eq!(std::fs::read(ws.join("Cargo.toml")).unwrap(), previous);
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let lang = temp.path().join("auto-lang");
        std::fs::create_dir_all(&lang).unwrap();
        std::fs::write(lang.join("Cargo.toml"), "[workspace]\n").unwrap();
        for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
            let package = lang.join("crates").join(name);
            std::fs::create_dir_all(package.join("src")).unwrap();
            std::fs::write(package.join("src/lib.rs"), "").unwrap();
            std::fs::write(
                package.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "controls::service_workspace_needs_no_ui_but_preserves_ui_members",
                "--nocapture",
            ])
            .env(CHILD, temp.path())
            .env("AUTO_LANG_ROOT", lang)
            .env("AUTO_UI_SOURCE_ROOT", temp.path().join("missing-ui"))
            .env("AUTO_RUST_WORKSPACE", temp.path().join("workspace"))
            .env_remove("AUTO_DELIVERY_SOURCE_ROOT")
            .env_remove("AUTO_LANG_CRATE")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn lock_identity_preserves_empty_absent_and_read_failure() {
        assert_eq!(source_owner::fnv1a(b""), 0x811c9dc5);
        assert_eq!(source_owner::fnv1a(b"a"), 0xe40c292c);
        assert_eq!(
            rust_workspace::workspace_lock_identity(Ok(Vec::new())).unwrap(),
            Some("811c9dc5".into())
        );
        assert_eq!(
            rust_workspace::workspace_lock_identity(Err(std::io::ErrorKind::NotFound.into()))
                .unwrap(),
            None
        );
        let error = rust_workspace::workspace_lock_identity(Err(
            std::io::ErrorKind::PermissionDenied.into(),
        ))
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    }
}
