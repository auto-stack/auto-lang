// Plan 289: Transpiler tests (a2c/a2r/a2ts) gated behind test-trans feature
mod a2c_tests;
mod env_restore;
// Plan 768: TestCase protocol adapter + runner integration (real VM/a2r)
mod test_cases_tests;
// PLAN-790: clipboard media 2934..2937 host provider——core 剪断/降级契约/
// 装配优先/ownership 回归锚
mod plan790_clipboard_media_provider;
// PLAN-794: scroll controller/子锚点 9900..9905+9922/9923 host provider——
// core 剪断/降级契约/装配身份/真实 UI adapter 接线回归锚
// PLAN-804 V-04/V-05: 真 OS 实机证据档——env 门控（AUTO_LANG_DIALOG_E2E/
// AUTO_LANG_DND_E2E=1 才运行，日常档零 OS 面零交互；详见文件头）。
// KNOWN-DEBT 396 rider (Plan 415-B1): stdlib .rs.at ↔ a2r-std signature parity
mod a2r_c_abi_gates;
mod a2r_std_signature_parity;
mod a2r_tests;
mod a2ts_tests;
// Plan 577: a2ts directed compile-level probes (.at → tsc --noEmit)
mod a2ts_directed_probes;
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
mod ffi_dep_parity_tests; // PLAN-592 T6: dep 三轨行为对拍(VM/oracle/a2r)
mod ffi_dual_tests; // Plan 212 Phase 3D.1: FFI dual-test infrastructure
mod ffi_tests;
mod field_access_tests; // Plan 056: Field access tests
mod generic_spec_tests; // Plan 057: Generic spec tests
mod list_growth_tests;
mod list_tests; // Comprehensive List operation tests (Plan 051)
mod may_tests;
mod mem_tests;
mod trait_vm_tests; // Plan 417-E4: spec default-method inheritance
                    // Plan 565 P0: mem-profile 归因报告（#[ignore] 诊断，仅 mem-profile feature）
mod fs_tree_parity; // PLAN-681: fs.tree VM/a2r-std 双轨逐字节对拍 (P670-D1 缺口④/AC-05)
#[cfg(feature = "mem-profile")]
mod mem_profile_report_tests;
mod memory_quick_test;
mod memory_tests;
mod ownership_tests;
mod phase3_tests; // Plan 125: Phase 3 polymorphic routing tests
mod pointer_tests; // Plan 052: Pointer type tests
mod read_text_range_parity; // Plan 673 T-03: 分块读 VM/a2r-std 双轨逐字节对拍 (P670-D1)
mod stdlib_tests;
mod storage_integration_tests;
mod storage_tests;
mod string_tests;
// template_tests removed - Plan 091 (deprecated Interpreter dependency)
mod test_generic_full;
mod test_generic_parse;
mod test_generic_simple;
mod test_let_generic;
mod use_semantics_tests;
mod vm_functions_tests;
                              // vm_tests and autovm_tests merged - Plan 118
                              // autovm_tests removed - tests consolidated into vm_tests
mod actor_state_tests;
mod actor_tests; // Plan 327 Phase 1: Task/Msg actor handler execution
mod autodown_tests;
mod generator_tests; // Plan 326: generator for-loop regression tests
mod infer_tests;
mod vm_tests; // Plan 327: actor state field persistence
              // `mod async_probe_tests;` was removed: it referenced a non-existent file (a
              // research placeholder from the "VM 真异步调度统一调研报告" commit, self-marked
              // "调研后删除") and broke `cargo test --lib` on master. The research report
              // itself lives in docs/plans/.
              // Plan 289: Book listing tests gated behind test-book feature (slow: ~5-7s each)
