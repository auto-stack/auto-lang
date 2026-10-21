# Plan 815 — T-04 R1 needs_fix 修复记录（work 侧）

对应审查：[review.md](review.md)（R1 六项，绑定 90aed7fc7）。
修复树：`D:/autostack/.wt/lang-815/auto-lang`（分支 `codex/plan-815-dev`），
基线 `90aed7fc7`，修复提交见 §尾。逐项：finding → 不变量 → 受影响面 → 修复与永久反例 → 当前结果。

## F-815-T04-R1-01 / P1：交付包缺必需 path 闭包

- **不变量**：真实 producer 生成的交付根必须自持全部 workspace path 依赖——fresh `cargo generate-lockfile`/`metadata` 零外部回读。
- **受影响面**：DELIVERY_CRATES 表 + 由它生成的交付根 members/来源清单（资源指纹机制按表驱动，自动跟随；DELIVERY_ROOT_INPUTS/投影 spec 未变——本轮确认其正确性与遗漏无关）。
- **修复**：表 23→**25**，补 `auto-project-model`（auto-tool-protocol 的非 optional path 依赖）与 `auto-toolchain-core`（auto-man-core 的非 optional path 依赖）。
- **永久反例**：`rust_ui::tests::delivery_crates_path_closure_is_complete`——静态遍历表内每 crate 的 `[dependencies]/[build-dependencies]` path 项，凡指向兄弟 crate 且**无投影剥除行**者必须在表内（投影 spec=PROJECTION_AUTO_LANG/CORE 的 strip_deps 豁免：auto-lang 的 auto-test-case/autodown-core 等闭包外 optional/仓外 path 由 vendored manifest 投影剥除，不经交付解析）。该测试在修复前红（首报 auto-test-case 类缺陷），修复后绿。
- **教训修正**：首轮修复曾把 optional 的 auto-test-case 一并入表（26 项），真实 driver V01 立即暴露其为错——该 crate 依赖 `vendor/autodown-test-parser`（仓外 vendored 输入，交付闭包不含），且投影 spec 本就剥除其行。终值 25 + 投影感知断言。
- **结果**：真实 792 driver 全程（`python test/fixtures/modularity/generated-ui-consumer/driver.py`）：generate（crates=27=25 源+app+…按 manifest 计数）→ V01 fresh resolve+真实 build → V02 重定位 locked build+metadata 身份 → V03 负控族 → V04 dev 同源，EXIT=0（收据 driver-receipt 由脚本自身维护）；首轮 26 项跑红与修正后全绿的两轮日志均在本仓 scratch 会话留存，证明正反控双向区分。

## F-815-T04-R1-02 / P1：auto-ui-compiler lib-test E0433

- **不变量**：迁入测试的 dev-deps 必须挂到新 owner（传递依赖不可导入）。
- **修复**：auto-ui-compiler `[dev-dependencies]` 补 `tempfile`（原 auto-lang dev-dep 随 store_deps_scan_tests 迁移所需）。
- **结果**：`store_deps::store_deps_scan_tests::extract_store_deps_ignores_comment_examples` PASS；同一包 lib-test 现可编译（此前 E0433 于任何测试执行前）。

## F-815-T04-R1-03 / P2：旧公开路径断裂

- **不变量**：迁移不得删除旧公开 API 路径；仓内调用点改名不构成兼容证明。
- **修复**：
  - `auto_man::vue`：`pub use auto_ui_tools::vue` 升为门面模块（`pub use auto_ui_tools::vue::*`）+ 转发 `vue_run_compat::{run_vue_project, run_vue_project_with_request, start_gallery_back_proxy}`；
  - `auto_man::rust_ui`：同构转发 `rust_ui_vm_compat::{run_vm_ui, run_vm_ui_with_request, start_vm_server}`；
  - `auto_lang` 根：补回 `pub use auto_lang_core::strip_line_block_comments`（T-04 主体期该行随家族块被删、替换锚点失效而静默丢失）+ 确认 `extract_store_deps_from_file/extract_store_imports_from_ast` 转发在位。
