---
plan_id: PLAN-1131
status: archived
completion_kind: delivered
feature_name: "AV 标准库公共类型与旧导入兼容"
author: [Codex]
created_at: 2026-11-24
updated_at: 2026-11-24
plan_revision: 2
current_step: 8
total_steps: 8
depends_on: [PLAN-972, PLAN-1095, PLAN-1121]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/stdlib/design/ace-collections.md, docs/specs/auto-av/language-parity.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1131] AV 标准库公共类型与旧导入兼容

## 0. 变更摘要

2026-11-24 起草。承接 1121 的 `D-1121-MAP-PARAM-ANNOTATION`，再核对旧文本、构建器和 Result 导入中尚未承接的普通 AV 用例。

**r2（2026-11-24，用户裁定“本计划补齐多文件包”）**：r1 只在单文件普通面交付，V-01 原文的“多模块”未兑现。r2 保持目标、仓库集与 V-01 原文不变，新增多文件 `packages` 装配线上的容器公共类型与旧导入（T-06/T-07、AC-05），把复审的两条代码备注纳入关闭（T-08），并重开复审与冻结两步。r1 已完成的 T-01～T-03 与其证据保留；r1 的复审 pass 只覆盖单文件面。

## 1. 目标

在 Windows AV 普通工程中，可将现有 `HashMap`、`HashSet` 写入函数形参和返回类型，并在跨模块调用中继续执行已支持的容器操作。补齐本计划列出的旧标准库调用和导入形态，已有等价行为通过原用例和新版公共入口对账。全类型域的泛型容器、通用隐式转换和额外旧包族各沿其语言或标准库合同交付。

## 2. 架构方案

公共类型引用经既有标准库装配和名称解析绑定到同一受检类型身份，形参、返回值和方法接收者沿既有参数传递、所有权和集合底座执行。保留用户侧同名声明、导入别名、跨模块可见性和遮蔽规则。旧导入从原语料取名，接入现有公共 Auto 模块或按已批准的等价拼写迁移，避免复制容器或文本算法。

## 3. 技术栈

`experimental/ace/crates/core/src/source/{resolve,stdlib_refs}.rs`、`stdlib/ace/asm/map.at`、`stdlib/auto/{hashmap,hashset,result}.at`、`experimental/ace/crates/av/tests/cases/plan1121_map_set.rs`、`crates/auto-lang/test/cases/language/migration/legacy-map.tsv`。

r2 实际触面（T-06 核定后）：`experimental/ace/crates/core/src/source/adapter.rs`（包面嫁接与冻结旧模块源）、`experimental/ace/crates/core/src/source/packages/binding.rs`（`type_name` 与 `use auto.result`）、`experimental/ace/crates/source-query/src/modules/packages.rs`（`compile_package_product_auto_prepared`）、新增 `experimental/ace/crates/av/tests/cases/plan1131_multi_module.rs`。

实施树为 `D:/autostack/.wt/lang-1131/auto-lang`，分支 `plan-1131-dev`。

实施期实际触面（T-01 核定后的路径修正，范围未变）：`experimental/ace/crates/core/src/source/{resolve,adapter}.rs`、新增 `stdlib/ace/asm/builder.at`、新增 `experimental/ace/crates/av/tests/cases/plan1131_public_types.rs`、`legacy-map.tsv`。`stdlib_refs.rs`、`asm/map.at`（整数 map 装配线）与 `stdlib/auto/{hashmap,hashset,result}.at`（旧契约面，仅作取名来源）未改动。

## 4. 需求分析与背景调查

取材基线 `816585732`。仓库结构见 [总览](../../specs/overview.md)，以 [核心库现行规范](../../specs/ace-migration/m2-stdlib-core.md)、[集合规范](../../specs/stdlib/design/ace-collections.md)、[普通 AV 语言规范](../../specs/auto-av/language-parity.md) 和 [1121 归档](1121-av-stdlib-map-set.md) 为准。1121 已交构造、查改和删除，其复审记录了签名注解 `source.undefined`。绑定器现有 `__std_HashMap`、`__std_HashSet` 记录及局部值形状追踪，跨模块传递和返回需要验证真实类型身份与存储状态。迁移表仍有早期 hold 行，行状态本身无法证明当前源码缺功能。

用户授权建立三个 AV 收尾计划，本计划范围为 auto-lang 文档中的公共类型与旧导入兼容；修正代码和验证结果留给实施阶段。无新增工期或资源预算。

## 5. 详细设计

- T-01 用 1121 的实际键值形状核跨模块形参、返回和调用者续用。记录当前公共类型来源、同名遮蔽、值形状跨调用的事实和所有权传递方式；T-02 据此选择既有类型/实例机制。签名需要表达容器支持域，矛盾形状须在可判定处具名诊断；单文件和多模块从各自正常入口观察。
- T-03 以旧 `legacy.13_collections.014–019` 的 `StrBldr`、`legacy.08_strings` 已登记的导入和方法、`legacy.26_str_method_on_heap.001–002`、`legacy.28_enum_methods.006_stdlib_result` 为候选矩阵。记录原源码、预期、当前模块导入和观察结果，逐条区分直接运行、已有等价实现、需补齐和已有批准差异。使用现有迁移表逐行记录，增加明确的兼容检查与入口。
- 982 的旧脚本整体验收和主干已登记回归沿原责任推进。若同一解析文件与 1133 并行修改，提交前核对相邻分支语义及两批定向用例。

- **r2 T-06/T-07（多文件包面，2026-11-24 用户裁定“本计划补齐多文件包”）**：普通 AV 的多模块正常入口是 PLAN-1085 的 Auto 包闭包（`bind_packages_auto` / `compile_package_product_auto`），同一 Auto 解析器。做法：把 `map_str.at`/`builder.at` 作为一个隐藏段并入绑定后的程序（同该入口已有的 `graft_package_u64_parse` 先例）；包绑定器把三个容器名当作内建名义类型原样交给解析器（模块自己声明或导入的同名类型优先）；`use auto.str: a, b` 是模块成员导入，由冻结的 `auto::str` 模块（`text.at` 改名 + 同一组转发函数）承接，`use auto.result` 按模块导入特判。
- **与 D-1125-01 的关系**：不相干。该既有红的成因是 `stdlib/ace/asm/list.at` 的私有 `fn join_into`（owner PLAN-118），只影响并入冻结装配标准库集的装配线；本计划的用例只含工程模块，不触达 `std::list`，本计划不修改它。

### 规范增量

