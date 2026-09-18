# blueprint

> **Status**: active
> 路径：`blueprints/`  | 技术栈：Markdown spec（TOML frontmatter）+ .at 参考实现

AutoUI **Blueprint**（原 Block，PLAN-639 更名；代码缩写 `bp`）——三层
Widget / Blueprint / App 的中间层（Design 17）："Skill 级" UI 单元包：自然语言
spec + 参考实现 + gotchas，供 agent 组装 widgets。契约六问见
[contract.md](contract.md)。

## 目标与范围

- 每个 blueprint 是一个包：`blueprints/<kind>/<name>/` 下 `spec.md`（TOML
  frontmatter + NL 正文）+ `reference/<v>.at`（每变体一个参考实现）+ `gotchas.md`
  （反例 {wrong, why, right}）。
- **三通道分级消费**（PLAN-639 SD-02）：
  - **L1 import/bind**（平台化主通道）：应用 pac.at 声明依赖 + `use` 跨包导入 +
    声明式绑定工件（`src/front/bps/<name>.bind.at`，GENERATED）——零副本，无漂移；
  - **L2 copy reference**：`auto bp add --reference` 拷贝参考实现，落地归应用
    （eject 语义保留，头注登记来源）；
  - **L3 agent generation**：agent 读 spec 生成，`auto bp check` + `auto build`
    循环校验；结构性变体走提升评审（`promotions:`）。
- **版本面（PLAN-647 终版裁定，P639-D3 核销）**：`blueprints/` = 单版本滚动
  权威源（升版=全体消费方下次构建重构建；跨仓对齐走 git 层，bp 层禁自造
  pin/lock，锁面触发条件见 contract Q5⑤）；pac.at `version` 仅展示元数据
  非约束面；spec frontmatter 顶层与 pac.at `dep` 声明的版本类键均显式报错
  （双护栏，contract Q5 五项）。
- 不做：`auto bp` 命令实现本体在 auto-cli（cmd_bp）；组件原语在 packages/widgets；
  运行时动态插件/manifest 加载（终态另议）。

## 官方默认集判定标准（PLAN-640）

官方默认集（Tier 0，`blueprints/` 磁盘目录）是策展集不是杂物间——新包入集
须**五条全过**，防止官方集膨胀退化：

1. **跨 app 出现频率**：该区块在 ≥2 个真实应用形态中出现，或为业界 Block
   策展共识收录的模式（shadcn/ui Blocks、Ant Design Pro、PatternFly、SAP
   Fiori pattern 库可引用锚点）。
2. **契约可成文且非平凡**：`props` / `actions` / `dataSource` 至少一类有真实
   数据接口（纯静态装饰组合不入选——那是 recipe/主题的事）。
3. **palette ⊆ AURA registry ∪ schema package-origin tags 且双端可发射**：palette 每项过
   `palette_drift` 零漂移门禁——合法集 = `WidgetRegistry` ∪ schema `package_origin` tags
   （PLAN-643：official 组件包供给名经词汇面入列，chart 四 tag 首批；词表外未知名仍拒绝），
   且 vue 发射轨与 VM 解释轨均可产出产物。
4. **状态机含量 ≥ 三态**：包内至少承载三态行为（如 loading/empty/error、
   step 分步、success/error）——防"静态摆件"混入 blueprint 层。
5. **NL spec 可描述、可被 agent 组装**：六问可答（contract.md），L3 通道
   （agent 读 spec → 组装 `.at` → `auto bp check`）可走通。

## 模块架构

```mermaid
graph LR
  subgraph form
    login[form/login]
    signup[form/signup]
    settings[form/settings]
    wizard[form/wizard]
  end
  subgraph navigation
    sidebar_nav[navigation/sidebar-nav]
    sidebar_shell[navigation/sidebar-shell]
  end
  subgraph dashboard
    overview[dashboard/overview]
  end
  subgraph data-display
    note_list[data-display/note-list]
    dt_crud[data-display/data-table-crud]
    master_detail[data-display/master-detail]
  end
  subgraph feedback
    empty_state[feedback/empty-state]
    result_page[feedback/result-page]
  end
  subgraph editor
    note_editor[editor/note-editor]
  end
  click login "./form-login/" "form/login"
  click signup "./form-signup/" "form/signup"
  click settings "./form-settings/" "form/settings"
  click wizard "./form-wizard/" "form/wizard"
  click sidebar_nav "./navigation-sidebar-nav/" "navigation/sidebar-nav"
  click sidebar_shell "./navigation-sidebar-shell/" "navigation/sidebar-shell"
  click overview "./dashboard-overview/" "dashboard/overview"
  click note_list "./data-display-note-list/" "data-display/note-list"
  click dt_crud "./data-display-data-table-crud/" "data-display/data-table-crud"
  click master_detail "./data-display-master-detail/" "data-display/master-detail"
  click empty_state "./feedback-empty-state/" "feedback/empty-state"
  click result_page "./feedback-result-page/" "feedback/result-page"
  click note_editor "./editor-note-editor/" "editor/note-editor"
```

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| contract | Blueprint 六问契约（输入/输出/状态归属/变体/打包解析/双形态） | active |
| form/login | 登录表单 blueprint（minimal / with_sso / two_column） | active |
| form/signup | 注册表单 blueprint | active |
| form/settings | 设置页 blueprint（分区 + 危险区确认） | active |
| form/wizard | 分步向导 blueprint | active |
| navigation/sidebar-nav | 侧边导航（三段式内容）blueprint | active |
| navigation/sidebar-shell | 应用壳 blueprint（header + sidebar + 内容槽 + user menu） | active |
| dashboard/overview | 仪表盘总览 blueprint | active |
| data-display/note-list | 笔记列表展示 blueprint | active |
| data-display/data-table-crud | 查询表格 CRUD blueprint | active |
| data-display/master-detail | 主从视图 blueprint | active |
| feedback/empty-state | 空态三分法 blueprint | active |
| feedback/result-page | 结果页 blueprint | active |
| editor/note-editor | 笔记编辑器 blueprint | active |
