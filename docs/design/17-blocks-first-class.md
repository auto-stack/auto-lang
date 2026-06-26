# 17 — Blocks as a First-Class Tier

> **状态**:设计文档(概念设计)
> **日期**:2026-06-26
> **关联**:Design [16 App Generation](16-app-generation-and-ai-authoring.md)、widget 层(Plan 331/336/337)、`examples/a2ui-composer`、`examples/a3ui-replica`
> **目的**:把 **block** 提升为 widget / block / app 三层里的"一等公民",并回答核心难题——**如何定义"常见、又需要灵活订制"的 block 的范围与边界**。

---

## 1. 背景:三层里的"中层"缺位

| 层 | 粒度 | 当前 Auto 的"身份" | 缺口 |
|---|---|---|---|
| **Widget** | 原语(Button/Input/Card) | ✅ 真值源:`WidgetRegistry` + `@auto-ui/widgets` + `examples/vue-gallery` | 已立 |
| **Block** | 组合卡(LoginForm/DataTable/KanbanColumn/StatsBoard) | ❌ 散落在 `examples/a2ui-composer`、`a3ui-replica`、`examples/ui/001-014`;无 registry、无规范、无身份 | **本文档** |
| **App** | 完整产品(路由/shell/后端契约) | ✅ 基准阶梯(Design 16,M1-M6) | 已立 |

中层缺位的代价:在 app 生成(Design 16)时,**缺一个中间粒度的复用目标**。AI 只能在 widget(太细,组合空间爆炸)与 app(太粗,一次性生成整体)之间二选一。block 正是连接两层的榫卯:**app = shell + 路由→block 选择 + block 数据接线;block = widgets + 局部状态 + 数据源**。

## 2. 核心张力:common yet flexible

- **Widget**:紧契约(props/variants),低自由 → 可**黑盒复用**。
- **Block**:有"可辨识形状"(DataTable/LoginForm 一眼认出是哪类模式),但每个 app 都要改(字段、列、数据源、样式、动作)。
- **App**:独一无二,完全定制。

把 block 当 widget 那样做黑盒 → 配置项爆炸,永远不够用(一个 LoginForm 的 props 会膨胀到 SSO/2FA/魔法链接/校验规则/主题…)。把 block 当纯示例 → 不算复用,只是样板代码。

### 2.1 本设计的回答

> **block 是"被拥有的源码配方(owned source recipe)",不是黑盒组件。**

借鉴 shadcn 的 copy-own 哲学,叠合 Auto 的"单一 `.at` 源"优势:
- 消费者用 `auto block add <name>` 把一份 `.at` 源码拷进项目,**改源码即订制**。
- "扩展点"是源码里**标好的、文档化的编辑区**(注释标记 + 文档说明),**不是 N 个 runtime props**。
- 数据通过一个 **typed `dataSource`** 注入,block 与具体后端解耦。

这天然解决"边界"难题:**边界 = 当你改到超出配方骨架,就完整接管源码(你本就拥有它)**。block 因此是一个**有限集合**(常见模式的骨架),不是无限可配的引擎。这也回避了一个 AURA 当前缺口——通用命名投影(slot projection)尚未一等公民(见 §11 Phase D);owned-source 模式不依赖它即可工作。

## 3. 三层定义(粒度阶梯)

| 维度 | Widget | **Block** | App |
|---|---|---|---|
| 复用单位 | 单个原语 | **组合卡(一个可辨识 UX 模式)** | 完整产品 |
| 自由度 | 低 | **中(在 kind 词汇表内)** | 高 |
| 契约形态 | props/variants | **行为契约 + 扩展点词汇表 + dataSource** | 路由/shell/全局状态 |
| 复用方式 | import / 拷贝(shadcn) | **adopt-and-edit(拥有源码)** | 从基准派生 |
| Registry | `WidgetRegistry` | **`BlockRegistry`(待建)** | 基准阶梯(M1-M6) |
| Gallery | `vue-gallery` | **`blocks-gallery`(待建)** | `examples/ui/*` |
| AI 复用粒度 | 词法 | **句法(模式级复用)** | 篇章 |

## 4. 范围与边界:如何圈住"自由"(用户最关心的问题)

### 4.1 三个边界判据