| delta_id | 方式 | 目标 | 规则变化与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | `docs/specs/ace-migration/m2-stdlib-core.md` | 补可引用的容器类型、已验旧导入和等价拼写 | AC-01、AC-02 |
| SD-02 | modify | `docs/specs/stdlib/design/ace-collections.md` | 记录跨模块传递、返回、最后所有者及值形状的合同 | AC-01 |
| SD-03 | modify | `docs/specs/auto-av/language-parity.md` | 记录普通入口可运行的兼容矩阵及具名差异 | AC-02 |
| SD-04 | modify | `docs/specs/ace-migration/m2-stdlib-core.md` | 补多文件包面的容器公共类型与旧导入入口（r2） | AC-05 |
| SD-05 | modify | `docs/specs/stdlib/design/ace-collections.md` | 补容器跨真实模块边界的传递合同（r2） | AC-05 |
| SD-06 | modify | `docs/specs/auto-av/language-parity.md` | 兼容矩阵补多文件包面一列（r2） | AC-05 |

## 6. 测试设计

| 用例 | 任务 | 实际入口与输入 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | T-02 / AC-01 | 普通 AV 多模块 AutoDown Case。模块导出接收和返回容器的函数，调用者传入 1121 支持的字符串键和整数或字符串值，使用返回值继续查询、替换和删除 | 签名解析到同一公共类型。`score` 从 `9` 更新为 `11`，另一键与集合去重结果独立可读；调用结束后按受检所有权规则观察保留者和最后释放 |
| V-02 | T-02 / AC-01 | 同名用户类型、模块别名、错误值形状、局部和返回值逃逸负例，重放 1121 的缺键与释放对账 | 同名声明和导入按既有优先级绑定；错误形状在其约定阶段具名拒绝；失败路径和最后 owner 的分配/释放可核对 |
| V-03 | T-03 / AC-02 | 原构建器、字符串导入/堆字符串、Result 消费者及必要的等价新版源码，从普通 AV 工程入口执行 | 原可比 stdout/返回值逐项匹配；兼容改写附原诊断和新版独立预期；原未选项和已有裁定不计为实跑通过 |

| V-04（r2） | T-07 / AC-05 | 真实多模块：`lib.at` 导出接收/返回容器的函数，`main.at` 经 `use app.lib.…` 调用，经 `compile_package_product_auto`（Auto 包闭包）编译后在 AV 上执行 | 跨模块边界的容器是同一受检类型；被调模块的插入/删除对调用模块可见；调用模块对返回的容器续用；另一实例不受被调修改影响；错值形状具名拒；释放对账平衡 |
| V-05（r2） | T-07 / AC-05 | 多文件下的旧导入：`use auto.str: …` / `use auto.result`（包面的模块成员导入语法） | 导入名绑定到既有文本模块/内建 Result 族；模块同名类型与工程自带的 `auto` 包同名模块优先；未知模块/成员保持具名拒 |

实施时先重放 1121 定向用例及旧行，复审复验跨模块与所有权反例。冻结并集为 `cargo t ace-source ace-semantics ace-lowering ace-av ace-stdlib case-av-language-parity`，修改旧标准库资产时加 `stdlib`，共享类型事实改动若进入 A2X 消费路径，选相应已有集成组并记录理由。Windows 适用腿实跑，Linux 按 [1063](../1063-ace-stdlib-linux-deferred-validation.md) 交接。

**r2 更正**：r1 曾把“多模块”解释为单文件内的跨函数调用；用户 2026-11-24 裁定该解释不被接受，V-01 的“多模块 AutoDown Case”保持原义，由 V-04/V-05 在真实多文件包面兑现。下段保留为 r1 历史记录。

入口口径（T-01 核定，r1，已被 r2 更正）：普通 AV 的"多模块"在单文件普通 Auto 面即**同文件内跨函数**（模块导出函数 = 顶层 `fn`）；该面没有 `use` 项，也不走 `packages` 装配线。V-01/V-02 因此落在 `experimental/ace/crates/av/tests/cases/plan1131_public_types.rs`（真实公开编译链 `compile_language_source_with` → 实例化 → MIR → ABC 准入 → AV 执行 + arena 账本），与 1121 同一入口与同一语料纪律。

## 7. 验收标准

- [x] AC-01 · owner_stage=`work`。V-01/V-02 通过，跨模块类型身份与容器资源行为均有独立观察。（2026-11-24：`v01_*` 4 例 + `v02_*` 8 例；独立预期 12121 / 604 / 20 / 1117，错形状 `Verify:source.type-mismatch`、未知面 `source.map-face-unknown`、同函数面冲突 `source.map-face-mismatch`、跨函数异面条目走受检通道 771、缺键后 owner 存活 11、最后所有者释放对账）
- [x] AC-02 · owner_stage=`work`。V-03 逐行闭合，规范与迁移表对应到实际运行或已批准的差异。（2026-11-24：14 条旧行逐行登记 `legacy-map.tsv` PLAN-1131 行组，canonical=`av.parity.1131.public-types`；逐行结果见 §9 T-03）
- [x] AC-03 · owner_stage=`review`。在实施树独立沿类型引用、绑定和执行验证 AC-01/02，重放关键正反例和受影响近邻，核定规范增量。（2026-11-24：独立上下文复审 4 轮，最终 `pass` 绑定 `22f8d7c3d`，**范围限单文件面**；r2 新增范围由 AC-06 覆盖；见 §9）
- [x] AC-04 · owner_stage=`merge`。冻结组结果及现行规范回写经核对，收据合入、归档和清理。（2026-11-24：见 §9 merge 记录）
- [x] AC-05 · owner_stage=`work`（r2）。（2026-11-24：`plan1131_multi_module.rs` 13 例全绿，提交 `c7e04b756`；见 §9 r2 work 记录）V-04/V-05 在多文件 `packages` 装配线通过：容器公共类型与旧导入跨真实模块边界成立，与单文件面同一受检类型身份；D-1125-01 不被本计划修改。
- [x] AC-06 · owner_stage=`review`（r2）。独立复审覆盖 r2 新增范围与新提交；r1 的复审记录只覆盖单文件面。（2026-11-24：独立上下文复审 `pass` 绑定 `c7e04b756`，两次针对性确认至 `138bc91cc`；见 §9）

## 8. 执行步骤

当前 harness 有 TODO 工具时，实施前登记下列任务 ID。首个可执行任务为 T-01，依赖计划均已归档。

- [x] T-01 核定类型与兼容矩阵
  - owner_stage=`work`。在实施树从 `resolve.rs` 和 1121 用例复现形参/返回问题，冻结 V-01/V-02 的实际类型来源和所有权观察点；从迁移表记录 V-03 原用例与候选路径。交 §9 核定记录，供 AC-01/02。
  - [✅ 已完成 2026-11-24] 基线探针（实施树 `b0413e949`，提交前）：形参/返回/字段写 `HashMap`/`HashSet` → `Bind:source.undefined`；`var`/`let` 同名注解**不报类型错**而接收者静默变 i32。类型来源 = adapter 嫁接的 `__std_HashMap`/`__std_HashSet` 记录（`map_str.at`），构造回退已有、注解回退缺失（两处 `vt_of_ann*` 只查 `rec_index`）。值面按解析器局部变量追踪（每函数各自观察）。所有权：持资源记录实参按共享引用传递；`return` 局部容器后调用者可续用。差额：`StringBuilder` 不存在；`.upper()`/`.lines()` 缺失；单文件面在 Text 阶段整体拒 `use`；`Result.Ok`/`is_ok`/`is_err` 不支持。证据 `D:/autostack/plan-artifacts/plan-1131/t01-probe-baseline.txt`、`t01-probe-round2.txt`（merge 时随目录删除）。
