# Plan 360: 015-notes UI 现代化 + 主题色切换

> **目标**: 将 015-notes 从简陋的功能原型升级为视觉精致、交互流畅的现代笔记应用，并引入与 auto-forge 一致的 5 色主题色切换系统。

---

## 1. 现状分析

### 1.1 当前 UI 问题（基于截图分析）

| # | 问题 | 影响 |
|---|------|------|
| P1 | **整体过于平淡** — 全白背景 + 窄边框，缺少层次感和视觉锚点 | 显得廉价、像 demo 而非成品 |
| P2 | **侧边栏与编辑区无视觉分隔** — 仅靠 `border-border` 细线，无阴影/圆角/间距 | 空间感不足，左右区域混为一体 |
| P3 | **Header 太简陋** — 只有一行文字 + 按钮，无 logo/图标/品牌感 | 缺少专业感 |
| P4 | **笔记列表项信息密度低** — 只有标题，无时间预览/摘要/置顶标记 | 列表浏览效率低 |
| P5 | **空状态缺失** — 无笔记时编辑区完全空白 | 用户体验断裂 |
| P6 | **Tag 显示粗糙** — 纯色块 + 文字，无现代 pill/chip 设计 | 视觉不一致 |
| P7 | **暗色模式体验粗糙** — 仅反转颜色，无平滑过渡 | 切换时闪烁，观感差 |
| P8 | **按钮风格不统一** — primary/secondary/destructive/gost 混用，无设计语言 | 视觉噪音 |
| P9 | **缺少微交互** — 无 hover 动效、无过渡动画 | 交互僵硬 |
| P10 | **主题色单一** — primary 固定为深蓝/近黑，无法自定义 | 个性化缺失 |

### 1.2 auto-forge 主题色系统（参考标准）

auto-forge 的主题色配置由 `useAccentColor.ts` + `SettingsMenu.vue` 实现：

**5 色预设**（与 auto-forge 完全对齐）：

| 名称 | brand1 (hex) | primaryHsl (shadcn) | 说明 |
|------|-------------|---------------------|------|
| **indigo** | `#6366f1` | `239 84% 67%` | 默认主色 |
| **coral** | `#e85d75` | `350 75% 64%` | 暖色系 |
| **ocean** | `#3b82f6` | `217 91% 60%` | 蓝色系 |
| **sage** | `#10b981` | `160 84% 39%` | 绿色系 |
| **amber** | `#f59e0b` | `38 92% 50%` | 金色系 |

**核心机制**：
- 主题色选择后，通过 `document.documentElement.style.setProperty('--primary', hsl)` 动态注入 CSS 变量
- 持久化到 `localStorage`，key 为 `notes-accent-color`
- UI 为 5 个圆形色板按钮，选中态有环形高亮

### 1.3 015-notes 当前语义 token 系统

已有完整的 shadcn 语义 token 体系（`bg-background`、`text-foreground`、`bg-primary` 等），定义在 `gen/front/vue/src/assets/index.css` 的 `:root` 和 `.dark` 中。**这套系统是好的基础**，Plan 360 的所有样式改进都将继续使用语义 token，不引入硬编码颜色。

---

## 2. 设计方案

### 2.1 整体设计语言

**设计理念**：受 Notion / Bear / Apple Notes 启发的现代笔记应用风格。

**关键设计原则**：
1. **卡片化布局** — 侧边栏和编辑区作为独立"卡片"，有微妙阴影和间距
2. **充足留白** — 增加内边距，让内容呼吸
3. **层次感** — 通过阴影、透明度叠加、微妙的背景色差异建立视觉层次
4. **圆角统一** — 全局 `rounded-xl` (12px) 为基础圆角
5. **平滑过渡** — 所有交互元素加 `transition-colors duration-200`
6. **主题色驱动** — primary 色由用户选择，贯穿整个 UI

### 2.2 布局改进

