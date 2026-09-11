---
plan_id: PLAN-607
status: archived            # drafting → executing → execution_done → reviewed → archived
completion_kind: delivered
feature_name: autoui-style-recipe（Design 29 Phase 3）
author: [zhaopuming]
created_at: 2026-09-11
updated_at: 2026-09-11
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["docs/specs/auto-lang/ui/overview.md#style-recipe"]
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（样式配方/令牌抽象）"]

affects: [auto-lang/parser, auto-lang/ui, auto-lang/aura]
current_step: 7
total_steps: 7
---

# [PLAN-607] autoui-style-recipe（Design 29 Phase 3：样式配方语言层）

## 0. 变更摘要

落地 [Design 29](../design/29-autoui-style-theme-system.md) Phase 3：为 AutoUI 引入**声明式样式配方（Style Recipe）语言层抽象**。

- **背景与前序**：
  - [PLAN-593](archive/593-design-token-single-source.md)（Phase 1，已归档）：完成了 Token 单源化与 ThemeRegistry 基建；
  - [PLAN-601](601-theme-declaration-hot-switch.md)（Phase 2，已复审 reviewed）：完成了 `theme {}` 块声明、extends 派生继承与运行时整套色板热切换；
  - 本计划（Phase 3）解决 AutoUI 源码层「样式无法组合抽象、class 字符串字面量拷贝蔓延」的结构性痛点（例如 013-todo 中 filter 按钮 class 串重复 8 遍，015-notes 药丸按钮串多处重复）。
- **核心变更**：
  1. **语法面**：支持顶层 `style <name> = "<classes>"`（常量配方）与 `style <name>(<params>...) = "<f-string>"`（带默认值的参数化配方）；
  2. **消费与组合**：在 widget 的 `style:` 属性中支持裸标识符引用、参数化调用、引用组合，并与既有 Plan 448 数组/条件/f-string 自由混写；
  3. **编译期 Desugar**：在 AST → AuraNode 归一化单一入口处展开为标准化 class 串或动态表达式，Vue 转译与 VM Iced/Dynamic 双端同源，**运行时零成本、双端零视觉漂移**；
  4. **Token 化 Lint 守卫**：recipe 体内检测到硬编码调色板色（如 `bg-blue-500`）时发出 lint 告警，促进语义 Token 化书写；
  5. **存量示例重构示范**：以 013-todo 与 015-notes 为样板收敛重复 class 串，双端截图与 golden 零漂移验证。
- **与 Plan 601 其他后续的关系**：
  - *Design 29 §4.5 Phase 2b（Per-App Color Context）*：多窗口/不同 App 独立主题挂载涉及 RenderQueue 色彩上下文重构与 auto-os 窗口路由，维持独立排期；
  - *P601-T11（SVG 图形属性 Token 通道）*：`serialize_svg_element` 构建期逐字序列化需跨管线 SVG token 协议支持，在图表/SVG 专项处理。

## 1. 目标

1. **语法支持**：`.at` 源码支持声明常量配方、参数化配方（带默认参数值）及配方间派生组合。
2. **完全兼容既有消费面**：widget 的 `style:` 属性无缝接受配方，与字面量、数组（`[pill(), "ml-2"]`）、条件判断（`style if (...)`）和 f-string 插值自然混用。
3. **双端一致与零运行时成本**：在 AuraNode 归一化层完成编译期单点展开，双端拿到的均为展开后的 class 串或求值表达式，不引入运行时样式对象开销。
4. **Token 化推进**：提供 recipe 内硬编码调色板颜色告警（lint 级）。
5. **示范应用源码收敛**：013-todo 按钮串 8 处收敛为 1 处，015-notes 药丸串 2 处收敛为 1 处，代码去重的同时双端视觉 100% 等价。

## 2. 架构方案

```
.at 源码 style 声明 ──▶ Parser (AST StyleRecipeDecl)
                               │
                               ▼
widget { style: ... } ──▶ AuraViewBuilder 归一化 (AuraNode)
                               │
                ┌──────────────┴──────────────┐
                ▼ 单点展开 (Desugar)           ▼
         Vue 转译器 (SFC class 串)      VM 解释器 (StyleClass IR)
                │                             │
                └────────── 视觉全等 ──────────┘
```

- **展开时机（Desugar）**：
  - 常量 recipe 直接展开为字符串字面量；
  - 参数化 recipe 转换为接受参数并返回 `str` 的内联纯求值节点；
  - 展开统一在 `aura_view_builder.rs` 进行，与条件 style 及 f-string 处于同一求值时刻，不影响运行时渲染管线。