- [x] T-02 交付容器签名与跨模块传递
  - owner_stage=`work`。复用 T-01 确认的绑定/装配，在现有集合语料或新增具名集合 Case 中验 V-01/V-02，重放 `plan1121_map_set`；对应 AC-01、SD-01/02。
  - [✅ 已完成 2026-11-24] `resolve.rs::grafted_migration_rec`：`vt_of_ann`/`vt_of_ann_coll` 的记录查找在程序未声明该名时回退到嫁接记录（与构造回退 `mapset_graft_rec` 共用一张表）→ 形参/返回/字段/局部注解绑定到同一记录身份。`v01_*` 4 例 + `v02_*` 8 例全绿；`plan1121_map_set` 重放 26/26。
- [x] T-03 补齐旧导入与调用
  - owner_stage=`work`。从已有文本和结果模块补齐 T-01 的实际差额，更新 `legacy-map.tsv` 并执行 V-03；对应 AC-02、SD-01/03。
  - [✅ 已完成 2026-11-24] 新增 `stdlib/ace/asm/builder.at`（`__std_StringBuilder`，纯 Auto：`List<Str>` + `+` 拼接 + f-string 整数渲染，无自有算法）；`.upper()` 并入既有 case intrinsic、`.lines()` 接文本模块；adapter 识别整行 `use auto.str: …`/`use auto.result` 并按字节原位抹空，导入名经转发函数接到 `text.at`（`__std_text_*`）与内建 Result 族；`Result.Ok/Err` 限定构造、`is_ok/is_err` = `is` 臂测试。`v03_*` 15 例 + adapter 单测 4 例全绿；`legacy-map.tsv` 14 行改登 mapped（diff 恰 14 增 14 删，左侧原用例列未动）。
- [x] T-04 独立复审（r1、r2 两部分均已通过）
  - owner_stage=`review`。重放 V-01～V-03 关键正反例，核所有权、旧行结果和规范差异；对应 AC-03。
  - [x] r1 复审已完成（2026-11-24，`pass` 绑定 `22f8d7c3d`，范围限单文件面）。
  - [x] r2 复审（AC-06）：覆盖 T-07/T-08 与新提交；重放受影响的 r1 用例。（2026-11-24 `pass`，见 §9）
- [x] T-06 包面有界调查与决策记录（r2）
  - owner_stage=`work`。只读调查 + 探针，不改产品代码。核实 §5 的 (a)(b)(c) 三项假设：容器记录进入绑定后程序的可行通道；旧导入在包面语法下能否表达、入口在哪一层；只用 app 文件的装配是否能绕开 D-1125-01。交 §9 决策记录（入口、触面文件、机制、未决项）。对应 AC-05；若结论是现有机制不可行，返回 `needs_replan`。
- [x] T-07 交付多文件包面的容器类型与旧导入（r2）
  - owner_stage=`work`。依赖 T-06。按决策记录实现，新增真实多模块用例验 V-04/V-05，写 SD-04/05/06；对应 AC-05。
- [x] T-08 关闭复审代码备注 R-1131-08/09（r2）
  - owner_stage=`work`。`mask_legacy_imports` 扫描失败分支返回原源而非空串；源不含 `use auto.` 时提前返回不扫描；各补单测。  - 说明：用户未对这两条单独裁定；r2 默认一并关闭（安全方向），用户可反对。
- [x] T-05 冻结验证与归档（r2 重开）
  - owner_stage=`merge`。执行 §6 审定并集，记录实跑、基线失败、排除及未跑，按现行规范和工作树保护流程收尾；对应 AC-04。

## 9. 复审记录

2026-11-24 · `stage: new` · `PLAN-1131 r1` · `outcome: pass` · `next: work / T-01`。当前为实施前草案，T-01～T-05、AC-01～AC-04 均未执行。

stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: b0413e949 | next: work

授权记录：`/goal 完成计划1131`（2026-11-24）= work → review → 有界修复 → merge 的整链授权（建树、执行、复审、合并、归档、清理）。工作树组 `D:/autostack/.wt/lang-1131/{auto-lang,auto-down}` 由 `scripts/new-wt-group.sh lang-1131 --branch plan-1131-dev` 建立，基面 master@`b0413e949`，auto-down 兄弟树 detached@`456c86a`（未改动）。主检出开工时仅有文档类改动与计划草稿，无代码 WIP。

### T-01 核定记录（2026-11-24）

| 观察点 | 基线事实（`b0413e949`） |
| --- | --- |
| 公共类型来源 | adapter 在源提及 `HashMap`/`HashSet` 时嫁接 `stdlib/ace/asm/map_str.at` 的 `__std_HashMap`/`__std_HashSet`；构造调用有回退，类型注解无回退 |
| 形参/返回/字段注解 | `Bind:source.undefined: no type named HashMap in this module` |
| `var`/`let` 注解 | 不报类型错，接收者静默按 i32 解析（既有洞，非本计划引入；见 D-1131-VAR-LET-ANNOTATION） |
| 同名遮蔽 | 用户 `type HashMap` 在注解与构造上均优先（5） |
| 值形状跨调用 | 值面表 `map_faces` 按解析器局部变量键控，形参是被调函数的新局部；调用点无被调方面信息 |
| 所有权传递 | 持资源记录实参 = 共享引用（被调插入对调用者可见）；`return` 局部容器后调用者可续用，arena 对账平衡 |
| 错值形状 | int / 另一记录 / 另一容器作实参 → `Verify:source.type-mismatch`（注解回退后可判定） |
| 单文件面 `use` | Text 阶段 `source.unsupported: use is not part of this source profile` |
| 多文件包面 | `packages` 装配线无嫁接通道；未准备工作树里现成 `stdlib_asm_972` 13/15 红 = 已知基线红 **D-1125-01**（master `list.at` 私有 `fn` 经 graft `std::list`），非本计划引入、不在本计划交付面 |

### T-03 旧行逐行结果（2026-11-24，原源码未改写）

