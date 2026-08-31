---
plan_id: PLAN-496
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m5-desktop-surface
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "schema/projection-protocol-v1.md: 版本 v1.4 不变内字段扩展（§2.1 桌面本体面字段族 __desktop_bg/__desktop_icons/__desktop_hidden + §4 storage 键增量三键 + §5 金样补 desktop_surface_* 三测/T3/a2vue 金样 + §6 变更记录「v1.4 内字段扩展」条——零新动词，复用 activate/open_settings）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——settings 面板三分区 → 四分区（新增「外观」壁纸入口：DraftWallpaper/SaveWallpaper storage 直写 + 召唤注入 cfg_wallpaper 快照；487 判定先例延续——非几何无动词）"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——桌面本体面（assets/desktop.at 第五 shell 面，常驻不召唤：boot 装载挂桌面层 z 槽（壁纸之上/App 虚拟窗之下——view 装配 Stack 先于 z_order 推层）；DesktopState.desktop_app/desktop_wallpaper + HostCtx.desktop_fields + split_mut windowless 第六路/split_ref_desktop + boot 常载降级；投影注入 __desktop_bg/icons/hidden（boot 一次，无指纹门控）+ 图片壁纸宿主底层元素）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——桌面数据链（storage 三键 shell.desktop.wallpaper 路径|#hex boot 解析回退 #090e1a / icons 自定义 id 逗号串 / hidden 排除 id 逗号串；pinned ∪ custom 合并去重 hidden 排除，icon/label 注册表解析缺省 app-window/id；增删重启生效=487 pinned 同语义）"
  - "crates/auto-lang/assets/desktop.at: 新增——DesktopSurface 面（根 bg ${.__desktop_bg} 插值铺色/固定网格 8 列/mouse-area ondblclick=activate 两臂/button oncontextmenu.prevent=本地面板三项（打开/移除 hidden 直写/更换壁纸→open_settings）/空白点击 BlankPress 关菜单=463 语义）"
  - "crates/auto-lang/src/ui_gen/vue.rs: 修改——插值 class 修复（interpolated_class_parts：${.field} 静态段入 class + 插值段转 :class 拼接表达式；extract_classes 双臂 + shadcn row/col class/style 臂 + push_style_class 五点——此前插值原样落静态 class 浏览器侧废 token）"
  - "crates/auto-lang/src/ui/iced/renderer.rs: 修改——ondblclick VM 全链（View::MouseArea.on_double_click 字段双映射 + iced mouse_area on_double_click + extract.rs codegen 事件分流补位）+ convert_view_messages 补 MouseArea 显式臂（此前 VM 动态路径落 Empty 兜底，484 图表族经 Rust codegen 未暴露）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——shell-track M5 落地（S9 桌面本体：壁纸/图标网格/入口第五面 + z 槽消费 + ondblclick 原语；虚拟文件夹/图标拖摆按计划非目标未夹带）"
  - "GOAL-007: AutoUI 跨端视觉一致——desktop.at 双端同源 a2vue 金样 + vue 生成器插值 class 缺口修复（金样暴露）"

affects: [auto-lang/ui]
current_step: 8
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
   [✅ 已完成] commit d87bcd8bd：DesktopSurface 面落盘（根 bg `${.__desktop_bg}`
   插值铺色/网格占位/空白点击 BlankPress/右键三项菜单/移除 hidden 直写）；
   装载冒烟随步骤 4 T1 用例（本步不单独跑）
2. **z 槽挂载**：`crates/auto-lang/src/ui/iced/renderer.rs` 懒挂载至桌面
   层 z 槽（6968 段同型，z 槽符号 grep 定位）。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] commit a1adc8114：session.rs desktop_app/desktop_fields/
   split_ref_desktop（windowless 第六路拆借）+ shell.rs DESKTOP_AT builder +
   boot 常载（shell 挂载邻位，失败降级不阻断）+ view 装配层 Stack 最底
   推层（先于 z_order 虚拟窗循环）。验证：check 212 警告=基线零新增；
   `cargo t ui` 776 绿 + `--features ui-iced iced` 117 绿 + `session` 72 绿
   （renderer/session 测试须 ui-iced 特性，487 先例）
3. **数据链**：boot 读两键 + pinned 合并去重 + `__desktop_surface` 投影
   注入 + T2 单测。
   验证：`cargo t session && cargo t ui`。
   [✅ 已完成] commit 98967accd：load_desktop_wallpaper（#hex 直传/路径验在/
   坏值回退 `session::DESKTOP_WALLPAPER_DEFAULT`=#090e1a）+ load_desktop_id_list
   （icons/hidden 逗号串）+ inject_desktop_surface（pinned∪custom 去重 hidden 排除，
   `__desktop_bg`/`__desktop_icons`/`__desktop_hidden` 三字段族注入）+ boot
   inject_dock_pinned 邻位接线 + view 图片壁纸底层元素。T2 双测
   desktop_surface_storage_roundtrip_and_wallpaper_resolution / _merge_dedupe_and_injection
   绿（真 desktop.at 装载）；`--features ui-iced session` 72 绿 + `iced` 119 绿 +
   `cargo t ui` 776 绿
