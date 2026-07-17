# Plan 357: 015-notes Pin 图标 / 目录操作 / Tag 编辑 / Dark Mode / 主题色 (v4)

> **类型**: 功能增强
> **状态**: 实施中
> **日期**: 2026-07-17
> **设计文档**: [Design 19](../design/19-theming-and-dark-mode.md)

---

## v4 待做（UX 改进）

### 1. Tag 编辑独立于 Edit 模式

**问题**：tag 只能在 Edit 模式下修改（输入框 + "+" 按钮）。

**期望**：
- 文章 tag 列表 hover 时，末尾出现 "+" 按钮，点击弹出 tag 输入
- 每个 tag hover 时，右侧出现 "×" 按钮，点击删除该 tag
- 不需要进入 Edit 模式

**实现**：
- editor.at：tag 行始终显示，用 `group-hover` 实现 hover 显示 +/×
- tag 输入框用一个 `show_tag_input` 状态控制（点击 + 时切换为 true）
- 删除 tag 直接调 `RemoveTag(t)` handler

**约束**：Auto view DSL 不支持 CSS `group-hover`（那是 Tailwind 的 `group` + `group-hover:` 机制，需要特定 HTML 结构）。退回方案：tag 行始终显示 tag badge（可点击删除）+ 一个 "+" 按钮始终可见。

### 2. 新增 tag 后导航栏 tag 列表不更新

**问题**：左侧导航栏的 tag 按钮是硬编码的（intro/ideas/home/work），新增 tag 后不显示。

**根因**：view DSL 不支持遍历 `note.tags` 嵌套 for（parser 限制 + OOM），所以无法动态收集所有 tag。

**解决方案**：
- Store 里加 `all_tags []str` 状态
- Init / UpdateTags / AddTag / RemoveTag 后重新计算 all_tags
- 导航栏用 `for tag in .store.all_tags` 遍历

**风险**：store handler 里的 `for` 循环 + 数组操作可能触发解析问题（之前 `all_tags` 被移除正是因为此）。需要验证。

### 3. Pin 操作改为标题 hover 图标

**问题**：Pin 仍需点击文章下方的 Pin 按钮。

**期望**：
- 标题旁始终显示一个 pin 图标
- 未 pinned：灰色/透明图标
- 已 pinned：彩色图标（primary 色）
- 点击图标切换 pin 状态
- 移除文章下方的 Pin 按钮

**实现**：
- editor.at：标题行加一个 pin 按钮（文字 "📌" 或 "📍"）
- `style:if .note.pinned` 控制颜色（pinned = primary 色，未 pinned = muted 色）
- `onclick: .TogglePin`
- 移除工具栏的 Pin 按钮

---

## 文件改动

| 文件 | 改动 |
|---|---|
| `editor.at` | Tag 行：始终显示 + 删除 × + 添加输入；标题行：pin 图标按钮；移除工具栏 Pin 按钮 |
| `notes_store.at` | 加 `all_tags []str` 状态 + 计算逻辑 |
| `app.at` | 导航栏 tag 列表：改用 `for tag in .store.all_tags` |

---

## 已完成（v1-v3）

- ✅ Pin 📌 emoji（编辑器标题旁）
- ✅ Tag 筛选（`note.tags.includes()`）
- ✅ Tag 编辑（编辑模式输入框）
- ✅ 根目录 + 文件夹 "+" 按钮
- ✅ Dark Mode（生成器注入 `:class="{ dark: store.dark_mode }"`）
- ✅ 语义化 token（bg-primary, text-foreground 等）
- ✅ 后端 seed data 从 db.at 读取
- ✅ create_note 加 folder 参数
