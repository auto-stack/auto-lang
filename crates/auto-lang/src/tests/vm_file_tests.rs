// Plan 177: VM File-Based Test Framework
// Similar to a2r_tests, reads .at files from test/vm/ directory
// Supports three assertion types:
//   .expected.out    — stdout output from print()
//   .expected.result — return value (last expression)
//   .expected.error  — expected runtime error

use crate::error::AutoResult;
use crate::{run, run_with_capture};
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_vm(case: &str) -> AutoResult<()> {
    // Parse test case name: "01_basics/001_hello" -> "hello"
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = read_to_string(d.join(format!("test/vm/{}/{}.at", case, name)))?;

    // Check .expected.error — expect runtime error
    let err_path = d.join(format!("test/vm/{}/{}.expected.error", case, name));
    if err_path.is_file() {
        let result = run(&src);
        assert!(
            result.is_err(),
            "Expected error but got: {:?}",
            result
        );
        return Ok(());
    }

    // Execute with stdout capture
    let (result, stdout) = run_with_capture(&src)?;

    // Check .expected.out — stdout output
    let out_path = d.join(format!("test/vm/{}/{}.expected.out", case, name));
    if out_path.is_file() {
        let expected_out = read_to_string(&out_path)?;
        if stdout != expected_out {
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.out", case, name));
            std::fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, expected_out);
    }

    // Check .expected.result — return value
    let res_path = d.join(format!("test/vm/{}/{}.expected.result", case, name));
    if res_path.is_file() {
        let expected_res = read_to_string(&res_path)?;
        if result != expected_res {
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.result", case, name));
            std::fs::write(&wrong_path, &result)?;
        }
        assert_eq!(result, expected_res);
    }

    Ok(())
}

// === 01_basics ===
#[test] fn test_01_basics_001_hello() { test_vm("01_basics/001_hello").unwrap(); }
#[test] fn test_01_basics_002_arithmetic() { test_vm("01_basics/002_arithmetic").unwrap(); }
#[test] fn test_01_basics_003_str_upper() { test_vm("01_basics/003_str_upper").unwrap(); }

// === 01_basics (continued) ===
#[test] fn test_01_basics_004_uint() { test_vm("01_basics/004_uint").unwrap(); }
#[test] fn test_01_basics_005_byte() { test_vm("01_basics/005_byte").unwrap(); }
#[test] fn test_01_basics_006_unary() { test_vm("01_basics/006_unary").unwrap(); }
#[test] fn test_01_basics_007_group() { test_vm("01_basics/007_group").unwrap(); }
#[test] fn test_01_basics_008_comp() { test_vm("01_basics/008_comp").unwrap(); }
#[test] fn test_01_basics_009_comp_false() { test_vm("01_basics/009_comp_false").unwrap(); }
#[test] fn test_01_basics_010_eq() { test_vm("01_basics/010_eq").unwrap(); }
#[test] fn test_01_basics_011_eq_false() { test_vm("01_basics/011_eq_false").unwrap(); }

// === 02_bit_ops ===
#[test] fn test_02_bit_ops_001_binary_literal() { test_vm("02_bit_ops/001_binary_literal").unwrap(); }
#[test] fn test_02_bit_ops_002_bitwise_ops() { test_vm("02_bit_ops/002_bitwise_ops").unwrap(); }
#[test] fn test_02_bit_ops_003_bit_scan() { test_vm("02_bit_ops/003_bit_scan").unwrap(); }
#[test] fn test_02_bit_ops_004_not_flip() { test_vm("02_bit_ops/004_not_flip").unwrap(); }
#[test] fn test_02_bit_ops_005_bitfield() { test_vm("02_bit_ops/005_bitfield").unwrap(); }

