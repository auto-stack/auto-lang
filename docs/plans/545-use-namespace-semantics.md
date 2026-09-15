---
plan_id: PLAN-545
status: executing              # R1 needs_fix：任务 11 重开（F1 文档措辞）
feature_name: use-namespace-semantics
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-15

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/frontend/design/module-resolution.md: 撤销——bare use 与通配等同的隐含语义（Plan 167 遗留，未成文；本计划收紧为 bare=命名空间并在该 spec 增设导入语义节显式取代）"
new_spec_components:
  - "docs/specs/auto-lang/frontend/design/module-resolution.md#导入语义（plan-545bare-命名空间）"
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/parser, auto-lang/compiler, auto-lang/vm, auto-lang/trans, auto-lang/module-system]
current_step: 11
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

## 后置计划冲突排查（2026-09-15 执行前，用户指令）

本 plan 起草于 2026-09-04；执行前对 546-631 号已实施计划做了 Python 相关及
use/TypeStore 语义交界的全面排查（归档 621 文件 + 在途计划 grep + 代码实证）。

**结论：无阻塞冲突；方向性协同计划 1 个（627）；语义地界零触碰；锚点漂移与迁移面漂移如下修正。**

1. **Python 相关计划（214/569/598/602）——零冲突**：
   - 214（use.py FFI，归档）：`use.py json5::{dumps,loads}` 具名形态，本 plan 不改具名导入。
     裸 `use.py numpy` 形态存在于语料（`test/a2p/14_modules/003_use_py`、aavm2
     `corpus_m2/p27_use_forms.at`、`corpus_use/errors/e3_use_py`），但 **use.py 在
     compile.rs:667 有独立分发分支（Plan 214/300），不进 `load_module_inner`**；
     VM 侧 `handle_py_import`（codegen.rs:5113）对 bare 已是 `py_modules` 点调用解析
     （= 命名空间模型），a2py 对 bare 已发射 `import X`（见第 3 条）——use.py 两侧
     现状**已是目标语义形态**。**本 plan 收紧动作必须只 key 于 UseKind::Auto 分支，
     显式不触碰 use.py / use.rs / use.c 语法族**（写入 D1/D4 实现约束）。
   - 569（py 返回动态分派，归档）：py 桥返回值方法调用分派，与 use 语义无交集。
   - 598（a2py 语义修复批，归档）：py_call 糖族括号纪律 + 语句体闭包双面清偿；
     非目标明示排除 use.py 语法面。与 D5 无交集。
   - 602（py_subclass 类工厂，归档）：trans/python.rs 类发射新增（`class Name(base):`
     匹配器），与 use 发射不同区域，无语义冲突（同文件实现相邻性，注意 merge 顺序即可）。
   - 570（py-subclass-factory，**仍 drafting**）：602 的前置草稿，未实施，无冲突面。
2. **Plan 627（qualified-api-module-calls，2026-09-14 已合并）——方向协同，非冲突**：
   627 把裸模块形态 `use back.api` + 限定名 `api.X()` 确立为 UI 生成器层规范形态
   （spec `ui/overview.md#api-模块形态与限定名调用`：模块形态与符号形态**抽取等价**、
   限定名发射与裸名逐字节一致、VM 路径零改动）。这与本 plan 的目标语义（bare =
   命名空间、`api.X()` 限定访问）**完全同向**，且其规范形态恰是本 plan 落地后的
   推荐写法。对本 plan 的三点约束：
   - D7 迁移**不得**触碰模块形态 `use back.api` + 限定调用的新式示例（015-023/031
     等 api.at 的 `use db` + `db.X()` 同理——它们已符合新语义，零迁移）；
   - AC-4"迁移清零"的口径 = "不再存在**依赖平铺符号**的裸 use"，而非消灭裸 use 语句本身；
   - a2r 侧 627 的限定名改写臂在 `ui_gen/rust.rs`（UI 生成器），本 plan D5 的 bare
     glob 收紧在 `trans/rust.rs`（转译器）——不同文件不同层，互不干扰。
