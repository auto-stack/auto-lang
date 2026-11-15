# ace-migration: m2-schema — Schema 领域语言包与 AGE 节点投影（H0 已验切片）

> Status: **H0 早期切片已验**（PLAN-979 T-H0-I/V/R，2026-11-15；独立复审三轮、终审 pass @
> `71472d975`，rebase 后交付提交见计划 §9）。完整 AGE Node 语法/语义、属性与字段、
> model/msg/on、传统 AutoUI 全模式、GUI/FFI/宿主与 A2R 腿、完整分母映射（PLAN-979
> T-01..T-08 与 980/981）**未交付**——本文件不把它们标为已支持。
> 来源：[PLAN-979](../../plans/979-ace-m2-schema.md) §5 H0 合同 + T-H0-I 冻结决定、
> §9 三轮复审收据（F1–F12）；Design55/ACE M2 路线。本文件只写**已验行为与接口**。

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
- 未验/未声明：GUI、FFI 宿主、产品运行、A2R 腿、完整 Node/属性/字段/`model`/`msg`/`on`、
  传统 AutoUI 全模式、完整分母映射。

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

- 每 **(实例, 字段)** 一个状态函数 `fn __age_state_<i>_<field>() <ty>`：该实例的 `model` 初值 + 定向 ops 内联（字段标识符→实例局部），返回**该字段**；`<ty>` 取字段声明类型（`int`/`i32` → `int`，`bool` → `bool`；其余类型未接线，具名拒）；
- 驱动 `fn __age_drive() int`：各实例**观测字段**的 `__age_state_<i>_<observed>()` 按 **256^i** 求和（instance 0 在个位）。观测字段缺省为**首个 `int` 字段**（故以 bool 开头的模型也可观测），也可由 `compile_component_scenario_observing(..., observed)` 按名指定（须存在且为 `int`：未知名 `component-unknown-field`，非 int `component-unsupported`）；
- 渲染求值函数 `fn __age_c_text_<i>_<j>() Str`：叶表达式中的字段引用绑定为**该字段**的 `__age_state_<i>_<field>()` 调用，再 `.clone()` 返回（多字段模型因此可正确绑定，不再有"只认首字段"的错绑）。

三者与普通项一起经**唯一** text-record verify（无第二解释器/发射器）；`desugar_hash`（对受检模块文本取 content_sha）标识受检工件身份，源哈希 `input_hash` 不含脱糖。

### 6.3 观察与双后端

- AV：`ace_av::age_component::observe_component(src, path, alias, scenario)` → `{encoded, instance_texts}`；`observe_component_observing(..., observed)` 指定观测字段；驱动与逐实例渲染函数各经共享 lower→MIR 校验→ABC→AV 执行取真值。静态面另有 `observe_component_texts(src, path, alias)`：多节点逐个文本叶取共享 Str 真值（按节点分组的**值**证据）。
- A2R：同一 `CheckedModule` 以 `__age_drive` 为 i32 入口 → Rust fileset → **真实 rustc** → 运行 → `value:<encoded>`。
- 已验场景值（同源同场景）：plain 双实例 **514**（2+2·256）、helper 场景 **259**（3+1·256）、N-tick 矩阵 **516**（4+2·256）。**typed view 的渲染文本只在 AV 侧观察**；A2R 证明的是同源场景驱动 + helper/N-tick，不得表述为"typed view 在两后端渲染"。
- **编码边界**：256^i 仅在各实例观测状态 < 256 时单射；instance 3 需 < 128 才不溢出 i32（溢出为 AV `Trap{IntOverflow}` 响亮失败，非静默别名）。

### 6.4 拒绝面（组件）

`source.age.component-shape`（形状：无 model 字段/空 view/多 node）、`component-unknown-field`（观测字段名不存在）、`component-unsupported`（未接线：非 `int`/`bool` 字段类型、无 `int` 字段可观测、未支持表达式形状）、`component-duplicate`（重复 `on <Msg>`）、`component-event`（未声明事件/未声明字段赋值）、`component-scenario`（实例越界/0 或 >4 实例），以及 H0 既有的 `source.age.slot|structure|text-type|leaf-expr|text-count|hidden-name|legacy-widget|domain-frontend`。全部保原 span，不静默丢字段/行为。

### 6.5 本切片边界（pending，不得据此声明支持）

场景面单 node 定义（静态面可描述同模块多个 node）、单模块（多模块 Node 工程需 960 模块图前门与领域面组合）、`int`/`bool` 模型字段（`str`/其他类型未接线）、观测字段须为 `int`、无越权/未声明实例引用校验（V-10）、传统 UI 字段级覆盖与产品清单归 980、完整 AGE 产品运行归 981。
