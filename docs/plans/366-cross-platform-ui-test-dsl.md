# Plan 366: 跨平台 UI 测试契约与 DSL（长期方向）

> **状态**: 设计阶段，暂不实现。当前用 AutoDown 契约 + Playwright 执行（见 §6）。
>
> **目标**: 让 UI 测试契约**目标无关**，通过转译器生成各后端的可执行测试。

---

## 1. 背景与动机

### 1.1 问题：Playwright 锁定 Web 后端

当前 015-notes 的 UI 测试用 Playwright（TypeScript）写成 `.spec.ts`。这对 **Vue/Web 后端**有效，但 AutoUI 是**跨平台**的：

| AutoUI 后端 | 运行时 | 测试执行器 |
|-------------|--------|-----------|
| Vue / Web | 浏览器 | Playwright / TS |
| Rust / iced | 桌面原生 | auto-ui-cli（Python / Auto） |
| Jet / Compose | Android | Espresso / UI Automator |
| ArkTS | 鸿蒙 | ArkTS UI 测试 |

**同一个功能的测试意图**（如"点 Shopping List → 显示牛奶"）在四个后端上要用四种不同语言重写。这违背了 Auto 的核心价值——**写一次，到处编译**。

### 1.2 核心洞察：测试也应该是目标无关的

Auto 代码通过 `a2ts` / `a2py` / `a2r` 转译器生成目标特定代码。**测试应该遵循同样的模式**：

```
                  ┌─── a2ts(test) ──→ Playwright .spec.ts (Web)
                  │
Auto 测试 DSL ────┼─── a2py(test) ──→ auto-ui-cli 脚本 (iced 桌面)
  (目标无关)       │
                  ├─── a2kt(test) ──→ Espresso 测试 (Android)
                  │
                  └─── a2ark(test) ─→ ArkTS UI 测试 (鸿蒙)
```

一份测试契约，N 个执行后端。

### 1.3 为什么这是"系统设计"而非"小改进"

设计一套 Auto 测试 DSL 涉及：
- **DSL 语义**：浏览器/原生的通用操作原语（点击、断言可见、等待、取属性）
- **抽象边界**：哪些 Web 概念（CSS 选择器、DOM）不该泄漏到 DSL
- **转译器**：每个后端一个 `a2X(test)` 转译器
- **运行编排**：`auto test:ui` 如何启动被测应用 + 跑测试 + 收集结果
- **断言映射**：`assert_color(button, "coral")` 在每个后端如何实现

这是数周的工程，不该在 015-notes 迭代中做。但设计要先记录，避免遗忘。

---

## 2. 设计草案：Auto 测试 DSL

### 2.1 扩展现有单元测试语法

Auto 已有 `#[test]` 单元测试。UI 测试 DSL 在此基础上扩展 **UI 操作原语**：

```auto
test "T1: 笔记切换更新内容" {
    // 启动被测应用（由 test runner 注入 URL 或进程句柄）
    ui.open(.app_url)
    
    // 操作原语（目标无关）
    ui.click("button:has-text('Shopping List')")
    
    // 断言原语
    ui.assert_visible(".editor-content")
    ui.assert_text_contains(".editor-content p", "Milk")
}

test "T12: 暗色模式下主题色生效" {
    ui.open(.app_url)
    ui.click("button:has-text('Dark')")
    ui.click("[data-accent='coral']")
    
    // 颜色断言（语义化，非 RGB 字面量）
    ui.assert_color("button:has-text('New')", .accent_primary)
}
```

### 2.2 关键设计决策

#### A. 选择器策略

Web 用 CSS 选择器，桌面用 Accessibility ID，移动用 resource-id。**DSL 应提供抽象选择器**：

```auto
// ❌ 不要这样（Web 特定）
ui.click(".sidebar .note-item:nth-child(2)")

// ✅ 这样（语义化，转译器映射到各后端的选择器机制）
ui.click(.note_titled("Shopping List"))
ui.click(.button_labeled("New"))
ui.click(.element_with_role("tab", "Pinned"))
```

这要求被测应用暴露**语义锚点**（aria-label / test-id / accessibility id）。这是测试可移植性的前提。

#### B. 断言的抽象层次

```auto
// 颜色断言：用语义名而非字面值
ui.assert_color(.new_button, .accent_primary)

// 转译器知道 .accent_primary 在当前 accent 下解析成什么
// Web: getComputedStyle(el).backgroundColor
// iced: widget.style().background
```

#### C. 时序与等待

各后端的异步模型不同（Web 有网络延迟，桌面有事件循环）。DSL 提供**语义等待**：

```auto
ui.wait_for(.note_titled("Meeting Notes"), .visible)
// 转译器映射：Web → page.locator().waitFor()，iced → 轮询事件
```

### 2.3 转译器：a2ts(test)

最简单的后端（因为 Web 最成熟）。把 Auto 测试 DSL 翻译成 Playwright `.spec.ts`：

```typescript
// 自动生成，勿手改
import { test, expect } from '@playwright/test'

test('T1: 笔记切换更新内容', async ({ page }) => {
  await page.goto(process.env.APP_URL!)
  await page.click("button:has-text('Shopping List')")
  await expect(page.locator('.editor-content')).toBeVisible()
  await expect(page.locator('.editor-content p')).toContainText('Milk')
})
```

### 2.4 转译器：a2py(test)（桌面）

```python
# 自动生成，由 auto-ui-cli 执行
def test_T1_note_switching(auto_ui):
    auto_ui.open()
    auto_ui.click(note_titled("Shopping List"))
    assert auto_ui.is_visible(".editor-content")
    assert "Milk" in auto_ui.text_of(".editor-content p")
```

