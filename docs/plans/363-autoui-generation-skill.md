# Plan 363: AutoUI 代码生成 Skill — 安全生成 + 模式库 + 交互式向导

> **目标**: 提供一个 Skill，让 AI（或人类开发者）在创建或修改 AutoUI 项目时，**默认生成正确的、符合不变量的代码**，把 Plan 361 的校验规则前置到"生成阶段"而非"验证阶段"。

---

## 1. 为什么需要这个 Skill

### 1.1 当前痛点

本次会话暴露的问题有一个共同模式：**AI 生成的 .at 代码在语法上合法，但在语义上违反了生成器/运行时的隐式契约**。

例子：
- AI 写了两个 `autodown_editor` 在不同 `if` 分支 → 合法，但触发 key 冲突
- AI 写了 `use store: NotesStore` 但 store handler 里用了 `.store.X` → 合法，但某些生成路径会丢 store_deps
- AI 写了 `autodown_editor` 但不知道需要导入 CSS → 合法，但运行时样式错乱

这些**不是语法错误，而是"陷阱模式"**。人类专家知道要避免，但每次都要手动提醒 AI。

### 1.2 Skill 的价值

一个专门的 AutoUI Skill 可以：
1. **编码陷阱知识**：把已知的反模式及其修复固化成可引用的规则
2. **提供经过验证的模式库**：常见 UI 结构（列表-详情、CRUD 表单、主从布局）的标准写法
3. **强制生成时校验**：生成 .at 后自动跑 Plan 361 的校验
4. **引导正确的扩展姿势**：加组件、加 store、加 API 的正确顺序和检查点

---

## 2. Skill 架构

### 2.1 三层知识结构

```
crates/autoui-skill/  (或 .claude/skills/autoui/)
├── SKILL.md                    ← 主入口，Skill 触发条件和使用流程
├── reference/
│   ├── syntax-guide.md         ← Auto UI DSL 完整语法参考
│   ├── semantic-tokens.md      ← bg-primary / text-foreground 等 token 清单
│   ├── generator-contracts.md  ← 生成器契约（Plan 361 的不变量）
│   └── known-pitfalls.md       ← 已知陷阱模式及修复
├── patterns/
│   ├── list-detail.md          ← 列表-详情布局标准模式
│   ├── crud-form.md            ← CRUD 表单标准模式
│   ├── master-detail.md        ← 主从布局
│   ├── tree-navigation.md      ← 树形导航
│   ├── editor-integration.md   ← AutoDownEditor 集成模式
│   ├── store-pattern.md        ← store composable 模式
│   └── dark-mode.md            ← 暗色模式集成
├── templates/
│   ├── new-project/            ← 新项目脚手架模板
│   ├── new-widget/             ← 新 widget 模板
│   ├── new-store/              ← 新 store 模板
│   └── new-page/               ← 新页面模板
└── checks/
    └── pre-commit-checklist.md ← 提交前检查清单
```

### 2.2 SKILL.md 核心逻辑

```markdown
---
name: autoui
description: Generate and modify AutoUI (.at) projects safely. Use when creating
  new UI widgets, stores, or pages; when adding AutoDownEditor; when debugging
  UI generation issues. Enforces generator contracts and known-good patterns.
---

# AutoUI Generation Skill

## When to use
- Creating a new AutoUI project (`auto new app`)
- Adding a widget / store / page to an existing project
- Modifying .at files (style, structure, handlers)
- Integrating AutoDownEditor or other stateful components
- Debugging "generated code doesn't work" issues

## Workflow
1. **Classify the task**: new project / add component / modify existing / debug
2. **Select pattern**: match the task to a pattern in patterns/
3. **Generate**: produce .at code following the pattern
4. **Validate**: run `auto build` and check warnings (Plan 361)
5. **Update acceptance contract**: if behavior changes, update `tests/acceptance.atd`
   in lockstep (add/modify the T-entry for the affected feature)
6. **Generate test skeleton**: for new features, emit a `.spec.ts` skeleton
   matching the new T-entry (see §3.4)
7. **Smoke test**: run `pnpm test` in `tests/` to verify no regression
   (~45s if dev server is running; ~60s including server startup)

## Critical rules (ALWAYS follow)
- **改 .at 时，同步更新 acceptance.atd**（契约是功能的 single source of truth）
- **新功能必须配测试**——不要让功能"裸奔"
- **修 bug 时加回归测试**（标注"历史教训"，如 T12-DARK）
- 引用 generator-contracts.md 的关键不变量（见 §3）
```