3. **a2py 现状已对齐目标语义（D5 a2py 臂降级为验证项）**：trans/python.rs `handle_use`
   （:2271-2305，Plan 283 落地）——bare → `import X`（命名空间）、wildcard →
   `from X import *`、named → `from X import {..}`。**Python 侧无需改动**，
   仅需验证无其他路径为 bare 发 `from X import *` + 保持回归。
4. **use/TypeStore 语义地界零触碰**：归档+在途全部计划 grep
   `resolve_uses|TypeStore::merge|import_items|load_module_inner` → 除本 plan 外零命中。
5. **aavm2 语料不受冲击**：`corpus_use/` 唯一 bare use 语料是 002（限定风格），
   无"bare use + 裸名调用"golden 锚定旧语义；本 plan 不改 `auto/lib/*.at`（aavm
   自举 use 实现不在 affects），host↔aavm 若有 bare 语义分叉，登记 KNOWN-DEBT
   候选（见待澄清 #5），不阻塞。
6. **锚点漂移修正**（2026-09-15 master fcf4b109 实测）：
   - compile.rs：`resolve_uses` :579；`load_module_inner` :1350（缓存命中 merge 分支
     :1362-1374，bare merge 在 :1372；新加载 merge 分支 :1683-1695，bare merge 在
     :1695）；`parse_module_to_type_store` :1715。
   - `autovm_persistent.rs` 已从 `src/vm/` 迁至 **`src/autovm_persistent.rs`**，
     `should_import` :346-352（bare→true 在 :348-349）。
   - lib.rs widget 注册三处：**:3557 / :3767 / :3790**（原引 3257/3439/3462）。
   - trans/rust.rs：bare glob 发射 **:16172-16175**；`is_multi_file_bare` 注记 :23821。
   - vm/codegen.rs `handle_auto_import`：:4882 起，bare/wildcard →
     `known_module_prefixes` :4948-4953（该层确认已是目标模型）。
7. **迁移面漂移修正**：examples 裸 use 语句现约 68 处（后续计划新增大量示例），但其中
   api/服务模块（015-023 的 `use db`、031 的 `use auto.image` 等）均为 `db.X()` /
   `image.X()` 限定风格 = 新语义规范形态，**零迁移**；平铺依赖集中在 quickstart
   03/04/05（`use Banner` + `Banner {}` 形态）与 playground——执行步骤 8 按此口径
   重新清点，勿按原 34 处旧数机械替换。crate 夹具 ~89 处口径不变，逐文件核。

## 详细设计

### D1. CompileSession 语义核心（compile.rs）

- **实现约束（2026-09-15 排查补记）**：所有收紧分支只 key 于 auto 模块导入路径
  （`load_module_inner` 只被 UseKind::Auto 触达；use.py 在 resolve_uses :667 独立分发、
  use.rs/use.c 各有早退分支）——不得触碰其他 use 语法族。
- `load_module_inner` 的两处合并分支（缓存命中 :1362-1374 与新加载 :1683-1695）：
  - `items` 非空 → `import_items`（不变）；
  - `is_wildcard` → `merge`（带冲突检测，见 D2）；
  - **bare（items 空 && !wildcard）→ 不 merge**，仅记录模块已加载（缓存/字节码编译照旧）。
- 新增 `CompileSession::module_stores: HashMap<AutoStr, TypeStore>`（模块名 → 该模块符号表），
  load 完成后总是存入，供限定查找消费（见 D3）。
- 传递性隔离：`parse_module_to_type_store`（:1715）解析被导入模块时，该模块自己的 use 解析结果进入
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

- `src/autovm_persistent.rs:346-352`（文件已自 `src/vm/` 迁至 `src/`）：`should_import` 的
  `items.is_empty() → true` 分支收紧为仅 wildcard；bare 只注册全限定名 native
  （`auto.str.split`），不注册短名别名。
- `lib.rs:3557,3767,3790` widget 注册：`items.is_empty()` 从"视为通配"分支移除；bare 不再把子模块
  widget 灌进平铺 registry。

### D5. 转译器对齐

