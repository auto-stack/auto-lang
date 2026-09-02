# Plan 440: 027-file-manager 文件管理器（App 轨道填洞 ④）

> **状态**: ✅ 全部完成（2026-09-02）——Vue 产物 build 0 错误（vue-tsc + vite 1.65s），VM 模式实机 desktop_mcp 49/49 全部 PASS。
> **来源**: [Design 21 §5](../design/autoui/examples-app-track.md) 填洞路线第 4 项。
> **关联**: [Plan 422](archive/422-popover-primitive-menubar-contextmenu.md)（右键菜单原语）、[Plan 438](archive/438-025-dashboard.md)（系统监视器先例）、[Plan 464](archive/464-launcher-app.md)（启动器先例）、[Plan 354](archive/354-notes-app-upgrade.md)（015 树状导航先例）
> **目录**: `examples/ui/027-file-manager/`｜pac `name: "file-manager"`｜端口 4027

## 1. 目标与平台缺口

**文件管理器**（AutoOS 默认应用刚需，双栏 Finder/Explorer 形态）。钉住：

- **双栏与目录导航**：左侧快速访问侧栏（主目录、文档、下载、图片、音乐、回收站）+ 存储空间仪表盘，右侧主文件浏览区。
- **面包屑与历史栈**：动态路径分段面包屑（支持任意层级点击跳转）+ GoBack / GoForward / GoUp 历史导航。
- **双视图呈现**：明细列表（List）与图标网格（Grid）一键平滑切换。
- **排序与过滤**：按名称、大小、类型、修改时间升降序排序；实时搜索输入关键字过滤；隐藏文件开关切换。
- **右键菜单实战**：Plan 422 Popover 原语实战（打开、重命名、复制、剪切、粘贴、删除）。
- **内联编辑与新建/删除**：列表内就地重命名输入框；新建文件夹/文件模态弹层；删除二次确认对话框 Popover。
- **剪贴板与配置持久化**：复制/剪切/粘贴跨目录流转；`localStorage` 持久化保存视图模式与隐藏文件开关（VM 端映射至 storage 文件）。

## 2. 架构设计与 M0 Spike 结论

- **M0 Spike**：AutoUI 应用层采用沙箱化虚拟文件系统模型（平行数组结构：`item_ids`, `item_names`, `item_paths`, `item_parents`, `item_is_dirs`, `item_sizes`, `item_size_strs`, `item_file_exts`, `item_types`, `item_dates`, `item_hiddens`, `item_colors`），无需侵入底层 crates 即可在 Vue 端与 VM(Iced) 端以 100% 一致的数据契约运行。
- **视图渲染推导**：`files_view` 在 model 中预置带类型的模板对象，确保 TypeScript / vue-tsc 准确推导字段类型。
- **Popover 定位与关闭**：利用 Plan 422 的 `popover (open, x, y, ondismiss)` 原语，实现右键上下文菜单、新建弹层、删除二次确认对话框。

## 3. Phase 执行记录

### M0 — Spike 与规约设计
- 验证双端运行可行性与数据流模型，制定 `examples/ui/027-file-manager/SPEC.md`。

### M1 — 浏览形态与视图模式（Vue & VM）
- 实现了双栏布局（左侧导航 + 存储空间，右侧主视图）。
- 实现了面包屑导航、历史前进/后退/上一级。
- 实现了列表（List）与网格（Grid）视图切换。
- 实现了名称、大小、类型、日期排序与隐藏文件开关、实时搜索过滤。

### M2 — 操作、弹层与剪贴板
- 接入 Plan 422 Popover 上下文菜单（打开/重命名/复制/剪切/粘贴/删除）。
- 实现了内联就地重命名交互（Enter 提交，Esc/Cancel 取消）。
- 实现了新建文件/文件夹模态弹层与删除二次确认 Popover。
- 实现了剪贴板状态管理与 Toast 交互反馈。

### M3 — 自动化测试与持久化验证
- 编写 `tests/desktop_mcp.py`，覆盖 T1 至 T13 全流程 49 项断言。
- 验证进程重启后 `view_mode` 和 `show_hidden` 从 storage 文件恢复。

## 4. 验收（DoD）

- [x] M1：Vue 构建与 `vue-tsc` 0 错误通过；列表/网格切换、面包屑、排序、隐藏项、搜索过滤全部正常。
- [x] M2：右键菜单、新建弹层、删除确认、内联重命名、剪贴板复制/剪切/粘贴全链路通过。
- [x] M3：VM 模式实机运行，`tests/desktop_mcp.py` 49/49 测试项 100% PASS。
- [x] 配置持久化：进程重启后从 storage 自动恢复用户配置。

## 5. 独立复审记录（Independent Review Gate）

1. **Checklist 审计**：
   - 目录结构：`pac.at`, `SPEC.md`, `src/front/app.at`, `tests/desktop_mcp.py` 齐备。
   - 依赖与端口：pac 声明 `name: "file-manager"`, `port: 4027`。
   - 构建门禁：`auto build` 产出 Vue SFC，`vue-tsc && vite build` 0 错误构建成功。
   - MCP 门禁：`python tests/desktop_mcp.py` 运行 49 项断言全部 PASS。
2. **Workaround 扫描**：
   - 无侵入性 crates 修改，无全局污染。
   - 移除所有临时变通，`files_view` 类型声明清晰，事件派发机制完备。
3. **生态登记**：
   - 在 `examples/ui/README.md` 中完成 `027-file-manager` 登记。
