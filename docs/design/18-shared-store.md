# 18 — Shared Store (Rung 4: cross-widget/cross-route state)

> **状态**:设计文档(草案)
> **日期**:2026-07-02
> **战略文档**:[Design 16](16-app-generation-and-ai-authoring.md) Rung 4
> **关联**:Plan 345(`k1-shared-store-routing` 金丝雀)、015/025-notes、路由(Plan 105 `routes{}`/`outlet`)、blocks(design 17)
> **目的**:定义 Auto 的**跨 widget / 跨路由共享状态**机制,解锁多页 app(025 当前最大阻塞:无后端时跨页共享状态)。

---

## 1. 背景与缺口

Auto 的状态目前是**每 widget 私有**的:`widget` 里的 `model {}` → 生成 Vue `ref()`s,只在该 widget 内可见。015-notes 靠**后端作 shared source of truth**(每页 `use back.api: ...` 重新拉数据)掩盖了这个缺口 —— `auto-musk` 同理。

但**纯前端 / 无后端**(如 025-notes-extended)或**前端缓存层**(避免每页重复 fetch)需要**跨 widget / 跨路由共享的客户端状态**。这正是 025 单视图绕过路由的根本原因:路由页之间无法共享 `notes`。

> **现状核查**:仓库内**无**任何 SharedStore / global-state 设计(grep `sharedstore|pinia|defineStore|global state` 在 crates/docs/examples 均无命中)。本设计从零起草。

## 2. 生态调研(Vue / React)

### 2.1 Vue 生态

| 方案 | 形态 | 优点 | 缺点 |
|---|---|---|---|
| **Pinia**(官方,Vuex 后继) | `defineStore` → 模块级单例(state+getters+actions),`useXxxStore()` | Vue 3 正道、TS、devtools、**跨路由存活**(是模块) | 外部依赖、对简单状态偏重 |
| **provide / inject**(内置) | 祖先 `provide` reactive 对象,后代 `inject` | 零依赖、原生、树作用域 | 限 provider 子树、provider 卸载即重置、TS 弱 |
| **composable singleton** | 函数返回**模块级** `ref`,按 import 共享 | 最小、无库、天然单例语义 | 靠约定、无 devtools |

### 2.2 React 生态

| 方案 | 备注 |
|---|---|
| **Context** | `createContext`+Provider+`useContext` —— React 版 provide/inject。**坑**:change 重渲所有 consumer |
| **Redux** | 全局单 store,action/reducer/selector,重 |
| **Zustand** | 模块级极简全局 store(hook),当代流行(比 Redux 轻) —— 理念近 Pinia |
| **Jotai/Recoil** | 原子化、自底向上 |
| **React Query/SWR** | **服务端状态**(fetch/cache/sync),独立问题 |

### 2.3 架构轴(三选一,正交)

1. **单例(模块级)** —— Pinia / Zustand / composable-singleton。store 活在组件树**之外**,按 import 共享,**跨路由存活**。**最适合 app-global UI 状态。**
2. **树作用域(provider)** —— provide/inject / Context。祖先 provide,后代消费。**适合 theme/session/scoped 状态;provider 卸载即重置。**
3. **服务端状态** —— React Query / SWR / Pinia 数据插件。cache/sync/invalidate。**另一个问题**(归 Design 16 Rung 2 类型化后端契约)。

## 3. 核心决策

> **Auto 的 SharedStore = 模块级 composable 单例(Pinia 风格),无硬依赖。**

理由:
- 单例**正是**"跨路由共享状态"所需 —— 解决 025 路由阻塞(页间共享 store,不靠后端、不靠 props)。
- **复用现有 `widget` 的 `state`/`msg`/`on` 形态** —— 一个 `store` ≈ "没有 view 的 widget",在语法与 codegen 上与每-widget `model` 高度同构(只是模块级、无 props)。
- **不强制 Pinia** —— 生成一个 `useXxxStore()` composable 返回模块级 `ref`s。契合 Auto 的"最小依赖"取向(如 `@auto-ui/widgets` 不引 Tailwind)。若项目已用 Pinia,可后加一个"Pinia 模式"codegen(非 v1)。
- **provide/inject 留作第二机制**(theme/session scoped 状态) —— 与 store 正交,后续。

