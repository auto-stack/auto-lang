# Plan 361: 生成器加固 — 不变量检查 + 代码路径收敛 + 冒烟测试

> **目标**: 在不改变现有架构的前提下，用最低成本把"反复出现操作失效"这类问题在生成阶段就拦下来，而不是等运行时暴露。
>
> **范围**: 纯防御性改进。不引入新语言特性，不改 .at 语义，不动 AutoDown。所有改动都在 `auto-lang` / `auto-man` 内部。

---

## 1. 问题回顾：本次会话暴露的生成器缺陷

| 问题 | 根因 | 在哪个阶段本可拦截 |
|------|------|--------------------|
| 暗色模式 handler key 漏了点 | `"ToggleDarkMode"` vs `".ToggleDarkMode"` | 生成后 lint：handler 引用一致性 |
| NavTree/Editor 缺 store import | 三条生成路径之一漏传 store_deps | 代码路径收敛 + 生成后 lint：store 使用 vs import |
| Edit 后编辑框为空 | 同名组件固定 key 在 v-if 分支冲突 | 生成后 lint：同名组件 key 唯一性 |
| 底部奇怪的 `+` 号 | 用了 `@autodown/editor` 却没导入 CSS | 生成后 lint：npm dep ↔ CSS import 对应 |
| Cancel 无反应 | 上面的连带效应 | 上游修复即可 |

**共同模式**：所有问题都是"生成的代码违反了一个本应成立的不变量"。目前生成器只负责"生成"，不负责"验证"。

---

## 2. 核心设计：生成后校验（Post-Generation Validation）

### 2.1 新模块：`crates/auto-lang/src/ui_gen/validators.rs`

在 `VueGenerator::generate()` 的最后一步，对生成的 SFC 字符串跑一组**纯文本/AST 级检查**。每个检查返回 `Result<(), ValidationWarning>`，不阻塞生成但打印明确警告。

```rust
pub struct ValidationWarning {
    pub rule: &'static str,        // 规则名，如 "duplicate-component-key"
    pub severity: Severity,        // Error / Warning / Info
    pub widget: String,            // 所在 widget
    pub message: String,           // 人类可读说明
    pub fix_hint: Option<String>,  // 建议的修复方向
}

pub enum Severity { Error, Warning, Info }

/// 对一个生成的 SFC 跑所有校验规则
pub fn validate_sfc(sfc: &str, widget_name: &str, ctx: &ValidationContext) -> Vec<ValidationWarning>;
```

### 2.2 第一批校验规则（覆盖本次所有问题）

| 规则 ID | 检查内容 | 触发条件 | 严重度 |
|---------|----------|----------|--------|
| `R001` duplicate-component-key | 模板内同名组件的 `:key` 必须互不相同 | 同一标签出现 ≥2 次，且 key 相同或都缺失 | Error |
| `R002` store-usage-without-import | script 里引用了 `store.X` 但没有 `import { useXStore }` | 正则 `store\.\w+` 命中但无对应 import | Error |
| `R003` autodown-css-missing | 模板含 `AutoDownEditor` 但 main.ts 没导入 `@autodown/editor/style.css` | 跨文件检查 | Error |
| `R004` undefined-handler | `@click="X"` 的 X 未在 script 里定义 | 正则匹配 `@\w+="(\w+)"`，检查定义 | Warning |
| `R005` handler-key-format | 生成器内部 handler 查找使用带点格式 | 内部断言（不检查生成产物） | Error |
| `R006` emit-without-declaration | `emit('X')` 但 `defineEmits` 里没声明 X | 正则匹配 | Warning |
| `R007` orphan-event-binding | `@update="EditBody"` 但 script 里 EditBody 是空函数体 | AST 检查 | Info |

### 2.3 校验输出示例

```
⚠ Validation warnings for EditorPanel.vue:
  [R001 ERROR]   duplicate-component-key:
    Two <AutoDownEditor> instances share key 'AutoDownEditor'.
    In read-mode (line 171) and edit-mode (line 168).
    Fix: Vue will patch in place instead of remounting — this breaks
    components that rely on fresh mount (e.g. Tiptap). Give each a unique key.
  [R004 WARNING] undefined-handler:
    @update="EditBody" references function EditBody (line 168),
    but its body is empty. Did you forget to store the updated content?
```

这些警告会：
- 打印到 `auto build` / `auto run` 的输出
- 写入 `.auto/build/validation.log`（便于 CI 检查）
- 通过 exit code 反馈：有 ERROR 级警告时 `auto build` 返回非零（可配置关闭）

---

## 3. 代码路径收敛

### 3.1 现状：三条生成路径

```
compile_at_to_vue()              ← auto-lang/lib.rs
compile_at_to_vue_with_sub_widgets()  ← auto-man/vue.rs
from_workspace() 内联生成         ← auto-man/vue.rs
```

每条路径都要正确传递 `api_imports` / `store_deps` / `sub_widget_names`，这次就有一条漏了。

### 3.2 收敛方案：单一入口函数

在 `auto-lang` 新增一个 `generate_component` 公开函数，封装"从 .at 文件到 SFC"的完整流程：