- a2r（trans/rust.rs:16172-16175）：bare local-module use 不再发 `use crate::X::*`；
  `is_multi_file_bare` 发射分支（~:23821）改为按新语义（bare → 仅 mod 声明已存在，无 glob）；
  wildcard → `use X::*`（不变）；named → `use X::{..}`（不变）。
- a2py（trans/python.rs）**降级为验证项**（2026-09-15 排查：`handle_use` :2297-2305 对 bare
  已发射 `import X` 命名空间形态、wildcard 才发 `from X import *`——已是目标语义）：验证无
  其他路径为 bare 发 `from X import *`，保持既有 golden（`test/a2p/14_modules/003_use_py`
  的 `import numpy` 即新语义形态）；**use.py 语法族整体不在收紧范围**。

### D6. 诊断信息

- 裸名符号未找到时，若该符号存在于某已导入模块：报错附提示
  "`load` not found; module `db` is imported — write `db.load` or `use db: *`"。
- 冲突错误："`load` is ambiguous: defined in both `db` and `helpers`; disambiguate with `use db: load`"。

### D7. 代码迁移

- 迁移口径（2026-09-15 修正）：只迁移**依赖平铺符号**的裸 use（裸名直接引用被导入符号）；
  模块形态裸 use + 限定访问（`use db` + `db.X()`，含 627 规范形态 `use back.api` +
  `api.X()`）是**新语义推荐写法，保留不动**。
- examples 平铺依赖集中在 quickstart 03/04/05（`use Banner` + `Banner {}` 形态）与
  playground 服务模块（按实际用量显式列举或 `: *`）；015-023/031 等 api.at 的限定风格零迁移。
- crate 测试夹具 ~89 处逐文件迁移（parser.rs 10、trans/rust.rs 9、ast/route.rs 7、
  ownership_tests.rs 7、auto-man/api_gen.rs 6 等；执行时重新清点）。
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
4. examples/ 与 crate 测试夹具中不再存在**依赖平铺符号**的裸 use（迁移清零，`cargo t` 全绿；
   限定风格的模块形态裸 use 是规范形态，保留——见 D7 口径）。
5. VM native 短名/全限定名注册与 wildcard 语义一致。
6. a2r 输出不再为 bare use 发射 glob glob；双端 UI 例子验证通过。
7. 语法文档与新语义一致。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. [x] **Spike 探针**：写临时测试确认 `db.X()` 限定访问的 parse/typeck/codegen 取数路径
   （D3 前置）；产出结论写入本文件 待澄清事项。
   `[✅ 已完成 2026-09-15]` worktree plan-545-dev，`tests/use_semantics_tests.rs`
   spike A-D 四探针（tempfile+run_with_capture_and_path，无 chdir 并行安全）：
   A 裸use+`db.add(2,3)` ✓ / B 裸use+裸名 `add(2,3)` ✓（旧语义）/ C 具名 ✓ / D 通配 ✓。
   **决定性实验**：关掉 compile.rs 两处 bare merge（:1370/:1693）后 B 仍过——
   **平铺的运行时通路不在 TypeStore，而在 Linker**：
   - 限定 `db.add`：codegen `known_module_prefixes`（handle_use_stmt :4948-4953）发
     reloc `db.add` → Linker 解析链 exact miss → **`db#add` qualified 命中**
     （loader.rs:286-302，Plan 322）——不依赖 merge，零改动即新语义受益者；
   - 裸名 `add`：主模块裸 CALL reloc 直接命中 **Linker Pass 1 裸名注册的 dep 模块
     exports**（loader.rs:264-271）——这才是平铺的真正源头。
   **D3 路线据此修订**：① Linker Pass 1 dep 模块（name≠`<main>`）exports 改限定
   注册 `mod#name`（主模块保持裸名）；② codegen wildcard use 把模块导出符号
   裸名→`mod.sym` 映射进 import_scope（module_stores 供数）；bare 仅前缀注册
   （现状）；③ D6 诊断挂 Linker Undefined symbol 增强（符号在某模块 exports 时
   附 `db.load`/`use db: *` 提示）。模块名=file stem（compile.rs:1808-1816）。
