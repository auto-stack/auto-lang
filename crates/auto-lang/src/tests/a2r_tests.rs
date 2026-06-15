use crate::{
    error::AutoResult,
    trans::rust::transpile_rust,
};
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_a2r_with_base(base: &str, case: &str) -> AutoResult<()> {
    // Parse test case name: "01_basics/001_hello" -> "hello"
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/{base}/{case}/{}.at", name);
    let src_path = d.join(src_path);
    let src = read_to_string(src_path.as_path())?;

    let exp_path = format!("test/{base}/{case}/{}.expected.rs", name);
    let exp_path = d.join(exp_path);
    let expected = if !exp_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_path.as_path())?
    };

    let mut rcode = transpile_rust(&name, &src)?;
    let rs_code = rcode.done()?;

    if rs_code != expected.as_bytes() {
        let gen_path = format!("test/{base}/{case}/{}.wrong.rs", name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, rs_code)?;
    }

    assert_eq!(String::from_utf8_lossy(rs_code), expected);
    Ok(())
}

fn test_a2r(case: &str) -> AutoResult<()> {
    test_a2r_with_base("a2r", case)
}

fn test_cookbook(case: &str) -> AutoResult<()> {
    test_a2r_with_base("cookbook", case)
}

// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_a2r("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_sqrt() { test_a2r("01_basics/002_sqrt").unwrap(); }
#[test] fn test_01_basics_003_func() { test_a2r("01_basics/003_func").unwrap(); }
#[test] fn test_01_basics_004_doc_comments() { test_a2r("01_basics/004_doc_comments").unwrap(); }

// === 02_types ===
#[test] fn test_02_types_001_struct() { test_a2r("02_types/001_struct").unwrap(); }
#[test] fn test_02_types_002_enum() { test_a2r("02_types/002_enum").unwrap(); }
#[test] fn test_02_types_003_union() { test_a2r("02_types/003_union").unwrap(); }
#[test] fn test_02_types_004_pointer() { test_a2r("02_types/004_pointer").unwrap(); }
#[test] fn test_02_types_005_inheritance() { test_a2r("02_types/005_inheritance").unwrap(); }
#[test] fn test_02_types_006_object() { test_a2r("02_types/006_object").unwrap(); }
#[test] fn test_02_types_007_cstr() { test_a2r("02_types/007_cstr").unwrap(); }
#[test] fn test_02_types_008_mut_self() { test_a2r("02_types/008_mut_self").unwrap(); }
#[test] fn test_02_types_009_self_field() { test_a2r("02_types/009_self_field").unwrap(); }
#[test] fn test_02_types_010_ext_keyword() { test_a2r("02_types/010_ext_keyword").unwrap(); }

// === 03_control_flow ===
#[test] fn test_03_control_flow_001_if_basic() { test_a2r("03_control_flow/001_if_basic").unwrap(); }
#[test] fn test_03_control_flow_002_if_nested() { test_a2r("03_control_flow/002_if_nested").unwrap(); }
#[test] fn test_03_control_flow_003_if_multistmt() { test_a2r("03_control_flow/003_if_multistmt").unwrap(); }
#[test] fn test_03_control_flow_004_if_return() { test_a2r("03_control_flow/004_if_return").unwrap(); }
#[test] fn test_03_control_flow_005_for_range() { test_a2r("03_control_flow/005_for_range").unwrap(); }
#[test] fn test_03_control_flow_006_for_conditions() { test_a2r("03_control_flow/006_for_conditions").unwrap(); }
#[test] fn test_03_control_flow_007_while_loop() { test_a2r("03_control_flow/007_while_loop").unwrap(); }
#[test] fn test_03_control_flow_008_is_match() { test_a2r("03_control_flow/008_is_match").unwrap(); }
#[test] fn test_03_control_flow_009_is_multi_stmt() { test_a2r("03_control_flow/009_is_multi_stmt").unwrap(); }
#[test] fn test_03_control_flow_010_is_non_exhaustive() { test_a2r("03_control_flow/010_is_non_exhaustive").unwrap(); }

// === 04_strings ===
#[test] fn test_04_strings_001_fstring() { test_a2r("04_strings/001_fstring").unwrap(); }
#[test] fn test_04_strings_002_fstring_edge() { test_a2r("04_strings/002_fstring_edge").unwrap(); }
#[test] fn test_04_strings_003_multi_str() { test_a2r("04_strings/003_multi_str").unwrap(); }
#[test] fn test_04_strings_004_backtick_string() { test_a2r("04_strings/004_backtick_string").unwrap(); }
#[test] fn test_04_strings_005_escaped_quotes() { test_a2r("04_strings/005_escaped_quotes").unwrap(); }
#[test] fn test_04_strings_006_multi_fstr() { test_a2r("04_strings/006_multi_fstr").unwrap(); }

// === 05_expressions ===
#[test] fn test_05_expressions_001_arithmetic() { test_a2r("05_expressions/001_arithmetic").unwrap(); }
#[test] fn test_05_expressions_002_unary() { test_a2r("05_expressions/002_unary").unwrap(); }
#[test] fn test_05_expressions_003_indexing() { test_a2r("05_expressions/003_indexing").unwrap(); }
#[test] fn test_05_expressions_004_blocks() { test_a2r("05_expressions/004_blocks").unwrap(); }
#[test] fn test_05_expressions_005_ref_expr() { test_a2r("05_expressions/005_ref_expr").unwrap(); }
#[test] fn test_05_expressions_006_range_expr() { test_a2r("05_expressions/006_range_expr").unwrap(); }
#[test] fn test_05_expressions_007_composition() { test_a2r("05_expressions/007_composition").unwrap(); }
#[test] fn test_05_expressions_008_field_composition() { test_a2r("05_expressions/008_field_composition").unwrap(); }
#[test] fn test_05_expressions_009_comprehensive() { test_a2r("05_expressions/009_comprehensive").unwrap(); }
#[test] fn test_05_expressions_010_or_keyword() { test_a2r("05_expressions/010_or_keyword").unwrap(); }
#[test] fn test_05_expressions_011_no_left_shift() { test_a2r("05_expressions/011_no_left_shift").unwrap(); }