mod book_listing_tests;
// Plan 177: VM file-based test framework(Plan 568: 纯 .at 语料 golden 档,aavm 腿已迁出;
// PLAN-700: 原 Plan 289 语料档门拆除,语料三族常驻编译——tv 降为纯 nextest filter 档)
// // Plan 568: vm_file_tests 内嵌 aavm 腿整体迁入
mod aavm_runner_tests;
mod vm_file_tests;
// Plan 564: 重内存测试守门(NEXTEST/AUTO_LANG_HEAVY_MEM 双通道,防裸 cargo test
// 全并发)。PLAN-726 T-02 拆除 test-aavm feature 门并升 pub(crate):machine_gate
// 机器级单实例闸门为跨档跨树基建(画廊围栏在 lib.rs 顶层、churn-1m 在 crate::vm、
// aavm 族在 crate::tests,三树共用)。
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_a2r; // Plan 447 部分② Phase 7: AA2R is 发射对齐主 a2r 闸门
// Plan 565 L1: 语料闸门 once-compiled runner(编译一次+File.read_text 注入)
mod aavm2_corpus_runner;
// // Plan 568: 原无门(日常档在跑),随 aavm 系入独立档
mod aavm2_m1; // Plan 432 S1: M1 lexer token 流一致性闸门
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_m2; // Plan 432 S2: M2 parser AST dump 一致性闸门
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_m3; // Plan 432 S3: M3 typeinfo .type 一致性闸门
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_m4; // Plan 432 S4: M4 codegen 字节码结构一致性闸门
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_m5; // Plan 432 S5: M3 主里程碑 —— 全管线行为一致性闸门
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_repro_242; // Plan 432 D26: VM 字符串池 RC 回归复现(ignore,修复后转绿)
// // Plan 568: AAVM/AA2R 独立档
mod aavm2_t3; // Plan 532: t3 里程碑档——嵌套塔解释栈零漂移最终验收(大版本升级专用)
// // Plan 568: AAVM/AA2R 独立档
mod aavm_at_mode_tests; // Plan 531: aavm.at a2r 模式入口(位置参数)验收锚
mod conformance_tests; // AutoVM output regression tests (golden-file); VM↔a2r parity is in test/parity/
mod cookbook_vm_tests; // Plan 240: Cookbook VM output comparison tests
pub use auto_lang_core::heavy_gate;
mod plan569_py_dispatch_tests;
                                     // PLAN-691 T-04: 顶层散测试文件归位（src/*_tests.rs → src/tests/，2026-09-22）。
                                     // 以下 mod 声明自 lib.rs 迁入，cfg 门与溯源注释原样保留；
                                     // 文件本体对应 git mv 至 src/tests/<name>.rs（rename 纯 move）。
                                     // test_runner 例外留在 lib.rs——VM native auto.test.* 生产依赖（Plan 263）。
                                     // ============================================================

#[cfg(test)]
mod test_util; // Plan 266 Phase 4: Differential testing utilities
               // Plan 088: Parameter passing mode tests