### 3.4 测试套件集成（Plan 366a 补充）

Skill 不仅是"生成 .at 代码"，还要**生成和维护测试**。这是防御纵深的关键一环：

#### 三层产物

```
改功能时，Skill 同步产出/更新：
├── src/front/foo.at          ← 实现（已有）
├── tests/acceptance.atd      ← 验收契约（T 条目）
└── tests/foo.spec.ts         ← 可执行测试骨架
```

#### 测试骨架生成规则

当 Skill 生成一个新功能（如"分享笔记"）时，自动产出：

**acceptance.atd 里加条目：**
```markdown
### T14: 分享笔记
- **当** 用户点击 "Share" 按钮
- **那么** 生成一个可分享的链接
- **契约依据**：C4（handler 引用一致性）
```

**share.spec.ts 骨架：**
```typescript
test('T14: 分享笔记', async ({ page }) => {
  // TODO: implement per acceptance.atd T14
  // - click Share button
  // - assert share link generated
})
```

#### 回归测试的强制规则

当 Skill 诊断并修复一个 bug 时，**必须**生成一个回归测试，格式：
```markdown
### T14-REGRESS: <bug 简述>
- **历史教训**：<为什么这个 bug 曾发生>
- **契约依据**：C-XXX
- **回归断言**：<测试要验证的不变量>
```

本次会话的实例：T12-DARK（dark-accent 失效）就是这么来的——它的 acceptance.atd 条目标注了"历史教训"和 C-DARK-1 契约。


---

## 3. 核心知识：Generator Contracts（生成器契约）

这是 Skill 的灵魂——**把生成器的隐式假设变成显式契约**。

### 3.1 `generator-contracts.md` 示例内容

```markdown
# Auto UI → Vue Generator Contracts

这些是生成器对 .at 代码做出的假设。违反它们会导致生成的 Vue 代码行为异常。

## C1: 组件实例身份

**契约**: 同一模板内，同名组件的每次使用都会获得唯一的稳定 key。

**这意味着**:
- ✅ 可以在两个 v-if 分支里用同名组件（如两个 AutoDownEditor），
  生成器会给它们不同的 key（AutoDownEditor-1, AutoDownEditor-2）
- ⚠️ 但如果两个分支的组件需要**保持状态连续性**（如编辑器内容），
  考虑用单实例 + prop 切换，见 [editor-integration.md]

**违反症状**: 组件切换后状态丢失、Tiptap editor 空白、子组件报 unmount 错误

## C2: Store 依赖传播

**契约**: `use store: Name` 声明会被提取并传递给生成器，生成对应的
`import { useNameStore }` 和 `const store = reactive(useNameStore())`。

**这意味着**:
- ✅ 在 widget 里写 `use store: NotesStore` 后，可以直接用 `store.X`
- ⚠️ store_deps 的提取依赖**解析 `use store:` 语句**。如果你用其他语法
  （如 `use back.store: X`）引用 store，生成器不会识别

**违反症状**: 生成的 .vue 里 `store is not defined`

## C3: 第三方组件 CSS 依赖

**契约**: 使用有 CSS 副作用的 npm 依赖时，生成器会自动导入其样式表。

**当前自动处理的**:
- `@autodown/editor` → `import '@autodown/editor/style.css'`

**新增 CSS 依赖时**: 需要在 `auto-man/vue.rs` 的 `generate_main_ts`
里添加对应的注入逻辑，或在 .at 里手动提示。

## C4: Handler 引用一致性

**契约**: 模板里 `onclick: .X` 引用的 handler X 必须在 `on { .X -> ... }` 块里定义。
生成器会为未定义的 handler 生成空函数（`// TODO: handler not defined`）。

