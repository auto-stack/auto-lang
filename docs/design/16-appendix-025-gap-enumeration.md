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

## N2 复核(2026-07-02):实为"约定未文档化",非 codegen bug

N2 经 canary 复核:**路由 codegen 本身正确**。原探针把 page widget 内联在 app.at,导致 page 未生成、router 的 `@/pages/<name>.vue` import 悬空。约定是:路由目标页必须放 `src/front/pages/<name>.at`,`cmd_vue.rs` 才会生成到 `src/pages/<name>.vue`(与 router import 路径对齐)。canary 按约定写即绿。归档为"文档/约定"项,非语言/codegen 缺口。

## N3 复核(2026-07-02):已在 master 实现

N3(handler 内局部可变变量)经 canary 复核:**已工作**。codegen 正确发射 `let i = 0`、裸局部赋值 `i = i + 1`(无 `.value`),state 赋值仍用 `.value`。原 gap 枚举基于较旧 base;master 后续提交已补此能力。canary 作回归测试钉住。非缺口。

## OOM 根因与修复(2026-07-02)

**根因**:`parser.rs::parse_event_handler` 的实参收集循环只处理 `,`/`;` 分隔;而 `parse_event_arg` 只消费 Dot/Ident/Int/Str,遇运算符(`+` 等)即 break。故事件实参里的二元表达式(如 `.Bump(.n + 1)`、`.Bump(1 + 1)`)只消费第一个操作数,留下运算符 token;caller 循环无法处理该 token,无限 push 空串 → ~48GiB OOM。

**修复**:`parse_event_arg` 的循环里增加对二元运算符(`+ - * / % == != < > <= >= && ||`)的消费(push `" op "` 并 continue)。`examples/capability-tests/oom-event-binop-arg/` 作回归(原 RED,现 GREEN)。

**遗留(独立)**:state-ref 事件实参(`.n`)codegen 为 `this.n`(Vue 下应为 `n`)—— `.Bump(.n)` 不带 binop 也有此问题,与 OOM 无关,另列。
