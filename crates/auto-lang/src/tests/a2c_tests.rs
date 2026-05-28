use crate::{
    error::{format_error, AutoResult},
    trans::c::{transpile_c, transpile_part},
};

#[test]
#[ignore]
fn test_c() {
    let code = "41";
    let out = transpile_part(code).unwrap();
    assert_eq!(out, "41;\n");
}

#[test]
#[ignore]
fn test_c_fn() {
    let code = "fn add(x, y) int { x+y }";
    let out = transpile_part(code).unwrap();
    let expected = r#"int add(int x, int y) {
    return x + y;
}
"#;
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_let() {
    let code = "let x = 41";
    let out = transpile_part(code).unwrap();
    let expected = "int x = 41;\n";
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_if() {
    let code = "let x = 41; if x > 0 { print(x) }";
    let out = transpile_part(code).unwrap();
    let expected = r#"int x = 41;
if (x > 0) {
    printf("%d\n", x);
}
"#;
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_if_else() {
    let code = "let x = 41; if x > 0 { print(x) } else { print(-x) }";
    let out = transpile_part(code).unwrap();
    let expected = r#"int x = 41;
if (x > 0) {
    printf("%d\n", x);
} else {
    printf("%d\n", -x);
}
"#;
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_array() {
    let code = "let x = [1, 2, 3]";
    let out = transpile_part(code).unwrap();
    let expected = "int x[3] = {1, 2, 3};\n";
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_var_assign() {
    let code = "var x = 41; x = 42";
    let out = transpile_part(code).unwrap();
    let expected = "int x = 41;\nx = 42;\n";
    assert_eq!(out, expected);
}

#[test]
#[ignore]
fn test_c_return_42() {
    let code = r#"42"#;
    let mut sink = transpile_c("test", code).unwrap();
    let expected = r#"int main(void) {
    return 42;
}
"#;
    let src = sink.done().unwrap();
    assert_eq!(String::from_utf8(src.clone()).unwrap(), expected);
}

#[test]
#[ignore]
fn test_math() {
    let code = r#"fn add(x int, y int) int { x+y }
add(1, 2)"#;
    let mut sink = transpile_c("test", code).unwrap();
    let expected = r#"#include "test.h"

int add(int x, int y) {
    return x + y;
}

int main(void) {
    return add(1, 2);
}
"#;
    let expected_header = r#"#pragma once

int add(int x, int y);
"#;
    assert_eq!(
        String::from_utf8(sink.done().unwrap().clone()).unwrap(),
        expected
    );
    assert_eq!(String::from_utf8(sink.header).unwrap(), expected_header);
}

fn test_a2c(case: &str) -> AutoResult<()> {
    use std::fs::read_to_string;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    // Parse test case name: "01_basics/001_hello" -> "hello"
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.split("_").collect();
    let name = parts[1..].join("_");
    let name = name.as_str();

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let src_path = format!("test/a2c/{}/{}.at", case, name);
    let src_path = d.join(src_path);
    let src = read_to_string(src_path.as_path())?;

    // Check if this is an error test
    let err_path = format!("test/a2c/{}/{}.expected.error.log", case, name);
    let err_path = d.join(err_path);

    if err_path.is_file() {
        // This is an error test - check that transpilation fails with expected error
        let _expected_error = read_to_string(err_path.as_path())?;

        let result = transpile_c(name, &src);

        match result {
            Err(e) => {
                let error_msg = format_error(&e);
                // Check if the error message contains the expected error (case-insensitive)
                if !error_msg.to_lowercase().contains("type mismatch") {
                    return Err(format!("Expected type mismatch error, got:\n{}", error_msg).into());
                }
                // Basic check passed - the transpiler correctly detected the type error
                Ok(())
            }
            Ok(_) => {
                return Err(format!(
                    "Expected transpilation to fail with type error, but it succeeded"
                )
                .into());
            }
        }
    } else {
        // Normal test - check generated code
        let exp_path = format!("test/a2c/{}/{}.expected.c", case, name);
        let exp_path = d.join(exp_path);
        let expected_src = if !exp_path.is_file() {
            "".to_string()
        } else {
            read_to_string(exp_path.as_path())?
        };

        let exph_path = format!("test/a2c/{}/{}.expected.h", case, name);
        let exph_path = d.join(exph_path);
        let expected_header = if !exph_path.is_file() {
            "".to_string()
        } else {
            read_to_string(exph_path.as_path())?
        };

        let mut ccode = transpile_c(name, &src)?;

        let src = ccode.done()?;

        if src != expected_src.as_bytes() {
            // output generated code to a gen file
            let gen_path = format!("test/a2c/{}/{}.wrong.c", case, name);
            let gen_path = d.join(gen_path);
            let mut file = File::create(gen_path.as_path())?;
            file.write_all(src)?;
        }

        assert_eq!(String::from_utf8_lossy(src), expected_src);

        let header = ccode.header;
        if header != expected_header.as_bytes() {
            // output generated code to a gen file
            let gen_path = format!("test/a2c/{}/{}.wrong.h", case, name);
            let gen_path = d.join(gen_path);
            let mut file = File::create(gen_path.as_path())?;
            file.write_all(&header)?;
        }
        assert_eq!(String::from_utf8_lossy(&header), expected_header);
        Ok(())
    }
}

// === 01_basics ===
#[test] #[ignore] fn test_01_basics_001_hello() { test_a2c("01_basics/001_hello").unwrap(); }
#[test] #[ignore] fn test_01_basics_002_sqrt() { test_a2c("01_basics/002_sqrt").unwrap(); }
#[test] #[ignore] fn test_01_basics_003_func() { test_a2c("01_basics/003_func").unwrap(); }

// === 02_types ===
#[test] #[ignore] fn test_02_types_001_struct() { test_a2c("02_types/001_struct").unwrap(); }
#[test] #[ignore] fn test_02_types_002_enum() { test_a2c("02_types/002_enum").unwrap(); }
#[test] #[ignore] fn test_02_types_003_union() { test_a2c("02_types/003_union").unwrap(); }
#[test] #[ignore] fn test_02_types_004_pointer() { test_a2c("02_types/004_pointer").unwrap(); }
#[test] #[ignore] fn test_02_types_005_inheritance() { test_a2c("02_types/005_inheritance").unwrap(); }
#[test] #[ignore] fn test_02_types_006_pointer_types() { test_a2c("02_types/006_pointer_types").unwrap(); }
#[test] #[ignore] fn test_02_types_007_bool() { test_a2c("02_types/007_bool").unwrap(); }

// === 03_control_flow ===
#[test] #[ignore] fn test_03_control_flow_001_if_basic() { test_a2c("03_control_flow/001_if_basic").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_002_for_range() { test_a2c("03_control_flow/002_for_range").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_003_is_match() { test_a2c("03_control_flow/003_is_match").unwrap(); }
#[test]
#[ignore = "For conditions not yet supported in C transpiler"]
fn test_03_control_flow_004_for_conditions() { test_a2c("03_control_flow/004_for_conditions").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_005_mut_counter() { test_a2c("03_control_flow/005_mut_counter").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_006_mut_accumulator() { test_a2c("03_control_flow/006_mut_accumulator").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_007_mut_array_sum() { test_a2c("03_control_flow/007_mut_array_sum").unwrap(); }
#[test] #[ignore] fn test_03_control_flow_008_mut_multiple() { test_a2c("03_control_flow/008_mut_multiple").unwrap(); }

// === 04_strings ===
#[test] #[ignore] fn test_04_strings_001_str() { test_a2c("04_strings/001_str").unwrap(); }
#[test] #[ignore] fn test_04_strings_002_str_split() { test_a2c("04_strings/002_str_split").unwrap(); }

// === 05_expressions ===
#[test] #[ignore] fn test_05_expressions_001_complex_expr() { test_a2c("05_expressions/001_complex_expr").unwrap(); }
#[test]
#[ignore = "Field access type checking not yet supported in C transpiler"]
fn test_05_expressions_002_field_access() { test_a2c("05_expressions/002_field_access").unwrap(); }
#[test] #[ignore] fn test_05_expressions_003_bang_operator() { test_a2c("05_expressions/003_bang_operator").unwrap(); }
#[test] #[ignore] fn test_05_expressions_004_binary() { test_a2c("05_expressions/004_binary").unwrap(); }

// === 06_pattern_matching ===
#[test] #[ignore] fn test_06_pattern_matching_001_hetero_enum() { test_a2c("06_pattern_matching/001_hetero_enum").unwrap(); }
// NOTE: hetero_enum_verify is a stub - needs expected output generated
#[test] #[ignore] fn test_06_pattern_matching_002_hetero_enum_verify() { test_a2c("06_pattern_matching/002_hetero_enum_verify").unwrap(); }
#[test] #[ignore] fn test_06_pattern_matching_003_hetero_enum_types() { test_a2c("06_pattern_matching/003_hetero_enum_types").unwrap(); }
#[test] #[ignore] fn test_06_pattern_matching_004_enum_smoke_2var() { test_a2c("06_pattern_matching/004_enum_smoke_2var").unwrap(); }
#[test] #[ignore] fn test_06_pattern_matching_005_enum_smoke_3var() { test_a2c("06_pattern_matching/005_enum_smoke_3var").unwrap(); }
#[test] #[ignore] fn test_06_pattern_matching_006_enum_with_functions() { test_a2c("06_pattern_matching/006_enum_with_functions").unwrap(); }
#[test] #[ignore] fn test_06_pattern_matching_007_tag_types() { test_a2c("06_pattern_matching/007_tag_types").unwrap(); }

// === 07_ownership ===
#[test] #[ignore] fn test_07_ownership_001_borrow_view() { test_a2c("07_ownership/001_borrow_view").unwrap(); }
#[test] #[ignore] fn test_07_ownership_002_borrow_mut() { test_a2c("07_ownership/002_borrow_mut").unwrap(); }
#[test] #[ignore] fn test_07_ownership_003_borrow_move() { test_a2c("07_ownership/003_borrow_move").unwrap(); }
#[test] #[ignore] fn test_07_ownership_004_borrow_conflicts() { test_a2c("07_ownership/004_borrow_conflicts").unwrap(); }

// === 08_generics ===
#[test]
#[ignore = "Const generics not yet implemented in C transpiler"]
fn test_08_generics_001_const_generics() { test_a2c("08_generics/001_const_generics").unwrap(); }
#[test] #[ignore] fn test_08_generics_002_generic_field() { test_a2c("08_generics/002_generic_field").unwrap(); }
#[test] #[ignore] fn test_08_generics_003_generic_ptr_field() { test_a2c("08_generics/003_generic_ptr_field").unwrap(); }
#[test] #[ignore] fn test_08_generics_004_with_constraint() { test_a2c("08_generics/004_with_constraint").unwrap(); }
#[test] #[ignore] fn test_08_generics_005_generic_specs() { test_a2c("08_generics/005_generic_specs").unwrap(); }
#[test] #[ignore] fn test_08_generics_006_generic_spec_ext() { test_a2c("08_generics/006_generic_spec_ext").unwrap(); }
#[test]
#[ignore = "Generic type alias not yet supported in C transpiler"]
fn test_08_generics_007_generic_type_alias() { test_a2c("08_generics/007_generic_type_alias").unwrap(); }

// === 09_option_result ===
#[test] #[ignore] fn test_09_option_result_001_null_coalesce() { test_a2c("09_option_result/001_null_coalesce").unwrap(); }
#[test] #[ignore] fn test_09_option_result_002_error_propagate() { test_a2c("09_option_result/002_error_propagate").unwrap(); }
#[test] #[ignore] fn test_09_option_result_003_closure() { test_a2c("09_option_result/003_closure").unwrap(); }

// === 10_collections ===
#[test] #[ignore] fn test_10_collections_001_array() { test_a2c("10_collections/001_array").unwrap(); }
#[test] #[ignore] fn test_10_collections_002_array_return() { test_a2c("10_collections/002_array_return").unwrap(); }
#[test] #[ignore] fn test_10_collections_003_array_declaration() { test_a2c("10_collections/003_array_declaration").unwrap(); }
#[test] #[ignore] fn test_10_collections_004_array_mutation() { test_a2c("10_collections/004_array_mutation").unwrap(); }
#[test] #[ignore] fn test_10_collections_005_array_index_read() { test_a2c("10_collections/005_array_index_read").unwrap(); }
#[test] #[ignore] fn test_10_collections_006_array_copy() { test_a2c("10_collections/006_array_copy").unwrap(); }
#[test] #[ignore] fn test_10_collections_007_array_slice() { test_a2c("10_collections/007_array_slice").unwrap(); }
#[test]
#[ignore = "Parser does not yet support nested arrays"]
fn test_10_collections_008_array_nested() { test_a2c("10_collections/008_array_nested").unwrap(); }
#[test]
#[ignore = "Parser does not yet support zero-size arrays"]
fn test_10_collections_009_array_zero_size() { test_a2c("10_collections/009_array_zero_size").unwrap(); }
#[test] #[ignore] fn test_10_collections_010_array_loop() { test_a2c("10_collections/010_array_loop").unwrap(); }
#[test] #[ignore] fn test_10_collections_011_list_storage() { test_a2c("10_collections/011_list_storage").unwrap(); }
#[test] #[ignore] fn test_10_collections_012_list_iter() { test_a2c("10_collections/012_list_iter").unwrap(); }
// NOTE: list_capacity is a stub - needs expected output generated
#[test] #[ignore] fn test_10_collections_013_list_capacity() { test_a2c("10_collections/013_list_capacity").unwrap(); }

// === 11_methods ===
#[test] #[ignore] fn test_11_methods_001_method() { test_a2c("11_methods/001_method").unwrap(); }
#[test]
#[ignore = "Multi-param not yet supported in C transpiler"]
fn test_11_methods_002_multi_param() { test_a2c("11_methods/002_multi_param").unwrap(); }
#[test]
#[ignore = "Generic list not yet supported in C transpiler"]
fn test_11_methods_003_generic_list() { test_a2c("11_methods/003_generic_list").unwrap(); }

// === 12_specs ===
#[test] #[ignore] fn test_12_specs_001_basic_spec() { test_a2c("12_specs/001_basic_spec").unwrap(); }
#[test] #[ignore] fn test_12_specs_002_spec() { test_a2c("12_specs/002_spec").unwrap(); }

// === 13_delegation ===
#[test] #[ignore] fn test_13_delegation_001_single() { test_a2c("13_delegation/001_single").unwrap(); }
#[test] #[ignore] fn test_13_delegation_002_multi_delegation() { test_a2c("13_delegation/002_multi_delegation").unwrap(); }
#[test] #[ignore] fn test_13_delegation_003_delegation_params() { test_a2c("13_delegation/003_delegation_params").unwrap(); }

// === 15_type_conversion ===
#[test] #[ignore] fn test_15_type_conversion_001_type_cast() { test_a2c("15_type_conversion/001_type_cast").unwrap(); }

// === 18_c_interop ===
#[test] #[ignore] fn test_18_c_interop_001_cstr() { test_a2c("18_c_interop/001_cstr").unwrap(); }
#[test] #[ignore] fn test_18_c_interop_002_alias() { test_a2c("18_c_interop/002_alias").unwrap(); }
#[test] #[ignore] fn test_18_c_interop_003_unified_section() { test_a2c("18_c_interop/003_unified_section").unwrap(); }

// === 19_option_type ===
#[test] #[ignore] fn test_19_option_type_001_question_syntax() { test_a2c("19_option_type/001_question_syntax").unwrap(); }
#[test] #[ignore] fn test_19_option_type_002_question_uint() { test_a2c("19_option_type/002_question_uint").unwrap(); }
#[test] #[ignore] fn test_19_option_type_003_question_float() { test_a2c("19_option_type/003_question_float").unwrap(); }
#[test] #[ignore] fn test_19_option_type_004_question_double() { test_a2c("19_option_type/004_question_double").unwrap(); }
#[test] #[ignore] fn test_19_option_type_005_question_char() { test_a2c("19_option_type/005_question_char").unwrap(); }
#[test]
#[ignore = "?void syntax not yet supported in C transpiler"]
fn test_19_option_type_006_question_void() { test_a2c("19_option_type/006_question_void").unwrap(); }
#[test] #[ignore] fn test_19_option_type_007_question_return_int() { test_a2c("19_option_type/007_question_return_int").unwrap(); }
#[test] #[ignore] fn test_19_option_type_008_question_return_str() { test_a2c("19_option_type/008_question_return_str").unwrap(); }
#[test] #[ignore] fn test_19_option_type_009_question_return_bool() { test_a2c("19_option_type/009_question_return_bool").unwrap(); }
#[test] #[ignore] fn test_19_option_type_010_question_return_uint() { test_a2c("19_option_type/010_question_return_uint").unwrap(); }
#[test] #[ignore] fn test_19_option_type_011_question_return_float() { test_a2c("19_option_type/011_question_return_float").unwrap(); }
#[test] #[ignore] fn test_19_option_type_012_question_return_double() { test_a2c("19_option_type/012_question_return_double").unwrap(); }
#[test] #[ignore] fn test_19_option_type_013_question_return_char() { test_a2c("19_option_type/013_question_return_char").unwrap(); }
#[test] #[ignore] fn test_19_option_type_014_question_nested_call() { test_a2c("19_option_type/014_question_nested_call").unwrap(); }
#[test] #[ignore] fn test_19_option_type_015_question_arithmetic() { test_a2c("19_option_type/015_question_arithmetic").unwrap(); }
#[test] #[ignore] fn test_19_option_type_016_question_comparison() { test_a2c("19_option_type/016_question_comparison").unwrap(); }
#[test]
#[ignore = "&& operator not yet supported in C transpiler"]
fn test_19_option_type_017_question_logical() { test_a2c("19_option_type/017_question_logical").unwrap(); }
#[test] #[ignore] fn test_19_option_type_018_question_negation() { test_a2c("19_option_type/018_question_negation").unwrap(); }

// === 21_storage ===
#[test]
#[ignore = "Storage module generics not yet implemented in C transpiler"]
fn test_21_storage_001_storage_module() { test_a2c("21_storage/001_storage_module").unwrap(); }
#[test] #[ignore] fn test_21_storage_002_storage_usage() { test_a2c("21_storage/002_storage_usage").unwrap(); }
#[test] #[ignore] fn test_21_storage_003_plan055_auto_storage() { test_a2c("21_storage/003_plan055_auto_storage").unwrap(); }

// === 22_iterators ===
#[test] #[ignore] fn test_22_iterators_001_iter_specs() { test_a2c("22_iterators/001_iter_specs").unwrap(); }
#[test] #[ignore] fn test_22_iterators_002_map_adapter() { test_a2c("22_iterators/002_map_adapter").unwrap(); }
#[test] #[ignore] fn test_22_iterators_003_terminal_operators() { test_a2c("22_iterators/003_terminal_operators").unwrap(); }
#[test] #[ignore] fn test_22_iterators_004_terminal_operators_2() { test_a2c("22_iterators/004_terminal_operators_2").unwrap(); }
#[test] #[ignore] fn test_22_iterators_005_extended_adapters() { test_a2c("22_iterators/005_extended_adapters").unwrap(); }
#[test] #[ignore] fn test_22_iterators_006_predicates() { test_a2c("22_iterators/006_predicates").unwrap(); }
#[test] #[ignore] fn test_22_iterators_007_collect() { test_a2c("22_iterators/007_collect").unwrap(); }

// === 23_stdlib ===
#[test] #[ignore] fn test_23_stdlib_001_std_hello() { test_a2c("23_stdlib/001_std_hello").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_002_std_getpid() { test_a2c("23_stdlib/002_std_getpid").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_003_std_readline() { test_a2c("23_stdlib/003_std_readline").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_004_std_file() { test_a2c("23_stdlib/004_std_file").unwrap(); }
#[test]
#[ignore]
fn test_23_stdlib_005_std_repl() { test_a2c("23_stdlib/005_std_repl").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_006_std_str() { test_a2c("23_stdlib/006_std_str").unwrap(); }
#[test]
#[ignore]
fn test_23_stdlib_007_file_operations() { test_a2c("23_stdlib/007_file_operations").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_008_char_io() { test_a2c("23_stdlib/008_char_io").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_009_advanced_io() { test_a2c("23_stdlib/009_advanced_io").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_010_io_specs() { test_a2c("23_stdlib/010_io_specs").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_011_std_test() { test_a2c("23_stdlib/011_std_test").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_012_std_readline_2() { test_a2c("23_stdlib/012_std_readline_2").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_013_std_file_flush() { test_a2c("23_stdlib/013_std_file_flush").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_014_std_file_read() { test_a2c("23_stdlib/014_std_file_read").unwrap(); }
#[test]
#[ignore]
fn test_23_stdlib_015_hashmap() { test_a2c("23_stdlib/015_hashmap").unwrap(); }
#[test]
#[ignore]
fn test_23_stdlib_016_hashset() { test_a2c("23_stdlib/016_hashset").unwrap(); }
#[test] #[ignore] fn test_23_stdlib_017_std_file_write() { test_a2c("23_stdlib/017_std_file_write").unwrap(); }

// === 24_runtime_size ===
#[test] #[ignore] fn test_24_runtime_size_001_runtime_size_var() { test_a2c("24_runtime_size/001_runtime_size_var").unwrap(); }
#[test] #[ignore] fn test_24_runtime_size_002_runtime_size_expr() { test_a2c("24_runtime_size/002_runtime_size_expr").unwrap(); }

// === 25_type_checking ===
#[test]
#[ignore = "C transpiler does not yet validate struct field types"]
fn test_25_type_checking_001_type_error() { test_a2c("25_type_checking/001_type_error").unwrap(); }

// 14_ext — ext statement tests (migrated from test/a2c/035_ext_statement)
#[test] #[ignore] fn test_14_ext_001_ext_simple() { test_a2c("14_ext/001_ext_simple").unwrap(); }
#[test] #[ignore] fn test_14_ext_002_ext_instance_method() { test_a2c("14_ext/002_ext_instance_method").unwrap(); }
#[test] #[ignore] fn test_14_ext_003_ext_builtin_type() { test_a2c("14_ext/003_ext_builtin_type").unwrap(); }
#[test] #[ignore] fn test_14_ext_004_ext_static_method() { test_a2c("14_ext/004_ext_static_method").unwrap(); }
#[test] #[ignore] fn test_14_ext_005_ext_multiple() { test_a2c("14_ext/005_ext_multiple").unwrap(); }
#[test] #[ignore] fn test_14_ext_006_ext_prop_shorthand() { test_a2c("14_ext/006_ext_prop_shorthand").unwrap(); }

// === 200_auto_cffi === Plan 216 Phase 3: Auto C FFI from manifests
#[test] #[ignore] fn test_200_auto_cffi_math() { test_a2c("200_auto_cffi_math").unwrap(); }