// === 03_variables ===
#[test] fn test_03_variables_001_var() { test_vm("03_variables/001_var").unwrap(); }
#[test] fn test_03_variables_002_var_assign() { test_vm("03_variables/002_var_assign").unwrap(); }
#[test] fn test_03_variables_003_var_mut() { test_vm("03_variables/003_var_mut").unwrap(); }
#[test] fn test_03_variables_004_var_if() { test_vm("03_variables/004_var_if").unwrap(); }
#[test] fn test_03_variables_005_let_binding() { test_vm("03_variables/005_let_binding").unwrap(); }
#[test] fn test_03_variables_006_let_asn_error() { test_vm("03_variables/006_let_asn_error").unwrap(); }
#[test] fn test_03_variables_007_var_reassignment() { test_vm("03_variables/007_var_reassignment").unwrap(); }
#[test] fn test_03_variables_008_simple_block() { test_vm("03_variables/008_simple_block").unwrap(); }

// === 04_control_flow ===
#[test] fn test_04_control_flow_001_if_true() { test_vm("04_control_flow/001_if_true").unwrap(); }
#[test] fn test_04_control_flow_002_if_false() { test_vm("04_control_flow/002_if_false").unwrap(); }
#[test] fn test_04_control_flow_003_if_else_if() { test_vm("04_control_flow/003_if_else_if").unwrap(); }
#[test] fn test_04_control_flow_004_if_with_bool() { test_vm("04_control_flow/004_if_with_bool").unwrap(); }
#[test] fn test_04_control_flow_005_if_in_array() { test_vm("04_control_flow/005_if_in_array").unwrap(); }
#[test] fn test_04_control_flow_006_is_stmt() { test_vm("04_control_flow/006_is_stmt").unwrap(); }
#[test] fn test_04_control_flow_007_asn_upper() { test_vm("04_control_flow/007_asn_upper").unwrap(); }
#[test] fn test_04_control_flow_008_is_or_pattern() { test_vm("04_control_flow/008_is_or_pattern").unwrap(); }
#[test] fn test_04_control_flow_009_is_or_groups() { test_vm("04_control_flow/009_is_or_groups").unwrap(); }
#[test] fn test_04_control_flow_010_as_int_to_float() { test_vm("04_control_flow/010_as_int_to_float").unwrap(); }
#[test] fn test_04_control_flow_011_as_float_to_int() { test_vm("04_control_flow/011_as_float_to_int").unwrap(); }
#[test] fn test_04_control_flow_012_to_str() { test_vm("04_control_flow/012_to_str").unwrap(); }
#[test] fn test_04_control_flow_013_to_int_from_str() { test_vm("04_control_flow/013_to_int_from_str").unwrap(); }

// === 05_loops ===
#[test] fn test_05_loops_001_for_range() { test_vm("05_loops/001_for_range").unwrap(); }
#[test] fn test_05_loops_002_range_inclusive() { test_vm("05_loops/002_range_inclusive").unwrap(); }
#[test] fn test_05_loops_003_range_literal() { test_vm("05_loops/003_range_literal").unwrap(); }
#[test] fn test_05_loops_004_for_each_object() { test_vm("05_loops/004_for_each_object").unwrap(); }

// === 06_arrays ===
#[test] fn test_06_arrays_001_array_literal() { test_vm("06_arrays/001_array_literal").unwrap(); }
#[test] fn test_06_arrays_002_array_index() { test_vm("06_arrays/002_array_index").unwrap(); }
#[test] fn test_06_arrays_003_array_update() { test_vm("06_arrays/003_array_update").unwrap(); }
#[test] fn test_06_arrays_004_array_of_objects() { test_vm("06_arrays/004_array_of_objects").unwrap(); }
#[test] fn test_06_arrays_005_array_multiple_mutations() { test_vm("06_arrays/005_array_multiple_mutations").unwrap(); }

// === 07_objects ===
#[test] fn test_07_objects_001_object_field() { test_vm("07_objects/001_object_field").unwrap(); }
#[test] fn test_07_objects_002_object_field_int_key() { test_vm("07_objects/002_object_field_int_key").unwrap(); }
#[test] fn test_07_objects_003_object_field_bool_key() { test_vm("07_objects/003_object_field_bool_key").unwrap(); }
#[test] fn test_07_objects_004_obj_set() { test_vm("07_objects/004_obj_set").unwrap(); }
#[test] fn test_07_objects_005_nested_object() { test_vm("07_objects/005_nested_object").unwrap(); }
#[test] fn test_07_objects_006_nested_object_y() { test_vm("07_objects/006_nested_object_y").unwrap(); }
#[test] fn test_07_objects_007_json() { test_vm("07_objects/007_json").unwrap(); }
#[test] fn test_07_objects_008_last_block_or_object() { test_vm("07_objects/008_last_block_or_object").unwrap(); }
#[test] fn test_07_objects_009_multiple_field_mutations() { test_vm("07_objects/009_multiple_field_mutations").unwrap(); }

