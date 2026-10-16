# PLAN-793 T-05/T-06 收据：V01..V04 门禁证据 + 净搬迁账 + 剩余 DAG

Worktree `D:/autostack/.wt/lang-793/auto-lang`（branch `codex/plan-793-dev`，
基面 `579f6dec0`）。

> **R1 修订（2026-10-16）**：按 F-793-R1-01..04 修复——§3 重写（真实
> Type/Name 接线用例 + compile 用例重标注为 parser 阶段 + 7/7 门禁）、
> §4 阶段归属纠正、§5 计数 18→20；golden fixture 入库（.gitignore 否定
> 规则 + 提交内字节=冻结件 SHA256）；门面 fmt 清洁（六文件同范围
> rustfmt --check 通过）；SD 素材增 SD-03（auto-lex 窄同步）并修正
> 门面/实现路径。历史行数快照（lib 151→现约 240 行等）以修复提交为准。

## 1. V01 — 独立消费方闭包（AC-01）

- fixture：`test/fixtures/modularity/diagnostics-consumer`（独立 workspace；
  正常依赖=auto-diagnostics + auto-lex + miette，全部在诊断基础闭包内）。
- `cargo test --manifest-path …/diagnostics-consumer/Cargo.toml`：
  **5/5 绿**（各族构造投影/From/attach+渲染/建议与 span 助手；期望值取自
  V02 冻结 golden 对应案例，非新造；测试收 `#[cfg(test)]` 模块，正常 lib
  编译零警告——对齐 F-789-R1-03）。
- `bash …/diagnostics-consumer/verify-deps.sh`：**PASS** —— normal/build
  闭包 = {auto-diagnostics→auto-lex→auto-val, miette/thiserror/strum/ecow
  系}，**零 auto-lang/auto-man/auto-vm/auto-ui**（非借主库 rlib：独立
  manifest 解析）。环境注记：sccache 守护端口故障（os error 10013），本批
  验证以 `RUSTC_WRAPPER=` 旁路执行。

## 2. V02 — 诊断投影逐字段相等（AC-02）

- 冻结件 `attachments/793/v02-golden-frozen.json`（38 例/12 族，SHA256
  `e97b9450031c73a3ca05280489e67b92b5236ab06f5fd7ff47a87dee2742a09d`，
  旧实现公开面生成；crate 内 `tests/fixtures/v02-golden.json` 字节副本）。
- 重放 `crates/auto-diagnostics/tests/v02_golden_replay.rs`：**38/38 逐字段
  相等**（display/code/severity/help/url/labels(text+offset+len)/
  related_codes/debug/固定 60 宽无色渲染）。含 Unicode CJK span 渲染、
  CJK 类型参数、重复 attach 幂等、多错误聚合 related 码、无 span OOM。

## 3. V03 — 同一类型互传 + 真实管线错误（AC-02/03）

- fixture：`test/fixtures/modularity/diagnostics-interop-consumer`
  （依赖 auto-lang + auto-diagnostics + miette）。
- `cargo test --manifest-path …/diagnostics-interop-consumer/Cargo.toml`：
  **7/7 绿**（R1 修复轮后）：
  - `same_type_compiles_both_directions`：LangError↔DiagError 函数签名互传
    +跨路径 match 同一枚举+span 助手两路径同值（编译通过即同一性证明）。
  - `real_parse_error_flows_through_both_paths`：真实坏语法
    `parse_preserve_error("let x = 1 +")` → MultipleErrors/`auto_syntax_E0099`
    （探针冻结值），经 auto_diagnostics attach_source 后子错误升级
    SyntaxWithSource 仍居语法族，format 含源文件名与原文案。
  - `real_type_error_keeps_stage`（R1 F-02 新增）：**真实类型检查接线**——
    `check_field_type`（infer/mod.rs，经 parser Pair 臂在 `P { x: <expr> }`
    结构字段赋值时调用，UI 会话 `CompilerSession::ui()` 场景触发）：坏字段
    → 内层 `TypeError::FieldMismatch`/`auto_type_E0106`、span(57,1)、
    display "field type mismatch"；有效控制程序（`x: 1`）完整 parse；
    经 auto_diagnostics attach_source 后 Type 子错误不降级，code 保持。
  - `real_name_check_error_keeps_stage`（R1 F-02 新增）：**真实名称检查
    接线**——`check_symbol`（parser.rs）解析期符号解析：未定义变量 →
    内层 `NameError::UndefinedVariable`/`auto_name_E0201`，help 走
    NameError 手写 Diagnostic（建议助手同一实现）；有效控制程序 parse OK。
  - `real_compile_parser_recovery_error`（R1 F-02 重标注）：`compile::
    compile_once` 对 `let x: int = "s"` 返回的是 **parser 错误恢复**
    （内层 SyntaxWithSource/SyntaxError::Generic/`auto_syntax_E0007`，
    compile 已预 attach 源码），**不是类型推断**——原版误标为 infer 阶段
    证据（F-793-R1-02），本用例改钉住该入口的 parser 阶段恢复行为
    （坏程序单调返回、不吞错）。
  - `real_route_error_maps_to_msg`：owner 侧 `From<RouteDiscoveryError>`
    经门面可用，投影逐字一致。
  - `question_mark_propagation_is_monotonic`：String/io 经单一定义 From，
    `?` 单调传播，坏输入不成功。
