#!/usr/bin/env python3
"""Migrate a2c tests from flat numbered dirs to categorized structure (Plan 171)."""
import os, shutil

BASE = os.path.dirname(os.path.abspath(__file__))

# Mapping: (old_dir, new_category_dir, new_test_dir, new_file_name)
MIGRATIONS = [
    # 01_basics (3)
    ("000_hello", "01_basics", "001_hello", "hello"),
    ("001_sqrt", "01_basics", "002_sqrt", "sqrt"),
    ("003_func", "01_basics", "003_func", "func"),
    # 02_types (7)
    ("006_struct", "02_types", "001_struct", "struct"),
    ("007_enum", "02_types", "002_enum", "enum"),
    ("013_union", "02_types", "003_union", "union"),
    ("005_pointer", "02_types", "004_pointer", "pointer"),
    ("128_inheritance", "02_types", "005_inheritance", "inheritance"),
    ("108_pointer_types", "02_types", "006_pointer_types", "pointer_types"),
    ("135_bool", "02_types", "007_bool", "bool"),
    # 03_control_flow (8)
    ("010_if", "03_control_flow", "001_if_basic", "if_basic"),
    ("011_for", "03_control_flow", "002_for_range", "for_range"),
    ("012_is", "03_control_flow", "003_is_match", "is_match"),
    ("031_for_conditions", "03_control_flow", "004_for_conditions", "for_conditions"),
    ("083_mut_counter", "03_control_flow", "005_mut_counter", "mut_counter"),
    ("083_mut_accumulator", "03_control_flow", "006_mut_accumulator", "mut_accumulator"),
    ("083_mut_array_sum", "03_control_flow", "007_mut_array_sum", "mut_array_sum"),
    ("083_mut_multiple", "03_control_flow", "008_mut_multiple", "mut_multiple"),
    # 04_strings (2)
    ("015_str", "04_strings", "001_str", "str"),
    ("030_str_split", "04_strings", "002_str_split", "str_split"),
    # 05_expressions (4)
    ("028_complex_expr", "05_expressions", "001_complex_expr", "complex_expr"),
    ("054_field_access", "05_expressions", "002_field_access", "field_access"),
    ("131_bang_operator", "05_expressions", "003_bang_operator", "bang_operator"),
    ("038_binary", "05_expressions", "004_binary", "binary"),
    # 06_pattern_matching (6)
    ("014_hetero_enum", "06_pattern_matching", "001_hetero_enum", "hetero_enum"),
    ("060_hetero_enum_verify", "06_pattern_matching", "002_hetero_enum_verify", "hetero_enum_verify"),
    ("032_hetero_enum_types", "06_pattern_matching", "003_hetero_enum_types", "hetero_enum_types"),
    ("046_mode", "06_pattern_matching", "004_enum_smoke_2var", "enum_smoke_2var"),
    ("041_tristate", "06_pattern_matching", "005_enum_smoke_3var", "enum_smoke_3var"),
    ("036_may_patterns", "06_pattern_matching", "006_enum_with_functions", "enum_with_functions"),
    # 07_ownership (4)
    ("023_borrow_view", "07_ownership", "001_borrow_view", "borrow_view"),
    ("024_borrow_mut", "07_ownership", "002_borrow_mut", "borrow_mut"),
    ("025_borrow_move", "07_ownership", "003_borrow_move", "borrow_move"),
    ("026_borrow_conflicts", "07_ownership", "004_borrow_conflicts", "borrow_conflicts"),
    # 08_generics (7)
    ("110_const_generics", "08_generics", "001_const_generics", "const_generics"),
    ("126_generic_field", "08_generics", "002_generic_field", "generic_field"),
    ("127_generic_ptr_field", "08_generics", "003_generic_ptr_field", "generic_ptr_field"),
    ("136_with_constraint", "08_generics", "004_with_constraint", "with_constraint"),
    ("112_generic_specs", "08_generics", "005_generic_specs", "generic_specs"),
    ("113_generic_spec_ext", "08_generics", "006_generic_spec_ext", "generic_spec_ext"),
    ("111_generic_type_alias", "08_generics", "007_generic_type_alias", "generic_type_alias"),
    # 09_option_result (3)
    ("118_null_coalesce", "09_option_result", "001_null_coalesce", "null_coalesce"),
    ("119_error_propagate", "09_option_result", "002_error_propagate", "error_propagate"),
    ("125_closure", "09_option_result", "003_closure", "closure"),
    # 10_collections (13)
    ("002_array", "10_collections", "001_array", "array"),
    ("029_array_return", "10_collections", "002_array_return", "array_return"),
    ("097_array_declaration", "10_collections", "003_array_declaration", "array_declaration"),
    ("098_array_mutation", "10_collections", "004_array_mutation", "array_mutation"),
    ("099_array_index_read", "10_collections", "005_array_index_read", "array_index_read"),
    ("100_array_copy", "10_collections", "006_array_copy", "array_copy"),
    ("101_array_slice", "10_collections", "007_array_slice", "array_slice"),
    ("102_array_nested", "10_collections", "008_array_nested", "array_nested"),
    ("103_array_zero_size", "10_collections", "009_array_zero_size", "array_zero_size"),
    ("104_array_loop", "10_collections", "010_array_loop", "array_loop"),
    ("117_list_storage", "10_collections", "011_list_storage", "list_storage"),
    ("122_list_iter", "10_collections", "012_list_iter", "list_iter"),
    ("055_list_capacity", "10_collections", "013_list_capacity", "list_capacity"),
    # 11_methods (3)
    ("008_method", "11_methods", "001_method", "method"),
    ("064_multi_param", "11_methods", "002_multi_param", "multi_param"),
    ("066_generic_list", "11_methods", "003_generic_list", "generic_list"),
    # 12_specs (2)
    ("016_basic_spec", "12_specs", "001_basic_spec", "basic_spec"),
    ("017_spec", "12_specs", "002_spec", "spec"),
    # 13_delegation (3)
    ("018_delegation", "13_delegation", "001_single", "single"),
    ("019_multi_delegation", "13_delegation", "002_multi_delegation", "multi_delegation"),
    ("020_delegation_params", "13_delegation", "003_delegation_params", "delegation_params"),
    # 18_c_interop (3)
    ("004_cstr", "18_c_interop", "001_cstr", "cstr"),
    ("009_alias", "18_c_interop", "002_alias", "alias"),
    ("027_unified_section", "18_c_interop", "003_unified_section", "unified_section"),
    # 19_option_type (18)
    ("076_question_syntax", "19_option_type", "001_question_syntax", "question_syntax"),
    ("077_question_uint", "19_option_type", "002_question_uint", "question_uint"),
    ("078_question_float", "19_option_type", "003_question_float", "question_float"),
    ("079_question_double", "19_option_type", "004_question_double", "question_double"),
    ("080_question_char", "19_option_type", "005_question_char", "question_char"),
    ("081_question_void", "19_option_type", "006_question_void", "question_void"),
    ("082_question_return_int", "19_option_type", "007_question_return_int", "question_return_int"),
    ("083_question_return_str", "19_option_type", "008_question_return_str", "question_return_str"),
    ("084_question_return_bool", "19_option_type", "009_question_return_bool", "question_return_bool"),
    ("085_question_return_uint", "19_option_type", "010_question_return_uint", "question_return_uint"),
    ("086_question_return_float", "19_option_type", "011_question_return_float", "question_return_float"),
    ("087_question_return_double", "19_option_type", "012_question_return_double", "question_return_double"),
    ("088_question_return_char", "19_option_type", "013_question_return_char", "question_return_char"),
    ("089_question_nested_call", "19_option_type", "014_question_nested_call", "question_nested_call"),
    ("090_question_arithmetic", "19_option_type", "015_question_arithmetic", "question_arithmetic"),
    ("091_question_comparison", "19_option_type", "016_question_comparison", "question_comparison"),
    ("092_question_logical", "19_option_type", "017_question_logical", "question_logical"),
    ("093_question_negation", "19_option_type", "018_question_negation", "question_negation"),
    # 21_storage (3)
    ("114_storage_module", "21_storage", "001_storage_module", "storage_module"),
    ("115_storage_usage", "21_storage", "002_storage_usage", "storage_usage"),
    ("116_plan055_auto_storage", "21_storage", "003_plan055_auto_storage", "plan055_auto_storage"),
    # 22_iterators (7)
    ("120_iter_specs", "22_iterators", "001_iter_specs", "iter_specs"),
    ("121_map_adapter", "22_iterators", "002_map_adapter", "map_adapter"),
    ("129_terminal_operators", "22_iterators", "003_terminal_operators", "terminal_operators"),
    ("130_terminal_operators", "22_iterators", "004_terminal_operators_2", "terminal_operators_2"),
    ("132_extended_adapters", "22_iterators", "005_extended_adapters", "extended_adapters"),
    ("133_predicates", "22_iterators", "006_predicates", "predicates"),
    ("134_collect", "22_iterators", "007_collect", "collect"),
    # 23_stdlib (17)
    ("137_std_hello", "23_stdlib", "001_std_hello", "std_hello"),
    ("138_std_getpid", "23_stdlib", "002_std_getpid", "std_getpid"),
    ("139_std_readline", "23_stdlib", "003_std_readline", "std_readline"),
    ("140_std_file", "23_stdlib", "004_std_file", "std_file"),
    ("141_std_repl", "23_stdlib", "005_std_repl", "std_repl"),
    ("142_std_str", "23_stdlib", "006_std_str", "std_str"),
    ("143_file_operations", "23_stdlib", "007_file_operations", "file_operations"),
    ("144_char_io", "23_stdlib", "008_char_io", "char_io"),
    ("145_advanced_io", "23_stdlib", "009_advanced_io", "advanced_io"),
    ("146_io_specs", "23_stdlib", "010_io_specs", "io_specs"),
    ("147_std_test", "23_stdlib", "011_std_test", "std_test"),
    ("148_std_readline", "23_stdlib", "012_std_readline_2", "std_readline_2"),
    ("150_std_file_flush", "23_stdlib", "013_std_file_flush", "std_file_flush"),
    ("151_std_file_read", "23_stdlib", "014_std_file_read", "std_file_read"),
    ("123_hashmap", "23_stdlib", "015_hashmap", "hashmap"),
    ("124_hashset", "23_stdlib", "016_hashset", "hashset"),
    ("149_std_file_write", "23_stdlib", "017_std_file_write", "std_file_write"),
    # 24_runtime_size (2)
    ("106_runtime_size_var", "24_runtime_size", "001_runtime_size_var", "runtime_size_var"),
    ("107_runtime_size_expr", "24_runtime_size", "002_runtime_size_expr", "runtime_size_expr"),
    # 25_type_checking (1)
    ("021_type_error", "25_type_checking", "001_type_error", "type_error"),
]

