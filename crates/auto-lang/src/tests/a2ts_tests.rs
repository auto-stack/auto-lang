use crate::{
    error::AutoResult,
    parser::Parser,
    trans::{Sink, Trans, typescript::TypeScriptTrans},
};
use std::fs;
use std::path::PathBuf;

fn test_a2ts(case: &str) -> AutoResult<()> {
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.split("_").collect();
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

// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_a2ts("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_func() { test_a2ts("01_basics/002_func").unwrap(); }

// === 02_types ===
#[test] fn test_02_types_001_struct() { test_a2ts("02_types/001_struct").unwrap(); }
#[test] fn test_02_types_002_enum() { test_a2ts("02_types/002_enum").unwrap(); }
#[test] fn test_02_types_003_alias() { test_a2ts("02_types/003_alias").unwrap(); }

// === 03_control_flow ===
#[test] fn test_03_control_flow_001_if() { test_a2ts("03_control_flow/001_if").unwrap(); }
#[test] fn test_03_control_flow_002_for() { test_a2ts("03_control_flow/002_for").unwrap(); }
#[test] fn test_03_control_flow_003_while() { test_a2ts("03_control_flow/003_while").unwrap(); }
#[test] fn test_03_control_flow_004_nested_if() { test_a2ts("03_control_flow/004_nested_if").unwrap(); }
#[test] fn test_03_control_flow_005_loop() { test_a2ts("03_control_flow/005_loop").unwrap(); }
#[test] fn test_03_control_flow_006_blocks() { test_a2ts("03_control_flow/006_blocks").unwrap(); }

// === 05_expressions ===
#[test] fn test_05_expressions_001_object() { test_a2ts("05_expressions/001_object").unwrap(); }
#[test] fn test_05_expressions_002_composition() { test_a2ts("05_expressions/002_composition").unwrap(); }
#[test] fn test_05_expressions_003_range_expr() { test_a2ts("05_expressions/003_range_expr").unwrap(); }

// === 06_pattern_matching ===
#[test] fn test_06_pattern_matching_001_hetero_enum() { test_a2ts("06_pattern_matching/001_hetero_enum").unwrap(); }

// === 07_ownership ===
#[test] fn test_07_ownership_001_union() { test_a2ts("07_ownership/001_union").unwrap(); }

// === 09_option_result ===
#[test] fn test_09_option_result_001_closure() { test_a2ts("09_option_result/001_closure").unwrap(); }

// === 11_methods ===
#[test] fn test_11_methods_001_method() { test_a2ts("11_methods/001_method").unwrap(); }
#[test] fn test_11_methods_002_struct_methods() { test_a2ts("11_methods/002_struct_methods").unwrap(); }
#[test] fn test_11_methods_003_ext() { test_a2ts("11_methods/003_ext").unwrap(); }

// === 12_specs ===
#[test] fn test_12_specs_001_basic_spec() { test_a2ts("12_specs/001_basic_spec").unwrap(); }
#[test] fn test_12_specs_002_spec() { test_a2ts("12_specs/002_spec").unwrap(); }

// === 13_delegation ===
#[test] fn test_13_delegation_001_delegation() { test_a2ts("13_delegation/001_delegation").unwrap(); }

// === 18_ts_interop ===
#[test] fn test_18_ts_interop_001_for_each() { test_a2ts("18_ts_interop/001_for_each").unwrap(); }