| 旧行 | 普通入口结果 | 分类 | 等价拼写 / 独立预期 |
| --- | --- | --- | --- |
| 13_collections.014–019（StrBldr ×6） | 运行；`0` / `hello world` / `hello!` / `count: 42` / `5` / `0` | 直接运行，逐字等于原 oracle | 无 |
| 08_strings.005 | 运行；`5` 同，bool 显示 `true/false` | 直接运行 + 展示差异 | int shim → `5\n1\n0` |
| 08_strings.006 | 运行，逐字 `HELLO\nhello` | 直接运行 | 无 |
| 08_strings.007 | 运行；`find`=6 同，bool 显示 `true` | 直接运行 + 展示差异 | int shim → `1\n1\n1\n6`；缺省 -1 |
| 08_strings.008 | 运行，逐字 `hi\nhello rust\nolleh` | 直接运行 | 无 |
| 08_strings.010 | 运行；内建 `str.char_at` 答 char，显示 `h/o` | 直接运行 + 展示差异 | `.to_codepoint()` 与导入的自由函数 → `104\n111` |
| 26.001（split 元素） | `let p1 = parts[1]` → `Bind:source.view-escape` | 已有批准差异（元素复制纪律 D-962） | `parts.get(1).clone()` → 逐字 `RS\nrs\nRS\nx` |
| 26.002 | 运行，逐字 `hello\nworld\nHELLO` | 直接运行 | 无 |
| 28.006（use auto.result） | 运行；bool 显示 `true/false` | 直接运行 + 展示差异 | int shim → `1\n0\n0\n1` |

导入优先级未改动：程序自声明函数 `len`（99）、类型/枚举/函数 `Result`、类型 `StringBuilder` 均优先；非整行旧形态的 `use` 保持原具名拒。

### work 交接就绪记录（2026-11-24）

- **work-owned AC**：AC-01、AC-02 全项通过；AC-03=review、AC-04=merge 未由 work 标记。
- **交付提交**：`ecbec21c5`（plan-1131-dev，基于 `b0413e949`；9 文件 +新增 `builder.at` 与 `plan1131_public_types.rs`）。
- **开放发现/覆盖缺口/未决合同**：无阻塞项。债务候选 3 条见 §10（均不在本计划合同内）。
- **本会话测试**：

| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 合计墙钟 |
| --- | --- | --- | --- | --- | --- | --- |
| `ace-av::conformance plan1131_public_types`（nextest，test-case-batch） | 27 执行 / 27 过 / 0 排除 | 复用已构建二进制 | 未单列 | 0.746s | 无 | 0.746s |
| 同套件 libtest 串行重放 | 27 / 27 | 同上 | 未单列 | 0.83s | 无 | 0.83s |
| `ace-core --lib adapter::`（含新增 4 例） | 13 / 13 | 增量构建，未单列 | 未单列 | 0.66–0.70s | 无 | — |
| 近邻重放 `plan1121` / `plan1120` / `plan1082` / `plan1083` / `plan1079` / `text_semantics` | 26 / 11 / 17 / 6 / 7 / 5，全过 | 同上 | 未单列 | 0.75 / 0.52 / 0.37 / 0.42 / 0.10 / 0.09s | 无 | — |
| 近邻重放 `plan1124_json_csv`（输入未变） | 37 / 37 | 同上 | 未单列 | 155.90s | 无 | 155.90s |

冷构建：conformance 测试二进制首次 `--no-run` 17:12:00→17:13:04（sccache 命中）；其后为 test 目标增量构建，每次约 1–2 分钟，未逐次计量。

- **新增用例行为与计时**（nextest 并行墙钟，含编译与执行的进程内全部准备；门槛 10s）：

| 用例 | 行为与断言 | 实测 |
| --- | --- | --- |
| v01_map_parameter_mutation_is_visible_and_the_caller_continues | 被调把 score 9→11，调用者续读/删除/集合去重，=12121 | 0.377s |
| v01_str_valued_map_crosses_a_parameter_and_a_result | str 面 map 经形参与返回，=604 | 0.390s |
| v01_set_crosses_a_parameter_and_a_result | set 经形参（插入可见）与返回（去重/删除），=20 | 0.274s |
| v01_annotations_bind_locals_fields_and_parameters_to_one_type | `var`/字段/形参注解同一身份，=1117 | 0.274s |
| v02_user_declared_hashmap_wins_in_annotations_and_construction | 用户同名类型优先，=5 | 0.535s |
| v02_wrong_value_shape_at_a_container_parameter_is_named | int/另一记录/另一容器 → `Verify:source.type-mismatch` | 0.524s |
| v02_unified_get_needs_an_observation_in_the_reading_function | 未知面 `source.map-face-unknown`；显式面/同函数观察 =442 | 0.390s |
| v02_conflicting_faces_on_one_parameter_are_named | `source.map-face-mismatch` | 0.200s |
| v02_other_face_entry_from_a_callee_fails_through_the_checked_channel | 被调加异面条目，调用者统一 get 受检失败可 catch，=771 | 0.082s |
| v02_missing_key_through_a_parameter_keeps_the_owner_usable | 缺键后 owner 存活，=11 | 0.252s |
| v02_container_storage_is_released_by_its_last_owner | 被调/调用者两种归属各 6 次分配且对账；失败路径对账 | 0.283s |
| v02_drop_inside_a_callee_is_an_end_of_use_marker | 被调 `drop()` 后调用者仍可读，=3 | 0.223s |
| v03_string_builder_rows_run_with_their_original_source | 六条旧行原源码，结果逐字 | 0.472s |
| v03_string_builder_composes_and_releases_its_parts | 组合 =4410；每次 build 账本差值恒定 | 0.443s |
| v03_user_declared_string_builder_wins | 用户同名类型优先，=7 | 0.412s |
| v03_lower_upper_on_literals_match_the_legacy_oracle | `hello\nworld\nHELLO` | 0.357s |
| v03_methods_on_split_and_lines_elements | 原写法 `source.view-escape`；等价拼写逐字 | 0.603s |
| v03_str_import_basic… | 原源码 `5\ntrue\nfalse`；shim `5\n1\n0` | 0.589s |
| v03_str_import_case… | 逐字 `HELLO\nhello` | 0.496s |
| v03_str_import_search… | 原源码 + shim `1\n1\n1\n6\n-1` | 0.409s |
| v03_str_import_transform… | 逐字 `hi\nhello rust\nolleh` | 0.473s |
| v03_str_import_char_at… | 原源码 `h\no`；码点拼写 `104\n111` | 0.426s |
| v03_result_import… | 原源码 bool 显示；shim `1\n0\n0\n1` | 0.205s |
| v03_result_readers_agree_with_the_is_test_and_keep_the_payload | =1001423 | 0.516s |
| v03_program_declarations_win_over_the_legacy_imports | 99 / 42 / 3 | 0.248s |
| v03_other_use_lines_keep_the_named_rejection | Text `source.unsupported`；未知名 `source.undefined` | 0.531s |
| v03_import_lines_mask_without_moving_the_program | CRLF=5；跨度不变；串内词不是导入 | 0.281s |

