# ace-migration: m2-age — AGE Node 作者语言的 ACE 接入（view-if 刀已验切片）

> Status: **view-if 刀已验**（PLAN-981 r19→r20，2026-11-19；文法/投影由 PLAN-980
> view-if 刀 `d69f29f91` 先落主、981 第二片 `386d6c748`+复审修复 `ac1ae182c` 收口
> 观察面；work→独立复审（fresh 子代理两轮，F-1 修复闭环）→phase fold 收据见计划
> §9）。本文件是 PLAN-981 frontmatter 声明的规范落点（`new_spec_components`），
> 随各批 phase 递增；**只写已验行为与接口**。
> 已交付（本节）：AGE `view` 条件节点（裸 `if`/`if-else`/`else if` 链）两面文法与
> 投影、描述面 `cond_spans` 事实、AV 平铺文本观察入口的 fail-closed 具名拒。
> 未交付（保持显名）：条件**渲染语义**（分支选择真值——消费车道 owner）；其余
> 十批（SOURCE/SESSION/LANGUAGE/PROOF/CONTENT/RUST2D/RUST3D/FUSION/AUDIT）
> 见 [PLAN-981](../../plans/981-ace-m2-age.md) §5–§8。
> 来源：PLAN-981 §9 r19/r20/复审收据；PLAN-980 §9 view-if 刀收据（文法/投影/
> 九消费者/grammar-id 重基线）。

## 1. view 条件节点：文法、投影与正式 grammar id

文法（`experimental/ace/grammars/auto-source.at`，两面 **age-component +
age-package**；`d69f29f91` 落定）：`age-view-item` 增条件臂——`age-view-if`
（`if <cond> { items } [else …]`，then 体走子规则 field 定位）+ `age-view-else`
（双 alt：块形 / else-if 链形——内层完整 `age-view-if` 自带 else，链任意长）。
分支体仍是 `age-view-item` 面（元素/文本/属性行/嵌套条件）。**age-h0 面无此规则**
（profiles 仅两面）——H0 view 内 `if` 在解析即拒，H0 观察面物理上收不到含条件
view 的描述（观察门不可达的文法根据）。

**正式 grammar id**（`plan981_grammar_scope.rs` 逐行冻结，13 邻居逐字节不变由
该测试自证）：age-component `620a0f73…`、age-package `30834de6…`（前值
`8292feae…`/`c0e635d2…`；981 r19 的 `b0ae9d90…/ecc64b8e…` 线被主线取代，不作数）。

投影（`ComponentViewItem::Cond(ComponentViewCond)`）：

| 字段 | 事实 |
|---|---|
| `cond: Expr` | 作者源形条件表达式（原源 span） |
| `then_items` / `else_items` | 分支子树（源序）；**else-if 链 = else_items 内单个嵌套 Cond**（内层自带 span） |
| `span: Span` | 整个条件节点，锚定作者 `if` 关键字语句（复审用例逐偏移断言） |

**两支文本叶平铺进 `view.texts`**（主线设计，保持）：分支叶与顶层叶同入绑定验证
通道——编译/绑定检查对含条件 view 的源**保持通过**（分支叶引用未声明名在编译处
红，见 `plan981_view_if.rs` bad/ok 对照）；平铺序不承载条件可见性（见 §3 观察门）。

## 2. 描述面条件事实：`cond_spans`

`ComponentViewOut.cond_spans`（组件面描述）与 `AgePackageComponent.cond_spans`
（包面描述）：view 内**全部**条件项的原源 span，**树序全量**——else-if 链每层、
元素内嵌套、命名 view 内的条件各一项；`cond_spans[0]` = 作者最前 `if`（观察门
诊断锚点）。收集助手 `collect_view_cond_spans` / `ComponentView::all_cond_spans`
（core `age_component.rs`）；结构体字段由编译强制全初始化（无静默缺席构造点）。
未知构造不静默丢：IF-981-FIELDS 同纪律的字段级普查行
`ComponentFieldVocab::view_conditional`（每嵌套层一行，span 锚各自 `if`）。

## 3. AV 平铺文本观察入口的 fail-closed 具名拒

**规则**：实际消费平铺文本的观察入口见 `cond_spans` 非空 → 具名拒
`source.age.view-conditional`，消息携带作者最前 `if` 字节偏移（`@<start>..`）
——否则两支会被当作**同时可见**。封闭面（复审独立枚举，恰 4 个，av crate 内
共用 `view_cond_observation_gate` 助手 + `view_conditional_observation_diag`）：

| 入口 | 消费面 |
|---|---|
| `observe_component_texts` | 静态 `view.texts` |
| `observe_compiled`（`observe_component`/`observing`/`with_initials` 暖路径） | 场景 `instance_texts` |
| `observe_age_package` | 包面平铺文本行 |
| `observe_component_states_and_texts` | NATIVE 免驱动双观察的 **texts 半边**（states 半边不消费平铺文本，不受影响；复审 F-1 补口） |

**不设门的面**（合同不变）：`compile_component_scenario_states`（states-only/SFC
车道——不遍历 view 项，980 fp_editor 翻转走的车道）；`observe_component_states`
（纯状态面）；`observe_age_h0`（文法不可达，§1）。auto-ui 消费者：桌面/native
装载点经 `view_contains_cond` 先拒（`ace_native.rs`/`ace_node_desktop.rs`）；SFC
模板从树发 v-if（不经观察面）；包桥 `.ok()?` 一致降级。

**撤门条件**：某车道定义了条件可见性观察语义（分支选择真值）后，撤该入口的
门并以真值用例顶替（owner：980 消费车道批 / 981 后续批）；撤门前该入口的
fail-closed 是正确行为，不是缺陷。

**永久用例**：core `plan981_view_if.rs`（5：states-only 可用、`cond_spans`
树序+逐偏移锚定、绑定 bad/ok 对照、包描述事实、普通 view 负控）；
av `plan981_view_if_av.rs`（5：四入口拒收+锚偏移、initials 道 texts 半边负控）
+ 980 侧 `plan980_view_if.rs`（4：形状/平铺/负控/深嵌可判）。判别力经复审变异
自证（拆门/吞门→用例红；diag span 归零→锚定断言红）。
