# ace-migration: m2-autoui — 传统 AutoUI 声明的 ACE 接入（基础范围已收口）

> Status: **基础范围已收口**（PLAN-980 r20 拆分→T-05 终审→T-06 归档，2026-11-19；
> 冻结门=ace-source PASS/ace-av 1426-1428 唯二红 host_console 预存对/UI 三套
> 27+8+7 全绿）。**已交付（基础范围）**：IF-980-FIELDS 字段分母清单+组件面 widget
> 槽位/view 树接线（002/001 全源投影）；STATE 实例 store；NATIVE MIN-A-UI+HELLO
> 首窗全链（lang `1991021a5`/ui `e529dde6`）；view-if/on-if/lambda-if 控制流三刀
>（组件面 lane 真复放+`__age_vl_` 结构化铸造）；包版 Let/If 消费门（F-R21-2，
> ui `7b8456e5`/`d2e7d7a4`）；④-A 包桥/包片段 lane/包版 SFC；P3 `style` 声明面
>（lang `e8e4c16da`）。
> **后继归属与已交付扩展**（PLAN-980 §0 移交表，归档件）：完整 AV/A2R/AC/游戏/PARITY 产品面
> 归 1071（UI-W1-AV 已入 §14）/1074/1075-1077/1068/985；Vue/Tauri 归 1072/1073。
> **PLAN-1072（Vue 与生产 TS/JS 接管，2026-11-19 已交付，见 §15）**：005/010 整例翻转、
> 组件面 `style` 替换 + `view-for` + 仅视图数据字段 + `expose` 点前缀对齐、`marshal_str_boundary`
> 统一 str 边界、`write_ace_vue_rt` 冷生成修复、`AC-LIF-07` 与 `n_assigns` 记账收口；
> `#6p`/包面多分支控制流重放及 `pages/`/`widgets/`/`stores/`/默认开关/`pnpm`+Vite+`auto run` CLI 闭环
> 记入 `KNOWN-DEBT-AND-RISKS.md`（与 PLAN-982 / 后继包分工）。
> 来源：[PLAN-980](../../plans/archive/980-ace-m2-autoui.md)（归档件）§5/§6/§9；
> 消费方：[PLAN-979](../../plans/archive/979-ace-m2-schema.md) r11 §5 V-11。
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
- **文本 literal 括号属性（PLAN-125）**。AGE 组件面和包面允许
  `text "literal" (style: "…") {}`。文本 literal 保留在文本叶序列，
  `text` 元素同时保留 label 和括号属性，供消费端投影。原有
  `text "literal" { style: "…" }` 继续通过同一文本叶和属性通道。

## 5. 边界与已知限制（未交付面保持显式）

- `style`/`setup`/`actions`/widget 级 `use{}` 四槽位在成员 span 具名拒绝
  （KNOWN-DEBT 登记，关闭条件在案）——具名拒绝即交付面，非静默缺口。
- `parse_ui` 为单文件语义：多模块工程源（`use store:` 等跨文件引用）独立解析
  报 `undefined variable`——完整批的语料验证按工程闭包入口执行。
- `ViewEvent.inline` 在 parse 输出上恒 `None`（内联 lambda parse 期铸造为
  `__evt` handler；分母行 Optional 并注记铸造去向）；插值文本在 002 形态归一为
  元素 `text` prop（`view.node.text.interp` 行同理）。
- 本切片不含窗口行为（NATIVE）与全模式联合覆盖（PARITY）。组件状态/事件执行
  已由 STATE 批交付（§6），但 DynamicComponent 产品默认链未切换（旧 bytecode
  链并行，退役归 PLAN-987）。
- **r11（2026-11-16 用户裁定）：JET/ARK 后端归 M3（v0.7），v0.6 不含**——
  980 的 B-JET/B-ARK 批与 V-B-JET/V-B-ARK/AC-B-JET/AC-B-ARK 随批迁移（原义务
  留计划 §9 历史）；PARITY 的 v0.6 联合覆盖=现役模式（NATIVE/WEB/RUST/TAURI）。
  WEB（Vue+TS）保留 v0.6（前置 L976）。
- f-string 文本叶（`${int}` → HIR `TextDisplay`）不可在 av.owned/1 车道执行：
  完整文本观察对 f-string 源具名拒绝；状态通道走 states-only（§6）。KNOWN-DEBT
  登记，关闭=profile/接线工作。
- emit/child→parent 路由未接线：属跨组件路由，归 981/IF-981-MODULE 多单元接线
  （KNOWN-DEBT）。

## 6. 组件面状态执行服务（STATE 批，2026-11-16）

PLAN-980 B-STATE-I/V/R 已验：组件形 widget 源的 model 状态与事件执行经
ACE（AGE Schema 场景脱糖 → 唯一共享 verify → AV 真值），替代旧
`handler_codegen` bytecode 合成路径**在本交付面上**的职责。收据见计划 §9
round 8/9（work/review 终核）。

- **states-only 场景编译**（ace-core `compile_component_scenario_states`）：
  发射状态函数与驱动、不发射文本叶求值函数（`instance_texts` 空行）。f-string
  叶的 `TextDisplay` 在 av.owned/1 不可 lowering（整模块 lowering 连带失败），
  状态通道不依赖文本叶——states-only 对 f-string 源可用；完整文本观察保持
  拒绝（`p980_state_fstring_view_leaf_states_observe_text_lane_refuses` 断
  `text-display`；复核反例断言 states-only 工件级无 `__age_c_text`）。
- **逐字段状态观察**（ace-av `observe_component_states` /
  `observe_component_scenario_states`）：按 `__age_state_<i>_<field>` def id
  执行，int/bool/str 全域映射（`ComponentFieldValue`）；与既有 `__age_drive`
  打包编码互补且一致（`p980_state_field_view_agrees_with_packed_driver`）。
- **每实例初值（props 种子）**（`ComponentInitials`/`InitialValue`）：mount 期
  覆盖源内 init（源与 `input_hash` 不变，`desugar_hash` 随初值变化）。拒绝面：
  未声明字段 `component-unknown-field`/类型不匹配与超 i32 域
  `component-unsupported`/单实例重复 `component-duplicate`/覆盖数错位
  `component-scenario`（`p980_state_initials_rejections_are_named`）。
- **scheduler 一次性执行入口**（ace-av `scheduler_run::run_task_source`）：
  968 调度器面（task/on、`Task.spawn`、`await`/`sleep`、虚拟时钟）的进程内
  公共 API——编译 → lower → MIR verify → ABC admit → `execute_sched` 到静止，
  stdout 由 ConsoleCapture 收集（UI 异步宿主；一次性运行，无跨调用状态）。
- **ACE 状态执行桥**（auto-ui-runtime `ui/ace_state_bridge.rs`，生产依赖
  ace-av）：`AceStateEngine`——keyed mount/unmount（世代递增）/晚到事件丢弃
  （`UnknownIdentity`/`InstanceReleased` 可区分）/重挂全新/dispatch 失败回滚
  （引擎不毒化）/props=mount 初值。**MIX-03 无 HTTP 版已验**：
  `spawn_async` 钉 (identity, mount generation)，任务先在 ACE scheduler 真实
  执行到静止（失败具名拒、不可伪造完成），完成回写只落在存活且同世代实例
  （`StaleGeneration` 丢弃对齐 CompletionPorts DropLate；HTTP 实腿归 983）。
  永久 case：`plan041_instance_ownership.rs` `p980_ace_state` 5 项（真实 002
  源；双 App 双实例隔离/store 形 handler/卸载-晚到-重挂/异步世代纪律/失败
  不伪造）。旧 plan041 4 case 不动（两路并存对照）。
- **已验输入**：examples/ui/002-counter 真源（sha256/16=04937c35325faa74，
  lang 侧冻结）经 `LegacyWidgetAlias::Accept` + initials 种子 + 定向 ops。
- **边界**（KNOWN-DEBT）：f-string 文本叶 AV 车道（上）；emit 路由与多单元源
  （host/store 内嵌跨引用）归 981/IF-981-MODULE；产品默认链未切换、旧
  bytecode 退役归 987；观测字段仍须 int（打包驱动口径不变）。

## 7. 首个 ACE AGE Node 文字窗口（HELLO 批，2026-11-16）

PLAN-980 T-HELLO-I/V/R 已验：AGE node 单 text 源 → 真实 Iced 主线程窗口
的全链（H0 共享 verify → AV 真值 → F0 受检调用 → `run_app` 泛型宿主渲染）。
收据见计划 §9 round 11/12。

- **有界决定**：窗口宿主选 `run_app::<C>` 泛型单窗链（View→into_iced 投影、
  Inter 字体、主题服务；ui_hello_loader 等 example 同链）；弃 daemon
  `run_session`（硬绑 DynamicComponent/旧 bytecode 链）。无第二传统接入线、
  无假 VmBridge。
