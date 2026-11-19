use ace_aac_syntax_conformance::{product_from_files, syntax_av_modules};

// Dev probe: compile the full syntax module set plus the file at $AT_PROBE.
#[test]
fn at_probe_runner() {
    let path = std::env::var("AT_PROBE").expect("AT_PROBE");
    let src = std::fs::read_to_string(&path).unwrap();
    let mut files = syntax_av_modules();
    files.push(("syntax_lexer.at".to_string(), include_str!("../../../aac/syntax/syntax_lexer.at").to_string()));
    files.push(("syntax_cst.at".to_string(), include_str!("../../../aac/syntax/syntax_cst.at").to_string()));
    files.push(("syntax_parse.at".to_string(), src));
    files.push(("main.at".to_string(), "fn main() Str {\n    return \"ok\"\n}\n".to_string()));
    match product_from_files("probe", &files, "app::main") {
        Ok(_) => println!("VERDICT OK"),
        Err(e) => println!("VERDICT ERR {e}"),
    }
}
