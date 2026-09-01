---
plan_id: PLAN-502
status: reviewed               # drafting → executing → execution_done → reviewed → archived (2026-09-01 复审通过)
feature_name: diagram Phase 1——flow-diagram v1(分层布局 + SVG 渲染 + hover 交互)
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: svg `<text>` 不支持约束(y 刻度标签注记处)——502 M1 svg text 直通双端落地后解除(chart 页自身未动,约束面 diagram 家族生效)"
new_spec_components:
  - "docs/specs/auto-lang/ui/design/diagram-components.md: flow-diagram 组件契约节(数据轨 props nodes/edges/direction、Sugiyama-lite 布局、svg text 标签机制、hover emphasis/tooltip 交互)"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致(Vue/VM 双端 parity 锁定)——diagram 家族双端同源:svg text 直通标签/Sugiyama-lite 布局/hover 交互双轨落地"

affects: [docs/specs/auto-lang/ui, docs/design/autoui/diagram-components.md, examples/widgets-gallery, examples/ui]
current_step: 7
total_steps: 7
---

# [PLAN-502] diagram Phase 1——flow-diagram v1(分层布局 + SVG 渲染 + hover 交互)

## 变更摘要

依据设计文档 [diagram-components.md](../design/autoui/diagram-components.md) §8 Phase 1
立项：交付 diagram 家族首个组件 **flow-diagram**——Sugiyama-lite 分层布局(纯 Auto)
+ v1 SVG 渲染(484 charts 同通路) + hover tooltip/emphasis(498 状态机模板)。

主体**零引擎改动**；唯一待定引擎改动 = svg `text` 直通(M1 标签发射对照验证若胜出,
落一处小引擎改动,单列步骤)。显式排除: DSL 糖(Phase 3)、节点 select(Phase 1.5,依赖
498 M0 on_click 臂)、边 hover/节点拖拽/pan-zoom(Phase 4/5)、其余图类型(Phase 2a/2b)。

## 目标

1. **M1 标签发射对照**(Phase 1 首个验证项,设计文档 §6.1/§10.1): svg `text` 直通
   (vue 轨平凡 / VM 轨 svgdoc 经 resvg 原生支持) vs DSL text 绝对定位 overlay
   (VM 轨动态 arbitrary 值支持面待核验)——产出决策记录,胜者成为全 diagram 家族
   的标签机制;
2. flow-diagram 用户态组件: 数据轨 props `nodes`/`edges`(record schema 见设计文档
   §5,含 `x/y` 钉住位预留,Phase 1 恒 -1)、`direction: "td"`;
3. **Sugiyama-lite 分层布局**(纯 Auto): rank(最长路径分层) → order(重心法 2–4 轮
   降交叉) → coord(层内等距 + 父居中一趟);`direction td/lr` 经转置;
4. 图元发射 + 边路由 v1: Init 把布局+几何写进 render 段记录,view `for seg in .segs`
   循环发射 svg `path/rect/ellipse`;边 = 节点 bbox 边界交点直线段;箭头/head/tail
   字形(arrow 三角形、diamond/circle/cf-* 小多边形)Init 手算(donut 弧先例);
5. hover 交互(498 三段式模板): `hoverNode str` 状态 + emphasis 高亮(平面二态)
   + 锚定 tooltip;命中 = 节点 bbox 绝对定位 mouse-area 兄弟层(svg 兄弟层不经
   svgdoc 序列化,484 增量先例);
6. 示例页 + 双端(vue/vm)目检一致。

## 架构方案

- **纯用户态组件**(对齐 charts 484: "引擎给笔,Auto 持笔",几何 = Auto 代码,Init 纯
  派生,ADR-19 重播种链路复用);组件落位官方组件包机制,消费方式
  `use { package: official from "components" }`;
- **零引擎改动起步**: 渲染走既有 svg 子树直通(vue)/ svgdoc 序列化(VM)双通路;
  hover 命中走 mouse-area 兄弟层;均无需引擎新臂;
- M1 若 svg `text` 直通胜出 → 一处小引擎改动(vue.rs svg 直通集 + aura.at/svg shape
  tag 集 + aura_view_builder 序列化臂),门禁随之升 Category B/C;
- 与 498 的关系: hover 状态机直接抄 498 三段式模板(Init 纯派生 + 最小状态写入 +
  view 状态投影);select 交互依赖 498 M0 的 on_click 臂,归 Phase 1.5 不在本计划。

## 需求分析与背景调查