- **桥（`ui/ace_hello.rs`）**：`HelloSession::load`（H0 面）产出 AV 真值文本
  （复制为 UI 所有 String）；`run_f0` **分派时重观察**——快照与新鲜 AV 真值
  逐字节一致（漂移具名红，T-HELLO-R 反例修复），同一文本字节经 967 冻结
  provider 的 `text_byte_len`（969 E0 门面 foreign 车道）证实被接收
  （callee=1/rejected=0）；幂等注册。
- **F0 面现实映射**：967 冻结 provider 计算文本事实不返回文本——窗口文本
  =AV 真值，F0 证同字节抵达，二者分立为已交付接口的既成事实。
- **F0 构建新鲜度缓存**（abi `rust_provider::build_source`）：按
  (provider digest, shim digest, rustc identity)+双产物在位跳过 rustc——
  显式预构建阶段的测试纪律支撑（篡改/缺产物/digest 变化即重建；永久 case
  `plan980_f0_cache`）。
- **真窗 exe（`examples/ace_hello_window.rs`）**：主线程 Iced 窗口 +
  `--prep-only` 预构建模式 + 帧计数/证据 JSON/自动退出（Process.exit 语义）。
  永久 case `tests/ace_ui_hello.rs` 7 项：CPU 面（SCHEMA/BODY/REJECT/
  LIFE 20 轮+teardown/漂移拒绝/坏源退出码）恒跑；窗口+FFI 面
  （`AUTO_UI_ACE_HELLO_E2E=1` 排窗，prep 后执行）：静态 text 与普通函数
  返回中文/ASCII 双 fixture、AV/F0/帧/cwd 断言。命令：
  `cargo test -p auto-ui-runtime --test ace_ui_hello -- --test-threads=1`。
- **fixture**：`examples/ui/ace-hello-minimal/src/front/{app,app-body}.at`。
- **边界**：完整 B0 打包闭包（发行包形态）pending（本批交仓外 cwd/预构建零
  rustc/缺前置具名拒绝最小面）；V-HELLO-LIFE 以 20 轮 CPU 生命周期+每证据
  1 真窗轮实现（进程内二次 iced 事件环被 PLAN-698 禁止；测试内 RSS 断言不稳
  定，具名）；AXR0 同源 A2R 腿归 RUST 批；不改产品默认链。

## 8. 同源 A2R 先行切片（RUST 批 AXR0，2026-11-16）

PLAN-980 B-RUST-I/V/R 已验：同源 AGE 组件经**新 A2R**（902/979 面）生成
Rust → 真 rustc → 真运行 → 窗口派生显示；旧 AST emitter 未替换（边界在案）。
收据见计划 §9 round 14–16。

- **Leg A（同源语义）**：`examples/ui/ace-hello-minimal/src/front/app-a2r.at`
  （node+model+on+单 text）→ 组件场景 A2R 文件集（`ui/ace_a2r_bridge.rs`
  `component_a2r_fileset`：场景编译→SemanticView→lower `__age_drive`→
  record-text 三文件+manifest）→ 构建道真 rustc → `value:2`（冻结场景
  2×Inc）。改 scenario/handler/初值即改产物字节（发射身份 oracle）。
- **Leg B（F0×A2R，967/T-12 桥）**：同一文本字节经 text-record 源 →
  文件集（provider `#[no_mangle]` 符号 + extern 块）→ rustc 链接**预构建**
  F0 provider（新鲜度缓存，零 rustc 于测试）→ `value:9`（"Hello A2R" 字节
  数）——同字节经 provider 的 A2R 侧证明。
- **构建道纪律**：真 rustc 只在显式构建道 `examples/ace_a2r_axr0_build.rs`
  （一次性，回执 `target/ace-a2r-axr0/receipt.json`）；日用例（
  `tests/ace_a2r_axr0.rs` 4 项）只做发射控制+回执消费+排窗窗口腿。
- **窗口腿（V-UI-AXR0）**：`ace_hello_window --a2r-run <exe>` 派生 A2R 产物
  exe、显示其真实 stdout（"A2R value:2"）——零 AV/旧 VM；改函数结果随变
  于 drive 值层。
- **ace-bridge feature 门（结构性）**：STATE/HELLO/RUST 的 ACE 桥生产依赖
  经 `ace-bridge`（默认关）optional 化；交付投影（rust_ui）剥 7 optional
  行+dev ace-av+feature 改写为空（autodown-core 同型先例）——792 受管交付
  闭包保持。AXR0/HELLO/STATE 测试命令带 `--features ace-bridge`。
- **交付闭包修复（driver 43/43 全绿）**：①DIALECT 批 ace-core dev-dep
  越根 → 投影 `strip_dev_deps`（新引擎面，三态幂等）+ compiler 专用 spec；
  ②spec 须全行形态（裸名键取空误入已剥离分支——单测实证）；③bindgen 的
  ace-foreign-contract（967/973 谱系预存）→ DELIVERY_VENDOR_TREES 补
  foreign-contract 树。原工程 runner（PLAN-792 driver V01..V06）真实执行
  **43/43 PASS**（V01 fresh-lock/V02 重定位 locked/V03 负控/V06 全 profile
  矩阵+vm-run；jet/ark 环境门控为 driver 原语义）。
- **边界**：产品级 `rust_ui.rs` 旧 emitter 未替换（多文件 store/组件/
  sidecar/props/msg/route 对齐待 981 面接线，KNOWN-DEBT）；文本不可经 A2R
  出岛（902 冻结 i32 `value:` 协议）——窗口文本腿以 A2R stdout 上窗实现；
  ui_gen::rust 2 例 master 预存红（台账）；AXR0-PACK 完整打包闭包 pending。

## 9. ACE 原生 run 通道（NATIVE 批，2026-11-16）

PLAN-980 B-NATIVE-I/V/R 已验：001 形真源经 ACE 面编译 + AV 真值 + 现有
View→Iced 窗口服务的**原生 run 通道**；`AUTO_UI_ENGINE=ace` 引擎缝接入
`-r vm` 流（旧 vm 通道不变）。收据见计划 §9 round 19–21。

- **免驱动场景编译**（core，980 入口专属）：observed 缺省且 model 无 int
  字段（001 形：str/bool + 文本叶）时不发射 `__age_drive`，texts/states 照
  发；979 冻结打包入口（`compile_component_scenario`/`_observing`→
  `observe_compiled`）保持编译期具名拒（口径不变，p979 16/16 回归实证）。
- **av 面**：`observe_component_states_and_texts`（逐字段 int/bool/str +
  文本叶 Str 真值双观察）+ `project_component_candidates` 再导出（view 树
  消费）。
- **UI 通道**（`ui/ace_native.rs`，feature=ace-bridge）：`AceNativeSession`
  ——001 真源（legacy `widget` 头 Accept 别名）装载：文本叶值（AV）+ 属性
  （style/selectable 经候选投影 view 树按声明序对齐）+ 状态；**状态写**
  =按当前状态重建初值→重编译→重观察（001 的外部 state 写语义），
  `dark_mode` 写翻转窗口主题线程局部（`style_iced_adapter`，与旧链同
  机制）。窗口=`run_app` 泛型宿主（`examples/ace_native_window.exe`：
  帧内状态写/证据 JSON/自动退出；关闭重开=顺序多轮进程级窗口）。
- **引擎缝**（run_vm）：`AUTO_UI_ENGINE=ace` → `run_ace_native_entry`
  （无条件编译，feature 关=具名拒）；出面外源 ACE 面具名拒，不静默回旧 VM。
- **永久 case**（`tests/ace_native_window.rs` 5 项）：001 装载（selectable/
  style 入叶）、状态写主题翻转（CPU 可观察）、出面外具名拒、双叶属性对齐
  反例、排窗 e2e（真窗+帧内写+两轮关闭重开）。命令：
  `cargo test -p auto-ui-runtime --features ace-bridge --test ace_native_window -- --test-threads=1`（窗口面 AUTO_UI_ACE_HELLO_E2E=1 + prep 构建 exe）。
- **V 行点名门**：desktop_behavior 11 过/1 ignored（原条件 P488-D1）+
  image_surface_contract 1/1 不动绿；`auto run -r vm` 隔离工程身份=driver
  V06e 真跑证据复用（vm-session 子进程=本工具 exe，MCP/cancel 全过）。
- **边界**：文字选择/复制、IME、媒体、终端的**真实交互**实测属仓内既有档
  （selectable_text 单测族+手工/环境门控档；宿主适配器安装面已随通道接）；
  路由/页面切换的组件面运行语义未交付（001 无路由；归 981/后续批）；
  DynamicComponent 默认链未切换（沿 STATE 批既定边界）。

## 10. 分母与命令冻结（T-01，2026-11-16，r11 口径）

v0.6 面积（29 任务）的分母/来源/命令/oracle 单表——各批实测冻结，后续批按
此备料开工；漂移即修本表。**r11 裁定后 JET/ARK 归 M3（v0.7）不在本表。**

### 10.1 批次×状态×冻结命令