- **未跑/待办**：§6 冻结并集 `cargo t ace-source ace-semantics ace-lowering ace-av ace-stdlib case-av-language-parity` **未执行**，归 merge 在冻结提交执行。原因：2026-11-24 17:48 核查机器占用，其他会话正在构建（`cargo build -p auto-ui-tools --features ace-vue --bin auto-ui-tool…`、`ui-119` 的 `p119_diag` 示例），按 2026-11-14 裁定不与他人并跑。本计划不触及 A2X 消费路径（改动限于单文件普通面的 adapter 嫁接与 resolve 注解/方法分派），不选 A2X 集成组。Linux 腿按 1063 交接。
- **健康**：`ace-core` 15 条警告全部为既有（含 1121 遗留的 `owned_str_arg` 多余 `mut`），新增代码零新警告、零调试输出；仓库基线非 rustfmt 干净（ace-core 758 处差异），新增代码按邻近风格书写。

2026-11-24 · stage: work | plan_id: PLAN-1131 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1131/auto-lang | branch: plan-1131-dev | base_commit: b0413e949 | code_commit: ecbec21c5 | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | task_ids: T-01/T-02/T-03 | evidence: 本 §9（原始探针输出在 D:/autostack/plan-artifacts/plan-1131/，merge 时折叠删除） | blockers: 无 | next: review（T-04）；冻结并集待 merge（T-05）

stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: ecbec21c5 | next: review

### 独立复审记录（2026-11-24 · T-04 · AC-03）

stage: review | review_scope: final（本计划未分设终审角色） | reviewer_identity: 独立子代理上下文（与实现会话分离，同一工作树只读；临时探针用后删除） | plan_id: PLAN-1131 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-1131/auto-lang | branch: plan-1131-dev | reviewed_commit: 22f8d7c3d577bcdb8bdbaf57b3c5b631406acc48 | base_commit: b0413e949 | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | spec_inputs（worktree 冻结副本 sha256）：m2-stdlib-core.md `911c94d6…4d98`；ace-collections.md `8b145108…3e66`；language-parity.md `af087286…a531` | acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass（本记录） | pending_closeout_gates: §6 冻结并集（merge） | next: merge（T-05）

**修复循环（3 次，达上限）**：

| 轮 | 被审提交 | 结论 | 阻塞发现 → 修复提交 |
| --- | --- | --- | --- |
| 1 | `ecbec21c5` | needs_fix | R-1131-01 多行字符串内形似导入的行被抹空；R-1131-02 同名枚举注解回归 → `44ee23ee0` |
| 2 | `44ee23ee0` | needs_fix | R-1131-01 未闭合（转义引号翻转状态）→ `f34edffa1` |
| 3 | `f34edffa1` | needs_fix | R-1131-01 未闭合（行注释内引号、字符字面量 `'"'`）→ 诊断：文本启发式不是正确 oracle → 改用语法自带扫描器的字面量范围 → `22f8d7c3d` |
| 4 | `22f8d7c3d` | **pass** | R-1131-01 作为不变量闭合 |

**AC 证据图**：AC-01 pass——`v01_*`/`v02_*` 12 例 + 复审自选反例（main→f→g 调用链 3157、返回的 map 再传 334、字段 map `f(h.m)` 991 且账本 6/6、跨两帧返回 21、递归 11/4、失败路径 13/13 live 0、被调局部 builder 释放 10/10、循环内调用链 39/39）。AC-02 pass——迁移表 diff 恰 14 行且左侧 7 列零差异、558 行均 12 列；13_collections/08_strings/26.002 测试内“原源码”与原文件逐行一致（28.006 仅少首行注释）。AC-03 = 本记录。AC-04 merge 待办。

**非阻塞发现（均已登记债务或已改文档）**：R-1131-03 规范坏链接（已改，含 1121 遗留一处）；R-1131-04 “仅 ASCII”措辞（已改为实测）；R-1131-05 形参容器不可直接返回/存入记录（规范已补，D-1131-PARAM-RETURN）；R-1131-06 `Result.Ok("y")` 被拒（内建 `Ok("y")` 同拒，既有，D-1131-RESULT-STR-PAYLOAD）；R-1131-07 split 用例的原源码为节选（注释已注明）；R-1131-08 `mask_legacy_imports` 扫描失败分支返回空串而非原源（潜在，40 个失败路径探针均未触达——扫描器发 error-leaf token 而非 Err；D-1131-MASK-NOTES）；R-1131-09 每次普通编译无条件扫描（无 `use auto.` 预检；同行 × 字面量比较；未基准测试，视为备注，D-1131-MASK-NOTES）。

**范围解释（项 G，待用户裁定）**：V-01 原文为“普通 AV 多模块 AutoDown Case”“单文件和多模块从各自正常入口观察”；交付落在单文件普通面的同文件跨函数（沿 1121 先例：其 AC 亦写“普通 AV 包”而 26 例同在此入口）。复审意见：披露充分，但属未经批准的收窄，merge 前须交用户认可并把 V-01 文字改到与实际一致。多文件 `packages` 装配线无嫁接通道，且另有基线红 D-1125-01（归因由症状推断，复审未核实）。**未获裁定前不合入。**

**复审计时**（第 4 轮）：`plan1131_public_types` nextest 29/29、单例最长 0.588s；libtest 重放 0.97s；`plan1121` 26（0.71s）、`plan1120` 11（0.51s）、`plan1082` 17（0.37s）、`plan1083` 6（0.43s）、`plan1079` 7（0.10s）、`text_semantics` 5（0.10s）；adapter 单测 13（0.68s）。新增用例累计 29 例（work 27 + 修复轮 2），全部 ≤10s。原始探针输出在 `D:/autostack/plan-artifacts/plan-1131/ac03-review-probe*.txt`、`ac03-rereview{,2,3}-probe*.txt`（merge 时随目录删除）。

stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: ecbec21c5 | next: work
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 44ee23ee0 | next: review
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 5 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 44ee23ee0 | next: work
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: f34edffa1 | next: review
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 7 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: f34edffa1 | next: work
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 8 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 22f8d7c3d | next: review
stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 9 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 22f8d7c3d577bcdb8bdbaf57b3c5b631406acc48 | next: merge（待用户裁定 V-01 口径；冻结并集待机器空闲）

stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 10 | from: merge | to: user | status: reviewed | outcome: waiting | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 22f8d7c3d577bcdb8bdbaf57b3c5b631406acc48 | next: user（两项裁定，见 §10）；冻结并集未执行——2026-11-24 18:26 机器被其他会话占用（`auto-ui-runtime` nextest、plan-1068 `prepare-native`）

stage: goal | plan_id: PLAN-1131 | plan_revision: r1 | round: 11 | from: merge | to: new | status: reviewed | outcome: needs_replan（用户裁定范围变更） | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: 22f8d7c3d577bcdb8bdbaf57b3c5b631406acc48 | next: user（目标暂停中；恢复后先做有界修订，再 work）

