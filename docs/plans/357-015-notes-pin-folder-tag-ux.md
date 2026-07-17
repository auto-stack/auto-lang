# Plan 357: 015-notes Pin 图标 / 目录操作 / Tag 编辑 / Dark Mode (v2)

> **类型**: 功能增强
> **状态**: 实施中
> **日期**: 2026-07-17

## v2 新增问题（2026-07-17）

1. Pin 显示文字 "Pinned" 而非图标 → 改用 emoji 标记
2. Tag 筛选无效（点击 tag 后列表全空）→ 修复筛选逻辑
3. Tag 添加后导航栏不更新 → tag 列表需要动态化
4. 根目录无 "+" 按钮 → 添加
5. 无新建目录功能 → 暂缓（需后端 Folder CRUD，a2r 不支持）
6. Dark Mode 按钮无效 → 通过 CSS class 切换实现

## 修复方案

### 1. Pin → 用 emoji 而非文字
- 编辑器标题：`📌` 替代 "Pinned" 文字
- 移除 editor.at 的 Pin 按钮（用标题旁的图标代替）
- 导航栏笔记标题前：pinned 加 `📌`

### 2. Tag 筛选修复
- 当前 bug：`if .store.active_tag == ""` 控制显示——选了 tag 后 `active_tag != ""` 导致全隐藏
- 修复：改为 `if .store.active_tag == "" || note.tags.contains(.store.active_tag)`
- 但 view DSL 不支持 `||`——用两个 `if` 嵌套替代

### 3. 根目录 "+" 按钮
- 在根目录笔记列表前加一个 "+" 按钮（无文件夹标题行，直接放在树顶部）

### 4. Dark Mode
- 根元素用 `style:if` 失败（生成器不支持根元素条件 class）
- 改法：Dark Mode handler 里用 JS eval 切换 document class
- 但 Auto store handler 不能调 JS——退回：在 App.vue 手动加 dark CSS 支持
- **实际方案**：Dark Mode 暂时只切换 store 状态，加一个 `dark` class 到 body 通过 eval