4. **图标交互**：双击 activate 两臂 + 右键三项菜单（402 先例）+ 移除
   storage 写。
   验证：`cargo t desktop_mcp`（T1 用例）。
   [✅ 已完成] commit 4f3dd69d9：ondblclick 全链（parser 泛识别既有 +
   extract.rs codegen 分流补位 + View::MouseArea.on_double_click 字段双映射 +
   iced on_double_click 接线）+ convert_view_messages 补 MouseArea 显式臂
   （VM 动态路径此前落 Empty 兜底——484 图表族经 Rust codegen 不经该转换
   未暴露，本臂同时修复 hover 面板 VM 缺席潜在缺口）。T1 =
   desktop_surface_at_loads_interactions_and_dispatch 绿（条目渲染数/双击臂
   转换前后存活/IconMenu→菜单三项/MenuRemove→hidden 直写/BlankPress/
   activate+open_settings 动词记录）；`cargo t desktop_mcp` 名义门零匹配
   （487 先例，实际覆盖 = desktop_surface_* 三测族）；iced 120 绿 +
   session 72 绿 + 默认快档 3302 绿
5. **settings 壁纸入口**：`crates/auto-lang/assets/settings.at` 增壁纸
   分区（storage 直写）。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] commit 0e6f42c09：settings.at 四分区（+外观）——壁纸输入
   DraftWallpaper/SaveWallpaper（storage 直写 `shell.desktop.wallpaper`，
   空草稿不落键；487 非几何无动词判定同款）+ Nav appearance 分支 +
   toggle_settings 召唤注入 cfg_wallpaper 快照。验证：
   settings_appearance_wallpaper_section_writes_storage 绿（settings 族 8/8；
   `cargo t desktop_mcp` 名义门同 487）+ iced 121 绿
6. **层级验证**：T3（App 窗口覆盖图标断言）。
   验证：`cargo t ui`（或 iced-layout-tests 档）。
   [✅ 已完成] commit 3fae185cb：iced-layout-tests 档
   desktop_surface_z_slot_window_covers_icons——真 desktop.at 面（map_msg 丢
   消息进 testbench）+ 真 VWinState 虚拟窗（wm_add_win 同源），按 view()
   Stack 装配序层叠，断言首枚图标格 ⊂ 窗矩形 (0,0,400,300)（被覆盖几何，
   G3）+ 窗客户区/chrome 同区共存。layout_tests 16/16 绿
7. **双端对拍**：I8 金样一条（desktop.at vue 端装载基线）。
   验证：a2vue/vue 套件绿。
   [✅ 已完成] commit 6dbcc0057：真资产同源金样 test_a2vue_desktop_surface_asset
   （include_str 同文件零拷贝；金样 test/a2vue/desktop_surface_asset/expected.vue，
   AUTO_LANG_UPDATE_GOLDEN=1 重生成）+ @dblclick/@contextmenu.prevent 事件面断言。
   金样暴露并修复 vue 生成器插值 class 缺口：`${.field}` 此前原样落静态 class
   （浏览器侧废 token）——interpolated_class_parts 拆分（静态段入 class +
   插值段转 `:class` 拼接表达式），落点 extract_classes 双臂 + shadcn row/col
   class/style 臂 + push_style_class。验证：金样绿 + 默认快档 3303 全绿
   （+1 新测零回归）
8. **实机冒烟 + 收尾**：T4 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] commit ae85967ce：T4 报告 `docs/plans/reports/496-t4-live-smoke.md`
   + 截图两帧（预写键 boot 单帧三断言：壁纸铺底/增删持久三图标/窗口覆盖
   层级；交互项 OS 注入受阻转 headless 全链对表——487 先例）；desktop.at
   补空 Init 消 boot 噪音；投影协议文档同步（§2.1 字段族/v1.4 内扩展变更
   记录/§5 金样补登）。健康检查：check 警告 212=基线零新增、fmt 漂移
   master 既有、无调试打印；`cargo t ui` 复跑绿

## 复审记录

**复审**（/auto-plan:review，2026-08-31，基线 plan-496-dev @ e7ec6bcb4）：

### 验收标准逐条复验（verify, don't trust）