```
改前:
┌─────────────────────────────────────┐
│ Notes                    [+ New]    │  ← 平淡的 header
├──────────┬──────────────────────────┤
│ All Pin Rec│                        │
│ [search]   │                        │
│ [tags]     │     编辑区(全白)        │
│ • Note 1   │                        │
│ • Note 2   │                        │
│ • Note 3   │                        │
│            │                        │
│ ────────   │                        │
│ Dark Mode  │                        │
└──────────┴──────────────────────────┘

改后:
┌──────────────────────────────────────────────────┐
│  📝 Notes              [🎨 accent]    [+ New]    │  ← 带 emoji 图标 + 主题色按钮
├────────────────┬─────────────────────────────────┤
│ ┌────────────┐ │ ┌─────────────────────────────┐ │
│ │ All Pin Rec│ │ │  Note Title          📌     │ │
│ │ 🔍 Search  │ │ │  2024-01-15 · #work #idea  │ │
│ │            │ │ │                             │ │
│ │ 📁 Notes   │ │ │  Markdown content...       │ │
│ │  · Note 1  │ │ │                             │ │
│ │  · Note 2  │ │ │                             │ │
│ │ 📁 Work    │ │ │                             │ │
│ │  · Note 3  │ │ │                             │ │
│ │            │ │ │  [Edit]        [Delete]    │ │
│ │ 🌙 / ☀     │ │ └─────────────────────────────┘ │
│ └────────────┘ │                                 │
└────────────────┴─────────────────────────────────┘
     ↑ 卡片化: bg-card + shadow-sm + rounded + gap
```

### 2.3 具体组件改进

#### A. Header（app.at）
- 添加笔记 emoji 📝 作为 logo
- "+ New" 按钮改为更圆润的 `rounded-full` + 主题色背景
- 在右侧添加主题色切换按钮（圆形色板入口）

#### B. NavTree 侧边栏（sidebar.at）
- 整体包裹为卡片：`bg-card rounded-xl shadow-sm m-2`
- View tabs 改为 segmented control 风格（圆角容器 + 滑块高亮）
- 搜索框增加左侧搜索图标（用 emoji 🔍 或 CSS）
- 文件夹标题改为带文件夹 emoji 📁 的标题行
- 笔记列表项增加：时间预览（`text-xs text-muted-foreground`）、置顶标记（📌 图标）
- Dark Mode 按钮改为月亮/太阳 emoji 切换（🌙/☀），放在更自然的位置

#### C. EditorPanel（editor.at）
- 整体包裹为卡片：`bg-card rounded-xl shadow-sm m-2`
- 标题区域增加更多留白
- Tag pills 改为现代 chip 设计：`rounded-full bg-primary/10 text-primary`
- 按钮统一设计语言：
  - Primary: `bg-primary text-primary-foreground rounded-lg shadow-sm hover:bg-primary/90`
  - Secondary: `bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/80`
  - Ghost: `text-muted-foreground hover:bg-accent rounded-lg`
  - Destructive: `bg-destructive/10 text-destructive rounded-lg hover:bg-destructive/20`

#### D. 空状态
- 无笔记时显示居中引导：大 emoji 📝 + "Create your first note" + New 按钮

#### E. 暗色模式优化
- 在 `index.css` 的 `*` 选择器或 `body` 上添加 `transition: background-color 0.3s, color 0.3s`
- 调整暗色模式的阴影（dark mode 下阴影应更深、透明度更低）

---

## 3. 主题色切换系统设计

### 3.1 Store 扩展（notes_store.at）

在 `NotesStore` 中新增主题色状态：

```auto
model {
    // ... existing ...
    var accent_color str = "indigo"    // indigo | coral | ocean | sage | amber
}
```

新增 handler：
```auto
.SetAccent(name) -> {
    .accent_color = name
}
```

### 3.2 CSS 变量注入方案

**关键设计决策**：主题色的 CSS 变量注入需要在运行时完成，而不是编译时。

方案：在 `App.vue` 的 `onMounted` 中，根据 `store.accent_color` 动态设置 `document.documentElement.style.setProperty('--primary', hslValue)`。

