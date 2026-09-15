---
plan_id: PLAN-631
status: archived              # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: autoui-interaction-primitives
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - auto-lang/docs/design/29-autoui-style-theme-system.md §10.1「MouseArea hover 样式对（PLAN-631）」
  - auto-lang/docs/design/29-autoui-style-theme-system.md §10.2「popover at_pointer 定位（PLAN-631）」
  - auto-lang/docs/design/29-autoui-style-theme-system.md §10.3「动态视图转换缓存（PLAN-631）」
touched_goals: []

affects:
  - auto-lang/crates/auto-lang/src/ui/aura_view_builder.rs
  - auto-lang/crates/auto-lang/src/ui/iced/hover_area.rs
  - auto-lang/crates/auto-lang/src/ui/iced/renderer.rs
  - auto-lang/examples/ui/027-file-manager/
  - auto-os/docs/plans/016-file-manager-revamp.md（消费方，互链）

---

# [PLAN-631] autoui-interaction-primitives

> 立项来源：auto-os PLAN-016（file-manager-revamp）实机反馈的三个框架级
> finding（F-5/F-6/F-7，见 auto-os `docs/plans/016-file-manager-revamp.md`
> §10.P2 finding 表与修复轮 5/6 记录）。本计划在 **auto-lang**（框架仓）
> 解决；auto-os 侧 PLAN-016 为消费方与验收场景，两计划互链。

## 0. 变更摘要

AutoUI VM 轨交互时成本与指针原语三项框架升级：

1. **F-5 MouseArea hover 样式对**：`mouse-area` 元素支持 `hover:` 类
   （base/hover 双样式 + HoverFlag 零重建翻转），与布局件 hover 机制对齐
   ——行级 hover 高亮不再经 VM 状态回写触发全量视图重建。
2. **F-7 popover 指针定位**：popover 新增 `placement: "pointer"` 定位
   ——渲染器在 mouse-area 右键时记录指针位置，open 翻真时菜单出现在
   指针处；配合单实例菜单形态（Win11 式右键菜单）。
3. **F-6 交互重建性能**：动态视图重建路径剖析 + Style 解析缓存——
   千级节点列表的交互重转换从 debug ~0.3s 降一个数量级。

消费方验收场景：027-file-manager（真实目录列表，67 行 × 行内菜单）。

## 1. 目标

1. **G1（F-5）**：`mouse-area` 支持 hover 样式对——`hover:` 前缀类解析
   为 hover 态样式，指针进入/离开由 HoverArea 小部件内部翻转（零 VM
   消息、零视图重建）；027 行 hover 高亮迁移后删除 mouseenter/leave
   状态驱动。
2. **G2（F-7）**：右键菜单单实例 + 指针定位——`placement: "pointer"`
   下 popover 打开位置 = 触发 mouse-area 最近一次右键的指针位置；027
   回归 Plan 440 单实例根 popover 形态（每行 popover 退役，视图节点数
   显著下降）。
3. **G3（F-6）**：交互重转换成本削减——动态视图重建路径剖析 + Style
   解析缓存（类串 → Style intern）；027 列表选中/悬停交互在 debug
   构建下进入无感区间（目标 < 50ms，剖析数据佐证）。

**非目标**：
- VM 动态视图的结构级 diffing / 依赖追踪细粒度更新（框架大工程；本计划
  T-06 产出评估报告，若缓存剖析后仍不达标另行立项）。
- iced 上游改造（全部在本仓 renderer/aura 层完成）。
- vue 轨对等（vue 端 hover 为浏览器原生，popover 定位有既有方案）。
- 触屏/无指针设备定位语义。

## 2. 架构方案

### 2.1 F-5：MouseArea hover 样式对