| 批 | 状态 | 冻结命令（实测） | 输入源（冻结身份） |
|---|---|---|---|
| DIALECT | ✅ 落主 | lang `cargo t ace-source ace-syntax`；ui `cargo test -p auto-ui-dialect --lib` + `-p auto-ui-compiler --test ui_domain_validation` | 002 真源 sha16=04937c35325faa74；ast.rs=1d10f5ff2566422e |
| STATE | ✅ 落主 | lang `cargo t ace-source ace-av ace-host`（ace-host 预存红具名排除）；ui `cargo test -p auto-ui-runtime --test plan041_instance_ownership --test plan033_b1_scope_isolation -- --test-threads=1` | 002 真源（同上）；store 形 fixture（内联冻结） |
| HELLO | ✅ 落主 | `AUTO_UI_ACE_HELLO_E2E=1 cargo test -p auto-ui-runtime --features ace-bridge --test ace_ui_hello -- --test-threads=1`；prep=build example + `--prep-only` 预构建 F0（target/ace-hello-f0 缓存 stamp） | ace-hello-minimal/{app,app-body}.at（仓内 fixture） |
| RUST(AXR0) | ✅ 落主 | lang `cargo t ace-a2x generated-delivery-controls`；ui `--features ace-bridge --test ace_a2r_axr0`；构建道=`ace_a2r_axr0_build.exe` 一次性（回执 target/ace-a2r-axr0/）；driver=PLAN-792 全量（43/43） | app-a2r.at + "Hello A2R"=9 字节；F0 冻结 provider |
| NATIVE | ✅ 落主 | `--features ace-bridge --test ace_native_window`（窗口面同 env 门控）；V 行点名=desktop_behavior+image_surface_contract 原样；run 缝=`AUTO_UI_ENGINE=ace` | 001 真源（examples/ui/001-helloworld） |
| WEB | 待做（976 已归档解锁） | 见 §6 V-B-WEB 行（cargo tu 单进程 + 真实浏览器挂载）；I 冻结 ts provider 消费命令时更新本表 | 待 I 冻结（vue 例程源） |
| TAURI | 待 WEB | 见 §6 V-B-TAURI | 待 I 冻结 |
| PARITY | 待 NATIVE✓/WEB/TAURI；**源面（V-UI-SOURCE）+NATIVE 腿可先行** | 见 §6 V-B-PARITY + V-UI-SOURCE | 001/002 + 覆盖表（V-UI-SOURCE） |

### 10.2 模式网格（r11：v0.6=NATIVE/WEB/RUST/TAURI）

| 格 | 状态 |
|---|---|
| NATIVE | ✅（001 真窗/状态写主题/关闭重开；交互档=既有手工/环境门控档） |
| RUST | AXR0 ✅；多文件 emitter 替换待 IF-981-MODULE 消费（已落主 e352e462b） |
| WEB | 976 provider 已归档（660fe72b5）——适配器切换归本包 WEB 批 |
| TAURI | 传递依赖 WEB |
| JET/ARK | M3（v0.7）——r11 裁定 |

### 10.3 共享纪律（冻结）

- 前门短窗单写权（979 grammar/schema 共享文件）逐批核取；981 executing 中，
  消费其面时按 979 规约。
- F0 provider 构建走 target/ace-hello-f0 新鲜度缓存（预构建后测试零 rustc）；
  真编译一律显式构建道（978 裁定）。
- 排窗：`AUTO_UI_ACE_HELLO_E2E=1` 单实例串行；GPU/媒体档沿仓内既有门。
- 预存红台账（KNOWN-DEBT）：ace-abi source_foreign 25、ui_gen::rust 2、
  ace-host 13——不判 980、不充 PASS。

## 11. WEB 首切片：976 provider 的 UI fragment 消费（2026-11-16）

PLAN-980 WEB 批首个已验切片（B-WEB 任务保持开放）：生产表达式形态经
**976 TS/JS provider**（fragment 接口）发射+执行+宿主映射成立；完整适配器
切换（ts_adapter 四处旧 TypeScriptTrans 委托替换 + 全 UI 合法表面 + 浏览器
挂载档）为后续步。收据见计划 §9 round 23。

- **桥（`ui/ace_ts_provider.rs`，feature=ace-bridge）**：UI 表达式/语句
  **源文本** → 合成包装函数（core 面：int 域；宿主状态=参数/`var` 可变局部；
  体含 return 不补尾——HIR 块图连通）→ `ace_core::source::compile_source`
  （与 976 tsfrag 同一前门）→ `SemanticView`（按源名取函数事实）→ 976
  `emit_{expr,stmt}_fragment`（checked-helper 模式：操作数住 helper 行、
  表达式本体=临时引用；`a2x_*` helper 义务首用序报告）→ **宿主映射**（两跳：
  发射名→源名→reactive 宿主名，如 `count`→`count.value`——词法整词替换；
  976 合同的反应性/捕获/宿主变量职责在适配器）。
- **真 node 验证**：`run_fragment_on_node` 拼宿主驱动（helper 导入+预声明
  宿主变量+fragment 行+断言）跑真 node（FRAG_OK 协议，`A2TS_NODE` 覆写）。
- **永久 case**（`tests/ace_ts_provider.rs` 4 项）：绑定表达式（count+1→
  count.value+a2x_add+node 42）、赋值语句（var sum 可变局部→node 42）、
  未定位/越界=具名拒绝、源类型错误带 ACE 码透传。命令：
  `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ts_provider -- --test-threads=1`。
- **边界**：str/结构形态与 watch/setup 回调形状未接（core 面 int 域+
  fragment 语句面限制——完整合法表面需 text-record/collections 面扩展或
  适配器组合）；ts_adapter 生产委托未替换；浏览器挂载档未建。**无旧
  emitter 回退**：新车道未覆盖形态=具名拒绝。

## 12. MIN-A-UI 通用 Node 桌面入口（NATIVE r13 批，2026-11-16）

PLAN-980 B-NATIVE-I/V/R（r13 重开范围）已验：通用 Node 项目入口 +
**实际 UI-FFI 效果** + **typed 事件回 AV**。收据见计划 §9 round 24–26；
r11 §9 的快照窗口面保留给自己的测试/exe，但 `AUTO_UI_ENGINE=ace` 产品缝
自本批起指向 `run_ace_node_desktop_entry`（§9 引擎缝行为由此取代）。

- **ui980 provider**（`ui/ace_node_desktop.rs`，hooks trampoline）：依赖
  无关注解源经 `build_source` 构建（root=target/ace-node-desktop，新鲜度
  缓存；lang 侧通用注册面 `auto_lang_ace_host::register_source_provider`）。
  **967 冻结 AV 外呼可调用域 = view + 2×i32（`ViewI32I32ToI32`）**——7 函
  数全部 `(view,i32,i32)->i32`：`ui_open`（tag+instances）、
  `ui_apply_text/ui_apply_str`（view 即载荷，instance\*1000+slot 打包，当前
  session）、`ui_apply_int/ui_apply_bool`（tag+打包+值，当前 session）、
  `ui_poll_event/ui_close`（tag+显式句柄）。具名状态码：-1 未知句柄 /
  -2 过期或未开 / -3 越槽 / -4 坏字节 / -5 错车道标签 / -100 hooks 未装。
  宿主与 AV 经同一 DLL 模块（LoadLibraryW 引用计数）共享 statics；UI
  session 状态（显示模型/typed 队列/世代/证据计数）归 auto-ui-runtime。
- **事件回 AV（业务全在 AV）**：窗口点击→typed 队列（世代纪律）→tick
  受控调用点 AV 外调 `ui_poll_event`（事件编码 (instance+1)\*1000+
  (target+1)）→`AceStateEngine::dispatch`（§6 场景重放）→状态差量经 AV
  外调 `ui_apply_*` 提交实际 UI 更新→Iced（`run_app` 泛型宿主，tick 订阅
  驱动）。`text_byte_len` 不是 UI dispatch（判别力反例已固化）。
- **三类源**：ace-hello-minimal 双源（字面量/函数中文）+ `examples/ui/
  ace-node-counter/src/front/app.at`（002 计数业务派生 node）。文本面两
  段：driverless-full（带 model/on）→静态观察面（无 model 的 Hello 类具
  名回退；静态面拒 `on`=具名）。
- **产品缝**（run_vm）：`AUTO_UI_ENGINE=ace` → `run_ace_node_desktop_entry`
  （feature 关=具名拒；r11 `run_ace_native_entry` 保留为快照面 API）。
- **永久 case**（`tests/ace_node_desktop.rs` 8 项 + 排窗 e2e 1 项）：轨迹
  0→1→0→0 与两实例定向 2:1、hello 双源字节级 FFI、坏 Schema（无绑定
  target/未声明 msg）、坏 ABI（未知句柄/越槽）、close/reopen/late 世代
  纪律、byte_len 反例、撤权/复权、车道标签+跨 session 队列隔离+双重
  close（复核 authored）。命令：`cargo test -p auto-ui-runtime --features
  ace-bridge --test ace_node_desktop -- --test-threads=1`（排窗
  `AUTO_UI_ACE_NODE_E2E=1`，prep=构建 example+`--prep-only`）。
