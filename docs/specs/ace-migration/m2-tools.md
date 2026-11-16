# ace-migration: m2-tools — 调试LSP测试与Playground接管

> Status: **A期PROOF切片已交付**（PLAN-971 B-PROOF-I/V/R，2026-11-16 phase fold；
> lang `e1a600ebe`、auto-ui `2f70bbf7`）。B期全工具迁移（T-02/03/08/04：
> `#[test]` 发现迁 producer、断点/step/栈/locals、LSP 分析后端换 ACE、Playground
> 接管、acworker 线扩展）**未交付**——前置 M2A-08（PLAN-985 A批）。本文件只记录
> **已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。

## A期已验接口（显式ACE多文件工程往返，协议能力）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 中立项目契约 | `auto_lang_core::ace_entry` 项目面：`AceModuleSpec{package,version,id,source}`、`AceProjectRequest{modules,entry_module,entry_def_id,tests,capabilities,wall,cancel}`、`run_ace_project`/`run_ace_project_named`、`install_ace_project_executor`（单属主、同指针幂等、`shutdown_ace_project_executor` 释放）、`AceProjectReport{engine,outcome,tests,exit_code,diag_count,wall_ms}`；未注册=具名 `ace.host/project-executor-unregistered`，未知与全部旧引擎名=`ace.engine/unknown` 零dispatch（永不切旧VM）；`PACKAGES_CAPABILITY`/`packages_capabilities()` 便捷面 | `crates/auto-lang-core/src/ace_entry.rs` |
| 项目结果分态 | `AceProjectOutcome{Success{value,steps}, Diagnostic{module=原文件`pkg::mod`身份, stage, code, message, span=模块内原span}, Denied, ResourceDenied, Cancelled, HostFailure}`；协议级 `exit_code`=0 iff entry成功且全部请求测试passed——**测试失败永不退出0** | 同上 |
| 参考项目宿主 | `auto_lang_ace_host::project::ProjectHost`（`project_host()` 单例；`install()` 并装项目宿主，E0 `shutdown()` 语义不变、不触碰项目注册表）；消费960包面 `compile_package_product`→`PackageAvTarget::cold_with_entry`；`PackageDiagnostic{module,原span}`（binder已归位到模块内）直映中立契约 | `crates/auto-lang-ace-host/src/project.rs` |
| 世代预算/取消 | 每世代 `ExecLimits.wall` 封顶为请求**剩余**预算（F-971-R-01；单测 `p971_generation_wall_is_the_requests_remaining_budget` 钉死，非静默2s默认），请求取消旗标透传 `execute_program`（AV in-run 轮询）；相位界检查（编译前/每测试前/entry前）cancel/wall；每请求/每测试 scratch store 于 temp 建立、Drop 回收（停止回收可观察：运行后无 `ace-p971-project-*` 残留） | 同上 |
| 测试车道（A合同） | 请求显式携带测试名（A期发现留 legacy `test_runner::collect_tests`——AST walk `f.is_test`；B期T-02迁发现进producer）；`fn <name>() int`：完成且返回0=passed；非零返回/trap/链接失败=failed（带module+原因）；跨模块重名=ambiguous failed；未知名=not-found failed（module空）；每测试独立世代+独立store（隔离） | 同上＋fixture |
| LSP 真 client/CLI 面 | `auto-lsp` bin（stdio JSON-RPC）：didOpen坏源→`publishDiagnostics` 定位**该文档**（单行源 line0、字符界内）；全量 didChange 修复→诊断清空（150ms debounce）；未知方法=JSON-RPC error；shutdown(params=null)→null result，exit→进程码0；非LSP进程（错binary）=客户端具名失败不挂起。**分析后端仍 legacy auto-lang-core**（T-08/B 迁移，本fixture锁协议行为） | `auto-ui/crates/auto-lsp/tests/lsp_cli_roundtrip_971.rs` |
| 永久验证 | lang：`auto-lang-ace-host/tests/p971_project_proof.rs`（真实 app+math 跨模块 `eval(2,3,4)`=2+3*4=14、math坏源诊断定位 `math::core` 原span含 `nosuch`、修复后消失、bad_check/trap_check/missing_check→exit_code=1、cancel→Cancelled+无scratch残留、1ms墙spin拒绝、旧引擎名4种具名拒、合法近邻复跑14）＋core单测（registry/engine纪律+世代墙接线） | 固定组 `cargo t ace-facade`（28例）；`cargo test -p auto-lsp`（全包） |

## A期边界（现值开放项）

- 仅协议能力：断点/step/栈/locals、运行内步级取消粒度、`#[test]` 发现迁ACE、LSP分析后端换ACE、Playground接管、多文件工程经 acworker/1 线——全部归B期（T-02/03/08；T-01冻结记录在 PLAN-971 §5）。
- 编译域=960包面 TextRecord 裁剪：`while` 不在面内（条件循环=`for <cond> { }`）；`let` 局部不可变（可变局部=`var`）；入口=`pkg::mod` 模块键＋`d_<sym>` def，entry模块 `main` 保持符号 `main`。
- run期 trap 的 span 为 AV `SpanInfo` 映射（E0 同映，stage=`run`），module 身份为空——**源级**诊断才携带 module/原span；B期 source-map 义务不因A切片扩张。
- 端到端墙判别受冻结步预算（1M instr）与编译时长限制：世代内墙**接线**由单测判别；端到端负控（1ms墙spin）证明拒绝不挂起，不为修复提供独立判别（如实记录）。
- `ac_protocol.rs`（acworker/1 镜像）无漂移不改；项目面不经 case-worker 进程（一致性adapter面非产品入口）。
- 平台：Windows x64（桥 crate Windows-only）；UI 同。
