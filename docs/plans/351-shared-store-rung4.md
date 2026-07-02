# Plan 351: Shared Store (Rung 4) — 跨 widget/跨路由共享状态

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-07-02
> **战略文档**:[docs/design/18-shared-store.md](../design/18-shared-store.md)(Rung 4)
> **关联**:Plan 345(`k1-shared-store-routing` 金丝雀,RED 待实现)、015/025-notes(路由阻塞)、blocks(design 17)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

落地 Design 18 的 SharedStore:**模块级 composable 单例**(Pinia 风格,无硬依赖)。解锁 AutoUI 的跨 widget/跨路由客户端状态 —— 这是 025 多页路由的直接阻塞、Design 16 Rung 4。

完成后:
- `store` 顶层声明可用(`state` + `msg` + `on`,与 view-less widget 同构)。
- 生成 `useXxxStore()` composable(模块级 `ref`s + action 函数),无 Pinia 依赖。
- widget/page 经 `use store: <Name>` 消费,跨路由共享。
- `k1-shared-store-routing` 金丝雀(两路由页经 store 共享状态)从 RED 转 GREEN。

## 2. 关键决策(来自 Design 18)

| 决策点 | 结论 | 理由 |
|---|---|---|
| **形态** | 模块级 composable 单例 | 跨路由存活;最小依赖;复用 widget 的 state/msg/on |
| **codegen** | `useXxxStore.ts`(模块级 ref + actions) | 不强制 Pinia;契合 Auto 最小依赖取向 |
| **语法** | `store Name { state{} msg{} on{} }` | 与 widget 同构(去 view/routes/props) |
| **消费** | `use store: <Name>` | 类 `use back.api:` 风格 |
| **与 model** | 并存(model 私有,store 共享) | 分层:局部 UI 状态 vs 跨页共享 |
| **服务端状态** | 不混入;经 store action 接 `#[api]` | Rung 2 类型化契约分层 |

## 3. 非目标(v1,留给后续)

- Pinia 硬绑定(v1 用 composable 单例)。
- `computed`/getters。
- 持久化(localStorage/SSR)。
- devtools 集成。
- store 间依赖(v1 禁止,避免循环)。

---

## 实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 parser/codegen 后 `cargo build -p auto`;UI 验证用 `auto build` + **vue-tsc**(Plan 345 教训:build 成功 ≠ 类型正确)。
>
> **Goal:** `store` 声明 → composable 单例 codegen → `use store:` 消费 → 金丝雀(跨路由共享状态)GREEN。
>
> **Architecture:** `store` AST(复用 state/msg/on)→ AURA `AuraStore` → `useXxxStore.ts`(模块级 ref + actions);widget `use store:` 生成 import + `const store = use…()` + `store.` 表达式 `.value`。

## Pre-flight: Worktree

```bash
git worktree add -b plan-351/shared-store ../auto-lang-351
cd ../auto-lang-351
```

---

## Phase 1 — 金丝雀(RED)

**目标**:写 `capability-tests/k1-shared-store-routing/` —— 两路由页经一个 `store` 共享一个计数器/列表。

**Files:** `examples/capability-tests/k1-shared-store-routing/{pac.at, src/front/app.at, src/front/stores/counter.at, src/front/pages/{home,settings}.at, README.md, .gitignore}`

**Step 1:** `counter.at` 声明 `store CounterStore { state { var count int = 0 } msg Msg { Inc; Dec } on { .Inc -> { .count = .count + 1 } .Dec -> { .count = .count - 1 } } }`。
**Step 2:** `app.at` = 路由 shell(`routes { "/" -> home; "/settings" -> settings }` + `outlet`)。
**Step 3:** `home.at` 页 `use store: CounterStore`,显示 `store.count` + 增/减按钮。`settings.at` 页也 `use store: CounterStore`,显示同一 `store.count`(验证跨页共享 + 路由切换后值保留)。
**Step 4:** `auto build` → 预期 RED(`store` 未实现,解析失败)。
**Commit:** `test(cap-k1): shared-store-routing canary (RED)`。

