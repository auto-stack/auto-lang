# Plan 345: Gap 金丝雀测试 —— 逐差距 TDD 实现 AutoUI 平台特性

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-30
> **战略文档**:[Design 16](../design/16-app-generation-and-ai-authoring.md)、[025 差距枚举](../design/16-appendix-025-gap-enumeration.md)
> **前置**:025-notes-extended(SPEC + 已知差距)、差距枚举(本文依据)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

为 025 差距枚举里的**每个差距**建一个**最小金丝雀工程**(纯前端 Auto UI app,聚焦单一差距),初始**红**(`auto build`/vue-tsc 失败或不达预期),把对应平台特性实现到**绿**(金丝雀通过)。金丝雀既是 TDD 规格,又是该特性的"怎么用"示例(回流 `/auto-lang-creator` skill),还是回归测试。

**不是**重新扩展 025/015;而是**先把平台特性补齐**,再回头让 025/015 受益。

## 2. 金丝雀载体

新建 `examples/capability-tests/<gap-id>-<slug>/`,每个一个 `pac.at` + 最小 `src/front/app.at`(+ 必要子 widget)。命名示例:

```
examples/capability-tests/
  n1-contains-includes/        # .contains → .includes codegen
  k2-child-handler-binding/    # 父子 handler 通讯(callback-prop 最小形态)
  n2-routing-codegen-paths/    # 路由 page-module 路径修复
  k1-shared-store-routing/     # Rung-4 共享 store + 跨页路由
  n3-handler-local-vars/       # handler 内局部可变变量
  k3-markdown-interop/         # markdown/JS stdlib bridge
```

每个金丝雀 README 写明:探针构造、预期行为、当前红/绿状态、对应特性 commit。

## 3. 优先级与阶段(按 枪杆 × 范围,见枚举 §优先级)

### Phase 1 — N1:`.contains` → `.includes` codegen 修(最高 ROI)✅ DONE(2026-06-30)
- **金丝雀** `n1-contains-includes/`:字符串 `.contains` + 列表 `.contains` 各一例;`auto build` + vue-tsc 须过。
- **修**:codegen 发射 `.includes(...)`(字符串与列表均然;或在 AST 层把 `.contains` 语义映射到 JS `.includes`)。
- **附带受益**:025 的 latent vue-tsc 失败随之消失(回归验证)。
- **Commit**:`fix(codegen): emit .includes for str/list .contains (gap N1)`。

### Phase 2 — K2/N4:父子 handler 绑定 ✅ DONE(2026-06-30)(callback-prop 最小形态)
- **金丝雀** `k2-child-handler-binding/`:父 widget 传 `on_select: .Selected` 给子;子内 `onclick: .Pick(id)` → 调 `on_select(id)` → 父 `.Selected(id)` 改 state。
- **修**:codegen 支持"callback 类型 prop";子在 handler 里调用该 prop = 发射对父 handler 的调用(经 props 透传)。最小可用形态:callback-prop(不做完整 event-bubbling DSL)。
- **Commit**:`feat(ui): child→parent handler binding via callback props (gap K2/N4)`。

### Phase 3 — N2:路由 codegen page-module 路径 ✅ DONE(2026-07-02,实为约定非 bug)
- **金丝雀** `n2-routing-codegen-paths/`:`routes { "/" -> use home; "/x" -> use xpage }`;生成的 `router/index.ts` 引用的 page 模块必须存在。
- **修**:路由 codegen 把 page widget 生成到 `router/index.ts` 所引用的路径(对齐 `@/pages/<name>.vue` 或修正引用)。
- **Commit**:`fix(codegen): route page-module paths exist (gap N2)`。

### Phase 4 — K1:Rung-4 共享 store + 跨页路由
- **金丝雀** `k1-shared-store-routing/`:两页共享一份 notes state(一页列表、一页详情);经 store 读写。
- **修**:引入最小 store 概念(全局/模块级 reactive state),路由页读 store;`routes{}` 页可消费 store。
- **范围较大**;可单列子计划。**Commit**:`feat(ui): shared store for cross-route state (Rung 4, gap K1)`。

### Phase 5 — N3:handler 内局部可变变量 ✅ DONE(2026-07-02,已在 master 实现)
- **金丝雀** `n3-handler-local-vars/`:`var i = 0; for n in ns { .sum += n; i += 1 }` 须解析通过。
- **修**:parser/AST 支持 handler 块内 `var` 局部声明 + 赋值(不进 state)。
- **Commit**:`feat(lang): local mutable vars in handler blocks (gap N3)`。

### Phase 6 — K3:markdown / JS-interop bridge(最大,最后)⏸ DEFERRED(独立子计划)
- **金丝雀** `k3-markdown-interop/`:`markdown .body` 渲染 marked 输出。
- **修**:最小 JS-interop bridge(`extern`/FFI 到 JS fn,如 `marked`);codegen 发射对应调用。
- **本计划不实施**:需新的 JS-interop/FFI 基建(范围最大),按本计划原始排序"最大,最后"独立成子计划。当前 pass 完成 N1/K2/N4/N2/N3;K3 + 已跟踪的 OOM(callback+计算型实参)单列后续。

---

## 4. 验证策略

每个金丝雀:`auto build` 绿 **且** 生成 Vue 经 `vue-tsc --noEmit` 过(025 的教训:仅 `auto build` 成功 ≠ 类型正确)。Phase 1 完成后,重跑 025 的 `auto build` + vue-tsc 作回归(应转绿)。

## 5. Definition of Done

- [ ] `examples/capability-tests/` 下 6 个金丝雀工程(或按实际合并),每个有 pac.at + app.at + README。
- [ ] N1、K2/N4、N2、N3 的特性落地,对应金丝雀 `auto build` + vue-tsc 绿。
- [ ] 025 回归:Phase 1 后 025 的 vue-tsc 转绿(N1 修复)。
- [ ] K1、K3 范围大,可各自单列子计划(本 Plan 标注"延后/单列")。
- [ ] 每个金丝雀 README 回流 `/auto-lang-creator` skill(特性→示例→文档闭环)。
- [ ] worktree 分支在绿后合并回 `master`。

## 6. 后续

- 金丝雀全绿后,回头**扩展 025**(路由、block 采纳、markdown 现在可做)→ 再 M-merge 进 015。
- 正式 Design-16 gap 分析(AI 全量再生 + 量修复轮次 N)在明显差距补齐后做,捕获长尾。