- **边界**：f-string 文本叶在两条文本车道均具名拒（沿 §5 既有债务；计数
  值显示走 int 状态槽呈现）；真实鼠标/IME 交互属手工/环境门控档（沿 §9）；
  打包位为 MIN-A 界内约定（实例 ≤4、槽 <1000）；完整 001/002 主题/全部
  组件/Widgets 归 W1 B-AV。

## 13. WEB 切片 #2：fragment 车道覆盖普查（2026-11-16）

PLAN-980 WEB 批切片 #2 已验：类型域参数（int/bool——str 被 profile 拒）
+ **车道覆盖普查冻结表**（`tests/ace_ts_provider.rs::
p980_web_lane_census_frozen_forms`，11 形态逐项 Ok/具名拒）。收据见计划
§9 round 27。

- **关键事实（切换前置义务）**：976 fragment 车道经 `compile_source`
  （**Core profile**）编译——**比较算子（`>`/`==`）、布尔算子（`!`/`&&`）、
  `str` 类型、f-string 插值、多函数源均不在 profile 内**（具名码
  `source.unsupported`/`source.lex`/`source.syntax`）；非字面 var-init
  （参数引用/表达式初始化）触发 `verify.block-structure` 拒（O-C2 形状
  缺口——字面初始化的两步赋值形态 OK）。今日 OK 面=字面 init 两步赋值/
  return 表达式（int 算术）+宿主 reactive 映射+真 node 执行。
- **推论**：ts_adapter 委托替换的实底不是（只是）AST→HIR 线程化——
  生产 UI 表达表面（比较/布尔/str/插值/调用）**先须扩 fragment 编译
  profile/入口**（976 后继或 richer entry），普查表即该义务的可测分母
  （表变化须具名修订，防静默漂移）。

### §13.1 入口扩域评估结论（切片 #3，2026-11-16）

**组件面 = fragment 扩域可行入口（机制已钉）**：生产形态源（`!` 一元/
str/bool 字段——Core profile 具名拒的形态）经组件面 states-only 编译 →
`SemanticView::from_checked` → 976 fragment 发射器可运行（`p980_web_
component_face_entry_fragments`：span 定位→发射→宿主映射→真 node 读值
FRAG_OK）。**形态=脱糖调用链**：handler 内 model 访问脱糖为
`__age_state_<i>_<field>()` 调用——fragment 为调用形，宿主映射面=
"状态函数调用 `fn()` → reactive 值"（吃括号替换，测试已固化）。**O-C3**：
脱糖 drive 的 span 精度（`!.on` 源段定位落到 count 调用——PLAN-964 span
移位同族边界）：精确 per-handler 定位=适配器接线（切片 #4）前置。结论：
WEB 完整切换不需要 976 后继扩 profile 前置——组件面入口路线成立；剩余=
span 精度/逐 handler 定位 + ts_adapter 接线 + 全量切换 + 浏览器档。

### §13.2 逐 handler 精确定位（切片 #4，O-C3 解除，2026-11-16）

`ComponentFragmentLane`（`ui/ace_ts_provider.rs`）：生产形态源 → 组件面
states-only 编译（ops=**待定位 handler 的 msg 重放**——ops 空则 handler
体缺席状态函数，切片#4 探针实证）→ 状态函数 HIR 的 expr **span=原源坐
标** → `emit_expr_at_span(fn, span)` 精确定位发射。**同 span 多表达式取
索引序最大者=组合形**（`!on` 的 NOT 与纯读形同 span，NOT 在后——
`(!l1___age_s_0_on)` 发射实证）；host 映射（`l1___age_s_0_on`→reactive
值）后真 node 取反执行（!false===true）。逐 handler 区域可分辨（Inc 的
`1` vs Toggle 的 `!on`）；越界 span/无命中/未知状态函数=具名拒。永久
case `p980_web_handler_precise_location`。**WEB 剩余**=ts_adapter 四委托
点接此 lane（源 span 从 widget 提取管线取）→ 全量切换 → 浏览器档。

### §13.3 lane 消费真实生产源（切片 #5，2026-11-16）

`ComponentFragmentLane` 坐标基准明确为 **f-string 脱糖后源**（组件面入口
统一脱糖，`ace_core::source::fstr::desugar_fstrings_widget` 公共出口；
无 f-string 源时与原源一致）——lane 暴露 `desugared_source()` 并提供
`emit_expr_at_text`（文本锚包含定位，同 span 组合形优先）。**真实生产
002 全源**（msg 6 变体/model 3 字段/design/f-string/view 内联 lambda）
经 lane 端到端：ToggleMode 的 `!.dark_mode` NOT 组合形（`l2___age_s_0_
dark_mode` 局部名，字段序即局部序）按脱糖源 span 精确定位→宿主 reactive
映射→真 node `!false===true`；Inc handler 同 lane 在 count 状态函数定位
（002 的 `.count += 1` 首现于 view 内联 lambda——**on 块 occurrence 推导**
是适配器配方）。永久 case `p980_web_lane_consumes_production_002_source`。
**WEB 剩余**：OnHandler 方言层 span 捕获 + LogicPayload 源区间线程化 +
ts_adapter 四委托点接线 → 90 项全量 → 浏览器档（多会话）。

### §13.4 提取携带 span → lane 接线（切片 #6a，2026-11-16）

方言 `OnHandler` 增源 span（`#[serde(default)] (byte_offset, byte_length)`
**原源坐标**；on 块解析捕获，view 内联 lambda 铸造形 None）——DIALECT 字段
清单登记 `widget.on.handler.span`（002 计数 6：声明形带、铸造形缺）。
`extract_handler_spans(decl)`（compiler，零涟漪独立面）→ 真实解析管线
（CompilerSession+Parser+extract）→ 提取区间核验 → lane 区间内文本锚 →
NOT fragment：永久 case `p980_web_extraction_carried_spans_feed_lane`
（ace_ts_provider 9/9）。**O-C4**：span 起点落 pattern 中段小偏移（终点
正确、body 完整）——#6b 校准。**观察**：vue_capabilities 10 红=master
预存（stash 取证，疑 981 邻面；KNOWN-DEBT 在案）。**WEB 剩余**：span
起点校准 + 生成器源通道 + ts_adapter 四委托点接线 → 90 项 → 浏览器档。

### §13.5 O-C4 解除：span 语义=body 覆盖区间 + 接线设计决策（切片 #6b，2026-11-16）

**O-C4 解除（重定义为语义而非缺陷）**：全 6 handler span 诊断实证——
`OnHandler.span` 的捕获语义是 **body 覆盖区间**（起点在 pattern 末 token
至 body `{` 之间（±1 token，随 cur_pos 快照点波动），终点=body 闭括号
后；`.ToggleMode` 含 on 块尾 `}` 的 1 token 余量）。**lane 合同只需
expr 包含性**——对全部 handler 成立（002 六例核验）。精确 pattern 起点
不承诺、不需要。

**四委托点接线设计决策（双路线，#6c 起实施）**：
- **路线 A（表达式级，受阻）**：ts_adapter 两处 Expr 委托点直接换 lane——
  **受阻于旧 AST 无 span**（auto_lang_core::ast 的 Expr/Stmt 枚举零位置
  信息；方言 parse_body 产出旧 AST；为每 variant 加 span=语言侧大手术）。
  文本回印定位需 Auto AST printer（不存在）。不采用（本轮裁定）。
- **路线 B（handler 体级，采纳）**：script 生成器按 handler 携
  **源文本 + extract_handler_spans 区间**整经 lane 发射——状态函数 HIR
  内该区间中的 exprs（原源 span）即该 handler 的表达式集，逐 expr 发射
  +宿主映射组装 handler 函数体。改动面：vue/script.rs handler 生成道 +
  VueGenerator 源通道（`generate_with_source`）——UI 侧自有面，无语言侧
  手术。**剩余**：路线 B 实施 → vue_capabilities 全量双轨 → 浏览器档。

### §13.6 路线 B 核心：lane_handler_bodies（切片 #6c，2026-11-16）

`ui/ace_vue_script.rs`（feature=ace-bridge）：`lane_handler_bodies(src,
path, patterns)` → 每 handler 的 Vue script 函数体行（reactive 语义）。
算法：lane（ops=去点 msg 重放）→ pattern**带箭头**锚定位体区间（view 内
联引用无箭头——排除歧义）→ **写目标=体文本 LHS 首个 `.field`**（所有状
态函数含全部 handler 的重放表达式，span 包含性不可区分写路径——具名边
界：单赋值形 002 族已验，多语句/控制流形体归 #6d 结构分析）→ 区间最宽
RHS 发射 → 宿主映射（`l<i>___age_s_<inst>_<field>` → `field.value`，剥
实例段）→ `[helper 行..., field.value = rhs;]`。永久 case
`p980_web_route_b_handler_bodies`（ace_ts_provider 10/10）：002 五
handler 体全派生（Inc=加、ToggleMode=取反、UseAuto=字面量），**Inc 体
真 node 执行 41→42**（a2x_rt helper 就位）。**WEB 剩余**：SFC 拼接
（#6d）→ 全量双轨 → 浏览器档。