**这意味着**:
- ✅ 先在 `msg Msg` 里声明，再在 `on` 里定义，最后在 `view` 里引用
- ⚠️ 如果只在 view 里引用但 on 块里没有，生成的函数体为空，点击无反应

## C5: 暗色模式检测

**契约**: 生成器通过检测 `.ToggleDarkMode` handler（注意前导点）来判断是否注入暗色模式 class。

**这意味着**:
- ✅ 在 on 块里定义 `.ToggleDarkMode -> { ... }`，生成器会自动在根元素加 `:class="{ dark: ... }"`
- ⚠️ handler 名必须**精确**是 ToggleDarkMode，其他名字（如 ToggleTheme）不会触发

## C6: 列表渲染与 key

**契约**: `for x in .items` 生成的列表项会自动绑定 `:key` 到循环变量。

**这意味着**:
- ✅ 列表项的 key 是自动的，不需要手动指定
- ⚠️ 如果列表项是**有状态组件**（如 editor），note 切换时组件会销毁/重建。
  这可能触发第三方库的 unmount 问题。见 [editor-integration.md]

## C7: 同名组件的 key 唯一性（Plan 360 教训）

**契约**: 同一模板内，同名组件的每次使用都会获得唯一的稳定 key（如
`AutoDownEditor-1`、`AutoDownEditor-2`），通过生成器的 `widget_key_counter`。

**这意味着**:
- ✅ 可以在两个 v-if 分支里用同名组件（如读/写双 editor），生成器会给不同 key
- ⚠️ 如果两个同名实例**需要状态连续性**（editor 内容不丢失），考虑用单实例 + prop 切换
- ⚠️ 自定义 key 目前不被 .at 支持——key 完全由生成器管理

**违反症状**: v-if 切换后组件状态丢失、Tiptap editor 空白、`view is not available`

**回归测试**: T1（笔记切换）、T5（Edit/Save/Cancel）

## C8: CSS 变量作用域（Plan 360 教训）⚠️ 关键

**契约**: 所有修改 CSS 变量（如 `--primary`）的子系统，写入的元素必须和
Tailwind/shadcn 的 CSS 规则（如 `.dark { --primary: ... }`）在**同一个 DOM 元素**上。

**背景**: 生成器把 `.dark` class 加到 `#app > div`（根组件），但 CSS 变量
继承机制下，子元素同名变量会**覆盖**父元素的继承值。如果 accent 系统把
`--primary` 写到 `<html>`，而 `.dark` 规则在 `#app > div`，子元素 `.dark` 的
值会赢 → accent 在暗色模式下失效。

**这意味着**:
- ✅ 改 CSS 变量的代码必须考虑"目标元素是否和消费它的 class 在同一层"
- ✅ 如果不确定，写入时**同时覆盖所有可能的目标元素**（见 applyAccent 的 applyToDark）
- ⚠️ 切换 dark/light 时，要清理旧模式留下的 inline 残留

**违反症状**: 亮色模式主题色生效，暗色模式主题色失效（显示默认色）

**回归测试**: **T12-DARK**（关键回归）、C-DARK-1（契约验证）

**历史教训**: Plan 360 实施时，applyAccent 只写 `<html>`，结果暗色模式下
accent 失效。修复需要同时写 `<html>` 和 `.dark` 元素 + 清理残留。

## C9: 主题色 accent 系统（Plan 360 新增）

