# Plan 103: AutoUI Component Gallery Documentation Site

## Context

AutoUI 需要一个组件库文档站点来展示其能力，类似于 shadcn-vue 的文档风格。该站点将：
- 展示 Auto 源码和转译后的 Vue 代码
- 提供实时组件预览
- 支持代码复制功能
- 后续可升级为交互式 Playground

**用户决策**:
- 站点类型: 组合方案（完整组件库 + 示例应用 + 后续 Playground）
- 组件范围: 阶段 1 基础组件（Button, Input, Text, Card, Badge, Label, Accordion, Tabs）
- 技术架构: 扩展现有 `api-example` 的 Auto 风格
- Playground: 渐进式（先静态代码块 + 复制按钮）

---

## Goals

1. 构建文档站点展示 AutoUI 组件
2. 每个组件提供可复制的 Auto 源码
3. 显示转译后的 Vue 代码供学习对比
4. 渲染实时 Vue 组件预览
5. 支持渐进式增强（先静态，后交互式 Playground）

---

## Site Structure

```
examples/component-gallery/
├── pac.at                      # Workspace root config
├── source/
│   └── front/
│       ├── pac.at              # Frontend package config
│       ├── app.at              # Main app with sidebar navigation
│       ├── components/         # Auto component definitions
│       │   ├── button.at
│       │   ├── input.at
│       │   ├── card.at
│       │   ├── badge.at
│       │   ├── accordion.at
│       │   └── tabs.at
│       └── pages/              # Documentation pages
│           ├── index.at        # Home/Overview
│           ├── button.at       # Button documentation
│           ├── input.at        # Input documentation
│           ├── card.at         # Card documentation
│           └── ...
└── generated/                  # Generated Vue output
    └── vue/
        ├── src/
        │   ├── components/
        │   │   ├── ui/         # shadcn-vue components
        │   │   └── docs/       # Documentation components
        │   └── pages/          # Page components
        ├── App.vue
        └── main.ts
```

---

## Component Scope (Phase 1)

| Component | Description | Props |
|-----------|-------------|-------|
| Button | Clickable button | text, onclick, variant, disabled |
| Input | Text input | value, placeholder, onchange, type |
| Text | Text display | content (inline or interpolated) |
| Card | Container with header/content | title, variant |
| Badge | Status indicator | text, variant |
| Label | Form label | text, for |
| Accordion | Collapsible sections | items, defaultOpen |
| Tabs | Tab navigation | items, defaultTab |

---

## Implementation Phases

### Phase 1: Project Infrastructure (2-3 days) ✅ COMPLETE

**Goal**: Set up workspace structure and build pipeline.

**Tasks**:
- [x] Create directory structure
- [x] Write `pac.at` workspace config
- [x] Write `source/front/pac.at` frontend config
- [x] Create main `app.at` with sidebar navigation layout
- [x] Set up Vue project generation pipeline

**Files to Create**:
```
examples/component-gallery/pac.at
examples/component-gallery/source/front/pac.at
examples/component-gallery/source/front/app.at
```

**Verification**:
- `auto.exe vue examples/component-gallery` generates valid Vue project
- Generated project runs with `npm run dev`

---

### Phase 2: Core Components Definition (3-4 days) ✅ COMPLETE

**Goal**: Define all 8 Phase 1 components in Auto.

**Tasks**:
- [x] Create `components/button.at` with variants
- [x] Create `components/input.at` with types
- [x] Create `components/text.at` with interpolation
- [x] Create `components/card.at` with slots
- [x] Create `components/badge.at` with variants
- [x] Create `components/label.at`
- [x] Create `components/accordion.at` with state
- [x] Create `components/tabs.at` with state

**Files to Create**:
```
examples/component-gallery/source/front/components/button.at
examples/component-gallery/source/front/components/input.at
examples/component-gallery/source/front/components/text.at
examples/component-gallery/source/front/components/card.at
examples/component-gallery/source/front/components/badge.at
examples/component-gallery/source/front/components/label.at
examples/component-gallery/source/front/components/accordion.at
examples/component-gallery/source/front/components/tabs.at
```

**Verification**:
- All components transpile without errors
- Generated Vue code compiles

---

### Phase 3: Documentation Pages (3-4 days) ✅ COMPLETE

**Goal**: Create documentation pages for each component.