### §13.7 SFC 拼接（切片 #6d，2026-11-16）

`splice_lane_handler_bodies(sfc, bodies)`（`ui/ace_vue_script.rs`）：lane
派生体替换编译器 SFC 内对应 handler 函数体——锚=`function <Variant>():
void {` 括号配对；pattern 去点=Variant；只动 handler 体，**锚外段落字节
不变**（模板/状态声明区前后对账断言）。永久 case `p980_web_sfc_splice`
（ace_ts_provider 11/11）：002 真实编译器管线（parse→extract→
VueGenerator SFC）+ 六 lane 体 → splice——Inc/其他 handler 体替换、
template 与 pre-state 区 byte-identical、ToggleMode 取反体就位。
**WEB 剩余**：`generate_sfc_with_lane` 一体化入口 + 90 项全量双轨 +
浏览器挂载档。

### §13.8 一体化入口 + 覆盖普查 v2（切片 #6e，2026-11-16）

`generate_sfc_with_lane(src, path)`（`ui/ace_vue_script.rs`）：源文本 →
真实编译器管线 SFC + lane handler 体拼接，一调返回 `(sfc, bodies, absent)`
——**absent 中的 handler 保持编译器旧道体**（双轨期合同：缺席不拒绝）。
**覆盖普查 v2 冻结表**（`p980_web_unified_entry_census_v2`，代表性能力族
语料，2026-11-16 实测）：msg 引用赋值族全覆盖（counter-3h=3 spliced、
str 字面量=1）；无 on 块/watch-only=passthrough（0 spliced）；
**参数化 handler（`.Add(t) ->`）= lane 缺席，旧道保持**（#6f 扩面：参数
进入 lane 体）。ace_ts_provider 12/12。**WEB 剩余**：参数化 handler 扩面
→ 90 项全量双轨 → 浏览器挂载档（013/017 playwright 先例）。

### §13.9 参数化 handler 域缺口具名（切片 #6f，2026-11-16）

**结论**：参数化 handler（`.Add(t) ->`）与内联 lambda 铸造 handler 的 lane
覆盖**不是 UI 适配器可闭合项**——根因=组件面 `ScenarioOp::Send` **无
payload 字段**（参数化事件不可场景重放，状态函数 HIR 无从携带参数）+
铸造形无源锚。属**语言侧后继义务**（979/981 谱系扩展 ScenarioOp 或参数
绑定面）。双轨期合同下二者永远走编译器旧道直至语言侧交付。普查 v2 冻结
表已扩两行（param-handler/inline-lambda-only=absent；ace_ts_provider
12/12）。**WEB 剩余（UI 侧）**：90 项全量双轨（普查表=分母仪表）+
浏览器挂载档（013/017 playwright 先例）。

### §13.10 全量双轨普查 + 语言侧前置清单（切片 #6g，2026-11-16）

**双轨普查冻结聚合表**（`p980_web_full_dual_track_census`，ace_ts_provider
13/13）：vue_capabilities 全部 83 源语料（文件级 r#"…"# 提取）经
`generate_sfc_with_lane`——ok=72 / err=11（2 store + 9="duplicate view
block"族=**master 预存红同族**，归 triage）/ spliced=0 / absent=23。
**state-slot 同长归一化**（`state {`→`model {`，5↔5 字节 span 零位移）
已入 lane compile——匿名 msg+state 形态可进 lane。

**语言侧前置清单（组件面语法域缺口×3——非 UI 可闭合，979/981 谱系）**：
①`ScenarioOp::Send` 无 payload（参数化事件不可重放）；②具名 msg 语法
（`msg Msg {...}`）不在组件面槽位语法——真语料主导形态；③参数绑定面
（铸造 lambda handler 无源锚）。三者交付前，真语料 handler 体全部走编译
器旧道（双轨期合同——缺席不拒绝）。**WEB 批 UI 侧状态**：机制链全通
（组件面→HIR→span 定位→发射→reactive 映射→函数体→SFC 拼接→一体化
入口→双轨仪表），交付切换的语言侧前置已具名；浏览器挂载档待做。

### §13.11 浏览器挂载档（切片 #6h，2026-11-16）

`examples/ui/ace-web-lane/tests/test_lane_browser.mjs`（013-todo 先例：
playwright 解析→无头 chromium→DOM 断言）+ 永久 case
`p980_web_lane_browser_mount`（排窗 `AUTO_UI_ACE_WEB_E2E=1`）：002 的
lane 派生 Inc handler 体在**真实浏览器** reactive 宿主（`{value, notify}`
对象 + DOM 更新回调）中执行——点击按钮 → `a2x_add(count.value, 1)` →
`count.value = t0` → DOM 文本 41→42。a2x_rt helper 内联（ES module 的
export 语句剥除/checked-add 语义按 rt_body.js 冻结式手写——`<script>`
非 module 环境限制）。ace_ts_provider 14/14。**WEB 批 UI 侧全部项已交**
（机制链 + 双轨仪表 + 浏览器档）；真语料 spliced 切换待语言侧前置×3
（§13.10）。

### §13.12 参数环境 + 内联 lambda origin：981 B-W1 消费（切片 #6j/#6k，2026-11-17）

981 B-W1（IF-981-EVENT-PAYLOAD-B/MSG-B/PARAMS-B 落主）后，WEB lane 消费
三接口恢复并扩展带参/内联 lambda 覆盖：

- **param handler=声明级检查函数道**（#6j）：`Send` 对载荷变体是 arity 域
  错误（981 语义）→ ops 只含 unit handler；带参 handler 的发射源=组件面
  为每个 handler 铸的 `__age_hv_<n>_<h>_<pos>` 检查函数（形参=真 handler
  形参，载荷类型来自声明拼写）。lane 以 span 相交发现（`check_fn_covering`），
  发射后 `___age_hv_` 发射名按位映射到 SFC 函数签名形参名（`p<i>___age_hv_
  <n>_<h>_<pos>` → `params[pos]`）。无代表值、无假数值：str 载荷 owned
  转移=`a2x_str_clone`（单次求值拷贝，PARAMS-B 语义）；多载荷
  `(int,int)`=双参参与求值（`a2x_add(dx, dy)`）；bool 直读；float 等不可
  承载载荷不铸检查函数=具名缺席（fail-closed，981 派发守卫同族），旧道承
  载。无参数环境（unit spec 求导载荷变体）=编译期具名拒（arity 域错误，
  零效果）——不再像 981 前拼接常量体（"spliced=1 不等价动态载荷支持"的
  判别由 `p980_web_param_environment` 分列承担：typed/fragment/splice/
  node + 双负控）。
- **view 内联 lambda=值函数道**（#6k，V-EVT-B-ORIGIN 与 981 LANGUAGE 联
  合）：语言侧（`age_component.rs` 组件面）为 view 树（含命名 view）的
  每条 `Lambda(assigns)` 铸 `__age_vl_<seq>` 值函数（DFS 序；字段 init
  let + 赋值体（`.field`→实例局部，无参环境）+ return 首字段；赋值 RHS
  **原源 span** 保留；`__age_vl_` 入保留名表）。UI 侧：脱糖源 `=> {` 区
  间清单（`lambda_body_regions`，`desugared_lane_source` 公共坐标出口与
  lane 同基准）↔ 带体 `__evt_*` 铸造 handler（方言 mint 计数序；空体=
  bare-input sync/空 lambda 不配对）——计数失配全体缺席（fail-closed，
  不猜测配对）。SFC 合成 `__evt_*` 函数体经 lane 拼接（真实 body/origin：
  原 span→值函数→发射→SFC 一一可追）；负控：拼接体不得是旧 TS fallback
  形（`n.value += 1`——lane 形=`a2x_add` helper）。002 真源端到端 9/9
  （6 on-block + 3 view lambda 全 lane 派生）。
- **lambda 体结构化（PLAN-980 lambda-if 刀，2026-11-19）**：值函数道从
  assigns-only 扩为**结构化语句面**——`Lambda` 投影携带 `stmts`
  （Assign/If 序保持；`assigns`=深平铺兼容面；调用形仍 foreign 不铸，
  #6p 合同不变）；`lambda_value_fn` 对 `If` 递归降版 `Stmt::If`（分支
  独立、view-stmt 方言无 let=无 scope 遮蔽；cond 与分支 RHS 原源
  span），HIR→a2x-ts `if/else` 文本经 `stmt_fragments_in` **If 整语句
  发射**（on-if 刀 e 环机械复用——分支赋值不裸喷成无条件序列、条件不
  丢；UI 侧零产品代码改动实证）。深验证面：通道值位置拒与 frontdoor
  绑定收集覆盖 If cond+分支 RHS（`walk_stmt_exprs` 模块级共享）。
  **零文法改动**（grammar-id 零漂移，ace-source 冻结面实证）。永久用
  例：core `plan980_lambda_if`（5：结构化投影/序保持/else-if 链/嵌套/
  调用形 foreign）、av `plan980_lambda_if`（4：铸造可判别 `__age_vl_0`
  /分支未声明字段拒/cond 未绑定名拒/调用形不铸负控）、UI
  `p980_lambda_if_lane`（区间发现+If 整语句片段+a2x_add 判别器+缺席
  负控）+浏览器排窗档 `p980_lambda_if_browser_mount`（toggle 两分支
  真实走到）。005 语料归因：str `!=` 条件铸造绿；整例缺席=use/style/
  design 既名墙连带（剥离墙片段 3/3 派生）。