### r2 修订记录（2026-11-24）

stage: new | plan_id: PLAN-1131 | plan_revision: r1 → r2 | outcome: pass（可在已记录授权下进入 work） | reason: 用户 2026-11-24 裁定“本计划补齐多文件包面”——经批准的范围变更 | changed: 新增 T-06/T-07/T-08、AC-05/AC-06、V-04/V-05、SD-04/05/06；重开 T-04（r2 部分）、T-05；AC-03 标注范围限单文件面 | retained: T-01～T-03 及其证据、V-01 原文、目标与仓库集 | next: work / T-06

修订要点：r1 的复审 pass 与 3 次修复计数都只对应 r1 的单文件面范围；r2 的复审是针对新增范围的新一轮（修复循环上限按技能默认 3 次重新计）。r1 的冻结并集从未产生结果（准备步骤于 18:57 按裁定终止），r2 在新冻结提交上执行。

**未决项**：R-1131-08/09 的处理用户未单独裁定，r2 默认纳入 T-08 一并关闭；若 T-06 结论为“只用 app 文件的装配绕不开 D-1125-01”，需要用户决定由谁先修（PLAN-118 的 owner 事项）。

### r2 work 记录（2026-11-24 · T-06 / T-07 / T-08）

**T-06 决策记录**

| 假设 | 核实结果 |
| --- | --- |
| 入口 | 普通 AV 的多模块正常入口是 PLAN-1085 的 Auto 包闭包：`bind_packages_auto` / `compile_package_product_auto`（`use app.<模块>.<名>`，同一 Auto 解析器）。此前试的 `compile_assembled_package`（plain 装配线，TextRecord profile）与 `compile_assembled_aggregates_inputs`（并入冻结装配集的聚合线）都不是它。 |
| (a) 容器记录进入绑定后程序 | 可行。该入口已有先例 `graft_package_u64_parse`（把辅助函数并入绑定后的程序）；新增同款 `graft_package_migration_types`。包面的模块支持记录类型：`pub type` 带字段与方法可跨模块使用，入口模块内也可声明。 |
| (b) 旧导入在包面的语法 | `use auto.str: len` 被解析为模块 `auto.str` 的成员导入；`use auto.result`（无成员）需按 `use std.json` 的先例特判为模块导入。 |
| (c) 同一解析器可直接复用 | 基本成立，缺一处：包绑定器对类型注解先做模块符号解析，需把三个容器名当作“内建名义类型”放行（同 `List`/`Iter` 的处理），且必须排在“本模块声明/导入”之后，否则模块同名类型会被遮蔽（实现中发现并修正）。 |
| 与 D-1125-01 的关系 | 不相干：本计划的用例只含工程模块，不并入冻结装配标准库集，不触达 `std::list`。D-1125-01 的成因已核实（`list.at` 私有 `fn join_into`，owner PLAN-118），本计划不修改它。 |

**T-07 交付**（提交 `c7e04b756`）

- `adapter.rs`：`graft_package_migration_types`、`is_migration_type_name`、`legacy_auto_str_module_source`（冻结 `text.at` 改名 + 与单文件面同一组转发函数，公开函数）、`legacy_auto_result_module_source`、`legacy_auto_imports`。
- `packages/binding.rs`：`type_name` 放行容器名（置于模块声明/导入之后）；`use auto.result` 模块导入特判。
- `source-query/.../packages.rs`：`compile_package_product_auto_prepared` 在模块写了旧导入而工程未带同名模块时自动并入冻结模块；并调用新的嫁接。
- 规范 SD-04/05/06（三份规范 PLAN-1131 节增补；`language-parity.md` 中原“多文件包面未观察”一段改为实际交付内容）。

**T-08**：`mask_legacy_imports` 扫描失败分支改为返回原源；源不含 `use auto.` 时直接返回、不扫描。各补单测。

**过程中的失误（如实记录）**：定位阶段我把多个观测值压进一个整数再人工解码，多次读错（如把 312340、21132 误读为异常），一度把一个正确的用例登记成“未关闭缺陷”并写了 `#[ignore]`。改用“程序内逐项比较、返回满足的检查数”后确认：**包面没有产品缺陷**，13 例全绿；该 ignore 用例已删除。另：调试中被否定的假设——“丢弃返回值的方法调用语句在包面不执行”——实测不成立（四种写法均正常执行）。

**本会话测试（r2 work）**

| 测试模块 / 实际范围 | 结果 | 计时 |
| --- | --- | --- |
| `plan1131_multi_module`（新增 13 例，真实多模块） | 13 / 13 | 含在下述并集内 |
| `plan1131_public_types`（r1 29 例，回归） | 29 / 29 | 含在下述并集内 |
| nextest `test(/plan1131_/)`（42 例，test-case-batch） | 42 / 42 | 合计 0.724s，单例均 <1s |
| `ace-core --lib adapter::`（含新增 2 例，共 15） | 15 / 15 | 0.75s |
| 近邻重放 `plan1121` / `plan1120` / `plan1082` / `plan1083` / `plan1079` / `text_semantics` / `plan1124_json_csv` | 26 / 11 / 17 / 6 / 7 / 5 / 37，全过 | 1124 套件约 104–158s（既有套件），其余 <1s |

新增 13 例的行为：跨模块 map 形参+返回（1121）；跨模块 set 共享与去重（110）；三模块链 + 记录字段（1155，账本对账）；跨模块 StringBuilder（1515）；模块边界错形状 `source.type-mismatch`；模块同名类型优先（5）；依赖模块与入口的 `use auto.str`（2114）；`use auto.result` + `Result.Ok/Err` + `is_ok/is_err`（1152）；工程同名 `str` 模块优先（99）；未知模块 `source.module-missing`；无容器对照（42）；单模块多条目 map（21101101）；两键图经被调模块覆盖与删除后五项检查全成立（5）。该面既有边界：`print` 需要宿主（用例不依赖）；非入口函数的 `!int` 形参不被接纳。

**未跑/待办**：§6 冻结并集仍未执行（归 merge 在新冻结提交上执行）。Linux 腿按 1063 交接。

stage: work | plan_id: PLAN-1131 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1131/auto-lang | branch: plan-1131-dev | base_commit: b0413e949 | code_commit: c7e04b756 | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | task_ids: T-06/T-07/T-08 | evidence: 本 §9 | blockers: 无 | next: review（T-04 的 r2 部分，AC-06）；冻结并集待 merge（T-05）

stage: goal | plan_id: PLAN-1131 | plan_revision: r2 | round: 12 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: c7e04b756 | next: review

### r2 独立复审记录（2026-11-24 · T-04 的 r2 部分 · AC-06）

