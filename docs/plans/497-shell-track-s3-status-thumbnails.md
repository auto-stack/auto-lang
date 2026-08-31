---
plan_id: PLAN-497
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-s3-status-thumbnails
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

# [PLAN-497] shell-track S3——Status 栏（时钟 / 托盘组 / 每窗口真缩略）

## 变更摘要

Design 25 §2 S3（Status 栏：时钟/托盘/每窗口缩略管理）——设计里唯一整体
未动的 shell 表面，也是桌面特性线的**收官件**：

1. **时钟**：dock 区时钟（shell 本地 tick，30s 粒度；不走投影——避免高频
   投影抖动）。
2. **托盘组**：dock 端右侧归组（479 通知铃铛 + 时钟 + 状态图标挂载点容器；
   app 图标注册 API 后置为非目标）。
3. **每窗口真缩略**：**离屏快照 API**（wid → 图像）——设计定位"缩略=离屏
   快照（路线 B lite）"，现状 v1 图标占位（switcher `mru_icons`、dock
   `app-window` 占位）。本期建 UI 侧快照核心（复用 `ui/headless/` 渲染
   地基），登记 `window_thumbnail` widget（Design 25 §4 登记族，I4 双端），
   并接入三处消费者：**switcher 行缩略、dock 条目 hover 预览、pager 分区
   hover 预览**。

## 目标

- **G1 时钟**：dock 右端常驻时钟（HH:MM），本地 tick 驱动，零投影流量。
- **G2 托盘组**：铃铛（479）+ 时钟 + 挂载点容器成组右置；布局在 dock 配置
  （position 顶/底）两态下均正确。
- **G3 快照核心**：`snapshot_window(wid) -> Option<ImageData>`（离屏渲染
  App 视图树 → RGBA/PNG；降采样到缩略尺寸）；新鲜度策略 = 召唤时即时抓取
  + 短 TTL 缓存 + relayout/关闭失效。
- **G4 widget 登记**：`window_thumbnail` 进 `schema/aura.at` +
  WidgetRegistry（I4）；vm 臂 = 快照 image 渲染（无快照回退 icon 占位）；
  vue 臂 = 组件登记 + v1 占位渲染（双端行为差异记待澄清①）。
- **G5 消费者三处**：switcher 行（icon→缩略，开关保留 icon 兜底）；dock
  条目 hover 预览（422 popover 先例）；pager 分区 hover 预览（该区窗口
  缩略小网格，条目≤4 截断）。
- **非目标**：native docked 窗口的 DWM 缩略（待澄清②）；托盘 app 图标
  注册 API（挂载点先行）；S8 shell IME；快照后台定时刷新（召唤式即取即用）。

## 架构方案

```
快照链：wid → AppSession 视图树快照(缓存视图) → 离屏渲染(ui/headless 地基)
        → 降采样 RGBA → window_thumbnail(image) / 消费面注入
消费者：switcher.at 行（召唤注入 mru_thumbs 平行列表，同 mru_icons 模式）
        shell.at dock 条目 hover → popover 预览（422 先例）
        shell.at pager 分区 hover → 分区缩略网格
时钟/托盘：shell.at 本地 tick 状态（setInterval 同型宿主定时）+ 右端组容器
```

- **快照取材点**：既有渲染缓存（`cached_rendered`/视图树）而非重演 VM——
  渲染树已每帧维护，离屏栅格化是纯渲染侧动作；实现路径 T1 spike 定案
  （headless 复用 vs iced overlay 离屏 target）。
- **投影协议**：消费者注入走既有平行字符串列表模式（`mru_thumbs` 同
  `mru_icons`），不新增协议动词/字段族——快照数据不走投影（体积大、
  召唤式，由宿主直接注入控件资产）。

## 技术栈

iced 离屏渲染（headless/testbench 地基）+ image 降采样 + 既有 popover/
投影注入管线。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-31）

- **设计依据**：Design 25 §2 S3 行（风险列注明"缩略与 IME 触及深水区"——
  真缩略即本期深水点，故 T1 spike 先行）；§6 挂起注记"缩略管理（S3 真缩略）
  → 挂 386 复活（离屏快照=路线 B lite；v1 图标占位）"——386 全阶段已归档，
  解锁。
- **现状核验**：switcher 消费 `mru_icons` 平行列表（assets/switcher.at:11/30，
  icon 占位）；dock native 条目占位 `"app-window"`（486）；479 铃铛在 dock；
  无时钟；UI 侧无 wid→图 API（480 的快照为协议/v2a 形态侧，非 UI 消费件）。
- **可复用**：`ui/headless/`（离屏渲染地基）；422 `ui/iced/popover.rs`
  （hover 弹层）；472/478 平行列表注入模式；479 铃铛。
- **排程**：494/495/496 在 review（即将释放会话）；本期与三者改动面交叠
  仅 shell.at/switcher.at（496 也动 shell 资产——后合者 rebase，预期 hunk
  级）。**桌面特性线在本计划之后仅剩长线项**（457/S8 与增强型债务）。

## 详细设计

### 1. 快照核心（ui/iced/ 新 snapshot.rs）

- `WindowSnapshot { rgba: Vec<u8>, w, h }`；`snapshot_window(wid)`：
  从渲染缓存取视图 → 离屏栅格化（T1 定路径）→ box 降采样（长边 ≤256）；