- **浏览器档**：`test_lane_browser_params.mjs`（真实 input 事件实读参/
  多载荷点击/bool 点击）、`test_lane_browser_lambda.mjs`（真实点击
  5→6）与 `test_lane_browser_lambda_if.mjs`（toggle 两分支真实走到）
  ——playwright 无头 chromium，排窗 `AUTO_UI_ACE_WEB_E2E=1`。
- **census 具名修订**：v2 表 #2（param-handler 0→1 恢复拼接 + param-read/
  float-payload 行）、#3（inline-lambda-only 0+absent→1）与 #10
  （2026-11-19 lambda-if 行新增——if 体 lambda 翻转为结构化铸造+If 整
  语句 lane 发射；005 族，判别力=`p980_lambda_if_lane`）。全量普查聚合
  (83 源 ok=72/err=11/spliced=0/absent=23) **未变**——真语料阻塞形非
  param/lambda（其主导形=无 on 块结构源/多语句体/store/computed 族，#6d/
  #6g 具名边界保持）。
- **边界（诚实）**：单赋值 handler 家族（002 族）为已验面；多语句体/控制
  流/局部 var 归 #6d 结构分析；真语料全量 spliced 切换仍是 WEB 批剩余义
  务（B-WEB-I/V/R 勾选前置）。981 冻结 hash 已由其 owner 重基线（grammar
  零改动，#6i 语义差注明）——r41 两红已闭，非 980 面。

### §13.13 V-B-WEB 处置：text-leaf 门放宽 + state 别名 + 多赋值发射 + census taxonomy（切片 #6l，2026-11-18）

- **text-leaf 形状门只约束文本观测车道**（lang `age_component.rs`：`emit_view_texts=true` 才要求 view 含 `text` 叶）——states-only 车道（状态/检查/值函数，无文本叶发射）不依赖文本叶；button/input-only 真实工程视图（vue_capabilities 主导形态）经 states-only 可编译可拼接。完整文本观察车道保持原门。
- **方言 `state` 槽位别名=model**（auto-ui `declarations.rs`/`grammar/widget.rs`）：与 lane/ACE 侧 #6g 同一别名在方言层落定。此前 `state {}` 落 widget 体 fallback 臂被误解析为**视图元素**，真 `view {}` 随之伪报 "duplicate view"——vue_capabilities parse 族 3 预存红（cap_unused_handler/cap_null_init/cap_event_arg，898-N5 时代已知债）的根因。`model`+`state` 并存仍按重复拒绝。ACE 语法零改动（UI 侧归一化承载不变——无 grammar-id 扰动）。
- **多赋值逐语句发射**（lane `stmt_fragments_in`）：按 Assign RHS 表达式 span⊆handler 区间选取、语句序=源序、`emit_stmt_fragment` 逐句发射。HIR Stmt 无 span（按 value 表达式 span 过滤——字段 init/return 的 span 在区间外天然排除）。**语法层只收平铺 `.field (=|+=|-=) expr`**（`age-view-stmt`）——控制流/let/调用在语法层被拒，**不可能被静默展平**（错误语义防线）。片段临时名按片段改号（`t3`→`t{k}_3`）防多片段同域 `const t0` 撞声明（片段文本里裸 `t<digits>` 只可能是发射器临时名——其余名字族走 `l<i>_*`/`p<i>_*`/宿主映射名）。
- **976 片段接口缺口（#6l 实测）**：owned 局地的赋值/返回语句片段引用函数域簿记变量 `a2x_ret_owned_skip`，其 `let` 声明只在整函数发射路径——片段文本裸引用。宿主边界统一补声明（引用才补）。同族：`a2x_str_free` 等簿记 helper 由宿主按需内联（JS-string 宿主归约=no-op）。
- **`widget_emit_set` 名册精化**（vue/mod.rs）：prop 派生 Pascal emit 只有 handler 体**实际调用**（`on_x(...)`/`props.on_x(...)`）才入名册——声明而未调用的 prop 让父侧名册谎报（`@Delete` 永不发射，真名=消息变体 `DeleteBlock`；cap_444_callback_channel 直通断言红）。
- **census 具名修订 #4**：(72,11,0,23)→**(75, 8, 1, 27)**——state 别名解锁 parse 族 3 源（err-3）；[15]（无 text 叶源）spliced（+1）；新 parse 通源 handler 入缺席分母（absent+4）。
- **缺席分母 taxonomy（七族具名，逐族归属）**：①`$event` 视图实参/元素括号属性行（`input (placeholder: …)`）等**年龄面语法缺口**族——语言面收敛决策（跨 981 冻结语法，非 UI 适配器可闭）；②float/f64 模型字段与 float 载荷——981 明确不支撑域（fail-closed）；③双 widget 同文件源——组件面单 node 形状（IF-981-MODULE 域）；④控制流/let/var/调用 handler 体——组件面只收平铺赋值（语法层拒，需语言面体模型扩展决策）；⑤空体 handler（无可发射语句——语义 no-op）；⑥引号事件键/handler（`."save"` 形——语法缺口族①）；⑦store/HTML 片段伪源（提取伪影，非 widget 源）。
- **vue 门（V-B-WEB 具名命令）**：72/82→**77/82**——余 chart×5=PLAN-643 契约分歧陈旧断言（chart 元件已降 name-only（tier=package_origin），官方组件包持真契约；测试仍断言内建 vue mapping——归属 chart/官方包线处置，不单方弱化）。vmodel_fold 断言刷新至 Plan 008/DEBTS #6 单参取文本的后来设计。
- **边界（诚实）**：语料全量 spliced（1→全覆盖）剩余=七族 taxonomy——①④⑥为年龄面语法/体模型收敛（多年面、跨 981 冻结 grammar-id，需语言面 owner 决策）；②为 981 不支撑域；③为 IF-981-MODULE；⑤⑦语义上无可拼接。B-WEB-I/V/R 勾选前置于上述族的归属处置。

### §13.14 年龄面语法收敛 I：事件绑定调用形/事件键修饰符/元素括号属性（切片 #6m，2026-11-18）

- **语法**（`auto-source.at`）：`age-event-key`（ident 基名+`age-event-key-mod` 尾规则链，`onclick.self`——双 AGE 面）；`age-event-binding`（`.Msg(args)` 调用形+`age-event-binding-args`/`age-event-arg`——仅 age-component；**多步 opt 不被组合器支持**，括号段/体块段归子规则）；`age-element-props-block`（`input (placeholder: "s") { … }` 括号属性行+可选体块——仅 age-component）。
- **`$event` 承载**：UI 侧**同长归一化**（`$event`→`nevent`，6↔6 字节 span 零位移，沿 state→model 同法；lane 编译链与公共坐标出口双接线）——不引入全局 token（首版 evt-dollar token 实测扰动**全部 15 个 profile** 的 grammar-id，撤回）；归一化后按普通 ident 进 pratt 实参。**投影丢弃实参**（宿主/DOM 事件通道，本面不求值不验证——模板发射由方言 AST 承担，此投影无消费方）。
- **grammar-id 具名重基线**（981 `plan981_grammar_scope` 内注明，975 先例法）：仅 `age-component`（1fb56340…，前值 96ecc437…）与 `age-package`（874a32e1…，前值 14561796…——共享 age-event-key 规则族）两值漂移；**其余 13 profile 逐字节不变**（实测对照冻结值）。
- **census 具名修订 #5**：(75,8,1,27)→**(75,8,4,24)**——[4]（modifier 键）、[18]/[59]（$event 绑定，含括号属性形）spliced；absent-3。
- **剩余缺席族**（taxonomy 更新）：③双 widget（IF-981-MODULE）、④控制流/let/调用体（语言面体模型扩展）、⑤空体、⑥引号事件键/handler（引号键族——本批未收，方言层合同）、②float 域、⑦伪源。①的括号属性/修饰符/$event 子族已闭。
- **永久用例**：lang `plan980_web_grammar_6m.rs`（四形编译）；UI `p980_web_event_arg_forms`（三形端到端 lane 拼接）。

### §13.15 ③ node 选择器（切片 #6n，2026-11-18）

