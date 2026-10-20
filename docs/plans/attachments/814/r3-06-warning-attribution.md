# R3-06 迁移告警全量归因（PLAN-814 R3-03b 批次）

## 采集方法

- 桶：`cargo check -p auto-lang-core -p auto-parser-api -p auto-ui-dialect
  -p auto-ui-compiler -p auto-lang`（stderr 全量落盘）。
- 对照：受审基线 `d905e796c`（R3 修复批 2）与本批次工作树各采集一次；
  告警以（文本, 文件路径）为键归一（行号随编辑平移不计入）。
- 结果：基线 296 == 迁后 296，**新增 0 / 消失 0**——R3-03b 本体（函数表
  端口/trait 迁移/86 测试随迁/清单边删除）零新增告警。

## 迁移残留告警清偿（T-02/T-03 引入、基线已存在，本批顺手归位）

基线的 unexpected_cfgs 族（引用旧包 feature 名的恒假门）逐点归因与处置：

| 位置 | 归因 | 处置 |
|---|---|---|
| `auto-ui-compiler/src/ui_gen/api.rs:407/475`（`feature = "ui"` 双臂） | T-03 迁移时 `ui` feature 未随代码声明——**实臂（pac.at ui_config 装载）自 T-03 起静默走 None 桩**，行为漂移而非纯告警 | compiler 声明 `ui = []`；旧 auto-lang `ui = ["auto-ui-compiler/ui"]` 转发恢复恒开语义（default 链 ui-iced→ui） |
| `auto-ui-compiler/src/ui_gen/{rust.rs:16698, vue.rs:34452}`（`all(test, feature = "ui")` 测试门） | 同上——**plan571 等生成器测试自 T-03 起从 tu 档静默丢失**（R3-05 同族的隐性选择面缺口） | 同上转发；tu 档形态复测 compiler 侧 plan571 恢复运行（本批实录 3+1 过） |
| `auto-ui-compiler/src/style/mod.rs:160-168`（`ui-iced` 重复门 + `ui-headless`） | headless 适配器迁移时 feature 未声明（恒假死模块）+ 复制粘贴双门 | 删多余 `ui-iced` 门；compiler 声明 `ui-headless = []`；旧包 `ui-headless` 转发 |
| `auto-lang-core/src/vm/ffi/term_engine.rs:433`（`all(test, feature = "ui")` 测试面） | T-02 迁移死代码——SDK 无 `ui` feature 且 `crate::ui::terminal_provider` 已不存在（声明即编译红，恒不编译） | 删除；真测试面=上层 `auto-lang ui::terminal_provider::app_window_px_for_test`（含同断言，现存） |

清偿后 unexpected_cfgs = **0**（五 crate check 全量）。

## 归因为预存、不属本 Plan 迁移面的告警（不动）

- `widget.rs:567 events` 未用（auto-ui-dialect 既有）；
- SDK/旧包 lib.rs 的 aavm `AUTO_LIB_FILES*`/`aavm2_lib_source` 未用族、
  未用 import/doc 注释族等——基线同在（296 集合内），与 814 迁移无因果。

## 附注

- `cargo fmt`：本批触面文件全部过 fmt（见提交）；全仓预存格式漂移不在
  本批范围。
- 验证命令（本批实录）：`cargo check -p ... 五包` ×2（基线/迁后差集）、
  `cargo check -p auto-ui-compiler --features ui`（实臂编译）、
  `cargo test -p auto-ui-compiler --lib --features ui plan571`（4 过）、
  `cargo test -p auto-ui-compiler -p auto-lang --lib --features
  auto-lang/ui-iced plan571_button_preset`（tu 形态 compiler 侧 3 过）。

## R4 修订：基面回溯原 master 203e2ce91（口径修正）

R4 判定：上版以中途基线 d905e796c（296==296）只能证明"R3 修复批零新增"，
不能证明"814 全迁移相对原 master 零新增"。本节补齐：临时 worktree@
203e2ce91 上 `cargo check -p auto-lang`（409 warning 行/279 去重）vs 当前
五 crate check（421/289），按（文本, 迁移路径归一）对照。

结论：绝大多数"差集"是同一警告随代码搬家（`crates/auto-lang/src/X` →
`crates/auto-lang-core/src/X` / `auto-ui-compiler/src/X`），真实净新增
约 14 条，逐条归因如下；另有约 4 条净消失（迁移顺带解决，如
`static WARNED_API_NOOP`、`started_at`/`key` 未读等）。

| 净新增告警 | 归因 | 处置 |
|---|---|---|
| SDK lib.rs `AUTO_LIB_FILES`/`AUTO_LIB_FILES_V2`/`aavm2_lib_source` never used（3 条，另有旧包 `AUTO_LIB_FILES` 1 条转死） | T-02 大迁移的 aavm 常量双份（SDK 副本无调用方；上层 aavm 测试自足后上层副本转死） | aavm 族拟退役（AGENTS §AAVM）；B 阶段单源化时一并清偿，A 不动 |
| 旧包 lib.rs 未用 import 族（`AutoError`/`ast_parse_ext::*`/`lexer::*`/`Array,Node,Value`/`AtomicBool,AtomicUsize,Ordering`，约 5 条） | T-02 再导出门面拆分残留（门面 import 在新结构下无消费方） | 同上归 B 清偿；非 R3-03b/R4 批新增 |
| SDK lib.rs `std::path::PathBuf` 未用 import + unused doc comment（2 条） | T-02 迁移残留 | 归 B 清偿 |
| `field prev is never read`（vm/ffi/stdlib.rs） | T-02 搬家后可见性变化 | 归 B |
| http_server.rs 私有性三连（`ParkStage`/`DispatchCtx`/`ApiBody` more private） | 814 迁移期可见性漂移（vs d905e796c 已在，非 R3-03b/R4 批新增） | 归 B |

**边界声明**：本表不声称"814 全迁移零新增告警"——净新增 ≈14 条全部为
T-02/T-03 迁移残留且已在册（aavm 退役/B 清偿域），R3-03b 与 R4 修复批
相对其各自受审基线零新增（前节双基线差集 0）。R4 要求整理的 fmt 项
（ui_cache.rs:371、auto-macros tests 导入序、macros src/lib.rs）已全部
rustfmt 过（见 R4 修复批提交）。
