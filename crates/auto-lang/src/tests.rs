// Plan 289: Transpiler tests (a2c/a2r/a2ts) gated behind test-trans feature
#[cfg(feature = "test-trans")]
mod a2c_tests;
#[cfg(feature = "test-trans")]
mod a2r_tests;
#[cfg(feature = "test-trans")]
mod a2ts_tests;
mod atom_tests;
// Plan 075: Unified API tests
mod unified_api_tests;
// Plan 073 Phase 9.1: Performance benchmarking
mod perf_benchmark_tests;
#[path = "tests/plan377_bench.rs"]
mod plan377_bench; // Plan 377 §4.3: 单槽化性能验收
// config_tests removed - Plan 091 (deprecated Interpreter dependency)
mod const_generic_integration_tests; // Plan 052: Const generic integration tests
mod const_generic_tests; // Plan 052: Const generic parameter tests
mod default_storage_tests; // Plan 052: DefaultStorage type alias tests
mod dstr_tests;
mod error_tests;
// Plan 094: Hybrid FFI Bridge tests
mod ffi_tests;
mod field_access_tests; // Plan 056: Field access tests
mod ffi_dual_tests; // Plan 212 Phase 3D.1: FFI dual-test infrastructure
mod generic_spec_tests; // Plan 057: Generic spec tests
mod trait_vm_tests; // Plan 417-E4: spec default-method inheritance
mod list_growth_tests;
mod list_tests; // Comprehensive List operation tests (Plan 051)
mod may_tests;
mod mem_tests;
// Plan 565 P0: mem-profile 归因报告（#[ignore] 诊断，仅 mem-profile feature）
#[cfg(feature = "mem-profile")]
mod mem_profile_report_tests;
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
mod vm_json_float_read_tests; // Plan 474: __json_object 浮点字段 Dot 读回归（plan011④）
// vm_tests and autovm_tests merged - Plan 118
// autovm_tests removed - tests consolidated into vm_tests
mod vm_tests;
mod infer_tests;
mod autodown_tests;
mod generator_tests; // Plan 326: generator for-loop regression tests
mod actor_tests; // Plan 327 Phase 1: Task/Msg actor handler execution
mod actor_state_tests; // Plan 327: actor state field persistence
// `mod async_probe_tests;` was removed: it referenced a non-existent file (a
// research placeholder from the "VM 真异步调度统一调研报告" commit, self-marked
// "调研后删除") and broke `cargo test --lib` on master. The research report
// itself lives in docs/plans/.
// Plan 289: Book listing tests gated behind test-book feature (slow: ~5-7s each)
#[cfg(feature = "test-book")]
mod book_listing_tests;
// Plan 289: VM file tests gated behind test-vm-files feature
#[cfg(feature = "test-vm-files")]
mod vm_file_tests; // Plan 177: VM file-based test framework(Plan 568: 纯 .at 语料 golden 档,aavm 腿已迁出)
#[cfg(feature = "test-aavm")] // Plan 568: vm_file_tests 内嵌 aavm 腿整体迁入
mod aavm_runner_tests;
#[cfg(feature = "test-aavm")] // Plan 564: 重内存测试守门(NEXTEST/AUTO_LANG_HEAVY_MEM 双通道,防裸 cargo test 全并发;aavm 系迁入 test-aavm 后随系门控)
mod heavy_gate;
#[cfg(feature = "test-vm-files")]
mod cookbook_vm_tests; // Plan 240: Cookbook VM output comparison tests
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_a2r; // Plan 447 部分② Phase 7: AA2R is 发射对齐主 a2r 闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm_at_mode_tests; // Plan 531: aavm.at a2r 模式入口(位置参数)验收锚
#[cfg(feature = "test-aavm")] // Plan 568: 原无门(日常档在跑),随 aavm 系入独立档
mod aavm2_m1; // Plan 432 S1: M1 lexer token 流一致性闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_m2; // Plan 432 S2: M2 parser AST dump 一致性闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_m3; // Plan 432 S3: M3 typeinfo .type 一致性闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_m4; // Plan 432 S4: M4 codegen 字节码结构一致性闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_m5; // Plan 432 S5: M3 主里程碑 —— 全管线行为一致性闸门
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_t3; // Plan 532: t3 里程碑档——嵌套塔解释栈零漂移最终验收(大版本升级专用)
#[cfg(feature = "test-aavm")] // Plan 568: AAVM/AA2R 独立档(原 test-vm-files)
mod aavm2_repro_242; // Plan 432 D26: VM 字符串池 RC 回归复现(ignore,修复后转绿)
#[cfg(feature = "test-vm-files")]
mod conformance_tests; // AutoVM output regression tests (golden-file); VM↔a2r parity is in parity/
mod plan569_py_dispatch_tests; // Plan 569 D2: py 返回值方法分派动态化——codegen 决策核单测（无 pyo3 依赖）