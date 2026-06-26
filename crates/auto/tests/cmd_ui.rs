//! Integration tests for the `auto ui` subcommand (Plan 331).

use std::fs;
use std::process::Command;

/// Path to the built `auto` binary under test.
fn auto_bin() -> String {
    env!("CARGO_BIN_EXE_auto").to_string()
}

#[test]
fn ui_list_prints_registered_widgets() {
    let out = Command::new(auto_bin())
        .args(["ui", "list"])
        .output()
        .expect("failed to spawn auto");
    assert!(out.status.success(), "auto ui list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("button"), "stdout: {stdout}");
    assert!(stdout.contains("input"), "stdout: {stdout}");
    assert!(stdout.contains("label"), "stdout: {stdout}");
}

#[test]
fn ui_build_writes_self_contained_sfcs() {
    let tmp = tempfile_dir();
    let out = Command::new(auto_bin())
        .args([
            "ui",
            "build",
            "--target",
            "vue",
            "--out",
            tmp.to_str().unwrap(),
            "--widgets",
            "button,input,label",
        ])
        .output()
        .expect("failed to spawn auto");
    assert!(out.status.success(), "build failed: {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("wrote 3 widgets"), "stdout: {stdout}");

    // Each widget dir + SFC exists.
    let button_vue = tmp.join("button/Button.vue");
    let input_vue = tmp.join("input/Input.vue");
    let label_vue = tmp.join("label/Label.vue");
    assert!(button_vue.exists(), "{button_vue:?} missing");
    assert!(input_vue.exists(), "{input_vue:?} missing");
    assert!(label_vue.exists(), "{label_vue:?} missing");

    let button_sfc = fs::read_to_string(&button_vue).unwrap();
    assert!(button_sfc.contains("<script setup"), "button script");
    assert!(button_sfc.contains("<template>"), "button template");
    assert!(
        !button_sfc.contains("@/components/ui/"),
        "button must be self-contained"
    );
    assert!(button_sfc.contains("reka-ui"), "button uses reka-ui");

    // Support files emitted alongside.
    assert!(tmp.join("button/index.ts").exists(), "button index.ts");
    assert!(tmp.join("button/variants.ts").exists(), "button variants.ts");
    let idx = fs::read_to_string(tmp.join("button/index.ts")).unwrap();
    assert!(idx.contains("Button"), "index re-exports Button: {idx}");
}

/// Create a unique temp dir. Uses std only (no tempfile dep).
fn tempfile_dir() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    // Process id + a coarse counter via nanos-free source: combine pid + a
    // static atomic to keep dirs unique within a run.
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    p.push(format!("auto-ui-cmd-{}-{}", std::process::id(), n));
    fs::create_dir_all(&p).unwrap();
    p
}
