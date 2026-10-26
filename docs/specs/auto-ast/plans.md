# auto-ast — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 799 | frontend-ast-model-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-799 建立：AST 模型（Code/Stmt/Expr/Type/UI 兼容节点/ModulePath）与 ToNode/ToAtom/AtomWriter 序列化单一实现自 auto-lang 逐字抽取（33 文件 git-mv 零复制），依赖仅 auto-val/auto-lex(token::Pos)/serde（lang-core 域），ast-model-consumer 外部探针钉住闭包零 auto-lang/Parser/VM/UI；V02 冻结 golden 36 case×三投影（旧实现生成，SHA256 1e74a8df…）crate 内永久重放；auto-lang ast.rs 单行 glob 薄门面 + 解析便利层（ParserExt 六 impl+markdown oracle）归上层 ast_parse_ext；F-799-R1-01 教训：门面导出面以"与其他常用 glob 面无不同 item 撞名"为限（auto_val::AutoResult 与 error::AutoResult 不同 item，回私有，永久回归 tests/ast_facade_glob_compat.rs） |
| [841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md) | Core AST 仅保通用节点与 Godot Scene；UI 专属 Stmt/模型归 auto-ui-dialect，投影 golden 保持。 |
