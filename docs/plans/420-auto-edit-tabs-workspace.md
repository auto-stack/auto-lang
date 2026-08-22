# Plan 420: auto-edit 多 tab 工作区(关闭/打开/脏标记/拖拽)

> **状态**: 📋 已立项待实施(2026-08-22,源自 414 §6 后续项 + 418 文件 natives 就绪后的能力补齐)
> **来源**: Plan 414 §6(tab 关闭按钮、`+` 打开、拖拽排序、脏标记"为后续")/ 418 §7(dialog/fs natives 已落地,ActOpen/ActSave 因阻塞对话框无法自动化)
> **关联**: 413(code_editor 多实例 registry)/ 418(动作层与矩阵)

---

## 0. 一句话结论

**041 从"固定双 tab 演示"升级为动态 tab 列表**:关闭真实生效、`+` 经文件对话框开新 tab、编辑脏标记、拖拽排序;顺带打通文件打开/保存的自动化验证通道。

## 1. 现状盘点

- `CloseTab(name)` 仅 console_log("(MVP: dummy)")(`041 app.at`);两个 tab 为模板硬编码(`if .tab == 0/1` 分支×2,编辑器 key tab-main/tab-util 固定)。
- `+` 按钮不存在;脏标记不存在(`.edits` 是计数,非 dirty 语义);无拖拽。
- natives 就绪:`dialog_open/filter`、`dialog_save`(推测,以 catalog 为准)、`fs.read_text/write_text`(418 §1.2 矩阵)。
- 编辑器实例按 key 注册(core LRU registry),动态 tab 天然支持任意数量 editor 实例。
- 自动化盲区:阻塞式 rfd 对话框无法被 MCP 驱动(418 §7.2),文件往返仅人工。

## 2. 方案要点

- **model 改造**:tabs 从两个固定 var 组改为数组态(state Vec 或 VM 列表——VM `[...]` 字面量+push 模式已有先例);TabState = {key, title, path, dirty, closed};active 索引。模板层用 `for` 渲染 tab 条(条件双分支删除)。
- **CloseTab**:closed=true,tab 条隐藏;全关时显示空态。**"+"按钮**:dialog_open → fs.read_text → append tab → 激活。
- **脏标记**:oninput / external_dirty(418 §8.8 已有握手)置 dirty;ActSave 成功后清;close 时 dirty 弹确认(popover 依赖 422,先 console 简化亦可)。
- **拖拽排序**:tab 头 mouse_area 拖拽(app 层交换数组;iced 无原生 reorder)。
- **自动化通道(解 418 §7.2 盲区)**:`AUTO_OPEN_PATH`/`AUTO_SAVE_PATH` 环境变量旁路——设置时 ActOpen/ActSave 跳过 rfd 直接用定值路径(仅测试构建语义,prod 忽略);矩阵 T 组补:开定路径文件→编辑→dirty→保存→dirty 清→重读一致。
- **ActSave**:path 空→dialog_save;有 path→直写。

## 3. Phases

- **P1 model 动态化**:tabs 数组+for 渲染+既有双 tab 迁移(矩阵 29/31 不回归为底线)。
- **P2 关闭/打开/保存闭环** + 环境变量旁路;矩阵文件往返组。
- **P3 脏标记** 全链路(含 external_dirty 握手复用)。
- **P4 拖拽排序**(纯 app 层,最后做,可延后)。

## 4. 验收

- MCP 矩阵:开定路径文件→编辑→`.dirty=true`→保存→清→重读一致;关 tab→+→重开;29/31 既有组全绿。
- 人工:拖拽排序、真实 rfd 对话框往返。

## 5. 风险

- VM 列表 state 的模板 for 渲染稳定性(probe 路径/vtree 对齐——§8.4① 教训,for 路径方案已有 Fix A 先例)。
- rfd 旁路语义混入 prod(必须 env 门控+文档注明仅测试)。