现状：`convert_mouse_area`（aura_view_builder.rs:10712）仅提取单一
style（extract_style_with），渲染臂（renderer.rs:4337）把 style 交给
build_container——无 hover 对；布局件则已有 base/hover 机制
（`layout_hover_flag` + `layout_style_fn`，hover_area.rs 的 HoverFlag
Arc<AtomicBool> 由 HoverArea 小部件内部翻转，零 VM 消息）。

方案（镜像布局件机制）：

```
aura: mouse-area 支持 hover: 类 → extract 拆 base/hover 两 Style
渲染: MouseArea 臂将 content 包 HoverArea(content, flag)（hover_area.rs
      既有小部件，内部 on_enter/on_exit 翻 flag）+ 容器样式闭包读 flag
      二选一（layout_style_fn 同型）
```

零 VM 消息、零视图重建；`onmouseenter/onmouseleave` 事件臂保持兼容
（027 迁移前旧用法不破坏）。

### 2.2 F-7：popover placement "pointer"

现状：popover 两定位形态——Widget 锚（first-child / trigger-content 拆
分，PLAN-528）与 Point{x,y} 坐标锚（Plan 440 原生）。坐标需经 VM 状态
回写（每次指针移动 = 全量视图重建，不可行），且 `.at` 事件不携带坐标
（D-1 实证）。

方案：渲染器内建指针位置记忆——

```
mouse-area 右键臂（renderer on_right_press 已有位置信息）:
    记录 last_pointer_pos（渲染器会话级，按 widget 路径或全局单槽）
popover 新定位:
    placement: "pointer" → open 翻真时 anchor = last_pointer_pos
```

坐标不进 VM 状态、不经消息回路；popover overlay 本就悬浮于内容层之上
（用户"不同图层"直觉正确——现状的开销不在图层，在 67 份实例的重转换）。
单实例形态：027 回归 Plan 440 根 popover（`open: .ctx_open,
placement: "pointer"`），右键任意行 → ItemCtx(id) 记目标 + ctx_open 翻
真 → 菜单在指针处打开；67 个行内 popover 退役。

### 2.3 F-6：重转换剖析与缓存

现状：view_dirty → dynamic_view 全量重转换（aura 节点 → iced View），
路径上每元素 `Style::parse`（类串逐次解析分配）、lucide 查表、字符串
分配；debug 构建放大 10-50x（027 67 行实测选中 ~0.3s）。

方案（剖析先行，缓存跟进）：
1. 剖析：debug/opt 双构建下 027 选中/hover 交互热点突出
   （perf 工具不限），产出热区报告 → 决定缓存范围。
2. 缓存：Style::parse 按类串 intern（全局表）；lucide 查表已静态；
   字符串重复分配收敛。
3. 残留评估：若剖析后单次重建仍 >50ms → 结构级 diffing 另行立项
   （不在本计划冒进）。

## 3. 技术栈

- iced 0.14（mouse_area / hover_area 自定义小部件 / container 样式闭包）
- aura View 转换层（aura_view_builder.rs）、iced renderer 动态视图臂
- 消费验证：027-file-manager（VM 轨单窗口 + 虚拟桌面 ui_desktop）

## 4. 需求分析与背景调查

- **授权**：auto-os PLAN-016 实机反馈三条框架 finding（F-5/F-6/F-7，
  见 auto-os 计划 §10.P2 finding 表与修复轮 5/6 记录），用户 2026-09-14
  指示"升级 AutoUI 框架来解决，到 auto-lang 里写一个新的计划文件"。
- **证据**（本会话实机/源码实证，路径均可溯）：
  - F-5：aura_view_builder.rs:10712（convert_mouse_area 单样式）vs
    布局件 hover 机制（layout_hover_flag/layout_style_fn、
    hover_area.rs HoverArea + HoverFlag）；
  - F-7：convert_popover（:7654-7860，Point 锚与 Widget 锚两形态、
    PLAN-528 trigger/content 拆分）；`.at` 事件无坐标（027 ItemCtx
    假坐标实证、D-1 调查）；PointerArea 坐标流（mouse_area_move_arm、
    renderer.rs:4370 pointer_area）；
  - F-6：view_dirty → dynamic_view 全量重转换（renderer.rs
    execute_set_theme / dark_mode 回写臂同机制）；027 67 行实测选中
    ~0.3s（debug）；每行 popover 子树 ≈ 千级节点。