---

## 3. 测试契约的分层

```
┌──────────────────────────────────────────────────┐
│ 第 1 层：验收契约（AutoDown）                     │
│   tests/acceptance.atd                            │
│   人/AI 读，声明"测什么"，领域语言                │
│   → 功能的 single source of truth                 │
└──────────────────────────────────────────────────┘
                    ↓ 翻译（手动/AI辅助）
┌──────────────────────────────────────────────────┐
│ 第 2 层：可执行测试契约（Auto 测试 DSL）          │
│   tests/*.at                                      │
│   目标无关，含操作 + 断言原语                     │
│   → 跨平台测试的"源代码"                          │
└──────────────────────────────────────────────────┘
                    ↓ 转译（a2ts/a2py/a2kt）
┌──────────────────────────────────────────────────┐
│ 第 3 层：后端特定测试（生成物）                   │
│   tests/*.spec.ts (Web)                           │
│   tests/test_*.py (iced)                          │
│   → 实际由测试运行器执行                           │
└──────────────────────────────────────────────────┘
```

**当前（Plan 366 暂不实现第 2 层）**：直接从第 1 层手动翻译到第 3 层（AutoDown → Playwright .spec.ts）。等 DSL 成熟后再插入第 2 层。

---

## 4. 测试的"语义锚点"前提

跨平台测试 DSL 要求被测应用暴露**统一的锚点**。当前 .at 生成的 Vue 代码缺少这些。需要约定：

### 4.1 在 .at 里支持 `test_id` 属性

```auto
button "New" {
    onclick: .NewNote
    test_id: "new-note-btn"     // 新增：生成 data-testid
}
```

转译到各后端：
- Web: `<button data-testid="new-note-btn">`
- iced: `widget.push(Attribute::TestId("new-note-btn"))`
- Android: `android:tag="new-note-btn"`

测试用 `ui.click(.test_id("new-note-btn"))`，跨后端稳定。

### 4.2 语义角色

对常见 ARIA 角色，提供快捷锚点：

```auto
ui.click(.button_labeled("New"))    // role=button + accessible-name
ui.click(.tab_labeled("Pinned"))    // role=tab + accessible-name
```

---

## 5. `auto test:ui` 命令设计

```bash
# 自动化全流程
auto test:ui [--target vue|iced|all] [--watch]

# 等价于：
#   1. auto build（生成目标应用）
#   2. 启动应用（vite dev / iced 二进制）
#   3. a2ts(test) 转译 .at 测试 → .spec.ts
#   4. playwright run / auto-ui-cli run
#   5. 收集结果，关闭应用
#   6. 报告：哪些 T1-T13 通过/失败
```

`--watch` 模式（配合 Plan 362）：.at 改动后增量重建 + 重跑受影响的测试。

---

## 6. 当前可落地的部分（Plan 366a）

在 DSL 成熟前，先用**双层方案**获得 80% 的价值：

### 6.1 AutoDown 验收契约（第 1 层）

`examples/ui/015-notes/tests/acceptance.atd` — 用领域语言声明 T1-T13 + 契约依据。

### 6.2 Playwright 执行（第 3 层）

`examples/ui/015-notes/tests/*.spec.ts` — 手动/辅助翻译自契约，立即执行。

### 6.3 两层的对应关系

每个测试有 ID（T1, T2...），契约和 spec.ts 用同一 ID 关联：

```
acceptance.atd:
  ## T12: 暗色模式下主题色生效
  - 契约依据：C-DARK-1

accent-dark.spec.ts:
  test('T12: accent in dark mode', ...)
```

### 6.4 运行命令

`auto test:ui`（简化版，仅 Web 后端）：
1. 确保 dev server 在跑
2. `npx playwright test`
3. 报告结果

---

## 7. 实施路线

| 阶段 | 内容 | 时间 |
|------|------|------|
| **366a（现在）** | AutoDown 契约 + Playwright spec.ts + `auto test:ui` | 1-2 天 |
| **366b（未来）** | Auto 测试 DSL 原语设计（click/assert/wait 语义） | 1-2 周 |
| **366c（未来）** | a2ts(test) 转译器 | 1 周 |
| **366d（未来）** | .at 支持 `test_id` 属性 | 2-3 天 |
| **366e（未来）** | a2py(test) 转译器（iced 桌面） | 1-2 周 |
| **366f（未来）** | `auto test:ui --target all` 多后端 | 1 周 |

### 366a 的验收标准

- [ ] `tests/acceptance.atd` 覆盖 T1-T13 + 本次 dark-mode accent bug 的回归测试
- [ ] `tests/*.spec.ts` 能用 `npx playwright test` 跑通
- [ ] `auto test:ui` 一键运行（启动 dev server + 跑测试 + 报告）
- [ ] 故意改坏（如恢复 dark-mode accent bug）能让测试红
- [ ] 契约里标注了 C-DARK-1 / C-CSS-1 等生成器契约依据

---

## 8. 与其他 Plan 的关系

- **Plan 361**：校验规则在生成时检查静态契约；本 Plan 的运行时测试补充了校验框架抓不到的动态行为
- **Plan 362**：`auto watch` 配合本 Plan 的 `--watch` 模式，实现"改 .at → 自动重跑测试"
- **Plan 363**：generator-contracts.md 里的 C-DARK-1 等契约，是本 Plan 测试的依据；Skill 在生成新功能时同步生成测试骨架
