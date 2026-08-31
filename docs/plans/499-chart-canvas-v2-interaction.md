---
plan_id: PLAN-499
status: drafting
feature_name: chart v2 canvas 交互——axisPointer 跟随光标/mousemove 流/动画过渡
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-08-31

supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: v2 canvas 交互节(mousemove 流/crosshair/动画)"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——交互能力对齐(canvas 轨)"
affects: [docs/specs/auto-lang/ui, crates/auto-lang/src/ui, auto-musk]
current_step: 0
total_steps: 5
---

# [PLAN-499] chart v2 canvas 交互——axisPointer 跟随光标/mousemove 流/动画过渡

## 变更摘要

Plan 484 的 chart 交互 v1(锚定 tooltip/槽位高亮)受两轨限制:VM svgdoc 静态无交互、
mousemove 高频流未开。本计划启动 **v2 canvas 阶段**:跟随光标的十字线(axisPointer)、
逐扇区/逐点 hover 的动画过渡。前置调研: Plan 386 RenderQueue、iced canvas Program、
mousemove 事件流进 VM 的性能设计。

## 目标

1. mousemove 事件流进 VM 的**限频通道**(目标 ≤30Hz 采样,不冲击 RenderQueue);
2. **axisPointer 十字线/竖带指示**跟随光标(折线图),tooltip 跟随移动;
3. VM donut **逐扇区直接 hover**(svgdoc 静态限制的解除路径)或等价方案;
4. hover 态**过渡动画**(vue CSS transition / VM 数值插值),双端观感对齐。

## 架构方案

- **mousemove 流**: iced mouse_area on_move 事件 → 限频采样(帧同步,每帧最多一条)
  → 走 Plan 386 RenderQueue 批量通道进 VM → 组件内命中计算(纯派生);
- **donut 逐扇区**: 两条路径评估——(a) svgdoc 事件能力扩展(svg-text 同族,需 resvg/
  usvg 交互层,成本高);(b) canvas 层直接绘制扇区+代码命中测试(ECharts 模型,推荐);
- **动画**: vue 侧 CSS transition;VM 侧数值插值(请求帧驱动)——双端观感对齐策略在
  调研后定。

前置依赖: Plan 386(RenderQueue)、Plan 484(v1 交互与组件形态)、Plan 492(引擎三族,
已归档)。

## 需求分析与背景调查

- 来源: Plan 484 交互 v1 的既知边界(DEBT 挂账): 跟随光标 axisPointer、hover 动画、
  VM donut 逐扇区 hover(svgdoc 静态限制)。
- Plan 386: RenderQueue 批量渲染通道——mousemove 流的性能承压面。
- 参考: ECharts axisPointer(line/shadow/cross 三形态)、emphasis/doubleDownplay、
  tooltip.trigger(axis/item)。

## 详细设计

（调研后填写——本计划为研究型,调研产出: mousemove 流的采样/批量/帧同步方案、
canvas Program 桥接的组件协议、双端动画对齐策略）

## 测试设计

- mousemove 流压测(30Hz 持续流下 RenderQueue 队深/帧率);
- axisPointer 双端渲染一致性;
- 动画帧率基线(vue CSS vs VM 插值)。

## 验收标准

1. 折线图 hover 出现跟随光标的十字线 + 锚定 tooltip,双端一致;
2. 持续 mousemove 流下无帧率塌陷(基线对照);
3. donut 逐扇区 hover(vm 轨)按调研结论落地或明确不可行结论归档。

## 执行步骤

- [ ] M1 调研: Plan 386 RenderQueue + mousemove 流方案设计(产出设计文档)
- [ ] M2 mouse_area on_move 限频流通道落地 + 压测
- [ ] M3 axisPointer 十字线(折线)双端实现
- [ ] M4 donut 逐扇区方案落地或归档结论
- [ ] M5 动画过渡(vue CSS / VM 插值)对齐
- [ ] M6 复审与归档准备

## 复审记录

（复审时填写）

## 待澄清事项

1. mousemove 流的采样率与批量策略需要压测数据支撑(M2 前置);
2. canvas Program 桥接(437 v2 决议)与本计划的关系: 本计划只做交互流,绘制桥接
   是否同批实施需 M1 调研后定。