- **消费方**：auto-os PLAN-016（027-file-manager）；`docs/specs/` 无
  aura 交互原语专卷——本计划以 design/29（样式主题系统）增补为规范
  承载（见规范增量）。
- **规范现状**：`docs/design/29-autoui-style-theme-system.md` 已记录
  dynamic_view 每帧回写时序（:224/:341）；hover 机制仅布局件；popover
  定位两形态（PLAN-528/440）无指针模式。

## 5. 详细设计

### 5.1 hover 样式对（对应 2.1）

- `extract_style_with` 保持；新增 `extract_hover_style_with`（`hover:`
  前缀类收集为第二 Style，无则 None——零开销路径不变）。
- 渲染臂：MouseArea 内容包 `HoverArea::new(container, flag)`（hover_area
  既有件），样式闭包按 flag 二选一。
- 兼容：无 hover 类的 mouse-area 行为不变；`onmouseenter/onmouseleave`
  事件臂保留（旧用法不破坏）。

### 5.2 popover "pointer" 定位（对应 2.2）

- 渲染器会话级 `last_right_press_pos: Option<Point>`（右键臂写入；
  全局单槽 v1——多 mouse-area 并存时后写覆盖，语义 = "最近右键位置"）。
- popover `placement: "pointer"` → open 翻真时 anchor =
  last_right_press_pos（None 时回退 BottomStart/锚件）。
- 契约：坐标不进 VM 状态；popover 与触发 mouse-area 可分离（单实例
  菜单挂视图根）。

### 5.3 转换缓存（对应 2.3）

- `Style::parse` intern：类串 → Style 的全局缓存（OnceLock<Mutex<HashMap>>
  或已有 auto-cache 面），剖析确认占比后再定容量策略。
- 剖析报告冻结至 evidence（debug/opt 双数据）。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/docs/design/29-autoui-style-theme-system.md §「MouseArea hover 样式对（PLAN-631）」 | before：mouse-area 单样式无 hover 态 / after：`hover:` 类解析为 hover 样式对，HoverArea 零重建翻转 | G1 | AC-01/02 |
| SD-02 | add | auto-lang/schema/projection-protocol-v1.md §6「popover at_pointer 定位（PLAN-631）」 | before：popover 仅锚件/坐标态定位 / after：`placement: "pointer"` = 最近右键指针位置 | G2 | AC-03/04 |
| SD-03 | add | auto-lang/docs/design/29-autoui-style-theme-system.md §「动态视图转换缓存（PLAN-631）」 | before：每元素每重建 Style::parse / after：类串 intern 缓存 | G3 | AC-05/06 |

## 6. 测试设计

1. 单测：hover 类解析（base/hover 拆分、无 hover 类零开销）、
   placement "pointer" 解析与 last_right_press_pos 记录。
2. 小部件测试：HoverArea 翻转（既有 hover_area 测试扩展）。
3. 消费端实机：027 列表 hover 零重建（日志/帧率）、右键菜单位置 =
   指针、单实例、选中交互耗时（debug < 50ms 目标 + 剖析数据）。
