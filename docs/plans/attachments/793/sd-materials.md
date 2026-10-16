# PLAN-793 SD 素材草案（merge 沉淀用；work 不改 live spec/ledger）

> SD-01 → 新建 `docs/specs/auto-diagnostics/project.md`
> SD-02 → 修改 `docs/specs/auto-lang/frontend/overview.md`
> SD-03 → 窄同步修改 `docs/specs/auto-lex/project.md`（R1 F-793-R1-04 增补：
> 现行 auto-lex spec 的错误归属表述随 PLAN-793 已过时，须一并纠正，否则
> 落地后留下相互矛盾的 current-state 规范）
> 结构对齐 docs/specs/auto-lex/project.md（PLAN-789 先例）。
> **R1 修正**（F-793-R1-04）：门面=`crates/auto-lang/src/error.rs`，
> 实现=`crates/auto-diagnostics/src/error.rs`（原 §2 两处路径写反/混写）；
> 冻结公开面计数 18→**20**（6 函数+方法名 14：attach_source/find_best_match/
> format_error/pos_to_span/print_error/span_from + AutoError/AutoResult/
> ComptimeError/LexerError/LexerErrorWithSource/MietteError/MsgWithSource/
> NameError/Result/RuntimeError/SyntaxError/SyntaxErrorWithSource/TypeError/
> Warning）。

---

## SD-01 全文：docs/specs/auto-diagnostics/project.md（新建）

```markdown
# auto-diagnostics

> **Status**: active
> 路径：`crates/auto-diagnostics`  | 技术栈：Rust（依赖仅 auto-lex + auto-val + miette/thiserror）
>
> PLAN-793 交付：诊断基础 crate——`AutoError` 聚合器与语法/类型/名称/运行时/
> comptime/警告各族诊断、source-attach 包装、投影/建议助手的单一实现自
> auto-lang 逐字抽取（基面 579f6dec0），auto-lang 的 `error.rs` 成为薄门面。
> 本 spec 为该冻结增量的 canonical 沉淀。

## 模块与依赖

`crates/auto-diagnostics` 是 AutoLang 通用错误/警告诊断的唯一实现：
`src/error.rs`（`AutoError` 聚合 + `SyntaxError`/`TypeError`/`NameError`/
`RuntimeError`/`ComptimeError`/`Warning`、`SyntaxErrorWithSource`/
`LexerErrorWithSource`/`MsgWithSource`、`attach_source`/`print_error`/
`format_error`/`find_best_match`、`From<String>/&str/io::Error/LexError>`、
`AutoResult`、miette `MietteError/Result` re-export、auto-lex
`LexerError/pos_to_span/span_from` re-export）。normal 依赖闭包仅 auto-lex
（词法错误单一定义，PLAN-789）+ auto-val（`AutoStr`）+ miette/thiserror
诊断基建——无 auto-lang/auto-man/auto-vm/UI/AST/parser/推断反向边
（module-boundaries 域 `lang-core`）。外部消费门：
`test/fixtures/modularity/diagnostics-consumer/` 独立 manifest
（依赖 auto-diagnostics/auto-lex/miette）构建+测试 + `verify-deps.sh`
闭包断言即独立可消费证据。

auto-lang 侧兼容门面：`crates/auto-lang/src/error.rs` 为纯 re-export
（同一类型身份，跨 crate 互传；`auto_lang::error::*` 20 名冻结公开面，
见 docs/plans/attachments/793/t01-consumer-closure.md）。不保留第二份
诊断实现。模块专有错误转换留 owner 侧（孤儿规则合法）：
`From<RouteDiscoveryError> for AutoError`（route/discovery.rs）、
`From<UnificationError> for TypeError`（infer/unification.rs）。

## 诊断契约

- `TypeError::CannotModifyViewParam.param` 载荷类型为 `auto_val::AutoStr`
  （= auto-lang `ast::Name` 的同一底层类型 ecow::EcoString）——字段类型
  身份保持，构造侧零转换，error→AST 依赖边因此切断。
- 诊断投影逐字段冻结：code/severity/help/url/labels(text+offset+len)/
  related/Display/带源渲染以 PLAN-793 V02 冻结 golden（38 例/12 族，
  `docs/plans/attachments/793/v02-golden-frozen.json`，SHA256
  `e97b9450…2a09d`，旧实现生成）为对拍基线，由
  `crates/auto-diagnostics/tests/v02_golden_replay.rs` 永久重放
  （期望件 `tests/fixtures/v02-golden.json` 为冻结件字节副本，随库走）。
- `attach_source` 语义：Lexer→LexerWithSource、Syntax→SyntaxWithSource、
  Msg→MsgWithSource（零 span(0,0)）、MultipleErrors 逐子递归 attach
  （子错误升级为对应 WithSource 形态）；已 attach 形态重复 attach 幂等
  （_ => err 原样返回）。SourceSpan offset/len、NamedSource、错误排序
  与 `multiple_errors_display` 文案不变。
- 词法错误经 PLAN-789 单一定义：结构化五变体 re-export 同一
  `LexerError`（`AutoError::Lexer`），`LexError::Raw` → `AutoError::Msg`
  （无诊断 code），不复制词法错误。

## 边界与已知限制

- 本 crate 不承载 parser/AST/推断类型：多错误收集上限（parser 错误恢复
  的全局 error limit）留在 auto-lang `lib.rs`；route/discovery、
  infer/unification 等模块专有 From 留 owner 侧。
- 同一性合同：`auto_lang::error::X` ≡ `auto_diagnostics::error::X`——
  消费者不得把公开错误降级为 String（吞诊断 code）。
  `test/fixtures/modularity/diagnostics-interop-consumer/`（依赖
  auto-lang + auto-diagnostics）为永久互传/阶段保持门（含真实
  parse/type/name/route 管线错误阶段保持用例）。
```