- **语言面**：`compile_component_scenario_states_at(src, …, node_index)`——多 node/widget 同文件源按声明序**选一编译**（越界具名拒）；None 形态保持单 node 形状门。命名仍单 node 形（`__age_state_0_<field>`——非选中 node 不进本编译）；**无组合/跨引用语义**——完整多 node 组合=IF-981-MODULE 域边界保持。
- **UI 面**：lane `compile_at`/`lane_handler_bodies_spec_at`；`generate_sfc_with_lane` 逐 widget 生成并**聚合** bodies/absent（单 widget 源行为不变）。
- **census 具名修订 #6**：(75,8,4,24)→**(75,8,4,26)**——[65][66] 第二 widget（空体）经选择器入分母；两源第一 widget（载荷声明+参数 handler）**仍缺**：widget 头参数表（`widget N(props)`）不在年龄面语法+④调用体/`;`分隔——[65]/[66] 全解锁的下一层（w1 独立编译已证选择器机制正确）。
- **永久用例**：UI `p980_web_multi_widget_node_selector`（逐节点发射/一体化聚合/越界负控）。
- **剩余边界（④/⑥ 终局陈述）**：④控制流/let/调用体=**外部名面**（`fs.read_dir`/`listusers`/`complete`——back.api 导入+FFI/方法/动态值；无外部名的纯结构扩展零语料收益）——需 use-import/foreign-contract（967/973 谱系）设计决策后才有安全承载，不做未验证旁路；⑥引号键族实查**零拼接收益**（全为空体/PARAMS-B 受阻源）。widget 头参数表+`;`分隔=④ 子项随该决策一并评估。

### §13.16 调用语句域合同：fail-closed 承载（#6o）+ 通道调用完整承载（#6p，2026-11-18）

本节合并落档 #6o 的调用语句 fail-closed 合同（其收据引用的 §13.16 此前未写入规范——#6p 补档并实装）与 #6p 的通道调用承载。

**事实修正（#6p 探针证伪 #6o 前提）**："wide-ops 无调用后缀"不成立——pratt 调用后缀一直在场（文法 `call` 块 + 引擎 WantOp 臂）：`.x = on_select(1)` **解析通过**、在 **Bind** 阶段以 `source.undefined` 拒；调用**语句**形经 #6o `age-view-call-stmt` 解析。#6o 后实况缺口（master 在场）：①通道调用（callee=widget 头参数/宿主通道 prop）无承载——检查函数只铸赋值，**调用语句被静默丢弃**（[65] `.Bump` 拼接体只有 `n.value = next;`，`on_select(next)` 丢失）；②`handler_check_fn` 不守 `body_foreign`（#6o 收据声称"检查函数不铸"，实际照铸→部分体 verify+部分拼接）；③域外调用 handler 进 ops=整 lane 编译失败=**兄弟 handler 连带全体缺席**。

- **通道调用（语句位）完整承载**：callee ∈ widget 头参数 → 投影为 `ComponentChannelCall` 行（channel/args 原源 span；具名实参不收=fail-closed）；**实参经检查函数死 let 验证**（无注解推断；形参/字段可见性+类型/ownership 共享 verify 真核——`on_x(typo)` 红 `source.undefined`；未验证拼写绝不进拼接体）。lane 发射：赋值片段与通道行按**源 span 序 merge**；通道行=emit 契约事件形（`on_x` 去 `on_` 的 Pascal 形 ∈ msg 变体 → `emit('<Pascal>', args)`）或回调 prop 形（`props.on_x(args)`）——老道 PLAN-668 R-23/Plan 444 同约定（SFC defineEmits/defineProps 声明机制承载）；实参按精确 span 经 a2x-ts 发射+宿主映射。
- **fail-closed 闭合**：`body_foreign`（let/if/非头参数调用——④外部名面族）handler **检查函数不铸**（#6o 合同实装）→ lane 具名缺席（旧道承载）；**值位置**（赋值 RHS/通道实参嵌套）通道调用=效应作值，域内具名拒 `source.age.component-unsupported`（非泛化 `source.undefined`）；**ops 派发**通道 handler=零效果具名拒（场景重放车道无宿主，不做丢通知的部分重放）；view 内联 lambda 体含非赋值语句→值函数**不铸**（此前静默跳过调用语句=部分拼接丢句，同族闭合）。
- **ops 豁免（兄弟保全）**：lane 侧方言预扫描——handler 体含调用语句（裸 `f(...)` 表达式语句）的 unit handler 不进 ops（通道走检查函数道；域外走缺席）；此前任一调用体进 ops=编译失败=全体缺席。
- **census 具名修订 #8**：聚合 (75,8,7,23) **不变**——[65] 计数在 #6o 已计入（部分拼接态）；本切片修**内容正确性**（`.Bump` 体补全 `emit('Select', next);`）与兄弟保全语义；判别力由 `p980_web_channel_call_lane` 六臂断言承担。
- **永久用例**：lang `plan980_web_6p_channel_call.rs`（六例：行投影/值位置拒/ops 拒/死 let 判别力/域外标记/foreign lambda）；UI `p980_web_channel_call_lane`（完整体文本/混合兄弟/props 形/RHS 缺席/lambda 缺席）。
- **剩余边界（具名）**：④完整承载（外部调用 `fs.read_dir`/`listusers` 等进受检体）=976 后继目标域决策（a2ts.foreign-unsupported 域合同变更，§13.15 终局陈述保持）；lambda 面通道调用承载=后续切片（本批只闭"不铸"防丢句）；watch/lifecycle 体调用语句仍按方言合同跳过（#6o 边界）；`on` 块/lambda 体多语句单行形（`a; b` 不换行）=`list-nl` 文法既定边界。

## 14. UI-W1-AV 接收（PLAN-1071，2026-11-19）

- **接手关系**：980 §0 移交表——B-AV-I/V/R 与 981 SESSION/LANGUAGE/PROOF/FUSION/CONTENT 中传统 UI、Widgets 与 UI 基线混合见证所需 AV 格归 1071（UI-W1-AV，GOAL-002）。本计划承接 980/981 已交基础，按组件组先交正式子面；游戏与扩展场景完整 AV 由 1075 接。
- **U01 输入子面（正式交付）**：980 phase fold #14（lang `95b0ccb49`/ui `91d549ea`）——`ace_node_desktop` 11/11（CPU+真窗）、`p980_u01_window_input_e2e`（真窗键入 i0 `你好Auto`/42、`abc` 具名拒零效果、i1 `乙侧`/-5、两实例隔离）、`p980_node_desktop_permission_revocation_refuses_before_dispatch`（撤权）、关开/late/双关。
- **981 W1 五组证据汇入**（绑定 `3047b3481`，980 §9 :1070 入账）：G-INSTANCE 双实例 2:1、G-LIFECYCLE 关开、G-VIEW/FRAME/INPUT 75/75、G-BADEDIT 坏编辑保旧（会话侧语义归 auto-ui R64-1）、G-RESOURCE 会话腿资源归零（AGE 节点资源面归 UI058/1068）。
- **生产入口收据（1071 实跑，2026-11-19）**：`ace_ui_hello` 7/7（含真窗 e2e）、`authoring_session` 4/4、`mixed_regions_lifecycle_flow` 7/7、`desktop_behavior` 11/11、`image_surface_contract` 1/1、`plan041_instance_ownership` 4/4、`plan033_b1_scope_isolation` 2/2、`theme_scope_isolation` 4/4、lang `ace-source` PASS / `ace-av` 1426/1428。
- **010 列表字段边界（交 1072 共享接口）**：`010-contact-form` 的 `faqs`（list-of-structs）为**仅视图数据源**（model 只读、`for` 渲染、无事件写）；列表字段在生成面按只读数据源处理（Vue v-for/等价），不作为可变 state 通道；AGE 列表语义归 collections owner 单写。
- **预存红台账（具名排除）**：lang `ace-av` `host_console::{av_console_entry_selection_cannot_bypass_main_policy, av_console_owned_front_door_enforces_console_entry_only}`（981 §9 既有漂移）；UI `gallery_vue_golden`（auto-os widgets-gallery 源↔auto-ui golden 基线跨仓漂移，V-B-PARITY 面归 985/auto-ui owner 线，983 r25 已点名）。
- **边界（保持）**：U02–U06 示例级 AV 完整逐例仍压 980 ④-A 解锁路径（980 §9 :1739「B-AV-I 保持未勾——全分母仍压 ④」）；`UI-W1-AV` 最终汇总收据由 980 `B-AV-R` 签发（981 §9 交接约定），本接收记录其已交子面与生产入口证据，不冒全包完成。
- **PLAN-104 补记（U02 AV harness 能力与边界，2026-11-19，merge 落地）**：AV 组件面 `ace_node_desktop` 的显示模型 harness 现支持——①view 条件项按实例状态求值可见性（不可见叶不呈现、叶序保持）；②元素 label（`h2`/`span` 族）作为展示行（不占 AV 叶编号）；③`view for` 循环体叶按仅视图数据（`ComponentViewDataField`）展开为具体叶（010 faqs：4 项×q/a=8 叶），不支持形状具名拒；④`style` 面不被 harness 消费（按原字节空白化，样式由 A2Vue/1072 负责）；⑤`ui_apply_bool` 的 Auto 面标量为 `i32`（0/1）。**保持边界**：foreign handler（`print` 等，D-④-4）在 AV 与 Vue lane 均具名缺席（`absent`/零效果拒）；entry-λ `oninput` 与无消息输入在 AV 场景面无派发面，输入类格以 A2Vue 收据为准（2026-11-19 用户裁定）；U02–U06 示例级 AV 完整逐例仍随 ④-A 解锁路径推进。