stage: review | review_scope: final | reviewer_identity: 独立子代理上下文（与实现会话及 r1 复审者均分离；同一工作树只读；临时探针用后删除） | plan_id: PLAN-1131 | plan_revision: r2 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-1131/auto-lang | branch: plan-1131-dev | reviewed_commit: c7e04b756d91154a9233400924ba319ee6f5d404（r2 基 `22f8d7c3d`） | base_commit: b0413e949 | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | acceptance_results: AC-05 pass、AC-06 pass、T-08 pass（R-1131-09 有运行证据；R-1131-08 的扫描失败分支按代码核对，复审者 11 组破损源均无法让扫描失败，无运行证据） | pending_closeout_gates: §6 冻结并集（merge） | next: merge（T-05）

**证据**：复审者自选约 30 个多模块程序（所有权/账本、实例隔离、遮挡矩阵、旧导入矩阵、T-08、单文件面回归抽查），r2 合同内无缺陷；r2 合同内的“意外结果”均为探针自身错误或合同外的既有行为。

**非阻塞发现与处理**：

| ID | 发现 | 处理 |
| --- | --- | --- |
| R2-1131-01 | 计划 V-04 仍写旧入口名 `compile_assembled_package` | 已改正本文件 §3/§5/§6 |
| R2-1131-02 | “工程同名模块优先”只对 `auto` 包的同名模块成立；原同名用例并未测该规则 | 已补永久用例（`auto::str` 99、`auto::result` 7），原用例改标为普通绑定对照；规范措辞改准 → `3deeecfdd` |
| R2-1131-03 | “另一实例不受影响”无永久用例 | 已补 `v04_two_instances_stay_isolated_through_one_callee`（账本对账）→ `3deeecfdd` |
| R2-1131-04 | `legacy_auto_imports` 按行文本匹配，多行字符串内或 `use auto.string` 也命中；后果只是多并入一个无人使用的冻结模块 | 不改，债务 D-1131-PKG-IMPORT-DETECT |
| R2-1131-05 | 跨模块类型冲突诊断不报类型名；`use auto.str`（无成员）报的模块名不准 | 不改，债务 D-1131-PKG-DIAGNOSTICS |

**合同外观察（既有行为，未改）**：`fail(...)` 之后仍有语句报 `verify.block-structure`；函数别名 `as` 在该面不支持（类型别名可用）；`LEGACY_RESULT_MODULE` 作为裸名不可达。

**针对性确认**：`3deeecfdd` `pass`（仅三个文件：多模块用例 + 两份规范；产品代码未变；`plan1131_multi_module` 15/15、`plan1131_public_types` 29/29、adapter 单测 15/15；nextest `test(/plan1131_/)` 44/44、合计 0.764s、单例最长 0.617s）。`138bc91cc`（仅两份规范里的“13 例”→“15 例”）复审者已确认 `pass`（差异仅两个数字，工作树干净），规范 sha256：`m2-stdlib-core.md` `6e50581b…61f2f`、`language-parity.md` `101f20c4…5ef2f`、`ace-collections.md` `991d82eb…095d`（未变）。

stage: goal | plan_id: PLAN-1131 | plan_revision: r2 | round: 13 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1131/auto-lang | commit: c7e04b756 | next: merge（T-05；冻结并集待机器空闲）
### merge 收尾记录（2026-11-24 · T-05 · AC-04）

**PLAN-1131:r2 · prepared** — 复审基线 `138bc91cc`（r2 复审 `pass` 绑定 `c7e04b756`，两次针对性确认至 `138bc91cc`）；规范增量 SD-01～SD-06（三份规范，worktree 冻结副本 sha256：`m2-stdlib-core.md` `6e50581ba972dba8…`、`language-parity.md` `101f20c49ff5f8cee…`、`ace-collections.md` `991d82eb742d437f…`）；交付提交见 `landed`。

**冻结固定组并集**（§6，在冻结提交 `138bc91cc` 上执行；准备 19:57:19→20:09:19 共 12 分 00 秒，退出码 0；执行 20:14:42→20:18:45 共 4 分 03 秒，退出码 1）：

| 组 | 结果 | 执行数 | 归因（均有直接或独立证据） |
| --- | --- | --- | --- |
| ace-semantics | PASS | 295 | — |
| ace-lowering | PASS | 9 | — |
| ace-stdlib | PASS | — | — |
| ace-source | 红 5 | 1035（1030 过 / 5 败 / 1 跳过） | `text_record_source` 五例；**基面逐名对照一致** |
| ace-av | 红 46 | 1749 | **基面逐名对照一致**：37 条属 D-1125-01 族（`stdlib_asm_972`×13、`stdlib_sys_973`×11、`bstr_1064`×4、`kv_zip_1064::v03_*`×4、`sys_d01_d06_1064`×3、`probe972`×1、`v10_layers_972`×1），9 条为继承基线（`av_owned_first_case`、`av_owned_recipe`、`av_test_batch`、`component_event_sessions_1106`、`resource_915`×3、`layout::tests`） |
| case-av-language-parity | 红 2 / 错误 2 | 574（570 过 / 214 未选 / 161 跳过） | 2 红 = D-1129-12（`av.parity.1082.empty-array-rejects`、`av.parity.1084.boxed-nil-arithmetic-script-catch`）；2 错误 = D-1129-04（`py.slicing.1127.numpy-torch`、`py.float-args.1129.torch-lr-computed`，torch 冷缓存导入超过墙钟预算） |

**基面对照（直接证据）**：在本计划工作树组内建基面提交 `b0413e949` 的临时 detached 工作树，编译同样的测试二进制（20:21:58→20:23:22，84 秒），只重跑 `ace-source`/`ace-av` 的 51 个失败用例所在的测试模块（nextest 选中 70 例：19 过 / 51 败，0.742 秒）。与并集失败名单逐名比对：**失败集合完全相同；没有任何一条在基面上通过；基面也没有名单之外的失败**——`ace-source` 与 `ace-av` 零新增红。

**parity 组的证据**：该组在空闲门控下重跑一次（20:27:31→20:29:28），结果与首次相同（2 红 / 2 错误），故 2 条 torch 错误不能归因于并发。未做本计划自己的基面 parity 对照（基面工具准备一次因 rustc 进程异常退出失败，重试已主动终止）：2 红依据 D-1129-12——PLAN-1129 的 merge 门在 master 上发现并做过 master-only 重放，失败消息与本记录逐字相同；2 错误依据 D-1129-04——PLAN-1129 复审登记为 torch 冷缓存墙钟预算的间歇性问题，并写明 1127 的 torch 用例同样暴露。两者均为今日其他计划登记的未关闭债务，早于本计划。

**本计划的用例**：44 例（`plan1131_public_types` 29 + `plan1131_multi_module` 15）均在 `ace-av` 组内，无一出现在失败名单。