需要新增一个 **accent palette 映射** — 这是一段纯前端 JS 逻辑，通过 Vue generator 注入到 `App.vue` 的 `<script setup>` 中。

**实现方式**：在 `vue.rs` 的 store composable 生成中，当检测到 `accent_color` 状态变量时，自动注入 accent palette 数据和 apply 逻辑。

### 3.3 主题色选择器 UI

在 Header 右侧添加一个调色板图标按钮，点击后弹出 5 色选择面板。

由于 Auto DSL 目前不支持 dropdown/popover 原语，采用**简化方案**：
- 在 NavTree 底部（Dark Mode 按钮旁）添加一个内联的主题色选择区
- 5 个小圆形色板按钮横排，选中有环形高亮

```
┌──────────────────────┐
│  ●  ●  ●  ●  ●       │ ← 5 色板 (indigo/coral/ocean/sage/amber)
│  🌙 Dark Mode        │
└──────────────────────┘
```

### 3.4 Accent Palette 数据

与 auto-forge 对齐的 5 色定义（注入到 store composable 或 App.vue）：

```typescript
const ACCENT_PALETTES: Record<string, { brand1: string; primaryHsl: string }> = {
    indigo: { brand1: '#6366f1', primaryHsl: '239 84% 67%' },
    coral:  { brand1: '#e85d75', primaryHsl: '350 75% 64%' },
    ocean:  { brand1: '#3b82f6', primaryHsl: '217 91% 60%' },
    sage:   { brand1: '#10b981', primaryHsl: '160 84% 39%' },
    amber:  { brand1: '#f59e0b', primaryHsl: '38 92% 50%' },
}
```

---

## 4. 实现计划（分阶段）

### Phase 1: CSS 基础增强（index.css + tailwind.config）

**目标**：建立视觉层次基础，不改变 .at 逻辑。

改动文件：
- `gen/front/vue/src/assets/index.css` — 添加过渡动画、调整暗色阴影
- `gen/front/vue/tailwind.config.cjs` — 添加自定义阴影/动画配置

具体：
1. `body` 添加 `transition: background-color 0.3s ease, color 0.3s ease`
2. 暗色模式调整：背景从纯黑改为微妙的深蓝灰 (`222.2 47% 6%` → `222.2 47% 8%`)，减少对比度疲劳
3. 添加自定义 shadow 工具类：`.shadow-card { box-shadow: 0 1px 3px rgba(0,0,0,0.08), 0 1px 2px rgba(0,0,0,0.06); }`

> **注意**：`index.css` 和 `tailwind.config.cjs` 是生成文件。需要确认它们是否由 auto-man 模板生成。如果是，则需要修改模板（auto-man 中的模板字符串）。

### Phase 2: 布局卡片化（app.at + sidebar.at + editor.at）

**目标**：将三大区域改为卡片化布局，增加视觉层次。

#### 2a. app.at — Header 美化 + 卡片间距
```auto
view {
    col {
        style: "w-full h-screen flex-col bg-muted/30 p-2 gap-2"

        // header (卡片化)
        row {
            style: "items-center justify-between px-4 py-3 bg-card rounded-xl shadow-sm"
            row {
                style: "items-center gap-2"
                text "📝" { style: "text-xl" }
                h2 "Notes" { style: "text-lg font-bold text-foreground" }
            }
            button "+ New Note" {
                onclick: .NewNote
                style: "px-4 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors"
            }
        }

        // body (卡片化)
        row {
            style: "flex-1 gap-2"
            NavTree(...)
            col {
                style: "flex-1"
                if .store.notes.len() > 0 {
                    EditorPanel(...)
                } else {
                    // 空状态
                    col {
                        style: "flex-1 items-center justify-center bg-card rounded-xl"
                        text "📝" { style: "text-6xl mb-4 opacity-50" }
                        text "No notes yet" { style: "text-lg text-muted-foreground" }
                        button "Create your first note" {
                            onclick: .NewNote
                            style: "mt-4 px-6 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium hover:bg-primary/90 transition-colors"
                        }
                    }
                }
            }
        }
    }
}
```

