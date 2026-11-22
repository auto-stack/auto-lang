# ace-migration: m2-age — AGE Node 作者语言的 ACE 接入（view-if / on-if 组件面已验切片）

> Status: **view-if 刀已验**（PLAN-981 r19→r20，2026-11-19；文法/投影由 PLAN-980
> view-if 刀 `d69f29f91` 先落主、981 第二片 `386d6c748`+复审修复 `ac1ae182c` 收口
> 观察面；work→独立复审（fresh 子代理两轮，F-1 修复闭环）→phase fold 收据见计划
> §9）。**on-if 组件面刀已验**（PLAN-981 r21，2026-11-19；§4）。本文件是
> PLAN-981 frontmatter 声明的规范落点（`new_spec_components`），
> 随各批 phase 递增；**只写已验行为与接口**。
> 已交付：AGE `view` 条件节点（裸 `if`/`if-else`/`else if` 链）两面文法与
> 投影、描述面 `cond_spans` 事实、AV 平铺文本观察入口的 fail-closed 具名拒；
> 组件事件体（`on`）hook-local `let`/`if-else` 两面同开+执行真值（§4）。
> 未交付（保持显名）：view 条件**渲染语义**（分支选择真值——消费车道 owner）；
> 其余十批（SOURCE/SESSION/LANGUAGE/PROOF/CONTENT/RUST2D/RUST3D/FUSION/AUDIT）
> 见 [PLAN-981](../../plans/981-ace-m2-age.md) §5–§8。
> 来源：PLAN-981 §9 r19/r20/r21/复审收据；PLAN-980 §9 view-if/on-if/lambda-if
> 刀收据（文法/投影/九消费者/grammar-id 重基线/lane 结构化）。

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

## 4. 组件事件体 hook-local `let`/`if-else`：两面同开与执行真值（on-if 组件面刀）

文法（r21 起，两面 age-component + age-package）：`age-on-let`/`age-on-if`/
`age-on-else` 与 `age-on-stmt` 双臂**去 age-package 门**（G2 时组件面裁剪不含
两臂的旧合同被本刀取代——组件面 let/if 消费已由共享链交付）。三处语句面：

| 语句面 | 规则 | 内容 |
|---|---|---|
| 直接形 `on <Msg> { … }` 体 | `age-on-stmt` | 裸名赋值 + `let` + `if`（两面同开） |
| widget 形 handler 体 | `age-on-widget-stmt` | 点前缀赋值 + 调用（组件面）+ `let` + `if`（`age-on-if`）+ `age-view-if-stmt` 兜底臂（PEG 序后置——`age-on-if` 整体失败才命中，承载 980 的分支内调用形 foreign/链式作者面） |
| `if` 分支体 | `age-on-branch-stmt` | 裸名/点前缀赋值**两拼写并集** + `let` + 嵌套 `if` |

`age-on-else` 双 alt：块形 / **else-if 链形**（镜像 980 `age-view-if-else`——
链式作者面不因并集回归；投影 els = 单个嵌套 If，递归）。watch/lifecycle 等
`age-view-stmt` 其它消费体不动（view 面方言臂属 980 谱系）。

投影（`project.rs`）：`project_on_stmt_inner` 三处共用 helper（+点前缀臂走
`collect_view_stmt` 同一脱糖）；widget 形 `stmts` **源序**（let 先于 if 钉住）、
`assigns` = stmts **全深度收平**（`flat_handler_assigns` 两面共用，与 G2 直接形
同合同——分支赋值不消失、执行语义以 `stmts` 为准）。**assign-only 消费者见
Let/If 必须具名拒**（stmts 契约）：auto-ui 两处消费门已落——lane/components
通道**真复放**（`stmt_fragments_in` 根块 `hir::Stmt::If` 整语句发射，auto-ui
`56725cd5`）；`ace_package_sfc` 脚本重放**具名拒** `source.age.sfc-handler-stmt`
（auto-ui `7b8456e5`，永久负例在案）。执行/校验零第二求值器：G2 共享链
`hook_stmts_lower` 消费 stmts+形参。