**Tasks**:
- [x] Create `pages/index.at` - Home page with component overview
- [x] Create `pages/button.at` - Button documentation
- [x] Create `pages/input.at` - Input documentation
- [x] Create `pages/card.at` - Card documentation
- [x] Create `pages/badge.at` - Badge documentation
- [x] Create `pages/label.at` - Label documentation
- [x] Create `pages/accordion.at` - Accordion documentation
- [x] Create `pages/tabs.at` - Tabs documentation

**Files to Create**:
```
examples/component-gallery/source/front/pages/index.at
examples/component-gallery/source/front/pages/button.at
examples/component-gallery/source/front/pages/input.at
examples/component-gallery/source/front/pages/card.at
examples/component-gallery/source/front/pages/badge.at
examples/component-gallery/source/front/pages/label.at
examples/component-gallery/source/front/pages/accordion.at
examples/component-gallery/source/front/pages/tabs.at
```

**Verification**:
- All pages transpile correctly
- Navigation between pages works

---

### Phase 4: Code Display Components (2-3 days) ✅ COMPLETE

**Goal**: Create components for displaying code with copy functionality.

**Tasks**:
- [x] Create `CodeBlock` widget for syntax-highlighted code
- [x] Create `CopyButton` widget for clipboard copy
- [x] Create `CodeTabs` widget for switching Auto/Vue code
- [x] Integrate with documentation pages

**Files to Create**:
```
examples/component-gallery/source/front/components/code_block.at
examples/component-gallery/source/front/components/copy_button.at
```

**Verification**:
- Code displays with syntax highlighting
- Copy button copies code to clipboard

---

### Phase 5: Navigation and Routing (2-3 days) ✅ COMPLETE

**Goal**: Implement sidebar navigation and page routing.

**Tasks**:
- [x] Create `Sidebar` widget with component list
- [x] Create `NavLink` widget for navigation items
- [x] Implement client-side routing logic
- [x] Add active state tracking

**Files to Create**:
```
examples/component-gallery/source/front/components/sidebar.at
examples/component-gallery/source/front/components/nav_link.at
```

**Verification**:
- Navigation between pages works
- Active state shows correctly

---

### Phase 6: Build and Deploy Pipeline (1-2 days) ✅ COMPLETE

**Goal**: Set up build process and deployment configuration.

**Tasks**:
- [x] Configure Vite build settings
- [x] Add GitHub Pages deployment workflow
- [x] Create npm scripts for build/deploy
- [x] Test production build (pending transpiler readiness)

**Files to Create**:
```
.github/workflows/deploy-component-gallery.yml
```

**Verification**:
- Production build succeeds
- Site deploys to GitHub Pages

---

## Critical Files Reference

| File | Purpose |
|------|---------|
| `examples/api-example/pac.at` | Workspace config pattern to follow |
| `examples/api-example/source/front/app.at` | Main widget pattern |
| `crates/auto-lang/src/ui_gen/vue.rs` | Vue generator reference |
| `docs/plans/099-shadcn-vue-migration.md` | Component mapping reference |
| `docs/plans/098-aura-schema.md` | AURA schema reference |

---

## Dependencies

### Existing Dependencies
- AURA → Vue transpiler (Plan 096, 098, 099)
- shadcn-vue component mappings
- Workspace configuration system

### New npm Dependencies
- vue ^3.4.0
- vue-router ^4.0.0 (optional)
- @vueuse/core ^10.0.0
- shadcn-vue components
- prism.js or shiki (syntax highlighting)

---

## Success Criteria

1. **Functionality**
   - All 8 components display correctly
   - Code blocks show Auto source and Vue output
   - Copy button copies code to clipboard
   - Navigation between pages works
   - Component previews render live

2. **Quality**
   - Generated Vue code compiles without errors
   - No console errors in browser
   - Mobile-responsive layout

3. **Developer Experience**
   - Single command: `auto.exe vue examples/component-gallery`
   - Hot reload during development
   - Clear documentation structure

---

## Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1 | 2-3 days | Project infrastructure |
| Phase 2 | 3-4 days | Component definitions |
| Phase 3 | 3-4 days | Documentation pages |
| Phase 4 | 2-3 days | Code display components |
| Phase 5 | 2-3 days | Navigation and routing |
| Phase 6 | 1-2 days | Build and deploy |
| **Total** | **13-19 days** | |

---

