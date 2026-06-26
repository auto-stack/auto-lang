# 17 — Blocks as a First-Class Tier (Skill Model)

> **状态**:设计文档(概念设计)
> **日期**:2026-06-26
> **关联**:Design [16 App Generation](16-app-generation-and-ai-authoring.md)、widget 层(Plan 331/336/337)、`/auto-lang-creator` skill、`examples/a2ui-composer`、`examples/a3ui-replica`
> **目的**:把 **block** 提升为 widget / block / app 三层里的"一等公民",并回答核心难题——**如何定义"常见、又需要灵活订制"的 block 的范围与边界**。
> **核心论点**:Widget : Tool :: **Block : Skill**。block 不是预烘焙的代码库,而是**自然语言 spec + 结构化契约,由 AI 用 widget 当积木组装出 `.at`**;消费者拥有生成的源码、可改可 eject。库的原子是 spec(及 reference 输出),不是成品代码。

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

把 block 当 widget 那样做黑盒 → 配置项爆炸,永远不够用(一个 LoginBox 的 props 会膨胀到 SSO/2FA/魔法链接/校验规则/主题/视觉密度…,**变体空间是高维的,参数覆盖不到**)。把 block 当纯示例 → 不算复用,只是样板代码。

### 2.1 关键类比:Block 之于 Widget,正如 Skill 之于 Tool

> **Widget : Tool :: Block : Skill。**

- **Widget ↔ Tool**:粒度小、功能清晰、固定实现、靠**参数**给不同行为。
- **Block ↔ Skill**:变体多到无法参数化;用**自然语言**描述意图,AI **组装**多个 widget + 逻辑成可用模块;按**结果(acceptance)评判**,而非按 config。

正如 Skill 是"模型要遵循的自然语言指令 + 对 tool 的编排"(不是预烘焙的程序),block 也不是预烘焙的代码工艺品,而是**自然语言描述的能力 + 对 widget 的编排**。

### 2.2 本设计的回答(基于该类比)

> **block 的本质是 Skill:一个"自然语言规格 + 结构化契约",由 AI 用 widget 当积木组装出 `.at`;消费者拥有生成的源码、可改可 eject。**

由此,§2.1 的几个判断要更新:
- **库的原子是"规格(spec)",不是烘焙好的代码。** `BlockRegistry` 存的是 block 规格(NL 意图 + 结构化 frontmatter),不是成品 `.at`。`auto block add` 读 spec + 当前 app 的需求,**AI 生成**一份定制 `.at`。
- **AI 是组装机制本身。** 订制的主通道是"用自然语言说你想要什么",不是配 20 个 props。
- **靠 acceptance(验收)兜底,不是靠参数化。** 没有"一个 LoginBox 覆盖大多数 app"的参数集;有的是一份验收清单(编译过、暴露必需扩展点、只用调色板 widget、处理 loading/error/empty、可访问、满足所述意图)—— AI 输出按此评判。

**从原 owned-source 模型保留下来的(现在是"生成/输出"侧的机制,而非配置):**
- **拥有并可编辑的 `.at` 输出** —— 你仍拿到源码、可改可 eject。**拥有的是输出,不是库。**
- **kind + 扩展点词汇表(§4.2/§7)** —— 这些现在活在**规格里**,作为**约束 AI 的结构**("你在造一个 Form 类 block;必须暴露这些扩展点;只能用这些 widget")。kind 是**规格的 schema**,不是运行时类型。
- **`dataSource` / eject / design token** —— 对生成的输出而言不变。

### 2.3 双产物模型(库 vs AI 的二元消解)

每个 block 在库里发布**两份产物**,消费者二选一:

1. **Spec(即 Skill)** —— NL + 结构化 frontmatter。AI 按 app 现场实例化、订制。
2. **Reference 输出(`.at`)** —— 一份已知良好的生成实例。既是 (a) 想要"标准版"消费者的默认拷贝源,**又是 (b) spec 的回归测试**:"AI 能否仅凭 spec 复现一个等价物?"不能 → spec 缺了东西。

于是**两者都有**:要开箱即用的标准 LoginBox → 拷 reference;要适配你的 SSO+2FA+品牌 → 让 AI 照 spec 生成。维护 reference 反过来校验 spec。这也呼应 `/auto-lang-creator` skill —— **block spec 本质是一份"经库策展的、专门的 skill 文件"**。

具体 spec 形态示例(LoginBox):

```
# block: form/login
kind: form
palette: [Input, Button, Label, Checkbox, Separator, ErrorState]
dataSource: { attempt: (creds) => Session, providers?: []Provider }
extension_points: [fields, submit, third_party, success, error_display]
variants: [minimal, with_remember, with_sso, magic_link]
acceptance:
  - auto build green; 只用调色板 widget(+ 原生)
  - 暴露 {fields, submit, error} 扩展点为 EDIT 区
  - 处理 loading + error 状态(契约)
  - 可访问:label 关联、error 播报、键盘可达
---
# Intent
凭据采集表单,向后端 session 端点认证。

# 本 block 吸收的变化(per-app 变体)
fields / SSO / 2FA / captcha / 成功跳转 vs 内联 / 校验规则 / 密码规则 / 视觉密度。

# 组装指引
每个字段用 Label+Input;submit → dataSource.attempt;Button 显 loading;
错误经 error 槽显示;输出里标 EDIT 区。

# Pitfalls
别把端点写死(用 dataSource);别漏 loading/error(是契约);…
```