- 断言修正记录（3 处初版断言语义错误，非实现问题）：attach 后子错误升级
  为 WithSource 是既有语义；Windows io 消息不含路径；format_error 输出
  ≠ Display 长度。
- R1 事实发现（供前端外置批次参考，非本批债）：`typeck/param_check.rs`
  （CannotModifyViewParam 的生产检查器 ParamChecker）**无生产调用方**
  （Plan 088 未接线）；infer 的 `ctx.errors` 向量不冲刷进 parse 返回值
  ——Type 族生产错误面=check_field_type（已接线，UI 会话）。

## 4. V04 — 生产消费 + 777 metadata（AC-03/04）

- Parser：真实 parse 错误（V03）+ crate 内 `cargo t error_tests` 4/4 +
  `cargo t error_spans` 8/8（生产 parser 错误恢复路径走新单一实现）。
- Type/Name 检查：真实 `check_field_type`（Type/E0106）与 `check_symbol`
  （Name/E0201）接线用例（V03，R1 修复轮）。
- compile：`compile_once` 的 parser 阶段恢复用例（V03，R1 重标注）；其
  错误面归 parser，不冒称 infer 证据。
- LSP：`cargo check -p auto-lsp` 过（diagnostics.rs 的
  AutoError/Warning 映射面编译不变）；深行为归 merge-owned 裸 `cargo t`。
- CLI：`cargo check -p auto` 过。
- 777 metadata（`scripts/modularity-audit.py` observe）：`auto-diagnostics`
  域=lang-core ✓；新声明边 3 条全部合法向下/同层：
  `auto-lang→auto-diagnostics`、`auto-diagnostics→auto-lex`、
  `auto-diagnostics→auto-val`；cargo metadata 交叉核对 46 边零差异。
- strict 门：唯一种红 `config_drift_since_baseline`（本批合法配置变更
  后基线重锚，见 §6）；无任何新增禁向边（grandfathered=2 均预存）。

## 5. 净搬迁账（AC-04）

| 项 | 数 |
|---|---|
| auto-lang/src/error.rs | 1231 行 → 26 行薄门面（diff +19/−1228） |
| crates/auto-diagnostics/src/error.rs | 1237 行（逐字迁移；与 HEAD 原版 diff **仅 3 处**：头注 +4 行、`use crate::ast::Name`→`use auto_val::AutoStr`、`param: Name`→`param: AutoStr`+2 行注） |
| crates/auto-diagnostics/src/lib.rs | 22 行（边界合同文档头） |
| crates/auto-diagnostics/tests/ | V02 重放 341 行 + 冻结 fixture 字节副本 |
| fixture diagnostics-consumer | 271 行（lib 210 + manifest 31 + verify-deps 30） |
| fixture diagnostics-interop-consumer | 179 行（lib 151 + manifest 28） |
| 根 Cargo.toml | members +4（含注）、exclude +9（两 fixture） |
| crates/auto-lang/Cargo.toml | +3（依赖声明） |
| .autoos/module-boundaries.toml | +5（lang-core 域登记） |
| error→ast 边 | **消除**（error.rs 不再 import crate::ast；冻结面 **20 名**全 re-export——R1 F-04 计数修正） |

## 6. 剩余前端 DAG（抽取后真实状态，AC-04 可回放）

```
auto-lang::parser ─→ auto-lex（PLAN-789）          [已断]
                  ├→ ast / types / infer / session / module_cache [残留：前端外置批次]
                  ├→ dialect/ui.rs（UiDialect）      [残留：同上]
                  └→ error（薄门面 → auto-diagnostics）[本批改为合法向下边]
auto-lang::error ─→ auto-diagnostics（re-export 门面，无 ast 边）[PLAN-793 本批]
auto-diagnostics ─→ auto-lex + auto-val + miette/thiserror      [本批建立]
ParserBridge → LexerState（auto-lex 类型）                      [789 冻结矩阵不变]
owner 侧 From：route/discovery.rs（AutoError）、infer/unification.rs（TypeError）[合法留存]
```

剩余边 owner：parser→ast/types/infer/session/module_cache/dialect 全部归
前端外置批次（Design 37 A 波后续）；error 门面→auto-diagnostics 为常驻
形态，后续按消费方迁移进度收窄（同 789 token/lexer 兼容出口策略）。

## 7. 基线重锚

`--refresh-baseline` 于本批全部树变更提交后在组树执行（对齐
PLAN-790/791 落地流程；789 c87c4e99c 同为显式重锚恢复 strict 绿）。
执行记录见 §9 复审记录 T-06 行。

## 8. 合流面（与同批计划的文件交集复核）

- 792（生成式 UI 独立构建）：共改**根 Cargo.toml**（exclude 段）——
  792 在途 worktree（ccf66f013 基面）若也改 exclude/members，合流时
  union 解冲突（789/790/791 三轮先例）。792 若已冻结其独立闭包清单，
  本批新增 crates/auto-diagnostics 不入其闭包（其 fixture 只依赖
  rust_ui/ui_gen 面），但若 792 先落地，需按 T-08 通知刷新闭包（互链）。
- 794/795（native/scroll、lock/target）：零文件交集。
- 787/788 在途实验线：触面（ac-grammar、autodown native）与 error 面
  无交集；若其分支基于旧 error.rs 内联改动，rebase 时以本批门面为准。