# Directories to explicitly delete (may tests, redundant enum smoke, duplicates)
DELETE_DIRS = [
    # May tests (replaced by Option/Result)
    "033_may_basic", "034_may_string", "035_may_bool", "037_may_nested", "052_may_storage",
    # Redundant enum smoke tests (keep 046_mode, 041_tristate, 036_may_patterns)
    "042_direction", "045_status", "048_result", "049_phase", "050_level", "051_state",
    "053_type", "056_side", "057_flow", "058_gate", "059_path",
    "065_size", "067_speed", "068_power", "069_signal", "070_zone", "071_mode2",
    "072_link", "073_source", "074_target", "075_format",
    # Duplicate hetero enum tests
    "022_typed_node_checking", "063_simple_hetero_enum",
    # Duplicate question tests (identical to 094)
    "094_question_literal", "095_question_zero", "096_question_negative",
    # Runtime arrays backup (superseded by 106+107)
    "105_runtime_arrays_backup",
    # Generic hetero enum (ignored, transpilation not implemented)
    "109_generic_hetero_enum",
]


def get_old_file_name(old_dir):
    """Extract file name from old dir like '032_hetero_enum_types' -> 'hetero_enum_types'."""
    parts = old_dir.split("_", 1)
    return parts[1] if len(parts) > 1 else old_dir


