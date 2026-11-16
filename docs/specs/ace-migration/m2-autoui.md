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
> [PLAN-979](../../plans/archive/979-ace-m2-schema.md) r11 §5 V-11（T-08/T-04 回验）。
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
