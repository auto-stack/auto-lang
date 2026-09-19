# 浮层交互投递语义(absolute/Overlay → iced Stack + opaque)

## 范围

into_iced(Rust Component 轨)与 render_dynamic_view(VM 轨)把
`absolute`/`Overlay` 浮层映射为 iced `Stack` + `opaque` 后的鼠标
事件投递规则。来源 PLAN-023 T-01 判决工件
(auto-term `docs/plans/evidence/023/t01-verdict.md`,2026-09-19;
根修 commit plan-023-dev fc6ccc810)。022 SD-01(命中区几何归属,
terminal-widget-chrome.md)为本篇的下层消费方:本篇定"层间归谁",
彼篇定"层内组件归谁"。

## 硬规则(双端渲染器同源)

1. **捕获边界 = 浮层内容矩形**。`opaque()` 只包浮层内容元素,几何
   spacer/padding 留在捕获边界外。按钮按压落点在内容矩形内 → 本层
   认领(opaque 对 ButtonPressed 落点在自身 bounds 内无条件
   capture_event);落点在 spacer/空白区 → 穿透到下层。与 CSS
   absolute 命中语义一致。反例(禁止):对含 spacer 的整层包
   opaque——Stack 子层共原点(iced stack.rs layout 对所有子层给
   同一 ZERO→size limits),spacer 行矩形会覆盖从原点起的全部下层
   区域,吞掉下层槽位全部按压(PLAN-022 右面板交互死根因)。
2. **capture 生命周期:每事件复位**。iced_runtime `UserInterface::
   update` 对每事件新建 `Shell`(user_interface.rs:318),capture
   不跨事件泄漏;`Stack::update` 逆序(顶层先)遍历,遇
   `is_event_captured()` 即短路。因此"第一轮后全灭"类症状永远
   不是 capture 泄漏,应查捕获边界几何或测量方法。
3. **top 偏移用 Column padding,不用零宽 Space**。Shrink 列内
   零宽(w=0)Space 的主轴高度被 flex 吞(R8 实录:w=0 → 子件
   y 不动;w=1 → 正确;iced_core-0.14.0 flex.rs 源码推演与二进制
   矛盾,机理归因 KNOWN-DEBT 023-R8)。left 偏移用行内非零宽
   Space(可靠)。right 锚 Fill 行夹持为退路(Fill 根捕获过宽,
   当前无消费方)。
4. **光标抬升是分层悬停语义,不在 widget 层对抗**。Stack 对
   "非顶层子级报告非 None mouse_interaction 的认领点"抬升光标
   (Cursor::Levitating → position()=None → is_over()=false),
   下层不再响应 hover/按压门控。这是正确行为(上层交互件认领的
   点不属于下层);022 的"终端 mouse_interaction 抬升级联"假设
   就此终结(终端 mouse_interaction 恒 None 是配合项,非修复点)。
5. **模拟器与实车同轨**。iced_test simulator 与实车走同一条
   `UserInterface::update` + Stack + opaque 路径,投递语义一致;
   差异仅在实车每帧重灌 view + winit 真事件。headless 复现对
   投递缺陷具有判定力。

## 测试断言面纪律

`View::on_click(closure)` 在 **builder 期立即调用一次** harvest
静态消息(`view.rs:on_click` = `button_onclick = Some(handler(()))`),
闭包副作用此后不再发生。headless 交互测试一律以**消息流**断言
(`simulator::into_messages` / 分次 simulate 后聚合),禁止闭包
计数副作用——PLAN-022 T-05 的"交替点击全灭"轨迹即此伪影
(计数恒 1,与投递无关),022 的"移除 opaque 轨迹不变"实验亦被
其致盲。样例:`p022_stack_click_tests.rs`(转正面)+ `examples/
p023_probe.rs`(V/R 矩阵 + 载具形状 R3)。

## 已知限制

- 零宽 Space flex 主轴失效的 iced 内部机理未归因(023-R8,
  上游勘验非本层义务;padding 规则已绕开)。
- divider 拖拽捕获层(z-30 全窗 MouseArea,Press 期 dragW=clientW)
  依赖"捕获边界=自身矩形"规则:拖拽期有意吞全窗按压,Drop 后
  归零释放——常驻不构成空闲态杀手(c793bd2 语义,T-04 探针佐证)。