## SD-02 变更点：docs/specs/auto-lang/frontend/overview.md

（R1 修正：门面是 `crates/auto-lang/src/error.rs`；`crates/auto-diagnostics/
src/error.rs` 是单一定义实现，二者不可混写。）

1. §依赖关系（行 19-26 附近）：`error.rs` re-export `LexerError` 段后补
   ——PLAN-793 起 `crates/auto-lang/src/error.rs` 为 auto-diagnostics
   薄门面（`AutoError` 聚合与各族诊断单一定义在
   `crates/auto-diagnostics/src/error.rs`，同一类型身份）；
   **删除"`error.rs` 依赖 `ast::Name`"表述**（TypeError::CannotModifyViewParam
   载荷现为底层 `AutoStr`，error→AST 边已切断）。
2. §源码清单（行 83 附近）：auto-lex error re-export 行改为两行——
   `crates/auto-lang/src/error.rs`（薄门面，20 名 re-export）+
   `crates/auto-diagnostics/src/error.rs`（单一定义，PLAN-793 诊断外置）。
3. §剩余依赖/边界（行 164 附近）：外置进度更新——词法（789）+ 通用诊断
   （793）已外置；明确剩余环：parser→ast/types/infer/session/
   module_cache/dialect/ui（前端外置批次 owner），不冒称整个前端已无环。

## SD-03 变更点：docs/specs/auto-lex/project.md（窄同步，R1 增补）

现行 spec 三处表述随 PLAN-793 已过时，窄改不动词法契约本身：

1. §模块与依赖 末段（auto-lang 侧兼容出口）："`From<LexError> for
   AutoError` 做等价投影……"——From 实现与 AutoError 定义随 PLAN-793
   迁至 auto-diagnostics（`crates/auto-diagnostics/src/error.rs`）；
   auto-lang `error.rs` 为其薄门面。改为注明归属 auto-diagnostics。
2. §词法行为与错误契约 第二条："本 crate 不是错误聚合器——多错误收集/
   上限/attach_source 归 auto-lang `error.rs`"——attach_source/
   多错误聚合（AutoError/MultipleErrors）现归 auto-diagnostics；多错误
   收集上限（全局 error limit）仍归 auto-lang `lib.rs`（parser 恢复策略）。
   §边界与已知限制 中"`error.rs` 的 `ast::Name` 为未迁残留"一条删除
   （PLAN-793 已切断该边，Name 载荷=AutoStr 同一类型）。
