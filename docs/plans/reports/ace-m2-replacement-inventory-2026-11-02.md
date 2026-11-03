# ACE M2 全场景替换盘点与发布关键路径（2026-11-02 冻结，2026-11-03 审计执行）

> PLAN-922 交付物。**纯只读审计快照 + 拆包**：本报告不执行任何迁移/切默认/disable/认证，
> 不修改跨仓产品；M2 门是否通过只能由 REP/V06 门的实际执行收据证明（见
> [M2 合同 r2](ace-replacement-first-m2-contract-2026-11-02.md)、
> [v0.6 发布门增补](ace-v06-m2-release-gate-2026-11-02.md)）。
> 盘点口径与证据边界规范见
> [docs/specs/ace-migration/replacement-scope-and-evidence.md](../../specs/ace-migration/replacement-scope-and-evidence.md)（SD-01/02）。

## 0. 读取基线（T-01）

| repo | branch@SHA | 状态 | 备注 |
|---|---|---|---|
| auto-lang | master@4276f493b | 54 项 docs/** WIP + 3 非 docs 未跟踪（examples/ui/015-notes/、scripts/tools/native-fixture/Cargo.lock、test/fixtures/reader-import-budget/） | 919 merge 在途（lang-919 树）；921 executing（lang-921 树 9c81ee6d7）；本审计只读 |
| auto-down | master@7c4d5cb | clean | jade-garden 生产服务在内 |
| auto-ui | master@cfe325d4 | **6 项代码 WIP**（dynamic_build.rs、ui_tests.rs 改动 + 未跟踪 stylekit 测试/test/、docs/design/autoui/） | 他方会话 WIP，只具名不消费 |
| auto-man | master@47d8f7b | clean | |
| auto-web | master@3de5524 | clean | |
| auto-os | main@65fc70d | 16 项 WIP（apps/auto-charts 子模块指针 + docs/** 等） | |
| auto-gen | master@593b4ee | clean | 对新 main **已不可编译**（见 F-06） |
| auto-musk | main@de92b90 | 仅 pycache 未跟踪 | |

其余（auto-edit、auto-launcher、auto-vscode、auto-zed、auto-coder、auto-code-rs、auto-ai、book、ag-examples、scratch-consumer、jade-garden）按只读 grep/清单核对。跨仓 WIP 一律不当已批准交付（计划 §5.4）。

## 1. 迁移分母主表（T-02/T-03）

行身份 = 类别前缀 + 序号。列：入口/目标（file:line 证据）、旧 producer、实际 consumer、
分类（分类定义见 SD-01：**IV**=implemented-verified / **IW**=implemented-unwired /
**ES**=evidence-stale / **CM**=capability-missing / **UK**=unknown）、M2 后继（§4 工作标签）。

### A. 语言仓公共入口（auto-lang）

| 行 | 入口（证据） | 旧 producer | 实际 consumer | 分类 | 后继 |
|---|---|---|---|---|---|
| A-01 | 执行漏斗 `run`/`run_with_capture*`（core entry.rs:8-163 → run_bridge.rs:18/:57 → lib.rs:210 `execute_autovm_with_deadline_and_input`：CompileSession→Parser→CTEE→vm::codegen::Codegen→loader::Linker→AutoVM::run_task_loop→extract_autovm_result lib.rs:762） | 旧 VM 全管线 | UI vm 模式非 UI 文件（run_vm.rs:42）、playground、book、facade 测试 118 文件 | **CM**（AV 有执行核对等 M1 corpus，但无 facade 对等入口/无产品接线） | M2-EXEC |
| A-02 | `eval_config_with_vm`（facade lib.rs:284 → entry.rs:735；config.rs:31/90 `AutoConfig::from_file/new/read` 全部落到此） | 旧 VM config 求值（`Codegen::new_for_config`） | **生产**：AutoMan 全命令配置求值（pac.at 双过 probe+final automan.rs:123-169、am.at:2896、index.at、.am/state.at、成员 pac）、AutoUI（run_vm.rs:225、rust_ui.rs:3374/6934/6950、vue.rs:2740）、auto-man tests、auto-man facade | **CM**（ACE 无 config 求值入口） | M2-EXEC / M2-MAN |
| A-03 | `run_file`/`run_file_with_args`/`run_with_session`/`run_with_mode`/`run_file_with_auto_mode`（facade lib.rs:414/438/386/594/698） | 旧 VM | jade-garden vm_server.rs:41、auto-musk vm_backend.rs:63、auto-web main.rs:19、Man cmd_run、playground、facade/集成测试 | **CM** | M2-EXEC |
| A-04 | 调试族 `debug_file(_agent)`/`create_vm_from_source`/`create_vm_from_abt`/`run_abt`（lib.rs:449/459/261/273；core entry.rs:2979/3037；vm::debugger Gdb/JsonAgent 控制器） | 旧 VM debugger | Man `auto debug`（main.rs:919,924）、playground agent_debug/session.rs:61、debugger/session.rs:28 | **CM**（ACE 无调试器接面） | M2-DEBUG |
| A-05 | 转译族 `trans_rust_merged`/`trans_python`/`trans_javascript`/`trans_gdscript`/`trans_tscn`/`trans_godot`/`trans_typescript(_to)`/`transpile_r2a_file`/`trans_auto_s2s`/`dump_lowered`/`trans_c`/`trans_rust(+_with_session)`/`validate_rust_output`/`init_rust_ffi`/`init_py_ffi`（facade lib.rs:422-535、:3-19；core script_entry.rs:32-575、entry.rs:2386-2914） | 旧 AST Trans 栈（见 B 表） | Man `auto c/rust/python/.../r2a/trans`（cmd_trans.rs 全 13 点）、watch（cmd_watch.rs:430）、UI tauri/ts/ark、Web api_gen、book、playground、stdlib 发射 | **CM/IV 混合**（Rust/C 有新 HIR provider=IV 域内；其余目标全 CM，见 B 表） | M2-A2X-* / CA-* |
| A-06 | REPL/会话/daemon：`autovm_repl::main_loop`（autovm_repl.rs:236）、`AutovmReplSession::run`（autovm_persistent.rs:461，增量保状态）、`AutovmDaemon run_stdio/run_pipe`（autovm_daemon.rs:124/168）、`AutovmClient`/`eval_one_shot`（autovm_client.rs:327）、`McpServer`（mcp/server.rs:24）、mcp/session_manager.rs | 旧 VM 会话态 | Man `auto old-repl`/无参 REPL（main.rs:769,930）、`auto serve`/`req`（cmd_req.rs）、`auto mcp`（main.rs:867）、plan082 测试 | **CM**（ACE 无 REPL/daemon/MCP/持久会话入口；AV resident/hot-reload 能力已交付 866 但无这些 CLI/协议面） | M2-EXEC / M2-SESSION |
| A-07 | 语言内测试 natives `shim_test_run_{vm,a2c,a2ts,a2r}_dir`（vm/ffi/stdlib/sys_env.rs:998-1110+ → entry.rs:927/1108/1229/1365）+ `test_file`/`test_code*`（entry.rs:3280 等）+ `test/runners/{vm,a2r,a2ts,a2c}_tests.at` | 旧 VM 执行 `#[test]` | Man `auto test`（cmd_test.rs:106）、自举测试套 | **CM**（ACE 无测试执行入口；M1 TestCase 协议是父侧 harness 非 #[test] 语义） | M2-TEST |
| A-08 | 仓内二进制：`auto-vm`（crates/auto-vm main.rs:35 生产 VM CLI）、`auto-case`（bin/auto-case.rs:354/427/467 avm/a2r 适配器→旧引擎；`--adapter ac`→ACE worker ac.rs:38）、`a2r-emit-917-cert`、`test-case-pilot`、`ace-certify`（ACE 侧）、`autogen`（bin/autogen.rs:235/259 AutoInterpreter）、`auto-playground`（HTTP 服务 /api/run、/api/notebook/transpile 全目标） | 旧引擎/转译器 | 开发者/工具入口；playground 为生产 HTTP 服务 | **IW**（ac 适配器=唯一 ACE 接线，测试工具级） | M2-EXEC / M2-RETIRE |
| A-09 | 引擎选择 `execution_engine.rs:9-48`：单变体 AutoVM；`AUTO_EXECUTION_ENGINE=evaluator` 弃用仍 AutoVM | — | 全仓 | **CM**（无真分派点） | M2-EXEC |
| A-10 | `pub use auto_lang_core::vm` 整包（facade lib.rs:170-177，含 vm::engine/ffi/host_bridge/backend_abi/host_capabilities）+ `interpreter`/`trans`/`compile`/`autovm_*`/`mcp`/`test_cases`/`test_runner`/`comptime`/`a2r_std` 整模块 re-export | 旧引擎内部面 | **生产直连**：UI vm_bridge（127 处引用）/back_proxy/handler_codegen/host capabilities、Web dispatch、OS term/backend_abi、musk host_bridge/extern_impl、jade-garden host_bridge | **CM**（ACE 无对等嵌入/宿主 API；这是 REP-06 宿主完整性的核心缺口） | M2-HOST |
| A-11 | `AutoInterpreter`（interpreter/mod.rs:60/136 eval_template）树遍历解释器（VM backed） | 旧解释器 | **生产**：AutoMan 模板 new/init（asset.rs:43-47）、auto-gen（已断）、autogen CLI | **CM** | M2-MAN / M2-GEN |
| A-12 | `a2r_std`（生成物运行时 prelude；lib.rs:67） | 中立 | a2r 生成物全消费者（coder/ai/edit/os apps/musk/book 夹具） | **中立库，保留**（SD-02 归属规则） | M2-RETIRE 分类 |

### B. 全部 A2X 目标（旧 AST Trans 栈，core trans.rs:6-18；全消费 AST+`Sink` 经 `Trans` trait trans.rs:158）

| 行 | 目标 | 旧入口（证据） | 生产/测试 consumer | 分类 | 后继 |
|---|---|---|---|---|---|
| B-01 | **Rust (a2r)** | `RustTrans`:92、`transpile_rust`:2308、`transpile_rust_project`:2742、`_merged`:3570 + 10 子模块 | **生产**：auto-generation-core/shared_generation.rs:106/160/261/288（生成管线）、Man target.rs:93/613 + cmd_trans、Web api_gen.rs 7 点、UI rust_ui.rs:1050/1071/1091 + tauri_backend.rs:130-141、watch、a2r-emit-917-cert、stdlib_assembly/emission、book harness:27、auto-code-rs a2r-project（休眠） | **IV（AX-A 域内）**：HIR→Rust 新 provider 已交付（892/902），**但产品生成管线仍全走旧 Trans**（=IW 接线缺口）；a2r cert 缺口（binding a2r 列 emit-ok-run-unverified/snapshot-only-cert-gap） | CA 接线 / M2-A2X-R |
| B-02 | **C (a2c)** | `CTrans`:34、`transpile_c`:4570 | **生产**：Man `auto c`/`a2c-stdlib`（cmd_a2c_stdlib.rs:63-87，唯一 Man 直驱 CTrans）、build/export native（target.rs:125/258 + CTranspilationCache）、stdlib_assembly/emission、book:60 | **IW**：CA-A（921）在途交付 HIR→C scalar；record/text 等 CA-B..E 未启；产品 a2c-stdlib/build 仍旧 | CA-A(在途)/CA-B..E |
| B-03 | Python (a2p) | `PythonTrans`:16 | Man cmd_trans:44/150、playground、book:101 | **CM**（无新 provider） | M2-A2X-T2 |
| B-04 | JavaScript (a2j) | `JavaScriptTrans`:8 | Man cmd_trans:56/166、playground、corpus a2j（本仓无 Rust 消费者） | **CM** | M2-A2X-T2 |
| B-05 | TypeScript (a2ts) | `TypeScriptTrans`:30 + ts_expr/ts_stmt/ts_types/ts_runtime（runtime_file_content:57） | **生产**：UI vue/jet/ark 生成器表达式 lowering（ts_adapter.rs:28/858/2383、ark_adapter.rs:20/141/629）、Man Target lang:"ts"（util.rs:185-212）、cmd_trans:92、run_a2ts_file_test、playground、book:142 | **CM**（UI 生成器生产依赖=高优先） | M2-A2X-T1 |
| B-06 | GDScript (a2gd) | `GDScriptTrans`:9 | cmd_trans:68/182、script_entry:441/496、corpus a2gd | **CM**（消费者仅 CLI/corpus） | M2-A2X-T2 |
| B-07 | TSCN/Godot | `TscnGenerator`:58、`generate_from_ast`:508、`trans_godot` | cmd_trans:198/215、corpus a2gd/tscn、examples/godot | **CM** | M2-A2X-T2 |
| B-08 | r2a 逆向（Rust→Auto） | `transpile_r2a`:14（syn） | Man cmd_trans:250、script_entry:568 | **CM**（独立逆向工具；按真实归属审查，非语言引擎义务） | M2-A2X-T2 决策 |
| B-09 | auto_s2s/emit 源改写 | `lower_source`:63、`emit_code`:21、4 规则组 | 旧 VM 管线自用（core lib.rs:236）、avm.rs 证据 | **CM**（新前端语义键取代） | M2-EXEC 内含 |
| B-10 | escape 分析 | `EscapeAnalyzer`（escape/mod.rs:15，Plan-310） | **未接线**（无输出消费） | 退役候选（历史研究） | M2-RETIRE |
| B-11 | 相邻发射器 `stdlib_assembly/emission.rs`、`vm::codegen`/`template_codegen`（字节码）、`vm::disasm` | — | stdlib a2r/a2c 分层发射、VM 字节码 | stdlib 发射随 A2X 接线迁；codegen/disasm 随旧 VM 退役 | M2-A2X / M2-RETIRE |

### C. AutoUI（auto-ui @ cfe325d4）

| 行 | 模式/面 | 证据 | 分类 | 后继 |
|---|---|---|---|---|
| C-01 | **vm/Iced 动态模式全管线** | 分派 tool_ops.rs:334-340；run_vm.rs:278（merged/split）、:71 start_vm_server（src/back/api.at 真 AutoVM HTTP server）、:254 backend_abi cdylib、:58-61 execute_vm_entry→run_file_dynamic_ui、--internal-vm-exec 子进程（host.rs:582）、back_proxy.rs:1587 自建 AutoVM::new | **CM**（REP-01/06 最大单点；AV 无宿主桥） | M2-UI |
| C-02 | vue 模式后端双档 | run_vue.rs:400-421：`AUTO_BACKEND_IMPL=vm`→旧 VM server（:405-412）；默认 rust→a2r 生成 axum（:415）；gallery 每 demo AutoVM 会话（:463） | vm 档 **CM**；rust 档随 B-01 | M2-UI / M2-A2X-R |
| C-03 | tauri 后端生成 | tauri_backend.rs:130-141 调旧 `trans::rust::transpile_rust`（api.at/db.at→db.rs；types/commands 仍 TODO 占位 :144-201） | **IW**（有新 provider 未接线） | M2-A2X-R |
| C-04 | rust(a2r) 静态模式 | rust_ui.rs:1050/1071/1091 经 auto_api_gen；pac.at 经 AutoConfig（:3374/6934/6950） | 生成 **IW**；config 求值 **CM** | M2-A2X-R / M2-MAN |
| C-05 | jet/ark 生成器 TS lowering | ts_adapter.rs:28-29/858/2383、ark_adapter.rs:20-21/141/629 委托旧 TypeScriptTrans | **CM**（B-05 前置） | M2-A2X-T1 |
| C-06 | vscode 模式 | vscode.rs（脚手架，消费 gen/vue/dist） | 无引擎依赖 | 不迁（观测确认） |
| C-07 | **VM 内部直连面** | vm_bridge.rs:43-47（AutoVM/ParkedSegment/Linker/AutoTask/VirtualFlash，127 引用）、handler_codegen.rs:26 真 VM Codegen 合成 handler、read_state/call_handler（:1019/:3484 经 call_fn_by_name_segment）、host_capabilities natives 30+（runtime lib.rs:90-124：terminal 34/code-editor 22/clipboard/scroll/frame/i18n/dialog/DnD/media）、窗口退出 subscriptions.rs:242-268、热重载 hot_reload.rs、panic-probe:225-229 | **CM**（REP-06 核心证据：UI 深耦合 vm 内部 API，非 facade） | M2-HOST / M2-UI |
| C-08 | vendored frozen SDK 副本 | experimental/autographics-auto-driver/.sdk-source + sdk/frozen（全量引擎 vendored，钉死） | 退役图**必须**含副本清理（REP-10 包内容） | M2-RETIRE |
| C-09 | aura 数据模型 | types.rs:1089-1104 `LogicPayload{AstStmts, Bytecode}`（Bytecode 为 VM 保留臂，当前只产 AstStmts） | 随 C-01 重定义 | M2-UI |
| C-10 | 测试面 | ui_tests.rs 110 模块 + 集成（g11/g16/gallery_golden/ui_snapshots/desktop_behavior 等） | REP-11 测试迁移义务 | M2-TEST-MIG |
| C-11 | 安装 | 无仓内发布打包；Vue/Tauri 工程运行时生成（tauri_backend.rs:59-60）；资产 rust-embed 内嵌；auto-ui-tool 由 Man CLI 子进程 spawn（Design 37） | 发行物=Man 安装器载荷 | M2-REL |

### D. AutoMan（auto-man @ 47d8f7b）

| 行 | 面 | 证据 | 分类 | 后继 |
|---|---|---|---|---|
| D-01 | 裸脚本/`run`/`service` vm 轨 | cmd_run.rs:32/54/68；automan.rs:1733-1764 execute_autovm_with_path（32MB 线程）；http_service.rs:81-125 serve_vm | **CM** | M2-EXEC/M2-MAN |
| D-02 | 配置求值（**全部命令入口**） | pac.at 双过（automan.rs:123-169）、am.at:2896、index.at（index.rs:56/122/145）、.am/state.at、pac.atom.at:951、成员 pac（:1255/1836/2003）、依赖 pac（pac.rs:717）、up.rs:31/87（CLI 臂已禁） | **CM**（A-02 前置） | M2-MAN |
| D-03 | `auto test` | cmd_test.rs:106 test_file 旧 VM `#[test]`；runners/*.at | **CM**（A-07 前置） | M2-TEST |
| D-04 | 转译命令面 | cmd_trans.rs 13 点全旧家族；**a2c-stdlib 生产 CTrans**（cmd_a2c_stdlib.rs:63-87） | B 表逐行 | M2-A2X-* |
| D-05 | build/export native | target.rs:125/258 transpile_c+CTranspilationCache:201、:93/613 transpile_rust(_project)、:517-529 TauriGenerator、CMake/IAR/GHS exporter+ninja builder | **IW/CM 混合** | CA-*/M2-A2X-R |
| D-06 | 模板 new/init | asset.rs:43-47 AutoInterpreter（PLAN-803 冻结语义） | **CM** | M2-GEN |
| D-07 | REPL/daemon/req/debug/mcp/cffi | main.rs:769/873/890/917-926/820-864 | **CM** | M2-SESSION/M2-DEBUG |
| D-08 | 静态项目模型新臂 | project_model_adapter.rs:1-14（auto_project_model 仅 probe，"无生产 pac 读路径消费"，旧 VM oracle 对照） | 已有中立实现待接线 | M2-MAN |
| D-09 | 安装器 | man-install.py（core/full、staging、SHA256 manifest、清 7 env :29-45）；载荷 auto.exe+auto-ui-tool+sdk/ 源+stdlib .at；stdlib 解析序 stdlib_assembly/loader.rs:25-50 | REP-08/11 义务 | M2-REL |

