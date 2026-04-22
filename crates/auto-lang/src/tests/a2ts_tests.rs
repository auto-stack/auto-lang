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
#[test] fn test_01_basics_003_comments() { test_a2ts("01_basics/003_comments").unwrap(); }
#[test] fn test_01_basics_004_unary() { test_a2ts("01_basics/004_unary").unwrap(); }
#[test] fn test_01_basics_005_multi_expr() { test_a2ts("01_basics/005_multi_expr").unwrap(); }
#[test] fn test_01_basics_006_const() { test_a2ts("01_basics/006_const").unwrap(); }
#[test] fn test_01_basics_007_nested_call() { test_a2ts("01_basics/007_nested_call").unwrap(); }
#[test] fn test_01_basics_008_bool_ops() { test_a2ts("01_basics/008_bool_ops").unwrap(); }

// === 02_types ===
#[test] fn test_02_types_001_struct() { test_a2ts("02_types/001_struct").unwrap(); }
#[test] fn test_02_types_002_enum() { test_a2ts("02_types/002_enum").unwrap(); }
#[test] fn test_02_types_003_alias() { test_a2ts("02_types/003_alias").unwrap(); }
#[test] fn test_02_types_004_type_alias() { test_a2ts("02_types/004_type_alias").unwrap(); }
#[test] fn test_02_types_005_nested_struct() { test_a2ts("02_types/005_nested_struct").unwrap(); }
#[test] fn test_02_types_006_enum_simple() { test_a2ts("02_types/006_enum_simple").unwrap(); }
#[test] fn test_02_types_007_many_fields() { test_a2ts("02_types/007_many_fields").unwrap(); }

// === 03_control_flow ===
#[test] fn test_03_control_flow_001_if() { test_a2ts("03_control_flow/001_if").unwrap(); }
#[test] fn test_03_control_flow_002_for() { test_a2ts("03_control_flow/002_for").unwrap(); }
#[test] fn test_03_control_flow_003_while() { test_a2ts("03_control_flow/003_while").unwrap(); }
#[test] fn test_03_control_flow_004_nested_if() { test_a2ts("03_control_flow/004_nested_if").unwrap(); }
#[test] fn test_03_control_flow_005_loop() { test_a2ts("03_control_flow/005_loop").unwrap(); }
#[test] fn test_03_control_flow_006_blocks() { test_a2ts("03_control_flow/006_blocks").unwrap(); }
#[test] fn test_03_control_flow_007_async_fn() { test_a2ts("03_control_flow/007_async_fn").unwrap(); }
#[test] fn test_03_control_flow_008_await_expr() { test_a2ts("03_control_flow/008_await_expr").unwrap(); }
#[test] fn test_03_control_flow_009_promise_return() { test_a2ts("03_control_flow/009_promise_return").unwrap(); }
#[test] fn test_03_control_flow_010_while_loop() { test_a2ts("03_control_flow/010_while_loop").unwrap(); }
#[test] fn test_03_control_flow_011_nested_loops() { test_a2ts("03_control_flow/011_nested_loops").unwrap(); }

// === 04_strings ===
#[test] fn test_04_strings_002_fstring_nested() { test_a2ts("04_strings/002_fstring_nested").unwrap(); }
#[test] fn test_04_strings_003_fstring_multi() { test_a2ts("04_strings/003_fstring_multi").unwrap(); }
#[test] fn test_04_strings_004_fstring_edge() { test_a2ts("04_strings/004_fstring_edge").unwrap(); }
#[test] fn test_04_strings_005_str_split() { test_a2ts("04_strings/005_str_split").unwrap(); }
#[test] fn test_04_strings_006_str_replace() { test_a2ts("04_strings/006_str_replace").unwrap(); }
#[test] fn test_04_strings_007_str_case() { test_a2ts("04_strings/007_str_case").unwrap(); }
#[test] fn test_04_strings_008_str_find() { test_a2ts("04_strings/008_str_find").unwrap(); }

// === 05_expressions ===
#[test] fn test_05_expressions_001_object() { test_a2ts("05_expressions/001_object").unwrap(); }
#[test] fn test_05_expressions_002_composition() { test_a2ts("05_expressions/002_composition").unwrap(); }
#[test] fn test_05_expressions_003_range_expr() { test_a2ts("05_expressions/003_range_expr").unwrap(); }
#[test] fn test_05_expressions_004_ternary() { test_a2ts("05_expressions/004_ternary").unwrap(); }
#[test] fn test_05_expressions_005_composition() { test_a2ts("05_expressions/005_composition").unwrap(); }
#[test] fn test_05_expressions_006_index() { test_a2ts("05_expressions/006_index").unwrap(); }
#[test] fn test_05_expressions_007_cast() { test_a2ts("05_expressions/007_cast").unwrap(); }

