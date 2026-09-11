---
plan_id: PLAN-611
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: vwin-thumb-swr-titlemenu-outside-close
author: ["zhaopuming"]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 5
total_steps: 5
---

# [PLAN-611] vwin-thumb-swr-titlemenu-outside-close

> **立案形态说明**：用户实机报障两题后指令「直接修」（免 worktree，master 主检出
> 直改），本计划为**事后追记**——完整记录本会话的根因裁决、代码改动与验证证据。
> 修复均为 auto-lang 框架层缺陷（iced 桌面轨），无 `.at`/schema/spec 面，
> `supersedes/new_spec_components` 留空。

## 变更摘要

桌面壳 iced 轨两处缺陷根修（均在 `crates/auto-lang/src/ui/iced/`）：

1. **窗口缩略 SWR**（用户报障：任务栏 hover 缩略图每 ~2s 跳变 fallback icon；
   复诊发现虚拟桌面/app 切换后的缩略"失效"同根因）——`window_thumbnail`
   渲染臂从「TTL 过期即删条目跌 fallback」改为 **stale-while-revalidate**：
   过期旧图续帧 + `request_capture` 冷却去重静默重抓。
2. **标题栏右键菜单点外关闭补线**（用户报障：无外部点击关闭能力）——
   PLAN-526 T37 的「任意 GlobalPress 已清」合同**在案但从未接线**，
   `WmCommand::GlobalPress` 臂补 `title_menu = None`。

两文件 `+80/−12`；新增测试 1 枚；无 schema/协议/`.at` 面。

## 目标

- G1：hover 任务栏图标持续显示缩略图，不随 `SNAPSHOT_TTL`(2s) 到点跳变 icon。
- G2：虚拟桌面切换回来 / Alt-Tab 召唤 switcher，缩略立即以"上次旧图"呈现
  （异步刷新为新图），不再冷启动断档显示 fallback icon。
- G3：窗口标题栏右键菜单，左键点外部（桌面空白/其他窗/本窗其他区域）即关闭；
  点菜单项动作执行且菜单收起。

## 需求分析与背景调查

### 缺陷一：缩略图 2s 跳变（TTL 过期帧裸跌 fallback）

机制链（复现链路与用户截图一致）：

- 任务栏条目 hover → `shell.at:202` popover 内 `window_thumbnail(wid,
  fallback_icon)`；switcher 行（`switcher.at:77/84`）与 pager 预览同消费该渲染臂。
- 渲染臂每帧查进程级 TTL 缓存（PLAN-497 T2）：命中直绘；miss → 本帧
  fallback lucide icon + `request_capture`（宿主 update 排空 → 一次整窗
  screenshot → 裁剪降采样 → `cache_put`）。
- **根因**：`snapshot_window` 对过期条目「惰性清除」实现成读取即删——
  hover 持续时每 2s（`SNAPSHOT_TTL`，恰为用户观察到的切换周期）必然跌入
  miss 分支，等异步抓取回填的几个帧里裸跌 fallback icon，循环往复。
- **复诊扩展**：分区切换本身不清缓存（`wm_set_workspace` 只动 WM 状态），
  切换后的"失效"是目标窗上次可见已超 TTL、条目早过期——与 hover 闪烁同根因；
  `invalidate_all`（主题/壁纸/dock 重排）是像素真正作废的场景，保持硬清不动。

### 缺陷二：标题栏右键菜单点外不关（T37 合同漏接线）

- 机制分野：icon/任务栏/blank/分区条目/关机确认 5 处菜单走 `.at` 层
  `popover` + `ondismiss`（点外/Esc 由 popover 机制负责，PLAN-010/011 修好
  的即此链）；标题栏菜单是唯一 chrome 自绘浮层（`virtual_window.rs
  title_menu_panel` + `WmCommand::TitleMenuOpen/Close` 状态机）。
- **根因**：`title_menu` 全仓仅 open（`TitleMenuOpen` 臂）与显式
  `TitleMenuClose` 两个写点；`GlobalPress` 臂只做 `focus_soft` 命中聚焦，
  T37 设计注释「任意 GlobalPress 已清」从未实现——菜单一旦打开状态上永不
  撤除（连点菜单项都不关，项消息不含 close）。且 `GlobalPress` 仅映射左键
  按下（renderer 订阅面 `ButtonPressed(Left)`），右键点外连事件源都没有。

## 架构方案

行为语义与 popover 阵营对齐（右键开、点外关、点项收），机制不强行统一：
标题菜单锚定在虚拟窗 chrome 坐标系（Rust 直绘层），与 `.at` popover 的
视图树锚定不同构；统一需把菜单内容搬入 shell.at 或扩展窗口投影协议，
留待 chrome 侧菜单数量到达重复度阈值再立项（见待澄清③）。

## 详细设计

**T1（snapshot.rs）**：新增 `snapshot_window_stale(wid) ->
Option<(WindowSnapshot, bool)>`——存在即返回（含过期，不删除），附新鲜度。
原 `snapshot_window` 「仅新鲜即 Some、过期即删」合同原样保留（switcher
`mru_thumbs` 就绪标记与既有测试消费面零扰动）。模块头注释补 SWR 契约节。