- 设计文档: docs/design/autoui/diagram-components.md(2026-08-31 定稿)——§4 DSL 语法
  (本计划只用数据轨)、§5 record schema、§6.1 v1 SVG 轨与布局算法、§7.2/§7.3 交互与
  命中区模型、§10.1 标签发射风险;
- charts 484 先例: 组件化机制(`ui_gen/widget/component_registry.rs`,Builtin > Local
  > Package)、Init 几何派生(1-2-5 nice step / path d 串)、svgdoc 双端通路、
  `vm/tests_chart_geometry.rs` 端到端几何对拍;
- 已知约束: svg 无 `<text>`(M1 对照验证的由来);svgdoc 静态 → VM 轨元素级事件不可用
  (mouse-area 兄弟层命中);f-string px 类 492 定案后直写合法(overlay 轨先例 =
  donut tooltip);
- 静态糖(`node/edge/group` DSL 词汇)不在本计划——Phase 3 parser 扩展;本计划只交付
  数据轨 + 组件,静态糖到位前示例以数据轨书写。

## 详细设计

### M1 标签发射对照验证(决策点)
- svg `text` 直通原型: vue 轨 svg 直通标签集加 `text`(vue.rs `map_tag` 同族,平凡);
  VM 轨 svgdoc 序列化加 `text` 元素(aura_view_builder serialize_svg_element 同族,
  resvg 栅格化);双端渲染冒烟;
- overlay 对照: 动态 `left-[${x}px]` arbitrary 值在 VM/iced 轨解析支持面核验
  (donut tooltip 仅有固定值 `top-[20px]` 先例);
- 产出: 决策记录(回填设计文档 §6.1 标签发射条),胜者进 M5。

### M2 flow-diagram 组件骨架
- props: `nodes`/`edges` List(record schema §5: id/label/shape/group/x/y…)、
  `direction: str = "td"`;
- Init 管线骨架: props 解析 → 布局占位(等距网格) → render 段记录;view svg 发射
  冒烟(矩形节点 + 直线边),双端可见。

### M3 Sugiyama-lite 布局核
- rank/order/coord 三趟 + direction 转置;全 float 纪律(中间量显式 float);
- 几何对拍测试(参照 `vm/tests_chart_geometry.rs`): 固定小图 → 断言层位/交叉数/
  坐标确定性。

### M4 边路由 + 箭头字形
- bbox 边界交点直线段;head/tail 字形(arrow/diamond/circle/cf-one/cf-many)Init
  预计算端点角度与小多边形 path。

### M5 标签 + hover 交互
- 标签按 M1 决策落地(节点/边标签);
- `hoverNode str = ""` 状态机 + emphasis 双分支(线宽/opacity)+ 锚定 tooltip;
  节点 bbox 绝对定位 mouse-area 兄弟层命中。

### M6 示例与双端验证
- 示例页(widgets-gallery 新页或 examples/ui/ 独立示例,对齐 024-charts 结构);
  vue/vm 双端目检;gallery golden 基线。

### M7 复审与归档准备

## 测试设计

- 布局几何端到端(参照 `vm/tests_chart_geometry.rs`): records → rank/order/coord →
  确定性断言;
- 门禁分级: 纯 .at 组件步骤 = Category A;若 M1 动引擎 → 该步升 Category B
  (`cargo check -p auto-lang` + `cargo t ui` 局部模块) + Category C
  (`cargo test -p auto-lang --test schema_drift`,svg shape tag 集变更触发);
- 双端一致性: 示例目检(用户验收) + gallery golden(基线更新)。

## 验收标准

1. M1 决策记录落地,标签机制双端可用(节点/边标签正确渲染);
2. flow-diagram 静态图渲染: 分层布局目检与 dagre 相当口径(设计文档 §6.1 验收口径,
   非像素对齐),`direction td/lr` 正确;
3. hover 节点 → emphasis 高亮 + tooltip,双端一致;
4. head/tail 箭头字形正确(arrow/diamond/circle/cf-* 至少 arrow 一档);
5. 门禁绿(cargo t + golden;若动引擎 + schema_drift)。

## 执行步骤

- [x] M1 标签发射对照验证(svg text 直通原型 vs overlay;产出决策记录)
  - [✅ 已完成] svg text 直通胜出并落引擎:vue 轨 in_svg_subtree 上下文分流(vue.rs)+ VM 轨 svgdoc text 序列化臂(aura_view_builder.rs);双轨单测 plan502_diagram_tests(2 pass,ui-iced)+ 双端视觉冒烟(vue/vm 截图 Track A 中英文完美渲染)+ schema_drift/a2vue/484/498/499 回归全绿;决策记录回填设计文档 §6.1/§7.4/§10.1;worktree 提交 b7218a6cc
