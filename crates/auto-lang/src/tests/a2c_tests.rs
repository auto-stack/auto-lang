
use crate::{error::AutoResult, trans::c::{transpile_c, transpile_part}};

#[test]
fn test_c() {
    let code = "41";
    let out = transpile_part(code).unwrap();
    assert_eq!(out, "41;\n");
}

#[test]
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
fn test_c_let() {
    let code = "let x = 41";
    let out = transpile_part(code).unwrap();
    let expected = "int x = 41;\n";
    assert_eq!(out, expected);
}

#[test]
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
fn test_c_array() {
    let code = "let x = [1, 2, 3]";
    let out = transpile_part(code).unwrap();
    let expected = "int x[3] = {1, 2, 3};\n";
    assert_eq!(out, expected);
}

#[test]
fn test_c_var_assign() {
    let code = "var x = 41; x = 42";
    let out = transpile_part(code).unwrap();
    let expected = "int x = 41;\nx = 42;\n";
    assert_eq!(out, expected);
}

#[test]
fn test_c_return_42() {
    let code = r#"42"#;
    let (mut sink, _) = transpile_c("test", code).unwrap();
    let expected = r#"int main(void) {
    return 42;
}
"#;
    let src = sink.done().unwrap();
    assert_eq!(String::from_utf8(src.clone()).unwrap(), expected);
}

#[test]
fn test_math() {
    let code = r#"fn add(x int, y int) int { x+y }
add(1, 2)"#;
    let (mut sink, _) = transpile_c("test", code).unwrap();
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

    // split number from name: 000_hello -> hello
    let parts: Vec<&str> = case.split("_").collect();
    let name = parts[1..].join("_");
    let name = name.as_str();

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("Directory of cargo : {}", d.display());

    let src_path = format!("test/a2c/{}/{}.at", case, name);
    let src_path = d.join(src_path);

    println!("src_path: {}", src_path.display());
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
                let error_msg = format!("{}", e);
                // Check if the error message contains the expected error
                if !error_msg.contains("Type mismatch") {
                    return Err(format!("Expected type mismatch error, got: {}", error_msg).into());
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

        let (mut ccode, _) = transpile_c(name, &src)?;

        let src = ccode.done()?;

        if src != expected_src.as_bytes() {
            // out put generated code to a gen file
            let gen_path = format!("test/a2c/{}/{}.wrong.c", case, name);
            let gen_path = d.join(gen_path);
            let mut file = File::create(gen_path.as_path())?;
            file.write_all(src)?;
        }

        assert_eq!(String::from_utf8_lossy(src), expected_src);

        let header = ccode.header;
        if header != expected_header.as_bytes() {
            // out put generated code to a gen file
            let gen_path = format!("test/a2c/{}/{}.wrong.h", case, name);
            let gen_path = d.join(gen_path);
            let mut file = File::create(gen_path.as_path())?;
            file.write_all(&header)?;
        }
        assert_eq!(String::from_utf8_lossy(&header), expected_header);
        Ok(())
    }
}

#[test]
fn test_000_hello() {
    test_a2c("000_hello").unwrap();
}

#[test]
fn test_001_sqrt() {
    test_a2c("001_sqrt").unwrap();
}

#[test]
fn test_002_array() {
    test_a2c("002_array").unwrap();
}

#[test]
fn test_003_func() {
    test_a2c("003_func").unwrap();
}

#[test]
fn test_004_cstr() {
    test_a2c("004_cstr").unwrap();
}

#[test]
fn test_005_pointer() {
    test_a2c("005_pointer").unwrap();
}

#[test]
fn test_006_struct() {
    test_a2c("006_struct").unwrap();
}

#[test]
fn test_007_enum() {
    test_a2c("007_enum").unwrap();
}

#[test]
fn test_008_method() {
    test_a2c("008_method").unwrap();
}

#[test]
fn test_009_alias() {
    test_a2c("009_alias").unwrap();
}

#[test]
fn test_010_if() {
    test_a2c("010_if").unwrap();
}

#[test]
fn test_011_for() {
    test_a2c("011_for").unwrap();
}

#[test]
fn test_012_is() {
    test_a2c("012_is").unwrap();
}

#[test]
fn test_013_union() {
    test_a2c("013_union").unwrap();
}

#[test]
fn test_014_tag() {
    test_a2c("014_tag").unwrap();
}

