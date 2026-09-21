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
| 5 | **打包与解析** | 包 = `blueprints/<kind>/<name>/`（`spec.md` + `reference/<v>.at` + `gotchas.md`；kind 词表（PLAN-640 对齐磁盘现实）：form/navigation/dashboard/data-display/feedback/editor/layout/composite）。应用经 pac.at `dep` 声明依赖（PLAN-635 声明门控：物化未声明=幽灵依赖阻断）；解析走 `resolve_module_path` 既有链（base_dir→父目录→逐级 deps/path dep 探测）。**连字符变体规则（PLAN-649，SD-01）**：点号 use 路径按字面量转换未命中时，对含下划线的路径段按段序枚举连字符（kebab）变体再探——字面量优先保序（`foo_bar/` 与 `foo-bar/` 并存时前者胜），变体组合上限 4；覆盖本地 probe、back 根映射与 dep probe_pkg 三探测点，VM/vue 双轨同链受益。bp 包磁盘 kebab 命名（`feedback/empty-state/`）与 use 下划线书写（`bps.feedback.empty_state...`）各自不变（`plan649_bp_tests` 正/负断言；640 登记 8/13 包 L1 直连不可达由此销号）。**bps-gallery 发现机制（PLAN-676 SD-01，取代 PLAN-640 import.meta.glob 零产物裁定）**：画廊目录发现=`auto bp list --format at` 生成产物 `examples/bps-gallery/src/front/registry.at`（`pub fn all_bps() List` 静态表；提交入库 + CI 漂移校验步重跑发射器 diff 非空即红）——Auto 化后 VM 臂无法读盘，双臂同源要求静态表（PLAN-625 ui-gallery registry.at 先例）；字段避名 `spec_text`（spec 是 .at 保留字，pac_text 同型）。`auto bp list` 仍为权威 registry 视图，registry.at 是其只读物化。原零产物裁定（Vite glob，PLAN-640）随手写壳退役并作废。**kind 治理规则**：新增 kind 须随 spec 沉淀更新本词表，并同步 gallery kindOrder 偏好序（PLAN-676 起在 `examples/bps-gallery/src/front/catalog.at` 的 `kind_preference()`；原 `src/bps.ts` 已随手写壳退役）；禁止在包名里自造词表外 kind。**版本面规则（PLAN-647 终版裁定，P639-D3 收口）五项**：①**单版本**——`blueprints/` 为单调滚动的权威源，同一时刻每包仅一个有效版本，历史由 git 承载（无目录内多版本共存）；②**升版语义**——包内容变更即新版本，全体消费方**下次构建即重构建**（无缓存失效协议、无兼容窗口承诺；breaking change 以 spec 契约字段演进 + gotchas 登记；对齐兜底=解析序声明门控 + master CI 门禁，如 build-bps-gallery 的 `blueprints/**` paths 触发）；③**pac.at `version` 定性**——库级展示元数据（工具面如 `auto bp list` 可示），**非约束面**，消费方不得据此做版本判断；随目录级演进手工递增、不承诺语义化；④**跨仓对齐**——消费仓（如 auto-down）以 git 对齐（CI pin auto-lang commit 或同 commit 家族检出），**禁止**在 bp 层自造 pin/lock，直到锁面机制正式立项；⑤**锁面触发条件**——出现仓族外消费者 / blueprints 独立仓化或独立发版节奏 / 消费方需要长周期不跟随 master 的冻结构建，三者任一满足即立项 Option B（lock/pin 机制，另立计划）。 |
| 6 | **双形态同语义** | 同一 bp 在 a2ts(vue 发射轨) 与 AutoVM(iced 解释轨) 行为一致：结构、状态、事件语义全等；`actions{}`/`menubar`/`toolbar` 配置两轨均发射/解释（PLAN-639 补齐 vue 轨）。双端验证按 autoui-verifier 双端模式执行。 |

## 消费通道与产物纪律（SD-02 摘要）

| 通道 | 形态 | 产物纪律 |
| --- | --- | --- |
| **L1 import/bind**（平台化主通道） | 应用 pac.at 声明依赖 → `use` 跨包导入 bp 参考实现 + 声明式绑定（`src/front/bps/<name>.bind.at`：slot→布局容器、action 注入、token 覆盖） | **零副本**；bind 工件带 GENERATED 头注，是产物不是资产——重生成安全 |
| **L2 copy reference** | `auto bp add --reference` 拷贝参考实现 | 落地文件归应用所有（eject 语义，离线/深度定制场景）；头注登记行（来源 bp + 版本 + 日期）为回迁 L1 留账面 |
| **L3 agent generation** | agent 读 spec + 需求 → 组装 `.at` → `auto bp check` + `auto build` 验收门 | 产出后走**变体提升评审**（spec 未声明的结构性差异 → `promotions:` 小节提案 → 评审进 spec 或留应用侧登记 DEBTS）；落地文件同样登记行 |

## 组件级参与门（PLAN-070 T-05 增补）