// === 08_strings ===
#[test] fn test_08_strings_001_fstr() { test_vm("08_strings/001_fstr").unwrap(); }
#[test] fn test_08_strings_002_fstr_expr() { test_vm("08_strings/002_fstr_expr").unwrap(); }
#[test] fn test_08_strings_003_str_index() { test_vm("08_strings/003_str_index").unwrap(); }
#[test] fn test_08_strings_004_int_to_str() { test_vm("08_strings/004_int_to_str").unwrap(); }
#[test] fn test_08_strings_005_str_import_basic() { test_vm("08_strings/005_str_import_basic").unwrap(); }
#[test] fn test_08_strings_006_str_import_case() { test_vm("08_strings/006_str_import_case").unwrap(); }
#[test] fn test_08_strings_007_str_import_search() { test_vm("08_strings/007_str_import_search").unwrap(); }
#[test] fn test_08_strings_008_str_import_transform() { test_vm("08_strings/008_str_import_transform").unwrap(); }

// === 09_functions ===
#[test] fn test_09_functions_001_fn_simple() { test_vm("09_functions/001_fn_simple").unwrap(); }
#[test] fn test_09_functions_002_fn_named_args() { test_vm("09_functions/002_fn_named_args").unwrap(); }
#[test] fn test_09_functions_003_fn_multiple() { test_vm("09_functions/003_fn_multiple").unwrap(); }
#[test] fn test_09_functions_004_fn_nested() { test_vm("09_functions/004_fn_nested").unwrap(); }
#[test] fn test_09_functions_005_fn_in_expr() { test_vm("09_functions/005_fn_in_expr").unwrap(); }
#[test] fn test_09_functions_006_fn_local_var() { test_vm("09_functions/006_fn_local_var").unwrap(); }
#[test] fn test_09_functions_007_closure() { test_vm("09_functions/007_closure").unwrap(); }
#[test] fn test_09_functions_008_closure_typed() { test_vm("09_functions/008_closure_typed").unwrap(); }
#[test] fn test_09_functions_009_forward_decl() { test_vm("09_functions/009_forward_decl").unwrap(); }

// === 10_types ===
#[test] fn test_10_types_001_type_compose() { test_vm("10_types/001_type_compose").unwrap(); }
#[test] fn test_10_types_002_int_enum() { test_vm("10_types/002_int_enum").unwrap(); }
#[test] fn test_10_types_003_generic_instantiation() { test_vm("10_types/003_generic_instantiation").unwrap(); }
#[test] fn test_10_types_004_generic_field_x() { test_vm("10_types/004_generic_field_x").unwrap(); }
#[test] fn test_10_types_005_generic_field_y() { test_vm("10_types/005_generic_field_y").unwrap(); }
#[test] fn test_10_types_006_field_addition() { test_vm("10_types/006_field_addition").unwrap(); }
#[test] fn test_10_types_007_type_instance_prop() { test_vm("10_types/007_type_instance_prop").unwrap(); }
#[test] fn test_10_types_008_nested_type_instance() { test_vm("10_types/008_nested_type_instance").unwrap(); }
#[test] fn test_10_types_009_access_fields_in_method() { test_vm("10_types/009_access_fields_in_method").unwrap(); }
#[test] fn test_10_types_010_ext_method() { test_vm("10_types/010_ext_method").unwrap(); }
#[test] fn test_10_types_011_enum_is_match() { test_vm("10_types/011_enum_is_match").unwrap(); }
#[test] fn test_10_types_012_enum_dot_match() { test_vm("10_types/012_enum_dot_match").unwrap(); }
#[test] fn test_10_types_013_fn_return_obj() { test_vm("10_types/013_fn_return_obj").unwrap(); }

