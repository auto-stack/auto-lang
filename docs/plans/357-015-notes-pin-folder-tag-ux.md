# Plan 357: 015-notes Pin 图标 / 目录操作 / Tag 编辑 / Dark Mode / 主题色 (v3)

> **类型**: 功能增强
> **状态**: 实施中
> **日期**: 2026-07-17
> **设计文档**: [Design 19](../design/19-theming-and-dark-mode.md)

---

## v3 更新（2026-07-17）

根据 Design 19（主题色方案），015-notes 的所有 `.at` 源码需要从硬编码颜色迁移到语义化 token。

### 已完成（v2）

- ✅ Pin 图标（📌 emoji，编辑器标题旁）
- ✅ Tag 筛选（`note.tags.contains()` → `note.tags.includes()`）
- ✅ Tag 编辑（编辑模式：输入框 + 添加 + 删除）
- ✅ 根目录 "+" 按钮
- ✅ 文件夹 "+" 按钮（新建该文件夹笔记）
- ✅ Dark Mode（生成器注入 `:class="{ dark: store.dark_mode }"`）

### v3 待做（Phase 1：语义化 token 迁移）

**目标**：把所有 `.at` 文件里的硬编码颜色替换为 shadcn 语义化 class。

映射表：
| 硬编码 | 语义化 | 出现位置 |
|---|---|---|
| `bg-white dark:bg-gray-900` | `bg-background` | app.at 根元素 |
| `text-gray-800 dark:text-gray-100` | `text-foreground` | app.at 根元素 |
| `bg-blue-500` | `bg-primary` | app.at + editor.at 按钮 |
| `text-white`（按钮文字） | `text-primary-foreground` | app.at + editor.at 按钮 |
| `bg-blue-50` | `bg-accent` | app.at 选中行 |
| `text-blue-700` | `text-accent-foreground` | app.at 选中行 |
| `text-gray-700` | `text-foreground` | app.at 列表项 |
| `text-gray-400` / `text-gray-500` | `text-muted-foreground` | app.at 文件夹标题 |
| `border-gray-200 dark:border-gray-700` | `border-border` | app.at 分隔线 |
| `dark:bg-gray-800` | （移除，bg-background 自动处理） | app.at header |
| `bg-gray-100` | `bg-muted` | app.at 标签按钮 |
| `bg-blue-100` | `bg-accent` | app.at 选中标签 |
| `bg-blue-500` | `bg-primary` | app.at 选中标签 |
| `bg-red-500` | `bg-destructive` | editor.at 删除按钮 |
| `bg-gray-500` | `bg-secondary` | editor.at Pin/Cancel 按钮 |
| `text-red-600` | `text-destructive` | app.at Clear 按钮 |

### 文件改动

| 文件 | 改动 |
|---|---|
| `app.at` | 所有颜色 class 替换为语义化 token；移除 `dark:` 前缀 |
| `editor.at` | 同上 |

### 验证

- `auto run` 生成无错误
- playwright-cli 截图：浅色模式 + 深色模式都正确
- 颜色与之前硬编码版本视觉一致（或更好）

---

## v2 问题清单（已全部解决）

1. ✅ Pin 显示 "Pinned" 文字 → 改用 📌 emoji
2. ✅ Tag 筛选无效 → 改用 `note.tags.contains()` 
3. ✅ Tag 添加后导航栏不更新 → tag 列表为硬编码，筛选使用 `note.tags.includes()`
4. ✅ 根目录无 "+" → 添加 "Notes" 标题 + "+" 按钮
5. ✅ Dark Mode 无效 → 修改生成器：检测 ToggleDarkMode handler，注入 `:class="{ dark: store.dark_mode }"`
6. 无新建目录功能 → 暂缓（需后端 Folder CRUD）

---

## v1 问题清单（已全部解决）

1. ✅ Pin 图标：编辑器标题旁 📌 + 导航栏分支
2. ✅ 文件夹新建：Work/Personal 标题旁 "+" 按钮
3. ✅ Tag 编辑：编辑模式输入框 + 添加 + 删除
4. ✅ create_note 加 folder 参数（db.at + api.at）
5. ✅ Store NewNoteInFolder(str) action