---

## Phase 2 — Parser:`store` 声明

**Files:** `crates/auto-lang/src/parser.rs`、`crates/auto-lang/src/ast.rs`

**Step 1:** `store` 为 `scene: ui` 上下文关键字(同 `widget` 的处理,parser.rs:418 的 contextual list)。
**Step 2:** 新 AST 节点 `StoreDecl { name, state(ModelBlock), messages(Vec<MsgDecl>), on(OnBlock) }`(复用现有 ModelBlock/MsgDecl/OnBlock 类型,不新造)。
**Step 3:** `parse_store_decl()` —— 解析 `store Name { state{} msg{} on{} }`(复用 parse_model/parse_msg_decl/parse_on_block)。
**Step 4:** 顶层 stmt 收集:在 parse 循环里识别 `store` → `parse_store_decl()` → `Stmt::StoreDecl`。
**Step 5:** 单测:`store Foo { state { var x int = 0 } msg Msg { Inc } on { .Inc -> { .x = .x + 1 } } }` 解析成 StoreDecl,字段齐全。
**Commit:** `feat(parser): 'store' declaration (scene:ui contextual keyword)`。

---

## Phase 3 — AURA extraction

**Files:** `crates/auto-lang/src/aura/types.rs`(新 `AuraStore`)、`crates/auto-lang/src/aura/extract.rs`(提取 fn)

**Step 1:** `AuraStore { name, state_vars: Vec<AuraStateDef>, messages: Vec<AuraMessage>, handlers: HashMap<String, LogicPayload> }`(与 AuraWidget 同构,去 view/routes/props/lifecycle 暂留)。
**Step 2:** `extract_store_from_decl(decl: &StoreDecl) -> ExtractResult<AuraStore>` —— 复用 extract_model_fields / extract_msg_decl / handler 提取逻辑。
**Step 3:** 在 `ui_build_shadcn_with_widgets` / `ui_build_shadcn_with_sub_widgets` 里收集 `Stmt::StoreDecl` → `Vec<AuraStore>`,随 widgets 一并返回(或经一个新字段/独立返回)。
**Step 4:** 单测:StoreDecl → AuraStore,字段映射正确。
**Commit:** `feat(aura): extract AuraStore (state+msg+handlers)`。

---

## Phase 4 — Codegen:`useXxxStore.ts` 生成

**Files:** `crates/auto-man/src/vue.rs`(或 cmd_vue)+ `crates/auto-lang/src/ui_gen/vue.rs`(store→composable 生成器)

**Step 1:** `VueGenerator::generate_store_composable(store: &AuraStore) -> String` —— 生成:
```ts
import { ref } from 'vue'
const <var> = ref<<type>>(<init>)   // 每个 state var 一个模块级 ref
export function use<Name>Store() {
    return {
        <var>,                        // state refs
        <MsgName>: (<params>) => { <handler body> },  // actions
        ...
    }
}
```
- handler body 复用 `transpile_handler_body`(ts_adapter);store state 在 handler 里是模块级 ref → 用 `.value`(handler body 已对 state ref 发射 `.value`)。
- action 名 = msg 名;带参 msg → 带参 fn(参数名从 handler 的 `.Foo(p)` 取,同 widget handler 参数名处理)。

**Step 2:** cmd_vue(或 auto-man vue.rs `regenerate_source_files` / generate)把每个 store 写到 `gen/front/vue/src/stores/use<Name>Store.ts`。

**Step 3:** 金丝雀 build:`counter.at` → `useCounterStoreStore.ts` 产出,内容正确(模块级 ref + Inc/Dec actions)。
**Commit:** `feat(codegen): generate useXxxStore composable (module-level singleton)`。

---

## Phase 5 — 消费:`use store:` 在 widget/page

**Files:** `crates/auto-lang/src/parser.rs`(use 解析)、`aura/extract.rs`、`ui_gen/vue.rs`

