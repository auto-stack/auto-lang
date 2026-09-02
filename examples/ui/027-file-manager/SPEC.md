# SPEC — 027-file-manager（Plan 440）

AutoOS 默认文件管理器应用（Finder / Explorer 双栏形态）。同一份 `src/front/app.at` 运行于 Vue 模式（`auto run`）与 VM / Iced 模式（`auto run -r vm`）。

---

## 1. 架构与布局设计 (Layout & Components)

界面采用现代 macOS Finder / GNOME Files 风格的响应式双栏布局：

```
+------------------------------------------------------------------------------------+
| [ < ] [ > ]  / root / Documents / Projects [🔍 搜索当前目录...] [ ⊞ 网格 | ≡ 列表 ] |
+------------------+-----------------------------------------------------------------+
| 快捷访问 (FAVORITES) |  名称        大小        类型       修改日期         操作   |
| 🏠 主目录 (Home)   | --------------------------------------------------------------- |
| 📁 文档 (Docs)     | 📁 Projects     --         文件夹     2026-08-30 11:20  [···]   |
| 📥 下载 (Downloads)| 📄 notes.txt    1.2 KB     文本文档   2026-08-31 16:45  [···]   |
| 🖼️ 图片 (Pictures) | 📊 report.xlsx  89.4 KB    电子表格   2026-08-20 18:00  [···]   |
| 💻 代码 (Projects) |                                                                 |
| 🗑️ 回收站 (Trash)   |                                                                 |
|                  |                                                                 |
| ---------------- |                                                                 |
| 存储占用 (Quota)  | 底部状态栏: 3 个项目 | 总计 90.6 KB | 选定: notes.txt (1.2 KB)        |
| [====---] 42/128G| [新建文件夹] [新建文件] [显示隐藏文件] [当前路径: /root/Documents]   |
+------------------+-----------------------------------------------------------------+
```

### 1.1 左侧导航栏 (Sidebar)
- **快捷访问分组 (Favorites / Quick Access)**:
  - `Home` (`/root`)
  - `Documents` (`/root/Documents`)
  - `Downloads` (`/root/Downloads`)
  - `Pictures` (`/root/Pictures`)
  - `Music` (`/root/Music`)
  - `Trash` (`/root/Trash`)
- **存储配额条 (Storage Quota)**:
  - 视觉化进度条指示磁盘使用情况（如 `42.5 GB / 128 GB`）。

### 1.2 顶部操作与路径栏 (Top Toolbar & Breadcrumbs)
- **历史导航**:
  - `GoBack` (`<`): 返回历史上一路径（若在历史栈顶则置灰/禁用）。
  - `GoForward` (`>`): 前进历史下一路径。
- **面包屑路径条 (Breadcrumbs)**:
  - 解析当前路径为层级按钮（如 `root` > `Documents` > `Projects`），点击任意一级可直接直达该目录。
- **搜索与过滤 (Search / Filter)**:
  - 搜索框输入实时过滤当前视图内的文件/文件夹。
- **视图切换 (View Modes)**:
  - `List View`（详细列表：名称、大小、类型、修改日期、快捷操作）。
  - `Grid View`（大图标卡片网格：高保真色彩图标、名称、大小）。
- **工具按钮**:
  - `新建文件夹` (New Folder)
  - `新建文件` (New File)
  - `隐藏文件开关` (Toggle Hidden)

### 1.3 交互与弹层系统 (Interactions & Popovers)
- **Plan 422 弹层原语集成**:
  - **右键上下文菜单 (Context Menu)**: 坐标锚 Popover，提供 `打开` (Open)、`重命名` (Rename)、`复制` (Copy)、`剪切` (Cut)、`粘贴` (Paste)、`删除` (Delete)。
  - **删除确认弹层 (Delete Confirmation)**: 唤起确认 Popover，防止误删重要文件。
  - **新建弹层 (New Item Modal / Popover)**: 输入新文件/文件夹名称，带非法字符校验与重名检查。
- **内联就地重命名 (Inline Rename)**:
  - 列表/网格项名称处就地转为输入框，Enter 确认，Esc 取消。
- **剪贴板机制 (Clipboard Workflow)**:
  - 复制/剪切后将源项路径与操作记录到剪贴板，支持跨目录粘贴。

---

## 2. 状态模型 (State Model)

```
// 目录与文件元数据
id: int
name: str
path: str
parent_path: str
is_dir: bool
size: int
size_str: str
ext: str
icon: str
modified: str
is_hidden: bool
color: str
```

### 状态持久化 (Storage Persistence)
通过 `storage.get` / `storage.set` 在 Vue（`localStorage`）与 VM（会话存储文件）双端保持：
- `fileman.view_mode`: `"list"` | `"grid"`
- `fileman.sort_col`: `"name"` | `"size"` | `"date"` | `"type"`
- `fileman.sort_dir`: `"asc"` | `"desc"`
- `fileman.show_hidden`: `"true"` | `"false"`

---

## 3. 排序与过滤算法 (Sort & Filter Engine)

1. **目录优先原则**: 无论选择哪种排序方式，文件夹始终置顶在普通文件之前（符合 OS 标准习惯）。
2. **多字段排序**:
   - `name`: 按名称字典序升序/降序。
   - `size`: 文件夹按 0 算，文件按字节大小数值升序/降序。
   - `date`: 按日期字符串升序/降序。
   - `type`: 按扩展名字典序升序/降序。
3. **隐藏文件过滤**: 若 `show_hidden == false`，剔除所有以 `.` 开头的文件/目录。
4. **搜索词过滤**: 忽略大小写，匹配项名称包含 `search_q`。

---

## 4. 测试与验证矩阵 (Test Matrix · desktop_mcp.py)

- **T1: 初始结构与快照** (Title, Toolbar, Sidebar, Quick Access, Initial File Count)
- **T2: 目录导航** (点击进入子目录、面包屑段更新、返回上一级)
- **T3: 视图切换** (List ↔ Grid 切换并保持项一致)
- **T4: 排序与隐藏开关** (按大小/名称排序、隐藏文件显隐切换)
- **T5: 搜索过滤** (输入关键词过滤出目标文件)
- **T6: 新建文件与文件夹** (弹层创建、新项在视图中呈现)
- **T7: 内联重命名** (启动编辑、修改名称、提交后新名称生效)
- **T8: 剪贴板复制/剪切/粘贴** (复制文件 → 切换目录 → 粘贴生成副本)
- **T9: 删除确认与执行** (唤起确认弹层、确认后项从列表中移除)
- **T10: 配置持久化** (存储写入、状态保持)
