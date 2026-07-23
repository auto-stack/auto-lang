# Block 层（一等公民 / Skill 模型）

## 范围

widget / block / app 三层中的中间层：组合卡（LoginForm/DataTable/KanbanColumn）作为一等公民的定义、生成与消费机制。落地代码 `ui_gen/block/`（registry/spec）+ 顶层 `blocks/` 包目录。

## 核心论点

> Widget : Tool :: Block : Skill。

block 不是预烘焙代码库，而是**自然语言 spec + 结构化契约**，由 AI 用 widget 当积木现场组装出 `.at`；消费者拥有生成的源码、可改可 eject。库的原子是 spec（及 reference 输出），不是成品代码。订制的主通道是自然语言，兜底是 acceptance 验收清单而非参数化。

## 双产物模型

每个 block 发布两份产物，消费者二选一：

1. **Spec（即 Skill）**：frontmatter（`kind`/`palette`/`extension_points`/`dataSource`/`variants`/`acceptance`）+ NL 正文（intent/变体说明/组装指引/gotchas 指针）。
2. **参考实现集**：每个 variant 一份 `.at`（minimal/with_sso/magic_link…），三重价值——默认拷贝源、spec 仓回归 fixture（"AI 能否仅凭 spec 复现"）、变体范围的物化样本。

外加 **gotchas.md**：负样本教学，结构为 {错误做法, 为什么错, 正确做法}，随 AI 实际失败模式累积（自增强闭环）。

```
blocks/form/login/
  spec.md  reference/{minimal,with_sso,magic_link}.at  gotchas.md
```

## 边界机制

- **三个判据**：In-scope = 一个可辨识 UX 模式 + 固定行为契约 + 有界扩展点词汇表；Out-of-scope = 横切基础设施（auth/routing/theming/全局 store）与跨模式业务流（归 app 层）；**eject = 天花板**，超出词汇表则消费者完整接管。
- **kind 分类法**（圈住自由的关键机制）：Form / Data-display / Feedback / Layout / Composite，每类一套固定扩展点词汇表（如 DataTable 类：`columns`/`row`/`cell(:col)`/`empty`/`loading`/`error`/`toolbar`/`pageSize`）。
- **命名变体**取代无限 props（`compact`/`with-filters`/`dense` 等预调档位）。
- **配色/间距归 design token**，不进 block 内部。
- **订制分流**：数据驱动 → schema/config；结构性 → slot（Phase D，AURA 命名投影，较远）；风格 → token；超词汇表 → eject。

## 数据契约

block 不绑死端点（不知道 `/api/notes`），接 typed fetcher `dataSource` + 参数或 `#[api]` 类型引用；**loading/error/empty 是一等 view 状态**（强制槽，非事后补丁）——直接对接 app 生成 Rung 2 的数据生命周期。

## 现状

`ui_gen/block/spec.rs:BlockSpec`（frontmatter 解析）与 `registry.rs:BlockRegistry`（`scan_dir`/`palette_drift`）已实现；`blocks/` 下有 form/data-display/editor/navigation 四类包。plan-342/343 自述"未开始"，代码已先行（分歧，以代码为准）。`auto block add/list` CLI 未实现。

## 显式非目标

- 不做配置驱动的万能 block 引擎（低代码地狱）。
- 不做预烘焙成品库为主形态（spec + reference 才是；reference 是 spec 的默认产物与回归 fixture）。
- 不定 `dataSource` 类型系统细节（归 Rung 2）。
- eject 后的 spec 升级回流短期不做。
- 开放：AI 生成可复现性（acceptance + reference 锚定收敛，是否需确定性种子留 Phase B）；block 泛型（`DataTableBlock[T]`）取决于 Auto 泛型在 UI 场景的成熟度。

> 来源: docs/design/17-blocks-first-class.md；crates/auto-lang/src/ui_gen/block/{spec,registry}.rs；blocks/