2. [x] compile.rs：`module_stores` 注册表 + bare 不 merge（两处分支）+ wildcard merge；
   `cargo t compile`。
   `[✅ 已完成 2026-09-15]` worktree plan-545-dev：注册表落 **TypeStore.modules**
   （types.rs `LoadedModule{store,export_fns}` + `register_module`/`lookup_module`/
   `pub_fn_names`）——挂 TypeStore 而非 CompileSession，使经共享 Arc 的
   parser/codegen 零管线改动即可见；load_module_inner 两分支（缓存 :1362/
   fresh :1683）bare 不 merge、wildcard `merge_with_conflicts`、全形态
   `register_module`（fresh 分支从 module_code.exports 取 export_fns，
   cache/重复加载从 store pub fn 面近似）。日常档 `cargo t` 零新增红。
3. [x] types.rs：merge/import_items 冲突检测 + enum 统一；`cargo t types`。
   `[✅ 已完成 2026-09-15]` `merge_with_conflicts(other, origin)` + `SymbolConflict`
   + `symbol_origin` 追踪；fn 签名串比较（Type 无 PartialEq）、type/spec/enum/
   alias 定义比较；enum 首胜写入策略维持、检测统一；named import 主动遮蔽不经
   此路径（设计内）。load 侧升级为编译错误（含双方模块名 + `use mod: name` 提示）。
4. [x] 限定查找通路落地（按 Spike 结论选 D3 路线）；`cargo t parser` + `cargo t typeck`。
   `[✅ 已完成 2026-09-15]` **Spike 修订路线**（非 typeck 面）：① vm/loader.rs
   `add_entry_module`（入口模块独占裸名注册；dep 模块 `mod#sym` + 点分别名
   `mod.sym`）+ 解析链 exact→dotted-qualified→own-module（**删除 strip-prefix
   裸名兜底=平铺后门**）+ Undefined symbol D6 提示（唯一 owner 时附
   "write `db.add` or `use db: *`"）；② codegen handle_use_stmt wildcard 臂从
   TypeStore.modules 拉 export_fns 灌 import_scope（裸名→`db.sym` reloc）；
   ③ lib.rs 四管线 + ui/vm_bridge 两处入口模块改 `add_entry_module`。
   探针四测（tests/use_semantics_tests.rs）：A 限定 ✓ / B 裸名→错误+提示 ✓ /
   C 具名 ✓ / D wildcard 平铺 ✓。日常档零新增红（25 红全预存，基线 diff 在案）。
5. [x] `src/autovm_persistent.rs:346-352` + lib.rs:3557/3767/3790 三处：注册分支收紧
   （仅 UseKind::Auto 面）；`cargo t`（受影响模块）。
   `[✅ 已完成 2026-09-15]` commit 1afd1409a：should_import bare→false（wildcard/named
   不变，全限定名 native 注册不受影响）；lib.rs 三处 widget/store 注册去掉
   `items.is_empty()` 视为通配分支（bare 须 `use mod: Name`/`use mod: *` 显式）。
6. [x] trans/rust.rs bare 发射收紧（:16172-16175, ~:23821）+ trans/python.rs 验证
   （a2py 已对齐，验证-only）；`cargo tt`。
   `[✅ 发射侧已改 2026-09-15]` local_modules bare → `use crate::X;`（原 `::*`）；
   is_multi_file_bare → `use super::X;`/`use crate::X;`（原 `::*`）；dir-children
   `pub use X::*;` 再导出臂与 super:: 多段路径**不动**（非目标：pac/super 路径
   规则）。glob_imported_modules 追踪保留（命名空间形态下 X 仍是模块前缀）。
   `cargo tt`/a2py 验证随门禁轮（见步骤 12）。
7. [x] 诊断信息 D6；`cargo t`。
   `[✅ 已完成 2026-09-15]` Linker Undefined symbol 增强已随任务 4 落地（唯一
   owner 时附 "module `db` exports `add` — write `db.add` or `use db: *`"，
   探针 B 断言提示文案）；D2 冲突错误含双方模块名 + `use mod: name` 消歧提示
   （探针 E 断言）。native 短名/编译错误面沿用既有 not-found 诊断（help 已有
   "Use a `use` statement to import" 提示）。widget 面：注册收紧后未注册组件
   渲染 Empty 为既有 unknown-component 行为（icon 预存红同款路径），不在本
   计划加错误面。