- **In-scope(一个 block 该承担的)**:一个可辨识 UX 模式 + 一个固定**行为契约** + 一组**有界的扩展点词汇表**(由 block 的 *kind* 决定,见 4.2)。
- **Out-of-scope(交给 app 层组合)**:横切基础设施(auth / routing / theming / 全局 store)、跨多模式的业务工作流。
- **逃生口(eject)= 天花板**:block 不追求覆盖 100%;当订制超出其扩展点词汇表 → 消费者完整接管(已拥有源码)。这是 block 保持"有限"的关键,防止退化为"上帝组件"。

### 4.2 block kind = 限制自由的关键机制

**不泛泛定义"一个 block",而是定义"类别",每类有固定的扩展点词汇表。** 某个 block 实例只能在它所属 kind 的词汇表内变化;超出 → eject。这是把"自由"圈住的核心:

> 例:所有 **DataTable 类** block 暴露相同扩展点词汇表:`columns` / `row` / `cell(:col)` / `empty` / `loading` / `error` / `toolbar` / `pageSize`。
> 一个具体 DataTable 在这套词汇里变化;消费者要"行内嵌视频播放器" → 已超出词汇表 → eject 接管。

词汇表把"可订制"从"无边 props"收敛为"几类固定切面",既保灵活又可枚举、可文档化、可被 AI 当稳定目标。

### 4.3 变体(variants)而非无限 props

一个 block 自带**命名变体**(`compact` / `with-filters` / `dense`),而不是 30 个布尔 prop。变体是预先调好的"已订制档位";超出变体 → 改源码或 eject。

### 4.4 配色/spacing 归 design token,不进 block

block 不内联颜色/间距硬编码,而消费 token(未来的 Design Token Compiler)。这让同一份 block 源码在不同主题下零改动可用 —— 订制走 token,不走 block 内部。

## 5. Auto 中的 block 解剖

一个 block = 一个 `widget`,它:

1. 在 `view` 里**组合 widgets**(来自 `@auto-ui/widgets` 调色板)。
2. 声明 `model`(自身状态)+ `msg`(**行为契约**:加载/失败/选中/提交/分页…)。
3. 接一个 **`dataSource`** 作为 prop(typed fetcher 或 `#[api]` 引用 + 参数)—— **block 与具体后端解耦**(它不知道也不该知道 `/api/notes`)。
4. **扩展点**:Phase 1 = 源码里标好的编辑区(注释标记 + 文档);Phase 2 = AURA 命名 slot(运行时投影,见 §11)。

```auto
// 伪代码:一个 DataTable block 的形态
use widgets: Table, Pagination, EmptyState, ErrorState
use back.api: Note   // 仅类型,不绑具体端点

widget DataTableBlock[T](items []T, columns []Column, on_select msg) {
    msg Msg { PageChanged(int), SortChanged(str), Retry }
    model { var loading bool = true; var error str = ""; var page int = 0 }
    view {
        // ── EDIT: toolbar(扩展点:工具条)──
        // ── EDIT: column header render(扩展点)──
        Table(items, columns) { … }
        // ── EDIT: empty / loading / error(扩展点)──
        Pagination(page) { onchange: .PageChanged }
    }
    on { /* 行为契约:抓取/重试/分页/排序,接 dataSource */ }
}
```

`EDIT:` 注释 = 文档化的编辑区(owned-source 的扩展点形态)。AI/人类在区内改,不改骨架契约。

## 6. 订制模型:Auto 在 config–composition 谱系上的落点

谱系两端:纯 config(一堆 props)↔ 纯 composition(小组件自拼)。

**Auto 的选择:默认 adopt-and-edit(拥有源码),辅以三种订制渠道,按"变化类型"分流:**

| 变化类型 | 订制渠道 | 例子 |
|---|---|---|
| 数据驱动(结构化) | **schema/config**(columns、fields 定义) | 表格列、表单字段集 |
| 结构性(布局) | **slot**(Phase 2 AURA 命名投影) | 自定义 cell 渲染、工具条 |
| 风格 | **design token** | 主题/间距 |
| 超出词汇表 | **eject**(接管源码) | "我要在表格行嵌视频" |

Auto 明确**不做**"配置驱动的万能 block 引擎"(会退化成低代码地狱)。owned-source 是默认;config/slot 是便利;eject 是兜底。

## 7. block kind 分类法(每类 = 一套扩展点词汇表)