## Future Enhancements (Out of Scope)

1. **Interactive Playground** - Live code editing with hot reload
2. **Search** - Component search functionality
3. **Theming** - Dark/light mode toggle
4. **More Components** - Phase 2/3 components
5. **API Documentation** - Auto-generate from schema

---

## Known Issues (待修复)

### Phase 7: 下一步计划 ✅ 已完成

#### 1. 侧边栏导航切换页面内容 (高优先级) ✅ 已修复

**问题描述**:
当前侧边栏的导航按钮点击后能够更新 `currentPage` 状态，但主内容区域没有根据状态变化显示不同的组件文档页面。

**原因分析**:
- AURA → Vue 转译器对 `if/else if/else` 链式条件渲染的支持可能不完整
- 需要在 `view` 块中实现基于字符串比较的条件分支

**期望行为**:
```auto
view {
    row {
        // Sidebar
        col {
            button (text: "Button", onclick: .GoToButton) {}
            button (text: "Input", onclick: .GoToInput) {}
        }

        // Main content - 根据当前页面显示不同内容
        if .currentPage == "button" {
            ButtonDoc {}
        } else {
            if .currentPage == "input" {
                InputDoc {}
            } else {
                DefaultDoc {}
            }
        }
    }
}
```

**相关文件**:
- `crates/auto-lang/src/aura/extract.rs` - `extract_view_node()` 对 `ViewNode::Conditional` 的处理
- `crates/auto-lang/src/ui_gen/vue.rs` - `node_to_html()` 对 `AuraNode::Conditional` 的 Vue 模板生成
- `crates/auto-lang/src/parser.rs` - `parse_view_conditional()` 条件表达式解析

**修复方案**:
1. ✅ 添加字符串字面量解析支持到 `parse_condition_expr()` (TokenKind::Str)
2. ✅ 修复 Vue 生成器中双引号转义问题（使用单引号替代）
3. ✅ 更新 app.at 使用嵌套 if/else 条件渲染

**已修复 (2026-03-02)**:
- `parser.rs`: 添加 `TokenKind::Str` 支持解析 `"button"` 等字符串
- `vue.rs`: `convert_condition()` 将双引号转换为单引号避免 Vue 模板语法冲突
- `app.at`: 实现完整的 8 页面条件切换

#### 2. shadcn-vue 初始化问题 (中优先级)

**问题描述**:
首次运行 `npx shadcn-vue@latest add button` 时需要用户确认创建 `components.json`，导致组件可能没有正确添加。

**修复方案**:
- 在项目模板中预置 `components.json` 配置文件
- 或使用 `--yes` 标志自动确认

#### 3. 样式美化 (低优先级)

**问题描述**:
页面布局和样式需要进一步美化，匹配 shadcn-vue 文档站点的视觉效果。

**修复方案**:
- 添加 Tailwind CSS 类名到生成的 HTML
- 实现响应式侧边栏（可折叠）
- 添加代码语法高亮

---

## 里程碑记录

### 2026-03-02: Component Gallery 首次成功运行

**成就**:
- ✅ `auto.exe vue examples/component-gallery` 成功生成 Vue 项目
- ✅ 开发服务器成功启动 (`npm run dev`)
- ✅ 页面在浏览器中正常显示
- ✅ 侧边栏导航按钮可点击
- ✅ shadcn-vue Button 组件正常渲染
- ✅ 基础文档布局（标题、描述、预览区域、代码占位符）

**文件统计**:
- 24 个 `.at` 源文件
- 1 个 README.md
- 1 个 GitHub Actions workflow

**下一步**:
- ~~实现页面内容切换~~ ✅ 已完成
- 完善各组件的文档内容
- 添加真正的代码展示（而非占位符）

### 2026-03-02: 侧边栏导航内容切换实现

**成就**:
- ✅ 修复 `parse_condition_expr()` 支持字符串字面量解析
- ✅ 修复 Vue 生成器引号转义问题
- ✅ 实现 8 个组件页面的完整条件切换
- ✅ `npm run build` 成功构建生产版本

**修改文件**:
- `crates/auto-lang/src/parser.rs` - 添加 TokenKind::Str 支持
- `crates/auto-lang/src/ui_gen/vue.rs` - 修复 convert_condition() 引号问题
- `examples/component-gallery/source/front/app.at` - 完善条件渲染逻辑