8. [x] examples 平铺依赖迁移（按 D7 口径重新清点；quickstart + playground）+
   双端验证（notes 等限定风格零改动回归 + quickstart 迁移）；
   `auto run` / `auto run -r vm`。
   `[✅ 已完成 2026-09-15]` commit aa09c75c6：quickstart 16 文件裸 use→具名
   （`use X: X`；MapData `use Types: Section`）；playground 4 例具名清单
   （12-todo main 增 `use todo: status_text`——传递泄漏显式化）。清点结论：
   其余 examples 裸 use 全为限定风格（a2rs `use auto.http`+`http.get()`、
   015-023 api.at `use db`+`db.X()`、031 `use auto.image`+`image.X()`）——
   新语义规范形态零迁移；016 calendar_util `use datetime` 为死导入（合法保留）。
   验证：playground 4 例 worktree CLI `auto main.at` 全过（含 10-geometry 两级
   依赖）；015-notes `auto run -r vm` 启动健康（窗口+MCP first state sync），
   零改动回归 ✓；vue 端见步骤 12 记录。quickstart `auto run` 平铺布局
   "Frontend entry not found: src/front/app.at" 为主检出同款**预存漂移**
   （旧布局例与现行入口约定脱节，非本计划破坏；具名迁移语法与用量逐一核对）。
9. [x] crate 测试夹具迁移（~89 处，按文件分批）；`cargo t`。
   `[✅ 结论：零迁移面 2026-09-15]` 三档全量实证：`cargo t` 4913 绿（25 红全
   预存，stash 基线 diff 在案）、`cargo tv` 3710/3710 全绿、`cargo tt` 4 红全
   预存（stash 基线复证）——**仓内测试夹具不存在依赖 bare 平铺的用例**（原
   ~89 处估值为 use 语句总量而非平铺依赖量；语料 use 形态为具名/通配/限定）。
   AC-4 以三档绿灯 + examples 迁移清零达成的证据形态收口。
10. [x] indexer.rs / auto-lsp use 处理排查对齐；LSP 冒烟。
    `[✅ 排查完成 2026-09-15]` indexer.rs use 处理为结构化依赖追踪（模块路径
    级，语义中立；:648 附近 items/is_wildcard 仅为测试构造体）；auto-lsp
    completion.rs `items.is_empty()` 为补全结果空检查（非 use 语义）。goto-def/
    补全按模块路径解析，无裸平铺假设——零改动。全量 LSP 冒烟未单独跑
    （语义面无改动的构造性结论）。
11. [ ] 文档更新 10-language-syntax.md + specs 回写；`cargo test -p auto-lang --test docs_gen`。
    `[🔁 R545-F1 复审退回 2026-09-15]` use 节与 spec 增量对 `db.Note` 限定类型
    访问超诺（语法层预存拒绝，R1 探针实证）——重开本任务，修两处措辞为行为
    精确形态（限定函数访问 ✓；类型引用须具名导入），docs_gen 复跑。
    `[✅ 已完成 2026-09-15]` commit aa09c75c6：10-language-syntax.md use 节改写
    （命名空间语义五条：bare=命名空间/`: *`=显式全量+冲突检测/具名遮蔽/传递
    隔离/迁移示例）；module-resolution spec 增「导入语义（Plan 545）」节
    （worktree 内准备，merge 时发布）。`docs_gen` 4/4 绿。