## 4. `store` 声明语法

```auto
store NotesStore {
    state {
        var notes []Note = []
        var active_id int = 0
        var loading bool = false
        var error str = ""
    }

    msg Msg { Load; Add(Note); Select(int); ClearError }

    on {
        .Load -> {
            .loading = true
            .error = ""
            // 可调 back.api(有后端时)或内存操作
            .loading = false
        }
        .Add(n) -> { .notes.push(n) }
        .Select(i) -> { .active_id = i }
        .ClearError -> { .error = "" }
    }
}
```

- `store` 是**新顶层声明**(与 `widget`、`type`、`fn` 同级),仅 `scene: ui` 下为上下文关键字(同 `widget` 的处理)。
- 形态 = `widget` 去掉 `view`/`routes`/props,只留 `state` + `msg` + `on`(+ 可选 `lifecycle`,如 `.Init`)。
- 一个文件可声明多个 store;通常放 `src/front/stores/<name>.at`。

## 5. Codegen(composable 单例)

每个 `store FooBar` 生成一个 `src/stores/useFooBarStore.ts`:

```ts
// generated from src/front/stores/notes_store.at
import { ref } from 'vue'
import type { Note } from '@/types'

const notes = ref<Note[]>([])
const active_id = ref<number>(0)
const loading = ref<boolean>(false)
const error = ref<string>('')

export function useNotesStoreStore() {
    return {
        // state (readonly refs exposed)
        notes,            // 消费者读: store.notes
        active_id,
        loading,
        error,
        // actions (msg handlers as functions)
        Load: () => { loading.value = true; error.value = ''; loading.value = false },
        Add: (n: Note) => { notes.value.push(n) },
        Select: (i: number) => { active_id.value = i },
        ClearError: () => { error.value = '' },
    }
}
```

**关键点**:
- `ref`s 是**模块级**(在 `useXxxStoreStore` 函数外声明)→ 真·单例,跨路由存活。
- handler 名 = msg 名(`Load`/`Add`/...)。带参 msg → 带参 fn。
- `state` 里的初始化表达式 = ref 初值。
- 消费者**读**用 `store.notes`(模板里 Vue 自动解包);**写**经 action(`store.Select(i)`),不直接改(契约清晰)。codegen 可加 `readonly` 包装保护。

## 6. 消费:`use store:`

widget/page `use store: <Name>` 声明依赖;codegen 在该组件里 `const store = useXxxStoreStore()` 并把 `store.` 前缀的表达式正确解包(模板内 `store.notes` 直接用;script 内 `store.notes.value`)。

```auto
widget NotesListPage {
    use store: NotesStore
    view {
        col {
            if .store.loading { text "Loading…" { } }
            for note in .store.notes {
                button note.title { onclick: .store.Select(note.id) }
            }
            style: "..."
        }
    }
}
```

- `use store:` 类似 `use back.api:` —— 一个 import 声明,codegen 生成对应 import + `const store = use…()`。
- 多个 store 可同时 `use`(逗号分隔)。
- 模板里 `store.xxx` 由 Vue 自动解包 ref;script/handler 里 codegen 发射 `store.xxx.value`(复用现有 `.value` 逻辑,扩展到 `store.` 前缀)。

## 7. 与路由的交互(解锁多页)

```
widget App {
    routes { "/" -> list; "/note/:id" -> detail; "/archive" -> archive }
    view { outlet }
}
widget ListPage { use store: NotesStore; view { for n in .store.notes {...} } }
widget DetailPage { use store: NotesStore; view { ... .store.notes[.store.active_id] ... } }
```