**契约**: store 声明 `accent_color` 状态时，生成器自动注入完整的 accent 系统：
- `ACCENT_PALETTES` 数据（5 色 HSL）
- `applyAccent(name, isDark)` 函数（设置 CSS 变量 + localStorage）
- `SetAccent` handler 自动联动 applyAccent
- `ToggleDarkMode` handler 自动联动 applyAccent（暗色 lightness +4% 补偿）
- 模块级 bootstrap（从 localStorage 恢复）
- `accent_names` getter（供 UI 渲染色板）

**这意味着**:
- ✅ 用户只需声明 `var accent_color str = "indigo"` + `.SetAccent(name)` handler
- ✅ 暗色模式下 lightness 自动补偿，无需手动处理
- ⚠️ 受 C8 约束——applyAccent 的目标元素必须覆盖 `.dark` 元素

**回归测试**: T12-LIGHT、T12-DARK、T12-ROUNDTRIP
```

### 3.2 `known-pitfalls.md`：本次会话的所有教训

```markdown
# Known Pitfalls（已知陷阱）

## P1: 双 AutoDownEditor 模式切换

**反模式**:
```auto
if .editing == true {
    autodown_editor { content: .edit_body, can_edit: true }
}
if .editing == false {
    autodown_editor { content: .note.body, can_edit: false }
}
```

**问题**: 两个实例切换时，Tiptap 的 onUnmounted 可能访问已销毁的 editor.view.dom，
导致 "The editor view is not available" 错误，中断模式切换。

**修复**: 生成器现在给两个实例不同的 key，且 CodeBlockMenu 加了 isDestroyed 保护。
但如果可能，**优先用单实例 + canEdit 切换**。

## P2: 固定 key 导致状态丢失

**反模式**: 给有状态组件赋固定 key `:key="'MyEditor'"`，期望它在 note 切换时保持实例。

**问题**: 固定 key 意味着组件永不销毁，props 变化时只 patch。对需要响应 content 变化
重新初始化的组件（Tiptap），patch 不够，需要 remount。

**正确做法**: 让生成器自动管理 key。如需保持实例，改用 prop 切换架构。

## P3: store_deps 在多路径生成时丢失

**反模式**: 假设 `use store: X` 总会被正确传递给生成器。

**问题**: 生成器有三条代码路径（Plan 361 之前），某些路径会漏传 store_deps。

**修复**: Plan 361 收敛为单一路径。但开发新生成路径时，**必须传递 store_deps**。

## P4: 暗色模式 handler 名不精确

**反模式**: 用 `.ToggleTheme` 或 `.ToggleDark` 而不是 `.ToggleDarkMode`。

**问题**: 生成器只识别精确字符串 ".ToggleDarkMode"。

**修复**: 固定使用 ToggleDarkMode，或让生成器支持配置（见 Plan 360）。

## P5: 用了 @autodown/editor 但没导入 CSS

**反模式**: 在 pac.at 声明 npm_deps 后，假设 CSS 会自动加载。

**问题**: 生成器（Plan 360 之前）不会自动注入 CSS import。

**修复**: Plan 360 已修复。新增有 CSS 副作用的依赖时，更新 `generate_main_ts`。
```

---

## 4. 模式库设计

### 4.1 Pattern 文件结构

每个 pattern 文件遵循统一格式：

```markdown
# Pattern: 列表-详情布局

## 适用场景
需要展示一个列表，点击列表项在右侧/弹窗显示详情。

## 标准 .at 结构

[完整的 .at 代码示例]

## 关键约束
- 列表项使用 `for x in .items`，key 自动生成
- 详情区用 `if .items.len() > 0` 防空
- 列表和详情可以通过 store 共享 active_id

## 常见变体
### 变体 A: 详情在右侧（主从布局）
[code]

### 变体 B: 详情在弹窗
[code]

## 陷阱
- 列表项如果是有状态组件（editor），见 [editor-integration.md]
- active_id 跨组件传递时，优先用 store，不用 props 链

