mod a2c_tests;
mod a2r_tests;
mod a2ts_tests;
mod atom_tests;
// Plan 075: Unified API tests
mod unified_api_tests;
// Plan 073 Phase 9.1: Performance benchmarking
mod perf_benchmark_tests;
// config_tests removed - Plan 091 (deprecated Interpreter dependency)
mod const_generic_integration_tests; // Plan 052: Const generic integration tests
mod const_generic_tests; // Plan 052: Const generic parameter tests
mod default_storage_tests; // Plan 052: DefaultStorage type alias tests
mod dstr_tests;
mod error_tests;
// Plan 094: Hybrid FFI Bridge tests
mod ffi_tests;
mod field_access_tests; // Plan 056: Field access tests
mod generic_spec_tests; // Plan 057: Generic spec tests
mod list_growth_tests;
mod list_tests; // Comprehensive List operation tests (Plan 051)
mod may_tests;
mod mem_tests;
mod memory_quick_test;
mod memory_tests;
mod ownership_tests;
mod phase3_tests; // Plan 125: Phase 3 polymorphic routing tests
mod pointer_tests; // Plan 052: Pointer type tests
mod stdlib_tests;
mod storage_integration_tests;
mod storage_tests;
mod string_tests;
// template_tests removed - Plan 091 (deprecated Interpreter dependency)
mod test_generic_full;
mod test_generic_parse;
mod test_generic_simple;
mod test_let_generic;
mod vm_functions_tests;
// vm_tests and autovm_tests merged - Plan 118
// autovm_tests removed - tests consolidated into vm_tests
mod vm_tests;
mod infer_tests;
mod autodown_tests;
mod book_listing_tests;
mod vm_file_tests; // Plan 177: VM file-based test framework