**并发与环境如实记录**：首次启动并集时（20:12）与界面线会话的测试撞车，按裁定立即终止并作废（只终止本计划自己的进程）；第二次在确认空闲的同一瞬间启动（20:14:42），10 秒后界面线会话又开始测试与编译，并集全程与之并行。各组 nextest 单组计时只有 `ace-source` 在日志里可见（14.161 秒），其余组的单组计时未单独观测；parity 的 2 条 torch 错误经空闲重跑排除并发因素。

**landed**：master 自基面前进 10 个提交（全部只改 PLAN-1068 的计划文件，无代码/规范/配置变化）。在计划工作树把 `plan-1131-dev` 变基到 master，`git range-diff b0413e949..138bc91cc master..HEAD` 七个补丁全部相等，且变基后的树在 `docs/plans` 之外与 `138bc91cc` 无差异。主检出 `git merge --ff-only plan-1131-dev` → 交付提交 **`c361477bf`**（无合并提交）。新旧映射：`ecbec21c5→8869f6799`、`44ee23ee0→252b0170d`、`f34edffa1→815c87996`、`22f8d7c3d→d36b692af`、`c7e04b756→8d3b4bb27`、`3deeecfdd→f667627f7`、`138bc91cc→c361477bf`。落地后另一会话（plan-119）又在 master 上提交了若干提交；`c361477bf` 在 master 历史中。落地冒烟复用同一棵树的证据（nextest `test(/plan1131_/)` 44/44）：主检出带有其他会话未提交的代码改动（`experimental/ace/aac/**` 等），未在主检出上另行构建。

**canonical_verified**：master 上三份规范的 blob 与 `138bc91cc` 完全一致（`09d4050e6a15`、`102c63f815c1`、`c96adc5b2cd9`），sha256 前缀 `6e50581b` / `101f20c4` / `991d82eb` 与复审记录吻合。`stdlib/ace/asm/builder.at`（sha256 前缀 `34d8987e`）在库；它不属于 972 装配集（`stdlib_assembled_files()` 不列）。

stage: merge | plan_id: PLAN-1131 | plan_revision: r2 | outcome: pass | delivery_commit: c361477bf | canonical_specs: docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/stdlib/design/ace-collections.md, docs/specs/auto-av/language-parity.md | archive_path: docs/plans/archive/1131-av-stdlib-public-types-and-legacy-imports.md | cleanup: 见下

**部署观察**：本计划交付物是编译器/标准库源码与测试，无生产面制品在本范围内；未触发 release 构建或重启——记为 stale-artifact 观察（无受影响制品）。

**批量回归到期核**：待 `cleaned` 后核对 `docs/plans/.last-batch-regression.json`。
## 10. 待澄清事项

公开类型的绑定机制已由 T-01 核定（注解回退到嫁接记录，同名用户声明优先），未改动既有类型或导入优先级，无需实施决策。

债务候选（复审登记，均不阻塞本计划）：

- **D-1131-CROSS-FN-FACE**：调用点不做静态值面核对。被调方可向调用者的 map 加入另一面条目；静态不可判定（值面按函数各自观察），运行时面检查兜底（统一 `get` 走受检 ValueError）。关闭条件 = 签名可表达值面或两遍解析后在调用点具名拒。
- **D-1131-VAR-LET-ANNOTATION**：`var x 未知类型 = …`/`let` 在普通 Auto 面静默通过（注解仅作推断期望，诊断被丢弃），早于本计划。关闭条件 = 未知记录类型在局部注解位具名拒，且既有语料无回归。
- **D-1131-PACKAGES-LANE**：多文件 `packages` 装配线没有 HashMap/HashSet/StringBuilder 与旧 `auto.*` 导入的嫁接通道；该线当前另有基线红 D-1125-01。关闭条件 = 该线自己的容器/导入切片。

### 等待用户裁定（2026-11-24，merge 前置）

复审已 `pass`（绑定 `22f8d7c3d`），但以下两项是用户的决定，未获答复前**不合并**（已向用户提问一次，未得到选择；等待轮 1/3）：

1. **V-01 的“多模块”口径**（复审项 G）。原文“普通 AV 多模块 AutoDown Case / 单文件和多模块从各自正常入口观察”，交付为单文件普通面的同文件跨函数（沿 1121 先例）。建议：按单文件口径接受，V-01 文字改到与实际一致，多文件包面由 D-1131-PACKAGES-LANE 承接；另一选择是要求本计划补齐多文件包面（需修订范围与预算）。
2. **两条非阻塞代码备注**（R-1131-08 扫描失败分支返回空串而非原源，潜在且实测不可达；R-1131-09 无 `use auto.` 预检）。修复循环已达 3 次上限。建议：原样合并，保留 D-1131-MASK-NOTES；另一选择是先修再合并（改变已复审提交，需额外一轮针对性复审，须用户豁免上限）。

**恢复路径**（同一目标可续）：得到裁定后，在机器空闲时于冻结提交 `22f8d7c3d` 执行 `cargo t ace-source ace-semantics ace-lowering ace-av ace-stdlib case-av-language-parity`（若裁定为“先修”则在新提交上执行），核对规范回写，再按 merge 流程合入、归档、`wt-guard.ps1` 后清理工作树与 `plan-artifacts/plan-1131`。

### 用户裁定（2026-11-24）：本计划补齐多文件包面

对“V-01 的多模块口径”用户裁定：**本计划补齐多文件包**，不按单文件口径接受。这是经用户批准的范围变更（复审项 G 的第二种处理），当前 `reviewed` 状态与 `22f8d7c3d` 上的复审 pass **只覆盖单文件面的交付**，不构成合并许可。

修订后要交付的内容（尚待 `/auto-plan:new` 写成有界修订，目标恢复后执行）：

- 多文件 `packages` 装配线上的公共容器类型：`HashMap`/`HashSet`（及 `StringBuilder`）的构造与形参/返回/字段注解跨**真实多个模块文件**（`use pkg.module.symbol`）绑定到同一受检类型；被调模块对容器的修改对调用模块可见。
- 同一面上的旧导入：`use auto.str: …` / `use auto.result` 的等价入口。
- 先核清该线的既有红 D-1125-01（本计划此前只是按症状推断归因，未经核实）：它是否挡住本计划要加的用例；若挡住，需要决定由谁先修。
- V-01/V-02 增加真实多模块用例；V-01 的“多模块 AutoDown Case”文字保持原义，不再改写。
- 冻结并集在新冻结提交上重跑（之前在 `22f8d7c3d` 上的准备于 18:57 按本裁定终止，没有产生任何并集结果）。

未裁定的第二项（R-1131-08/09 两条代码备注原样合并还是先修）顺延到修订时一并处理；因范围变更会有新提交，原先“3 次修复上限”的约束如何适用也需在修订中明确。

主检出上本计划文件与 `KNOWN-DEBT-AND-RISKS.md` 的改动尚未提交。
