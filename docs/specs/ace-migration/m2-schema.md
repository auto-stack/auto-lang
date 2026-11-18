# ace-migration: m2-schema — Schema 领域语言包与 AGE 节点投影（M2-A 已验交付）

> Status: **M2-A 通用接面已验**（PLAN-979，2026-11-16；T-05 复审 pass，绑定代码 `e352e462b`；规范沉淀与复审记录见计划 §9）。
> 已交付：H0 早期切片（AGE node/view/text + 普通函数文本）、组件面（typed 候选/场景脱糖/AV+A2R
> 真跑）、通用领域包与 typed 投影接面、字段级投影完整性机制（`field_manifest`），以及
> IF-980-FIELDS / IF-981-MODULE|REF|FIELDS|ORIGINAL 五个正式交接的落主回验（981 `e352e462b`、
> 980 `3de0bea0b`；字段子面 `46f4ac0ec`）。
> **不在本文件声明**：完整 AGE 语言/五 view 产品化（981/LANGUAGE）、传统 AutoUI 全模式与窗口
> （980/PARITY 等）、GUI/FFI/宿主与两游戏产品腿、AGE-L04/AG-A 完整分母（§9 范围外登记）、
> 跨仓 auto-ui 规范（owner 待沉淀）。
> 来源：[PLAN-979](../../plans/archive/979-ace-m2-schema.md) §5 H0 合同 + T-H0-I 冻结决定、§9 三轮
> H0 复审收据（F1–F12）与 M2-A 交付收据；Design55/ACE M2 路线。本文件只写**已验行为与接口**。

## 1. 领域激活与身份

| 面 | 事实 |
|---|---|
| profile | `ace_core::source::SourceProfile::AgeH0`，slug `age-h0`，常量 `AGE_H0_SOURCE_PROFILE` = `auto.source.age-h0.draft`；工程/模块**显式选择**，不从扩展名或内容推断 |
| 领域入口 | `ace_core::source::age_h0::compile_age_h0(src, module_path, LegacyWidgetAlias, capabilities) -> AgeH0Compilation`（`description` + `checked` + `input_hash` + `atom_text`） |
| grammar | `experimental/ace/grammars/auto-source.at` 的 `age-h0` 裁剪面（= text-record 全表面 + AGE 规则）；GrammarId = 裁剪后 canonical bytes（含上下文名称表与规则诊断标签） |
| 程序格式 | program codec v5（v4 = 上下文名称表 + `Atom::CtxName`；v5 = 规则诊断标签）；parser ABI `ac-syntax-parser/3` |
| 通用入口 | `parse_front`（含片段）、`parse_program_with`、`parse_program_via_cst`、`parse_fragment_front`、`compile_parts_with`、`compile_program_parts` 对 `AgeH0` 一律具名拒绝 `source.domain-frontend`；领域路径走 crate 内部 `parse_domain_whole_module` |

Schema 数据（不是运行时领域知识）：`contextual(version: 1)` 表逐项声明
`spelling` / `token` / `role`，规则步 `name(of: "<id>")` 只在**声明的槽位**要求该拼写。
同一拼写在其他位置仍是普通标识符；词法面未新增保留字。

| role | 声明拼写 | 说明 |
|---|---|---|
| `node` | `node`（首个声明 = 规范），`widget`（同角色别名） | `widget` 仅在显式 `LegacyWidgetAlias::Accept` 下准入；关闭时声明头 span 受控拒绝 `source.age.legacy-widget`，不回落旧引擎 |
| `view` | `view` | 沿用现 Auto 源面既有保留字 `r-view`（未解除；普通 Auto 同样拒绝 `let view = 1`） |
| `text` | `text` | 普通标识符 |

身份：兼容配置进入领域描述与 `(source, grammar)` 身份说明；本切片无投影缓存，故
`alias` 不在 `(grammar_id, input_hash)` 对中（完整缓存接入时须并入该对）。

## 2. 已验输入形状（H0）

```auto
node Hello {            // 一个 node 声明；node 名是普通 ident（单声）
    view {              // 一个 view
        text "Hello" {} // 一个 text 叶：字面量 或 同工程普通函数调用
    }
}
```