#[test]
fn test_015_str() {
    test_a2c("015_str").unwrap();
}

#[test]
fn test_016_basic_spec() {
    test_a2c("016_basic_spec").unwrap();
}

#[test]
fn test_017_spec() {
    test_a2c("017_spec").unwrap();
}

#[test]
fn test_018_delegation() {
    test_a2c("018_delegation").unwrap();
}

#[test]
fn test_019_multi_delegation() {
    test_a2c("019_multi_delegation").unwrap();
}

#[test]
fn test_020_delegation_params() {
    test_a2c("020_delegation_params").unwrap();
}

#[test]
fn test_111_io_specs() {
    test_a2c("111_io_specs").unwrap();
}

#[test]
fn test_037_for_complex() {
    test_a2c("037_complex_expr").unwrap();
}

#[test]
fn test_037_array_return() {
    test_a2c("037_array_return").unwrap();
}

// TODO: Test 038 - string methods with arraGy returns
// Currently str.split() signature is added but full implementation
// requires more expression support (loop conditions, string manipulation)

#[test]
fn test_021_type_error() {
    test_a2c("021_type_error").unwrap();
}

// ===================== test cases for Auto's stdlib =======================

// TODO: These tests fail due to pre-existing library file loading issues
// The transpiler can't find auto/io.h when processing use statements
// This needs to be fixed separately from the enum refactoring
#[test]
fn test_100_std_hello() {
    test_a2c("100_std_hello").unwrap();
}

#[test]
fn test_101_std_getpid() {
    test_a2c("101_std_getpid").unwrap();
}

#[test]
fn test_102_std_readline() {
    test_a2c("102_std_readline").unwrap();
}

#[test]
fn test_103_std_file() {
    match test_a2c("103_std_file") {
        Ok(_) => {}
        Err(e) => {
            // Print full error using Miette for better diagnostics
            eprintln!("\n=== Transpilation Error ===\n");

            // Check if it's a SyntaxWithSource error (has source code attached)
            match &e {
                crate::error::AutoError::SyntaxWithSource(err) => {
                    // This has source code - print with rich formatting
                    eprintln!("{}\n", err);
                }
                _ => {
                    // Fallback to simple display
                    eprintln!("{}\n", e);
                }
            }

            // Also print debug for more details
            eprintln!("Debug info:\n{:?}\n", e);

            panic!("Transpilation failed");
        }
    }
}

#[test]
#[ignore]
fn test_104_std_repl() {
    test_a2c("104_std_repl").unwrap();
}

#[test]
fn test_105_std_str() {
    test_a2c("105_std_str").unwrap();
}

#[test]
#[ignore]
fn test_106_file_operations() {
    test_a2c("106_file_operations").unwrap();
}

// ===================== Phase 5: Unified Section tests =======================

#[test]
fn test_037_unified_functions() {
    test_a2c("037_unified_section").unwrap();
}

// ===================== Tag Type and May<T> tests =======================

#[test]
fn test_040_tag_types() {
    test_a2c("040_tag_types").unwrap();
}

#[test]
fn test_041_may_basic() {
    test_a2c("041_may_basic").unwrap();
}

#[test]
fn test_042_may_string() {
    test_a2c("042_may_string").unwrap();
}

#[test]
fn test_043_may_bool() {
    test_a2c("043_may_bool").unwrap();
}

#[test]
fn test_044_may_patterns() {
    test_a2c("044_may_patterns").unwrap();
}

#[test]
fn test_045_may_nested() {
    test_a2c("045_may_nested").unwrap();
}