**AV 执行真值**（`plan981_on_if_av.rs`，组件面首跑）：双派发分支选择（第二派发
else 赢、单派发 then 赢——两支同放/条件失效必红）；裸 if 假跳真跑；hook-local
let 驱动条件+体（三 Tick→4，第三次假条件保持）。**具名类型边界**：`str` 载荷与
字面量 `==`（借用 vs owned）是共享检查既有边界（980 已另落 AGE 旗控放行
`age_eq`——text 族入口严格保持拒，774 负控不破）。

**正式 grammar id**（第三次重基线，r21 并集后）：age-component `61799f12…`、
age-package `93cd6814…`（前值线：`620a0f73→e3859aed`（980 on-if，master 线）与
`620a0f73→ab4c2000`（981 分支线）在并集树双失真后实测重冻结；13 邻居逐字节
不变）。980 lambda-if 零文法改动（id 不随其漂移）。

**永久用例**：core `plan981_on_if_component.rs`（4：widget if/else 投影+参数+
两支平铺判别、裸 if 无 else、let 源序+不进平铺、无控制流负控）+ g2 翻正
（`p981_hook_let_and_if_open_on_both_faces`）；av `plan981_on_if_av.rs`（3 真值）。
980 侧配套：`plan980_on_if.rs`（4，含 else-if 链/分支调用 foreign/Str==str 放行）
+ `plan980_lambda_if.rs`（core 5+av 4，值函数道结构化——`Lambda.stmts` 接共享
`project_view_stmt_children`，本仓投影侧无第二分支语义）。

## 5. Gallery 公共 AV 会话与适用边界（PLAN-1075）

**多模块闭包会话**：Gallery 作者来源是 `use <id>: …` 模块集（如 g01 的
resources/shapes/main），经 `AgeSession::open_modules(modules, entry, …)` 以同一
受检前门开为 AV 会话（`experimental/age-ace-session/src/age_session_core.rs`，
bridge 与 driver 共同编译的唯一实现；出口 `app::<entry>`）。`update_module(id, src)`
同 PID 单模块热更新：未名模块具名拒；编译失败保留上一好代/账面/代数（坏编辑保旧）；
成功递增 body 代数、新值即时可见（进程内 AV 重编译，无 cargo/rustc）。变体入口
文件名带连字符（`v-color`/`main-alt`）不是合法模块 id——由接线适配层净化为下划线；
受检前门对连字符 id 具名拒 `module-identity`（净化是接线职责，不是语言放宽）。

**适用边界**：受检前门作者面为平面场景行（`shape2d`/3D 冻结叶/组件实例行；嵌套
`group2d` 与 `character2d`/`solid2d`/`player3d` 等元素种类不在其中）。56 个
Gallery generated 闭包（22 入口×47 变体）中 **6 个**过门开为 AV 会话：g01 全部
5 变体 + g02/main（静态 GLB）；其余 50 个具名 pending（45 嵌套语法 `expected fn,
found node`、5 元素种类 `scene-element`），其前门扩展归图形能力线（AutoUI
099–102 MAP/ASSET/PBR/CLIP 及后续场景扩展计划），非本面义务。混合场景/两游戏归
1099/1100/1101。

**永久 oracle**：auto-ui driver `tests/plan1075_gallery_av_session.rs`（8 用例：
g01 四行声明序/几何/材质/hidden、v-color/v-order/v-position/v-visible 变体事实、
g02 model3d 行+Model3D 资源+GLB 路径符号实参、坏编辑保旧/热更新恢复/Run-Stop
净零/PID 稳定、连字符 id 与未知入口负控）+ `tests/fixtures_plan1075/`（17 个 .at
逐字冻结自 age-examples `ea0fc59`，manifest 逐文件 SHA256）。会话面基线入口：
`authoring_session`/`mixed_regions_lifecycle_flow`/`s2_age_routing`/`authoring_storage`
（driver）与 bridge `s1/s3/s4`（live ACE）。