| Kind | 职责 | 扩展点词汇表 | 示例 block |
|---|---|---|---|
| **Form** | 结构化输入与提交 | fields schema、submit、validation、actions(第三方登录等槽)、success/error | login、contact、settings、search |
| **Data-display** | 展示集合 + 状态 | columns/items renderer、**dataSource**、empty/loading/error、toolbar、分页/排序/过滤 | data-table、card-grid、stats-board、note-list(→ M1 用) |
| **Feedback** | 瞬时/状态反馈 | content、触发契约 | toast-region、empty-state、error-boundary、confirm-dialog |
| **Layout** | 容器/骨架 | named slots(主体区) | app-shell、sidebar、header、dashboard-grid、tabs-shell |
| **Composite/Action** | 带局部状态机的交互 | item renderer、actions、局部状态机 | kanban-column(→ M2)、comment-thread、chat-message-list(→ M3)、file-uploader |

每类 = 一份"扩展点契约"文档 + gallery 里若干实例。**M1-M3 基准需要的 block 正好覆盖 Data-display / Form / Composite 三类** —— block 生态与 Design 16 基准阶梯天然对齐。

## 8. 数据契约(`dataSource`)

- block **不绑死** `/api/notes`;接 typed fetcher(`dataSource: list_notes` + params)。
- 直接喂给 Design 16 的 **Rung 2(类型化后端契约)**:block 是"前端如何消费 typed API"的标准形态。
- **loading/error/empty 是一等 view 状态**(不是事后补丁)—— Rung 2 的数据生命周期在此落地为 block 的强制槽。这让每个数据型 block 天然带状态机,AI 生成时不会漏掉 loading/error。

## 9. block 的载体:registry + gallery + `auto block`

- **`BlockRegistry`**(`crates/.../ui_gen/block/`):按 `kind + name` 索引;每条含:所属 kind、组合的 widgets、扩展点词汇表、dataSource 期望签名、变体列表、`.at` 源码位置。
- **`examples/blocks-gallery`**:每类若干 block,且**每个 block 给"未订制 / 已订制"两视图对照**(展示边界:默认 vs eject 前能走多远)。
- **`auto block add <kind>/<name>`**:拷 `.at` 源码进消费者项目 + 提示关联 widget 依赖 + 标注 dataSource 接入点(复用 Plan 337 的同步思路)。
- **`auto block list` / `backlog`**:对照 widget registry 与 app 实际用到的 block,看缺口(与 337 的 `auto ui backlog` 同构)。

## 10. 与其它两层的关系

- **向下**:block 组合 widget(消费 `@auto-ui/widgets` 调色板)。**widget 天花板直接决定 block 天花板** → Plan 337 TODO-A(全量 widget)同时抬高两层。
- **向上**:app = shell + 路由→block 选择 + block 数据接线(Design 16)。**block 是 app 生成的中间粒度复用目标**;把"生成一个 app"缩小为"每块生成/选一个 block + 接数据"。这正是降低 AI 生成空间、提升可靠性的关键。
- 三层形成完整复用阶梯:**widget(词法)→ block(句法/模式)→ app(篇章)**。

## 11. 实施阶段(概览,各自单独立 Plan)

- **Phase A(概念落地)**:确立 block kind 分类法 + `BlockRegistry` 数据模型 + `examples/blocks-gallery` 骨架(先 Form + Data-display 两类)。产出:扩展点词汇表文档 + 两类各 2 个 block。
- **Phase B(工具链)**:`auto block add/list` CLI(adopt-and-edit 拷源码)+ dataSource 接入约定 + 与 337 同构的同步/守卫测试。
- **Phase C(覆盖基准)**:每类补齐 block,优先覆盖 Design 16 的 M1-M3:note-list(Data-display)、login(Form)、chat-message-list(Composite)。
- **Phase D(依赖 AURA 扩展,较远)**:命名 slot / projection,让 block 可作"活组件"复用(非拷贝)。此前一律 adopt-and-edit。

## 12. 非目标 / 开放问题

- **不做**配置驱动的万能 block 引擎(低代码地狱)。
- **不在本设计**定 `dataSource` 的类型系统细节(归 Rung 2 / Design 16)。
- **开放**:eject 后的**升级路径**——消费者拥有源码后,block 后续改进如何 merge?短期不做(纯 adopt);长期可能需要 patch/rebase 流程。
- **开放**:per-block 的 token/主题接入细节(留给未来的 Design Token 设计)。
- **开放**:block 是否需要"泛型"(如 `DataTableBlock[T]`)—— 取决于 Auto 泛型在 UI 场景的成熟度。