vue 轨组件（非 app 壳）消费 ui_config 的参与条件 = **handler 交集**：组件
AST 声明的 handler 与 ui_config action 的 handler 有交集才注入 ActionsBlock
（发射的 handler fn + 快捷键 keymap 引用宿主自身 handler，无交集即 TS2304
泄漏类）。app 壳不豁免——占位 app 壳（真实宿主在别处，如 jade web）同样
受门约束。menubar/toolbar 视图合成不设 shadcn 模式门：渲染 `menubar {}/
toolbar {}` + actions 即选择加入命令面契约，宿主自备 menubar ui 模块 +
reka-ui（shadcn: off 项目如 jade-garden front 宿主化 ui/menubar+ui/button）。

## 变体提升评审（轻量流程）

1. L3（或深度定制的 L2）产出出现 spec 未声明的结构性差异；
2. 在对应 bp 包 spec.md 增 `promotions:` 小节：差异描述 → slot/action 点提案；
3. 评审通过 → 进 spec 正式声明（下个应用免费用）；拒绝 → 差异留应用侧并登记
   `docs/plans/KNOWN-DEBT-AND-RISKS.md`。

## 验证面

- 包完整性：registry 扫描（spec name ↔ 目录名一致、声明 variant 必有 reference 文件）。
- palette 无漂移：palette 声明的每个 widget 必须在**合法集**内（`BlueprintRegistry::palette_drift`）——合法集 = **AURA registry（WidgetRegistry tags，含 alias）∪ schema `package_origin` tags**（official 组件包词汇面，PLAN-643：chart 四 tag `area-chart`/`bar-chart`/`line-chart`/`donut-chart` 首批；484 裁定下 chart 只以包形态存在，palette 面经 schema 分类认识它，不注册回 WidgetRegistry）。词表外未知名仍拒绝（负断言：`pie-chart` 等既非 registry tag 亦非 package_origin 者照常报漂移）。**icon 归属落点（PLAN-649，SD-02）**：schema `builtin_widget` 内建进 palette 的归属=**WidgetRegistry 注册补齐**（修向候选①落地；icon 为首例——vue 轨 node_to_html icon 专臂发射 Lucide 组件/iconfile 位图，注册仅补词汇面准入、零发射扰动；候选②"合法集扩 builtin_widget tier 白名单"未采纳，unclassified 杂音防线不放松）。filetree palette 恢复最小集 `["icon", "text"]`（`plan649_bp_tests` 正/负断言）。
- L1 bind：GENERATED 头注 + slot/action id 与 spec 声明一致性（缺位即构建错）。
- **版本键面护栏（PLAN-647）**：spec.md frontmatter **顶层**版本类键（`version`/`pin`/`rev`/`tag`/`branch`/`commit`）与 pac.at `dep` 声明块内同类键均为**显式错误**（错误消息指向 Q5 裁定；负测试 `plan647_bp_version_tests`）——锁面语义正式引入前，任何版本面字段必须显式失败而非静默忽略，防"假版本约束"漂移进生态。`[dataSource]` 表内 `version` 槽名是合法 fetcher 签名，不属版本键。
- **bps 扫描 fn 转译（PLAN-645，SD-01）**：bp reference 的跨文件 `.at` fn 导入（bare `use tree_util:` / bps 限定 `use bps.<pkg>.<支撑件>:` 均可）由 vue 轨 bps 扫描发射路径按 plan522 式 helper 转译内联进消费 SFC——仅被引符号闭包（Q-1 默认裁定），未导入符号不入 SFC；内联只发生在 reference 侧扫描产物，L1 bind 工件仍零副本、GENERATED 语义不变。组合形态由此启用（filetree `default` 变体回归：`examples/capability-tests/047-bp-compose` + `plan645_bp_tests` 正/负断言；DEBTS 070 第二行销号）。
- **扫描根运行时解析（PLAN-645，SD-02）**：`BlueprintRegistry::with_defaults` 三级序 = `AUTO_BLUEPRINTS_ROOT` env 覆盖 → cwd 向上找 `blueprints/` 目录 → 编译期 `CARGO_MANIFEST_DIR` 兜底（与跨仓解析序同构）——`auto bp list/show/add/check` 在 worktree/外部检出内可见本仓包库（DEBTS 070 第一行销号）。
- **库包 dep 扫描发射纪律（PLAN-645）**：库形态 dep（bps 包库——无 `src/front/`、无 `front/` 的原目录形态）在 vue 全量构建 dep 扫描下按**模板源**对待：per-file strict 校验失败降为告警，不硬炸消费方构建（bp reference 可携带消费方契约导入，如 with_charts 的 `use { package: official from "components" }` 由消费方供给包目录，包内 standalone 编译必然 S003）；实际被消费变体的 SFC 缺席由 vite import 解析兜底。应用形态 dep（有 front 布局）strict 门禁不变。
- 双端：结构快照 + DOM 断言（vue 轨）与 vm-smoke（VM 轨）同绿。