| # | 验收标准 | 结论 | 证据 |
|---|---|---|---|
| 1 | T1–T3 绿；T4 实机清单 PASS 留痕 | **PASS** | 复跑：T1 `desktop_surface_at_loads_interactions_and_dispatch` + T2 双测（`--features ui-iced desktop_surface` 4/4）+ T3 `desktop_surface_z_slot_window_covers_icons`（iced-layout-tests 档 16/16）；T4 报告 `docs/plans/reports/496-t4-live-smoke.md` + 截图两帧（预写键 boot 单帧三断言：壁纸铺底/增删持久/窗口覆盖层级；交互项 OS 注入受阻转 headless 对表=472/478/479/487 家族先例） |
| 2 | I8：vue 端 desktop.at 同源装载对拍基线一条 | **PASS** | `test_a2vue_desktop_surface_asset`（真资产 include_str 同文件金样 + @dblclick/@contextmenu 事件面断言；compare 模式复跑绿）；金样暴露的 vue 插值 class 缺口已修（interpolated_class_parts 五点落码，默认档全量零回归） |
| 3 | schema 三件套绿（投影字段入文档同步）；cargo t ui 不回归；零警告 | **PASS** | `cargo tf` 3304/3304 全绿（含 schema_drift/docs_gen/component_registry 三件套）；协议文档同步（§2.1 字段族 + §5 金样补登 + §6 变更记录）；`cargo t ui` 777/777；check 警告 212 = master 基线零新增、fmt 漂移 = master 既有、无调试打印（降级 eprintln = shell 家族惯例） |
| 4 | 虚拟文件夹/拖摆确认未夹带 | **PASS** | desktop.at grep 无 folder/drag/拖词——固定网格 + 条目模型无层级（非目标核对） |

**全量门禁（本计划生命周期唯一全量运行点）**：`cargo tf` 3304/3304 全绿；
ui-iced 特性档补充全量 4141/4142——唯一红 `plan492_tests::m1_pkg_fstr::
pkg_canary_undefined_var_kills_bar_init` 经基点探针（39abc730f 临时 worktree
实证）为**基点既有红**，master 新顶（495 合并 08d060cba）已绿，合并时自愈
（登记 P496-2）。

### 遗漏/延后/workaround 排查

- **遗漏**：无——执行步骤 8/8 各有对应 commit 与验证证据（git log 八枚 +
  review 两枚复核）；计划 §详细设计 1–4 各小项均有落码或待澄清登记。
- **延后**：壁纸热切换（待澄清②计划内 v1 语义，非私自延后）；HICON 真图标
  /虚拟文件夹/拖摆/多 workspace 壁纸/幻灯片均为计划非目标。T4 交互实机照
  OS 注入受阻 → headless 对表（P496-1 登记，487 家族先例）。
- **Workaround/超计划范围修复（均已成文）**：① convert_view_messages 补
  MouseArea 显式臂（VM 动态路径既有 Empty 兜底缺口——484 图表族经 Rust
  codegen 未暴露；本计划双击原语依赖，同时修复 hover 面 VM 缺席）；
  ② vue 生成器插值 class 修复（金样暴露的双端缺口）；③ T3 测试复刻 view()
  Stack 装配序（view() 需活 iced 运行时——headless 允许形态 + 实机截图
  佐证）；④ desktop.at 补空 Init 消 boot 噪音（T4 执行期发现）。
- **执行期形态决策**（与计划文的偏差，均已登记待澄清事项尾节）：icons
  逗号串 + hidden 独立键 / 壁纸层 #hex-vs-图片双径 / 菜单固定位 /
  open_settings 无参——四项均为实现形态选择，语义与计划架构合同一致。

### 结论

**全验收 pass，无阻断债项** → `status: reviewed`，就绪 `/auto-plan:merge`。

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
  〔执行定案〕已登记为 §2.1 `__desktop_*` 字段族（v1.4 内字段扩展不升段，
  零新动词）；实投字段为 `__desktop_bg`/`__desktop_icons`/`__desktop_hidden`
  三枚（计划文 `__desktop_surface` 为族名泛称）。
- **执行期形态决策（复审登记，与计划文 §详细设计的偏差及理由）**：
  ① 图标 storage 形态：计划文 §数据链写 `[{id,icon,label}]` JSON——实操落
  **id 逗号串**（`shell.desktop.icons`，shell.dock.pinned 同形）+ 独立排除键
  `shell.desktop.hidden`（计划文「移除=写 icons 排除项」的键位分拆）。理由：
  .at 侧重写 JSON 串不安全（转义拼接），settings.at PersistPinned 已证逗号
  串重写链路；icon/label 由宿主注册表解析注入（.at 无注册表访问权，472 同
  裁定），store 侧无需存 icon/label。
  ② 壁纸层实现形态：计划文「壁纸层=desktop.at 根背景」——DSL 无重叠布局
  （iced 无绝对定位），实操拆 **#hex 由 desktop.at 根 bg 插值实铺
  （`__desktop_bg` 注入）/图片路径由宿主在面之下推壁纸图层**，z 序语义与
  计划架构图一致（壁纸 < 图标面 < App 窗口）。
  ③ 右键菜单位置：本地面板固定于网格上方（popover 锚定未用——坐标锚需
  光标位置 payload，v1 简化）。
  ④ 更换壁纸入口 = open_settings 无参（面板内手动导航外观分区——v1.4 动词
  无参保持，不增分区直达参数）。