// === 06_pattern_matching ===
#[test] fn test_06_pattern_matching_001_enum_pattern() { test_a2r("06_pattern_matching/001_enum_pattern").unwrap(); }
#[test] fn test_06_pattern_matching_002_struct_destructure() { test_a2r("06_pattern_matching/002_struct_destructure").unwrap(); }
#[test] fn test_06_pattern_matching_003_empty_variant_match() { test_a2r("06_pattern_matching/003_empty_variant_match").unwrap(); }
#[test] fn test_06_pattern_matching_004_hetero_enum() { test_a2r("06_pattern_matching/004_hetero_enum").unwrap(); }
#[test] fn test_06_pattern_matching_005_generic_hetero_enum() { test_a2r("06_pattern_matching/005_generic_hetero_enum").unwrap(); }
#[test] fn test_06_pattern_matching_006_enum_fn_param() { test_a2r("06_pattern_matching/006_enum_fn_param").unwrap(); }
#[test] fn test_06_pattern_matching_007_is_in_ext() { test_a2r("06_pattern_matching/007_is_in_ext").unwrap(); }
#[test] fn test_06_pattern_matching_008_hetero_enum_multistmt() { test_a2r("06_pattern_matching/008_hetero_enum_multistmt").unwrap(); }

// === 07_ownership ===
#[test] fn test_07_ownership_001_borrow_view() { test_a2r("07_ownership/001_borrow_view").unwrap(); }
#[test] fn test_07_ownership_002_borrow_mut() { test_a2r("07_ownership/002_borrow_mut").unwrap(); }
#[test] fn test_07_ownership_003_borrow_move() { test_a2r("07_ownership/003_borrow_move").unwrap(); }
#[test] fn test_07_ownership_004_borrow_conflicts() { test_a2r("07_ownership/004_borrow_conflicts").unwrap(); }

// === 08_generics ===
#[test] fn test_08_generics_001_type_alias() { test_a2r("08_generics/001_type_alias").unwrap(); }
#[test] fn test_08_generics_002_const_generics() { test_a2r("08_generics/002_const_generics").unwrap(); }
#[test] fn test_08_generics_003_generic_field() { test_a2r("08_generics/003_generic_field").unwrap(); }
#[test] fn test_08_generics_004_generic_ptr_field() { test_a2r("08_generics/004_generic_ptr_field").unwrap(); }
#[test] fn test_08_generics_005_with_constraint() { test_a2r("08_generics/005_with_constraint").unwrap(); }
#[test] fn test_08_generics_006_map_type() { test_a2r("08_generics/006_map_type").unwrap(); }
#[test] fn test_08_generics_007_no_tuple_generic() { test_a2r("08_generics/007_no_tuple_generic").unwrap(); }

// === 09_option_result ===
#[test] fn test_09_option_result_001_option() { test_a2r("09_option_result/001_option").unwrap(); }
#[test] fn test_09_option_result_002_option_construct() { test_a2r("09_option_result/002_option_construct").unwrap(); }
#[test] fn test_09_option_result_003_null_coalesce() { test_a2r("09_option_result/003_null_coalesce").unwrap(); }
#[test] fn test_09_option_result_004_error_propagate() { test_a2r("09_option_result/004_error_propagate").unwrap(); }
#[test] fn test_09_option_result_005_question_uint() { test_a2r("09_option_result/005_question_uint").unwrap(); }
#[test] fn test_09_option_result_006_question_float() { test_a2r("09_option_result/006_question_float").unwrap(); }
#[test] fn test_09_option_result_007_question_double() { test_a2r("09_option_result/007_question_double").unwrap(); }
#[test] fn test_09_option_result_008_question_return_int() { test_a2r("09_option_result/008_question_return_int").unwrap(); }
#[test] fn test_09_option_result_009_question_return_str() { test_a2r("09_option_result/009_question_return_str").unwrap(); }
#[test] fn test_09_option_result_010_question_return_bool() { test_a2r("09_option_result/010_question_return_bool").unwrap(); }
#[test] fn test_09_option_result_011_question_propagate() { test_a2r("09_option_result/011_question_propagate").unwrap(); }
#[test] fn test_09_option_result_012_question_return_float() { test_a2r("09_option_result/012_question_return_float").unwrap(); }
#[test] fn test_09_option_result_013_question_return_double() { test_a2r("09_option_result/013_question_return_double").unwrap(); }
#[test] fn test_09_option_result_014_question_return_char() { test_a2r("09_option_result/014_question_return_char").unwrap(); }
#[test] fn test_09_option_result_015_question_return_uint() { test_a2r("09_option_result/015_question_return_uint").unwrap(); }
#[test] fn test_09_option_result_016_question_return_float_v2() { test_a2r("09_option_result/016_question_return_float_v2").unwrap(); }
#[test] fn test_09_option_result_017_question_return_double_v2() { test_a2r("09_option_result/017_question_return_double_v2").unwrap(); }
#[test] fn test_09_option_result_018_question_return_char_v2() { test_a2r("09_option_result/018_question_return_char_v2").unwrap(); }
#[test] fn test_09_option_result_019_question_nested_call() { test_a2r("09_option_result/019_question_nested_call").unwrap(); }
#[test] fn test_09_option_result_020_question_arithmetic() { test_a2r("09_option_result/020_question_arithmetic").unwrap(); }
#[test] fn test_09_option_result_021_question_comparison() { test_a2r("09_option_result/021_question_comparison").unwrap(); }
#[test] fn test_09_option_result_022_question_literal() { test_a2r("09_option_result/022_question_literal").unwrap(); }
#[test] fn test_09_option_result_023_question_negation() { test_a2r("09_option_result/023_question_negation").unwrap(); }
#[test] fn test_09_option_result_024_question_zero() { test_a2r("09_option_result/024_question_zero").unwrap(); }
#[test] fn test_09_option_result_025_question_negative() { test_a2r("09_option_result/025_question_negative").unwrap(); }
#[test] fn test_09_option_result_026_list_basic() { test_a2r("09_option_result/026_list_basic").unwrap(); }
#[test] fn test_09_option_result_027_list_methods() { test_a2r("09_option_result/027_list_methods").unwrap(); }
#[test] fn test_09_option_result_028_list_may() { test_a2r("09_option_result/028_list_may").unwrap(); }
#[test] fn test_09_option_result_029_list_propagate() { test_a2r("09_option_result/029_list_propagate").unwrap(); }
#[test] fn test_09_option_result_030_list_coalesce() { test_a2r("09_option_result/030_list_coalesce").unwrap(); }
#[test] fn test_09_option_result_031_option_bool_field() { test_a2r("09_option_result/031_option_bool_field").unwrap(); }
#[test] fn test_09_option_result_032_fn_result_enum() { test_a2r("09_option_result/032_fn_result_enum").unwrap(); }
#[test] fn test_09_option_result_033_result_is_match() { test_a2r("09_option_result/033_result_is_match").unwrap(); }
#[test] fn test_09_option_result_034_result_bang_type() { test_a2r("09_option_result/034_result_bang_type").unwrap(); }