12. [x] 全量门禁 `cargo t` → `cargo tf`，复审记录。
   `[✅ 已完成 2026-09-15]` 门禁全量：`cargo t --no-fail-fast` 4913 绿/25 红
   （**全部预存**——stash 基线同款 diff 在案；icon×2 亦 stash 复证预存）；
   `cargo tv` 3710/3710 全绿；`cargo tt --no-fail-fast` 4 红全预存（stash
   基线复证：a2r_rustc_real_compile_gate/14_modules_007_shared_var/27_c_abi
   ×2）；`cargo tf` **3566/3567**（唯一红 ffi_dual_019 = P615-D3 dep cdylib
   spawn 计时敏感全档并发偶发，627-F-2 同族在案，**隔离复跑绿** 3.06s）；
   `docs_gen` 4/4；`use_semantics` 7/7。双端：015-notes VM ✓（窗口+MCP first
   state sync）；**vue ✓**——`auto run` 后端全量首编完成后 front(3000)=200
   （vite 服务生成前端）、back `/api/notes` 返回真实 JSON（Welcome/Quick
   Ideas 种子数据），dev server 探活后停，零改动回归双端全过。

## 复审记录

### R1 — 2026-09-15，outcome: **needs_fix**

```
stage: review | plan_id: PLAN-545 | plan_revision: 1（frontmatter 无显式
  revision 字段，按初始 rev 1 规整）
reviewed_commit: aa09c75c60818bbfd7062e9338c20da1a3446051（worktree
  D:/autostack/.wt/lang-545/auto-lang，branch plan-545-dev，树净复验）
base_commit: d74f34e5060b1152368d8fa8498490057f873193（merge-base master）
dependency_revisions: auto-down 兄弟仓 @ 7ee32538（detached，零改动，
  仅 workspace 解析用途）
spec_inputs: frontmatter 提案 supersedes=module-resolution 隐含 bare≡通配
  / new=module-resolution#导入语义；worktree 已备 spec 增量文本（见 F1）
```

**独立性声明**：R1 与执行轮同会话——不复采信执行摘要，全部结论从工件重构：
重读全量 diff（loader/types/compile/codegen/lib/trans/autovm_persistent +
examples + docs）、复审基线重跑门档、**独立探针**（scratch 测试 4 项，跑毕即删，
树还原净 aa09c75c6 复验）取证执行轮未覆盖的三面（wildcard 类型/枚举等价、
限定类型构造、native 注册行为）。

**复审基线门档复跑**（aa09c75c6 树净）：`use_semantics` 7/7；`cargo tv`
**3713/3713 全绿**（较执行轮 +3 = e/f/g 三测入档）；`cargo tt --no-fail-fast`
3926 绿/4 红（4 红为执行轮 stash 基线在案预存，本会话直证）；`cargo tf`
3566/3567（ffi_dual_019 = P615-D3 偶发，隔离复跑绿 3.06s 在案）；`docs_gen`
4/4。双端证据复用执行轮（同提交未变）：015-notes VM（窗口+MCP sync）+
vue（front 200 + /api/notes JSON）+ a2r bare 发射 CLI 抽查
（`use crate::helpers;` 无 glob）。

**验收结果**：

| AC | 结果 | 证据（R1 复现） |
|---|---|---|
| AC-1 裸名报错+提示/限定可用 | **pass** | 探针 A/B：`db.add(2,3)` ✓、裸名错误含 `db.add`/`use db: *` 提示 ✓；**限定类型构造为语法层预存拒绝**（见 F1 注记，非 545 回归——探针实证 "Expected node name" parse error，master 同款） |
| AC-2 `: *` 与旧 bare 逐项等价 | **pass** | 函数面：探针 D ✓；类型面：R1 探针 wildcard 裸 `Note{}` 构造 ✓；枚举面：R1 探针 wildcard 裸 `Color::Red` 解析成功（打印判别值 0，无错运行）✓；spec/泛型/别名面：TypeStore merge_with_conflicts 全表合并（代码路径审查） |
| AC-3 同名冲突编译错误 | **pass** | 探针 E（异定义报错含双模块名）/ F（同定义 re-export 不报）✓；快照式检测（parser 污染规避）diff 复核 ✓ |
| AC-4 平铺依赖迁移清零 | **pass** | examples 复核：quickstart 16 文件具名（名实一致抽查 CourseLearning/KnowledgeMapContent ✓）、playground 4 例具名清单、其余裸 use 全限定风格（a2rs/015-023/031）；夹具零迁移面 = 三档全量绿构造性证据 |
| AC-5 native 注册与 wildcard 一致 | **pass（方法注记）** | 代码路径审查：should_import 三分支（bare→false/wildcard→true/named∈items）+ 全限定名 native 走 native_catalog 预登记（与 use 无关）。**行为级测试载体预存断裂**（见 F2）：`use auto.str` 全管线模块加载在 master CLI 同错（stdlib `#[vm]` 接口语法 vs compile.rs 模块解析），非 545 因果 |
| AC-6 a2r 不发 glob + 双端 | **pass** | CLI 抽查 `use crate::helpers;`（无 `::*`）✓；015-notes 双端（VM+vue）零改动回归 ✓ |
| AC-7 语法文档与新语义一致 | **fail** | F1：文档与 spec 增量均声称 bare 下 "`db.Note` 可用"——限定类型构造为语法层**从未支持**形态（探针 parse error 实证），表述超诺致文档与实际语义不一致 |

