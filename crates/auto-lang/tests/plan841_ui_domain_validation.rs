const GOOD: &str = "widget App { view { text \"ok\" {} } }";

#[test]
fn sdk_ui_build_entries_validate_before_writing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("app.at");
    let bad = format!("{GOOD}\n{GOOD}\n");
    std::fs::write(&path, &bad).expect("source");

    for backend in ["rust", "vue", "jet"] {
        let output = dir.path().join(backend);
        let error = auto_lang::ui_build(path.to_str().unwrap(), "ui", backend, output.to_str())
            .expect_err("SDK backend must reject duplicate widgets");
        assert!(error.to_string().contains("duplicate widget declaration"));
        assert!(!output.exists());
    }
    let shadcn_output = dir.path().join("shadcn");
    let error = auto_lang::ui_build_shadcn(path.to_str().unwrap(), shadcn_output.to_str())
        .expect_err("SDK shadcn entry must reject duplicate widgets");
    assert!(error.to_string().contains("duplicate widget declaration"));
    assert!(!shadcn_output.exists());

    std::fs::write(&path, GOOD).expect("valid source");
    assert!(auto_lang::ui_build(path.to_str().unwrap(), "ui", "rust", None).is_ok());
    assert!(auto_lang::ui_build_shadcn(path.to_str().unwrap(), None).is_ok());
}
