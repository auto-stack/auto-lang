//! PLAN-815 R5（F-815-T05-R5-01）: the OLD PUBLIC build entries (no tool
//! host, no RunIo installed) keep their BASELINE return semantics — a
//! failed build prints guidance and returns Ok; the strict named-failure
//! contract belongs to the hosted/protocol form only (covered over the
//! real protocol by `tool_process::build_ui_jet_reports_real_failure_and_
//! missing_artifact`). These probes call the lib directly, unhosted.

use std::path::Path;

fn jet_project(dir: &Path) {
    std::fs::create_dir_all(dir.join("src").join("front")).unwrap();
    std::fs::write(
        dir.join("pac.at"),
        "name: \"legacysem\"\nrender: \"jet\"\nwindow: \"640x480\"\nfront: \"./src/front\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src").join("front").join("app.at"),
        "widget App {\n    view {\n        col { text `x` }\n    }\n}\n",
    )
    .unwrap();
}

/// Legacy jet build with a failing wrapper: prints guidance, returns Ok
/// (the R4 regression made this Err unconditionally).
#[test]
#[ignore = "heavy: real generation chain; serial evidence run via --ignored"]
fn legacy_jet_build_failure_returns_ok_with_guidance() {
    // Unhosted by construction: this test binary never installs a RunIo.
    assert!(auto_ui_tools::proc_host::current().is_none());
    let dir = tempfile::tempdir().expect("tempdir");
    jet_project(dir.path());
    auto_ui_tools::jet::generate_jet_project(dir.path(), None, true).expect("generate");
    let gradlew = if cfg!(windows) {
        dir.path().join("gen/front/jet/gradlew.bat")
    } else {
        dir.path().join("gen/front/jet/gradlew")
    };
    if cfg!(windows) {
        std::fs::write(&gradlew, "@echo off\r\n@exit /b 7\r\n").unwrap();
    } else {
        std::fs::write(&gradlew, "#!/bin/sh\nexit 7\n").unwrap();
        make_executable(&gradlew);
    }
    let result = auto_ui_tools::jet::build_jet_project(dir.path());
    assert!(
        result.is_ok(),
        "legacy public entry keeps its baseline Ok-on-failure semantics: {result:?}"
    );
}

/// Legacy ark build without a wrapper: prints guidance, returns Ok (the
/// R4 regression made this Err unconditionally).
#[test]
#[ignore = "heavy: real generation chain; serial evidence run via --ignored"]
fn legacy_ark_missing_wrapper_returns_ok_with_guidance() {
    assert!(auto_ui_tools::proc_host::current().is_none());
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src").join("front")).unwrap();
    std::fs::write(
        dir.path().join("pac.at"),
        "name: \"legacyark\"\nrender: \"ark\"\nfront: \"./src/front\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src").join("front").join("app.at"),
        "widget App {\n    view {\n        col { text `x` }\n    }\n}\n",
    )
    .unwrap();
    auto_ui_tools::ark::generate_ark_project(dir.path(), None, true).expect("generate");
    // The fresh scaffold ships no hvigorw — the legacy arm prints and Ok.
    let result = auto_ui_tools::ark::build_ark_project(dir.path());
    assert!(
        result.is_ok(),
        "legacy public entry keeps its baseline Ok-on-missing-wrapper semantics: {result:?}"
    );
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

#[cfg(windows)]
fn make_executable(_path: &std::path::Path) {}