**Step 1:** `use store: CounterStore` 解析(类似 `use back.api:` / `use sidebar:`),记录 widget 依赖的 store 名。
**Step 2:** AuraWidget 带 `used_stores: Vec<String>` 字段;codegen 见到 `use store: X` → 在组件 script 顶部 `import { useXStore } from '@/stores/useXStore'` + `const store = useXStore()`。
**Step 3:** 表达式:`store.notes` → 模板内 Vue 自动解包(`store.notes`);script/handler 内 → `store.notes.value`。扩展 expr codegen:`store.<field>` 走与 state ref 同样的 `.value` 规则(在 handler 里),模板里不解包。
**Step 4:** action 调用:`onclick: .store.Select(id)` → `@click="store.Select(id)"`(模板内直接调函数;store.Select 是 action)。
**Step 5:** 金丝雀 build + vue-tsc:`home`/`settings` 两页都 `use store: CounterStore`,读 `store.count`、调 `store.Inc`/`store.Dec`。**GREEN**。
**Commit:** `feat(codegen): 'use store:' consumption + store. expression handling`。

---

## Phase 6 — 路由集成验证 + 回归

**Step 1:** 金丝雀 `pnpm dev` / vite build:两路由页共享 `store.count`,在一页 Inc 后切到另一页值保留(**人工/构建验证**;runtime 视觉由用户确认)。
**Step 2:** 回归:
   - 015-notes(后端作 shared source,不受影响)build + vue-tsc(忽略其既有 comms WIP 错误,仅确认 store 改动未新增错误)。
   - 025-notes-extended build + vue-tsc 绿(可后续迁多页)。
   - capability-tests 全 canary(N1/K2/N2/N3/OOM)仍绿。
   - block tests 7/7。
**Step 3:** 更新 README/状态:`k1-shared-store-routing` GREEN。
**Commit:** `test(cap-k1): shared-store-routing GREEN + regression`。

---

## Definition of Done

- [ ] `store Name { state{} msg{} on{} }` 可声明(parser + AST + 单测)。
- [ ] AURA 提取 AuraStore(state + msg + handlers)。
- [ ] 生成 `use<Name>Store.ts`(模块级 ref + actions),无 Pinia 依赖。
- [ ] widget/page 经 `use store:` 消费;`store.<field>` 表达式在模板/脚本里正确(解包/`.value`)。
- [ ] 金丝雀 `k1-shared-store-routing` GREEN(auto build + vue-tsc + 两路由共享状态)。
- [ ] 回归:015/025 不新增错误;capability-tests canary 全绿;block tests 绿。
- [ ] `/auto-lang-creator` skill + blocks-gallery 的 dataSource 约定接 store(文档)。
- [ ] worktree 分支在 build + 测试绿后合并回 `master`。

## 后续(不在本 Plan)

- Pinia 模式 codegen(可选)、`computed`、持久化、devtools、store 间依赖。
- 把 025 拆成多页路由(M-merge 进 015 之前;025 自身可先多页化验证 store)。
- provide/inject 第二机制(theme/session scoped)。

---

## 风险

| 项 | 风险 | 缓解 |
|---|---|---|
| `store.` 表达式 `.value` 处理 | 与现有 state ref `.value` 逻辑可能冲突 | 复用 expr_to_js 的 state_names 机制,把 store 字段纳入"需 .value"集合 |
| handler body 在模块级 ref 上 | ts_adapter 当前按 widget 上下文生成;store 无 widget 上下文 | 给 store 生成单独的 ctx(state_names = store 字段),复用 transpile_handler_body |
| 路由 + store 集成 | 页面组件挂载/卸载与 store 单例生命周期 | store 是模块级,不受组件生命周期影响(单例天然存活)—— 验证即可 |
| `use store` 命名 | `store CounterStore` → `useCounterStoreStore`(双 Store 啰嗦) | 约定 `store Notes` → `useNotesStore()`,或 codegen 去重后缀(开放问题,见 design 18 §11) |
