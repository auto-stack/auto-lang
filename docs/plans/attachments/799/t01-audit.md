# PLAN-799 T-01 有界决策工件

基面：master@`143aff0a258fdca8bda9a3b8068dc8a9cced623c`（AST 面与取材基线
`30d3e1d2a` 零差异——`git diff 30d3e1d2..143aff0a2 -- ast 面Cargo.toml` 仅根
Cargo.toml +12 行，输入指纹仍有效）。审计日期 2026-10-17，全部结论来自源码
实读（`crates/auto-lang/src/ast.rs` 1782 行 + `ast/` 32 文件 9148 行，总 10930 行）。

## 1. 三分清单（模型 / 解析适配 / 测试）

### 1a. 迁入 crates/auto-ast（纯模型，32 文件 + 模块根）

`ast.rs` 模块根（去 `mod parsers;` 声明、去 `markdown_tests` 测试模块）+ 以下
31 个子模块（`ast/parsers.rs` 除外）：

alias, atom_helpers, body, branch, call, comptime, cover, dep_, enums, ext,
for_, fstr, fun, grid, hold, if_, is, module_path, node, on, range, route,
spec, store, tag, try_, task, type_alias, types, ui, union, use_。

- 定义面：`Code`/`Stmt`/`Expr`/`Name`(=AutoStr)/`Type` 及 32 文件全部
  pub struct/enum；traits `ToAtom`/`AtomWriter`/`ToAtomStr`(含 blanket
  impl)/`ToNode`；`impl From<Key> for Expr`(types.rs:871)、
  `impl From<Call> for Node`(node.rs:57) 等内部 From 全部随类型内聚。
- serde 面：仅 `fun.rs` 手工 `impl Serialize for Fn`（serialize_str("fn")），
  无 `#[derive(Serialize/Deserialize)]`——auto-ast 只需 serde trait 面，不需
  derive feature。
- 测试归属：ast/ 各子模块 `#[cfg(test)]` 共 95 test（body 2/branch 1/call 8/
  comptime 4/ext 7/fun 3/hold 1/module_path 5/route 5/spec 7/task 22/ui 4/
  use_ 6 + 其余 24 文件 0），全部只 `use super::*`/`crate::ast::{Expr,Type}`/
  `auto_val::AutoStr` 构造——**零 Parser/ScopeManager 引用**，随模型迁移。

### 1b. 留 auto-lang 上层（解析便利层）

- `ast/parsers.rs`（102 行）→ 新 `src/ast_parse_ext.rs`（私有 `mod`，旧
  `mod parsers` 本就非 pub，无外部路径消费）：`ParserExt` 对
  `Code`/`Name`/`Expr`/`Is`/`Arrow`/`OnEvents` 六 impl（OnEvents 经
  `Parser::from(..).skip_check()` 保旧 skip_check 语义）+ 其 5 个测试。
