# auto-lex

> **Status**: active
> 路径：`crates/auto-lex`  | 技术栈：Rust（依赖仅 auto-val + strum/miette/thiserror）
>
> PLAN-789 交付：词法基础 crate——`Token`/`TokenKind`/`Pos` 与
> `Lexer`/`LexerState` 的单一实现自 auto-lang 逐字抽取（基面 e4ac72a62），
> auto-lang 保留薄兼容出口与错误转换。本 spec 为该冻结增量的 canonical 沉淀。

## 模块与依赖

`crates/auto-lex` 是 AutoLang 词法层的唯一实现：`token.rs`（Token/TokenKind/
Pos、`keyword_kind`/`all_keywords`、Display）与 `lexer.rs`（`Lexer`/`LexerState`、
f-string 插值与 note、`capture_raw_block`、peek/buffer/checkpoint）。normal
依赖闭包仅 auto-val（AutoStr）与 strum/miette/thiserror 诊断基建——无
auto-lang/auto-man/auto-vm/UI/renderer 反向边（module-boundaries 域
`lang-core`，规则 `lang-core-no-monolith-no-man-no-tools`）。外部消费门：
`test/fixtures/modularity/lex-consumer/` 独立 manifest（唯一依赖 auto-lex）
构建+测试即独立可消费证据。

auto-lang 侧兼容出口：`crate::token`/`crate::lexer` 模块为纯 re-export
（同一类型身份，跨 crate 互传），`auto_lang::error::LexerError` 经
auto-diagnostics 链 re-export 本 crate 的 `error::LexerError`（PLAN-793 起
`From<LexError> for AutoError` 等价投影随 `AutoError` 定义迁至
auto-diagnostics：结构化五变体 → `AutoError::Lexer`，raw-block 字符串错误 →
`AutoError::Msg`，与抽取前旧臂一致）。不保留第二份词法实现。

## 词法行为与错误契约

- 结构化词法错误 `LexerError`（E0001 坏转义 / E0002 未闭合 char / E0003 空
  char / E0004 非法标识符起始 / E0005 未知字符）：诊断 code/label/help/span
  逐字保持旧定义；span 为 miette `SourceSpan`（offset/len 自 `Pos` 投影，
  `pos_to_span`/`span_from` 在本 crate）。
- 词法面全集 `LexError = Lexer(结构化) | Raw(String)`；`capture_raw_block`
  （widget `style { }` 原始块）的字符串错误落 `Raw`（旧臂 Msg 投影，无诊断
  code）。本 crate 不是错误聚合器——多错误聚合/attach_source（PLAN-793 起）
  归 auto-diagnostics，多错误收集上限（parser 错误恢复全局 error limit）仍归
  auto-lang `lib.rs`。
- 行为冻结基线：PLAN-789 V02 冻结 golden（34 例——Unicode/CRLF/注释三型/
  转义/f-string note/EOF/View/on-header/数字后缀/算子族/comptime token/
  raw-block，含 7 错误例；`docs/plans/attachments/789/v02-golden-frozen.json`，
  旧实现生成）由 `crates/auto-lex/tests/v02_golden_replay.rs` 永久重放；
  游标/checkpoint/buffer 语义由 `tests/v03_state_replay.rs` 钉住
  （Plan 395 `save_state`/`restore_state` 投机解析回滚的词法半边）。

## 边界与已知限制

- `on` 不是 `keyword_kind` 成员：它是 newstart 条件下的 header 关键字
  （`Lexer::header_keyword`），`all_keywords()` 不含 `on`（LSP 工具面契约，
  Plan 416 单一来源）。
- `View`/`On` 保留在共享词法层：前者还服务 Core 不可变借用与参数模式，
  后者还服务 Core on-events/task；`take`/`hold` 等同理。PLAN-841 起 UI
  专属 AST 归 auto-ui-dialect，词法层不复制 UI 领域分类。
- `Lexer` 在 auto-lang 内保持 crate 私有（`mod lexer` 不公开，抽取前同构）；
  auto-lang 的 `ParserBridge` checkpoint（`BridgeCheckpoint` 持 `LexerState`）
  字段不透明、恢复语义不变（PLAN-778 冻结矩阵）。
- 未迁残留（属前端外置后续批次，非本 crate 债）：parser→AST/TypeStore/
  InferenceContext/ModuleTracker/session 依赖、内建 UiDialect——见
  `docs/plans/attachments/789/t01-frontend-dag.md`（原"`error.rs` 的
  `ast::Name` 边"已由 PLAN-793 切断，载荷=底层 `AutoStr` 同一类型，
  见 [auto-diagnostics/project.md](../auto-diagnostics/project.md)）。

## PLAN-841 来源与验证

View/On 的 Core 共享分类保持，UI 领域文法经显式 Domain 组合，不按同名词法误删。 规范与验收来源：[PLAN-841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md)；复审及统一门证据见其归档记录。