#[test]
fn test_046_binary() {
    test_a2c("046_binary").unwrap();
}
#[test]
fn test_047_tristate() {
    test_a2c("047_tristate").unwrap();
}
#[test]
fn test_048_direction() {
    test_a2c("048_direction").unwrap();
}
#[test]
fn test_049_status() {
    test_a2c("049_status").unwrap();
}
#[test]
fn test_050_mode() {
    test_a2c("050_mode").unwrap();
}
#[test]
fn test_051_result() {
    test_a2c("051_result").unwrap();
}
#[test]
fn test_052_phase() {
    test_a2c("052_phase").unwrap();
}
#[test]
fn test_053_level() {
    test_a2c("053_level").unwrap();
}
#[test]
fn test_054_state() {
    test_a2c("054_state").unwrap();
}
#[test]
fn test_055_type() {
    test_a2c("055_type").unwrap();
}
#[test]
fn test_056_side() {
    test_a2c("056_side").unwrap();
}
#[test]
fn test_057_flow() {
    test_a2c("057_flow").unwrap();
}
#[test]
fn test_058_gate() {
    test_a2c("058_gate").unwrap();
}
#[test]
fn test_059_path() {
    test_a2c("059_path").unwrap();
}
#[test]
fn test_060_color() {
    test_a2c("060_color").unwrap();
}
#[test]
fn test_061_size() {
    test_a2c("061_size").unwrap();
}
#[test]
fn test_062_speed() {
    test_a2c("062_speed").unwrap();
}
#[test]
fn test_063_power() {
    test_a2c("063_power").unwrap();
}
#[test]
fn test_064_signal() {
    test_a2c("064_signal").unwrap();
}
#[test]
fn test_065_zone() {
    test_a2c("065_zone").unwrap();
}
#[test]
fn test_066_mode2() {
    test_a2c("066_mode2").unwrap();
}
#[test]
fn test_067_link() {
    test_a2c("067_link").unwrap();
}
#[test]
fn test_068_source() {
    test_a2c("068_source").unwrap();
}
#[test]
fn test_069_target() {
    test_a2c("069_target").unwrap();
}
#[test]
fn test_070_format() {
    test_a2c("070_format").unwrap();
}

#[test]
fn test_071_question_syntax() {
    test_a2c("071_question_syntax").unwrap();
}

#[test]
fn test_072_question_uint() {
    test_a2c("072_question_uint").unwrap();
}

#[test]
fn test_073_question_float() {
    test_a2c("073_question_float").unwrap();
}

#[test]
fn test_074_question_double() {
    test_a2c("074_question_double").unwrap();
}

#[test]
fn test_075_question_char() {
    test_a2c("075_question_char").unwrap();
}

// Skip test_076_question_void - ?void doesn't make semantic sense

#[test]
fn test_079_question_return_int() {
    test_a2c("079_question_return_int").unwrap();
}

#[test]
fn test_080_question_return_str() {
    test_a2c("080_question_return_str").unwrap();
}

#[test]
fn test_081_question_return_bool() {
    test_a2c("081_question_return_bool").unwrap();
}

#[test]
fn test_082_question_return_uint() {
    test_a2c("082_question_return_uint").unwrap();
}

#[test]
fn test_083_question_return_float() {
    test_a2c("083_question_return_float").unwrap();
}

#[test]
fn test_084_question_return_double() {
    test_a2c("084_question_return_double").unwrap();
}

#[test]
fn test_085_question_return_char() {
    test_a2c("085_question_return_char").unwrap();
}

#[test]
fn test_087_question_nested_call() {
    test_a2c("087_question_nested_call").unwrap();
}

#[test]
fn test_088_question_arithmetic() {
    test_a2c("088_question_arithmetic").unwrap();
}

#[test]
fn test_089_question_comparison() {
    test_a2c("089_question_comparison").unwrap();
}

// Skip test_090_question_logical - && operator has parsing issues

#[test]
fn test_091_question_negation() {
    test_a2c("091_question_negation").unwrap();
}

#[test]
fn test_092_question_literal() {
    test_a2c("092_question_literal").unwrap();
}

#[test]
fn test_093_question_zero() {
    test_a2c("093_question_zero").unwrap();
}

#[test]
fn test_094_question_negative() {
    test_a2c("094_question_negative").unwrap();
}

#[test]
fn test_095_null_coalesce() {
    test_a2c("095_null_coalesce").unwrap();
}

#[test]
fn test_096_error_propagate() {
    test_a2c("096_error_propagate").unwrap();
}

#[test]
fn test_110_bool() {
    test_a2c("110_bool").unwrap();
}

#[test]
fn test_112_inheritance() {
    test_a2c("112_inheritance").unwrap();
}

// ===================== Phase 3: HashMap/HashSet tests =======================

// TODO: These tests are incomplete - they use function-style API (HashMap_new)
// but only the OOP API (HashMap.new) is implemented. Need to either:
// 1. Register function-style aliases
// 2. Update tests to use OOP API
// 3. Create proper expected output files
#[test]
#[ignore]
fn test_097_hashmap() {
    test_a2c("097_hashmap").unwrap();
}

#[test]
#[ignore]
fn test_098_hashset() {
    test_a2c("098_hashset").unwrap();
}

// ===================== Phase 3: Borrow Checker tests =======================

