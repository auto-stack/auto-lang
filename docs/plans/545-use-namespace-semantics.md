---
plan_id: PLAN-545
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: use-namespace-semantics
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/parser, auto-lang/compiler, auto-lang/vm, auto-lang/trans, auto-lang/module-system]
current_step: 0
total_steps: 12
---

# [PLAN-545] use-namespace-semantics

## 变更摘要

裸 `use db` 的语义从"全量平铺导入"收紧为"仅引入模块命名空间"（Rust 2018 风格：引入模块名本身，
符号访问走 `db.X` 限定）；全量导入必须显式 `use db: *` opt-in。同时为 `TypeStore::merge` 增加
符号冲突检测，消除"后 merge 者静默覆盖"的正确性隐患。随本次语义变更一并迁移仓内全部依赖旧语义的
代码（examples + 测试夹具）并更新语法文档。

## 目标

1. **语义对齐 Rust**：`use db` ≡ Rust `use a::db;`（引入命名空间）；`use db: *` ≡ Rust `use a::db::*;`（显式全量）。
2. **消除静默冲突**：merge/import_items 同名冲突（来自不同模块且定义不同）产生编译诊断，而非静默覆盖。
3. **迁移面清零**：仓内所有依赖旧"裸 use = 全量导入"语义的 .at 代码（examples/、crate 测试夹具）完成迁移。
4. **文档一致**：`docs/design/10-language-syntax.md` 等语法参考与新语义一致。

非目标（明确不做）：
- 不改 `use db: a, b` 具名导入语义（已经是按需）。
- 不实现 Rust 式 lazy glob fallback 解析（resolver 侧 fallback namespace）——本 plan 的 wildcard 仍是
  编译期物化 merge；lazy 化等有性能证据再立项（见 待澄清事项 #3）。
- 不改 `pac.` / `super.` 路径解析规则。

## 架构方案

现状是**两层语义分野**（本 plan 的核心动因）：

| 层 | 现状行为 | 本 plan 后 |
|---|---|---|
| CompileSession TypeStore 层（`compile.rs` resolve_uses / load_module_inner:1164,1481） | bare 与 wildcard 同义：`store.merge` 全量平铺；named：`import_items` | bare：只注册命名空间，**不 merge**；wildcard：merge（显式 opt-in）+ 冲突检测；named：不变 |
| VM codegen 层（Plan 317/339/347，`vm/codegen.rs` handle_auto_import ~:4820） | bare → `known_module_prefixes`（已具备命名空间模型，`db.func()` → 限定 reloc）；named → `import_scope` 别名 | 基本不变（该层已是目标模型） |
| VM native/widget 注册（`autovm_persistent.rs:323`、`lib.rs:3257,3439,3462`） | bare 与 wildcard 同义：短名 natives / widget 全注册 | 仅 wildcard 全注册；bare 只保留全限定名 |

关键事实（已核实）：
- **限定访问 `db.X` 已是主流写法且机器支持完善**：`examples/ui/015-notes/src/back/api.at` 以
  `db.all_notes()` / `db.find_note(id)` 调用；a2rs 例子以 `http.get()` / `json.parse()` / `time.now_sec()`
  调用。examples 中 42 处裸 use 有 8 处本就是限定风格——这 8 处零迁移。
- **迁移面已量化**：examples 剩余 34 处裸 use 依赖平铺导入，其中 quickstart 单 widget 模块
  （`use Banner` + `Banner {}`）约 29 处、playground 服务模块 5 处；crate 内 .at 测试夹具约 89 处字符串
  （粗估，含转义形态）。改法机械：单 widget → `use X: X`；服务模块 → 显式列符号或 `: *`。
- **传递性污染**（compile.rs:1432 递归 `resolve_uses`）：被导入模块自己的 use 也 merge 进同一个
  session store，导入方被动获得传递符号。新语义下应隔离进被导入模块自己的命名空间。
- **a2r 转译器**（trans/rust.rs:14558-14710）注释明说旧语义："use X (bare, no items) means
  'import all from this module' → generate use crate::X::*;"，需同步改为不发射 glob。

## 需求分析与背景调查

（从 docs/specs/overview.md 与相关 module spec 取材）

- Plan 167（module-system，已归档）落地了 `pub use`、`use module: *` 语法与 `TypeStore::merge`/
  `import_items`，但把 bare `use db` 与 wildcard 等同——本 plan 撤销这一等同，将其视为设计缺陷。
