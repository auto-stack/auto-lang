---
plan_id: PLAN-627
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: qualified-api-module-calls
author: [zcode-session]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [ui/api-qualified-calls]
touched_goals: []

affects: [auto-lang/ui, auto-lang/interpreter]
current_step: 0
total_steps: 5
---

# [PLAN-627] qualified-api-module-calls

## 变更摘要

补齐 `use back.api` **模块形态**（无符号清单）+ **限定名调用**（`api.X()`）
在 a2r（rust 轨）与 vue 轨生成器的支持。VM/desktop 动态编译路径对限定名
调用**已工作**（`vm/codegen.rs` 明示 qualified 跳过 HTTP/桩拦截直落本地
字节码——PLAN-053 拦截仅匹配裸名 `Expr::Ident`），本计划零 VM 改动，
只补两个发射器的对称臂。

**动因（auto-term PLAN-017 T-02 实证，2026-09-14）**：裸导入形态在
merged VM/桌面动态编译下 `#[api]` 调用进 PLAN-053 no-op 桩（端到端死），
唯一进程内可达形态 = 限定名；但 a2r 对限定名报 E0425（`api` 未定义）、
vue 生成器 `api.ts` 只产裸函数（无 `api` 命名空间对象）。修复后
AutoTerm 统一工程入口（auto-term `app/`）四装载面（rust/vm/vue/桌面）
单源全绿。

## 目标

1. `use back.api`（模块形态，无 items）在 rust/vue 生成器等价于
   符号导入形态：api 客户端/桩照常生成（函数清单自 `src/back/api.at`
   `#[api]` 声明枚举——复用既有 api.at 解析）。
2. 前端 handler 内限定名调用 `api.X(...)` 在两发射器正确改写：
   - vue：`X()` 客户端调用（与裸名同发射）；
   - a2r merged：直调生成桩/吸收 fn（与裸名同发射）。
3. 裸名形态行为零变化（既有 corpus/金样不动）。

### 非目标

- VM/桌面动态编译路径改动（已工作，禁止触碰）。
- 裸名 merged no-op 桩语义（PLAN-053 现状维持；plan622 已钉守卫）。
- 新语法/新 pac 键。

## 架构方案

两发射器各补两处（对称四点）：

**抽取侧**：`extract_api_imports_from_ast`（`ui_gen/api.rs:971` 与
`auto-man/rust_ui.rs:541` 同构双写）现仅收 `use back.api: a, b` 的 items；
模块形态 items 空 → 清单空 → 客户端/桩不生成。补：模块形态时枚举
`src/back/api.at`（经既有 `resolve_back_api` 定位契约文件 + 既有
`#[api]` fn 解析——vue 侧 `api_gen` 已有同型枚举）产出函数名清单。

**发射侧**：
- vue：`ui_gen/vue.rs` 表达式发射对 call head `api.<name>` 且
  `<name>` ∈ api 清单 → 按裸名同路径发射客户端调用（store facade 的
  qualified-head 路由（api.rs Plan 559 W2）同族先例）。
- rust：`ui_gen/rust.rs` handler 表达式发射对 call head `api.<name>`
  且 ∈ 清单 → 发射裸名调用（命中文件级生成的桩/吸收 fn）。

## 需求分析与背景调查

- 017 矩阵实测（auto-term tmp 探针，已清理）：裸名 rust✓/vue✓/vm-split✓/
  vm-merged✗桩/桌面✗桩；限定名 vm-merged✓（几何 84×29、光标 (3,15)、
  29 行收割全实证）/rust✗E0425×13/vue✗（`api` 无定义）。
- 拦截语义：`vm/codegen.rs:8171-8211`——api_over_http（split）改写 HTTP、
  host_bridge（ash/cdylib）改写 auto.host.call、否则 PLAN-053 桩；
  三臂均 `matches!(call.name, Expr::Ident)` 仅裸名。qualified 直落
  `api.<fn>` 模块成员解析（merged = 本地字节码委托体）。
- B 分支否证（017 §10.1）：cdylib 路线因 `auto_lang::ui::terminal`
  静态注册表（pump/resize）与宿主分裂而桌面键入死——本计划是唯一
  可行的单源路径。
- 授权记录：auto-term PLAN-017 会话内用户裁定方向（限定名 + 跨仓补齐）
  ——2026-09-14 AskUserQuestion 未响应，按自主规约最佳判断推进，回滚
  路径（revert auto-term c157c80）在 017 §10.1 在案。

## 详细设计

（四点对称改动；文件行号为 2026-09-14 master 2c92f3755 锚）

| # | 文件 | 改动 |
|---|---|---|
| 1 | `crates/auto-lang/src/ui_gen/api.rs` `extract_api_imports_from_ast` | 模块形态分支：items 空 → 解析 api.at 枚举 `#[api]` fn 名 |
| 2 | `crates/auto-man/src/rust_ui.rs` `extract_api_imports_from_ast` | 同 #1（双写纪律） |
| 3 | `crates/auto-lang/src/ui_gen/vue.rs` 表达式发射 | qualified head `api.X` ∈ 清单 → 裸名同发射 |
| 4 | `crates/auto-lang/src/ui_gen/rust.rs` handler 表达式发射 | 同 #3 |

api.at 枚举实现复用既有解析（`auto-man/src/api_gen.rs` 读契约的
`#[api]` 收集逻辑；或 ast 级 walk），不新写第二套。

## 测试设计

- 单测：抽取侧模块形态（fixture：模块形态 use + api.at 契约）→ 清单
  等价符号形态；发射侧限定名 → 输出与裸名形态**逐字节一致**（金样对拍）。
- 端到端：auto-term `app/` 切限定名后 `auto build -r rust` 绿 +
  `auto run -r vue` 页起（017 侧 T-05 复验，跨仓证据）。
- 既有面：裸名 corpus/金样零 diff（构造性——改动仅新增分支）。

## 验收标准

- **AC-1** 模块形态抽取：模块形态 `use back.api` 产出的函数清单 ==
  符号形态（fixture 单测）。
- **AC-2** vue 限定名发射：`api.X()` 产物与裸名 `X()` 逐字节一致
  （金样对拍单测）。
- **AC-3** a2r 限定名发射：同 AC-2（rust 侧）。
- **AC-4** 裸名零回归：既有 a2ts/rust 金样与 corpus 测试全绿。
- **AC-5** 跨仓端到端：auto-term 限定名形态 rust 构建绿 + vue 起页
  + vm-merged 实机（既有 017 实证形态复验）。

## 执行步骤

- **T-1** 抽取侧双写（#1/#2）+ fixture 单测（AC-1）。
- **T-2** vue 发射臂（#3）+ 金样对拍单测（AC-2）。
- **T-3** a2r 发射臂（#4）+ 金样对拍单测（AC-3）。
- **T-4** 门档：`cargo check -p auto-lang` + 作用域测试（发射面金样
  模块）+ tv 若触及 vm 语料（预期不触）。AC-4。
- **T-5** 跨仓端到端（auto-term app/ 切限定名，三轨+桌面复验）。AC-5。
  （T-5 在 auto-term 017 会话执行，证据回链本计划。）

## 复审记录

## 待澄清事项