// === 11_compound_ops ===
#[test] fn test_11_compound_ops_001_add_eq() { test_vm("11_compound_ops/001_add_eq").unwrap(); }
#[test] fn test_11_compound_ops_002_sub_eq() { test_vm("11_compound_ops/002_sub_eq").unwrap(); }
#[test] fn test_11_compound_ops_003_mul_eq() { test_vm("11_compound_ops/003_mul_eq").unwrap(); }
#[test] fn test_11_compound_ops_004_div_eq() { test_vm("11_compound_ops/004_div_eq").unwrap(); }
#[test] fn test_11_compound_ops_005_chained() { test_vm("11_compound_ops/005_chained").unwrap(); }
#[test] fn test_11_compound_ops_006_div_eq_oneline() { test_vm("11_compound_ops/006_div_eq_oneline").unwrap(); }

// === 12_type_coercion ===
#[test] fn test_12_type_coercion_001_int_plus_float() { test_vm("12_type_coercion/001_int_plus_float").unwrap(); }
#[test] fn test_12_type_coercion_002_float_plus_int() { test_vm("12_type_coercion/002_float_plus_int").unwrap(); }
#[test] fn test_12_type_coercion_003_int_times_float() { test_vm("12_type_coercion/003_int_times_float").unwrap(); }
#[test] fn test_12_type_coercion_004_float_times_int() { test_vm("12_type_coercion/004_float_times_int").unwrap(); }
#[test] fn test_12_type_coercion_005_complex() { test_vm("12_type_coercion/005_complex").unwrap(); }
#[test] fn test_12_type_coercion_006_with_variable() { test_vm("12_type_coercion/006_with_variable").unwrap(); }

// === 13_collections ===
#[test] fn test_13_collections_001_hashmap_new() { test_vm("13_collections/001_hashmap_new").unwrap(); }
#[test] fn test_13_collections_002_hashmap_insert_str() { test_vm("13_collections/002_hashmap_insert_str").unwrap(); }
#[test] fn test_13_collections_003_hashmap_insert_int() { test_vm("13_collections/003_hashmap_insert_int").unwrap(); }
#[test] fn test_13_collections_004_hashmap_contains() { test_vm("13_collections/004_hashmap_contains").unwrap(); }
#[test] fn test_13_collections_005_hashmap_size() { test_vm("13_collections/005_hashmap_size").unwrap(); }
#[test] fn test_13_collections_006_hashmap_remove() { test_vm("13_collections/006_hashmap_remove").unwrap(); }
#[test] fn test_13_collections_007_hashmap_clear() { test_vm("13_collections/007_hashmap_clear").unwrap(); }
#[test] fn test_13_collections_008_hashset_new() { test_vm("13_collections/008_hashset_new").unwrap(); }
#[test] fn test_13_collections_009_hashset_insert() { test_vm("13_collections/009_hashset_insert").unwrap(); }
#[test] fn test_13_collections_010_hashset_duplicate() { test_vm("13_collections/010_hashset_duplicate").unwrap(); }
#[test] fn test_13_collections_011_hashset_remove() { test_vm("13_collections/011_hashset_remove").unwrap(); }
#[test] fn test_13_collections_012_hashset_size() { test_vm("13_collections/012_hashset_size").unwrap(); }
#[test] fn test_13_collections_013_hashset_clear() { test_vm("13_collections/013_hashset_clear").unwrap(); }
#[test] fn test_13_collections_014_sb_new() { test_vm("13_collections/014_sb_new").unwrap(); }
#[test] fn test_13_collections_015_sb_append() { test_vm("13_collections/015_sb_append").unwrap(); }
#[test] fn test_13_collections_016_sb_append_char() { test_vm("13_collections/016_sb_append_char").unwrap(); }
#[test] fn test_13_collections_017_sb_append_int() { test_vm("13_collections/017_sb_append_int").unwrap(); }
#[test] fn test_13_collections_018_sb_len() { test_vm("13_collections/018_sb_len").unwrap(); }
#[test] fn test_13_collections_019_sb_clear() { test_vm("13_collections/019_sb_clear").unwrap(); }
#[test] fn test_13_collections_020_list_new() { test_vm("13_collections/020_list_new").unwrap(); }
#[test] fn test_13_collections_021_list_push_pop() { test_vm("13_collections/021_list_push_pop").unwrap(); }
#[test] fn test_13_collections_022_list_push_pop_multi() { test_vm("13_collections/022_list_push_pop_multi").unwrap(); }
#[test] fn test_13_collections_023_list_len() { test_vm("13_collections/023_list_len").unwrap(); }
#[test] fn test_13_collections_024_list_is_empty() { test_vm("13_collections/024_list_is_empty").unwrap(); }
#[test] fn test_13_collections_025_list_clear() { test_vm("13_collections/025_list_clear").unwrap(); }
#[test] fn test_13_collections_026_list_get_set() { test_vm("13_collections/026_list_get_set").unwrap(); }
#[test] fn test_13_collections_027_list_insert_remove() { test_vm("13_collections/027_list_insert_remove").unwrap(); }
#[test] fn test_13_collections_028_list_reserve() { test_vm("13_collections/028_list_reserve").unwrap(); }
#[test] fn test_13_collections_029_list_comprehensive() { test_vm("13_collections/029_list_comprehensive").unwrap(); }
#[test] fn test_13_collections_030_list_multi_ops() { test_vm("13_collections/030_list_multi_ops").unwrap(); }
#[test] fn test_13_collections_031_list_index() { test_vm("13_collections/031_list_index").unwrap(); }
#[test] fn test_13_collections_032_list_varargs() { test_vm("13_collections/032_list_varargs").unwrap(); }
#[test] fn test_13_collections_033_list_varargs_empty() { test_vm("13_collections/033_list_varargs_empty").unwrap(); }
#[test] fn test_13_collections_034_list_for_iteration() { test_vm("13_collections/034_list_for_iteration").unwrap(); }
#[test] fn test_13_collections_035_list_for_empty() { test_vm("13_collections/035_list_for_empty").unwrap(); }