- 语言语法参考（docs/design/10-language-syntax.md:170-178）已把 `use db: load, save  // specific symbols`
  列为推荐形态；本 plan 使实现与文档推荐一致，并让"全量"成为显式决策。
- VM codegen 层自 Plan 317/339 起已按命名空间模型实现（`known_module_prefixes` / `import_scope`），
  说明命名空间模型是既定演进方向，本次是把 TypeStore 层拉齐。

## 详细设计

### D1. CompileSession 语义核心（compile.rs）

- `load_module_inner` 的两处合并分支（缓存命中 ~:1164 与新加载 ~:1481）：
  - `items` 非空 → `import_items`（不变）；
  - `is_wildcard` → `merge`（带冲突检测，见 D2）；
  - **bare（items 空 && !wildcard）→ 不 merge**，仅记录模块已加载（缓存/字节码编译照旧）。
- 新增 `CompileSession::module_stores: HashMap<AutoStr, TypeStore>`（模块名 → 该模块符号表），
  load 完成后总是存入，供限定查找消费（见 D3）。
- 传递性隔离：`parse_module_to_type_store` 解析被导入模块时，该模块自己的 use 解析结果进入
  `module_stores[该模块]`，不再递归 merge 进主 store。**若实现中发现解析上下文耦合过深，
  允许本子项降级为"维持现状传递 merge + 记录 KNOWN-DEBT"**，不阻塞主体语义。

### D2. 冲突检测（types.rs）

- `TypeStore::merge` / `import_items` 增加冲突报告：目标 store 已有同名符号且来源模块不同、
  定义不同 → 返回冲突列表；`load_module_inner` 将其升级为编译错误（错误信息含两个来源模块路径）。
- enum 的 `or_insert`（首胜）与其他表的 `insert`（末胜）统一到同一检测逻辑。
- 显式 named import 与已导入符号同名视为主动压制，不报错（与 Rust `use` 显式遮蔽一致）。

### D3. 限定查找通路（关键 spike）

- **第一步先写探针测试确认现状**：`db.all_notes()` 的类型解析与 codegen 今天的取数路径
  （flat store 裸名？qualified 名？codegen reloc `db.func` + exports？）。
- 若 typeck/parse 侧依赖 flat merge 才能解析 `db.X`：为 Parser/typeck 提供 `module_stores` 命名空间
  视图——限定路径 `db.X` 查 `module_stores[db]`，裸名只查本模块 + 已导入符号（named + wildcard）。
- codegen 层已就绪（`known_module_prefixes` → 限定 reloc），预期改动很小。

### D4. VM native / widget 注册对齐

- `autovm_persistent.rs:323`：`should_import` 的 `items.is_empty() → true` 分支收紧为仅 wildcard；
  bare 只注册全限定名 native（`auto.str.split`），不注册短名别名。
- `lib.rs:3257,3439,3462` widget 注册：`items.is_empty()` 从"视为通配"分支移除；bare 不再把子模块
  widget 灌进平铺 registry。

### D5. 转译器对齐

- a2r（trans/rust.rs:14688-14710）：bare local-module use 不再发 `use crate::X::*`；
  `is_multi_file_bare` 发射分支改为按新语义（bare → 仅 mod 声明已存在，无 glob）；
  wildcard → `use X::*`（不变）；named → `use X::{..}`（不变）。
- a2py（trans/python.rs）排查同形态逻辑并对齐（Python 侧对应 `from X import *` 语义边界）。

### D6. 诊断信息

- 裸名符号未找到时，若该符号存在于某已导入模块：报错附提示
  "`load` not found; module `db` is imported — write `db.load` or `use db: *`"。
- 冲突错误："`load` is ambiguous: defined in both `db` and `helpers`; disambiguate with `use db: load`"。

### D7. 代码迁移

- examples 34 处：单 widget `use Banner` → `use Banner: Banner`；playground 服务模块按实际用量
  显式列举（教学生例子顺势展示推荐风格）。
- crate 测试夹具 ~89 处逐文件迁移（parser.rs 10、trans/rust.rs 9、ast/route.rs 7、
  ownership_tests.rs 7、auto-man/api_gen.rs 6 等）。