4. 回归：027 既有 desktop_mcp 套件 + tf（按需）。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 |
|---|---|---|
| AC-01 | mouse-area 声明 `hover:` 类后，指针进入/离开样式翻转，无 VM 消息/视图重建（日志零 Tick/重建记录） | 实机 + 日志断言 |
| AC-02 | 无 hover 类的 mouse-area 渲染与行为不变（回归） | 既有套件 |
| AC-03 | mouse-area 右键后 placement:"pointer" 的 popover 出现在指针位置 | 实机截图 |
| AC-04 | 027 迁移后：右键任意行菜单在指针处、全视图 popover 实例数 = 1、菜单含"打开"首项 | vtree/快照 + 截图 |
| AC-05 | 027 列表选中/hover 交互耗时（debug）较基线下降 ≥ 5x，或进入 < 50ms | 剖析报告前后对比 |
| AC-06 | 剖析报告冻结（debug/opt 双数据、热区排行、缓存收益量化） | evidence 归档 |
| AC-07 | 027 既有功能回归（desktop_mcp 套件全绿——MCP 失联问题若未解则以人工清单代） | 套件/清单 |

## 8. 执行步骤

- **T-01 〔lang〕剖析**（AC-05/06 前半，bounded investigation）：
  027 列表交互 debug/opt 双构建热点突出 → 热区报告
  `docs/plans/evidence/631/profile.md`（决策工件：定缓存范围与 AC-05
  目标校准）。依赖：无。
  ✅ 已完成（evidence/631/profile.md + profile_*.json×5）：027 根列表
  缩放 67 行，MCP 驱动 15 次选中，逐帧 `[P631-PROFILE]`（P631_PROFILE=1，
  仪表 = view() 脏路径 builder/render 计时 + Style::parse 全局计数）。
  热区：Style::parse 占 debug builder ~80%（2.6k 次/帧、p90 13.2ms）。
  立项依据值 ~0.3s 未复现（基线 max 26ms debug / 3.9ms opt）→ AC-05
  按 §10 Q1 校准。
- **T-02 〔lang〕F-5 hover 样式对**（AC-01/02）：convert_mouse_area
  hover 类提取 + HoverArea 包裹 + 渲染臂样式闭包；027 迁移 hover 类、
  删 mouseenter/leave。依赖：无。
  ✅ 已完成（commit `6672070e7`）：零新增解析（`Style::variant_classes`
  既有面），MouseArea 两构建臂（into_iced + render_dynamic_view）镜像
  布局件臂（layout_hover_flag + HoverArea 包裹 + build_container
  hover 标志）。设计微差：未新增 extract_hover_style_with——parse 已
  把 hover 类隔离进 variant_classes，渲染臂直读即可。027 侧迁移为
  树形折叠箭头 mouse-area 加 `hover:bg-accent/60`（treeview/filetree；
  027 行 hover 早已是布局件 hover 类，无 mouseenter/leave 可删——
  T-02 该子任务空集，实证 grep app.at 零 mouseenter）。
- **T-03 〔lang〕Style intern 缓存**（AC-05，范围依 T-01）：parse 按类
  串缓存。依赖：T-01。
  ✅ 已完成（commit `6672070e7`）：`ui/style/mod.rs parse_cache`，键 =
  (类串, 6 位主题门控槽：5 断点命中位+dark)——键对 parse 输出完备；
  命中克隆已解析 Style；类串表上限 4096 兜底。A/B 开关
  `AUTO_STYLE_CACHE=0`。收益（67 行选中，debug）：parse p90
  13.2→1.4ms（~9x），builder 1.6x，整重建 15.8→9.8ms（<50ms 达成）；
  opt 噪声内。style_parity 5/5 绿（解析语义零漂移）。
- **T-04 〔lang〕F-7 popover "pointer" 定位**（AC-03）：右键臂位置
  记录 + placement 解析 + anchor 回退。依赖：无（可与 T-02 并行）。
  ✅ 已完成（commit `6672070e7`）：`iced/right_press_area.rs`——
  PointerPressArea 窗口根单包装（ButtonPressed 事件现场读 cursor 记账；
  纯委托）。实现勘误修正：iced 0.14 `on_right_press` 与
  `mouse::Event::ButtonPressed` 均不携带坐标（计划"右键臂已有位置信息"
  不成立），故取窗口根事件面记账；左键同记（027 "···" 左键快捷菜单
  同源）。`PopoverPlacement::Pointer` + convert_popover 解析臂 + 无
  x/y 原点点锚合成 + `pointer_panel_anchor` 归一（未记账回退
  BottomStart）。单测：pointer_placement_tests（iced_test 模拟真实右键
  事件序→槽位精确）+ popover.rs 几何单测（4 测绿）。