// === 10_collections ===
#[test] fn test_10_collections_001_array() { test_a2r("10_collections/001_array").unwrap(); }
#[test] fn test_10_collections_002_list_storage() { test_a2r("10_collections/002_list_storage").unwrap(); }
#[test] fn test_10_collections_003_map_func() { test_a2r("10_collections/003_map_func").unwrap(); }
#[test] fn test_10_collections_004_list_as_cast() { test_a2r("10_collections/004_list_as_cast").unwrap(); }
#[test] fn test_10_collections_005_method_chain() { test_a2r("10_collections/005_method_chain").unwrap(); }

// === 11_methods ===
#[test] fn test_11_methods_001_method() { test_a2r("11_methods/001_method").unwrap(); }
#[test] fn test_11_methods_002_struct_methods() { test_a2r("11_methods/002_struct_methods").unwrap(); }
#[test] fn test_11_methods_003_closure() { test_a2r("11_methods/003_closure").unwrap(); }
#[test] fn test_11_methods_004_static_fn() { test_a2r("11_methods/004_static_fn").unwrap(); }
#[test] fn test_11_methods_005_func_literal_return() { test_a2r("11_methods/005_func_literal_return").unwrap(); }
#[test] fn test_11_methods_006_ext_for() { test_a2r("11_methods/006_ext_for").unwrap(); }
#[test] fn test_11_methods_007_ext_from() { test_a2r("11_methods/007_ext_from").unwrap(); }
#[test] fn test_11_methods_008_empty_body() { test_a2r("11_methods/008_empty_body").unwrap(); }

// === 12_specs ===
#[test] fn test_12_specs_001_basic_spec() { test_a2r("12_specs/001_basic_spec").unwrap(); }
#[test] fn test_12_specs_002_spec() { test_a2r("12_specs/002_spec").unwrap(); }
#[test] fn test_12_specs_003_spec_delegation() { test_a2r("12_specs/003_spec_delegation").unwrap(); }

// === 13_delegation ===
#[test] fn test_13_delegation_001_single() { test_a2r("13_delegation/001_single").unwrap(); }
#[test] fn test_13_delegation_002_multi_spec() { test_a2r("13_delegation/002_multi_spec").unwrap(); }
#[test] fn test_13_delegation_003_multi_delegation() { test_a2r("13_delegation/003_multi_delegation").unwrap(); }

// === 14_modules ===
#[test] fn test_14_modules_001_rust_use() { test_a2r("14_modules/001_rust_use").unwrap(); }
#[test] fn test_14_modules_002_pub_use() { test_a2r("14_modules/002_pub_use").unwrap(); }
#[test] fn test_14_modules_003_pub_visibility() { test_a2r("14_modules/003_pub_visibility").unwrap(); }
#[test] fn test_14_modules_004_wildcard_import() { test_a2r("14_modules/004_wildcard_import").unwrap(); }

// Special: multi_file uses transpile_rust_project with its own assertions
#[test]
fn test_14_modules_005_multi_file() {
    use crate::trans::rust::transpile_rust_project;

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entry = d.join("test/a2r/14_modules/005_multi_file/main.at");

    let result = transpile_rust_project(entry.to_str().unwrap()).unwrap();

    // Check that all 4 files were generated
    assert!(result.contains_key("main.rs"), "Missing main.rs");
    assert!(result.contains_key("db.rs"), "Missing db.rs");
    assert!(result.contains_key("api/mod.rs"), "Missing api/mod.rs");
    assert!(result.contains_key("api/handlers.rs"), "Missing api/handlers.rs");

    // Validate main.rs
    let main_rs = String::from_utf8_lossy(&result["main.rs"]);
    assert!(main_rs.contains("mod db;"), "main.rs should have 'mod db;'");
    assert!(main_rs.contains("mod api;"), "main.rs should have 'mod api;'");
    assert!(main_rs.contains("fn main()"), "main.rs should have fn main()");

    // Validate api/mod.rs
    let api_mod = String::from_utf8_lossy(&result["api/mod.rs"]);
    assert!(api_mod.contains("pub mod handlers;"), "api/mod.rs should have 'pub mod handlers;'");

    // Validate db.rs
    let db_rs = String::from_utf8_lossy(&result["db.rs"]);
    assert!(db_rs.contains("struct Connection"), "db.rs should have struct Connection");
    assert!(db_rs.contains("fn connect()"), "db.rs should have fn connect()");

    // Validate api/handlers.rs
    let handlers_rs = String::from_utf8_lossy(&result["api/handlers.rs"]);
    assert!(handlers_rs.contains("use crate::db::*;"), "api/handlers.rs should have 'use crate::db::*;'");
    assert!(handlers_rs.contains("fn handle_request"), "api/handlers.rs should have fn handle_request");

    // Validate Cargo.toml
    assert!(result.contains_key("Cargo.toml"), "Missing Cargo.toml");
    let cargo_toml = String::from_utf8_lossy(&result["Cargo.toml"]);
    assert!(cargo_toml.contains("[package]"), "Cargo.toml should have [package]");
    assert!(cargo_toml.contains("name = \"005_multi_file\""), "Cargo.toml should have project name");
    assert!(cargo_toml.contains("edition = \"2021\""), "Cargo.toml should have edition = 2021");
}

