---
plan_id: PLAN-631
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-interaction-primitives
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - auto-lang/docs/design/29-autoui-style-theme-system.md §「MouseArea hover 样式对（PLAN-631）」
  - auto-lang/schema/projection-protocol-v1.md §6「popover at_pointer 定位（PLAN-631）」
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
- **T-02 〔lang〕F-5 hover 样式对**（AC-01/02）：convert_mouse_area
  hover 类提取 + HoverArea 包裹 + 渲染臂样式闭包；027 迁移 hover 类、
  删 mouseenter/leave。依赖：无。
- **T-03 〔lang〕Style intern 缓存**（AC-05，范围依 T-01）：parse 按类
  串缓存。依赖：T-01。
- **T-04 〔lang〕F-7 popover "pointer" 定位**（AC-03）：右键臂位置
  记录 + placement 解析 + anchor 回退。依赖：无（可与 T-02 并行）。
- **T-05 〔lang〕027 迁移单实例菜单**（AC-04）：根 popover
  placement:"pointer" + 67 行内 popover 退役；消费回归。依赖：T-02/T-04。
- **T-06 〔lang〕结构 diffing 评估报告**（bounded；AC-06 关联）：
  剖析后若仍不达标的 diffing/依赖追踪方案评估 → 决策工件（另立与否）。
  依赖：T-01/T-03。
- **T-07 〔lang〕收口**（AC-07）：027 回归套件/人工清单 + 文档。

Worktree：`.wt/lang-631/auto-lang`（plan-631-dev）；消费验证需 auto-os
侧 ui_desktop（AUTO_DESKTOP_APPS 指向本仓 examples/ui）。

## 9. 复审记录

（drafting——/work 后回填）

## 10. 待澄清事项

| # | 事项 | 去向 |
|---|---|---|
| 1 | AC-05 目标值（5x / <50ms）以剖析数据校准 | T-01 决策工件 |
| 2 | 结构 diffing 是否立项 | T-06 评估报告 → 用户裁定 |
| 3 | last_right_press_pos 全局单槽 vs 按 mouse-area 多槽 | T-04 实测定（先单槽） |
