// Plan 289: Transpiler tests (a2c/a2r/a2ts) gated behind test-trans feature
#[cfg(feature = "test-trans")]
mod a2c_tests;
// KNOWN-DEBT 396 rider (Plan 415-B1): stdlib .rs.at ↔ a2r-std signature parity
mod a2r_std_signature_parity;
#[cfg(feature = "test-trans")]
mod a2r_tests;
#[cfg(feature = "test-trans")]
mod a2r_c_abi_gates;
#[cfg(feature = "test-trans")]
mod a2ts_tests;
// Plan 577: a2ts directed compile-level probes (.at → tsc --noEmit)
#[cfg(feature = "test-trans")]
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
mod ffi_tests;
mod field_access_tests; // Plan 056: Field access tests
mod ffi_dual_tests; // Plan 212 Phase 3D.1: FFI dual-test infrastructure
mod ffi_dep_parity_tests; // PLAN-592 T6: dep 三轨行为对拍(VM/oracle/a2r)
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
mod fs_tree_parity; // PLAN-681: fs.tree VM/a2r-std 双轨逐字节对拍 (P670-D1 缺口④/AC-05)
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
mod vm_functions_tests;
mod vm_json_float_read_tests; // Plan 474: __json_object 浮点字段 Dot 读回归（plan011④）
mod use_semantics_tests; // Plan 545: use 命名空间语义（bare=命名空间 / : * = 显式全量）
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
#[cfg(feature = "test-aavm")] // Plan 565 L1: 语料闸门 once-compiled runner(编译一次+File.read_text 注入)
mod aavm2_corpus_runner;
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
mod plan569_py_dispatch_tests; // Plan 569 D2: py 返回值方法分派动态化——codegen 决策核单测（无 pyo3 依赖）// ============================================================
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
mod test_parser_arrow;
#[cfg(test)]
mod test_float_full;
#[cfg(test)]
mod test_double_lexer;
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
// Plan 077 Phase 8: Comprehensive integration tests (TODO: Fix compilation errors)
// #[cfg(test)]
// mod plan077_integration_tests;
#[cfg(test)]
mod error_spans_tests;
#[cfg(test)]
mod plan320_tests;
#[cfg(test)]
mod plan339_tests;
#[cfg(test)]
mod plan340_tests;
#[cfg(test)]
mod plan341_tests;
#[cfg(test)]
mod plan349_tests;
// Plan 446 批二: http natives 可用性回归（E1 res.status 哨兵 / E2 builder 链后续调用）。
#[cfg(test)]
mod plan446_batch2_tests;
// Plan 446 批二 B1: store 列表循环字段访问事件实参回归（corpus 驱动）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_b1_tests;
// Plan 446 批三: 动态值管线语义统一回归（D1/D2/D3/D6 探针）。
#[cfg(test)]
mod plan446_batch3_tests;
// Plan 446 批四: 打磨项回归（F3/D4/D5/E3 探针）。
#[cfg(test)]
mod plan446_batch4_tests;
// Plan 446 C1: popover 形态渲染回归锁（复审转常驻）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_c1_popover_tests;
// Plan 446 批五: 渲染层（§P/U1-U7）回归。
#[cfg(all(test, feature = "ui-iced"))]
mod plan446_batch5_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan370_store_vm_tests;
// Plan 442 A2: legacy `use store: X` facade regression corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_store_facade_tests;
// PLAN-622: named-store facade consumption-gap red corpus (a-e).
#[cfg(all(test, feature = "ui-iced"))]
mod plan622_store_facade_gap_tests;
// PLAN-667: 内存安全底线探针矩阵（捕获/RC/a2r，VM 生产路径）。
#[cfg(test)]
mod plan667_memory_safety_tests;
// PLAN-624: merged single-state cross-state resolution red corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan624_cross_state_tests;
// PLAN-066: 原生组件外部注册 SPI 语料（autodown_editor 迁移等价 + Element
// 通道 View::Custom + 未注册名保形）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan066_native_widget_tests;
// PLAN-066 T-06（auto-musk）: 055-4⑥ 过滤投影（computed 内 VmRef 域读取）
// + P536-D2 跨模块调用帧 SET_FIELD 达根态 red corpus。
#[cfg(all(test, feature = "ui-iced"))]
mod plan066_filter_projection_tests;
// PLAN-627: qualified api-module-call corpus（模块形态 use back.api +
// 限定名 api.X()：抽取/vue 发射/rust 发射三面）。
#[cfg(test)]
mod plan627_qualified_api_tests;
// PLAN-634 T-03: a2r 语句位置块尾分号语料（auto-term DEBTS #18 根修）。
#[cfg(test)]
mod plan634_block_tail_semi_tests;
// PLAN-024: named view (`view mini { ... }`) parsing corpus — desktop
// dashboard language extension.
#[cfg(test)]
mod plan024_named_view_tests;
// Plan 442 A3: `use.web` ext link regression corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_ext_link_tests;
// PLAN-019 T-06: own-module bare-call binding (vm merged link) corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan019_vm_own_module_link_tests;
mod plan051_ext_widget_tests;
// Plan 051 C7: `timer { ... }` 声明块（widget/store 周期计时器 DSL）。
mod plan051_timer_tests;
// Plan 051 Phase 2: 会话壳视觉五缺陷（子模块 use.web 注册表/容器 min-h/i18n 参数）。
mod plan051_p2_tests;
// Plan 442 A5: one-shot scheduler primitives regression corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_sched_tests;
// Plan 442 B-support: web-platform global bridges regression corpus.
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_webcompat_tests;
// Plan 442 C2: parser regressions for the Bug A family (method chains on
// use.rs-imported fn calls in argument position).
#[cfg(test)]
mod plan442_parser_regress_tests;
// Plan 442 Phase B gate: headless link probe against the real auto-musk
// corpus (sibling checkout). #[ignore] — manual-only, never in CI.
#[cfg(all(test, feature = "ui-iced"))]
mod plan442_musk_probe_tests;
// Plan 442 Phase C1 probe: VM-direct run of the auto-musk backend corpus
// (a2r sources). #[ignore] — manual-only, never in CI.
#[cfg(test)]
mod plan442_musk_backend_probe_tests;
// PLAN-059 T9 探针（musk DeleteConfirmDialog 端口链实机复现）：零参父
// handler 的子→父声明式路由帧对齐契约。
#[cfg(all(test, feature = "ui-iced"))]
mod plan059_child_emit_probe_tests;
// PLAN-536 探针套件（timer 失效/Init 挂载/absolute 悬浮/modal 绑定）。
// PLAN-536 T10 复挂载：声明原随 T1 落 lib.rs（6189f679f），被 b4d1ced7b
// （539 折叠前同步合并）冲突解决时静默丢弃——全套 17+ 探针自此在 master
// 为死代码未编译，复审遗漏扫描立案；本行恢复挂载。
#[cfg(all(test, feature = "ui-iced"))]
mod plan536_t1_reactive_probe_tests;
// PLAN-536 T12 探针（musk 发送链最小同形）：一参回调链帧对齐复验（KD-493①
// 在 aa92a821e 后的组件层面）+ PollStream 兜底链活性（streaming 置位先于
// Sse.open 抛点）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan536_t12_send_chain_probe_tests;
// PLAN-083 T-01：异步 HTTP 消息桥（Http.get_msg）——派生线程完成项入队、
// 载荷协议 {"ok","status","body"}、␟s␟ 载荷编码经 decode_payload 回填
// handler 的字符串实参往返（decode_payload 在 ui::dynamic，随 ui-iced 门）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan083_http_msg_bridge_tests;
mod auto_down_vm_server_probe_tests;
mod autodown_codegen_debts_tests;
// Plan 049 (auto-musk) style-parity: class.rs 支持度探针 + 对拍 dump（手动门,
// 跨仓 sibling 布局;T1 映射草案逐类断言见模块头注）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan449_style_parity_tests;
// PLAN-593（Design 29 Phase 1）零漂移基线：语义色期望表 + base_css 金样
// （先钉后改；S4/S5 registry 改造后同组断言即零漂移证明）。
// R1 门修正：断言面含 ui::style::theme（feature="ui" 门）——tf 档（无 ui）
// 编译期排除，日常档 t（ui-iced ⊃ ui）全量执行。
#[cfg(all(test, feature = "ui"))]
mod plan593_theme_registry_tests;
// Plan 606: Photo gallery thumbnails and fit parity test.
#[cfg(all(test, feature = "ui-iced"))]
mod plan606_gallery_tests;
// PLAN-615 T-07: calc 011 Programmer HEX 首光回归（VM 侧逻辑锚）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan615_calc_prog_tests;
// Plan 046 (auto-musk T2): obj receiver method family regression corpus.
#[cfg(test)]
mod plan046_obj_natives_tests;
#[cfg(test)]
mod plan367_viewfn_tests;
// Plan 408: component fn → independent Vue SFC synthesis (P2 residuals).
#[cfg(test)]
mod plan408_tests;
#[cfg(test)]
mod native_css_tests;
#[cfg(all(test, feature = "ui-iced"))]
mod plan370_015_behavior_tests;
// Plan 409 §6: link 子组件 VM 渲染缺口 regression tests (needs the gallery
// sources + the ui-iced interpreter path).
#[cfg(all(test, feature = "ui-iced"))]
mod plan409_tests;
// Plan 412: Layout Gallery + VM 布局引擎对齐 integration tests(12 页构建 +
// grid 重派生 + col-span 元数据 + 路由登记)。
#[cfg(all(test, feature = "ui-iced"))]
mod plan412_tests;
#[cfg(test)]
mod plan446_j1_repro_tests;
// Plan 437 Phase 2: VM 轨子组件 Init 生命周期钉子(渲染期 props→Init→build)。
#[cfg(test)]
mod plan437_child_init_tests;
#[cfg(test)]
mod plan352_tests;
#[cfg(test)]
mod plan353_tests;
// Plan 359 Phase 5 (G1/G2/G3): Concurrency bug fixes (spawn, generic types, channels)
#[cfg(test)]
mod plan348_concurrency_tests;
#[cfg(test)]
mod plan484_chart_component_tests;
// Plan 498: chart 交互状态机——M0 mouse-area on_click 引擎臂冒烟 +
// M1-M4 悬停态/legend 点击显隐断言(484 冒烟扩展)。
#[cfg(test)]
mod plan498_chart_interaction_tests;
// Plan 492: 引擎正确性专项——f-string 括号插值/primary-shorthand `[` 后缀/
// vue 文本内容表达式臂/包组件单 VM 编译链三族修复。
#[cfg(test)]
mod plan492_tests;
// Plan 499 M2: 指针移动限频流管道 e2e——pipe-payload 全链送达逻辑坐标。
#[cfg(test)]
mod plan499_pointer_stream_tests;
#[cfg(test)]
mod plan499_engine_float_to_int_tests;
// Plan 503: 桌面视觉刷新——style 串循环成员插值（VM/vue 双端）回归。
#[cfg(test)]
mod plan503_tests;
// PLAN-058（auto-down）：转介单②三件引擎缺口探针/回归锁——nanbox 整值
// float 保真（043 T6 债）+ use 子件 handler 体 computed 解析 + 引号 emit
// 计算实参派发路由（语料 test/ui/plan058_child_emit_computed）。
// 门与 musk p053_8 语料同款：探针走 DynamicComponent/VM 解释器，需
// ui-iced（tv/tt 等 feature 贫集档不编译本模块）。
#[cfg(all(test, feature = "ui-iced"))]
mod plan058_engine_gap_tests;
// PLAN-063（auto-down）：handler 参数槽 float 算术/比较坍缩回归锁（T-04b）——
// call_handler_for/push_value 派发路（demo 同构），与 058 on_with_input 路互补。
#[cfg(all(test, feature = "ui-iced"))]
mod plan063_handler_float_tests;
// PLAN-670 F-W3：code_editor 事件参数绑定（oninput/oncursor 循环变量/字面量/
// 点路径三类实参，062 T9 input 同款语义镜像）回归锁。
#[cfg(all(test, feature = "ui-iced"))]
mod plan670_code_editor_event_tests;
// PLAN-089(musk T-07①)：mouse-area press 链回归锁——wrapper=Container+
// content 入树+path 对齐（builder push(0) ↔ extractor 下探），MCP press
// 仪器面经 styled_vtree→path→extract 全链可达 on_click。
#[cfg(all(test, feature = "ui-iced"))]
mod plan089_mouse_area_press_tests;
// Plan 502 M1: diagram 标签发射——svg <text> 直通(vue 上下文分流 + VM
// svgdoc 内容序列化)与 overlay 动态 arbitrary 值双轨对照回归。
#[cfg(test)]
mod plan502_diagram_tests;
// Plan 492 M2 (族 A1): primary-shorthand `[` 后缀解析回归。
#[cfg(test)]
mod plan492_m2_tests;
// Plan 492 M3 (族 B): vue 文本内容表达式求值臂探针/回归。
#[cfg(test)]
mod plan492_m3_tests;
// Plan 492 M4 (族 C): 包组件单 VM 编译链分叉定位与修复。
#[cfg(test)]
mod plan492_m4_tests;
// Plan 492 M5: 包组件编译失败显式诊断(装载层+合成层)。
#[cfg(test)]
mod plan492_m5_tests;
// PLAN-053: auto-musk VM 轨上游缺陷跟踪伞回归锚(null 家族等值 /
// computed+helper 链)。
#[cfg(test)]
mod musk_vm_track_tests;
// PLAN-639: 跨包 blueprint .at 解析双轨消费门（VM 渲染结构 + vue 发射面）。
#[cfg(test)]
mod plan639_bp_tests;
// PLAN-640: Tier-0 官方默认集扩容门禁（全包契约完整面 + 代表包双轨断言）。
#[cfg(test)]
mod plan640_bp_tests;
// PLAN-643: chart 裸名归属统一（tag 双态归属）——with_charts 变体双轨 +
// palette 包词汇面正断言。
#[cfg(test)]
mod plan643_chart_tag_tests;
// PLAN-645: bps 扫描 fn 转译（DEBTS 070 第二行）——组合形态 reference 跨文件
// fn 内联正/负断言 + 047 组合夹具消费方面。
#[cfg(test)]
mod plan645_bp_tests;
// PLAN-647: bp 版本面裁定护栏（contract Q5）——spec frontmatter 版本键拒绝
// + pac.at dep 版本类键硬失败双负测试。
#[cfg(test)]
mod plan647_bp_version_tests;
// PLAN-649: bp 消费地基——解析链连字符变体探测（SD-01/contract Q5）+ icon
// 词汇面（SD-02）+ L1 直连端到端（AC-01 双轨）。
#[cfg(test)]
mod plan649_bp_tests;
// PLAN-657: L1 组装样板（047-bp-admin）——四包五变体直连全量 VM/vue 双轨
// 回归锚 + 参数化语料面（palette 零漂移 + dep 探测）。
#[cfg(test)]
mod plan657_bp_admin_tests;