#[cfg(test)]
mod plan_088_parser_tests;
#[cfg(test)]
mod plan_088_tests;
#[cfg(test)]
mod test_double_lexer;
// PLAN-789: 词法抽取类型身份/错误投影经兼容出口的 V03 树内侧。
#[cfg(test)]
mod plan789_lex_extraction;
#[cfg(test)]
mod test_float_full;
#[cfg(test)]
mod test_parser_arrow;
#[cfg(test)]
mod vm_types_tests;
// Plan 076 Phase 1: Generic type support tests
#[cfg(test)]
mod generic_tests;
// Plan 076 Phase 2: Monomorphization tests
#[cfg(test)]
mod monomorphize_tests;
// Plan 076 Phase 4: Storage strategy tests
#[cfg(test)]
mod storage_strategy_tests;
// Plan 076 Phase 5: Integration tests
#[cfg(test)]
mod bigvm_generic_integration_tests;
// Plan 077 Phase 2: Generic ListData<T> tests
#[cfg(test)]
mod generic_list_data_tests;
// Plan 077 Phase 3: HeapObject implementation tests
#[cfg(test)]
mod listdata_heap_object_tests;
// Plan 077 Phase 4: Unified object registry tests
#[cfg(test)]
mod unified_registry_tests;
#[cfg(test)]
mod plan320_tests;
#[cfg(test)]
mod plan341_tests;
#[cfg(test)]
mod plan349_tests;
// Plan 446 批二: http natives 可用性回归（E1 res.status 哨兵 / E2 builder 链后续调用）。
#[cfg(test)]
mod plan446_batch2_tests;
// Plan 446 批二 B1: store 列表循环字段访问事件实参回归（corpus 驱动）。
// Plan 446 批三: 动态值管线语义统一回归（D1/D2/D3/D6 探针）。
// Plan 446 批四: 打磨项回归（F3/D4/D5/E3 探针）。
// Plan 446 C1: popover 形态渲染回归锁（复审转常驻）。
// Plan 446 批五: 渲染层（§P/U1-U7）回归。
// Plan 442 A2: legacy `use store: X` facade regression corpus.
// PLAN-622: named-store facade consumption-gap red corpus (a-e).
// PLAN-667: 内存安全底线探针矩阵（捕获/RC/a2r，VM 生产路径）。
#[cfg(test)]
mod plan667_memory_safety_tests;
// Plan 442 A3: `use.web` ext link regression corpus.
// PLAN-732: autodown 编辑壳 wikilink 激活供给六环贯通 corpus。
// PLAN-739: 编辑回声→换页视面跟随回归（F-037-R1——input_values 键入条目
// 永生 × patch_input_values ADE 覆写钉死视面）。
// Plan 051 C7: `timer { ... }` 声明块（widget/store 周期计时器 DSL）。
// Plan 051 Phase 2: 会话壳视觉五缺陷（子模块 use.web 注册表/容器 min-h/i18n 参数）。
// Plan 442 A5: one-shot scheduler primitives regression corpus.
// Plan 442 B-support: web-platform global bridges regression corpus.
// Plan 442 C2: parser regressions for the Bug A family (method chains on
// use.rs-imported fn calls in argument position).
#[cfg(test)]
mod plan442_parser_regress_tests;
// Plan 442 Phase B gate: headless link probe against the real auto-musk
// corpus (sibling checkout). #[ignore] — manual-only, never in CI.
// Plan 442 Phase C1 probe: VM-direct run of the auto-musk backend corpus
// (a2r sources). #[ignore] — manual-only, never in CI.
#[cfg(test)]
mod plan442_musk_backend_probe_tests;
// PLAN-059 T9 探针（musk DeleteConfirmDialog 端口链实机复现）：零参父
// handler 的子→父声明式路由帧对齐契约。
// PLAN-536 探针套件（timer 失效/Init 挂载/absolute 悬浮/modal 绑定）。
// PLAN-536 T10 复挂载：声明原随 T1 落 lib.rs（6189f679f），被 b4d1ced7b
// （539 折叠前同步合并）冲突解决时静默丢弃——全套 17+ 探针自此在 master
// 为死代码未编译，复审遗漏扫描立案；本行恢复挂载。
// PLAN-536 T12 探针（musk 发送链最小同形）：一参回调链帧对齐复验（KD-493①
// 在 aa92a821e 后的组件层面）+ PollStream 兜底链活性（streaming 置位先于
// Sse.open 抛点）。
// PLAN-083 T-01：异步 HTTP 消息桥（Http.get_msg）——派生线程完成项入队、
// 载荷协议 {"ok","status","body"}、␟s␟ 载荷编码经 decode_payload 回填
// handler 的字符串实参往返（decode_payload 在 ui::dynamic，随 ui-iced 门）。
mod auto_down_vm_server_probe_tests;
// Plan 049 / PLAN-593 / Plan 606 / PLAN-615 / PLAN-733 / PLAN-749 /
// PLAN-748 的 UI 门测试随 PLAN-900 迁 auto-ui-runtime ui_tests（门与
// 文件同迁）；本 tests.rs 不再有 ui feature cfg。
#[cfg(test)]
mod plan046_obj_natives_tests;
// Plan 409 §6: link 子组件 VM 渲染缺口 regression tests (needs the gallery
// sources + the ui-iced interpreter path).
// Plan 412: Layout Gallery + VM 布局引擎对齐 integration tests(12 页构建 +
// grid 重派生 + col-span 元数据 + 路由登记)。
// Plan 437 Phase 2: VM 轨子组件 Init 生命周期钉子(渲染期 props→Init→build)。
#[cfg(test)]
mod plan352_tests;
#[cfg(test)]
mod plan353_tests;
// Plan 359 Phase 5 (G1/G2/G3): Concurrency bug fixes (spawn, generic types, channels)
#[cfg(test)]
mod plan348_concurrency_tests;
// Plan 498: chart 交互状态机——M0 mouse-area on_click 引擎臂冒烟 +
// M1-M4 悬停态/legend 点击显隐断言(484 冒烟扩展)。
// Plan 492: 引擎正确性专项——f-string 括号插值/primary-shorthand `[` 后缀/
// vue 文本内容表达式臂/包组件单 VM 编译链三族修复。
// PLAN-657: L1 组装样板（047-bp-admin）——四包五变体直连全量 VM/vue 双轨
// 回归锚 + 参数化语料面（palette 零漂移 + dep 探测）。
// PLAN-082: 脚本与 `>` 命令的结构化互操作——shell_query/run native 数据面
// （T-01 spike 三前提 + T-02 mock host 契约）。
#[cfg(test)]
mod plan082_shell_query_tests;
// PLAN-702: engine 段执行驱动单测（UI handler park/resume 语义 +
// 非 yield handler 同步对拍 + resume 段 .at try/catch 捕获）。
#[cfg(test)]
mod plan702_segment_tests;
mod plan781_budget_tests;
// PLAN-712 T-17（候选②）：Http.*_json 失败语义分层（传输失败 = 可捕获
// 异常 / 非 2xx 保持值面）——018 详情页吞错链的语义层回归锁。
#[cfg(test)]
mod plan712_http_error_semantics_tests;
mod plan724_http_client_tests; // PLAN-724: a2r HTTP 客户端收敛（golden + 探针 + http_e2e_plan724 编译运行腿）
                               // PLAN-727: HTTP 文件传输（31_plan727 golden + legacy 发射探针 + http_e2e_plan727 三方编译运行腿）
