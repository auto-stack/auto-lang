# auto-lex — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 789 | lexical-foundation-extraction（建 crate） | ✅（reviewed→archived） | archive/789-lexical-foundation-extraction.md | 本 crate 随 PLAN-789 建立：Token/TokenKind/Pos 与 Lexer/LexerState 单一实现自 auto-lang 逐字抽取（V02 34 例冻结 golden 对拍），auto-lang token/lexer 薄 re-export 出口，无 auto-lang/auto-man/UI 反向边 |
| 793 | diagnostics-foundation-extraction（spec 窄同步） | ✅（reviewed→archived） | archive/793-diagnostics-foundation-extraction.md | PLAN-793 诊断外置后本 spec 归属同步：`From<LexError> for AutoError` 与聚合/attach_source 归 auto-diagnostics（经 re-export 链身份不变），多错误收集上限仍归 auto-lang lib.rs；"error.rs 的 ast::Name 边"残留表述随边切断销行 |
| [841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md) | View/On 的 Core 共享分类保持，UI 领域文法经显式 Domain 组合，不按同名词法误删。 |