- [x] M2 flow-diagram 组件骨架(数据轨 props + Init 管线 + svg 冒烟)
  - [✅ 已完成] components/flow_diagram.at(props nodes/edges/direction + 等距网格占位布局 + 分桶段记录 + svg 循环发射)+ gallery 页/路由/nav(Diagrams 分组);双端视觉冒烟一致(vue/vm 截图 5 节点网格+连线);顺带两处 parser 修复(record 键 to 上下文化、link 位置参数文本形态[435 起预存 kitchen-sink 解析失败]);golden 基线同步;worktree 提交(见 git log M2)
- [x] M3 Sugiyama-lite 布局核(rank/order/coord + 几何对拍测试)
  - [✅ 已完成] DFS 回边检测 + 最长路径分层 + barycenter 双向 2 轮(stamp 去重)+ 等距/父居中 + td/lr 转置;镜像几何对拍 7 用例(plan502_m3_layout_core_parity)+ 真实组件 e2e(单实例 fixture);顺带两处引擎缺陷修复(SET_ELEM 栈标签丢失/I32_TO_F32 位再转换——float 列表元素写入与泛型 List 读入 typed float 局部两条通路);cargo t 4353 全绿;待澄清#1 落定:group v1 平铺忽略(Phase 2a);worktree 提交 193389c52
- [x] M4 边路由 + 箭头字形(bbox 交点直线 + head/tail 小多边形)
  - [✅ 已完成] bbox 交点直线段(轴满偏除尾差)+ arrow/diamond/circle 实心字形 + line dash/thick;glyphsDg 分桶发射;e2e 裁边精确断言+字形变体数学用例;cargo t 4354 全绿;worktree M4 提交
- [x] M5 标签 + hover 交互(hoverNode 状态机 + emphasis + tooltip)
  - [✅ 已完成] 节点/边标签 svg text(text-anchor middle/边中点)+ hoverNode 状态机(哨兵 999)+ emphasis 平面二态 + 锚定 tooltip + mouse-area 兄弟层命中;行为 e2e + vue 发射断言;双端视觉冒烟通过;cargo t 4356 全绿;worktree M5 提交
- [x] M6 示例页 + 双端验证 + golden 基线
  - [✅ 已完成] vue 轨 svg 内 for 包装 div→template(SVG 命名空间缺陷修复,in_svg_subtree 消费);页面 Interaction 节+P320 口径注记;双端视觉验证(vue LR/TD 双卡完整/VM M5 冒烟);golden 终态同步;cargo t 4356 全绿;worktree 提交 79f3a9fed
- [x] M7 复审与归档准备
  - [✅ 已完成] scoped 复验(cargo check + plan502 8 测试 + schema_drift 2)绿;待澄清三项全落定+执行期发现 7 项记录(5 修 1 债 1 口径);worktree 保持待 /auto-plan:review

## 复审记录

**复审人**: zcode(auto-plan:review) **时间**: 2026-09-01 **基线**: worktree plan-502-dev @ M6 提交 79f3a9fed + merge master(38 commits)后复验

### 逐条验收(verify, don't trust——全部本机重跑)

1. **M1 决策记录 + 标签双端可用 — PASS**:设计文档 §6.1/§7.4/§10.1 回填 diff 核验在案(svg text 直通胜出/overlay 退守 tooltip);`plan502_m1_vm_svgdoc_text_and_overlay`(VM svgdoc text 臂+overlay 对照)+ `plan502_m1_vue_svg_text_passthrough`(子树内直通/子树外 text→span 不变)双 PASS;组件节点/边标签 svg text 落地(flow_diagram.at M5 段)。
2. **静态图渲染 + direction td/lr — PASS**:`plan502_m3_layout_core_parity`(链/菱形/环回边剥离/降交叉/父居中/td-lr 转置/双跑确定性 7 用例)+ `plan502_m3_layout_geometry_e2e`(真实组件手算几何,td/lr 各一跑)PASS;dagre 相当口径的目检由执行期 M2/M6 双端截图支撑(commit 记录在案)。
3. **hover → emphasis + tooltip 双端一致 — PASS**:`plan502_m5_hover_and_labels_e2e`(hover 触发 emphasis/锚定 tooltip/NodeOut 复原哨兵)+ `plan502_m5_vue_emission`(@mouseenter/@mouseleave 落 SFC)PASS;hoverDg 状态机/哨兵 999/emphasis 双分支代码核验一致。
4. **head/tail 字形 — PASS(超额)**:`plan502_m4_glyph_variants`(diamond 四点/circle 双弧数学)+ M4 e2e arrow 精确裁边断言;arrow/diamond/circle 三档 + line dash/thick 齐备,超出"至少 arrow 一档"口径;cf-* v1 别名 circle(组件头注在案)。
5. **门禁绿 — PASS(带披露豁免)**:plan502 scoped 8/8;`cargo tv` 3494/3494;`cargo tf` 3350/3350(含 schema_drift/docs_gen/component_registry + 1M churn 档);gallery_golden 重采样后复跑绿;`cargo t` 4373/4374——唯红 `plan055_strip_html_tests` 经本机 **master 同红复证**为 master 既有(musk PLAN-055 批 c964ffa81 引入面,tf 档无 ui-iced 不触发),非本批回归,记债 **P502-1**。