// === 14_borrow ===
#[test] fn test_14_borrow_001_view_basic() { test_vm("14_borrow/001_view_basic").unwrap(); }
#[test] fn test_14_borrow_002_view_multiple() { test_vm("14_borrow/002_view_multiple").unwrap(); }
#[test] fn test_14_borrow_003_mut_basic() { test_vm("14_borrow/003_mut_basic").unwrap(); }
#[test] fn test_14_borrow_004_move_basic() { test_vm("14_borrow/004_move_basic").unwrap(); }
#[test] fn test_14_borrow_005_view_preserves() { test_vm("14_borrow/005_view_preserves").unwrap(); }
#[test] fn test_14_borrow_006_nested_view() { test_vm("14_borrow/006_nested_view").unwrap(); }
#[test] fn test_14_borrow_007_borrow_arithmetic() { test_vm("14_borrow/007_borrow_arithmetic").unwrap(); }
#[test] fn test_14_borrow_008_view_in_array() { test_vm("14_borrow/008_view_in_array").unwrap(); }
#[test] fn test_14_borrow_009_view_in_expr() { test_vm("14_borrow/009_view_in_expr").unwrap(); }
#[test] fn test_14_borrow_010_borrow_diff_types() { test_vm("14_borrow/010_borrow_diff_types").unwrap(); }
#[test] fn test_14_borrow_011_move_chaining() { test_vm("14_borrow/011_move_chaining").unwrap(); }
#[test] fn test_14_borrow_012_str_sliceview() { test_vm("14_borrow/012_str_sliceview").unwrap(); }
#[test] fn test_14_borrow_013_str_slice_multi() { test_vm("14_borrow/013_str_slice_multi").unwrap(); }
#[test] fn test_14_borrow_014_str_slice_nested() { test_vm("14_borrow/014_str_slice_nested").unwrap(); }
#[test] fn test_14_borrow_015_str_slice_in_array() { test_vm("14_borrow/015_str_slice_in_array").unwrap(); }
#[test] fn test_14_borrow_016_str_slice_take() { test_vm("14_borrow/016_str_slice_take").unwrap(); }
#[test] fn test_14_borrow_017_str_slice_mixed() { test_vm("14_borrow/017_str_slice_mixed").unwrap(); }
#[test] fn test_14_borrow_018_str_slice_preserves() { test_vm("14_borrow/018_str_slice_preserves").unwrap(); }

