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
