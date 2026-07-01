# 025 Gap Enumeration (lightweight) — Design 16 appendix

> **日期**:2026-06-30
> **来源**:从 `examples/ui/025-notes-extended/SPEC.md` 出发,用一组**探针 widget**逐个试探 AI 会自然写出的不确定构造,`auto build` → 捕获失败。
> **目的**:为"金丝雀测试 + 差距特性实现"提供**完整的差距清单**(已知 + 非obvious),避免只修已知的 3 个而漏掉其余。
> **方法学注**:这是**轻量枚举**(一次构建、探针法),**不是** Design 16 的正式 gap 分析(后者是 AI 全量再生 + 量修复轮次 N)。正式 gap 分析留到明显差距补齐后再做,以量长尾。

## 已知差距(构建 025 时直接撞到)

| # | 差距 | 表现 |
|---|---|---|
| K1 | **路由 / 跨页共享状态(Rung 4)** | `"/p" -> use <widget>` 不传 prop;无后端时跨页共享状态需共享 store(未建)。025 改单视图规避。 |
| K2 | **子→父 handler 绑定** | 自定义 widget 是 presentational(props in,无事件出);App 无法响应子 widget 的 msg。block 采纳因此受限。 |
| K3 | **Markdown / JS 互操作** | 无 markdown/JS stdlib bridge;`markdown .body` 这类无对应 codegen。 |

## 非 obvious 差距(本次枚举新发现)

| # | 差距 | 探针失败 | 影响 |
|---|---|---|---|
| N1 | **`.contains` codegen bug** | `vue-tsc: Property 'contains' does not exist on type 'any[]'`(列表);字符串同理(JS 也没有 `.contains`)。codegen 发射 `.contains()` 而非 `.includes()`。 | **025 自身潜在受影响**:025 用了 `note.title.contains` / `note.tags.contains`,虽 `auto build` 报"成功",但生成的 Vue 不过 vue-tsc。任何用 `.contains` 的 app 都中招。 |
| N2 | **路由 codegen 路径 bug** | `router/index.ts: Cannot find module '@/pages/probe_home.vue'`。路由配置引用的 page 模块不在该路径生成。 | 路由**不只是**缺共享状态(K1);**codegen 本身**就坏的。即便有共享 store,路由也得先修此。 |
| N3 | **handler 内无局部可变变量** | `var i int = 0; for n in .ns { i = i + 1 }` → `Parse error: Expected term, got RBrace`。 | AI 自然会写带计数器的循环(按 id 找索引、累加等);现需改用 state 字段(`.i`)绕过。语言层缺口。 |
| N4 | **callback-prop 绑定失败** | 子 widget `widget ProbeChild(items, on_select: msg)` 内调 `on_select(i)` → `Cannot find name 'on_select'`。 | K2 的具体形态:连"传一个回调 prop"这种最朴素的父子通讯都不工作。 |

## 优先级建议(给金丝雀计划)

按 **杠杆 × 范围** 排序:

1. **N1(`.contains` → `.includes` codegen 修)** —— 纯 codegen bug,范围小,**025 立即受益**(它已 latent 中招)。最高 ROI。
2. **K2 / N4(父子 handler 绑定)** —— 杠杆最高(解锁 block 采纳 + app 组合);范围中等(codegen/AST)。callback-prop 是最小可用形态。
3. **N2 + K1(路由)** —— 路由先修 codegen 路径 bug(N2),再补共享状态(K1,Rung 4)。范围较大。
4. **N3(handler 局部可变变量)** —— 语言层;范围中。可与 K2 同期。
5. **K3(markdown/JS-interop)** —— 需新 stdlib bridge;范围最大,最后做。

## 输出

本清单直接驱动下一计划:**gap 金丝雀测试**(每差距一个最小 Auto UI 工程,初始红、特性落地后绿),按上述优先级逐个实现。

## 后续发现(2026-06-30,K2 实现期)

**OOM:callback + 计算型 msg 实参**:子 widget 同时(1)收到 callback prop、(2)在事件绑定里用计算型 msg 实参(如 `onclick: .Bump(.n + 1)`)时,codegen 触发 51GB 内存分配( runaway)。单独 callback 或单独计算实参均不触发;是 callback codegen 与 Plan 339(`AuraExpr::If`)的交互。canary 用字面实参 `.Bump(1)` 绕过(演示 callback 通);此 OOM 单独跟踪为 codegen bug。
