# ace-migration: m2-autoui — 传统 AutoUI 声明的 ACE 接入（IF-980-FIELDS 字段分母已验切片）

> Status: **DIALECT 批已验**（PLAN-980 B-DIALECT-I/V/R，2026-11-16；work→review（含
> 复核反例两族）→phase fold 收据见计划 §9）。已交付：字段级投影完整性清单共享机制
> （ace-core `source::field_manifest`）+ `WidgetDecl` 完整分母可执行清单（auto-ui
> `-dialect p980_fields`，ui.widget-decl/v1）+ **AGE 组件面对 widget 成员槽位与 view
> 树的接线**——002-counter/001-helloworld 完整原 widget 源经同一 AGE Schema
> legacy-widget 别名全量投影并通过共享场景 verify（含反引号 f-string 绑定、str 状态
> 字段、复合赋值、`!` 一元、design 绑定表达式）；`node` 规范头副本同字段 ID 集；
> 别名关闭/坏形状/未接线槽位在原 span 具名拒绝。
> 未交付（保持显式）：`style`/`setup`/`actions`/widget 级 `use{}` 四槽位为具名拒绝
> （KNOWN-DEBT 登记）；design 的 if/else 值与求值行为（结构锚定）；STATE/NATIVE/
> WEB/RUST/TAURI/JET/ARK/PARITY 各批与 V-UI-FULL/真实窗口验收；959 全行/组件全模式
> 分母（T-01/PARITY）。
> 来源：[PLAN-980](../../plans/980-ace-m2-autoui.md) §5/§6/§9；消费方：
> [PLAN-979](../../plans/979-ace-m2-schema.md) r11 §5 V-11（T-08/T-04 回验）。
> 本文件只写**已验行为与接口**。

## 1. 共享字段级清单机制（ace-core）

`experimental/ace/crates/core/src/source/field_manifest.rs`（`cargo t ace-source` 组，
PLAN-980 起）：既有 `domain::mapping_complete` / `age_component::component_coverage`
（role 粒度）之上的**字段/构造 ID 粒度**层，Core 不含产品语义、不反依 UI crate：

| API | 事实 |
|---|---|
| `FieldManifest { domain, version: DomainVersion, rows }` | 版本化必需清单（交接件身份 `<domain>/v<major>.<minor>` + 行集） |
| `RequiredFieldRow` | 语义字段按 PLAN-980 r10 §5 冻结：`id/origin/source_span/parent_role/type_rule/applicability/projection_path/check_id/consumer_owner` |
| `FieldApplicability` | `Required{mode}` / `Optional{mode}` / `Conditional{mode, reason}`（条件适用必带可核理由） |
| `field_coverage(presented, manifest) -> FieldCoverage` | `required/present/missing/unknown`：必需行缺席（红）与投影侧**未知构造**（红，不静默丢弃）都具名可判；合法近邻/可选行缺席不影响 |
| `validate_field_manifest` | 清单自检（ID 唯一/非空、语义字段非空、条件理由非空、span 有序）——不可执行的表直接拒 |
| `component_field_ids(candidates, vocab)` | 组件面候选（`project_component_candidates` 产物，带原 span）→ 字段级出现集；ID 词汇表 `ComponentFieldVocab` 由清单 owner 注入 |

永久 case `tests/cases/plan980_field_manifest.rs`（6 项）：清单/覆盖破坏近邻；
`source → candidate → 共享 verify（compile_component_scenario_observing）→ 字段覆盖`
真实链路；删必需清单行 = unknown 红 / 换映射实现 = missing 红及恢复绿；
AGE 组件面对未接线成员（`msg`/`design`）在成员 span 具名拒绝（不静默略过）；
`widget` 兼容头别名闸（关闭=原 span 拒绝 `source.age.legacy-widget`；开启=经场景
verify 接受）与 `node` 规范头同字段 ID 集连接。

## 2. UI 字段分母（auto-ui-dialect `p980_fields`）

清单 `ui.widget-decl/v1`（115 行）按现行
`auto-ui/crates/auto-ui-dialect/src/ast.rs::WidgetDecl`（sha256/16 `1d10f5ff2566422e`）
及其全部引用类型逐构造冻结：模块级项（UiStmt 10 变体）、widget 19 字段（msg
variant/payload/payload-name/quoted，model field/var/primary/decorator，computed，
view/named view，on handler/stream，bind key，prop/default，routes def/param，
lifecycle，design entry，style，ext_import 5 kind/call-args/ref-fields，watch
source/immediate/deep，setup/ref-annotation，expose，actions action/attr/menubar/
menu/menu-item/toolbar/toolbar-item，timer entry/when）、store 声明、view 树 8 节点
形态（element/prop/style-binding/event/inline、text/interp/binding、for/index/key、
if/else、component、outlet/memo、memo/exact、link）、view-fn、style-recipe、theme-data。

- 穷尽抽取 `widget_field_occurrences(UiCode) -> Vec<FieldOccurrence>`：每个实际构造
  一个出现；view 树携带真实字节 span `(offset, len)`（AST 提供处），其余槽位
  span=`None`（AST 不携带，见 §4 边界）。