def is_category_dir(name):
    """Check if a directory name is a category dir (NN_name format)."""
    if "_" not in name:
        return False
    prefix = name.split("_")[0]
    return prefix.isdigit() and len(prefix) == 2


def migrate():
    moved = 0
    skipped = 0

    for old_dir, cat_dir, new_test_dir, new_file_name in MIGRATIONS:
        old_path = os.path.join(BASE, old_dir)
        if not os.path.isdir(old_path):
            print(f"SKIP (not found): {old_dir}")
            skipped += 1
            continue

        old_file_name = get_old_file_name(old_dir)
        cat_path = os.path.join(BASE, cat_dir)
        new_path = os.path.join(cat_path, new_test_dir)
        os.makedirs(new_path, exist_ok=True)

        # Copy .at file
        old_at = os.path.join(old_path, f"{old_file_name}.at")
        new_at = os.path.join(new_path, f"{new_file_name}.at")
        if os.path.exists(old_at):
            shutil.copy2(old_at, new_at)

        # Copy .expected.c file
        old_exp_c = os.path.join(old_path, f"{old_file_name}.expected.c")
        new_exp_c = os.path.join(new_path, f"{new_file_name}.expected.c")
        if os.path.exists(old_exp_c):
            shutil.copy2(old_exp_c, new_exp_c)

        # Copy .expected.h file
        old_exp_h = os.path.join(old_path, f"{old_file_name}.expected.h")
        new_exp_h = os.path.join(new_path, f"{new_file_name}.expected.h")
        if os.path.exists(old_exp_h):
            shutil.copy2(old_exp_h, new_exp_h)

        # Copy .expected.error.log file (for error tests like 021_type_error)
        old_err = os.path.join(old_path, f"{old_file_name}.expected.error.log")
        new_err = os.path.join(new_path, f"{new_file_name}.expected.error.log")
        if os.path.exists(old_err):
            shutil.copy2(old_err, new_err)

        # Skip .wrong files (stale output)
        files_copied = []
        for ext in [".at", ".expected.c", ".expected.h", ".expected.error.log"]:
            if os.path.exists(os.path.join(new_path, f"{new_file_name}{ext}")):
                files_copied.append(ext)

        moved += 1
        print(f"  {old_dir}/{old_file_name} -> {cat_dir}/{new_test_dir}/{new_file_name}  {files_copied}")

    print(f"\nMigrated {moved} test cases (skipped {skipped}).")

    # Delete specified dirs
    deleted = 0
    for d in DELETE_DIRS:
        dp = os.path.join(BASE, d)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  DELETED: {d}")
        else:
            print(f"  SKIP delete (not found): {d}")

    # Delete migrated old dirs
    for old_dir, _, _, _ in MIGRATIONS:
        dp = os.path.join(BASE, old_dir)
        if os.path.isdir(dp):
            shutil.rmtree(dp)
            deleted += 1
            print(f"  REMOVED old: {old_dir}")

    # Delete orphaned dirs (no .at file, not a category dir)
    orphaned = 0
    for entry in sorted(os.listdir(BASE)):
        entry_path = os.path.join(BASE, entry)
        if not os.path.isdir(entry_path):
            continue
        if entry in ("__pycache__",) or entry.startswith("."):
            continue
        if is_category_dir(entry):
            continue
        # Check if it has any .at file
        has_at = any(f.endswith(".at") for f in os.listdir(entry_path))
        if not has_at:
            shutil.rmtree(entry_path)
            orphaned += 1
            deleted += 1
            print(f"  ORPHAN deleted: {entry}")

    print(f"\nDeleted {deleted} directories ({orphaned} orphaned).")


if __name__ == "__main__":
    migrate()