```rust
pub struct ComponentGenOptions {
    pub sub_widgets: Vec<String>,      // 已知子组件名
    pub api_imports_override: Vec<String>,  // None 时自动从 use back.api 提取
    pub store_deps_override: Vec<String>,   // None 时自动从 use store 提取
}

pub fn generate_component_from_file(
    at_path: &Path,
    opts: ComponentGenOptions,
) -> AutoResult<GeneratedComponent>;

pub struct GeneratedComponent {
    pub vue_code: String,           // 第一个 widget 的 SFC
    pub all_widget_codes: Vec<String>,  // 所有 widget 的 SFC（app.at 里可能有多个）
    pub store_composables: Vec<(String, String)>,  // (filename, code)
    pub detected_api_imports: Vec<String>,
    pub detected_store_deps: Vec<String>,
    pub shadcn_components: HashSet<String>,
    pub validation_warnings: Vec<ValidationWarning>,
}
```

`auto-man` 的三条路径全部改为调用这个函数。`api_imports` / `store_deps` 的提取逻辑**只在这里写一次**。

### 3.3 迁移策略

- Phase A：新增 `generate_component_from_file`，内部调用现有逻辑
- Phase B：把 `auto-man` 的三条路径逐一切换到新入口
- Phase C：删除旧的 `compile_at_to_vue` / `compile_at_to_vue_with_sub_widgets` 内联逻辑

每一步都保持测试通过。

---

## 4. 015-notes 端到端冒烟测试

### 4.1 新文件：`examples/ui/015-notes/tests/smoke.spec.ts`

用 playwright 写一个固定的冒烟脚本，覆盖本次所有出问题的操作：

```typescript
import { test, expect } from '@playwright/test'

test.describe('015-notes smoke', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000')
    await page.waitForSelector('.autodown-editor-content-wrapper')
  })

  test('note switching preserves editor content', async ({ page }) => {
    // 切换 note 不应导致 Tiptap unmount 错误
    await page.click('button:has-text("Shopping List")')
    await expect(page.locator('.autodown-editor-content-wrapper p')).toContainText(/./)
    await page.click('button:has-text("Meeting Notes")')
    await expect(page.locator('.autodown-editor-content-wrapper p')).toContainText(/./)
  })

  test('edit mode shows current note content', async ({ page }) => {
    await page.click('button:has-text("Edit")')
    await expect(page.locator('input[placeholder="Note title..."]')).toHaveValue(/\S/)
    await expect(page.locator('.ProseMirror')).toContainText(/./)
  })

  test('cancel returns to read mode with original content', async ({ page }) => {
    const original = await page.locator('.autodown-editor-content-wrapper p').first().textContent()
    await page.click('button:has-text("Edit")')
    await page.click('button:has-text("Cancel")')
    await expect(page.locator('button:has-text("Edit")')).toBeVisible()
    await expect(page.locator('.autodown-editor-content-wrapper p').first()).toHaveText(original!)
  })

  test('dark mode toggles root class', async ({ page }) => {
    const root = page.locator('#app > div').first()
    const before = await root.evaluate(el => el.className.includes('dark'))
    await page.click('button:has-text("Dark Mode")')
    await expect(root).evaluate(el => el.className.includes('dark') !== before)
  })

  test('no stray plus button visible in read mode', async ({ page }) => {
    const plus = page.locator('.autodown-block-boundary-plus')
    await expect(plus).toHaveCSS('opacity', '0')
  })
})
```

### 4.2 运行方式

新增 npm script：`pnpm test:smoke`。`auto build` 后可选自动运行（通过 `--smoke` flag）。

这些测试**不需要后端真实数据**——前端能用 mock 数据跑通所有交互路径。

---

## 5. 实施计划

### Phase 1: 校验框架骨架（半天）
- [ ] 新建 `validators.rs`，定义 `ValidationWarning` / `Severity` / `ValidationContext`
- [ ] 实现 `validate_sfc` 框架，空规则集先跑通
- [ ] 在 `generate_sfc` 末尾调用，警告打印到 stderr

### Phase 2: 第一批规则（1 天）
- [ ] R001 duplicate-component-key（本次最痛的）
- [ ] R002 store-usage-without-import
- [ ] R003 autodown-css-missing
- [ ] R004 undefined-handler
- [ ] 每个 rule 配单元测试（用恶意 SFC 作 fixture）

### Phase 3: 代码路径收敛（1-2 天）
- [ ] 实现 `generate_component_from_file`
- [ ] 切换 `from_workspace` 的 front_dir 组件路径
- [ ] 切换 `from_workspace` 的 app.at 多 widget 路径
- [ ] 切换 pages/ 扫描路径
- [ ] 删除 `compile_at_to_vue` / `compile_at_to_vue_with_sub_widgets`

### Phase 4: 冒烟测试（半天）
- [ ] 015-notes 加 playwright 配置
- [ ] 实现上述 5 个测试
- [ ] 文档化运行方式

---

## 6. 验收标准

- [ ] 本次会话的 5 个问题，在校验框架下都能在 `auto build` 阶段被标记为 ERROR/WARNING
- [ ] 三条生成路径合并为一条，store_deps 丢失类问题不再可能发生
- [ ] `pnpm test:smoke` 在健康的 015-notes 上全绿
- [ ] 故意破坏（改回固定 key、删 CSS import）能让冒烟测试红
