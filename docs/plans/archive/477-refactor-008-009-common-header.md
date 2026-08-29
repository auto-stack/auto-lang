---
plan_id: PLAN-477
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: refactor-008-009-common-header
author: [Antigravity]
created_at: 2026-08-29
updated_at: 2026-08-29

supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [examples/ui/008-pricing-table, examples/ui/009-article-feed]
current_step: 4
total_steps: 4
---

# [PLAN-477] Refactor 008-pricing-table & 009-article-feed to Common Header

## 变更摘要

在 Plan 475 实现的 AutoMan 本地 UI 依赖机制基础上，将 `008-pricing-table` 和 `009-article-feed` 重构为通过 `pac.at` 引入 `examples/ui/common` 中的 `ExampleHeader` 组件，消除冗余的内联顶栏与设置面板代码，并通过双端（Vue + VM）自动化验证。

---

## 目标

- **G1（依赖声明）**：在 `008-pricing-table/pac.at` 和 `009-article-feed/pac.at` 中添加 `dep "common" { path: "../common" }`。
- **G2（组件复用）**：在两者的 `src/front/app.at` 中引入 `use common.header: ExampleHeader`，精简内联状态和顶栏视图。
- **G3（双端验证）**：通过 Playwright（Vue）与 AutoUI MCP（VM）对 008 和 009 分别进行全量回归与截图比对，确保功能与样式 100% 正常。

---

## 执行步骤

| 序号 | 步骤 | 操作目标与文件路径 | 验收标准 | 状态 |
|---|---|---|---|---|
| T1 | 重构 `008-pricing-table` | 更新 `008-pricing-table/pac.at` 与 `app.at` 引入 `ExampleHeader` | 编译正常 | [✅] |
| T2 | 验证 `008-pricing-table` 双端 | 运行 008 的 VM 与 Vue 自动化测试脚本 | 双端 100% 通过且截图一致 | [✅] |
| T3 | 重构 `009-article-feed` | 更新 `009-article-feed/pac.at` 与 `app.at` 引入 `ExampleHeader` | 编译正常 | [✅] |
| T4 | 验证 `009-article-feed` 双端 | 运行 009 的 VM 与 Vue 自动化测试脚本 | 双端 100% 通过且截图一致 | [✅] |

---

## 复审记录

### 1. Checklist Audit (逐项核对)
- [x] T1: `008-pricing-table` 的 `pac.at` 声明 `dep "common"`，`src/front/app.at` 成功替换为 `<ExampleHeader icon="💳", title="Pricing Table", badge="008" />`，移除了 180+ 行内联顶栏与设置代码。
- [x] T2: `008-pricing-table` 经 `test_008_vm.py` 和 `test_008_vue.mjs` 验证，暗色初始、设置面板打开、Coral 配色切换、亮色与暗色模式双端 100% 一致。
- [x] T3: `009-article-feed` 的 `pac.at` 声明 `dep "common"`，`src/front/app.at` 成功替换为 `<ExampleHeader icon="📰", title="Article Feed", badge="009" />`，移除了 180+ 行内联顶栏与设置代码。
- [x] T4: `009-article-feed` 经 `test_009_vm.py` 和 `test_009_vue.mjs` 验证，暗色初始、设置面板打开、Coral 配色切换、亮色与暗色模式双端 100% 一致。

### 2. Workaround & Debt Scan (遗漏与负债扫描)
- 零临时 hack / workaround。
- 测试脚本均使用相对路径定位截图输出目录，符合工程规范。

### 3. Health Check (健康检查)
- `auto build --gen-only` 在 008 和 009 均一次通过并正确生成 2 个 SFC 组件。
- 自动化测试全部通过。