- 未登记构造 = `FieldExtractRejection{ code: "ui.field-unknown", construct, span }`
  具名拒绝——AST 新增字段未登记分母时在抽取处红出，绝不静默略过。
- 双向对账 `reconcile_field_manifest(rows, constructs)`：清单行 ↔ 映射实现
  （`P980_CONSTRUCTS` 115 键）精确互达；删行/加行/删映射项分别可区分地红。
- auto-ui-compiler 经 dev-dep 消费 ace-core 共享检查（仅测试面；生产依赖闭包不变）。

## 3. 已验输入与独立 oracle（锚定 fixture）

`examples/ui/002-counter/src/front/app.at`（sha256/16 `04937c35325faa74` 冻结身份）：

- `parse_ui` → 穷尽抽取 → 共享覆盖：16 必需行（002 锚定子集）全覆盖、未知 0。
- 槽位计数独立 oracle：msg variant 9（6 声明 + 3 内联 lambda 铸造 `__evt_*`）、
  model field 3（均 `var`）、design entry 2（theme/mode）、on handler 9、view 元素
  10（col/text/2×row/6×button）、prop 11、event 6。
- 元素 span 与探针冻结值逐一相等（如 col (917,727)、`button "切换明暗"` (1464,112)；
  span 覆盖子元素区到闭括号，tag 词在 span 前）。
- 块锚点顺序（行注释剥离后）：`use themes` < `widget App` < `msg {` < `model {` <
  `design {` < `view {` < `on {`。
- 普通 Node 迁移副本：仅声明头 `widget`→`node` 差 2 字节，其余字节恒等
  （UI 面解析 `widget`；`node` 规范头经 AGE 组件面同 ID 连接，§1）。
- 语料清扫：全仓 1592 个 `.at`（排除 pac.at/target）中 697 个可独立解析源经
  parse+抽取**未知构造拒绝 0**。

## 4. AGE 组件面的 widget 成员接线（DIALECT 批，2026-11-16）

- **grammar**（`auto-source.at`，全部 `profiles: [age-component]` 门控；token 表
  零新增——其余 profile 的 GrammarId 不变）：成员槽位 `msg`/`design`/`named
  view`/`bind`/`routes`/`expose`/`lifecycle`/`computed`/`watch`/`timer`、model
  `var` 前缀、widget 形 `on { .Msg -> { 复合赋值 } }`（与 979 的 `on <Msg> {
  assigns }` 并存，投影为同一 handler 结构）、view 树（元素 `[label] { 项* }`/
  属性行 `key: value`（事件= `on` 前缀键，投影层归类，同 auto-ui 规则）/内联
  lambda `() => { 点赋值 }`/裸与带属性 text）、`use <mod>.<name>: A, B` 导入。
- **f-string**：widget 反引号形 `` `text ${expr}` `` 在 AGE 组件面入口统一源级
  desugar 为 `__fstr(...)`（沿 PLAN-964 机制与边界——f-string 内部 span 可能
  移位，外部字节精确；未闭合/嵌 brace 拒 `source.fstr-syntax`）。
- **共享 verify**：场景驱动支持 `str` 状态字段（观测仍限 int）；`.field` 自引用
  （投影为 `Field{base:Var("self")}`）与 `!` 一元在脱糖重命名通道内；text-record
  描述符附加 `text_display`/`fstr` case（binder/verify 共享实现原就支持）。
- **永久 case**（`cargo t ace-source`）：`plan980_dialect_full` ——002 冻结副本
  （sha256/16=04937c35）经别名 Accept 全量投影 + 2 实例场景共享 verify + 别名
  关闭原 span 拒 + 字段覆盖闭合/计数 oracle/Node 副本同 ID 集/删行反控；001
  冻结副本（b2763be51）带属性 text 腿；标识符上下文保持；新成员坏形状原位拒。
- **合同演进**：979 的 `text <expr> {}` 空括号冻结面扩为三形（空括号/带属性
  体/裸标签——001 真实语法）；979 反例 unknown-attribute 改无冒号形（979 文件
  内 PLAN-980 注记；具名结构拒绝合同不变）。

## 5. 边界与已知限制（未交付面保持显式）

- `style`/`setup`/`actions`/widget 级 `use{}` 四槽位在成员 span 具名拒绝
  （KNOWN-DEBT 登记，关闭条件在案）——具名拒绝即交付面，非静默缺口。
- `parse_ui` 为单文件语义：多模块工程源（`use store:` 等跨文件引用）独立解析
  报 `undefined variable`——完整批的语料验证按工程闭包入口执行。
- `ViewEvent.inline` 在 parse 输出上恒 `None`（内联 lambda parse 期铸造为
  `__evt` handler；分母行 Optional 并注记铸造去向）；插值文本在 002 形态归一为
  元素 `text` prop（`view.node.text.interp` 行同理）。
- 本切片不含运行时行为：组件状态/事件执行（980/STATE）、窗口（NATIVE）、各
  目标工程（WEB/RUST/TAURI/JET/ARK）与全模式联合覆盖（PARITY）均未交付。