// === 15_nested_mutation ===
#[test] fn test_15_nested_mutation_001_object_field() { test_vm("15_nested_mutation/001_object_field").unwrap(); }
#[test] fn test_15_nested_mutation_002_array_element() { test_vm("15_nested_mutation/002_array_element").unwrap(); }
#[test] fn test_15_nested_mutation_003_multiple_fields() { test_vm("15_nested_mutation/003_multiple_fields").unwrap(); }
#[test] fn test_15_nested_mutation_004_multiple_array() { test_vm("15_nested_mutation/004_multiple_array").unwrap(); }
#[test] fn test_15_nested_mutation_005_type_field() { test_vm("15_nested_mutation/005_type_field").unwrap(); }
#[test] fn test_15_nested_mutation_006_nested_object() { test_vm("15_nested_mutation/006_nested_object").unwrap(); }
#[test] fn test_15_nested_mutation_007_array_of_obj_field() { test_vm("15_nested_mutation/007_array_of_obj_field").unwrap(); }
#[test] fn test_15_nested_mutation_008_obj_array_element() { test_vm("15_nested_mutation/008_obj_array_element").unwrap(); }
#[test] fn test_15_nested_mutation_009_nested_array() { test_vm("15_nested_mutation/009_nested_array").unwrap(); }
#[test] fn test_15_nested_mutation_010_type_nested_field() { test_vm("15_nested_mutation/010_type_nested_field").unwrap(); }
#[test] fn test_15_nested_mutation_011_three_level() { test_vm("15_nested_mutation/011_three_level").unwrap(); }
#[test] fn test_15_nested_mutation_012_deep_array_obj() { test_vm("15_nested_mutation/012_deep_array_obj").unwrap(); }
#[test] fn test_15_nested_mutation_013_structure_preserve() { test_vm("15_nested_mutation/013_structure_preserve").unwrap(); }
#[test] fn test_15_nested_mutation_014_out_of_bounds_error() { test_vm("15_nested_mutation/014_out_of_bounds_error").unwrap(); }

// === 16_option_result ===
#[test] fn test_16_option_result_001_option_type() { test_vm("16_option_result/001_option_type").unwrap(); }
#[test] fn test_16_option_result_002_result_type() { test_vm("16_option_result/002_result_type").unwrap(); }
#[test] fn test_16_option_result_003_none_literal() { test_vm("16_option_result/003_none_literal").unwrap(); }
#[test] fn test_16_option_result_004_some_ctor() { test_vm("16_option_result/004_some_ctor").unwrap(); }
#[test] fn test_16_option_result_005_ok_ctor() { test_vm("16_option_result/005_ok_ctor").unwrap(); }
#[test] fn test_16_option_result_006_err_ctor() { test_vm("16_option_result/006_err_ctor").unwrap(); }
#[test] fn test_16_option_result_007_propagate_some() { test_vm("16_option_result/007_propagate_some").unwrap(); }
#[test] fn test_16_option_result_008_propagate_none() { test_vm("16_option_result/008_propagate_none").unwrap(); }
#[test] fn test_16_option_result_009_propagate_ok() { test_vm("16_option_result/009_propagate_ok").unwrap(); }
#[test] fn test_16_option_result_010_propagate_err() { test_vm("16_option_result/010_propagate_err").unwrap(); }
#[test] fn test_16_option_result_011_coalesce_some() { test_vm("16_option_result/011_coalesce_some").unwrap(); }
#[test] fn test_16_option_result_012_coalesce_none() { test_vm("16_option_result/012_coalesce_none").unwrap(); }
#[test] fn test_16_option_result_013_is_some_binding() { test_vm("16_option_result/013_is_some_binding").unwrap(); }
#[test] fn test_16_option_result_014_is_none_match() { test_vm("16_option_result/014_is_none_match").unwrap(); }
#[test] fn test_16_option_result_015_is_ok_binding() { test_vm("16_option_result/015_is_ok_binding").unwrap(); }
#[test] fn test_16_option_result_016_is_err_match() { test_vm("16_option_result/016_is_err_match").unwrap(); }