### 复审动作(基线同步)

- worktree 基线 e00a7f458 落后 master 38 commits——**原基线上 `cb_asynchronous_channel`/`cb_devtools_log_error` 两红为缺 master P499-7 清偿(7a8ac1d2e)所致,非 502 回归**(master 单跑该测试 PASS 复证)。已 merge master 入 plan-502-dev(冲突 2 处:lib.rs 测试模块双留、gallery golden 以 `GALLERY_GOLDEN_UPDATE=1` 重采样后复跑绿),同步提交 1913ef3df + 复审基线同步提交。
- 执行期发现#9(kitchen-sink `@autodown/engine` scaffold 依赖缺)核验为**已上游愈合**:P499-6 清偿(d0c23388d)kitchen-sink 再生成后 import 已不存在,无需立债。

### 遗漏/延后/workaround 扫描

- M1–M6 六提交与计划步骤一一对应(M7=簿记),无丢项;设计文档回填、golden、页面/路由/nav 均在 diff 中。
- 待澄清①②③落定记录在案;group 平铺(Phase 2a)/focus 模型(Phase 2a)/DSL 静态糖(Phase 3)均为计划文本**显式排除项**,非静默裁剪。
- P320 双卡共享根状态约束以 e2e 单实例 fixture + 页面口径注记双缓解在案(执行期发现#10,平台预存约束)。
- 哨兵 999 系 498 P498-1 惯例,非 workaround;无 TODO/hack 残留扫描通过。

**结论**: 5/5 验收 pass(验收 5 带披露豁免,债 P502-1 为 master 既有)。status → **reviewed**,交接 `/auto-plan:merge`(worktree 与分支保留)。

## 待澄清事项

1. **group 嵌套布局**: ✅ 执行中落定——v1 平铺忽略 group 字段(预案采纳),
   递归超节点归 Phase 2a;
2. **emphasis 模型**: ✅ 平面二态(498 同款),focus 模型归 Phase 2a;
3. **示例落位**: ✅ widgets-gallery 新页(/flow-diagram,Diagrams 分组),
   与 charts 四页同构;未占 examples/ui/ 0xx 编号。

## 执行期新增发现(均已就地解决)

4. **engine/SET_ELEM 丢栈标签**(M3):`xs[i] = 84.0` float 位模式当 Int 存——
   pop_i32+Value::Int 改 decode_tagged_nv(与 SET_FIELD 同型);回归在
   plan502 布局对拍套内;
5. **engine/I32_TO_F32 位再转换**(M3):泛型 List 元素读入 typed float 局部
   时已 float-tag 位被二次 int→float——tag 驱动透传修复;
6. **parser/record 键 `to`**(M2):Plan 162 方法关键字作 edge schema 键被拒——
   键位上下文化(Type/Tag/Task 先例);
7. **parser/link 位置参数**(M2):`link "label" {}` 不支持 → kitchen-sink
   自 435 起解析失败 → gallery vue 轨整站白屏(预存债)——位置文本形态
   修复 + 回归测试;
8. **vue/svg for 包装 div**(M6):svg 子树内多语句 for 被包 <div> 破坏
   SVG 命名空间(rect/text 不渲染)——in_svg_subtree 时改 <template v-for>;
9. **vue 轨 gallery 预存损坏(未修,非本批引入)**:kitchen-sink 的
   `@autodown/engine` import 不在脚手架依赖 → vite 编译失败 → 直连路由
   fallback 首页(导航点击可达)。与 502 无关,建议另立债务条目;
10. **P320 双卡 Init 序非树序**(M3/M6):同名组件双实例共享根状态,页面
    双卡末写者不确定——e2e 以单实例 fixture 断言;页面双卡演示口径注记。