- 路由页**不再需要 props** —— 共享状态经 store。
- `"/note/:id"` 的 `:id` 经路由参数取到后,页 handler 调 `store.Select(id)` 选笔记 → 详情页读 `store.active_id`。
- 这**正是** 025 路由阻塞的解法:无后端、跨页共享 `notes`/`active_id`。

## 8. 与既有概念的关系

| 概念 | 关系 |
|---|---|
| **每-widget `model`** | 私有局部状态(UI 局部,如编辑中的 `edit_title`)。`store` 是**共享**的;`model` 是**私有**的。两者并存。 |
| **`widget` 的 props** | 父→子单向传值(仍是首选)。`store` 是**任意位置共享**,不限于父子。 |
| **blocks(design 17)** | block 的 `dataSource` 可指向 store action(`store.Load`)或读 store state —— block 与 app 的数据经 store 解耦。 |
| **Rung 2 类型化后端契约** | 服务端状态( fetched data )经 `#[api]` 进 store action:`store.Load -> { .notes = list_notes() }`。store 存 UI + 缓存层;`#[api]` 是源头。两者分层。 |
| **provide/inject** | 第二机制(theme/session scoped),与 store 正交,后续。 |

## 9. 边界与非目标(v1)

- **不做** Pinia 硬绑定(v1 用 composable 单例;Pinia 模式可选后续)。
- **不做** getters/computed(v1;state 直读。可后续加 `computed {}`)。
- **不做** 持久化(localStorage/reload 存活)—— v1 纯内存;持久化另立。
- **不做** devtools 集成(composable 单例天然无;Pinia 模式才有)。
- **不做** SSR 同构(v1 纯 CSR)。
- store 的**模块级单例**在测试里有跨用例污染风险 —— 文档提示,v1 不处理。

## 10. 实施阶段(建议)

1. **金丝雀 `capability-tests/k1-shared-store-routing/`**(初始红):
   - 两个路由页经一个 `store` 共享一个计数器/列表(`store` 声明 + 两页 `use store:` + 一页改、另一页读)。
   - 验证:`auto build` + `vue-tsc` 绿;切路由后状态保留。
2. **Parser**:`store` 为 `scene: ui` 上下文关键字;新 `StoreDecl` AST 节点(复用 `state`/`msg`/`on` 解析)。
3. **AURA extraction**:提取 `AuraStore { name, state_vars, messages, handlers }`(与 AuraWidget 同构,去 view/routes/props)。
4. **Codegen**:生成 `useXxxStore.ts`(模块级 ref + actions);`use store:` → 消费组件生成 import + `const store = use…()` + `store.` 表达式的 `.value` 处理。
5. **回归**:015(后端作 shared source,不受影响)、025(可拆多页了)、blocks(design 17)、canaries 全绿。
6. **文档**:creator skill + blocks-gallery 的 dataSource 约定接 store。

## 11. 开放问题

- **store 之间依赖**(store A 用 store B):v1 禁止或仅允许单向?—— v1 禁止,避免循环。
- **响应式粒度**:`for note in .store.notes` 的列表项更新是否能 fine-grain(非整表重渲)?—— Vue reactivity 天然处理,v1 不额外优化。
- **类型导出**:store state 的 TS 类型如何导出给消费者(供 `store.notes: Note[]`)?—— 从 `state` 注解生成 `interface`,与 `types.at` 一致。
- **命名**:`store NotesStore` → `useNotesStoreStore()`(双 Store 啰嗦)?—— 约定 `store Notes` → `useNotesStore()`,或 codegen 去重后缀。

---

## 附录:与 Jade Garden / auto-down 的参照

jade-garden(AutoUI app)目前用**手写 composable**(`useSyncedScroll` 等)+ 组件局部 state 管理编辑器状态。若 store 落地,jade-garden 的跨 tab 状态(打开的文档、当前 workspace)可迁到 `store`,减少手写 composable。这间接验证 store 的真实需求。