### E. AutoWeb（auto-web @ 3de5524）

| 行 | 面 | 证据 | 分类 | 后继 |
|---|---|---|---|---|
| E-01 | CLI `auto-web run` | main.rs:19 run_file（with_http_service_launch） | **CM** | M2-WEB |
| E-02 | 宿主实现（owner-thread 模型） | host.rs:8-130（HttpServiceHost→serve_activated→Rc<AutoVM> 拆装）、http_server.rs:3012-3315（net 线程+owner loop）、server_dispatch.rs:699/856-859（vm.spawn_task+call_fn_by_name_segment_http/call_closure_segment）、parked 恢复 :3776/:4032、RequestScope 状态机 :3602-3619；拓扑契约：VM !Send 恒 owner 线程 | **CM**（AV 无 !Send 会话/HTTP 分派对等；**结构证据**：新引擎需对等 owner 会话 API） | M2-HOST/M2-WEB |
| E-03 | 阻塞档位与嵌入面 | http_server_handle.rs:10-208（ServerHandle 手写 HTTP/1）、server_legacy.rs:66-73；serve_owned（host.rs:47-59）**仓内无调用方**（公开嵌入面，外部 SDK 消费者 UK） | **CM** + UK | M2-WEB |
| E-04 | 测试 | http_e2e ~73 用例（真引擎+服务器，feature 门）、plan326/669 直建 AutoVM、http_runtime 子进程 | REP-11 | M2-TEST-MIG |
| E-05 | 生成 Rust 轨 | api_gen.rs 7 点 transpile（**不用 VM**）；axum0.7（VM 轨 0.8 双轨） | 随 B-01 | M2-A2X-R |
| E-06 | 发行 | delivery.md 20 SDK 包、web-install.py、standalone-consumer 夹具、rust_workspace.rs 生产源选择（env→安装 sdk/→repo）+ 910 生成成员身份 | REP-08/11 | M2-REL |