- 表达式首批 = 字面量或普通函数调用（`str`/`Str` 均可，受检文本种类从 Checked HIR 读回）。
  裸标识符叶（`text my_str {}`）在共享 text-record 面按构造器形状解析，属结构拒绝——
  完整表达式面归 T-02/T-08。
- 文本叶脱糖为普通隐藏函数 `fn __age_h0_text_<n>() Str { return (<expr>).clone() }`，
  与普通项一起经**唯一** `text-record` verify（resolve/typecheck/bind）；领域层不另建类型检查器。
- 值由共享 `lower → MIR 校验 → ABC 编码/解码/准入 → AV 执行` 取得（`ace_av::age_h0::observe_age_h0`）；
  不同源文本/不同函数体必须产出不同观察值。
- 上层声明名含 `__age_h0_text_` 前缀 = 具名冲突 `source.age.hidden-name`（防隐藏函数劫持）。

## 3. 诊断（已验形状）

| 类 | code | 说明 |
|---|---|---|
| 槽位 | `source.age.slot` | `expected \`<spelling>\` at the declared \`<role>\` slot`，span = 冒犯 token |
| 结构 | `source.age.structure` | `unexpected token in <construct>`；深度优先取最内层构造，span = 冒犯 token / EOF 零宽；仅整次解析失败时出现；**AGE 声明之前**的顶层杂散 token 仍走通用语法错误（不在任何已开始构造内） |
| 文本叶 | `source.age.text-type` | 脱糖 `.clone()`/类型拒绝（"叶表达式须为文本"）；保留共享消息与 span |
| 文本叶 | `source.age.leaf-expr` | 名字/参数/其它共享拒绝（保留共享消息；不得误标为 text-type） |
| 形状/命名 | `source.age.text-count`、`source.age.hidden-name`、`source.age.legacy-widget`、`source.domain-frontend` | 0/多 text 根、隐藏前缀冲突、别名关闭、通用入口拒绝 |

## 4. 拒绝与引擎边界

- plain `text-record` 面不接受 AGE 源；AGE 开/关对**普通模块**产出逐字节相同的共享 Atom 文本
  （同值/同型）；AGE 面 `use.rs`/`use.c` 与 text-record 同面（导入语义仍归 F0 共享层）。
- 不引入第二套解析/类型检查/求值：领域回调只产未受信候选（结构 + 原 span + 表达式），
  信任来自唯一 verify；值来自共享后端。
- 本包自建的 H0/组件切片不含：GUI、FFI 宿主、产品运行与窗口；完整 AGE 语言/五 view、传统
  AutoUI 全模式与完整产品分母的归属见 §9（981/980/LANGUAGE owner）。

## 5. 证据

plan 979 §9：T-H0-I（冻结决定与实现提交）、T-H0-V（V-H0-* 行、CLI 冒烟、受影响职责组并集）、
T-H0-R（三轮独立复审 F1–F12 全部关闭，终审 pass）。核心数字：core conformance 483 例全绿
（22 个 `p979_age_h0_*`）、av `p979` 7 例、syntax-grammar `p979` 13 例；受影响固定组
`ace-syntax ace-source ace-semantics ace-artifacts ace-av` 全 PASS（ace-av 的 `vctip.exe`
进程树为已记录环境边界）。后续核验一律以最新代码重跑，不引用本文件的数字当交付证明。

## 6. 组件面（T-03 已验切片，2026-11-15）

### 6.1 profile 与 typed 候选

- `SourceProfile::AgeComponent`（slug `age-component`，`auto.source.age-component.draft`）：**同一** `auto-source.at` 的另一裁剪（`age-h0` 的 H0 合同逐字节不变），自有 GrammarId；所有通用入口（整模块/片段/兼容桥/独立二进制）具名拒绝 `source.domain-frontend`。
- `project_component_candidates(src)`：未受信 typed 候选 —— `node`（名/声明头/原 span）、`model` 字段（名/类型词/初值/原 span；`int`/`i32`/`bool`/命名类型均解析）、`on <Msg>` 处理器（消息名 + 赋值清单，字段/右值/原 span）、`view` 文本叶（沿用 H0 形状）。
- 兼容：`widget` 头在 `LegacyWidgetAlias::Accept` 下等价同一 `node` 角色；关闭时声明头 span 具名拒 `source.age.legacy-widget`。

### 6.2 场景脱糖（消费者配置，不是源语言）