- `docs/design/10-language-syntax.md` use 节改写：bare = 命名空间、`: *` = 全量、冲突规则、迁移示例。

## 测试设计

- 单元（crates/auto-lang/src，新增 use_semantics_tests 或就近）：
  1. bare `use db` 后：裸名访问报错（含 D6 提示文案）；`db.f()` 限定访问编译成功；
  2. `use db: *` 行为与旧 bare 等价（符号平铺可见）；
  3. `use db: f` 具名导入回归不变；
  4. 冲突检测：两模块同名 fn 被 `: *` 同时拉入 → 编译错误含双方模块名；同名同定义（如 re-export）不报；
  5. native 短名注册：bare `use auto.str` 后 `split(...)` 不可用、`str.split(...)` 可用；`: *` 后两者皆可；
  6. （若 D1 传递性落地）db.at 内 `use json` 不使 json 符号出现在导入方裸名空间。
- 端到端：`examples/ui/015-notes`（db.X 风格，应零改动通过）、quickstart/03-05（迁移后）
  双端验证：`auto run` 与 `auto run -r vm`（按 autoui-verifier 技能双端一致性）。
- 门禁：Category B/C 分级——阶段内 `cargo check -p auto-lang` + `cargo t <module>`；
  合入前 `cargo t`；docs_gen 触碰语法参考时跑 `cargo test -p auto-lang --test docs_gen`；
  最终 `cargo tf`。

## 验收标准

1. `use db` 后裸用其符号 → 编译错误且诊断含 `db.` / `use db: *` 提示；`db.X()` 正常。
2. `use db: *` 与旧 bare 语义逐项等价（类型/函数/spec/enum/泛型/别名）。
3. 同名冲突有编译错误，不再静默覆盖。
4. examples/ 与 crate 测试夹具中不再存在依赖旧语义的裸 use（迁移清零，`cargo t` 全绿）。
5. VM native 短名/全限定名注册与 wildcard 语义一致。
6. a2r 输出不再为 bare use 发射 glob glob；双端 UI 例子验证通过。
7. 语法文档与新语义一致。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. [ ] **Spike 探针**：写临时测试确认 `db.X()` 限定访问的 parse/typeck/codegen 取数路径
   （D3 前置）；产出结论写入本文件 待澄清事项。
2. [ ] compile.rs：`module_stores` 注册表 + bare 不 merge（两处分支）+ wildcard merge；
   `cargo t compile`。
3. [ ] types.rs：merge/import_items 冲突检测 + enum 统一；`cargo t types`。
4. [ ] 限定查找通路落地（按 Spike 结论选 D3 路线）；`cargo t parser` + `cargo t typeck`。
5. [ ] autovm_persistent.rs:323 + lib.rs 三处：注册分支收紧；`cargo t`（受影响模块）。
6. [ ] trans/rust.rs bare 发射收紧 + trans/python.rs 排查对齐；`cargo tt`。
7. [ ] 诊断信息 D6；`cargo t`。
8. [ ] examples 34 处迁移 + 双端验证（notes 零改动回归 + quickstart 迁移）；
   `auto run` / `auto run -r vm`。
9. [ ] crate 测试夹具迁移（~89 处，按文件分批）；`cargo t`。
10. [ ] indexer.rs / auto-lsp use 处理排查对齐；LSP 冒烟。
11. [ ] 文档更新 10-language-syntax.md + specs 回写；`cargo test -p auto-lang --test docs_gen`。
12. [ ] 全量门禁 `cargo t` → `cargo tf`，复审记录。

## 复审记录

（review 阶段填写）

## 待澄清事项

1. **传递性隔离（D1 末项）风险兜底**：若解析上下文耦合过深，降级为维持现状 + KNOWN-DEBT，不阻塞主体。
2. **冲突检测严重级别**：默认按 error 设计；若存量 re-export 场景（Plan 376U crate-root `use X: sym`
   公共再导出）出现大量同名同定义误报，可对"定义相同"情形降为允许（已设计），仅"定义不同"报 error。
3. **lazy glob（按需解析）明确出期**：本 plan 不做 resolver 侧 fallback namespace；如后续 TypeStore
   merge 在大项目 profile 中成为瓶颈，另立 plan 做 Rust 式 lazy glob。
4. **Spike 待确认**：typeck 对 `db.X` 的符号取数路径（决定 D3 改动量）——执行步骤 1 首先回答。
