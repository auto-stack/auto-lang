# Blueprint 契约（六问）

> **Status**: active
> 归属：PLAN-639 SD-01（Design 17 延伸）。Blueprint = Widget / Blueprint / App
> 三层的中间层，代码缩写 `bp`；本契约是每个 blueprint 包必须可回答的六个问题
> 的裁定，消费机制（L1/L2/L3 三通道）见 [project.md](project.md)。

## 六问裁定

| # | 问题 | 裁定 |
| --- | --- | --- |
| 1 | **输入** | spec.md frontmatter 声明 data props（名/型/必选/缺省）。L1 绑定与 L2/L3 落地均须满足声明，缺省可省。props 是 bp 的数据接口；消费方经 props 传数据，不得绕过声明直改 bp 内部。 |
| 2 | **输出** | events 声明（名/负载）。行为统一走 **action 点**：bp 内只声明所需 action id 与语义契约，实现由消费应用经 `actions{}` 注册表注入。bp 不自带业务 handler——命令语义属于应用，bp 只约定"此处会上抛"。 |
| 3 | **状态归属** | bp 可带 **scoped store**（局部 UI 状态：折叠态/选中项等，随 bp 生命周期）。workspace/tabs/theme/keybinding 等**平台服务为单例注入，禁止 bp 私有副本**（jade §9.5 tabs_store 孪生教训成文：孪生 store 即漂移温床）。 |
| 4 | **参数与变体** | 样式小差异走 **token/recipe**（PLAN-635/607 机制：宿主 `style:` 消费面 + 跨包 recipe 导入）；结构性小差异走 **slot**；reference variants 是变体的物化样本（拷贝源的候选形态），不是参数化机制本身。小差异禁止 fork 包。 |
| 5 | **打包与解析** | 包 = `blueprints/<kind>/<name>/`（`spec.md` + `reference/<v>.at` + `gotchas.md`；kind 词表（PLAN-640 对齐磁盘现实）：form/navigation/dashboard/data-display/feedback/editor/layout/composite）。应用经 pac.at `dep` 声明依赖（PLAN-635 声明门控：物化未声明=幽灵依赖阻断）；解析走 `resolve_module_path` 既有链（base_dir→父目录→逐级 deps/path dep 探测）；版本面 MVP=主检出单版本。**kind 治理规则**：新增 kind 须随 spec 沉淀更新本词表，并同步 gallery kindOrder 偏好序（`examples/bps-gallery/src/bps.ts`）；禁止在包名里自造词表外 kind。 |
| 6 | **双形态同语义** | 同一 bp 在 a2ts(vue 发射轨) 与 AutoVM(iced 解释轨) 行为一致：结构、状态、事件语义全等；`actions{}`/`menubar`/`toolbar` 配置两轨均发射/解释（PLAN-639 补齐 vue 轨）。双端验证按 autoui-verifier 双端模式执行。 |

## 消费通道与产物纪律（SD-02 摘要）

| 通道 | 形态 | 产物纪律 |
| --- | --- | --- |
| **L1 import/bind**（平台化主通道） | 应用 pac.at 声明依赖 → `use` 跨包导入 bp 参考实现 + 声明式绑定（`src/front/bps/<name>.bind.at`：slot→布局容器、action 注入、token 覆盖） | **零副本**；bind 工件带 GENERATED 头注，是产物不是资产——重生成安全 |
| **L2 copy reference** | `auto bp add --reference` 拷贝参考实现 | 落地文件归应用所有（eject 语义，离线/深度定制场景）；头注登记行（来源 bp + 版本 + 日期）为回迁 L1 留账面 |
| **L3 agent generation** | agent 读 spec + 需求 → 组装 `.at` → `auto bp check` + `auto build` 验收门 | 产出后走**变体提升评审**（spec 未声明的结构性差异 → `promotions:` 小节提案 → 评审进 spec 或留应用侧登记 DEBTS）；落地文件同样登记行 |

## 变体提升评审（轻量流程）

1. L3（或深度定制的 L2）产出出现 spec 未声明的结构性差异；
2. 在对应 bp 包 spec.md 增 `promotions:` 小节：差异描述 → slot/action 点提案；
3. 评审通过 → 进 spec 正式声明（下个应用免费用）；拒绝 → 差异留应用侧并登记
   `docs/plans/KNOWN-DEBT-AND-RISKS.md`。

## 验证面

- 包完整性：registry 扫描（spec name ↔ 目录名一致、声明 variant 必有 reference 文件）。
- palette 无漂移：palette 声明的每个 widget 必须在**合法集**内（`BlueprintRegistry::palette_drift`）——合法集 = **AURA registry（WidgetRegistry tags，含 alias）∪ schema `package_origin` tags**（official 组件包词汇面，PLAN-643：chart 四 tag `area-chart`/`bar-chart`/`line-chart`/`donut-chart` 首批；484 裁定下 chart 只以包形态存在，palette 面经 schema 分类认识它，不注册回 WidgetRegistry）。词表外未知名仍拒绝（负断言：`pie-chart` 等既非 registry tag 亦非 package_origin 者照常报漂移）。
- L1 bind：GENERATED 头注 + slot/action id 与 spec 声明一致性（缺位即构建错）。
- 双端：结构快照 + DOM 断言（vue 轨）与 vm-smoke（VM 轨）同绿。
