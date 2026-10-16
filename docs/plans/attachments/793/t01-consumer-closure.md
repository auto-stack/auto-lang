# PLAN-793 T-01 收据：消费闭包冻结 + V02 诊断投影旧臂 golden

生成环境：worktree `D:/autostack/.wt/lang-793/auto-lang`（branch `codex/plan-793-dev`），
基面 `579f6dec0`（= inputs.json 冻结 commit `9c773601a` 之后的 master 推进；
error.rs/ast.rs/route/discovery.rs/Cargo.toml/.autoos/module-boundaries.toml 五文件
SHA256 与 inputs.json 指纹逐一核对一致，诊断面零漂移）。auto-down 兄弟组内
worktree detached @ `c39ac7f`（PLAN-726 T-04 基面钉 master）。

## 1. V02 golden 冻结

- 生成器：临时集成测试 `crates/auto-lang/tests/zz793_dump.rs`（已从树中清除，
  未提交），只走**旧** `auto_lang::error` 公开路径（即冻结面）。
- 38 例 / 12 族：syntax(3)、lexer 结构化(2)、lex_raw(1)、type(3，含
  Name=AutoStr 的 CJK 参数)、name(5，find_best_match 建议/无建议两态)、
  runtime(3，含无 span 的 OutOfMemory)、comptime(2)、warning(4)、
  from 基础转换(4：String/&str/io/LexError::Lexer)、attached(7：
  attach_syntax/attach_double 幂等/attach_msg 零 span/attach_lexer/
  with_source/with_source_lexer)、unicode(2：CJK 多字节 span 渲染 +
  CJK 类型参数裸错误)、multiple(3：裸聚合 related 码/嵌套 attach/
  单数 plural 空串)。
- 每例投影字段：display/code/severity/help/url/labels(text+offset+len)/
  related_codes/debug(含 NamedSource name)/rendered（固定
  GraphicalReportHandler：width=60 + `GraphicalTheme::unicode_nocolor()`，
  两侧同一构造，确定性对拍）。
- 冻结件：`docs/plans/attachments/793/v02-golden-frozen.json`
  （SHA256 `e97b9450031c73a3ca05280489e67b92b5236ab06f5fd7ff47a87dee2742a09d`）。
  重放侧（auto-diagnostics tests）期望一律取自该冻结件的字节副本
  （`crates/auto-diagnostics/tests/fixtures/v02-golden.json`），新实现不参与
  期望生成（V02 oracle 界）。
- 环境注记：本机 sccache 守护进程端口被占（os error 10013，换端口无效），
  T-01 转储以 `RUSTC_WRAPPER=` 旁路执行；不影响冻结内容。

## 2. 消费闭包冻结（迁出必须保持的面）

### 2.1 crate 外消费者（auto_lang::error 路径）

| 消费者 | 用法 |
|---|---|
| `crates/auto/src/main.rs:1` | `use auto_lang::error::AutoError`（+ WithSource 族/format_error/print_error 面） |
| `crates/auto-lsp/src/diagnostics.rs:1,78` | `use auto_lang::error::AutoError`；`auto_lang::error::Warning`（warning_to_diagnostic） |

auto-lang 是两者既有依赖，error.rs 薄重导出后路径不变，零改动。

### 2.2 crate 内消费者（crate::error:: 路径，~15 文件）

`lib.rs`（AutoError/AutoResult/StagedRunFailure）、`route/discovery.rs`、
`compile.rs`、`aura/validate.rs`、`infer/errors.rs`、`infer/unification.rs`、
`stdlib_assembly/plan.rs`、`trans/c.rs`、`src/tests/*`（error_tests、a2c、
storage 族、generic parse 族等）、`test_cases/mod.rs`。重导出后
`crate::error::X` 路径不变。无 `use crate::error;` 裸模块导入、无 lib 根
pub use error 项、无 crates/auto-lang/tests/ 集成测试引用 error 面。

### 2.3 所有者侧模块专有 From impl（不迁基础包，孤儿规则合法）

| impl | 位置 | 合法性 |
|---|---|---|
| `From<RouteDiscoveryError> for AutoError` | `route/discovery.rs:65` | RouteDiscoveryError 为 auto-lang 本地类型 ✓ |
| `From<UnificationError> for TypeError` | `infer/unification.rs:89` | UnificationError 为 auto-lang 本地类型 ✓ |

### 2.4 error.rs 内随迁的 From/聚合面

`From<String>`/`From<&str>`/`From<std::io::Error>`/`From<auto_lex::error::LexError>`
（789 映射：Lexer→AutoError::Lexer，Raw→Msg）随 AutoError 定义迁入
auto-diagnostics（AutoError 为本地类型，孤儿规则合法）。

## 3. Name 边核对（T01 核心问题）

error.rs 唯一 crate 内导入 `use crate::ast::Name` 仅服务
`TypeError::CannotModifyViewParam { param: Name }`（error.rs:9,702）。
`ast.rs:75` 声明 `pub type Name = AutoStr`，`auto_val::string` re-export
`ecow::EcoString as AutoStr`——**同一类型**。auto-diagnostics 直接使用
`auto_val::AutoStr` 即保持字段类型身份（构造侧 Name 即 AutoStr，零转换）。
§10 预设"TypeError 载荷非 Name 别名可表达"分支不触发。

## 4. 抽取后前端剩余 DAG（预登记，T-04/T-06 复核）

```
auto-lang::error ─→ auto-diagnostics（薄重导出门面）     [PLAN-793 本批消除 error→ast 边]
auto-diagnostics ─→ auto-lex（LexerError/LexError）+ auto-val（AutoStr）+ miette/thiserror
auto-lang::parser ─→ ast/types/infer/session/module_cache [残留，前端外置批次]
                  ─→ dialect/ui.rs（UiDialect）           [残留，同上]
ParserBridge → LexerState                                [789 冻结矩阵不变]
```

| 残留边 | owner |
|---|---|
| parser → ast/types/infer/session/module_cache/dialect | 前端外置批次（Design 37 A 波后续） |
| error 门面 → auto-diagnostics（合法向下边） | 常驻形态；后续按消费方迁移收窄 |

## 5. T-01 结论

诊断聚合/子类型/投影可整体在**不引入 AST/推断依赖**的条件下保持身份：
error.rs 对 crate 内类型的全部依赖 = `ast::Name`（=AutoStr 别名），
经 auto-val 直用同一类型后无环。§10 修订分支不触发。
