# PLAN-799 T-06 搬迁/净删/剩余边与 SD/来源收据

work 树：`D:/autostack/.wt/lang-799/auto-lang`，branch `plan-799-dev`，基面
master@`143aff0a258fdca8bda9a3b8068dc8a9cced623c`（=PLAN-800 R2 复审入库后
的 master tip；AST 面与取材基线 30d3e1d2 零差异）。

## 1. 搬迁与净删账

| 项 | 结果 |
|---|---|
| 模型搬迁 | `git mv` 33 文件（ast.rs + ast/ 32 文件）→ `crates/auto-ast/src/`；`ast/parsers.rs` → `crates/auto-lang/src/ast_parse_ext.rs`；全部 rename 可追溯（R/RM），零复制 |
| 文本适配 | 仅 4 类：`crate::token::Pos`→`auto_lex::token::Pos`（call/task）；4 处 doctest `auto_lang::ast`→`auto_ast::ast`；ast.rs 去 `mod parsers`+markdown_tests（迁上层）；模块根轻值同源 re-export（Op/AutoStr/Value/AutoNode/Shared/AutoResult/Pos，见 §3） |
| 旧模型删除 | 物理搬移即删除：auto-lang `src/ast/` 目录不复存在；`src/ast.rs` = 9 行薄门面（单行 glob re-export + 文档）；无第二份实现/转换副本（metadata：auto-lang→auto-ast normal 边，auto-ast 无任何 auto-lang 边） |
| 新增 | crates/auto-ast（Cargo.toml+lib.rs+模型+crate 内 V02 重放）；ast_parse_ext.rs（六 impl+5 测试+markdown_tests 426 行）；ast-model-consumer fixture（V01+V02）；777 domain 行；792 窄接线两处 |

## 2. 测试与门禁证据（work-owned，定向）

| 门 | 结果 |
|---|---|
| cargo check -p auto-ast / -p auto-lang / -p auto-man | 零错误；新代码零警告（auto-lang 411 条为仓库存量，与 master 同集） |
| cargo test -p auto-ast | 74 单测 + 1 V02 重放（36 case）+ 5 doctest 全绿 |
| ast_parse_ext 定向 | 11/11（六便利调用 + markdown test/ast oracle 01-06） |
| 代表消费者 | parser::tests 180 / infer:: 107 / vm::codegen 28 / trans::gdscript 63 / ui_gen::rust 83（+2 预存 ignored）全绿 |
| V01/V02 fixture | ast-model-consumer（manifest 仅 auto-ast）3 lib + 1 replay 全绿；门面路径 golden diff 36/36 逐字节一致 |
| ParserBridge 对照 | ui-dialect-consumer 26/26 |
| 777 audit | 域规则 `auto-ast=lang-core` 加入；strict 新边全合法（auto-ast→lex/val、lang→ast）；基线重采样须净树，提交后执行（见 §5） |
| 792 窄接线回放 | auto-man delivery 单测 9/9；**driver.py 25/25**（V01 fresh+real build 112s / V02 重定位 locked 110s+身份 / V03 a-j 全负控确定性红+复绿 / V04 dev 模式+模板同源）——11-crate 闭包含 auto-ast；收据 `review-r2/driver-receipt.json.txt`（worktree 侧 replay 副本随树清理丢失，r2 复审持久副本为准） |
| 794 兼容 | scroll-provider-consumer 5/5（八 native/record/参数类型经门面同身份） |
| rustfmt | 触面文件全 clean（examples/schema_loader 存量债与 master 同集，未触碰） |

**merge-owned pending（不在 work 声称）**：裸 `cargo t` + AST 触面档
`tv/tt/tu`、tf 批量回归到期判定、Spec/ledger 沉淀与归档、wt-guard 清理。
依据：2026-10-14 测试时序裁定 + 本计划 §6。

## 3. 表面增量披露（供 review 审；r1 修复后口径）

- auto-ast `ast` 模块根新增**同源 re-export**：`pub use auto_val::{AutoStr,
  Node as AutoNode, Op, Shared, Value}` + `pub use auto_lex::token::Pos`。
  理由：独立消费方（V01）manifest 仅依赖 auto-ast，构造 `Expr::Bina`（Op）、
  命名 Name 底型（AutoStr）、消费 `to_node()` 产物（Node/Value）必需。
  **F-799-R1-01 修复**：`auto_val::AutoResult` 首版曾公开再导出，与
  `auto_lang::error::AutoResult`（富 AutoError 版）是**不同 item**，令旧门面
  合法组合 `ast::* + error::*` 产生 E0659 歧义——已回退为模型内部私有；
  其余名经全 auto-lang glob 面（error/token/lexer/parser/vm）审计无不同
  item 撞名（Pos 与 token::Pos 同一 item）。永久回归：
  `tests/ast_facade_glob_compat.rs`（反例固化，编译期断言）+
  `tests/ast_cross_path_identity.rs`（parse-identity 探针 Cargo 化：双向
  互传/坏输入拒绝/双路径投影一致）。旧 `auto_lang::ast` 面零删减。
- 无行为改动：零算法触碰；golden 重放（旧实现生成）三路径（auto-ast 直连/
  门面/独立 fixture）逐字节一致证明投影不变。

## 4. 792/794 绑定 SHA 与并行合同执行

- 792：delivery `80293e2c1`（ff-only，archived by `c74e01200`）。后合入窄接线
  兑现：`rust_ui.rs DELIVERY_CRATES` 10→11 + `generated-ui-distribution.md`
  闭包行同步（"PLAN-799 后含 auto-ast"）；受影响 fresh/locked/重定位/保护
  反控已全量重放（§2），非复用旧收据。
- 794：delivery `5ddef1315`（archived by `c9c37968d`）。零接线需要：scroll
  record/参数经 `crate::ast` 门面同身份，consumer 5/5。
- 800（reviewed 未合）：无冲突面——其消费旧 `auto_lang::ast::ModulePath`
  路径由门面保持同一性；本计划未用其新 API。
- 共享写面：根 Cargo.toml（member+exclude 两处窄增）、auto-lang Cargo/lib.rs
  （依赖+模块声明两行）、module-boundaries.toml（域一行）、auto-man rust_ui.rs
  （闭包一行）——均为并行合同"小共享面"表的允许范围，无全文件重排。

## 5. 提交序列

1. `feat(frontend): PLAN-799 AST 模型 crate 抽取……`（实现+fixture+SD 素材+
   本收据，冻结代码提交）。
2. 净树上 `modularity-audit.py --refresh-baseline` 重采样 777 冻结基线 +
   `--mode strict` 复核（预期：仅 2 条 grandfathered，0 新违例）——chore 提交。

## 6. SD 素材清单

- SD-01（add）：`docs/specs/auto-ast/project.md`（新建，canonical 内容就绪）。
- SD-02（modify）：`docs/specs/auto-lang/frontend/overview.md` 现状节 PLAN-799
  段（门面/解析便利层归上层/未迁事实）已写入。
- ledger/INDEX/specs.json 投影 = merge-owned（auto-plan:merge 沉淀）。
- 附带（合流窄接线，非本计划 SD）：`generated-ui-distribution.md` 闭包行。
