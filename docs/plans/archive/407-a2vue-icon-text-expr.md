# Plan 407: a2vue 增强 — 支持 lucide 图标子节点 + text 节点 t() 表达式

> **纲领**: 解决 auto-musk Plan 022 遗留的两个 codegen 缺口，消除"重跑 codegen 后必须手动改 .vue"的 KNOWN-DEBT。
> **分支**: `plan407/a2vue-icon-text-expr`
> **状态（2026-08-10）**: ✅ 完成。parser 改动 3 行 + 2 个 golden case（005/006）+ 全量回归通过（parser 165/165, vue 169/169, a2vue 6/6）。

## 背景

auto-musk 的 .at 源码通过 auto-lang 转译器生成 Vue SFC。Plan 022 对齐原生 Vue 版前端时发现两个 codegen 缺口：

1. **lucide 图标组件不能作子节点**：想在 .at 里写 `button { Plus {} }` 生成 `<button><Plus /></button>`
2. **text 节点不支持函数调用表达式**：想在 .at 里写 `text t('nav.chat')` 生成 `{{ t('nav.chat') }}`

两个缺口导致 auto-musk 的 toolbar 图标和 i18n 接入只能"codegen 后手动改 .vue"（登记为 KNOWN-DEBT），脆弱且违背 AutoUI ".at 是 single source of truth" 的设计。

## 探索结论（已验证）

### 缺口 1：lucide 图标作子节点 —— **已能用，无需改代码**

parser 和 codegen 都已支持外部组件（`use { component }` 声明的）作任意节点的子节点：
- parser.rs:12827-12830 的 children 循环对子节点类型无过滤
- vue.rs:3636-3648 处理"text + children 共存"分支（此前未被测试覆盖）
- case 003（fn_call_prop）已证明外部组件作 col 子节点可行

**唯一要做的工作：写 golden case 006 验证 + 在 auto-musk .at 源里应用。**

### 缺口 2：text 节点 t() 表达式 —— **仅 parser 缺口**

codegen 完全就绪：
- `expr_to_vue_text_raw` 的 `Expr::Call` 分支（vue.rs:5698-5708）已能生成 `t('nav.chat')`
- `expr_to_vue_text`（vue.rs:5716）会包成 `{{ t('nav.chat') }}`
- AST 类型 `ViewPropValue::Expr(Expr)` 能装下 `Expr::Call`

**缺口仅在 parser**：`parse_view_node`（parser.rs:12570-12581）的 `has_ident_field_primary` 只 peek ident 后跟 `Dot`（字段访问），不识别 ident 后跟 `LParen`（函数调用）。导致 `text t('nav.chat')` 里 `t` 被当 bare ident 消费，`('nav.chat')` 残留导致解析错乱。

## 实施

### 第一部分：parser 增强（parser.rs，~3 行核心改动）

扩展 `has_ident_field_primary` 的 peek 条件，同时识别 Dot 和 LParen：

```rust
// Before:
let is_dot = next_token.kind == TokenKind::Dot;

// After:
let is_field_or_call = next_token.kind == TokenKind::Dot
    || next_token.kind == TokenKind::LParen;
```

**向后兼容性**：现有 5 种 primary-prop 检测分支互斥逻辑不变。bare-ident 分支（处理 ident 后不是 Dot/`(` 的情况）不受影响——它有 `get_primary_prop` 守卫。

### 第二部分：golden case（2 个）

#### Case 005: text 节点函数调用表达式（`005_text_fn_call`）
验证 `text t("nav.chat")` → `{{ t('nav.chat') }}`。

#### Case 006: lucide 图标作子节点 + text 共存（`006_icon_child`）
验证 `button { Plus { size: 14 } text "新建" }` → `<button><Plus :size="14" .../>新建</button>`。
同时覆盖此前未被测试的 text+children 共存分支（vue.rs:3636-3648）。

### 测试与回归

| 套件 | 结果 |
|---|---|
| a2vue golden（001-006） | 6/6 ✓ |
| parser 单测 | 165/165 ✓ |
| vue 单测 | 169/169 ✓ |

## 第三部分：auto-musk .at 源码回流（待 auto-musk 侧执行）

转译器改完后，回到 auto-musk 仓库把之前手动改的 .vue 改动回流到 .at 源文件：
- `src/front/chats_view.at`：toolbar emoji→lucide + i18n t()
- `src/front/app.at`：导航 i18n t() + WorkspaceSelector 接入
- 重跑 codegen 验证无需手动改 .vue

## 改动文件

| 文件 | 改动 |
|---|---|
| `crates/auto-lang/src/parser.rs` | `has_ident_field_primary` peek 加 LParen 检测（3 行）|
| `crates/auto-lang/test/a2vue/005_text_fn_call/input.at` + `.expected.vue` | 新增 golden case |
| `crates/auto-lang/test/a2vue/006_icon_child/input.at` + `.expected.vue` | 新增 golden case |
| `crates/auto-lang/src/ui_gen/vue.rs` | 注册 test_a2vue_text_fn_call + test_a2vue_icon_child |

## 范围边界（不做）

- **不改 vue.rs codegen** —— Expr::Call 的 text 位置渲染已就绪
- **不改 AST** —— ViewPropValue::Expr 已能装 Expr::Call
- **不做 PascalCase 自动 lucide 推断** —— 保留 `use { component }` 显式声明
- **不碰 aura/extract.rs** —— Element 分支直接 clone prop Expr