**SDK 身份**：driver `.sdk-source` = 冻结基座 681 文件 + overlay 1435 文件
@ auto-lang `0a167f611`（`prepare_sdk --from-snapshot` + `apply_overlay`，复用
受检前门不刷新 overlay）。原 M1 引擎/收据身份保持 age-examples `ea0fc59` 冻结
（`gallery.py validate --all` 12 IDs/22 subentries/6 locks；21/22 收据全绑
296/296 oracle）。

## AGE 视图标签与属性

`age-component` 和 `age-package` 的视图中，`divider` 和 `spacer` 支持裸标签、
体块和括号属性形。已有的 `sep` 也支持带体块或属性形。`outlet` 保留其裸标签及
既有属性形。未知裸标签继续在视图准入处报错。

视图属性中的 `for` 与 `to` 按键名投影，因而标签关联和导航目标保留原拼写及值。
`link` 支持原有路由形以及 `href` 属性形。上述词在其它语法位置的用途保持原合同。
两条前门共用语法，标准 AAC 文法数据以同一共享源生成。

## 6. 026 原样源双端写面（PLAN-113，2026-11-22）

用户 2026-11-22 授权计划内实现（106 B2 先例；写面 owner=本仓 AGE 组件/包面，
106/114 已归档无在途）。两面（age-component + age-package） enduring 行为：

- **组件面 CST**：「标量字段 + 行尾注释」后接裸 `List` 嵌套 children 记录
  列表的组合接纳（026 `dbNodesFull` 真语料首错；去注释/单 Lista 负正例在案）。
- **model `List` 裸写惰性化**：model 时点按元素记录型定型；显式 `List<T>`
  保持状态语义；view 迭代契约（p114 `ids`/`expanded`）保持；`[]T`/`[N]T`
  原合同不变。
- **记录元素 CtorNamed 改写**：异构记录字段过共享 resolver（object-elem
  消除）。
- **事件面**：handler 参数注解、`$event` 归一化（宿主占位具名）、注入
  helper 的 owned `Str` 形参。
- **包面补充**：widget lambda 与视图事件绑定引用未声明消息变体的合成
  （投影时按组件面同律）；legacy widget 回调 handler 体非纯透传时按参数
  拼写合成载荷槽（注解优先、缺省 `str`）；裸 `List` 参数默认与文字列表
  属性收**同质整数**元素（`pageSizes: List = [10, 25, 50]`），混合
  str/int 仍拒（p114 同质字面量契约不变）。
- **组件车道**：引用惰性列表的 handler fail-closed 跳过（检查函数 None +
  事件程序 retain；包面 p105 list-iteration pending surface 契约不变）；
  整模块投影 AGE 面 age_face 开（内联 helper fn 调用语句可过）。

**正式 grammar id**（PLAN-113 重基线后；`plan981_grammar_scope.rs` 逐行冻结，
13 邻居逐字节不变由该测试自证）：age-component `817d81cf…`、age-package
`2308100f…`（= 本计划 9bd00590… 叠加主线 106/107/114/123/1093/1095 后续刀后
的实测值；取值以该测试冻结行为准）。

**永久用例**：core `plan981_event_a`（p107_ legacy 回调/参数注解负正例）+
`plan981_grammar_scope`（id 冻结）；av `plan981_age_package_av`
（p107_/p114_/p981_ 包面消费面）；auto-ui 双端验收
`plan113_026_sqlite_bridge`（AV 真实库宿主桥，契约见
`auto-ui:docs/specs/runtime/ace-sqlite-host-bridge.md`）与
`plan113_026_vue_browser`（A2Vue lane 全链）。

**配套 additive 辅助**：`a2r_std::sqlite::clear_error()`（op 前清粘性
`LAST_ERROR`、op 后读——既有 op 成功不清错的 by-design 行为不变；宿主桥
per-op 诊断消费）。
