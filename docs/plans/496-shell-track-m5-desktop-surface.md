---
plan_id: PLAN-496
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m5-desktop-surface
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-496] shell-track M5——桌面本体（壁纸 / 图标网格 / 入口）

## 变更摘要

shell-track 第五站（Design 25 §6 M5，S9 桌面本体）：第五个 shell 面
`assets/desktop.at` 挂 **463 预留的桌面层 z 槽**（壁纸之上、App 窗口之下）——

1. **壁纸**：图片/纯色（storage 键 `shell.desktop.wallpaper`，boot 读入）；
   487 settings 面板增"外观-壁纸"入口（storage 直写，非几何无动词，
   487 判定先例）。
2. **图标网格**：pinned 应用复用 472 `shell.dock.pinned` 数据源 + 自定义
   条目 `shell.desktop.icons`；双击 = 未运行启动/运行中聚焦（472 activate
   两臂语义同款）；右键菜单（打开 / 从桌面移除 / 更换壁纸入口；402 右键
   mouse_area 先例）。
3. **空白桌面交互**：点击空白 = 取消选择/关闭 overlay（463 既有桌面点击
   语义复用）。

**虚拟文件夹 v1 不纳入**（待澄清③，需求出现再立项）；图标拖拽自由摆放
v1 不做（固定网格）。

## 目标

- **G1 壁纸**：`shell.desktop.wallpaper`（图片路径或色值）boot 读入全屏
  铺底；缺省回退 pack 默认纯色；settings 改后**下次启动生效**（热切换=
  增强候选）。
- **G2 图标网格**：网格排列（图标+名），双击启动/聚焦，右键菜单三项；
  数据源 = pinned ∪ 自定义条目；增删持久化（storage）。
- **G3 z 槽挂载**：desktop.at 内容渲染于壁纸层之上、App 虚拟窗口之下
  （463 桌面层 z 槽消费）；App 窗口拖过时图标自然被覆盖（层级正确性
  验证项）。
- **G4 双端同源**：I8——desktop.at 双端装载（vm 桌面 + vue 端同源对拍
  基线）。
- **非目标**：虚拟文件夹；图标自由拖摆/对齐线；多 workspace 壁纸；壁纸
  幻灯片/每时钟热切换；HICON 真图标提取（占位沿用，473/486 同款延期）。

## 架构方案

```
z 序（桌面窗口内，自下而上）
  壁纸层（image/solid 全屏）        ← desktop.at 根背景
  桌面图标网格（z 槽，463 预留）     ← desktop.at 主体
  App 虚拟窗口 / shell overlay 面    ← 既有（dock/switcher/settings…）
```

- **面形态**：`assets/desktop.at` 独立资产（与 shell.at 分离——z 槽生命
  周期与 overlay 面不同，常驻不召唤）；装载走既有 shell 资产内嵌+懒挂载
  管线（renderer.rs:6968 段同型，挂载目标=z 槽容器）。
- **数据**：`shell.desktop.wallpaper`（string：路径|`#hex`）、
  `shell.desktop.icons`（`[{id,icon,label}]`，与 pinned 合并去重）；boot
  注入投影（`__desktop_surface` 状态变量族，投影协议 v1.4 内字段扩展——
  仅字段不增动词，无版本升段）。
- **交互**：双击 → `desktop.activate(id)`（472 两臂动词复用，零新动词）；
  右键 → 本地面板（打开/移除/更换壁纸入口跳 settings）。

## 技术栈

纯 AutoUI（.at shell 面 + storage natives + 既有 activate 动词 + 402 右键
先例）。零新依赖、零 Win32、零新 widget（image/右键面板既有）。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-31）

- **设计依据**：Design 25 §2 S9（桌面本体：壁纸/桌面快捷方式/虚拟文件夹；
  风险"无"，纯 AutoUI 工程）+ §4.1（默认 pack 根声明 `Desktop`，壁纸/图标
  为根子组件；`desktop_surface` 名废弃——实际实现为独立 desktop.at 面，
  语义等同）；§6 驱动侧小特性"desktop 层 z 槽"已随 463 落地待消费。
- **shell 面成熟模式**：assets/{shell,switcher,notification_center,settings}
  .at 四件先例（进程内嵌/懒挂载/装载失败降级）；472 pinned 数据链
  （`shell.dock.pinned` 解析注入）；487 settings 面板（外观分区扩壁纸入口
  的落点）；402 右键 mouse_area；463 桌面点击空白语义。
- **排程**：队列空（490/491/492/493 均归档），494（真洞）/495（aavm 专项）
  并行起草中——三者改动面互不交叠（494=native_dock/renderer 透明段，
  495=vm codegen/aavm，本计划=shell 资产 + renderer 挂载段 [与 494 的
  透明段不同区] + settings 面板追加分区）。与 494 在 renderer.rs 轻同文件
  不同段，后合者 rebase 可解。

## 详细设计