#[test] fn test_14_modules_006_const_decl() { test_a2r("14_modules/006_const_decl").unwrap(); }
#[test] fn test_14_modules_007_shared_var() { test_a2r("14_modules/007_shared_var").unwrap(); }
#[test] fn test_14_modules_008_derive_attr() { test_a2r("14_modules/008_derive_attr").unwrap(); }
#[test] fn test_14_modules_009_const_before_ext() { test_a2r("14_modules/009_const_before_ext").unwrap(); }

// === 15_type_conversion ===
#[test] fn test_15_type_conversion_001_type_cast() { test_a2r("15_type_conversion/001_type_cast").unwrap(); }
#[test] fn test_15_type_conversion_002_to_convert() { test_a2r("15_type_conversion/002_to_convert").unwrap(); }
#[test] fn test_15_type_conversion_003_ptr_methods() { test_a2r("15_type_conversion/003_ptr_methods").unwrap(); }
#[test] fn test_15_type_conversion_004_box_arc() { test_a2r("15_type_conversion/004_box_arc").unwrap(); }

// === 16_interop ===
#[test] fn test_16_interop_001_async_fn() { test_a2r("16_interop/001_async_fn").unwrap(); }
#[test] fn test_16_interop_002_tokio_main() { test_a2r("16_interop/002_tokio_main").unwrap(); }
#[test] fn test_16_interop_003_field_attrs() { test_a2r("16_interop/003_field_attrs").unwrap(); }

// === 18_rust_std ===
#[test] fn test_18_rust_std_001_collections() { test_a2r("17_rust_std/001_collections").unwrap(); }
#[test] fn test_18_rust_std_002_fs() { test_a2r("17_rust_std/002_fs").unwrap(); }
#[test] fn test_18_rust_std_003_sync() { test_a2r("17_rust_std/003_sync").unwrap(); }
#[test] fn test_18_rust_std_004_time() { test_a2r("17_rust_std/004_time").unwrap(); }
#[test] fn test_18_rust_std_005_path() { test_a2r("17_rust_std/005_path").unwrap(); }
#[test] fn test_18_rust_std_006_box_cell() { test_a2r("17_rust_std/006_box_cell").unwrap(); }
#[test] fn test_18_rust_std_007_env_process() { test_a2r("17_rust_std/007_env_process").unwrap(); }
#[test] fn test_18_rust_std_008_thread() { test_a2r("17_rust_std/008_thread").unwrap(); }
#[test] fn test_18_rust_std_009_serde_json() { test_a2r("17_rust_std/009_serde_json").unwrap(); }
#[test] fn test_18_rust_std_010_regex() { test_a2r("17_rust_std/010_regex").unwrap(); }
#[test] fn test_18_rust_std_011_math() { test_a2r("17_rust_std/011_math").unwrap(); }
#[test] fn test_18_rust_std_012_vec() { test_a2r("17_rust_std/012_vec").unwrap(); }
#[test] fn test_18_rust_std_013_option_result() { test_a2r("17_rust_std/013_option_result").unwrap(); }
#[test] fn test_18_rust_std_014_iter() { test_a2r("17_rust_std/014_iter").unwrap(); }
#[test] fn test_18_rust_std_015_string_methods() { test_a2r("17_rust_std/015_string_methods").unwrap(); }
#[test] fn test_18_rust_std_016_cmp_ordering() { test_a2r("17_rust_std/016_cmp_ordering").unwrap(); }
#[test] fn test_18_rust_std_017_hash_map_ops() { test_a2r("17_rust_std/017_hash_map_ops").unwrap(); }

// === 18_pure_rust: Pure Rust output (no a2r_std dependency) ===

#[test] fn test_19_pure_rust_001_pure() { test_a2r("18_pure_rust/001_pure").unwrap(); }

// === 17_autocode: Real-world integration tests ===

fn autocode_src(name: &str) -> String {
    std::fs::read_to_string(format!("../../../auto-coder/src/{}.at", name)).unwrap()
}

// Source files in auto-coder/src/ not yet available — ignore until directory exists
#[test] #[ignore] fn test_17_autocode_001_types() { let src = autocode_src("types"); let mut r = transpile_rust("types", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_002_permission() { let src = autocode_src("permission"); let mut r = transpile_rust("permission", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_003_tools() { let src = autocode_src("tools"); let mut r = transpile_rust("tools", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_004_sse() { let src = autocode_src("sse"); let mut r = transpile_rust("sse", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_005_context() { let src = autocode_src("context"); let mut r = transpile_rust("context", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_006_settings() { let src = autocode_src("settings"); let mut r = transpile_rust("settings", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_007_agent() { let src = autocode_src("agent"); let mut r = transpile_rust("agent", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_008_anthropic() { let src = autocode_src("anthropic"); let mut r = transpile_rust("anthropic", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_009_openai() { let src = autocode_src("openai"); let mut r = transpile_rust("openai", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_010_session() { let src = autocode_src("session"); let mut r = transpile_rust("session", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_011_repl() { let src = autocode_src("repl"); let mut r = transpile_rust("repl", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_012_main() { let src = autocode_src("main"); let mut r = transpile_rust("main", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_013_mod() { let src = autocode_src("mod"); let mut r = transpile_rust("mod", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_014_tool_bash() { let src = autocode_src("tool_bash"); let mut r = transpile_rust("tool_bash", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_016_tool_file_read() { let src = autocode_src("tool_file_read"); let mut r = transpile_rust("tool_file_read", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_017_tool_file_write() { let src = autocode_src("tool_file_write"); let mut r = transpile_rust("tool_file_write", &src).unwrap(); r.done().unwrap(); }
#[test] #[ignore] fn test_17_autocode_018_tool_file_edit() { let src = autocode_src("tool_file_edit"); let mut r = transpile_rust("tool_file_edit", &src).unwrap(); r.done().unwrap(); }