// === 17_modules ===
#[test] fn test_17_modules_001_use_fn() { test_vm("17_modules/001_use_fn").unwrap(); }

// === 18_ffi ===
#[test] fn test_18_ffi_001_file_exists() { test_vm("18_ffi/001_file_exists").unwrap(); }
#[test] fn test_18_ffi_003_file_is_dir() { test_vm("18_ffi/003_file_is_dir").unwrap(); }
#[test] fn test_18_ffi_006_string_len() { test_vm("18_ffi/006_string_len").unwrap(); }
#[test] fn test_18_ffi_007_string_is_empty() { test_vm("18_ffi/007_string_is_empty").unwrap(); }
#[test] fn test_18_ffi_008_string_contains() { test_vm("18_ffi/008_string_contains").unwrap(); }
#[test] fn test_18_ffi_009_string_starts_with() { test_vm("18_ffi/009_string_starts_with").unwrap(); }
#[test] fn test_18_ffi_010_string_ends_with() { test_vm("18_ffi/010_string_ends_with").unwrap(); }
#[test] fn test_18_ffi_012_char_is_alpha() { test_vm("18_ffi/012_char_is_alpha").unwrap(); }
#[test] fn test_18_ffi_013_char_is_digit() { test_vm("18_ffi/013_char_is_digit").unwrap(); }
#[test] fn test_18_ffi_014_json_is_valid() { test_vm("18_ffi/014_json_is_valid").unwrap(); }
#[test] fn test_18_ffi_015_json_len() { test_vm("18_ffi/015_json_len").unwrap(); }
#[test] fn test_18_ffi_016_json_is_null() { test_vm("18_ffi/016_json_is_null").unwrap(); }
#[test] fn test_18_ffi_017_json_as_bool() { test_vm("18_ffi/017_json_as_bool").unwrap(); }
#[test] fn test_18_ffi_018_json_has_key() { test_vm("18_ffi/018_json_has_key").unwrap(); }

// === 19_rust_std ===
#[test] fn test_19_rust_std_001_time() { test_vm("19_rust_std/001_time").unwrap(); }
#[test] fn test_19_rust_std_002_duration() { test_vm("19_rust_std/002_duration").unwrap(); }
#[test] fn test_19_rust_std_003_pathbuf() { test_vm("19_rust_std/003_pathbuf").unwrap(); }
#[test] fn test_19_rust_std_004_duration_print() { test_vm("19_rust_std/004_duration_print").unwrap(); }
#[test] fn test_19_rust_std_005_instant_duration() { test_vm("19_rust_std/005_instant_duration").unwrap(); }
#[test] fn test_19_rust_std_006_sync() { test_vm("19_rust_std/006_sync").unwrap(); }
#[test] fn test_19_rust_std_007_pathbuf() { test_vm("19_rust_std/007_pathbuf").unwrap(); }
#[test] fn test_19_rust_std_008_box_cell() { test_vm("19_rust_std/008_box_cell").unwrap(); }
#[test] fn test_19_rust_std_009_duration_f64() { test_vm("19_rust_std/009_duration_f64").unwrap(); }

// === 20_permission ===
#[test] fn test_20_permission_001_scalar_mode() { test_vm("20_permission/001_scalar_mode").unwrap(); }
#[test] fn test_20_permission_002_ext_policy() { test_vm("20_permission/002_ext_policy").unwrap(); }

// === 21_conv (Plan 193: type conversion bugs) ===
// BUG: negative integer .to(String) — TYPE_TO_STR treats negative i32 as tagged string pointer
#[test] #[ignore = "BUG: (-1).to(String) outputs '<invalid string index: 0>' — TYPE_TO_STR misidentifies negative integers as string pool indices"]
fn test_21_conv_002_neg_i32_to_str() { test_vm("21_conv/002_neg_i32_to_str").unwrap(); }