// === 06_pattern_matching ===
#[test] fn test_06_pattern_matching_001_hetero_enum() { test_a2ts("06_pattern_matching/001_hetero_enum").unwrap(); }
#[test] fn test_06_pattern_matching_002_switch_guards() { test_a2ts("06_pattern_matching/002_switch_guards").unwrap(); }
#[test] fn test_06_pattern_matching_003_wildcard() { test_a2ts("06_pattern_matching/003_wildcard").unwrap(); }
#[test] fn test_06_pattern_matching_004_nested_destructure() { test_a2ts("06_pattern_matching/004_nested_destructure").unwrap(); }
#[test] fn test_06_pattern_matching_005_multi_pattern() { test_a2ts("06_pattern_matching/005_multi_pattern").unwrap(); }
#[test] fn test_06_pattern_matching_006_enum_match() { test_a2ts("06_pattern_matching/006_enum_match").unwrap(); }
#[test] fn test_06_pattern_matching_007_option_match() { test_a2ts("06_pattern_matching/007_option_match").unwrap(); }
#[test] fn test_06_pattern_matching_008_literal_match() { test_a2ts("06_pattern_matching/008_literal_match").unwrap(); }

// === 07_ownership ===
#[test] fn test_07_ownership_001_union() { test_a2ts("07_ownership/001_union").unwrap(); }

// === 08_generics ===
#[test] fn test_08_generics_001_generic_fn() { test_a2ts("08_generics/001_generic_fn").unwrap(); }
#[test] fn test_08_generics_002_generic_extends() { test_a2ts("08_generics/002_generic_extends").unwrap(); }
#[test] fn test_08_generics_003_generic_struct() { test_a2ts("08_generics/003_generic_struct").unwrap(); }
#[test] fn test_08_generics_004_multi_param() { test_a2ts("08_generics/004_multi_param").unwrap(); }

// === 09_option_result ===
#[test] fn test_09_option_result_001_closure() { test_a2ts("09_option_result/001_closure").unwrap(); }
#[test] fn test_09_option_result_002_option_basic() { test_a2ts("09_option_result/002_option_basic").unwrap(); }
#[test] fn test_09_option_result_003_null_coalesce() { test_a2ts("09_option_result/003_null_coalesce").unwrap(); }
#[test] fn test_09_option_result_004_result_ok_err() { test_a2ts("09_option_result/004_result_ok_err").unwrap(); }
#[test] fn test_09_option_result_005_option_pattern() { test_a2ts("09_option_result/005_option_pattern").unwrap(); }
#[test] fn test_09_option_result_006_is_some() { test_a2ts("09_option_result/006_is_some").unwrap(); }
#[test] fn test_09_option_result_007_is_none() { test_a2ts("09_option_result/007_is_none").unwrap(); }
#[test] fn test_09_option_result_008_unwrap() { test_a2ts("09_option_result/008_unwrap").unwrap(); }
#[test] fn test_09_option_result_009_or_else() { test_a2ts("09_option_result/009_or_else").unwrap(); }
#[test] fn test_09_option_result_010_option_map() { test_a2ts("09_option_result/010_option_map").unwrap(); }
#[test] fn test_09_option_result_011_filter() { test_a2ts("09_option_result/011_filter").unwrap(); }

// === 10_collections ===
#[test] fn test_10_collections_001_array_basic() { test_a2ts("10_collections/001_array_basic").unwrap(); }
#[test] fn test_10_collections_002_array_methods() { test_a2ts("10_collections/002_array_methods").unwrap(); }
#[test] fn test_10_collections_003_map_type() { test_a2ts("10_collections/003_map_type").unwrap(); }
#[test] fn test_10_collections_004_set_type() { test_a2ts("10_collections/004_set_type").unwrap(); }
#[test] fn test_10_collections_005_object_literal() { test_a2ts("10_collections/005_object_literal").unwrap(); }

// === 11_methods ===
#[test] fn test_11_methods_001_method() { test_a2ts("11_methods/001_method").unwrap(); }
#[test] fn test_11_methods_002_struct_methods() { test_a2ts("11_methods/002_struct_methods").unwrap(); }
#[test] fn test_11_methods_003_ext() { test_a2ts("11_methods/003_ext").unwrap(); }
#[test] fn test_11_methods_004_static_method() { test_a2ts("11_methods/004_static_method").unwrap(); }
#[test] fn test_11_methods_005_method_params() { test_a2ts("11_methods/005_method_params").unwrap(); }

// === 12_specs ===
#[test] fn test_12_specs_001_basic_spec() { test_a2ts("12_specs/001_basic_spec").unwrap(); }
#[test] fn test_12_specs_002_spec() { test_a2ts("12_specs/002_spec").unwrap(); }
#[test] fn test_12_specs_003_spec_generic() { test_a2ts("12_specs/003_spec_generic").unwrap(); }

// === 13_delegation ===
#[test] fn test_13_delegation_001_delegation() { test_a2ts("13_delegation/001_delegation").unwrap(); }
#[test] fn test_13_delegation_002_delegation_fn() { test_a2ts("13_delegation/002_delegation_fn").unwrap(); }

// === 14_modules ===
#[test] fn test_14_modules_001_basic_import() { test_a2ts("14_modules/001_basic_import").unwrap(); }
#[test] fn test_14_modules_002_named_import() { test_a2ts("14_modules/002_named_import").unwrap(); }
#[test] fn test_14_modules_003_export_fn() { test_a2ts("14_modules/003_export_fn").unwrap(); }

// === 15_type_conversion ===
#[test] fn test_15_type_conversion_001_cast() { test_a2ts("15_type_conversion/001_cast").unwrap(); }
#[test] fn test_15_type_conversion_002_to() { test_a2ts("15_type_conversion/002_to").unwrap(); }

// === 18_ts_interop ===
#[test] fn test_18_ts_interop_001_for_each() { test_a2ts("18_ts_interop/001_for_each").unwrap(); }