`ComponentScenario { instances: 1..=4, ops: [Send{instance, msg} | Tick{instance}] }`，其中 `Life.Tick` 在本切片建模为保留消息 `Tick`（运行身份归 981）。脱糖产物全部是**普通 Auto**：

- 每 **(实例, 字段)** 一个状态函数 `fn __age_state_<i>_<field>() <ty>`：内联该实例**全部**字段的 `model` 初值与定向 ops（字段标识符→实例局部），返回**该字段**；`<ty>` 取字段声明类型（`int`/`i32` → `int`，`bool` → `bool`，`str`/`Str` → `Str`，`f32`（历史同义拼写 `float`）→ `f32`；其余类型未接线，具名拒）。f32 状态由共享 resolver/adapter 的 typed-numeric 面承载（同型四则/比较），混宽不隐式提升（`source.type-mismatch`）；AGE 包面另开 f32 **注解位**（fn 参数/返回/`let`）——`ann-ty`/`ret-ty` 的 f32 臂（PLAN-981 `D-981-AGE-F32-SURFACE`，2026-11-18；AV 执行证据 `av/tests/cases/plan981_f32_surface.rs`：`x = x + vx * dt` 两 Tick 后 1.0→2.0）；
- 驱动 `fn __age_drive() int`：各实例**观测字段**的 `__age_state_<i>_<observed>()` 按 **256^i** 求和（instance 0 在个位）。观测字段缺省为**首个 `int` 字段**（故以 bool 开头的模型也可观测），也可由 `compile_component_scenario_observing(..., observed)` 按名指定（须存在且为 `int`：未知名 `component-unknown-field`，非 int `component-unsupported`）；
- 渲染求值函数 `fn __age_c_text_<i>_<j>() Str`：叶表达式中的字段引用绑定为**该字段**的 `__age_state_<i>_<field>()` 调用，再 `.clone()` 返回（多字段模型因此可正确绑定，不再有"只认首字段"的错绑）。

`ComponentCompilation::identity()` 给出与领域接面相同的证据元组 `{id, version, grammar_id, input_hash, compat}`（同源不同 compat 即失效）。

三者与普通项一起经**唯一** text-record verify（无第二解释器/发射器）；`desugar_hash`（对受检模块文本取 content_sha）标识受检工件身份，源哈希 `input_hash` 不含脱糖。

### 6.3 观察与双后端

- 静态面（无 `model`/`on` 的源）由 `compile_age_component` 描述并按需要取叶值；**任何** `model` 字段或 `on` 处理器在静态门都是 `component-unsupported`（`observe_component_texts` 因此只覆盖无模型源）；可解析但**没有 `node` 声明**的源在静态门被接受并返回空分组（空/不可解析源仍报 `source.syntax`）。
- AV：`ace_av::age_component::observe_component(src, path, alias, scenario)` → `{encoded, instance_texts}`；`observe_component_observing(..., observed)` 指定观测字段；驱动与逐实例渲染函数各经共享 lower→MIR 校验→ABC→AV 执行取真值。逐字段**宽值出口** `execute_component_def(compiled, def_id)` 返回共享 `Value`——f32 状态即 `Value::F32`（2026-11-18 PLAN-981 f32 切片）；窄枚举 `ComponentFieldValue` 仍只承载 int/bool/str（其 F32 扩展须与 auto-ui 的 `AceStateValue`、TS 能力类型与文本呈现**同窗**改，否则跨仓穷尽匹配编译红——登记于 `D-981-AGE-F32-SURFACE`）。静态面另有 `observe_component_texts(src, path, alias)`：多节点逐个文本叶取共享 Str 真值（按节点分组的**值**证据）。
- A2R：同一 `CheckedModule` 以 `__age_drive` 为 i32 入口 → Rust fileset → **真实 rustc** → 运行 → `value:<encoded>`。**f32 面（2026-11-18）**：wrapper 的入口合同仍是 i32（`A2xResult<i32>` / `value:<i32>`），f32 状态经逐字段 def（`__age_state_<i>_<field>`）在生成程序内取值并用比较折叠成判别性整数；`SemanticView` 在 text-record 族放行 F32 常量与 typed `Num` 算子（只 f32），record-text 文件集与通用/collections 族一样携带 `RT_NUMERIC`（否则生成的 `a2x_lit_f32`/`a2x_div_f32` 在 rustc 期 not found）。证据：`a2x-worker/tests/cases/plan981_age_package_a2r.rs` 的 `x = x + vx*dt`（x=3.0、y=1.5 → 7）与 vx 变异控制（→ 0）。
- 已验场景值（同源同场景）：plain 双实例 **514**（2+2·256）、helper 场景 **259**（3+1·256）、N-tick 矩阵 **516**（4+2·256）。**typed view 的渲染文本只在 AV 侧观察**；A2R 证明的是同源场景驱动 + helper/N-tick，不得表述为"typed view 在两后端渲染"。
- **编码边界**：256^i 仅在各实例观测状态 < 256 时单射；instance 3 需 < 128 才不溢出 i32（溢出为 AV `Trap{IntOverflow}` 响亮失败，非静默别名）。