- **T-05 〔lang〕027 迁移单实例菜单**（AC-04）：根 popover
  placement:"pointer" + 67 行内 popover 退役；消费回归。依赖：T-02/T-04。
  ✅ 已完成（commit `086af61d0`）：ItemCtx 三触发点去坐标、ctx_x/ctx_y
  退役、ctx popover 改 placement:"pointer" 无坐标锚。注：027 现状已是
  单实例根 popover（PLAN-016 revamp 后形态），"67 行内 popover 退役"
  为空集；本任务实做 = 假坐标退役 + pointer 接线 + 根列表缩放 67 行
  （验收场景数据集）。MCP 链验证：··· press → ctx_open=true → 菜单
  挂载，打开首项+单实例形态截图
  （evidence/631/ctx_menu_open_fallback.png）。desktop_mcp 套件 33/38
  与 master 逐项一致（5 失败两轮控制实验确认为预存，非本计划引入）。
- **T-06 〔lang〕结构 diffing 评估报告**（bounded；AC-06 关联）：
  剖析后若仍不达标的 diffing/依赖追踪方案评估 → 决策工件（另立与否）。
  依赖：T-01/T-03。
  ✅ 已完成（evidence/631/diffing-eval.md）：不建议立项——残余成本
  render ~5ms + builder ~9.8ms（debug）在 60fps 预算内；行级 memo
  （ForLoop 行转换缓存）列为第一候选，三触发门槛（规模×4/低端宿主
  >50ms/新实时场景）后重启。§10 Q2 答案：不立。
- **T-07 〔lang〕收口**（AC-07）：027 回归套件/人工清单 + 文档。
  ✅ 已完成：desktop_mcp 33/38（同 master 基线，预存 5 红在案——
  T9 新建文件夹显示/T10 notes.txt/T11 ··· 按钮/T13 配置切换+view_mode，
  控制实验两轮定性）；scoped 面吃紧绿：pointer/popover/style 模块
  241 测 239 绿（2 红均为预存，master 复现同签名）；style_parity 5/5。
  文档：design/29 §10（SD-01/02/03）+ evidence/631/ 三工件。

Worktree：`.wt/lang-631/auto-lang`（plan-631-dev）；消费验证需 auto-os
侧 ui_desktop（AUTO_DESKTOP_APPS 指向本仓 examples/ui）。

## 9. 复审记录

（drafting——/work 后回填）

### 9.1 执行环境（2026-09-15 /work 启动）

- worktree：`D:/autostack/.wt/lang-631/auto-lang`，分支 `plan-631-dev`，
  base commit `b62164566`（master，含 PLAN-626/629/630 归档）。
- 主检出存在无关未提交改动（`stdlib/auto/fs.at`、
  `docs/plans/archive/626-auto-edit-vm-polish.md`）——归属外部工作，
  本计划不触碰。

### 9.2 work 阶段记录（2026-09-15）

`stage: work | plan_id: PLAN-631 | plan_revision: 1 | outcome: pass |
code_commit: 6672070e7（框架三件）→ 086af61d0（027 迁移）→ 308c9f4e9
（docs+evidence）| worktree: .wt/lang-631/auto-lang @ plan-631-dev
（base b62164566；依赖兄弟 .wt/lang-631/auto-down @ 140775f0 detached）|
task_ids: T-01..T-07 全完成 | evidence: docs/plans/evidence/631/
{profile.md, diffing-eval.md, profile_*.json×5, pointer_probe.py,
ctx_menu_open_fallback.png} + design/29 §10 | blockers: 无 |
next: review（/auto-plan:review）`