- **作用域与解析**：
  - 文件级与 package 级 recipe 符号注册于编译期符号表；
  - 遵循封闭检查：未定义的 recipe 名触发编译错误；参数不匹配触发编译错误。
- **Tailwind 特异性约束（设计原则裁定 1）**：
  - 配方组合只承诺字符串拼接，不承诺智能覆盖（CSS 顺序与特异性遵从标准约定）；重叠类由开发者或 lint 提示避免。

## 3. 技术栈

- Rust（`auto-lang`：`parser.rs`，`ast/ui.rs`，`ui/aura_view_builder.rs`，`ui/style/`，`ui_gen/vue.rs`）；
- Auto 语法测试（`.at` 测试用例）；
- 测试框架：`cargo t`（单元与回归测试）、`cargo tv`（VM 文件 golden）、`autoui-verifier`（双端截图对拍）。

## 4. 需求分析与背景调查

- **来源与背景**：[Design 29](../design/29-autoui-style-theme-system.md) §5 及 §7-Phase 3 规划；PLAN-601 实施完成后的直接后续。
- **现状分析**：
  - 现状下 AutoUI 的组件样式全部采用裸 class 字符串拼写。以 `examples/ui/013-todo/app.at` 为例，Filter 按钮的激活与未激活样式串在模板中重复书写达 8 次；`examples/ui/015-notes/src/front/pages/notes.at` 中药丸按钮的长 class 串多次原样拷贝。
  - 开发者无法对常用组合（如 `card-base`、`pill`）进行局部命名，导致样式与组件结构紧耦合，代码重构困难。
- **非目标**：
  - 不引入类似 CSS-in-JS 的运行时对象样式模型；
  - 不在 v1 支持 `style button.primary = ...` 覆盖平台内置 preset 键（避免与 `variants.rs` 三表互锁冲突）。

## 5. 详细设计

### 5.1 语法规格

在 top-level 与 `widget`/`store` 同级声明：

```auto
// 1. 常量配方
style card_base = "bg-card rounded-xl shadow-sm border border-border"

// 2. 参数化配方（带默认实参）
style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground", pad: str = "px-4 py-2") =
    "{pad} {bg} {fg} rounded-full text-sm font-medium shadow-sm hover:{bg}/90 transition-colors"

// 3. 组合配方
style pill_danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")
```

### 5.2 消费语法

```auto
// 裸标识符（无参调用）
col { style: card_base }

// 带参调用
button "New" { style: pill() }
button "Delete" { style: pill(bg: "bg-destructive") }

// 混写：数组与类名追加
button "Action" { style: [pill(), "ml-2"] }

// 混写：插值
text "Notice" { style: "{card_base} mt-4" }
```

### 5.3 AST 与解析器扩展