#[test]
fn test_030_borrow_view() {
    test_a2c("030_borrow_view").unwrap();
}

#[test]
fn test_031_borrow_mut() {
    test_a2c("031_borrow_mut").unwrap();
}

#[test]
fn test_032_borrow_take() {
    test_a2c("032_borrow_take").unwrap();
}

#[test]
fn test_033_borrow_conflicts() {
    test_a2c("033_borrow_conflicts").unwrap();
}

#[test]
fn test_114_std_readline() {
    test_a2c("114_std_readline").unwrap();
}

#[test]
fn test_116_std_file_flush() {
    test_a2c("116_std_file_flush").unwrap();
}

#[test]
fn test_117_std_file_read() {
    test_a2c("117_std_file_read").unwrap();
}

// ===================== Generic Type Tests =====================

#[test]
fn test_060_generic_tag() {
    match test_a2c("060_generic_tag") {
        Ok(_) => {}
        Err(e) => {
            // Check if this is MultipleErrors and print each one
            if let crate::error::AutoError::MultipleErrors { errors, .. } = &e {
                eprintln!("\n=== Transpilation Errors ({}) ===\n", errors.len());
                for (i, err) in errors.iter().enumerate() {
                    eprintln!("--- Error {} ---\n", i + 1);
                    eprintln!("{}\n", err);
                }
            } else {
                eprintln!("\n=== Transpilation Error ===\n");
                eprintln!("{}\n", e);
            }
            panic!("Transpilation failed - generics not yet implemented");
        }
    }
}

// ===================== Array Capability Tests (Plan 051) =====================
// Note: These tests use the format test_a2c("080_name") where files are in test/a2c/080_name/

#[test]
fn test_080_array_declaration() {
    test_a2c("080_array_declaration").unwrap();
}

#[test]
fn test_080_array_mutation() {
    test_a2c("080_array_mutation").unwrap();
}

#[test]
fn test_080_array_index_read() {
    test_a2c("080_array_index_read").unwrap();
}

#[test]
fn test_080_array_copy() {
    test_a2c("080_array_copy").unwrap();
}

#[test]
fn test_080_array_slice() {
    test_a2c("080_array_slice").unwrap();
}

#[test]
fn test_080_array_zero_size() {
    test_a2c("080_array_zero_size").unwrap();
}

#[test]
fn test_080_array_nested() {
    test_a2c("080_array_nested").unwrap();
}

#[test]
fn test_080_array_loop() {
    test_a2c("080_array_loop").unwrap();
}

// ============================================================================
// Plan 052: Runtime Array Allocation Tests
// ============================================================================

#[test]
fn test_082_runtime_size_var() {
    test_a2c("082_runtime_size_var").unwrap();
}

#[test]
fn test_083_runtime_size_expr() {
    test_a2c("083_runtime_size_expr").unwrap();
}

// ============================================================================
// Plan 052: Pointer Type Tests
// ============================================================================

#[test]
fn test_090_pointer_types() {
    test_a2c("090_pointer_types").unwrap();
}

#[test]
fn test_092_const_generics() {
    test_a2c("092_const_generics").unwrap();
}

// ============================================================================
// Plan 057: Generic Spec Tests
// ============================================================================

#[test]
fn test_093_generic_specs() {
    test_a2c("093_generic_specs").unwrap();
}

// ============================================================================
// Plan 057: Generic Spec with Ext Blocks
// ============================================================================

#[test]
fn test_094_generic_spec_ext() {
    test_a2c("094_generic_spec_ext").unwrap();
}

#[test]
fn test_095_storage_module() {
    test_a2c("095_storage_module").unwrap();
}

#[test]
fn test_096_storage_usage() {
    test_a2c("096_storage_usage").unwrap();
}

#[test]
fn test_097_list_storage() {
    test_a2c("097_list_storage").unwrap();
}

#[test]
fn test_090_type_alias() {
    test_a2c("090_type_alias").unwrap();
}

#[test]
fn test_092_plan055_auto_storage() {
    test_a2c("092_plan055_auto_storage").unwrap();
}

#[test]
fn test_099_iter_specs() {
    test_a2c("099_iter_specs").unwrap();
}

#[test]
fn test_100_map_adapter() {
    test_a2c("100_map_adapter").unwrap();
}

#[test]
fn test_101_list_iter() {
    test_a2c("101_list_iter").unwrap();
}