- 缓存：`HashMap<AppId, (WindowSnapshot, Instant)>`，TTL 2s + 事件失效
  （relayout/close/dirty）；召唤式调用，无后台定时。
- **T1 spike**：headless 复用 vs overlay 离屏 target 的可行性与成本对比，
  结论回写待澄清③。

### 2. window_thumbnail 登记（I4）

- `schema/aura.at` 新 widget：props `{ wid: string, fallback_icon: string }`
  （backends：iced full / web component）；
- vm 臂：查询快照缓存 → image；miss → 异步触发抓取 + 本帧 fallback icon；
- vue 臂：组件登记 + v1 占位渲染（icon + 边框），双端差异记待澄清①。

### 3. 时钟与托盘组（assets/shell.at）

- dock 右端组：`[挂载点容器][铃铛(既有)][时钟]`；时钟本地 tick（宿主定时
  注入分钟字符串或 .at 本地 interval——执行期按 shell.at 定时先例定）；
- dock position 顶/底两态布局正确（487 set_dock_position 联动验证）。

### 4. 消费者

- **switcher**：召唤注入 `mru_thumbs`（平行于 mru_icons；缩略缺失项空串
  → 控件 fallback）；行渲染 icon→thumbnail 升级；
- **dock hover 预览**：条目 hover → popover（422 先例）内 window_thumbnail
  （wid=条目窗口）；
- **pager hover 预览**：分区标签 hover → popover 网格（该区窗口缩略 ≤4，
  超出 "+N"）。

## 测试设计

1. **T1 spike 成文**：快照路径对比结论（待澄清③）。
2. **T2 快照单测**：注入彩色节点视图 → snapshot_window 断言尺寸/中心像素
   色；TTL 过期/失效路径。
3. **T3 装载测**：shell.at 时钟/托盘组渲染；switcher mru_thumbs 注入行
   缩略；dock/pager hover popover 出现（desktop_mcp 五套同型）。
4. **T4 I4 对拍**：window_thumbnail vue 端登记与占位渲染金样（a2vue 体系）。
5. **T5 实机**：switcher 召唤见真缩略；dock hover 预览；pager hover 网格；
   时钟走字；顶/底 dock 两态；后台 App 缩略新鲜度（切换内容后再召唤更新）。

## 验收标准

1. T2–T4 绿；T5 实机清单 PASS 留痕。
2. 缩略缺失路径全程有兜底（icon 占位），无空白/panic。
3. `schema` 三件套绿（新 widget 登记）；`cargo t ui` 不回归；零警告。
4. 投影协议零改动（平行列表注入模式）；非目标项未夹带。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 spike**：快照路径对比（headless 复用 vs overlay 离屏 target）——
   临时最小 demo 验证一条可 行路径，结论回写待澄清③。
   验证：demo 产出一张真缩略图留痕。
2. **快照核心**：新建 `crates/auto-lang/src/ui/iced/snapshot.rs`
   （snapshot_window + TTL 缓存 + 降采样）+ T2 单测；`ui/iced/mod.rs` 登记。
   验证：`cargo check -p auto-lang && cargo t snapshot`。
3. **widget 登记**：`schema/aura.at` 增 window_thumbnail +
   `ui_gen/widget/registry.rs` spec；vm 臂（`ui/iced/renderer.rs` 增渲染臂）
   + vue 臂占位。
   验证：`cargo test -p auto-lang --test schema_drift && cargo test -p auto-lang --test docs_gen && cargo t ui`。
4. **时钟/托盘组**：`crates/auto-lang/assets/shell.at` dock 右端组（挂载点/
   铃铛归组/时钟 tick）。
   验证：`cargo t desktop_mcp`（T3 相关用例）。
5. **switcher 消费**：`crates/auto-lang/assets/switcher.at` 行缩略
   （mru_thumbs 平行注入）+ 宿主注入臂（renderer.rs switcher 召唤段）。
   验证：`cargo t desktop_mcp`。
6. **dock/pager hover 预览**：shell.at 条目 hover popover（422 先例）+
   pager 分区网格（≤4+"+N"）。
   验证：`cargo t desktop_mcp`。
7. **I4 对拍**：vue 端 window_thumbnail 占位金样（a2vue 体系挂靠）。
   验证：a2vue/vue 套件绿。
8. **实机冒烟 + 收尾**：T5 清单留痕；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① vue 端缩略形态**：v1 占位渲染（icon+边框）。真缩略的 web 端路径
  （transform 缩放的复制子树，同源 store 驱动）为后续增强——I4 要求的是
  登记同源，不要求本期行为对齐。
- **② native docked 窗口缩略**：DWM 缩略（`DwmRegisterThumbnail` 目标=桌面
  HWND、rect=预览框）技术上适配 dock hover 预览，但与 494 真洞透明模式的
  相互作用未验证——native 条目 hover v1 维持 icon 占位，待 494 合入后的
  实机反馈再定。
- **③ 快照路径（T1 回写）**：headless 复用 vs overlay 离屏 target——以
  spike 定案；若两条皆阻（如视图缓存不可栅格化），退路 = MCP 截图管道
  复用（已有 screenshot 基建），仅性能较差。
- **时钟 tick 机制**：.at 本地 interval vs 宿主定时注入，执行期按 shell.at
  既有定时先例（若有）定；无先例则宿主注入（60s 低频无投影压力）。
- **pager 网格密度**：≤4 截断为 v1 判定，实机可视性复核后可调。