### 1. assets/desktop.at（新，第五 shell 面）

- 根：全屏容器，背景 = 壁纸（image 路径存在→image 全屏 cover；否则色值
  solid；缺省 pack 默认色）。
- 图标网格：`for` 注入条目（pinned ∪ 自定义，去重），格 = icon（占位
  lucide）+ label；双击 → `desktop.activate(id)`；右键 → 三项菜单
  （打开=activate / 移除=storage 写 `shell.desktop.icons` 排除项 /
  更换壁纸= `desktop.open_settings()` 外观分区）。
- 空白点击 → `desktop.focus(0)` 式取消语义（463 现状对齐，执行期定具体
  动词或本地态）。

### 2. z 槽挂载（renderer.rs）

- 懒挂载 desktop.at 至桌面层 z 槽容器（6968 段同型）；层级在壁纸背景与
  App 窗口层之间（463 z 槽消费点，符号执行期 grep `z 槽`/桌面层定位）。

### 3. 数据链（boot 注入 + storage）

- boot 读 `shell.desktop.wallpaper`/`shell.desktop.icons` + pinned 合并 →
  注入 `__desktop_surface` 投影（含壁纸与条目两字段，指纹门控同族）；
- 移除/新增条目 = storage 直写 + 下次 boot 生效（v1 不做热刷新，菜单内
  提示"重启后生效"——487 pinned 同款语义）。

### 4. settings 面板增壁纸分区（487 资产追加）

- `assets/settings.at` 外观分区（或 Dock 分区旁新增"外观"）增壁纸路径/
  色值输入 + 保存写 `shell.desktop.wallpaper`（storage 直写，无动词）。

## 测试设计

1. **T1 装载测**：desktop.at 装载 + `__desktop_surface` 注入 headless
   （desktop_mcp 五套同型：条目渲染数/双击 activate 派发/右键菜单出现）。
2. **T2 storage 往返**：`shell.desktop.icons`/`wallpaper` 写读 + boot 合并
   去重（pinned ∪ 自定义）单测（472 T5 预置键先例）。
3. **T3 层级验证**：App 窗口开启时图标被覆盖（z 槽正确性——headless
   bounds/z 断言或实机截图）。
4. **T4 实机冒烟**：换壁纸重启生效；图标增删持久；双击启动/聚焦两臂；
   右键三项；空白点击取消；settings 壁纸入口闭环。

## 验收标准

1. T1–T3 绿；T4 实机清单 PASS 留痕。
2. I8：vue 端 desktop.at 同源装载对拍基线一条（a2vue/金样体系挂靠）。
3. schema 三件套绿（若投影字段入 schema/projection-protocol-v1.md 文档
   同步）；`cargo t ui` 不回归；零警告。
4. 虚拟文件夹/拖摆确认未夹带（非目标核对）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **desktop.at 骨架**：新建 `crates/auto-lang/assets/desktop.at`（壁纸
   背景 + 网格占位 + 空白点击声明）。
   验证：临时装载冒烟（随 T1 装载测）。
2. **z 槽挂载**：`crates/auto-lang/src/ui/iced/renderer.rs` 懒挂载至桌面
   层 z 槽（6968 段同型，z 槽符号 grep 定位）。
   验证：`cargo check -p auto-lang && cargo t ui`。
3. **数据链**：boot 读两键 + pinned 合并去重 + `__desktop_surface` 投影
   注入 + T2 单测。
   验证：`cargo t session && cargo t ui`。
4. **图标交互**：双击 activate 两臂 + 右键三项菜单（402 先例）+ 移除
   storage 写。
   验证：`cargo t desktop_mcp`（T1 用例）。
5. **settings 壁纸入口**：`crates/auto-lang/assets/settings.at` 增壁纸
   分区（storage 直写）。
   验证：`cargo t desktop_mcp`。
6. **层级验证**：T3（App 窗口覆盖图标断言）。
   验证：`cargo t ui`（或 iced-layout-tests 档）。
7. **双端对拍**：I8 金样一条（desktop.at vue 端装载基线）。
   验证：a2vue/vue 套件绿。
8. **实机冒烟 + 收尾**：T4 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **虚拟文件夹（③）**：v1 不纳入（设计列于 S9 但无现实需求锚点）——需求
  出现再立项；复审核对未夹带。
- **壁纸热切换**：v1 重启生效（与 pinned 同语义）；热切换（改后即时重铺）
  为增强候选——若实现成本证之为"投影字段重注入+指纹门控天然支持"则顺路
  做，T4 记录实况。
- **图标资产**：占位 lucide（486 "app-window" 先例）；pinned 条目自带 icon
  数据（472）优先用之。
- **空白点击语义**：463 现状的具体取消机制执行期对齐（动词或本地态）。
- **投影文档**：`__desktop_surface` 字段若登记进 schema/projection-protocol-v1.md
  为字段扩展（不升版本段），格式随 487 v1.4 实况对齐。
