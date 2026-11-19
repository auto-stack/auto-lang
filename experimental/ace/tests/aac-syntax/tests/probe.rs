use ace_aac_syntax_conformance::{product_from_files, syntax_av_modules};

// Dev probe: compile the full fixed syntax module set (from the lib) plus a
// trivial main. $AT_PROBE must name any existing file (workflow compat).
#[test]
fn at_probe_runner() {
    let _ = std::env::var("AT_PROBE").expect("AT_PROBE");
    let mut files = syntax_av_modules();
    files.push((
        "main.at".to_string(),
        "fn main() Str {
    return \"ok\"
}
".to_string(),
    ));
    match product_from_files("probe", &files, "app::main") {
        Ok(_) => println!("VERDICT OK"),
        Err(e) => println!("VERDICT ERR {e}"),
    }
}
