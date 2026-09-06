# Web 生态战略（Vue 主臂 + 契约独立 + 发射臂扩展）

**日期**: 2026-09-06
**状态**: Draft（骨架，待扩充）
**关联**: [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md)（伞形总战略）、
[GOAL-007/012](../../specs/goals.md)、Design [20](../20-autoui-separation-architecture.md)

---

## 0. 已定裁决（2026-09-06 讨论）

1. **契约独立原则升格**：AutoUI 契约（schema / design tokens / 行为规范，527 清单驱动
   路线）独立于任何宿主框架；Vue、VM/iced 是当前两臂，未来臂的增删不等于生态重建。
2. **Vue 主臂不动摇**：中国市场（非大厂尤甚）Vue 份额高于 React，与 Auto 目标用户重合；
   且 Vue 臂沉没投资最深（widgets/forge-ui/playground/双端 parity 机器均挂在它上面）。
3. **React = 发射臂扩展，非生态扩展**：复用既有 schema（shadcn 本为 React 系，语义对齐
   现成）；**时机 = GOAL-007（Vue/VM 双端 parity）收口之后**——主臂不稳不开新臂。
4. **Svelte 观察**，不预投资。
5. **自研浏览器运行时不做**（VM→WASM 之类）：失去宿主框架生态的免费午餐（npm 组件、
   调试、包体/启动）；保留为远期"VM 第三发射域"选项，硬需求出现再立项。

## 1. 现状锚点

- `ui_gen/vue.rs` + packages/ 四个 JS 包（@auto-ui/widgets、forge-ui、lab-ui、
  playground-vue）；GOAL-012（a2ts/a2js 完善、第二组件库、React/Svelte、responsive）
  规划中。

## 2. 待扩充

- [ ] React 臂立项前提、范围与 schema 复用方案（成本估算）
- [ ] a2ts 与 a2ark（ArkTS 为 TS 方言）的机制共享清单——Web 轨产能反哺鸿蒙轨
- [ ] 第二组件库选型（GOAL-012 既有条目的裁决）
- [ ] Responsive Layout 方案
- [ ] VM→WASM"第三发射域"选项的触发条件与代价评估（远期，不立项）