### 6.4 拒绝面（组件）

`source.age.component-shape`（形状：无 model 字段/空 view/多 node）、`component-unknown-field`（观测字段名不存在）、`component-duplicate`（重复 `on <Msg>` 或重复 `model` 字段名，均在第二处 span 报）、`component-unsupported`（未接线：`int`/`bool`/`str`/`f32` 之外的字段类型、`__age_drive` 打包观测缺少 `int` 字段、静态门的 `model`/`on`、未支持表达式形状）、`component-event`（未声明事件/未声明字段赋值）、`component-scenario`（实例越界/0 或 >4 实例），以及 H0 既有的 `source.age.slot|structure|text-type|leaf-expr|text-count|hidden-name|legacy-widget|domain-frontend`。全部保原 span，不静默丢字段/行为。生成名族（`__age_c_text_*`/`__age_state_*`/`__age_drive`）对用户顶层名保留，占用即 `source.age.hidden-name`。

### 6.5 本切片边界（pending，不得据此声明支持）

场景面单 node 定义（静态面可描述同模块多个 node）、本切片单模块（多模块 Node 工程由 981 `compile_age_package` 包前门交付，见 §9）、`int`/`bool`/`str`/`f32` 模型字段（`f64`/`double`/记录/变体等仍未接线）、`__age_drive` 打包观测字段须为 `int`（f32 走逐字段 def 执行，见 §6.3）、文本叶呈现 f32 仍未接线、跨实例状态引用（越权访问）尚未建模（V-10；实例越界与未声明事件**已**具名拒，见 §6.4）、传统 UI 字段级覆盖与产品清单归 980、完整 AGE 产品运行归 981。

## 7. 领域包与 typed 投影接面（T-02，已交付）

`ace_core::source::domain`：通用领域包接口；AGE/组件是首批注册包，不另建第二套解析/检查/发射器。

| API | 已验事实 |
|---|---|
| `DomainPackage { id, version, profile, hidden_prefix, compat, project }` | 包身份 + 未受信投影回调；测试包 `test.pair/v1` 证明 owner 可经公开 API 注册并被同一驱动消费 |
| `DomainCandidate { structure, behaviors, requirements }` | 未受信结构/行为引用/资源需求；越界/重复绑定/空角色/过深 → `source.domain-candidate`，不静默丢 |
| `DomainCompilation { id, version, grammar_id, compat, description, checked, input_hash, atom_text }` | 共享驱动 `compile_domain`：领域前门→包投影→兼容闸→隐藏名冲突→行为脱糖（普通 `Str` 函数）→**唯一** text-record verify→从 Checked HIR 读回 |
| `DomainIdentity { id, version, grammar_id, input_hash, compat }` + `same_evidence` | 包版本/切换、源变化、兼容配置变化均判失效（缓存可作键）；同输入稳定 |
| `DomainRegistry` / `DomainPackageConflict` | 同 id 同版本同 profile 幂等；冲突具名拒且**旧注册保留**（旧程序可继续编译） |
| `mapping_complete(description, required_roles)` | role 粒度覆盖：删必需 `view` 行 → `Err(["view"])`，合法近邻 `Ok` |

非领域 profile 由驱动具名拒 `source.domain-package`；通用解析/编译入口对领域 profile 拒
`source.domain-frontend`。领域回调只产候选，信任只来自共享 verify。

## 8. 字段级投影完整性（`field_manifest`，与 980/981 共用）