mod plan727_http_transfer_tests;
// PLAN-738 T-01：装配原型探针（真实入口层选择/死缓存/persistent 可见性/
// a2r 名称表漂移/六核心 parser 分母；T-02 起由正式 stdlib_assembly 族收编）
mod plan738_assembly_probe_tests;
// PLAN-738 T-02：stdlib_assembly 正式族（全库 inventory/provider 目录机器对照）
mod plan738_stdlib_assembly_tests;
// PLAN-776 T-01：HTTP 返回资源身份表示探针（TAG_OBJECT 凭据传播/数字不可
// 伪造/查表制对照；T-02 起扩展为返回资源全矩阵）
// PLAN-755 T-01/T-03：HTTP 公共面契约覆盖走查 + CFFI merge 存活守卫
mod plan755_http_contract_tests;
// PLAN-872 T-01：六族 builtin 绑定面实测探针（证据生成器，--nocapture 转录进绑定表）。
mod plan872_builtin_binding_probes;
// PLAN-872 T-04：六族 DISTINGUISH 区分性测试（生效语义钉死 + 绑定槽位漂移反控）。
mod plan872_builtin_semantics_tests;
// PLAN-712 T-17 第二层：router.param 的 VM 取值链（push → sync → 页 Init）。
#[cfg(test)]
// PLAN-705 T-01 spike：notify 驱动 owner loop + 取消/迟到完成单次终结
// 协议种子（T-02 统一登记表与 T-04 owner loop 的前提探针）。
// PLAN-707 T-01 spike：外部流/取消三前提红测（705 managed 取消缺口 +
// 流建立阻塞 owner + SSE 跨 chunk UTF-8 玷污；修复落地后逐个转绿）。
#[cfg(test)]
mod plan707_spike_tests;
// PLAN-707 T-02：可取消执行器（managed queued/active/retry 实际取消 +
// 许可回基线 + detached 消息桥反例；D-1 冻结协议验收）。
#[cfg(test)]
mod plan707_cancel_tests;
// PLAN-707 T-04：流客户端 shim 与 for-in 装配（手动/内联/变量三形态 +
// SSE poll/close/error 新面 + 队满终结性 + 许可独立）。
#[cfg(test)]
mod plan707_client_tests;
// PLAN-906: plan707_wait_tests 随 Web SSE 测试助手迁 lang-web-e2e 探针工程。
// PLAN-707 T-07：流资源生命周期收口探针（背压钉界/取消风暴/线程稳定/
// 预算终结/abort 表收口；真 TCP relay 在 plan707_stream_e2e_tests）。
#[cfg(test)]
mod plan707_stream_tests;
// scratch diagnostics (T-01, remove before landing)
// PLAN-746 (PG-MEM-1)：协作式执行截止——死循环确定超时返回、充裕截止
// 正常行为不变、执行线程 panic 干净 Err（SD-04）。
#[cfg(test)]
mod plan746_pg_mem_tests;

// PLAN-814 T-05：V-02 旧公开 API 编译 pin（Parser/桥/VM/NativeInterface/
// 根再导出/类型身份同一）。