## 验证
生成后检查：
- [ ] 列表项的 key 是唯一的
- [ ] 详情区有空状态处理
- [ ] store 的 active_id 正确更新
```

### 4.2 首批模式（从 015-notes 提炼）

| Pattern | 来源 | 复杂度 |
|---------|------|--------|
| list-detail | 015-notes 的 NavTree + EditorPanel | 中 |
| tree-navigation | 015-notes 的文件夹分组导航 | 中 |
| editor-integration | 015-notes 的 AutoDownEditor 集成 | 高 |
| store-pattern | 015-notes 的 NotesStore | 中 |
| dark-mode | 015-notes 的 ToggleDarkMode | 低 |
| tag-filter | 015-notes 的标签筛选 | 低 |
| hover-interaction | 015-notes 的 pin/tag hover 显示 | 低 |

---

## 5. 交互式向导：`auto ui wizard`

### 5.1 新项目向导

```bash
$ auto ui wizard new
? Project name: my-app
? App type: ( ) CRUD  ( ) Dashboard  ( ) Notes  (×) Custom
? Layout: (×) List-detail  ( ) Tabs  ( ) Wizard  ( ) Free-form
? Features: [×] Dark mode  [ ] Auth  [×] Tags  [ ] Search
? Backend: (×) Rust axum  ( ) None  ( ) External

Generating...
✓ src/front/app.at (list-detail layout)
✓ src/front/sidebar.at (NavTree widget)
✓ src/front/editor.at (EditorPanel widget)
✓ src/front/store.at (AppStore)
✓ src/back/api.at (CRUD endpoints)
✓ src/back/db.at (seed data)
✓ pac.at

Next steps:
  auto run    # 启动开发服务器
```

向导基于 Skill 的模式库，**只生成经过验证的组合**。

### 5.2 添加组件向导

```bash
$ auto ui wizard add widget
? Widget name: CommentBox
? Based on pattern: (×) CRUD form  ( ) Display only  ( ) Editor
? Needs store: (×) Yes  ( ) No
? Which store: AppStore

Generating CommentBox widget...
✓ src/front/comment_box.at
✓ Updated src/front/app.at (added `use comment: CommentBox`)
✓ Updated src/front/store.at (added comment state + handlers)

Validation:
✓ No warnings
```

---

## 6. Skill 的自更新机制

### 6.1 从 Bug 中学习

每次发现新的陷阱模式（通过 Plan 361 的校验或人工调试），流程化地更新 Skill：

```
发现 bug
  ↓
诊断根因
  ↓
是生成器契约违反吗？
  ├─ 是 → 写进 generator-contracts.md + 加校验规则（Plan 361）
  └─ 否 → 是已知反模式吗？
      ├─ 是 → 更新 known-pitfalls.md
      └─ 否 → 新增 pitfall 条目
