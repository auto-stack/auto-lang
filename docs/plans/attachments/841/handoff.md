# PLAN-841 实施交接（T-09）

## 冻结范围

- Core AST/lex/parser-api/SDK 不含 UI 私有模型、文法实现或 Man 反向边；
  SceneDecl、View/On Token、Core on/task/msg 与 `.view` 借用保持。
- UI owned `UiCode/UiStmt`、visitor、版本 codec、跨文件 owner、校验和
  lowering 是生产唯一路径；compiler、四生成器、runtime/renderer、
  auto-ui-tools 与 Man 均消费该路径。
- 旧 Core `ast::ui`/`Stmt` UI 臂、Parser UI 返回门面、`UiGrammar` 函数表、
  inventory 默认装配、compiler 空 `assemble_ui_grammar` 桩及 Aura 死
  `import_stmts` 字段已删除。测试与 fixture 的旧残边同步清零。

## SD-01 至 SD-05

| delta | 冻结内容 |
|---|---|
| SD-01 | `auto-ast/project.md` 改为 Core/Scene 单源；`auto-lex/project.md` 冻结 View/On 共享 Token 与非 UI 同名语义。 |
| SD-02 | `auto-parser-api/project.md` 冻结中立 Domain 端口；`auto-ui-dialect/project.md` 与 `design/full-grammar.md` 冻结 owned 文法/组合/寿命。 |
| SD-03 | `auto-ui-compiler/project.md`、`auto-ui-runtime/project.md` 冻结 UiCode→lowering→四生成器/VM/Iced 生产路径与缓存身份。 |
| SD-04 | `language-sdk/project.md`、`auto-lang/frontend/overview.md`、`repository-boundaries/generated-ui-distribution.md` 冻结 Core 准入、外部来源和 fresh/locked/重定位合同。 |
| SD-05 | 新增 `ui-domain-model/project.md`，冻结分类、所有权、codec/schema、源位置、最后 owner 与支持边界。 |

验收命令、结果、失败归因与测试 ID 映射见
[v07-v08-closeout.md](v07-v08-closeout.md)，V-03 另见
[v03-closeout.md](v03-closeout.md)。R1 三项 finding 的修复矩阵、原始反例
重放与外部方言 48 项补跑见 [repair-r1-closeout.md](repair-r1-closeout.md)。
R2 的真实工程入口/完整 codec 输入修复、两轮失败诊断与覆盖修正见
[repair-r2-closeout.md](repair-r2-closeout.md)。R3 的冷态合法配方初始化
顺序、实际 class 与私有/缺失/残留负控见
[repair-r3-closeout.md](repair-r3-closeout.md)。

## 交 PLAN-842 的过渡适配器清单

| 适配器 | 当前作用与退役边界 |
|---|---|
| `auto_lang::{ast,parser,session,error}` re-export facade | 只做同类型路径兼容，不含第二实现；842 按 consumer 迁移完成度退役。 |
| `AuraViewBuilder::with_registry_and_imports` 最后一个 `_import_stmts` 形参 | 仅为源码形兼容；不保存/读取，外部组件由 UI-owned 注册表提供。842 删除形参并迁移调用方。 |
| Core `Dialect`/`ExternalDialect` 三段注册与 Override/Append | 仅服务非 UI 方言；UI 已改显式 Domain 组合。842 评估非 UI consumer 后退役，不得借此回流 UI 类型。 |
| `UiParseContext` 名称别名 | 实为中立 `DomainParseContext` 的 UI 使用面别名，不含 UI 字段；842 统一调用方命名。 |
| `ExternalUiDialect` 测试组合器 | 仅 fixture 演示外部 model 优先并委托 UiDialect；不是生产全局注册表。842 收编外部方言迁移说明后删除旧优先级测试映射。 |
| 旧 schema 缓存拒绝路径 | 不是兼容读取；旧数据具名拒绝并重建。842 只在确认旧缓存承诺期结束后调整提示，不放宽 domain/tag 校验。 |

这些适配器均不承载第二份 UI AST、文法或运行时算法；除此之外无已知
UI 旧实现残边。

## 验证与后续角色

T-01..T-09 的 work-owned AC-01..05 已有定向证据。独立 review 负责
AC-06，必须复核实际三仓提交、SD、反例、来源和健康检查；统一裸
`cargo t` + `tv/tt/tu` 等 merge-owned 门禁在 review pass 后执行，当前
不标绿。最终 792 收据为 `v08-792-receipt.json`；最终三仓 commit 与
merge-pending 状态登记主计划 §9。