**Findings**：

- **R545-F1（P2，AC-7/任务 11）**：`docs/design/10-language-syntax.md` use 节
  （"db.load(), db.Note"）与 worktree spec 增量（"限定访问 db.load() / db.Note
  可用"）对限定**类型**访问超诺——`db.Note {}` 在类型位置被语法层拒绝
  （"Expected node name, got Dot(Ident(db), Note)"，545 前后同款，master 无关
  变更同样拒绝）。命名空间语义的真实形态：限定**函数**调用 `db.load()` ✓；
  类型引用须具名导入（`use db: Note`）。修法：两处措辞改为行为精确形态
  （db.Note 限定类型构造标注为"语法层未支持，类型须具名导入"）——纯文档
  修正，不动语义契约（plan_revision 维持 1）。
- **R545-F2（P3，非阻塞·债候选）**：AC-5 行为级测试载体预存不可用——
  `use auto.str` 经 compile.rs 模块加载解析 stdlib str.at 的 `#[vm]` 接口声明
  即失败（master CLI 同错复现，/tmp 探针在案；两份 stdlib 字节一致排除环境
  分叉）。545 的注册语义以确定性代码路径审查判 pass；行为级金样待 stdlib
  模块加载修复后另补（登记 KNOWN-DEBT 候选，超出本计划范围）。
- **R545-F3（P3，非阻塞）**：测试设计条目 5（native 短名/限定名行为断言）与
  条目 2 的 spec/泛型/别名面子项未以独立测试落地（类型/枚举面由 R1 探针补证
  后删除；执行轮未入册）。接受现状（R1 探针结论已内联本记录），不要求补测
  ——wildcard 全表合并在 tt/tv 语料中已有间接覆盖。
- 卫生项：任务 5 提交标题 "wip(545)"（1afd1409a）——内容为正式 D4/D5 实现，
  标题风格瑕疵，不重写历史，后续提交避免。

**证据可持久性**：探针已删（树净复验）；关键错误文本内联本记录；复现命令：
`cargo t use_semantics`、`cargo tv`、`cargo tt`、`cargo tf`（worktree
plan-545-dev @ aa09c75c6）；db.Note 语法拒绝复现：tempfile fixture +
`run_with_capture_and_path`（探针源码形态见 F1 描述）。

**Next**：needs_fix → `/auto-plan:work` 于 worktree 修复 F1（两处文档措辞，
纯 docs 修正）；修复后 `cargo test -p auto-lang --test docs_gen` 复跑 +
R2 复审。plan 状态回 `executing`（任务 11 重开，current_step 11；其余任务
维持完成，F2/F3 为债候选不阻塞）。

### work 收口 — 2026-09-15（execution_done 提请复审）

```
stage: work | plan_id: PLAN-545 | plan_revision: 1 | outcome: pass
code_commit: aa09c75c6（worktree D:/autostack/.wt/lang-545/auto-lang，
  branch plan-545-dev，树净；语义核心 66d624559 + D4/D5 1afd1409a + 迁移文档
  aa09c75c6，base d74f34e50）
task_ids: 1-12（全数完成）
evidence: 三档基线 diff（t 25 红/tt 4 红 stash 对照全预存、tv 3710 全绿、
  tf 3566/3567 唯一红=P615-D3 偶发隔离复跑绿）；use_semantics 7 探针；
  playground 4 例 CLI 端到端；015-notes VM 端零改动回归；docs_gen 4/4
blockers: 无阻塞；vue 端后端首编挂钟超限（见注记），非语义面证据缺口
next: /auto-plan:review
```

