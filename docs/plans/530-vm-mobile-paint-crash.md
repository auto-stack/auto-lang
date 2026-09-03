---
plan_id: PLAN-530
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: VM mobile 断点双份绘制 + 启动内存崩溃专项
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-03T15:30:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-530] VM mobile 断点双份绘制 + 启动内存崩溃专项

## 变更摘要

立项收录 widgets-gallery 人工检查期间发现的两个 **VM shell 层**问题
（均登记于 PLAN-528 观察区/W10，根因已初步定位但修复属 VM 架构深水区，
按用户裁决单独立项深挖）：

1. **A（源 W10）**：窗口宽 ≤768（mobile 断点）时页面内容整体 ×2 叠印
   ——绘制层双通道合成，控件树结构单份；
2. **B（源 OBS-1）**：VM 模式启动渲染期原生崩溃
   `memory allocation of 721554505560 bytes failed`（约 721GB 荒谬分配），
   间歇性。

两问题均不出示例层可修范围,涉及 iced 0.14 虚拟窗口表面生命周期、
fit 缩放合成与 Plan 527 T7 响应式重建的交互。

## 目标

1. A：定位 mobile 断点下双表面的来源（创建/残留），断点翻转时旧表面
   正确销毁,任意窗口宽下内容仅绘制一份。
2. B：定位 721GB 分配的调用点（尺寸计算溢出/未初始化），消除崩溃；
   崩溃可稳定复现时补回归测试。
3. 两问题修复后,widgets-gallery VM 模式在 700–1440px 全宽度区间
   逐页截图无异常（含跨断点往返）。

## 架构方案

- 排查入口 A：`crates/auto-lang/src/ui/iced/virtual_window.rs`、
  `renderer.rs`（fit 通道 Plan 512 / dock 合成 Plan 463 / 表面注册表）、
  `session.rs`（window_size 回写）；与 `crates/auto-lang/src/ui/style/mod.rs`
  Plan 527 T7 响应式门控的重建时序。
- 排查入口 B：崩溃点在首个 view 构建期（wgpu/iced 初始化后、Plan 412
  flex-wrap/sticky 降级告警之后）；疑似尺寸计算溢出（未初始化/负数→
  usize 回绕）。RUST_BACKTRACE=full + windbg/cdb 或 分页二分（逐页排除）
  定位。721GB ≈ 0xAB_XXXX_XXXX 量级,高度疑似 f32 NaN→usize 转换或
  元素数乘法溢出。
- 修复落点倾向：VM shell 表面生命周期管理（A）、尺寸计算防御（B）；
  均在 `crates/auto-lang` 内,不动示例。

## 需求分析与背景调查

- 问题 A 现象与对分实验全文：`docs/plans/528-widgets-gallery-review-fixes.md`
  W10 节（结构单份/绘制双份/两种宽度约束叠印/断点门控本身正确）。
- 问题 B 现象：PLAN-528 OBS-1（2026-09-03 02:48 首次记录,当日后续
  VM 启动均未复现——间歇性/疑似页面相关;同日 togglegroup/W10 实验
  多次 VM 启动正常）。
- 相关上游：Plan 527 T7（responsive 断点,验证仅覆盖样式解析层分叉,
  未覆盖真实窗口 resize/冷启动全回路）;Plan 512（fit 缩放一次性锁定）、
  Plan 463（WM→shell 状态注入）。
- 复现配方：
  - A：`examples/widgets-gallery/pac.at` `window: "700x900"` 冷启动即现
    （实验后已还原 1440x900）;推断 resize 跨 768 亦可触发。
  - B：master @ 7ab140c41 时代 `auto run`（VM merged）首启即崩;
    复现概率不稳定,建议保留当日环境线索（RTX 4060 Ti / Vulkan backend）。
  **强复现配方（2026-09-03 下午,W11 排查中实证）**：widgets-gallery
  VM 启动后停在首页（LineChart/DonutChart 动画 AnimLnTick/AnimDnTick
  持续 tick）,约 1–5 分钟内进程静默死亡（exit 无 panic、无 memory
  allocation 行,日志末尾即动画 tick 事件）。离开首页则存活。崩溃与
  首页动画 tick 强相关——优先 bisect 图表动画路径。

## 详细设计

（深挖后回填——当前为立项档案。预估方向：）

- A 假设 H1：断点翻转触发 view 重建时,虚拟窗口表面注册表新增了
  mobile 表面而旧表面未注销 → dock 合成叠印两表面（各自持有
  不同时期的布局宽度）。
- A 假设 H2：fit 缩放通道与直绘通道同时上屏（fit_pending 未清）。
- B 假设 H1：某 widget（疑似 svg/chart 家族或 grid 占位）在
  viewport 极值下 size 计算 f32→usize 回绕。
- B 假设 H2：AbstractView 缓存跨断点复用时 children 数被污染放大。

## 测试设计

- A：headless 断点翻转样本（700↔1440 往返）+ surface 注册表数量断言;
  MCP 截图逐宽度扫描（已有 69 页扫描基线可比对）。
- B：崩溃复现后,最小化 .at 样本沉淀为回归测试
  （`cargo t --features test-vm-files` 档,参照既有 VM 文件测试）。
- 回归底线：widgets-gallery 69 页 VM 逐页截图扫描（PLAN-528 W5 既有
  工具链,MCP 驱动脚本 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`）。

## 验收标准

- [ ] A：mobile 断点下任意页面内容仅绘制一份（树单份=像素单份）;
      跨断点往返（700↔1440 多次）无残留表面。
- [ ] B：721GB 崩溃根因定位并有确定性修复;原始复现路径（widgets-gallery
      VM 首启）连续 10 次启动无崩溃。
- [ ] 全程 cargo t iced + 既有 VM 文件测试绿。

## 执行步骤

（深挖时展开为原子任务;当前立项目录:）

1. [ ] A-复现固化:700x900 冷启动样本 + MCP 截图基线存档
2. [ ] A-表面追踪:在 surface 注册表/绘制循环加诊断计数,断点翻转时
       打印活跃表面清单（宽高+创建帧）,确认双表面假设 H1/H2
3. [ ] A-修复+回归
4. [ ] B-复现:bisect 页面样本（逐页独立 VM 启动）,RUST_BACKTRACE=full
       抓分配栈
5. [ ] B-修复+回归
6. [ ] 双修复后 widgets-gallery 全宽度区间逐页扫描验收

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. B 的崩溃是否与特定页面（chart/svgdoc 家族）强相关——首次崩溃时
   用户停在首页,但渲染是全树构建;待 bisect 证实。
2. A 的双表面在 dock/多窗口形态（非全屏单窗）下是否也复现。
