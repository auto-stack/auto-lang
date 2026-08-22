# Plan 425: component fn 双轨退役——AST 级糖化,widget 单轨

> **状态**: ✅ 执行完成(2026-08-23,worktree auto-musk;T1-T4 全落地,回归全绿——见 §6 执行记录)
> **前置**: Plan 408(已归档,component fn 机制);auto-musk PLAN-037 Phase 4(已合并——**musk 全部 24 个 component fn 已迁 widget,双轨在真实工程侧已清零**)。
> **仓库**: **auto-lang**(parser / aura/extract / ui_gen/api / ui_gen/vue)。
> **目标**: `component fn` 降级为 `widget` 的**语法糖**(AST 级转换),删除 fragment 双轨(parser/extract/render 三对平行路径),单一代码生成路径;顺带 `view {}` 包裹可选化(体即视图)。

---

## 6. 执行记录(2026-08-23)

- **T1 糖化 + view 可选化**(5c2cb2bd):`parse_component_fn_decl` 产出
  WidgetDecl(body 包 view、params→props 走 fragment hint 映射,产物逐字节
  等价——单测 `test_plan425_component_fn_widget_equivalence` 锁定);
  `parse_widget_decl` 体即视图自动包裹(单测
  `test_plan425_widget_view_optional`);api.rs 同文件 widget 名入
  all_sub_widgets(糖化引用走组件路径,同名 WidgetDecl 天然入列——§4 风险
  表对策兑现)。plan408 测试改 widget 回调契约(`on_<snake>` + Pascal 事件)
  与 App 前置(根=源序首个 widget);007 golden 补 `:key`(Plan 360 静态键
  进入单轨渲染,属性级差异按 §2.2 更新并记录)。
- **T2 删除双轨**(880247ae,净删 199 行):`extract_widget_from_fragment`/
  `fragment_param_type`/`fragment_to_component_node` 与 extract 两处
  `is_component` 分支删除;ViewFragmentDecl 瘦身为 view-fn 专用
  (name/params/body);`parse_component_fn_decl` 直接构造 WidgetDecl(块序
  use→computed→msg→model→watch→on→style→body 不变);api.rs component fn
  收集循环删除。**渲染单轨评估**:AuraNode::Component 类型保留(解释器
  a2ui/export/atom 与 ark/rust/jet 生成器仍消费),但 .at 源已无生产者
  (fragment_to_component_node 删除后唯一构造点消失)——两条渲染路径的
  分工边界:Element+known_sub_widgets 为 .at 唯一路径,Component 节点仅
  解释器/外部 AST 使用(§1.4 文档化结论)。
- **T3 存量迁移 + 文档**(180de0d8):k3 `item_cf.at.disabled` 复活验证
  (build + vue-tsc 绿;产物差异仅 msg 变量大小写——两轨均按声明原样发射,
  同拼写源逐字节相同);scenario-dialect spec 标注"糖,新代码请用 widget"
  + view 可选化语义;k3 README 记录验证结论。plan408_tests/golden 007/
  010 存量在 T1 已收编(007 补 :key,010 零变化)。
- **T4 收口**:auto-lang 3089 单测 + auto-man 6 + k2/k3/k4 canary + musk
  三测(auto build / cargo test / vitest 2 存量失败基线)全绿。

## 0. 背景与现状

PLAN-037 证实 component fn 无独有能力(widget 全覆盖 + 多出 expose/routes/tick),且双轨是
漂移温床(408 修缺陷时反复两边对齐;migrate_widget.py 在 musk 侧做的"改名 + view 包裹"
即本计划的 AST 级等价物,产物逐字节相同)。当前双轨:

| 轨道 | parser | extract | 父视图渲染 |
|---|---|---|---|
| widget | `parse_widget_decl` | `extract_widget_from_decl` | known_sub_widgets 路径 |
| component fn | `parse_fragment_decl_body_tail` | `extract_widget_from_fragment` | `AuraNode::Component` 分支(vue.rs ~4416) |

auto-lang 自有存量(迁移面):plan408_tests.rs、test/a2vue/007-010 golden、examples 少量
(015-notes 的 node_modules vendor 文件**不在范围**——独立钉版工具链编译)。

## 1. MVP

1. **糖化**:`component fn X(params) { blocks... body }` 在 parser 内直接产出
   `WidgetDecl`(body 元素包进 view)——与 `widget X(params) { blocks... view { body } }`
   同 AST。块顺序语法(params→use→computed→msg→model→on→body)不变。
2. **view 可选化**:widget 体若以元素开头(无 view 块),同样自动包裹——两种关键字
   均支持体即视图,人体工学对齐。
3. **单轨化**:删除 `ViewFragmentDecl`/`extract_widget_from_fragment`/
   `fragment_to_component_node` 路径;`api.rs` 的 component fn 收集循环删除
   (糖化后天然是 WidgetDecl)。`view fn`(非 component)保留内联机制不动。
4. **渲染单轨**(评估项):`AuraNode::Component` 分支与 sub_widget 路径的能力差异
   (P6/P11 曾分别修)在单轨后是否可合并——能合则合,不能合(外部 ext 组件仍需
   Component 节点)则文档化两条路径的分工边界。

## 2. 迁移面

1. plan408_tests.rs:component fn 源串改 widget(或保留——糖化后原样通过,仅断言注释更新)。
2. golden 007/010:期望产物应零变化(糖化产出同 AST→同 SFC);若 view 包裹导致
   属性级差异,更新 golden 并记录。
3. k3 canary:`item_cf.at.disabled` 复活验证糖化等价(产物 diff ItemRow)。
4. 文档:README/syntax.md 的 component fn 章节标注"糖,新代码请用 widget"。

## 3. 测试设计

- 等价性单测:同一组件两种写法 → 产物逐字节相同(糖化正确性核心断言)。
- 全量:3083+ 单测、k2/k3 canary、auto-musk 三测(musk 已全 widget,理论零影响——
  但作为最终验证门)。
- view 可选化:widget 体即视图的用例 + 混合块顺序用例。

## 4. 风险

| 风险 | 等级 | 对策 |
|---|---|---|
| 糖化后 `is_component` 语义丢失(sub_widgets 合并依赖它) | 🟡 | api.rs 合并循环删除后同名 WidgetDecl 天然入列;单测锁 |
| `view fn` 与 `component fn` 共享 parse_fragment 路径 | 🟡 | view fn(内联)路径保留,仅 component 关键字改道;分叉点在 dispatch 层 |
| 015-notes vendor 编译 | 🟢 | 不在范围,钉版工具链 |
| 外部仓库依赖 component fn 行为 | 🟡 | 糖化保语义(非删除),仅内部路径退役 |

## 5. 执行步骤(草案)

1. T1 parser 糖化 + view 可选化 + 等价性单测。
2. T2 删除 fragment 双轨(extract/api/渲染评估)+ 全量回归。
3. T3 存量迁移(plan408_tests/golden/k3 复活)+ 文档标注。
4. T4 收口:musk 三测 + 状态归档。