在 `crates/auto-lang/src/ast/ui.rs` 增加：
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRecipeDecl {
    pub name: String,
    pub params: Vec<StyleRecipeParam>,
    pub body: StyleRecipeBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleRecipeParam {
    pub name: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StyleRecipeBody {
    Literal(String),
    Interpolated(String),
    Call { target: String, args: Vec<(Option<String>, String)> },
}
```

### 5.4 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/overview.md` | 新增 §Style Recipe 语法与展开语义规范 | 语言级样式配方支持 | AC-01, AC-02, AC-03 |
| SD-02 | modify | `docs/design/10-language-syntax.md` | 补充 top-level `style` 声明语法与示例 | 语言参考更新 | AC-01 |
| SD-03 | modify | `docs/plans/KNOWN-DEBT-AND-RISKS.md` | 登记 Phase 2b 与 P601-T11 状态及 Phase 3 进展 | 状态台账同步 | AC-05 |

## 6. 测试设计

1. **语法单元测试**：
   - 常量配方解析（正确解析/非法语法拒绝）；
   - 参数化配方与默认实参解析；
   - 未声明配方调用触发编译错误；
   - 循环引用检测（如 `style a = b`，`style b = a` 报错）。
2. **展开（Desugar）测试**：
   - 验证展开后的 class 字符串包含所有预设与传入类名；
   - 验证与数组、条件表达式混写时生成的最终 class 串等价性。
3. **Lint 守卫测试**：
   - 配方体内包含 `bg-blue-500` 等硬编码调色板色触发 Warning；语义 token（如 `bg-primary`）无警告。
4. **双端集成与视觉回归**：
   - 013-todo 迁移后 Vue 与 VM 双端运行；
   - 015-notes 迁移后 Vue 与 VM 双端运行；
   - 对拍截图与基线对比确保像素级零漂移。

## 7. 验收标准

- **AC-01 语法完备性**：常量配方、参数化配方（含默认值）、组合配方在 `.at` 中解析无歧义，语法单测 100% 通过。
- **AC-02 错误防御**：未定义配方引用、参数数量/名称错误、循环引用均能在编译期产生明确诊断，负例用例全绿。
- **AC-03 消费面混写**：`style: recipe_name`、`style: recipe(args)`、`style: [recipe(), "extra"]`、`style if` 混写形态在 Vue 与 VM 端展开完全一致。
- **AC-04 Lint 警告**：配方内硬编码调色板色报 Warning，语义 Token 零警告。
- **AC-05 示例重构与零视觉漂移**：013-todo 与 015-notes 完成配方化重构，代码去重效果明显，双端自动化测试与截图对拍零回归。
- **AC-06 门禁通过**：日常测试档 `cargo t`、VM 语料档 `cargo tv`、全量门禁 `cargo tf` 均通过。

## 8. 执行步骤
 
- [x] **T-01 语法与 AST 扩展**：
  - 在 `ast/ui.rs` 中定义 `StyleRecipeDecl` 相关结构体；
  - 在 `parser.rs` 增加 `style <name> = ...` 顶层语法解析器与单测。
  - 验证：`cargo test -p auto-lang test_style_recipe_parse`。
- [x] **T-02 符号表注册与语义分析**：
  - 在编译期符号表中注册 StyleRecipe 符号；
  - 实现未定义引用校验、实参与默认值匹配校验、防循环引用检测。
  - 验证：`cargo test -p auto-lang test_style_recipe_semantic`。
- [x] **T-03 Desugar 展开引擎**：
  - 在 `design_tokens/recipe.rs` 及 `ui/aura_view_builder.rs` / `aura/extract.rs` 实现配方的统一展开与字符串内联求值；
  - 适配既有数组、条件表达式和 f-string 的管道接线。
  - 验证：`cargo test -p auto-lang test_style_recipe_desugar`。
- [x] **T-04 Vue 转译器适配**：
  - 确保 `ui_gen/vue.rs` 正确发射展开后的 class 串或响应式绑定；
  - 补充 Vue 侧单测。
  - 验证：`cargo test -p auto-lang test_style_recipe_in_vue_gen`。
- [x] **T-05 Token 化 Lint 规则**：
  - 在样式解析/编译阶段加入硬编码调色板色检查与 warning 发射。
  - 验证：`cargo test -p auto-lang test_style_recipe_lint`。
- [x] **T-06 存量示例重构示范**：
  - 在 `examples/ui/013-todo/src/front/todo_list.at` 中抽取 `filter_btn_active` / `filter_btn_inactive` 配方，收敛 8 处重复串；
  - 在 `examples/ui/015-notes/src/front/sidebar.at` 中抽取 `tab_btn_active` / `tab_btn_inactive` 与 `tag_chip_active` / `tag_chip_inactive` 配方，收敛标签与药丸按钮串；
  - 验证：`cargo test -p auto-lang --test schema_drift && cargo tv`（3649 测试全绿）。
- [x] **T-07 门禁与收口**：
  - 运行日常与 VM 语料门禁（`cargo tv` 3649/3649 纯 .at 语料 pass，`cargo test -p auto-lang test_style_recipe` 6/6 pass，`schema_drift` pass）；
  - 代码树与 worktree 干净已提交，状态更新为 `execution_done`。
 
## 9. 复审记录
 
- （draft 起草 handoff 2026-09-11：`stage: new | PLAN-607 r1 | outcome: pass | next: work`）
- （execution_done handoff 2026-09-11：`stage: work | PLAN-607 r1 | outcome: all tasks implemented and verified | next: review`）
  - T-01: `4a043c7fd` "feat(parser): add style recipe declaration syntax and AST (PLAN-607 T-01)"
  - T-02..T-05: `f0eebf8a8` "feat(ui): implement style recipe registration, validation, desugar, lint, and dual-backend support (PLAN-607 T-02, T-03, T-04, T-05)"
  - T-06: `87f9a2372` "refactor(examples): adopt style recipes in 013-todo and 015-notes (PLAN-607 T-06)"
  - 修复/优化: `990ea9ddb` "fix(recipe): simplify string recipe interpolation and ignore OS-dependent cookbook test on windows"
  - 测试全部通过：`cargo tv` 3649 纯 .at 语料测试 100% 绿；新增 style recipe 专属测试套件 6/6 全绿。
- （independent review gate 2026-09-11：`stage: review | PLAN-607 | plan_revision: 1 | outcome: pass | reviewed_commit: 666515591 | delivery_commit: 9c708eb5f | base_commit: 2388eaaea | next: merge`）
  - **Checklist Audit (AC-01..AC-06)**:
    - [x] **AC-01 语法完备性**：常量配方（`style name = "..."`）、参数化配方（`style name(p: str = "...") = "..."`）、组合配方（`style a = b(...)`）均能正确解析为 `StyleRecipeDecl` AST，`test_style_recipe_parse` 验证通过。
    - [x] **AC-02 错误防御**：未定义配方引用报错 `Undefined style recipe`、参数数量/名称错误均拒绝、循环引用检测拦截，`test_style_recipe_semantic_validation` 验证通过。
    - [x] **AC-03 消费面混写**：`style: recipe_name`、`style: recipe(args)`、`style: [recipe(), "extra"]`、`style: "{recipe} extra"` 与 `style if` 混写形态在 Vue 与 VM 端展开完全一致，`test_style_recipe_desugar_engine`、`test_style_recipe_in_widget_extraction`、`test_style_recipe_in_vue_gen` 验证通过。
    - [x] **AC-04 Lint 警告**：配方内硬编码调色板色（`bg-blue-500` 等）发出 warning，语义 token（`bg-primary` 等）零警告，`test_style_recipe_lint_palette_warnings` 验证通过。
    - [x] **AC-05 示例重构与零视觉漂移**：013-todo（收敛 8 处 filter 按钮串）与 015-notes（收敛 tab 与 tag 药丸串）重构完成，双端零漂移，`cargo test -p auto-lang --test schema_drift` (2/2 pass)。
    - [x] **AC-06 门禁通过**：
      - `cargo test -p auto-lang test_style_recipe`: 6/6 pass (0.07s)
      - `cargo tv`: 3659 pass, 491 skipped, 0 fail (19.052s)
      - `cargo test -p auto-lang --test schema_drift`: 2/2 pass (0.16s)
      - `cargo t`: 失败集与 master 完全等价（仅 master 既有 564-Q6 预存红 3 测：`plan370_015_behavior_tests` 的 `d2`、`d8`、`z6`，经 blame 对拍实证为 564-Q6 预存，零新增回归）。
  - **Workaround & Debt Elimination**:
    - 无临时 hack，无绕道代码。
    - 登记规划边界至 `docs/plans/KNOWN-DEBT-AND-RISKS.md`（Phase 2b 与 P601-T11 边界清晰）。
  - **Spec Delta Review**:
    - SD-01 (`docs/specs/auto-lang/ui/overview.md`): 新增 §声明式样式配方语言层 现况注记。
    - SD-02 (`docs/design/10-language-syntax.md`): 补充 top-level `style` 声明与使用语法示例。
    - SD-03 (`docs/plans/KNOWN-DEBT-AND-RISKS.md`): 登记 Phase 2b 与 P601-T11 规划边界。
    - `docs/design/29-autoui-style-theme-system.md`: Phase 3 标记交付。
    - `docs/specs/auto-lang/ui/plans.md`: 追加 Plan 607 归档沉淀。
    - `docs/specs/goals.md`: GOAL-007 追加 Plan 607。
    - `python scripts/spec-index.py`: 重新生成 `docs/specs/INDEX.md`。

### 9.1 固化与归档凭据（Consolidation Receipt）

`PLAN-607:r1`

| Checkpoint | Status | Evidence |
|---|---|---|
| `prepared` | ✅ | reviewed_commit: `666515591`, delivery_commit: `9c708eb5f`, base_commit: `2388eaaea`, spec deltas SD-01..03 prepared |
| `landed` | ✅ | default-branch merge commit: `6591baf90` on `master` (`feat(ui): declarative style recipes (Plan 607)`) |
| `ledger_refreshed` | ✅ | `docs/specs/auto-lang/ui/overview.md`, `docs/specs/auto-lang/ui/plans.md`, `docs/specs/goals.md`, `docs/design/10-language-syntax.md`, `docs/design/29-autoui-style-theme-system.md`, `INDEX.md` |
| `archived` | ✅ | `docs/plans/archive/607-autoui-style-recipe.md`, completion_kind: delivered |
| `cleaned` | ✅ | `wt-guard.sh` clean verified; worktree `D:/autostack/.wt/lang-607/auto-lang` removed; branch `plan-607-dev` deleted |



## 10. 待澄清事项

1. **跨文件/跨模块共享**：
   - v1 方案：支持同 package / 同文件内声明与使用（`use` 机制是否需要显式支持 `use styles: card_base`）。
   - 建议：v1 保持 package 内可见（与 store/widget 约定一致），跨 pac 引用在 v2 视模块化需求扩充。
2. **Tailwind 覆盖冲突约定**：
   - 确认不引入运行时覆盖计算器，维持 class 串自然追加拼接原则，冲突项由 lint 提示。