### F. 其余仓消费者（含安装/发行）

| 行 | repo/面 | 证据 | 分类 | 后继 |
|---|---|---|---|---|
| F-01 | auto-os host/desktop | main.rs:46-49 set_ui_host_install_hook、health.rs:96-141（storage_host_read、resolve_auto_cli、提示 `auto build -r rust`） | **CM**（依赖 A-10/C-07） | M2-OS |
| F-02 | auto-os apps/*（git 子模块 Auto 源应用）+ auto-term sidecar + at-gen + auto-os-config-back | term.rs:191-917（旧 ui::terminal/mem_guard/storage_host_publish）、at-gen main.rs:12-214（旧 ui 面生成物）、os-config-back lib.rs:13（vm::backend_abi Plan-061 ABI）、regen.sh（target/debug/auto.exe build --gen-only） | **CM**（旧 `auto_lang::ui` 面已从 facade 移除——**这些调用方对新 main 的可编译性 UK**，见 §7 UK-03） | M2-OS |
| F-03 | jade-garden back/server（auto-down 仓，生产 daemon） | vm_server.rs:12/17/41 host_bridge("jade.api")+run_file；vendored 副本另存于 auto-os apps/auto-notes/vendor | **CM** | M2-OS |
| F-04 | auto-musk `MUSK_BACKEND=vm` 轨 | vm_backend.rs:63 run_file(vm_entry.at)+:170/175 host_bridge、extern_impl.rs:4033 vm::ffi insert_http_response；生成 back 用旧 ui::image_pipeline/media_service；rust-workspace 生成物钉 auto-lang path（生成来源 UK，疑早于 906/910） | **CM** | M2-OS |
| F-05 | auto-edit（specs/rust-workspace 生成物） | main.rs:3/8/193-489+ 重度旧 auto_lang::ui 面 | **CM**（同 F-02 可编译性 UK） | M2-OS |
| F-06 | **auto-gen**（模板生成器工具） | lib.rs:90/93 `auto_lang::Universe`/`interp::Interpreter`——**均为已移除 API，对新 main 不可编译**；autodbc/autodcm 用 pre-rename `autolang::`/`autoui::`（休眠）；templates/ 目录缺失 | **ES**（stale：声称消费者但证据非当前祖先）——owner 裁定复活（迁 `auto_lang_core::interpreter`）或归档 | 用户决策 → M2-GEN |
| F-07 | auto-vscode | build-lsp.ps1:19-55 捆绑 auto-lsp exe；extension.ts:218-231；autoLang.build/run 命令 | 发行物依赖（REP-08 安装图） | M2-REL |
| F-08 | auto-launcher | 无 crate 依赖；README `auto run`/`-r vm`/`build -r rust`；tests/drive_vm.py | CLI 级消费 | M2-REL 验证 |
| F-09 | auto-coder / auto-ai | 仅 a2r-std prelude（coder/rust Cargo.toml:8；ai 各 rust/ 子 crate）——生成物运行时，无引擎 API | **中立保留**（A-12） | M2-RETIRE 分类 |
| F-10 | book | harness.rs:27/60/101/142/190/200（trans rust/c/python/ts + run/run_with_capture）+ build.rs:107-140 注册 | 测试 harness（REP-11 测试迁移） | M2-TEST-MIG |
| F-11 | ag-examples | 外部 AutoGraphics 引擎二进制显式传入（gallery.py --engine；engine-lock.json 钉 autographics-dev 分支 8b12632d） | 独立引擎线（非旧 AutoVM 消费者）；观测登记 | 不迁（边界行） |
| F-12 | 安装器/启动器 | auto-musk/launch-vm.cmd:27 硬编码 `auto.exe run --render vm`、vm-first-run.mjs:50、start-all.cmd 族（cargo run/npm）、auto-os regen.sh、vscode build-lsp | REP-08 全入口安装图 | M2-REL |

### G. 旧实现退役分类（T-03/V-04；顺序见 §5 退役表）

| 类 | 内容 | 处置原则（SD-02） |
|---|---|---|
| G-1 旧执行器 | vm::{engine,codegen,loader,debugger,disasm,abt,native_registry,virt_memory,ffi::stdlib 大部}、interpreter、autovm_{repl,persistent,daemon,client}、mcp、execution_engine 分派、run_bridge | 删除（Git 追溯）；能力由 ACE AV/AC 对等入口承接 |
| G-2 旧 AST 转译栈 | trans.rs 全家（rust/c/python/js/ts/gdscript/tscn/r2a/s2s/emit/py_known/escape）+ stdlib_assembly/emission 旧臂 | 删除；新 HIR provider 承接（Rust/C 在途）；无消费者目标随决策 |
| G-3 薄 compat/embedding API | facade re-export 面（vm/trans/interpreter/...）、`AutoConfig` 求值、a2r_std | 保名转发 ACE 或迁中立；a2r_std 中立保留 |
| G-4 中立库 | auto-val/auto-ast/auto-lex/auto-diagnostics/auto-parser-api/auto-atom/auto-project-model/auto-generation-core（生成载体）/auto-tool-*/auto-media-core/auto-web-contract | 按真实依赖核清保留（禁止夹带旧执行循环/发射器） |
| G-5 测试/注册/构建 | test_cases 协议 avm/a2r 适配器（mod.rs:751-760）、test/runners/*.at、a2r-emit-917-cert、book harness、UI 110 模块、corpus a2vue/a2ark/a2j（本仓无 Rust 消费者）、auto-case legacy-inventory 子命令 | successor ledger 迁移义务后退役；corpus 归属逐项裁定 |
| G-6 发行/副本 | man-install stdlib 旧载荷路径、a2c-stdlib CWD 约定、**vendored frozen SDK 副本 ×2**（autographics driver + auto-os notes vendor）、auto-lsp 捆绑、launch-vm.cmd | REP-08/10 安装包内容清零 |
| G-7 UI 宿主/渲染器 | iced renderer、host_capabilities natives、vm_bridge、back_proxy | **不盲删**（SD-02 中立规则）：随 M2-HOST 提供对等宿主接缝后改接 |

## 2. 能力/接线/证据分账（T-04/V-02、V-05）

### 2.1 M1 corpus 分账（binding.tsv 131 数据行/12 列 @ 4276f493b）

- **av（ABC core）列**：core/closures-source(823/865)/dyn(824)/resource(915)/resident+hot-reload(866)/console(863)/immutable(920) 已 ok——AV 侧 M1 语料基本闭合；
- **ac（native）列**：`gap:ac-composite-native` 12 + `gap:ac-console-profile` 15 + `gap:mn-scalar-only` 6 = **33 gap 格**（native composite/collections、console ABI、宽域）；
- **avm（legacy）列**：16 `diff-legacy-value` + 5 `no-typed-entry` + 9 `no-dyn-api` + 4 accepts + grammar/runtime 差异——legacy 分账已登记（差异行非绿）；
- **a2r 列**：`emit-ok-run-unverified`/`snapshot-only-cert-gap` 多格（golden 冻结但伪 Rust 不可编译——cert 缺口，917 在途）；
- B18 avm 格 ok:P920；旧 required 静态 14 格中 11 格已闭，余 **native R02/R03/R05 三格 open**——2026-11-03 另一会话已立 923 NF（0/12，fault/Place，关 R03+版本化 R02 部分）与 924 NC（0/14，有限 native 闭包，回填 R02/R05）认领，均 drafting 仅规划（merge 漂移刷新，见 §9）。

### 2.2 ACE 能力 × 产品场景映射（关键结论）

| 能力域 | ACE 现状（证据） | 产品需要（分母行） | 结论 |
|---|---|---|---|
| 语义执行（core/scalar/record/text/closures/dyn/resource/console/resident） | AV 已交付并有独立 oracle（specs/auto-av/*、binding av 列） | A-01/02/03、C-01、D-01/02、E-02 | **implemented-unwired**：能力在、产品入口零接线 |
| 嵌入/宿主 API（vm::engine 直连、host_bridge、backend_abi、host_capabilities、HttpServiceHost、spawn_task/call_fn_by_name、parked 段） | **无对等**（experimental/ace 无 host 接缝 crate） | A-10、C-07、E-02/03、F-01..F-04 | **capability-missing**（REP-06 核心） |
| REPL/持久会话/daemon/MCP/调试器 | 无（AV resident 有会话能力 866 但无这些协议面） | A-04/06、D-07 | **capability-missing** |
| `#[test]`/shim 测试执行 | 无（TestCase 协议是父侧 harness） | A-07、D-03、F-10 | **capability-missing** |
| config 求值（AutoConfig） | 无对等入口 | A-02、D-02、C-04 | **capability-missing** |
| HIR→Rust | a2x-rust IV（892/902） | B-01 产品管线 | **implemented-unwired** |
| HIR→C | CA-A 921 executing（1/14，身份已冻结） | B-02 | in-flight |
| Python/JS/TS/GDScript/TSCN/Godot provider | **无** | B-03..B-08（其中 TS=UI 生成器生产依赖） | **capability-missing** |
| native composite/collections、console ABI | ac 列 33 gap；native 明示拒 console（host-import-and-console.md "native console ABI is added=无"） | M1 native 格 + 未来 AC 产品面 | **capability-missing** |
| stdlib/host FFI 面 | 有限（M2 合同 §3"只补替换所需"） | A-10 natives 30+、C-07 | 逐项切片（M2-HOST 内） |

### 2.3 native 三格边界（V-05/D-02；2026-11-03 刷新：已由 923/924 认领）

- **认领状态**：本报告成稿时三格无立项；merge 漂移核对发现 2026-11-03 另一会话已立 **923 NF**（native 分配故障原子性与 Place 代际，drafting 0/12，独立关 R03+交版本化 R02 部分）与 **924 NC**（有限 native 闭包源码+R05+完整 R02，drafting 0/14，依赖 923）。本节证据与边界结论不变，仅后继归属从拟议标签刷新为实际计划。
- **R02（构造/clone 失败注入）**：av 列 ok:av-resource-915（915 交付 fault/Place 事实）；native 列 gap:ac-composite-native——915 的 fault 注入通道 owner=852 T-05 登记，native 侧需把 915 设施映射到 mn 配方（证据：915 spec + collections_driver MnError::Admit 通道 collections_driver.rs:11/46）。→ 923（部分）/924（完整回填）。
- **R03（活借用/旧 buffer）**：av ok:av-resource-915；native gap:ac-composite-native；avm 列 diff-legacy-value 已登记。同 R02 通道。→ 923。
- **R05（closure 创建者退出）**：av 列 ok:av-closures-823；**native 不支持闭包环境**——`collections_driver.rs:46` "collection function imports cannot carry closure environments" + `check_closure`:109 拒绝。**capability-missing**（本报告独立实证，与 924 立项依据一致）：需 native 闭包 ABI/lowering（源码/typed/MIR/执行 ABI 边界），不可从 collections 推定，也不自动 NA。→ 924。

### 2.4 C 后继（沿 921 接缝）

CA-A（921，在途）→ CA-B record/text/text-record → CA-C 集合/变体（List/Variant，AX-C 未启）→ CA-D 有限 public ABI（797 descriptor 不动）→ CA-E 四端/M1 总签章。各保独立资源/ABI 门（921 非目标已列）。M2 盘点不提前分配编号。

### 2.5 console 未来账（V-05"45"口径）

O01..O15 corpus 15 行 × 3 未闭轴（ac/avm/a2r 均 gap:console-profile-parity 或 gap:ac-console-profile）= **45 格**：AV 侧 profile 8 已交付（863），native console ABI 明示未加、avm/a2r parity 未接。M2 语义执行接管以 AV 承接 console（已 IV），45 格的 native/a2r 轴归各自后继，不自动 NA。

## 3. REP-01..11 / V06-ACE-01..05 门映射（T-04/V-03）

| 门 | 观察要点（合同） | 当前证据基线 | 拟后继（owner 仓） | 独立关闭 oracle 提要 |
|---|---|---|---|---|
| REP-01 原源与全消费者 | 全分母+新入口/owner 齐、原源/诊断/副作用符合 oracle | 本报告 §1 分母（首版）；产品入口零 ACE 接线 | M2-DEN（语言）+ 各域包 | 冻结分母逐行 + 独立原例 oracle（SCENARIOS/金样/行为差分） |
| REP-02 Rust 构建隔离 | 稳定宿主 Auto 编辑 rustc/cargo 0 | AV resident（866）已证会话内换版；**产品 IDE/编辑循环未接** | M2-EXEC→M2-UI | 真进程树观测（wmic/ETW 级）+强制 cargo 反控红 |
| REP-03 全局实际依赖 | 未改函数每函数工件 IO0、代际保持 | AX-B formal 窗口 hit/restart/body-edit P90=6/21/40ms（单函数域）；全局多模块/stdlib 未证 | M2-EXEC/全局增量包 | 跨工程/target/权限负依赖真实失效 |
| REP-04 真实在线热更新 | 同 PID 兼容编辑、旧链保活 | 866 resident-online（AV）；产品编辑器未接 | M2-UI | 同 PID 新旧调用链/失败原子 |
| REP-05 恢复与工作集 | fresh/hit/restart/body-edit 恢复重验 | 913 恢复接口已交付（AV/store）；产品面未接 | M2-EXEC | P/R/M 分账 + 坏数据修复再读 |
| REP-06 现有宿主完整性 | 全应用 stdlib/host/FFI/生命周期接通 | **最大缺口**：host_bridge/backend_abi/host_capabilities 30+ natives/HttpServiceHost 无对等（A-10/C-07/E-02/F-01..04） | M2-HOST（语言）+域包 | 每宿主能力真实交互/回调/退出（非 import/首帧） |
| REP-07 工程级周转 | 原工程 P90≤2000ms 及更严门 | M1 formal 数字（AX-B 等）为单函数域；原工程取样未做 | 各域包收尾 | 准备/语义/生产/安装/清理分列计量 |
| REP-08 全入口实际切换 | CLI/SDK/工具/安装真实装载 ACE | 全部入口现走旧引擎（§1）；安装图 F-12/D-09/E-06 | M2-REL | clean 安装/仓外/重定位/升级/独立旧版回滚 |
| REP-09 A2X 全目标接管 | 每目标新 provider 或中立适配 | Rust IV 未接线；C 在途；**TS/JS/Python/GDScript/TSCN/Godot 无 provider**（B 表） | M2-A2X-R/T1/T2 + CA-* | 每目标实际输出+独立金样/行为规则；逆向工具按归属 |
| REP-10 旧实现彻底 disable | default+all-features 旧引擎 0、dispatch/构建单元/发行 0 | 现状 100% 旧引擎；vendored 副本 ×2、launch 脚本、a2c-stdlib CWD 等安装面已列（G 表） | M2-RETIRE 系列 | 依赖图/构建单元/真实 dispatch/包内容核对 + 故意接回负控 |
| REP-11 产品及测试迁移收据 | 各域独立安装收包运行；新测试接管旧义务 | UI 110 模块、Man/Web tests、book harness、runners/*.at 已列（C-10/D/E-04/F-10/G-5） | M2-TEST-MIG + 域包 | 独立安装物上真实负载；无 dev worktree/绝对路径 |

V06-ACE-01..05 分别由：01=M1 收尾+本表 REP 全闭总复审；02=REP-01/09；03=REP-02..05/07；04=REP-10；05=REP-08/11。无空 oracle 门；未运行判据一律 pending。

## 4. 后继有界包清单（T-05；工作标签非 Plan 号）

每包：目标 / 非目标 / 依赖 ready / 独立 oracle / 写面（P=语言仓文档+代码，S=共享短窗，H=全文件单写）/ owner 仓。

| 标签 | 目标（含非目标边界） | 依赖 ready | 独立关闭 oracle | 写面/owner |
|---|---|---|---|---|
| **M2-DEN** | 分母冻结成 successor ledger（本报告行→可机读账）+后续 SHA 刷新规程；非目标：不实施迁移 | 本报告 | 逐行来源绑定+对账脚本（审计级） | P/语言 |
| **M2-EXEC** | facade 执行族对等（run/run_with_capture/config eval/文件执行）+`AUTO_EXECUTION_ENGINE` 真分派+stdlib 所需面；非目标：不迁 UI/宿主 | AV 能力（已 IV） | 原源对照 oracle（avm 列差异行作已知差）；原例负控 | P(核心 S:script_entry/entry/lib)/语言 |
| **M2-SESSION** | REPL/持久会话/daemon/client/MCP 协议面在 AV 会话上；非目标：调试器 | M2-EXEC | 会话语义 oracle（增量保状态/快照/reset） | P/语言 |
| **M2-DEBUG** | 调试器接面（Gdb/JsonAgent 对等）或裁定 M2 范围内替代；非目标：新调试器功能 | M2-EXEC | 断点/单步/变量读原例 | P/语言 |
| **M2-TEST** | `#[test]`/test_file/shim natives 对等+runners 迁移；非目标：不改 M1 corpus | M2-EXEC | auto test 原套件通过率对账 | P(与 921 H 窗协调 test_cases/mod.rs)/语言 |
| **M2-HOST** | **中立宿主能力层**：host_bridge/backend_abi/host_capabilities/HttpServiceOwner 会话（!Send owner 线程模型对等）/spawn_task/call_fn/parked——ACE 侧对等接缝+UI natives 30+ 先迁；非目标：不实现 UI 渲染 | M2-EXEC | 每能力真实交互/回调/退出/保活原例（UI/Man/Web 各至少一切片） | P(新 crate, 与 921 a2x 接缝协调)/语言 |
| **M2-A2X-R** | 产品生成管线切 HIR→Rust（shared_generation/api_gen/rust_ui/tauri_backend/watch）+stdlib 发射；非目标：不动 a2r cert 历史 | AX-A/AX-B IV；917 cert 收口 | 生成物 byte/行为对照+真 rustc 编译运行 | P(语言)+跨仓只读指导/UI+Man+Web 各自包 |
| **M2-A2X-T1** | TS provider（HIR→TS）首目标：满足 UI 生成器表达式 lowering（ts_adapter/ark_adapter/Man ts target）；非目标：全 TS 生态 | M2-DEN 行冻结 | UI 生成前后对照金样+独立 TS 行为规则 | P(ace workspace)/语言 |
| **M2-A2X-T2** | 其余目标分级决策+实现：Python/JS/GDScript/TSCN/Godot/r2a（按消费者分级：corpus/工具级可后置）；产出=每目标 provider 或具名中立适配或（无消费者时）退役裁定；非目标：不强造无人用目标 | M2-DEN | 每目标独立金样/行为 oracle 或归属裁定收据 | P/语言 |
| **M2-UI** | AutoUI vm 模式/vue-vm 档/gallery 切 ACE（vm_bridge/handler_codegen/back_proxy/dynamic_ui/aura Bytecode 臂重定义）；非目标：渲染器重写 | M2-HOST+M2-EXEC | UI 状态编辑/事件/外库回调/退出全链原例（29 示例应用回归） | 跨仓/auto-ui（owner 仓合同） |
| **M2-MAN** | Man 命令面：config 求值（或接 D-08 静态臂裁定）、裸脚本/run/service vm、test、模板（AutoInterpreter→ACE 或裁定）、REPL/daemon/debug/mcp 入口切换；非目标：项目管理规则 | M2-EXEC/SESSION/TEST/GEN | 全命令原行为对账（pac.at 双过语义保持） | 跨仓/auto-man |
| **M2-WEB** | Web 宿主切 ACE 会话（serve/dispatch/parked/blocking 档位/serve_owned 嵌入面 UK 处置） | M2-HOST | http_e2e 73 用例+请求生命周期/取消/SSE 原例 | 跨仓/auto-web |
| **M2-GEN** | 模板求值归属：AutoInterpreter 替代（ACE）或中立裁定；含 auto-gen 复活/归档决策（F-06 用户裁定项） | M2-EXEC | 模板输出原例对账 | P+跨仓/auto-man(+用户) |
| **M2-OS** | OS host/desktop、apps 子模块构建轨、auto-term/at-gen/os-config-back、jade-garden、musk vm 轨迁移；旧 ui 面可编译性先行修复裁定 | M2-HOST/UI/MAN | 各应用真实启动/交互/健康检查原例 | 跨仓/auto-os(+auto-down/musk) |
| **N-CLOSURE** |（已立项为 [924 NC](../924-native-closures-source-and-resource-closure.md)，2026-11-03 drafting——本标签退役，R02/R05 归其回填） | 923 | 924 合同自带 | P(ace workspace)/语言 |
| **N-RESOURCE** |（已立项为 [923 NF](../923-native-allocation-fault-and-place-conformance.md)，2026-11-03 drafting——本标签退役，R03+R02 部分归其） | 915 交付 | 923 合同自带 | P(ace workspace)/语言 |
| **CA-B/C/D/E** | C 线后继（record/text → 集合/变体 → 有限 public ABI → 四端/签章），沿 921 接缝各保资源/ABI 门 | CA-A merge | 每域冻结语料真 MSVC 编译运行独立 reference | P(ace workspace)/语言 |
| **M2-RETIRE-1..n** | 退役序执行（§5）；非目标：不提前于接管完成 | 各域包收据 | REP-10 判据（依赖图/dispatch/构建单元/发行 0+负控） | P/语言 |
| **M2-REL** | 发行/安装切换：man-install/web-install 载荷、frozen 副本 ×2 清理、auto-lsp、launch 脚本、stdlib 解析序、独立回滚物 | 域包+RETIRE 大部 | REP-08/11 安装收据（clean/仓外/重定位/升级/回滚） | 跨仓/各 owner |
| **M2-TEST-MIG** | UI 110 模块、Man/Web/book/runners 测试义务迁移 successor ledger+新测试接管 | 各域包 | 新测试在独立安装物上通过+旧义务映射读回 | P+跨仓 |

依赖图要点：M2-DEN 与 M2-EXEC 可即启（EXEC 依赖的 AV 能力已 IV）；M2-HOST 是 UI/WEB/OS 的串行根；A2X-R/T1 与 HOST 并行；RETIRE/REL 是汇尾。921（CA-A）与 922 不互阻。

## 5. 退役顺序表（V-04；全部"G"类逐项入序）

1. **接管先行的前提**（每序步的 gate）：对应域包 oracle 绿。
2. 顺序：① facade 分派点真切换（A-09）→ ② 产品入口逐个切（A 表/域包）→ ③ 测试/注册迁移（G-5，successor ledger 先立）→ ④ 旧 trans 栈无消费者目标裁定后删（G-2 分批：先 ts（T1 后）→ rust/c（接线后）→ 其余按 T2 裁定）→ ⑤ vm 内部直连面随 HOST/UI 收口删（G-1/G-7 分界：renderer/natives 保中立）→ ⑥ 构建单元/feature/dispatch 清零（含 vendored 副本 ×2、launch 脚本、a2c-stdlib CWD、frozen SDK）→ ⑦ 安装/发行清零+独立回滚物发布（G-6）→ ⑧ 负控（故意接回全红）。
3. compat 规则：`AutoConfig`/facade 名可保但只转发 ACE（G-3）；`a2r_std`、值/诊断/词法工具中立保留（G-4，boundaries 断言已有先例 a2x spec V-13）。

## 6. 关键路径与容量/排期（T-05 后半/V-06；D-03）

- **串行关键路径**：M2-DEN ∥ M2-EXEC → M2-HOST → {M2-UI, M2-WEB, M2-MAN}（UI 最重：vm_bridge 127 引用+natives 30+）→ M2-OS → M2-RETIRE/REL 汇尾。M2-A2X-R/T1 与 HOST 并行；M2-SESSION/TEST/DEBUG/GEN 挂 EXEC 后；CA/N 线独立并行。
- **单写窗约束**（§20.3）：H 面（test_cases/mod.rs、ac_av/ac_mn/a2r/ac_protocol、auto-test-case、fixed 组、workspace lock/manifest、binding/source-manifest）与 921 在途重叠——M2-TEST/EXEC 消费 H 时按 921 合并后的漂移重验排窗。
- **容量依据（实测锚点）**：近两周单格/单目标包（913/914/916/917/919/920/921-部分）均为单 owner 会话 1-3 天级；域级包参照 863（console：work 5 天+2 轮 review）、866（resident）；M2-UI 体量（vm_bridge 单文件 ~4600 行+natives 30+）**超过任一已交付单包**。
- **估量结论**：M2 全量 ≈ 20+ 有界包（§4），关键路径至少 4 层串行（EXEC→HOST→UI→RETIRE/REL）。以当前单 owner 串行速率，**乐观区间 ≈ 6-10 周、保守 ≈ 3-5 个月**（假设无大返工、跨仓 owner 并行到位）；**"晚一周"没有数据支持**——除非显著扩 owner 并行（UI/Man/Web/OS 四仓同时开工且 HOST 提前交付）。此为工程估量（非承诺），未知项：跨仓 owner 可用性、HOST 设计返工风险、R05/TS provider 难度。排期决策交用户（合同 §4"如实提交缺口与日历调整"）。
- 历史锚点不外推处：不从前 12 月底窗口推日期（合同 §4 明令）。

## 7. 具名 unknown / 待核清单（V-07；每项 owner+下一步）

| UK | 内容 | owner/下一步 |
|---|---|---|
| UK-01 | rust_ui.rs（~7800 行）是否在列出的 a2r 调用点外直调 trans::rust | M2-A2X-R 开工时逐行核 |
| UK-02 | serve_owned/listen_borrowed 公开嵌入面的仓外 SDK 消费者 | M2-WEB 发布前 survey（安装收据/文档面） |
| UK-03 | 旧 `auto_lang::ui` 面消费者（auto-term/at-gen/auto-edit/musk 生成 back）对新 main 的可编译性（facade 已移除该面，PLAN-900） | M2-OS 首任务：逐仓 cargo check 基线+修复裁定 |
| UK-04 | auto-os apps 的 rust-workspace 存于兄弟顶层 clone（auto-charts 等），未逐一审计 | M2-OS/M2-DEN 刷新时补 |
| UK-05 | corpus a2vue/a2ark/a2j 本仓无 Rust 消费者的真实消费方（疑外部脚本/历史） | M2-A2X-T2 裁定时核 |
| UK-06 | frozen SDK（.sdk-source vs sdk/frozen）钉定修订差异；ag-examples 引擎分支是否含旧引擎 | M2-RETIRE 副本清理时核 |
| UK-07 | auto-musk rust-workspace 生成物 producer 身份（钉 auto-lang path，疑旧生成器） | M2-OS |
| UK-08 | `experimental/ace` worker 二进制运行期定位策略（ac.rs:38）产品化要求 | M2-HOST/M2-REL |
| UK-09 | 919 merge 在途对 H 面/锁文件的漂移 | 按 919 merge 收据重验本报告受影响行（M2-DEN 规程） |
| UK-10 | auto-gen 复活 vs 归档（F-06，涉及用户工具意向） | **用户决策**（§10 D-04 边界内非本包可定） |

## 8. 自查（V-01..V-07 对账）

- **V-01 分母双向**：A-G 七类 60+ 行，每行有 file:line 入口与 consumer；类别覆盖 CLI/SDK/嵌入/REPL/服务/UI 全模式/Man 全命令/Web/Gen/OS/发行。类别删除对照：去掉任一模式（如 C-01 vm 模式）将使 REP-01/06 门无 oracle 可依——结构对账成立。
- **V-02 分类分账**：IV/IW/ES/CM/UK 五类均有实例（§2.2）；legacy 差异行（binding avm 列 16+）保历史分账未冲销；非祖先证据（auto-gen）标 ES 不冒支持。
- **V-03 门映射**：REP-01..11 + V06-ACE-01..05 全部有 owner/oracle/后继/前置（§3），无空 oracle。
- **V-04 退役闭环**：G-1..G-7 全分类+顺序+compat/中立规则；frozen 副本、vendored server、launch 脚本等"包内"项均入 G-6；旧执行循环不得标"中立"（SD-02 判据）。
- **V-05 native/C/console 分账**：三格边界+证据（collections_driver.rs:46/109）；C 后继沿 921；console 45 格口径=15 行×3 轴（§2.5），无自动 NA。
- **V-06 排期**：串并行/依赖/容量/锚点/未知分列（§6）；无编造精确日数；"晚一周"结论=无数据支持（显式）。
- **V-07 来源身份**：全报告 file:line + repo@SHA（§0）；跨仓 WIP 具名未消费；未知项 §7 全具名 owner。

## 9. 与现行文档的关系

- 本报告是审计快照，**不是**第二测试 registry/规则引擎；M1 binding/applicability 未重写（§5.4 边界）。
- 共同 ACE 战略/交接文档（docs/design/strategy/ace-implementation-handoff.md）在本包交付短窗内同步登记本报告入口（全文件短窗规则）；调和时并入该文件主检出未提交的各会话交接块，未丢弃任何内容。
- 后续正式 SHA（919/921/923/924 合并等）到来时按 M2-DEN 规程只刷新受影响行，不复制成永恒状态源。**首次刷新已于 merge 时执行**：native 三格归属更新为 923/924（§2.3/§4）。