#### 2b. sidebar.at — NavTree 卡片化 + 列表项增强
- 根容器改为 `bg-card rounded-xl shadow-sm flex flex-col h-full overflow-hidden`
- 笔记列表项增加时间摘要和置顶标记
- 底部增加主题色选择器 + Dark Mode 切换

#### 2c. editor.at — EditorPanel 卡片化 + 按钮统一
- 根容器改为 `bg-card rounded-xl shadow-sm flex flex-col h-full overflow-hidden`
- 统一所有按钮为设计语言
- Tag pills 改为 `bg-primary/10 text-primary`

### Phase 3: 主题色切换系统

#### 3a. Store 扩展
- `notes_store.at` 添加 `accent_color` 状态 + `SetAccent` handler

#### 3b. Accent 注入逻辑（vue.rs 生成器）
- 在 store composable 生成中，当检测到 `accent_color` 状态时注入：
  - `ACCENT_PALETTES` 数据
  - `applyAccent()` 函数（设置 CSS 变量）
  - `onMounted` 中自动 apply

#### 3c. 主题色选择器 UI（sidebar.at）
- 5 个圆形色板按钮
- 点击调用 `.SetAccent(name)` → store 更新 → applyAccent 注入 CSS

#### 3d. localStorage 持久化
- 在 store composable 的 `Init` handler 中读取 localStorage
- 在 `SetAccent` 中写入 localStorage

### Phase 4: 暗色模式 + 暗色主题色适配

#### 4a. 暗色模式过渡
- 确保所有元素有 `transition-colors`
- 暗色模式下阴影调整为更深

#### 4b. 暗色主题色亮度补偿
- 暗色模式下 primary 应稍亮（auto-forge 在 `.dark` 中调整了 primary 亮度）
- 在 `applyAccent` 中根据 `dark_mode` 状态调整 HSL lightness (+3-5%)

### Phase 5: 清理 + 验证

- 删除废弃的 `note_item.at`（已不被使用）
- 更新 README
- playwright-cli 全面验证（亮/暗模式 × 5 主题色 = 10 种组合截图）

---

## 5. 技术约束与注意事项

### 5.1 生成文件 vs 源文件

| 文件 | 类型 | 修改方式 |
|------|------|----------|
| `src/front/*.at` | **源文件** | 直接编辑 |
| `gen/front/vue/src/assets/index.css` | **生成文件** | 修改 auto-man 模板 |
| `gen/front/vue/tailwind.config.cjs` | **生成文件** | 修改 auto-man 模板 |
| `gen/front/vue/src/stores/useNotesStoreStore.ts` | **生成文件** | 修改 vue.rs store composable 生成 |

> **重要**：不能直接改生成文件，否则 `auto build` 会覆盖。需要找到 auto-man 中对应的模板代码。

### 5.2 Auto DSL 限制

当前 Auto DSL 不支持：
- ❌ Dropdown/Popover 组件 → 用内联色板替代
- ❌ 运行时 JS 注入 → 通过 store composable 生成的 TS 代码实现
- ❌ 直接写 CSS → 只能用 Tailwind class

### 5.3 性能考虑

- 主题色切换只改 CSS 变量，不触发组件重渲染
- `transition-colors` 可能影响首次渲染性能 → 只加在交互元素上，不加在全局 `*`

---

## 6. 验收标准

- [ ] 亮色模式下 UI 有清晰的视觉层次（卡片 + 阴影 + 留白）
- [ ] 暗色模式平滑过渡，无闪烁
- [ ] 5 种主题色全部正确生效（primary 按钮颜色变化）
- [ ] 主题色持久化（刷新页面后保持）
- [ ] 笔记列表项显示时间摘要和置顶标记
- [ ] 空状态有引导界面
- [ ] 按钮风格统一（primary/secondary/ghost/destructive）
- [ ] playwright-cli 截图验证 10 种组合（2 模式 × 5 主题色）