- `ast.rs` `markdown_tests`（~420 行，test/ast/*.test.md 五文件 oracle）→
  同迁 `ast_parse_ext.rs` 测试模块（用 `crate::parser::Parser` +
  `crate::scope_manager::ScopeManager` + regex + miette）。
- `ParserExt` trait 本体留 `parser.rs:22`（旧导入路径 `crate::parser::ParserExt`
  不变）；六 impl 编入 auto-lang 后所有 `use crate::parser::ParserExt` 调用点
  零改动。

### 1c. auto-lang 薄门面

`src/ast.rs` → `pub use auto_ast::ast::*;`（单行 glob）。`pub mod ui/cover/fstr`
经 glob 重导出保持 `auto_lang::ast::ui::ExtImport` 等路径与类型身份；
`crate::ast::parsers` 无外部引用（grep 全仓仅 parsers.rs 自身测试）。

## 2. 真实 crate DAG（T01 冻结）

```
auto-lang ──→ auto-ast ──→ auto-val（AutoStr/AutoResult[Box]/Node/Op/Value/
              │             Arg/Type/Array/Shared；37+ 处限定引用）
              │       ──→ auto-lex（token::Pos，仅 call.rs:3 / task.rs:8）
              │       ──→ serde（仅 fun.rs 手工 Serialize impl）
              └─（既有）→ auto-val/auto-lex/auto-diagnostics/…
```

- ast 模型文件外部 crate 引用穷举：`auto_val::`（Arg 37/AutoResult 46/
  AutoStr 11/Node 5/Op 23/Type 39/Value 4/Array 1/shared 1）、
  `use serde::Serialize;` 1 处；regex/miette/strum/ecow 等仅 markdown_tests
  （迁上层）。**无 feature 门控**（仅 `#[cfg(test)]`）。
- 两个 `AutoResult` 是不同类型：`auto_val::AutoResult<T>=Result<T,Box<dyn
  Error>>`（模型 AtomWriter 用）vs `auto_diagnostics::error::AutoResult<T>=
  Result<T,AutoError>`（ParserExt 用）——各自绑定随迁，不混用。
- 反向边审计：`crate::ast` 之外 ast 模型仅引 `crate::token`（=auto-lex 薄
  re-export）、`crate::error`/`crate::parser`/`crate::scope_manager`（仅
  parsers.rs/markdown_tests，均迁上层）。**无具名环**。
- 孤儿规则：parser 侧 `impl ParserExt for {六类型}` = 本地 trait + 外部类型
  → 上层合法；全仓无 auto-lang 其他模块对 AST 类型的外部 trait impl
  （grep `impl From|PartialEq|Hash|Serialize|Display` 仅命中 ast 内部两处 +
  注释）。pub(crate)/私有子模块使用点：**零**。

## 3. V02 旧臂冻结

- 探针：`attachments/799/v02-probe.rs`（36 case：core-code/6 表达式变体/
  泛型+三声明/5 模式/task+on 事件/3 导入/10 UI 兼容节点/5 嵌套+comptime；
  每 case Display+ToAtom+ToNode(Debug) 三投影；Node 用 IndexMap 序稳定）。
- 旧臂运行：`cargo test -p auto-lang --test v02_projection_probe --
  --nocapture --test-threads=1`（8 test 全绿）@143aff0a2（worktree
  lang-799，改动前状态）。
- golden：`attachments/799/v02-golden.txt`，SHA256
  `1e74a8df6a9eb0f9ca68296223d974795052ba29a05b415fd858893e3f1ac964`（去捕获噪声首空行后）。
  新臂重放：同探针仅换 use 行（`auto_ast::ast`），diff 为零即 V02 过；
  重放测试落 ast-model-consumer fixture（自含 golden 副本）。

## 4. 共享窗口与窄接线（合流面）

- 根 Cargo.toml：仅加 `"crates/auto-ast"` member + exclude 行给
  `test/fixtures/modularity/ast-model-consumer`（789 lex-consumer 同式）。
- `.autoos/module-boundaries.toml`：`[domains.members]` 增
  `"auto-ast" = "lang-core"`（lang-core 禁向规则自动生效：禁 lang-monolith/
  man/工具/UI）。
- 792 已交付（archived）→ 799 后合入窄接线：`crates/auto-man/src/rust_ui.rs`
  `DELIVERY_CRATES: [&str; 10]` 增 `"auto-ast"`（10→11，793 加
  auto-diagnostics 同式）+ `docs/specs/repository-boundaries/
  generated-ui-distribution.md` 闭包描述同步（merge 期落，SD 素材已记）。
- 794 已交付（archived）→ 无需窄接线：scroll record/参数类型经
  `crate::ast` 门面同身份；V05 跑 scroll-provider-consumer 门禁代表回放。
- 800 reviewed 未合：本计划不依赖其新 API；800 继续消费旧
  `auto_lang::ast::ModulePath` 路径（门面保持同一性），无冲突面。

## 5. 剩余边（本计划不动，SD-02 记录）

parser.rs 内 UI 文法/TypeStore/InferenceContext、`crate::ui` 五目录对
`crate::ast` 的引用、UiAst 终态分离（weld-ast-ui-variants R8）——均不在
本轮范围；门面保持它们零改动。
