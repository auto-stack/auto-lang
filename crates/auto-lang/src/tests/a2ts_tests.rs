use crate::{
    error::AutoResult,
    parser::Parser,
    trans::{Sink, Trans, typescript::TypeScriptTrans},
};
use std::fs;
use std::path::PathBuf;

fn test_a2ts(case: &str) -> AutoResult<()> {
    let parts: Vec<&str> = case.split("_").collect();
    let name = parts[1..].join("_");
    let name = name.as_str();

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2ts/{}/{}.at", case, name);
    let src_path = d.join(src_path);
    let src = fs::read_to_string(src_path.as_path())?;

    let _scope = crate::scope_manager::ScopeManager::new();
    let mut parser = Parser::from(src.as_str());
    let ast = parser.parse()?;
    let mut sink = Sink::new(name.into());
    let mut trans = TypeScriptTrans::new(name.into());
    trans.trans(ast, &mut sink)?;
    let ts_code = sink.done()?;

    let expected_path = format!("test/a2ts/{}/{}.expected.ts", case, name);
    let expected_path = d.join(expected_path);
    let expected = fs::read_to_string(expected_path.as_path())?;

    let ts_string = String::from_utf8_lossy(&ts_code);
    if ts_string != expected {
        let wrong_path = format!("test/a2ts/{}/{}.wrong.ts", case, name);
        let wrong_path = d.join(wrong_path);
        fs::write(&wrong_path, ts_code)?;
        panic!("Output differs from expected. Check {}.wrong.ts", name);
    }

    Ok(())
}

#[test]
fn test_000_hello() {
    test_a2ts("000_hello").unwrap();
}

#[test]
fn test_003_func() {
    test_a2ts("003_func").unwrap();
}

#[test]
fn test_006_struct() {
    test_a2ts("006_struct").unwrap();
}

#[test]
fn test_007_enum() {
    test_a2ts("007_enum").unwrap();
}

#[test]
fn test_010_if() {
    test_a2ts("010_if").unwrap();
}

#[test]
fn test_011_for() {
    test_a2ts("011_for").unwrap();
}

#[test]
fn test_013_while() {
    test_a2ts("013_while").unwrap();
}

#[test]
fn test_015_nested_if() {
    test_a2ts("015_nested_if").unwrap();
}

#[test]
fn test_017_loop() {
    test_a2ts("017_loop").unwrap();
}

#[test]
fn test_018_for_each() {
    test_a2ts("018_for_each").unwrap();
}

#[test]
fn test_014_closure() {
    test_a2ts("014_closure").unwrap();
}

#[test]
fn test_019_blocks() {
    test_a2ts("019_blocks").unwrap();
}

#[test]
fn test_008_method() {
    test_a2ts("008_method").unwrap();
}

#[test]
fn test_009_alias() {
    test_a2ts("009_alias").unwrap();
}

#[test]
fn test_013_union() {
    test_a2ts("013_union").unwrap();
}

#[test]
fn test_014_tag() {
    test_a2ts("014_tag").unwrap();
}

#[test]
fn test_017_struct_methods() {
    test_a2ts("017_struct_methods").unwrap();
}

#[test]
fn test_028_object() {
    test_a2ts("028_object").unwrap();
}

#[test]
fn test_029_composition() {
    test_a2ts("029_composition").unwrap();
}

#[test]
fn test_016_basic_spec() {
    test_a2ts("016_basic_spec").unwrap();
}

#[test]
fn test_017_spec() {
    test_a2ts("017_spec").unwrap();
}

#[test]
fn test_030_range_expr() {
    test_a2ts("030_range_expr").unwrap();
}

#[test]
fn test_018_delegation() {
    test_a2ts("018_delegation").unwrap();
}