// tool_grep requires 8MB stack for deep Pratt parser recursion
#[test]
#[ignore]
fn test_17_autocode_015_tool_grep() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let src = autocode_src("tool_grep");
            let mut r = transpile_rust("tool_grep", &src).unwrap();
            r.done().unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}

// Detailed error reporting test
#[test]
#[ignore]
fn test_17_autocode_019_detailed_errors() {
    use crate::parser::{Parser, CompileDest};

    // tool_grep.at is ~442KB and triggers deep recursion in the Pratt parser,
    // overflowing the default test thread stack. Use an explicit 8MB stack.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let base = "../../../auto-coder/src/";
            let files = [
                "tools", "sse", "context", "settings",
                "agent", "anthropic", "openai", "session", "repl", "main",
                "tool_bash", "tool_grep", "tool_file_read", "tool_file_write", "tool_file_edit",
            ];

            for name in &files {
                let path = format!("{}{}.at", base, name);
                let src = std::fs::read_to_string(&path).unwrap();
                let mut parser = Parser::from(&src);
                parser.set_dest(CompileDest::TransRust);
                match parser.parse() {
                    Ok(_) => println!("OK: {}", name),
                    Err(e) => {
                        let err_str = format!("{:?}", e);
                        let offset = extract_offset(&err_str);
                        let (line, col, source_line) = offset_to_line_col(&src, offset);
                        println!("FAIL: {} — byte {} = line {} col {}", name, offset, line, col);
                        println!("  | {}", source_line.trim_end());
                        println!("  | {:>width$}", "^", width = col);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

fn extract_offset(s: &str) -> usize {
    if let Some(pos) = s.find("SourceOffset(") {
        let rest = &s[pos + 13..];
        if let Some(end) = rest.find(")") {
            return rest[..end].parse().unwrap_or(0);
        }
    }
    0
}

fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize, String) {
    let mut line = 1;
    let mut last_newline = 0;
    for (i, ch) in src.char_indices() {
        if i == offset {
            return (line, i - last_newline + 1, get_line(src, offset));
        }
        if ch == '\n' {
            line += 1;
            last_newline = i + 1;
        }
    }
    (line, 0, String::new())
}

fn get_line(src: &str, offset: usize) -> String {
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[offset..].find('\n').map(|i| offset + i).unwrap_or(src.len());
    src[line_start..line_end].to_string()
}

// === Cookbook: Rust Cookbook a2r tests ===
// See docs/plans/240-rust-cookbook-a2r-tests.md for full classification

// -- cookbook/algorithms --
#[test] fn test_cookbook_algorithms_001_sort_int() { test_cookbook("algorithms/001_sort_int").unwrap(); }
#[test] fn test_cookbook_algorithms_002_sort_float() { test_cookbook("algorithms/002_sort_float").unwrap(); }
#[test] fn test_cookbook_algorithms_003_sort_struct() { test_cookbook("algorithms/003_sort_struct").unwrap(); }

// -- cookbook/file --
#[test] fn test_cookbook_file_001_read_lines() { test_cookbook("file/001_read_lines").unwrap(); }

// -- cookbook/os --
#[test] fn test_cookbook_os_001_env_variable() { test_cookbook("os/001_env_variable").unwrap(); }
#[test] fn test_cookbook_os_002_process_continuous() { test_cookbook("os/002_process_continuous").unwrap(); }
#[test] fn test_cookbook_os_003_error_file() { test_cookbook("os/003_error_file").unwrap(); }

// -- cookbook/datetime --
#[test] fn test_cookbook_datetime_001_elapsed_time() { test_cookbook("datetime/001_elapsed_time").unwrap(); }

// -- cookbook/science/mathematics/statistics --
#[test] fn test_cookbook_science_statistics_001_central_tendency() { test_cookbook("science/mathematics/statistics/001_central_tendency").unwrap(); }
#[test] fn test_cookbook_science_statistics_002_standard_deviation() { test_cookbook("science/mathematics/statistics/002_standard_deviation").unwrap(); }

// -- cookbook/science/mathematics/trigonometry --
#[test] fn test_cookbook_science_trigonometry_001_tan_sin_cos() { test_cookbook("science/mathematics/trigonometry/001_tan_sin_cos").unwrap(); }
#[test] fn test_cookbook_science_trigonometry_002_side_length() { test_cookbook("science/mathematics/trigonometry/002_side_length").unwrap(); }
#[test] fn test_cookbook_science_trigonometry_003_latitude_longitude() { test_cookbook("science/mathematics/trigonometry/003_latitude_longitude").unwrap(); }

// -- cookbook/mem --
#[test] fn test_cookbook_mem_001_lazy_cell() { test_cookbook("mem/001_lazy_cell").unwrap(); }

// -- cookbook/errors --
#[test] fn test_cookbook_errors_001_boxed_error() { test_cookbook("errors/001_boxed_error").unwrap(); }

// === Cookbook B-tier: External crate tests ===

// -- cookbook/algorithms/randomness (B-tier) --
#[test] fn test_cookbook_algorithms_004_rand() { test_cookbook("algorithms/004_rand").unwrap(); }
#[test] fn test_cookbook_algorithms_005_rand_choose() { test_cookbook("algorithms/005_rand_choose").unwrap(); }
#[test] fn test_cookbook_algorithms_006_rand_custom() { test_cookbook("algorithms/006_rand_custom").unwrap(); }
#[test] fn test_cookbook_algorithms_007_rand_dist() { test_cookbook("algorithms/007_rand_dist").unwrap(); }
#[test] fn test_cookbook_algorithms_008_rand_passwd() { test_cookbook("algorithms/008_rand_passwd").unwrap(); }
#[test] fn test_cookbook_algorithms_009_rand_range() { test_cookbook("algorithms/009_rand_range").unwrap(); }

// -- cookbook/cli (B-tier) --
#[test] fn test_cookbook_cli_001_clap_basic() { test_cookbook("cli/001_clap_basic").unwrap(); }

// -- cookbook/compression (B-tier) --
#[test] fn test_cookbook_compression_001_tar_compress() { test_cookbook("compression/001_tar_compress").unwrap(); }
#[test] fn test_cookbook_compression_002_tar_decompress() { test_cookbook("compression/002_tar_decompress").unwrap(); }

// -- cookbook/concurrency (B-tier) --
#[test] fn test_cookbook_concurrency_001_rayon_any_all() { test_cookbook("concurrency/001_rayon_any_all").unwrap(); }
#[test] fn test_cookbook_concurrency_002_rayon_map_reduce() { test_cookbook("concurrency/002_rayon_map_reduce").unwrap(); }
#[test] fn test_cookbook_concurrency_003_rayon_parallel_sort() { test_cookbook("concurrency/003_rayon_parallel_sort").unwrap(); }
#[test] fn test_cookbook_concurrency_004_crossbeam_spsc() { test_cookbook("concurrency/004_crossbeam_spsc").unwrap(); }

// -- cookbook/cryptography (B-tier) --
#[test] fn test_cookbook_cryptography_001_sha_digest() { test_cookbook("cryptography/001_sha_digest").unwrap(); }

// -- cookbook/datetime (B-tier, chrono) --
#[test] fn test_cookbook_datetime_002_checked() { test_cookbook("datetime/002_checked").unwrap(); }
#[test] fn test_cookbook_datetime_003_timezone() { test_cookbook("datetime/003_timezone").unwrap(); }
#[test] fn test_cookbook_datetime_004_current() { test_cookbook("datetime/004_current").unwrap(); }
#[test] fn test_cookbook_datetime_005_format() { test_cookbook("datetime/005_format").unwrap(); }
#[test] fn test_cookbook_datetime_006_parse_string() { test_cookbook("datetime/006_parse_string").unwrap(); }
#[test] fn test_cookbook_datetime_007_timestamp() { test_cookbook("datetime/007_timestamp").unwrap(); }

// -- cookbook/versioning (B-tier, semver) --
#[test] fn test_cookbook_versioning_001_semver_parse() { test_cookbook("versioning/001_semver_parse").unwrap(); }
#[test] fn test_cookbook_versioning_002_semver_increment() { test_cookbook("versioning/002_semver_increment").unwrap(); }
#[test] fn test_cookbook_versioning_003_semver_latest() { test_cookbook("versioning/003_semver_latest").unwrap(); }

// -- cookbook/encoding (B-tier) --
#[test] fn test_cookbook_encoding_001_json() { test_cookbook("encoding/001_json").unwrap(); }
#[test] fn test_cookbook_encoding_002_toml() { test_cookbook("encoding/002_toml").unwrap(); }
#[test] fn test_cookbook_encoding_003_csv_read() { test_cookbook("encoding/003_csv_read").unwrap(); }
#[test] fn test_cookbook_encoding_004_base64() { test_cookbook("encoding/004_base64").unwrap(); }
#[test] fn test_cookbook_encoding_005_hex() { test_cookbook("encoding/005_hex").unwrap(); }

// -- cookbook/errors (B-tier, anyhow) --
#[test] fn test_cookbook_errors_002_anyhow() { test_cookbook("errors/002_anyhow").unwrap(); }

// -- cookbook/file (B-tier, walkdir) --
#[test] fn test_cookbook_file_002_find_files() { test_cookbook("file/002_find_files").unwrap(); }
#[test] fn test_cookbook_file_003_recursive_size() { test_cookbook("file/003_recursive_size").unwrap(); }
#[test] fn test_cookbook_file_004_modified() { test_cookbook("file/004_modified").unwrap(); }

// -- cookbook/science/mathematics/complex_numbers (B-tier, num) --
#[test] fn test_cookbook_science_complex_001_add_complex() { test_cookbook("science/mathematics/complex_numbers/001_add_complex").unwrap(); }
#[test] fn test_cookbook_science_complex_002_create_complex() { test_cookbook("science/mathematics/complex_numbers/002_create_complex").unwrap(); }

// -- cookbook/text (B-tier, regex/unicode) --
#[test] fn test_cookbook_text_001_regex_replace() { test_cookbook("text/001_regex_replace").unwrap(); }
#[test] fn test_cookbook_text_002_regex_email() { test_cookbook("text/002_regex_email").unwrap(); }
#[test] fn test_cookbook_text_003_regex_hashtags() { test_cookbook("text/003_regex_hashtags").unwrap(); }
#[test] fn test_cookbook_text_004_graphemes() { test_cookbook("text/004_graphemes").unwrap(); }

// -- cookbook/web/url (B-tier, url) --
#[test] fn test_cookbook_web_url_001_base() { test_cookbook("web/url/001_base").unwrap(); }
#[test] fn test_cookbook_web_url_002_parse() { test_cookbook("web/url/002_parse").unwrap(); }
#[test] fn test_cookbook_web_url_003_fragment() { test_cookbook("web/url/003_fragment").unwrap(); }

// === Cookbook B-tier batch 2: Additional tests ===

// -- cookbook/algorithms --
#[test] fn test_cookbook_algorithms_010_rand_custom() { test_cookbook("algorithms/010_rand_custom").unwrap(); }
#[test] fn test_cookbook_algorithms_011_rand_dist() { test_cookbook("algorithms/011_rand_dist").unwrap(); }

// -- cookbook/cli --
#[test] fn test_cookbook_cli_002_ansi_term() { test_cookbook("cli/002_ansi_term").unwrap(); }

// -- cookbook/compression --
#[test] fn test_cookbook_compression_003_tar_strip_prefix() { test_cookbook("compression/003_tar_strip_prefix").unwrap(); }

// -- cookbook/concurrency --
#[test] fn test_cookbook_concurrency_005_rayon_iter_mut() { test_cookbook("concurrency/005_rayon_iter_mut").unwrap(); }
#[test] fn test_cookbook_concurrency_006_rayon_parallel_search() { test_cookbook("concurrency/006_rayon_parallel_search").unwrap(); }
#[test] fn test_cookbook_concurrency_007_crossbeam_complex() { test_cookbook("concurrency/007_crossbeam_complex").unwrap(); }
#[test] fn test_cookbook_concurrency_008_crossbeam_spawn() { test_cookbook("concurrency/008_crossbeam_spawn").unwrap(); }
#[test] fn test_cookbook_concurrency_009_global_mut_state() { test_cookbook("concurrency/009_global_mut_state").unwrap(); }
#[test] fn test_cookbook_concurrency_010_threadpool_walk() { test_cookbook("concurrency/010_threadpool_walk").unwrap(); }

// -- cookbook/cryptography --
#[test] fn test_cookbook_cryptography_002_pbkdf2() { test_cookbook("cryptography/002_pbkdf2").unwrap(); }
#[test] fn test_cookbook_cryptography_003_hmac() { test_cookbook("cryptography/003_hmac").unwrap(); }

// -- cookbook/devtools --

// === Plan 310 Phase 1: escape analysis pipeline verification ===
// These tests prove the analysis pass runs inside transpile_rust and produces
// non-trivial EscapeMaps. Phase 1 contract: output bytes unchanged, but the
// escape_results map must be populated for every function in the source.

#[test]
fn test_escape_analysis_runs_in_pipeline() {
    // Source with two functions: one with locals, one empty.
    let src = "fn add(a int, b int) int {\n    let s = a + b\n    return s\n}\n\nfn empty() {\n}\n";
    let summary = crate::trans::rust::escape_analysis_summary(src);
    // Both functions should be analyzed.
    assert!(summary.contains_key("add"), "add should be analyzed");
    assert!(summary.contains_key("empty"), "empty should be analyzed");
    // `add` has one binding (`s`), `empty` has none.
    assert_eq!(summary["add"], 1, "add should track 1 binding (s)");
    assert_eq!(summary["empty"], 0, "empty should track 0 bindings");
}

#[test]
fn test_escape_analysis_detects_multiple_bindings() {
    // A function with several locals and a nested for-loop scope.
    let src = "fn f() {\n    let a = 1\n    let b = 2\n    for i in 0..3 {\n        let c = a + i\n    }\n}\n";
    let summary = crate::trans::rust::escape_analysis_summary(src);
    // a, b at depth 0; i (loop var), c (loop body) at depth 1. The loop
    // variable form `for i in 0..3` is parsed as Iter::Named or Iter::Indexed;
    // either way c should be tracked. Conservative lower bound.
    assert!(
        summary["f"] >= 3,
        "f should track at least a, b, and the loop var (got {})",
        summary["f"]
    );
}

// === Plan 310 Phase 2: 19_ownership — escape-analysis codegen tests ===
#[test] fn test_19_ownership_001_local_borrow() { test_a2r("19_ownership/001_local_borrow").unwrap(); }
#[test] fn test_19_ownership_002_closure_capture() { test_a2r("19_ownership/002_closure_capture").unwrap(); }
#[test] fn test_19_ownership_003_return_escape() { test_a2r("19_ownership/003_return_escape").unwrap(); }
#[test] fn test_19_ownership_004_move_hint() { test_a2r("19_ownership/004_move_hint").unwrap(); }
#[test] fn test_19_ownership_005_async_move() { test_a2r("19_ownership/005_async_move").unwrap(); }
#[test] fn test_19_ownership_006_go_capture() { test_a2r("19_ownership/006_go_capture").unwrap(); }
#[test] fn test_cookbook_devtools_001_log_debug() { test_cookbook("devtools/001_log_debug").unwrap(); }
#[test] fn test_cookbook_devtools_002_log_error() { test_cookbook("devtools/002_log_error").unwrap(); }
#[test] fn test_cookbook_devtools_003_log_stdout() { test_cookbook("devtools/003_log_stdout").unwrap(); }
#[test] fn test_cookbook_devtools_004_log_custom() { test_cookbook("devtools/004_log_custom").unwrap(); }
#[test] fn test_cookbook_devtools_005_log_syslog() { test_cookbook("devtools/005_log_syslog").unwrap(); }
#[test] fn test_cookbook_devtools_006_log_env() { test_cookbook("devtools/006_log_env").unwrap(); }
#[test] fn test_cookbook_devtools_007_log_mod() { test_cookbook("devtools/007_log_mod").unwrap(); }
#[test] fn test_cookbook_devtools_008_log_timestamp() { test_cookbook("devtools/008_log_timestamp").unwrap(); }
#[test] fn test_cookbook_devtools_009_log_custom_location() { test_cookbook("devtools/009_log_custom_location").unwrap(); }
#[test] fn test_cookbook_devtools_010_tracing_console() { test_cookbook("devtools/010_tracing_console").unwrap(); }

// -- cookbook/encoding --
#[test] fn test_cookbook_encoding_006_endian_byte() { test_cookbook("encoding/006_endian_byte").unwrap(); }
#[test] fn test_cookbook_encoding_007_csv_delimiter() { test_cookbook("encoding/007_csv_delimiter").unwrap(); }
#[test] fn test_cookbook_encoding_008_csv_filter() { test_cookbook("encoding/008_csv_filter").unwrap(); }
#[test] fn test_cookbook_encoding_009_csv_invalid() { test_cookbook("encoding/009_csv_invalid").unwrap(); }
#[test] fn test_cookbook_encoding_010_csv_serde_serialize() { test_cookbook("encoding/010_csv_serde_serialize").unwrap(); }
#[test] fn test_cookbook_encoding_011_csv_serialize() { test_cookbook("encoding/011_csv_serialize").unwrap(); }
#[test] fn test_cookbook_encoding_012_csv_transform() { test_cookbook("encoding/012_csv_transform").unwrap(); }
#[test] fn test_cookbook_encoding_013_percent_encode() { test_cookbook("encoding/013_percent_encode").unwrap(); }
#[test] fn test_cookbook_encoding_014_url_encode() { test_cookbook("encoding/014_url_encode").unwrap(); }

// -- cookbook/errors --
#[test] fn test_cookbook_errors_003_backtrace() { test_cookbook("errors/003_backtrace").unwrap(); }
#[test] fn test_cookbook_errors_004_retain() { test_cookbook("errors/004_retain").unwrap(); }

// -- cookbook/file --
#[test] fn test_cookbook_file_005_duplicate_name() { test_cookbook("file/005_duplicate_name").unwrap(); }
#[test] fn test_cookbook_file_006_find_file() { test_cookbook("file/006_find_file").unwrap(); }
#[test] fn test_cookbook_file_007_ignore_case() { test_cookbook("file/007_ignore_case").unwrap(); }
#[test] fn test_cookbook_file_008_loops() { test_cookbook("file/008_loops").unwrap(); }
#[test] fn test_cookbook_file_009_png() { test_cookbook("file/009_png").unwrap(); }
#[test] fn test_cookbook_file_010_recursive() { test_cookbook("file/010_recursive").unwrap(); }
#[test] fn test_cookbook_file_011_sizes() { test_cookbook("file/011_sizes").unwrap(); }
#[test] fn test_cookbook_file_012_skip_dot() { test_cookbook("file/012_skip_dot").unwrap(); }
#[test] fn test_cookbook_file_013_same_file() { test_cookbook("file/013_same_file").unwrap(); }
#[test] fn test_cookbook_file_014_read_lines_temp() { test_cookbook("file/014_read_lines_temp").unwrap(); }

// -- cookbook/hardware --
#[test] fn test_cookbook_hardware_001_cpu_count() { test_cookbook("hardware/001_cpu_count").unwrap(); }

// -- cookbook/os --
#[test] fn test_cookbook_os_004_piped() { test_cookbook("os/004_piped").unwrap(); }
#[test] fn test_cookbook_os_005_process_output() { test_cookbook("os/005_process_output").unwrap(); }
#[test] fn test_cookbook_os_006_send_input() { test_cookbook("os/006_send_input").unwrap(); }

// -- cookbook/safety --
#[test] fn test_cookbook_safety_001_heapless() { test_cookbook("safety/001_heapless").unwrap(); }

// -- cookbook/science/mathematics/complex_numbers --
#[test] fn test_cookbook_science_mathematics_complex_numbers_003_math_functions() { test_cookbook("science/mathematics/complex_numbers/003_math_functions").unwrap(); }

// -- cookbook/science/mathematics/linear_algebra --
#[test] fn test_cookbook_science_mathematics_linear_algebra_001_add_matrices() { test_cookbook("science/mathematics/linear_algebra/001_add_matrices").unwrap(); }
#[test] fn test_cookbook_science_mathematics_linear_algebra_002_multiply_matrices() {
    // Nested array indexing c[i][j] triggers deep Pratt parser recursion on Windows debug
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(|| { test_cookbook("science/mathematics/linear_algebra/002_multiply_matrices").unwrap(); })
        .unwrap()
        .join()
        .unwrap();
}
#[test] fn test_cookbook_science_mathematics_linear_algebra_003_multiply_svm() { test_cookbook("science/mathematics/linear_algebra/003_multiply_svm").unwrap(); }
#[test] fn test_cookbook_science_mathematics_linear_algebra_004_vector_comparison() { test_cookbook("science/mathematics/linear_algebra/004_vector_comparison").unwrap(); }
#[test] fn test_cookbook_science_mathematics_linear_algebra_005_vector_norm() { test_cookbook("science/mathematics/linear_algebra/005_vector_norm").unwrap(); }
#[test] fn test_cookbook_science_mathematics_linear_algebra_006_invert_matrix() { test_cookbook("science/mathematics/linear_algebra/006_invert_matrix").unwrap(); }
#[test] fn test_cookbook_science_mathematics_linear_algebra_007_deserialize_matrix() { test_cookbook("science/mathematics/linear_algebra/007_deserialize_matrix").unwrap(); }

// -- cookbook/science/mathematics/miscellaneous --
#[test] fn test_cookbook_science_mathematics_miscellaneous_001_big_integers() { test_cookbook("science/mathematics/miscellaneous/001_big_integers").unwrap(); }
#[test] fn test_cookbook_science_mathematics_miscellaneous_002_math_functions() { test_cookbook("science/mathematics/miscellaneous/002_math_functions").unwrap(); }

// -- cookbook/text --
#[test] fn test_cookbook_text_005_filter_log() { test_cookbook("text/005_filter_log").unwrap(); }
#[test] fn test_cookbook_text_006_phone() { test_cookbook("text/006_phone").unwrap(); }
#[test] fn test_cookbook_text_007_from_str() { test_cookbook("text/007_from_str").unwrap(); }

// -- cookbook/versioning --
#[test] fn test_cookbook_versioning_004_semver_command() { test_cookbook("versioning/004_semver_command").unwrap(); }
#[test] fn test_cookbook_versioning_005_semver_complex() { test_cookbook("versioning/005_semver_complex").unwrap(); }
#[test] fn test_cookbook_versioning_006_semver_prerelease() { test_cookbook("versioning/006_semver_prerelease").unwrap(); }

// -- cookbook/web/mime --
#[test] fn test_cookbook_web_mime_001_filename() { test_cookbook("web/mime/001_filename").unwrap(); }
#[test] fn test_cookbook_web_mime_002_string() { test_cookbook("web/mime/002_string").unwrap(); }

// -- cookbook/web/url --
#[test] fn test_cookbook_web_url_004_new() { test_cookbook("web/url/004_new").unwrap(); }
#[test] fn test_cookbook_web_url_005_origin() { test_cookbook("web/url/005_origin").unwrap(); }