`ace_core::source::field_manifest` 在 role 粒度之上提供**字段/构造 ID 粒度**的版本化清单与覆盖判据（本文件写机制；分母数据归 owner）：

- `FieldManifest { domain, version, rows }` + `RequiredFieldRow { id, origin, source_span, parent_role, type_rule, applicability, projection_path, check_id, consumer_owner }`；
- `field_coverage(presented, manifest)` → `required/present/missing/unknown`：必需行缺席与投影侧未知构造都具名可判（不静默丢）；合法近邻/可选行缺席不影响；
- `validate_field_manifest`：ID 唯一/语义字段非空/条件理由非空/span 有序——不可执行的表直接拒；
- `component_field_ids(candidates, vocab)`：组件候选→字段出现集（ID 词汇表由清单 owner 注入，Core 不含产品语义）。

已落主分母（不在本文件复制）：UI `ui.widget-decl/v1` **115 行**（980，见 [m2-autoui](m2-autoui.md)）；AGE `AGE_CONSTRUCTS` **12 行** + `AGE_CONSTRUCTS_OUT_OF_SCOPE`（981，见 §9）。删除必需映射/删除清单行的反控在 owner 永久 case 中各自可辨。

## 9. age-package 交接回验与最终边界（2026-11-16，落主 `e352e462b`）

979 对 981/SOURCE 四个交接键在落主树独立回验（命令与逐行判定见计划 §9）：

- **IF-981-MODULE**：两文件 AGE 工程经同一包前门；AV 两实例 2:1 事件 + 各 4 Tick → **6/5**，helper 0→10 → **16/15**，typed view 文本随之变；A2R 同一 `CheckedModule` 经**真实 rustc** → **6**（改 helper → **16**）；模块换序身份不变、模块环具名拒、每模块符号保留。
- **IF-981-REF**：同模块免导入；跨模块须从声明模块导入（含 `pub use` 链）；`ref-undeclared` / `ref-invisible` / `ref-not-a-node` / `ref-ambiguous` 具名互斥，span 落原模块；修复后同入口恢复。
- **IF-981-FIELDS**：`AGE_CONSTRUCTS` **12 行** = node/model/model.init/on/on.lifecycle/view/view-2d/view-3d/view.text/view.node-ref/view.scene/view.scene.resource，分母冻结 + 逐行删反控；`AgeViewExit` 按**后缀字节**判定，未知后缀拒 `source.age.view-suffix`；`AGE_CONSTRUCTS_OUT_OF_SCOPE` = `AGE-L04`（981/LANGUAGE+966）与 `AG-A01..09`（980 与 981/LANGUAGE 并集）。
- **IF-981-ORIGINAL**：g01 三文件原源闭包（hash 与 age-examples 原件一致）→ typed 投影 + `row.cpu` 值：红圆 (-20,10,d30) / 蓝 quad (0,0,30×30) / 绿圆 (25,-15,d24) / 黄 quad (-45,-25,12×12, `visible:false`)，声明序、颜色 alpha、坏 kind 原 token span、改资源值变、恢复。

**边界（不得据此声明完整 AGE/产品支持）**：

- A2R 腿观察的是实例 0 的值（6/16）；实例 1 仅以状态符号存在性断言；typed view 渲染文本只在 AV 侧观察；
- g01 不进 A2R（`age.material2d.color.v1` 不在 compute 面）；含模块级 `const`/`var` 的包图被 `module.global.*` 拒绝（RUST 批）；
- AGE-L04（选择/条件）与 AG-A01..09 不在 age-package 分母；AGE-L03 的五 view 中 `view-portrait` / `view-widget` 未实现（981/LANGUAGE owner）；
- 传统 AutoUI 全模式/窗口（980/PARITY 等）、GUI/FFI/宿主与两游戏（981/SESSION/RUST/LANGUAGE）、完整产品分母（985）仍未交付；4 个 widget 槽位（style/setup/actions/widget 级 `use{}`）在 980 面为具名拒绝（D-980 债务）。

**证据**：计划 §9 T-03/T-08/T-04/T-05 收据；永久 case：core `plan981_*` 147 / `plan979_*` 57 / `plan980_*` 13，av `p981` 3 / `p979` 26，a2x-worker `p981` 2 / `p979` 7；T-04 六组并集 5/6 PASS（唯一红为 PLAN-1062 语料的既有基线红，具名排除、不充 PASS）。