frontmatter = AI 读取的机器可检契约;正文 = 人/AI 可读的组装说明;reference = 锚点。

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

## 9. block 的载体:spec 库 + 生成器 + gallery + `auto block`

> 按 §2 的 Skill 模型,库的原子是 **spec(+ reference 输出)**,不是烘焙好的代码。

- **`BlockRegistry`**(`crates/.../ui_gen/block/`):按 `kind + name` 索引;每条是**一份 block spec** = 结构化 frontmatter(`kind` / `palette` 子集 / `extension_points` 词汇表 / `dataSource` 期望签名 / `variants` / `acceptance` 验收清单)+ NL 正文(intent / 变体说明 / 组装指引 / pitfalls)+ 指向 **reference `.at`** 输出。
- **生成器(AI)**:消费 spec + 当前 app 的需求(自然语言)→ 生成定制 `.at`;由 acceptance 清单 + `auto build`/`vue-tsc` 编译 + preview 回路校验(回路本身是 Design 16 Rung 5)。可复用 `/auto-lang-creator` skill 基建。
- **`examples/blocks-gallery`**:每类若干 block;每个 block 给 **reference(默认输出) + AI 订制实例 + 标注的 EDIT 扩展点** 三视图对照 —— 同时是展示、是 spec 的回归 fixture、是"边界:默认能走多远、何时 eject"的样例。
- **`auto block add <kind>/<name>`** 两种模式:
  - `--reference`:拷 reference `.at`(开箱即用,等价 §2.3 的 adopt)。
  - 默认 / `--from "<自然语言需求>"`:AI 照 spec 生成定制 `.at`;提示关联 widget 依赖 + 标注 dataSource 接入点。
- **`auto block list` / `backlog`**:对照 widget registry 与 app 实际用到的 block,看缺口(与 337 的 `auto ui backlog` 同构)。

## 10. 与其它两层的关系

- **Widget : Tool / Block : Skill / App : 由 skill 编排的 plan**。
- **向下**:block 的 spec 指定它编排哪些 widget(消费 `@auto-ui/widgets` 调色板)。**widget 天花板直接决定 block 天花板** → Plan 337 TODO-A(全量 widget)同时抬高两层。
- **向上**:app = shell + 路由→block-intent,每个 intent 由其 spec 现场实现(Design 16)。**block 是 app 生成的中间粒度"生成目标"**(不是复用目标);把"生成一个 app"缩小为"每块照 spec 生成一个 block + 接数据"。
- 三层形成完整阶梯:**widget(词法)→ block(句法/模式,AI 编排)→ app(篇章)**。

## 11. 实施阶段(概览,各自单独立 Plan)

- **Phase A(spec 基础)**:定 block spec 格式(frontmatter schema + NL 正文 + acceptance)+ `BlockRegistry` 数据模型 + `examples/blocks-gallery` 骨架(先 Form + Data-display 两类)。每类手写 1-2 个 spec **及其 reference 输出**(reference 既是默认产物,也是回归 fixture)。产出:spec 格式规范 + 两类各 1-2 个 (spec, reference)。
- **Phase B(AI 生成器 + CLI)**:`auto block add` 两模式(reference 拷贝 / AI-from-spec 生成);生成器接 `/auto-lang-creator` 基建 + `auto build` 校验回路;dataSource 接入约定;与 337 同构的同步/守卫测试(spec ↔ reference 一致性、生成器能否复现 reference)。
- **Phase C(覆盖基准)**:每类补齐 spec,优先覆盖 Design 16 的 M1-M3:note-list(Data-display)、login(Form)、chat-message-list(Composite)。
- **Phase D(依赖 AURA 扩展,较远)**:命名 slot / projection,让 block 可作"活组件"运行时复用(非生成/非拷贝)。此前一律"spec→AI 生成"或"拷 reference"。

## 12. 非目标 / 开放问题

- **不做**配置驱动的万能 block 引擎(低代码地狱)。
- **不做**"预烘焙 block 成品库"作为主形态(spec + reference 才是;reference 是 spec 的默认产物/回归 fixture,不是库的原子)。
- **不在本设计**定 `dataSource` 的类型系统细节(归 Rung 2 / Design 16)。
- **开放**:AI 生成的**可复现性**——同 spec 多次生成可能产出不同 `.at`;靠 acceptance + reference 锚定 + 编译/preview 回路收敛。是否需要"确定性种子"留给 Phase B。
- **开放**:eject 后的**升级路径**——消费者拥有生成源码后,spec 改进如何回流?短期不做(纯 owned);长期可能需要 spec diff/patch 提示。
- **开放**:spec 的**作者侧工具链**——谁写 spec、如何验证 acceptance(spec 仓的 CI:每个 spec 能被 AI 复现出通过 acceptance 的 `.at`)。
- **开放**:block 是否需要"泛型"(如 `DataTableBlock[T]`)——取决于 Auto 泛型在 UI 场景的成熟度。