```

### 6.2 模式库的版本化

模式文件用 frontmatter 标注兼容性：

```markdown
---
version: 1.2
generator_version: ">=3.0"
autodown_version: ">=0.1.5"
validated_with: ["015-notes", "playground"]
last_reviewed: 2026-07-21
---
```

Skill 加载模式时检查版本兼容性，过时的模式会警告。

---

## 7. 实施计划

### Phase 1: 知识收集（1-2 天）
- [ ] 从本次会话提炼所有契约和陷阱（已起草：C1-C9, P1-P5）
- [ ] 审查 015-notes 的所有 .at 文件，提炼成功模式
- [ ] 审查生成器源码，列出所有隐式假设
- [ ] 输出 `generator-contracts.md` 初版（含 C1-C9）

### Phase 2: Skill 骨架（1 天）
- [ ] 创建 `.claude/skills/autoui/` 或 `crates/autoui-skill/`
- [ ] 写 SKILL.md 主入口（含 7 步 workflow，含测试维护）
- [ ] 整理 reference / patterns / templates 目录结构

### Phase 3: 核心模式（2-3 天）
- [ ] list-detail pattern（最常用）
- [ ] editor-integration pattern（最容易出错）
- [ ] store-pattern（含 accent_color 自动注入）
- [ ] dark-mode pattern（含 C8 CSS 变量作用域陷阱）
- [ ] 每个 pattern 配完整可运行示例

### Phase 4: 测试集成（1-2 天）⚠️ 新增
- [ ] Skill 生成新功能时，自动在 acceptance.atd 加 T 条目
- [ ] Skill 生成对应的 .spec.ts 骨架
- [ ] 修 bug 时 Skill 强制生成回归测试（含"历史教训"标注）
- [ ] 集成 Plan 361 的校验 + Plan 366a 的测试运行

### Phase 5: 向导工具（3-5 天，可选）
- [ ] `auto ui wizard new` 命令
- [ ] `auto ui wizard add widget` 命令（含测试骨架生成）
- [ ] 基于交互式 prompt 生成脚手架
- [ ] 集成 Plan 361 的校验

### Phase 6: 集成与文档（1 天）
- [ ] SKILL.md 引用 Plan 361/362/366 的工具链
- [ ] 在项目 README 推荐使用 Skill
- [ ] 记录"何时用 Skill"的决策树

---

## 8. 验收标准

### 质量指标

- [ ] Skill 覆盖本次会话的所有陷阱（P1-P5）
- [ ] generator-contracts.md 列出生成器的所有隐式假设（C1-C9，含 C7/C8/C9 新增）
- [ ] 模式库包含至少 7 个经过验证的模式（对应 015-notes 的核心结构）
- [ ] 用 Skill 生成一个新的 list-detail 应用，无需手动调试即可运行
- [ ] **Skill 生成新功能时，自动产出 acceptance.atd 条目 + .spec.ts 骨架**
- [ ] **Skill 修 bug 时，自动产出回归测试**

### 可衡量的改进

- [ ] AI 使用 Skill 后，生成的 .at 代码首次校验通过率 > 90%（无 Plan 361 警告）
- [ ] 新项目从零到可运行的步骤数减少 50%（向导 vs 手动）
- [ ] "操作失效"类 bug 在后续开发中复发率下降（通过 T1-T13 + 回归测试监测）
- [ ] **新功能的测试覆盖率：每个新 T 条目都有对应的 .spec.ts**


---

## 9. 与 Plan 361/362/366 的协同

```
Plan 361 (校验)          Plan 363 (Skill)
生成后检查  ←───────────  生成时引导
     ↑                          ↑
     │                          │
     └───── 同一套契约 ──────────┘
              (generator-contracts.md: C1-C9)

Plan 362 (快速反馈)
.auto watch 实时校验 ←── 触发 Skill 的 trap 检测

Plan 366 (测试套件)
acceptance.atd + .spec.ts ←── Skill 生成新功能时自动产出
        ↓
auto test:ui 跑测试 ←── Skill workflow 的第 7 步
```

四个计划形成闭环：
- **Plan 361** 是**后置静态防线**（生成后检查 SFC 文本）
- **Plan 362** 是**加速器**（让发现问题的成本变低）
- **Plan 363** 是**前置防线**（让问题根本不被生成）+ **测试维护者**（产出契约和测试骨架）
- **Plan 366** 是**运行时防线**（跨平台测试执行，目前 366a 用 Playwright，未来用 Auto 测试 DSL）

理想流程：
```
开发者用 Skill 生成代码 (Plan 363)
  → Skill 同步产出 acceptance.atd 条目 + .spec.ts 骨架 (Plan 363 §3.4)
  → auto watch 实时重建 + 静态校验 (Plan 362 + 361)
  → auto test:ui 跑运行时测试 (Plan 366a)
  → 发现新陷阱 → 反哺 Skill 知识库 (Plan 363 自更新)
     - 加 known-pitfall 条目
     - 加 generator-contract 条目
     - 加回归测试（标注历史教训）
```