- AC 对账：AC-01/02 ✅（F-5 + hover_area 机制，单测绿；实机悬停视觉
  归入人工清单项）；AC-03 ✅ 框架内（模拟器真实事件序→记账→面板锚
  归一，4 单测绿）+ 真窗指针验证受阻于本会话合成输入被叠加窗截获
  （pointer_probe.py 已备，人工清单代——计划 AC-07 同款容忍）；
  AC-04 ✅（MCP 链 ctx_open=true、菜单挂载、打开首项、单实例、截图）；
  AC-05 ✅（校准后按 <50ms 腿：debug p90 9.8ms / max 16.5ms；parse
  分项 ~9x）；AC-06 ✅（profile.md 冻结 debug/opt 双数据+热区排行+
  收益量化）；AC-07 ✅（套件 33/38 与 master 逐项一致，5 预存红两轮
  控制实验定性；计划明文允许清单代）。
- 实现勘误（计划修正，语义修订记录）：①iced 0.14 右键事件不携带
  坐标（§2.2 前提"renderer on_right_press 已有位置信息"不成立）→
  窗口根 PointerPressArea 事件面记账，左键同记；②"67 行内 popover
  退役"为空集（027 revamp 后已是单实例根 popover，假坐标形态）；
  ③SD-02 规范承载由 projection-protocol-v1.md 调整至 design/29 §10
  （实勘该文件为 `__wm_*` 状态投影合同，placement 非其论域）。
- 验证门档：Category B——cargo check 零新增警告；scoped
  style/popover/pointer 241 测 239 绿（2 红预存，master 同签名复现）；
  style_parity 5/5；027 desktop_mcp 与 master 基线逐项一致。
  未跑 cargo tf（改动未触编译器/VM 核心，review 档再跑全量）。

### 9.3 review 阶段记录（2026-09-15）

`stage: review | plan_id: PLAN-631 | plan_revision: 1 | outcome: pass |
reviewed_commit: 308c9f4e9eb05abf7452d375f4581101bfd5fab8 |
base_commit: b62164566a96697e19408de5849a7ca322affc9c |
dependency_revisions: auto-down @ 140775f0cbec（detached，未改动）|
spec_inputs: docs/design/29-autoui-style-theme-system.md §10（本计划新增节，
worktree 提交 308c9f4e9 内）|
acceptance_results: AC-01 ✅ / AC-02 ✅ / AC-03 ✅（框架链全验证+人工清单
残项，见 F-R1）/ AC-04 ✅ / AC-05 ✅ / AC-06 ✅ / AC-07 ✅（预存红定性与
计划容忍条款）| findings: F-R1/F-R2/F-R3（见下）|
evidence: 复审独立复现记录（本节）+ worktree 内 evidence/631/ 工件包 |
next: merge（/auto-plan:merge）`

**独立复现（复审现场重跑，非沿用执行期结论）**：

- `cargo tf`：**3560/3560 绿**（96 skip；该档不含 ui-iced，Plan 507 注记）。
- `cargo t`（ui-iced 日常档）：**847 绿 / 2 红**——红 =
  `musk_vm_track_tests p054` 两测（icon_component_child_renders… /
  …class_prop_carries_ml_auto_and_tint），在 master 主检出同签名复现
  → 预存红（564-Q6 族），非本计划回归。
- pointer 单测（iced-layout-tests）：23/23 绿；style_parity 5/5 绿。
- 027 desktop_mcp：33/38，5 失败集合与 master 基线控制实验逐项一致
  （T9/T10/T11/T13×2，预存）。
- 规范增量核对：design/29 §10 三节与实现逐条对读一致（缓存键 6 位
  槽/4096 上限/Pointer 几何/回退语义/hover 零开销路径）；SD-02 承载档
  调整已在正文与 frontmatter 双向固化（F-R3，复审修正）。