## 15. Vue 与生产 TS/JS 接管（PLAN-1072，2026-11-19）

- **交付来源**：[PLAN-1072](../../plans/archive/1072-autoui-vue-web.md)（归档件；冻结门 `cargo t ace-source` 923/923 PASS、`cargo t ace-av` 1426/1428 仅预存 `host_console`×2、UI 受影响 10 套件全部零失败、`plan1072_component_style` 19/19 通过）。
- **语言侧（`ace-core` & `auto-source.at`）**：
  - **组件面 `style` 惰性替换路径**（依 981 r18 会签 (c)3）：`style-head`/`style-item` 对 `[age-component]` 开放字面量声明（参数化/`pub` 仍由 `style-params` profile 门控留在包面），并补全 `age-event-key` `type-kw` 键臂（承接 `type: "password"`）；投影期 `apply_component_face_styles` 收集模块内 `style` 字面量表，并经共享 `replace_style_in_entry_expr`（`frontdoor.rs`）递归替换 view 内 `ComponentEntryValue::Expr` 与 `ComponentEntryValue::If`（`then_value` / `else_value` 两臂），统一对导入样式名、未声明 `style` 键标识符与带参调用形（`ExprKind::Call`）执行 `source.age.style-call` fail-closed 拒绝（不生成 `Item::Const`/`Fn`，不触发 resolve/全局表三条件）。
  - **条件样式与 `view-for` / 仅视图数据字段（010 边界）**：`age-view-for`（双名形 `for i, f in .expr`，两 AGE 面）投影为 `ComponentViewItem::For`（循环体文本叶不平铺进 `texts`）；`age-view-data-lit` 仅限无类型列表字面量投影为 `ComponentViewDataField`（沿 PLAN-1071 §14 仅视图数据边界，不进 `fields`/state/scenario，handler 写入报未声明字段具名拒）；`age-entry-value` 新增 `if` 条件样式值（两臂均经 `replace_style_in_entry_expr` 受检）；native/desktop 与包面 SFC 渲染腿分别以 `view-for-unsupported` 与 `sfc-view-for` 具名拒。
  - **`expose` 成员语法对齐**：`age-expose-member` / `age-expose-item` / `age-expose-name-tail`（`profiles: [age-component, age-package]`）并容 `.name`（如 `expose { .Inc, .count }`、多行无逗号 `.Inc\n.count`）与裸名（`expose { alpha, beta }`）；`AgeComponent` `grammar-id` 更新至 `fad57c5f81b4ee3ca191a472b952778f4666692e3760ed186e3f137c6ba164a1`、`AgePackage` 更新至 `06809c843671b91fe57ad77ddb6126a5bd094efc263cf0fc4beda4cdfb87111c`，13 个邻居 profile 逐字节保持不变（`plan981_grammar_scope` 4/4）。
- **UI 侧生成、运行时与冷生成修复（`auto-ui-runtime` & `auto-ui-tools`）**：
  - **`marshal_str_boundary` 统一 str 边界（收口 980 §10-1 `owned-wrap`）**：宿主（Vue `ref` / SFC 形参）以普通 JS 字符串承载；rt 文本值（`Uint8Array` / `A2xOwnedStr`）仅存于求值内部。宿主映射层对 `a2x_str_*` 行读取经 `wrap_host_reads` 包裹 `a2x_str_lit(...)`（跟踪单/双引号与反斜杠转义，绝不误改字符串字面量内容），写入经 `a2x_str_utf8(...)` 解码并释放 owned 副本，清理 `owned-swap` 快照与旧值释放块；状态函数道/检查函数道/λ 道三道统一经本层（`p980_web_param_browser_mount` 亦迁至官方 `script_runtime_body`）。
  - **配方注册显式化与 `__bind_*` 隔离**：`generate_sfc_with_lane` / `generate_sfc_with_lane_product` 在生成前显式调用 `prepare_file_style_recipes` / `replay_file_style_recipes`，使独立入口也能将 `style: NAME` 展开为字面 class 串；`__bind_*` 方言合成键排除出 lane handler 规格表以免误扰整组件 lane 派生。
  - **冷生成 `a2x_rt.mjs` 落盘与错误保旧恢复**：`VueProject::from_workspace_reads`（`components/*.at` 路径）继承 `ace_vue_rt` 且 `VueProject::generate` 补调 `self.write_ace_vue_rt()?;`，确保冷目录 `--gen-only` / `gen_vue_project` 完整写出 `src/a2x_rt.mjs` 及脚手架（`@vue/compiler-sfc` + `esbuild` 编译通过）。`incremental_compile_changed` 与 `gen_vue_project` 在语法错误（non-strict watch 返回 `Ok(0)` 告警 / strict 返回 `Err("Failed to compile")`）或 lane 拒（`Err("ace.vue-lane-unsupported")`）下均逐字节保留既有 `App.vue` 与 `a2x_rt.mjs`，修复源码后可恢复编译。
  - **005/010 整例翻转与双产品浏览器实证**：`005-login`（6/6 handlers, `absent=[]`, `test_005_login.mjs` 7 判据）与 `010-contact-form`（3/3 handlers, `absent=[]`, `v-for` over `faqs` + `field_col` 展开）在产品 lane SFC + 官方 `a2x_rt.mjs` 下全绿；`test_p1072_lifecycle_watch_expose.mjs` 实证 `setup` 严禁重入仅执行 1 次 + `ext_imports`、`watch` 触发与 `unmount()` 停听、`defineExpose` `.Inc()` 调用与 `style` 展开。
- **残余四项记账收口**：①`n_assigns`（`ace_ts_provider.rs:315-320, 392-397`、`ace_vue_script.rs:490-496`）取语言侧 `flat_handler_assigns(&stmts)` 跨分支深平铺计数，唯一消费者 `needs_fn` 在含分支赋值时保守为真、零语义漂移；②`AC-LIF-07`（生命周期全项）随 `plan1072_vue_web` + `.mjs` 浏览器实证闭合；③§10-1 `owned-wrap` 由 `marshal_str_boundary` 收口；④`#6p` 分支内调用形（`ace_ts_provider.rs:1619` 具名缺席 `.__evt_onclick_1`）与 `ace_package_sfc` 多分支控制流重放边界（`sfc-handler-stmt`、`sfc-view-for`）登账入 `KNOWN-DEBT-AND-RISKS.md`。
- **实现与验证边界（不超宣称，与 PLAN-982 / 后继计划分工）**：
  1. **开关与路径边界**：仅在 `auto-ui-tools --features ace-vue` + `AUTO_UI_ACE_VUE=1` 下对 `app.at`（`vue.rs:4173, 7684`）与 `components/*.at`（`vue.rs:4495`）启用 ACE lane 替换；默认构建、`front_dir` 兄弟文件（`vue.rs:7398`）、`pages/`（`vue.rs:4350`）、`widgets/`（`vue.rs:7757`）与 `stores/` 仍走旧 `VueGenerator` / `ts_adapter::TypeScriptTrans`。
  2. **双轨拼接边界**：`generate_widget_with_lane`（`ace_vue_script.rs:796-884`）基于 `VueGenerator::new().generate_sfc(&widget)` 仅拼接替换受检 `on` / `__evt_*` 体；`computed`、`watch`、`setup`、`ext_imports`、`.Init`/`.Destroy` 与 `absent` 处理器仍由 `VueGenerator` 发射（如含 `setup`+`use{}` 的 `Widget B` 维持双轨 fail-closed，`absent=[".Bump"]`）。
  3. **夹具与联合门边界**：1072 产品验证以 Rust `gen_vue_project` / `incremental_compile_changed`、`@vue/compiler-sfc` + `esbuild` 与 Playwright Node `http`（`test_010_form.mjs` stub `@/components/ui/*`、`test_p1072_lifecycle_watch_expose.mjs` stub `BOOT_EXT_STUB`；`plan980_010_form_short.rs` 浏览器腿仍挂旧 `App.vue`，而 `p1072_010_product_browser_mount` 挂 lane `App.sfc` 测三字段 `v-model` + `Submit`）为边界；公开 CLI `auto gen` / `auto build` / `auto run` 派发与完整 `pnpm` / Vite 打包运行由 **PLAN-982** 联合门与后续 `V-TS-CONSUMER-UI` 收尾任务承接（均登账入 `KNOWN-DEBT-AND-RISKS.md`）。