**T2（renderer.rs `AbstractView::WindowThumbnail` 渲染臂）**：换读口三态——
新鲜直绘；**过期 → 绘旧图 + `request_capture`**（500ms `REQUEST_COOLDOWN`
去重；重抓落地 `cache_put` 原子覆盖，无中间空档）；真 miss → 维持
fallback icon + 请求（headless 语义不变）。TTL 由「视觉断档点」转性为
静默刷新节奏，不引入后台定时器，不违 PLAN-497「无后台定时刷新」定案。

**T3（renderer.rs `WmCommand::GlobalPress` 臂）**：臂首无条件
`title_menu = None`（命中聚焦逻辑不变、先后有序）。落在菜单项上的按压经
订阅面原始事件同帧抵达，菜单收起 + 项动作照常执行（顺带修掉「点最小化
菜单还挂着」）。右键点外**刻意不接**：右键按下无全局映射，补映射会与
「右键打开菜单」产生同帧 open/close 竞序（次序无保证），风险大于收益。

**T4（snapshot.rs 测试）**：`t2_snapshot_stale_read_keeps_entry`——空缓存
miss / TTL 内报新鲜 / 过期续存不删且新鲜度翻转 / invalidate 硬撤对 SWR
读口同样生效，四段断言。

## 测试设计

| 层 | 手段 |
|---|---|
| 单测 | `cargo t t2_snapshot`（5/5，含新增 T4） |
| 模块档 | `cargo t iced`（ui::iced 全量 173） |
| 兜底档 | `cargo t`（日常全量 4747，含 schema_drift/docs_gen/component_registry） |
| 消归法 | 每轮与干净树 stash-对拍失败集，全等才算过 |

## 验收标准

- AC-1：`t2_snapshot_stale_read_keeps_entry` 绿（SWR 合同面）。
- AC-2：`cargo t iced` / `cargo t` 失败集与干净树全等（零新增）。
- AC-3：实机（重启桌面进程）——① hover 任务栏图标 >5s 不跳 icon；
  ② 切走再切回虚拟桌面，hover 立即出上次缩略图并静默刷新；③ 标题栏右键
  菜单点外关闭、点项动作执行且收起。（待用户实机复核，移交 review 关。）

## 执行步骤

- T1 snapshot.rs SWR 读口 + 模块头契约注释 [✅ 已完成]
  证据：`snapshot_window_stale` 落位 `cache_put` 之后；`snapshot_window`
  语义未动。
- T2 renderer.rs WindowThumbnail 渲染臂三态化（过期续绘 + 静默重抓）[✅ 已完成]
  证据：臂注释更新 SWR 语义；`wid_opt` 单次 parse 复用。
- T3 snapshot.rs 新增 `t2_snapshot_stale_read_keeps_entry` [✅ 已完成]
  证据：`cargo t t2_snapshot` 5/5 绿。
- T4 renderer.rs GlobalPress 臂补线 title_menu 撤除 [✅ 已完成]
  证据：臂首两行写点；`cargo t iced` 对拍无新增失败。
- T5 全量对拍验证 [✅ 已完成]
  证据：① SWR 后 `cargo t iced` 172/173（唯一红=lucide `film` 缺口，干净树
  同红，与 PLAN-611 无关）；② 日常全量 4747 测试 21 败，stash 对拍**失败集
  全等**（plan370×3/plan492/ffi_dual/aura_view_builder/desktop_protocol
  coverage/ui::layout×15/lucide，均 master 存量）；③ 标题菜单补线后 iced 档
  失败集 {dock_pager_hover_popovers, lucide} 与同 HEAD 干净树全等（前者见
  待澄清②）。

## 复审记录

（待 `/auto-plan:review`；AC-3 实机三判据请复审时向报障用户索取复核结论。）

## 待澄清事项

1. **标题菜单残余缝隙**（在案不修）：Esc 不关（popover ondismiss 含 Esc）；
   右键点外不关（竞序规避，见 T3）。待实机反馈再议权重。
2. **master 并发红：`desktop_mcp_dock_pager_hover_popovers`**——本会话工作
   期间 master 先后落了 PLAN-608 merge（VM 分发路径）、PLAN-610、PLAN-008
   merge（并发会话所为一一在案）；当前 HEAD 干净树该测试 6/6 稳定红
   （`HoverEnd: HandlerNotFound`）。疑似 608 引入，而 608/610 复审档
   （`tv`/`tf`）**不带 `ui-iced`**，盖不住 iced 桌面层——建议 608/610 责任
   会话补跑 `cargo t iced` 定责。本计划改动与其无关（对拍全等实证）。
3. **右键菜单机制统一**：现为「`.at` popover×5 + chrome 自绘×1」两套，行为
   语义已对齐；chrome 侧菜单再增多时立项把标题菜单 popover 化（需窗口几何
   投影进 shell.at 或协议扩展）。
4. lucide `film` 命中表缺口（`lucide_icon_coverage_manifest_all_hit` 红）
   为 master 存量，归在途 606/相关会话，不在本计划范围。

## 互链

- auto-lang：PLAN-497（快照核心/TTL 定案）、PLAN-526（T30/T36 popover 化、
  T37 标题菜单原案）、PLAN-589（shell/ P-7 权威翻转）、KNOWN-DEBT（待澄清②
  定责后回填）。
- auto-os：PLAN-010/011（popover ondismiss 解耦与命中带根修——本计划缺陷一/
  二的同族前案）、`shell/`（唯一真相源，本计划未触碰）。