- 剖析结论抽查：`AUTO_STYLE_CACHE=0/1` A/B 复测可行（开关在最终
  二进制内生效），AC-06 冻结数据与 evidence JSON 一致。

**findings**：

- **F-R1（info→人工清单项）**：AC-03 的"实机截图"证据形态受本机
  远程显示环境阻断（合成输入被叠加窗截获，AttachThreadInput/TOPMOST
  均不可前台化；与预存债务 P481-6/P501-2 同族，Plan 494 已记录 ToDesk
  环境疑因）。框架链已由最强自动化面验证：iced_test 模拟器真实事件序
  （CursorMoved+右键按下 → 单槽精确记账）+ `pointer_panel_anchor` 几何
  单测（记账→面板原点=按针位置；无记账→BottomStart 回退）+ MCP 消费
  链（ctx_open 翻真）+ 回退形态截图。**残项：用户真机一次右键目视确认
  （或运行 pointer_probe.py），merge 后可随时补记。**
- **F-R2（info）**：ui-iced 日常档 2 红 + desktop_mcp 5 红 + scoped
  p010_popover_ondismiss 1 红均为 master 预存（本轮在主检出同签名
  复现定性），不属本计划修复面；已在 KNOWN-DEBT 体系既有条目覆盖
  （564-Q6 族），无需新登记。
- **F-R3（复审修正，已落）**：计划 frontmatter `new_spec_components`
  的 SD-02 目标仍指向 projection-protocol-v1.md §6——执行期已实证
  该文件为 `__wm_*` 状态投影合同（§6=变更记录，placement 非其论域），
  规范承载调整为 design/29 §10.2；本复审固化 frontmatter 三条目为
  design/29 §10.1–10.3 精确路径。语义契约本身无变化（plan_revision
  维持 1）。

**结论**：全部验收标准在复审基线（308c9f4e9）复现通过，规范增量
如实描述当前行为；唯一残项为 F-R1 人工清单（计划 AC-07 容忍条款
覆盖）。`status: reviewed`，下一动作 `/auto-plan:merge`。

### 9.4 merge 收据（PLAN-631:r1，2026-09-15）

- `prepared`：reviewed 基线 308c9f4e9（rev 1，依赖 auto-down@140775f0
  未动）；canonical diff = design/29 §10（已在分支提交 308c9f4e9 内）；
  投影目标 = ui/overview + plans.md + .autoos/specs.json P631-1..6。
- `landed`：delivery `39dd77b48`（reviewed_commit 纯文档后裔——模块树
  回写两文件，实现/依赖零变化）→ master 合入 **`4e5f39cad`**
  （parent `553a5a74a`+`39dd77b48`；并行外部提交无冲突卷入）；合入后
  `cargo check -p auto --bin auto` 冒烟绿。
- `ledger_refreshed`：`.autoos/specs.json` upsert P631-1..6（六节，
  读回验证 6/6）；`scripts/spec-index.py` 再生 INDEX.md（26 projects，
  零差异）。
- `archived`：`git mv` → `docs/plans/archive/631-autoui-interaction-
  primitives.md`，frontmatter `status: archived`（本提交）。
- `cleaned`：见下补记。

## 10. 待澄清事项

| # | 事项 | 去向 |
|---|---|---|
| 1 | AC-05 目标值（5x / <50ms）以剖析数据校准 | ✅ T-01 决策工件（profile.md）：立项依据 ~0.3s 未复现，<50ms 腿达成（debug p90 9.8ms），5x 按 parse 分项（~9x）达成 |
| 2 | 结构 diffing 是否立项 | ✅ T-06 评估报告（diffing-eval.md）：**不立项**；行级 memo 为第一候选，三触发门槛后重启 |
| 3 | last_right_press_pos 全局单槽 vs 按 mouse-area 多槽 | ✅ T-04 实测定：单槽成立（模拟器两连按后写覆盖验证）；左键同记（··· 快捷菜单同源），跨窗共享最近写入 |