- **永久反例**：`crates/auto-man/tests/plan815_t04_compat.rs`（六入口+同模块生成主体支持面抽样，函数项引用=编译即 oracle）与 `crates/auto-lang/tests/plan815_t04_root_compat.rs`（根三函数）。修复前按审查 probes 复现断裂；修复后双 PASS。
- 全部转发零复制算法（`pub use` 纯转发）；运行主体仍按 M814-B 门归 T-05。

## F-815-T04-R1-04 / P2：资产搬迁漏消费接线

- **不变量**：唯一来源切换后，一切现行读取入口（脚本/锚定断言/围栏/锁文件）指向新源，旧 oracle 不删。
- **修复**（含全仓路径扫描 `auto-man/assets`/`packages/drawlist-renderer`，历史计划/设计文档叙述保留）：
  - `scripts/ui-layout-parity.mjs`：读取与头注 → `crates/auto-ui-tools/assets/wm/layout.ts`；
  - `auto-ui-compiler/src/ui_gen/{nav_contract,sidebar_contract}.rs`：锚定目录 join → `../auto-ui-tools/assets/shadcn-ui[/sidebar]` + 相关注释；
  - `auto-lang/tests/gallery_golden.rs:131` 来源围栏 bundled_root → 新址；`schema_drift.rs`/compiler `ui_gen/vue.rs` 注释路径同步；
  - `examples/remote/viewer/{package.json,pnpm-lock.yaml}`：importer/resolution/directory 三处 → `../../../crates/auto-ui-tools/drawlist-renderer`。
- **结果**：`node scripts/ui-layout-parity.mjs` 17/17 PASS（此前 ERR_MODULE_NOT_FOUND）；`nav_contract_matches_scaffold_assets`/`sidebar_contract_matches_scaffold_assets` PASS（nextest-full 档，日常档 default-filter 排除 ui_gen:: 与本轮无关）；`pnpm install --frozen-lockfile --offline`（viewer）EXIT=0（锁一致性）。
- **归因注记**：`gallery_vue_golden` 内容金样仍红——经组内 detached 探针（`7ffa2d88d`）复跑同红，且 814 树（无 T-04 改动）同红：**基线预存**（语料/基线追随类），非本轮引入；金样再基线属语料 owner 动作，不在 T-04 修复范围，登记待批量回归归因。

## F-815-T04-R1-05 / P2：fmt 改写 plan734 金样

- **不变量**：金样文件字节=生成器真实输出；格式化工具不得触金样。
- **修复**：金样自编译位迁至 `crates/auto-man/tests/fixtures/plan734_commands_fixture.rs.txt`（非 .rs→rustfmt/测试编译永不触及；字节自 `7ffa2d88d` 原文恢复），`auto-api-gen` 断言读取路径同步。
- **结果**：`api_gen::tests::test_plan734_tauri_fixture_matches_generator` PASS（生成器输出与原文逐字一致）；附带消解该文件作为幽灵测试目标的 5 项预存编译错（auto-man --tests 现零错误）。

## F-815-T04-R1-06 / P3：fmt 残留

- **修复**：`auto-lang/src/lib.rs` store_deps 删除块后多余空行清除；另回退本轮 fmt 波及的 `crates/auto-lang/examples/*` 6 文件无关格式化噪声（非本计划改动面）。变更 Rust 文件 rustfmt --check 通过（lib.rs 因 cfg-gated 模块解析告警不影响 exit 0 判定）。

## 回归与边界

- 修复后定向面：auto-ui-tools lib nextest（含新增闭包测试）、auto-api-gen lib、tool_process/protocol/独立 fixture（M815-A 面未动）、plan811 consumer、缝测试、792 driver 全程。
- 未跑整档（2026-10-14 时序裁定；merge 冻结门统一）。
- T-05 外部门（M814-B）不变。