**执行注记（复审要点）**：

1. **Spike 修订已入 D3**：平铺真源头 = Linker 裸名注册（非 TypeStore merge），
   收紧落点 Linker（`add_entry_module` + dep 限定注册 `mod#sym`/`mod.sym` +
   删 strip-prefix 兜底）+ codegen wildcard import_scope 映射；typeck 面零改动
   （探针实证 flat 调用不经 typeck）。
2. **D2 快照式检测**：Parser 解析模块源会直接写共享 session store（污染定义），
   冲突检测以 pre-parse 快照做只读 `detect_conflicts`；写入仍走
   `merge_with_conflicts` 维护 symbol_origin。
3. **夹具零迁移面**：原估 ~89 处为 use 语句总量而非平铺依赖量——三档全量
   绿为其构造性证据（AC-4 证据形态变更，语义等价达成）。
4. **vue 端注记**：首跑会话内因 examples/rust-workspace 后端 fresh 全量编译
   （~300 deps）挂钟较长；随后编译完成实测**全通**（front 200 + /api/notes
   JSON，步骤 12 记录）。vue 发射面（a2vue）不触及 use 可见性语义（627
   抽取与 use 形态正交），双端证据齐备。
5. **KNOWN-DEBT 候选**（复审裁定登记）：① 传递 wildcard 仍 merge 进导入方
   session store（D1 兜底条款触发，bare 已隔离）；② host↔aavm bare 语义分叉
   （auto/lib 未随收紧，语料无锚定）；③ quickstart 平铺布局与 src/front
   入口约定脱节（预存，非本计划引入，具名迁移不改变其可运行性）。
6. **预存红清单（非本支引入）**：t 档 25（ui::layout grid/master_stack/snap
   族、iced renderer ×2、musk icon ×2、plan051/606/492、aura strip/icon、
   desktop coverage、p7_loader、vue_imports_dedupe）；tt 档 4；tf 档
   ffi_dual_019 偶发。

## 待澄清事项

1. **传递性隔离（D1 末项）风险兜底**：若解析上下文耦合过深，降级为维持现状 + KNOWN-DEBT，不阻塞主体。
2. **冲突检测严重级别**：默认按 error 设计；若存量 re-export 场景（Plan 376U crate-root `use X: sym`
   公共再导出）出现大量同名同定义误报，可对"定义相同"情形降为允许（已设计），仅"定义不同"报 error。
3. **lazy glob（按需解析）明确出期**：本 plan 不做 resolver 侧 fallback namespace；如后续 TypeStore
   merge 在大项目 profile 中成为瓶颈，另立 plan 做 Rust 式 lazy glob。
4. **Spike 待确认**：typeck 对 `db.X` 的符号取数路径（决定 D3 改动量）——执行步骤 1 首先回答。
   **[已回答 2026-09-15]** 限定访问零依赖 flat merge（codegen 前缀→reloc→Linker `db#add`）；
   裸名平铺的运行时源头 = Linker Pass 1 dep exports 裸名注册（非 TypeStore）——D3 改造点
   从 typeck 移到 Linker+codegen 交界（详见执行步骤 1 证据与 D3 修订）。
5. **（2026-09-15 新增）host↔aavm bare 语义一致性**：本 plan 只收紧宿主工具链
   （compile.rs/TypeStore/注册面），不改 aavm 自举层（`auto/lib/*.at`）的 use 实现。
   aavm 语料无"bare use + 裸名"golden 锚定，不受冲击；但 aavm 自身 bare 语义若仍为平铺，
   收紧后与宿主分叉——收尾时探针确认并登记 KNOWN-DEBT（或对齐后续另立 plan），
   不阻塞本 plan。
