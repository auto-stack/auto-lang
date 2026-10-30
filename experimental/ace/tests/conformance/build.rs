fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap();
    println!(
        "cargo:rustc-env=AUTO_LANG_HOST_DIR={}",
        root.join("crates/auto-lang").display()
    );
    println!(
        "cargo:rustc-env=AUTO_LANG_CORE_DIR={}",
        root.join("crates/auto-lang-core").display()
    );
    println!(
        "cargo:rustc-env=AUTO_LANG_CASE_BIN={}",
        root.join(
            if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
                "target/debug/auto-case.exe"
            } else {
                "target/debug/auto-case"
            }
        )
        .display()
    );
    println!(
        "cargo:rustc-env=AUTO_LANG_PILOT_BIN={}",
        root.join(
            if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
                "target/debug/test-case-pilot.exe"
            } else {
                "target/debug/test-case-pilot"
            }
        )
        .display()
    );
}
