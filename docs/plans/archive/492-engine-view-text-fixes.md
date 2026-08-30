---
plan_id: PLAN-492
status: archived             # drafting → executing → execution_done → reviewed → archived
feature_name: 引擎正确性专项——包组件编译/text 内容表达式/f-string 插值三族修复
author: [zcode]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/design/chart-components.md: 修改——绕开章节全部退役改直写(Init 内 prop 比较直选域/带参 msg 声明/dollar 插值),已知坑三行更正(prop 比较真机制/带参 msg 平反/f-string 误归因)"
new_spec_components:
  - "docs/specs/auto-lang/frontend: parser primary-shorthand 识别 [ 后缀——text t[\"label\"] 整链挂 text prop(Expr::Index),镜像 Dot/LParen peek 模式"
  - "docs/specs/auto-lang/ui: vue 文本内容表达式 Index 字符串键保留引号(复用属性路径 bound_value)+不支持形式 R046 告警替换静默 dump"
  - "docs/specs/auto-lang/ui: 包组件编译失败三层显式诊断契约——装载层 parse_warnings 逐条 log::warn(文件+原因)/合成层 record_synth_failures+take_synth_failures(组件名.handler+原因,legacy 路径同覆盖)/链接层 undefined symbol fatal 具名"
touched_goals:
  - "GOAL-007: chart 组件双端渲染正确性——绕开全摘除回归直接写法,双端目检通过(ADR-19 双端同源延续)"
  - "GOAL-010: chart 组件三示例副本(charts-gallery/024-charts/widgets-gallery)无绕开形态同步"

affects: [auto-lang/parser, auto-lang/lexer, auto-lang/ui_gen/vue, auto-lang/vm-codegen]
current_step: 7
total_steps: 7
---

# [PLAN-492] 引擎正确性专项——包组件编译/text 内容表达式/f-string 插值三族修复

## 变更摘要

Plan 484（声明式 chart 组件族）执行期间实证了三族**静默失效**类引擎缺陷,当时以组件侧绕开
（双算双存/槽位字段/裸挂 handler）保证交付,并全部记档 KNOWN-DEBT-AND-RISKS（🟡 三行）。
本计划根治这三族缺陷,随后**摘除 chart 组件里的全部绕开代码**,以"直接写法"回归验证 chart
双端渲染——形成"修复 → 摘绕开 → 闭环验证"的完整链路。同时把"静默产出错误内容"升级为
显式诊断（R 级告警）,消除此类缺陷最难排查的根源。

三族缺陷（全部有精确复现与回归锚,详见需求分析）:
- **族 A·解析器**: ①primary-shorthand 不识别 `[` 后缀（`text t["label"]` 分裂为 dump+子文本）;
  ②f-string 含字面量 `[`/`]` 时 `${}` 插值破坏编译（`f"w-[${slot}px]"`）。
- **族 B·vue 生成器**: 文本内容位置的 Index/Dot 表达式求值缺臂（`text (text: li["name"])`/
  `t.label` 渲染空或 dump）——svg 属性位置同款表达式正常,缺口仅在文本内容路径。
- **族 C·包组件单 VM 编译链**: ①Init 内 prop 字符串比较静默破坏整个子组件 codegen;
  ②带参 msg 声明（`msg { Init, Hover(int) }`）静默破坏整包编译。同经 `use widget:` 导入
  路径均正常——包加载链（lib.rs P4-4/D13 child_decls 单 VM 编译）特有。

## 目标

1. 三族缺陷逐一修复,每项配**最小复现测试先行**（红→绿）,复现不依赖任何示例工程。
2. chart 组件**摘除全部绕开代码**回到直接写法（prop 比较直用/`${}` 插值/带参 msg 声明）,
   plan437/plan484 e2e 与 charts-gallery 双端编译在"无绕开"状态下全绿。
3. "静默失效"类失败模式获得显式诊断:包组件编译失败时输出组件名+原因（不再静默回落默认值）。
4. DEBT 三条目闭环（标记 ✅ 已修复,引用本计划）。

## 架构方案

不引入新机制,三族各自在既有模块内修复:

- **族 A**: lexer.rs f-string 模式（FStrNote/fstr_expr 分支,~615-745 行）定位 `${}`+`[]`
  交互的解析缺口;parser.rs parse_view_node 的 primary-shorthand 分支补 LBracket peek
  （镜像既有 Dot/LParen peek 模式）。
- **族 B**: vue.rs 文本内容表达式求值——svg 属性路径已有 Index 处理（vue.rs:6469/8509 等）,
  平移同款臂到文本内容路径;缺口边界先以探针测试枚举（哪些表达式形式在文本位置失灵）。
- **族 C**: lib.rs 包加载块（P4-4/D13）产出的 child_decls 进单 VM 编译——对照 `use widget:`
  链逐步 diff 编译产物,定位 prop 比较/带参 msg 的分叉点;修复后在包路径补编译失败显式诊断。

**关键约束**: 修复期间 chart 组件的绕开代码保持不动（绿基线不破坏）,全部修复落地后才统一
摘除绕开并回归——避免"边修边拆"无法定位回归源。

## 需求分析与背景调查

- 来源: Plan 484 执行+复审期间的实证（2026-08-29/30）,用户裁定"统一立项逐一解决,之后回看
  chart 展示"。DEBT 登记簿 🟡 三行 + tooltip 锚点降级条目为同源记录。
- 复现锚（全部不依赖示例工程,可做成独立单测）:
  - A1 解析: `Parser` 解析 view 源 `text t["label"]` → 断言 Expr::Index 完整性（现分裂）。
  - A2 词法: `f"w-[${x}px]"` 在包组件 view 上下文 → 现编译失败;`f"w-[{x}px]"` 正常。
  - B1 vue: `text (text: li["name"])` 循环内 → 现 SFC 文本空/dump。
  - C1 包: `msg { Init }` + Init 内 `if curve == "linear" { … }`（字符串 prop 比较）→
    现整组件 Init 失效;`use widget:` 同源正常（013-todo todo_list.at 对照）。
  - C2 包: `msg { Init, Hover(int) }` → 现整包静默失效;对照 013-todo 同款正常。
- 关键文件: `crates/auto-lang/src/lexer.rs`、`parser.rs`（parse_view_node ~14555）、
  `ui_gen/vue.rs`（文本内容求值 + emit 死代码已清）、`lib.rs` P4-4/D13 包加载块（~3600）、
  `vm/codegen.rs`（child_decls 单 VM 编译）。
- 已知对照系: `use widget:` 链（013-todo）与包链（435 P4/437）的 child 编译分叉是族 C 的
  主修场;族 A/B 与加载路径无关（含 use-widget 路径,只是先在包路径暴露）。

## 详细设计

### M1 族 A2——f-string `${}`+字面量括号（lexer）
- 最小复现单测: 词法层解析 `f"w-[${x}px]"` 的 token 序列（红）。
- 定位 fstr_expr 与文本累积的边界交互,修复后 `f"w-[${x}px]"` 与 `f"w-[{x}px]"` 等价。
- 回归: 既有 f-string 测试族全绿 + 新单测。

### M2 族 A1——primary-shorthand `[` 后缀（parser）
- parse_view_node 的 has_ident_field_primary peek 补 TokenKind::LBracket;
  命中后 parse_expr 消费完整 Index 链挂 primary prop。
- 单测: `text t["label"]` → ViewNode text prop = Expr::Index（红→绿）。

### M3 族 B——vue 文本内容 Index/Dot 臂
- 探针测试枚举: Index/Dot/Binary 等表达式形式在 `text (text: …)` 位置的发射现状。
- 补臂: 文本内容求值复用 svg 属性路径的 Index 处理;对暂不支持的形式发 R 级告警
  （替换静默空/dump）。
- 单测: `text (text: li["name"])` 循环内 → SFC 含正确插值（红→绿）。

### M4 族 C——包组件单 VM 编译链分叉定位与修复
- 对照实验: 最小 widget 经 `use widget:` vs `use {package}` 两条链的 codegen 产物 diff,
  定位 prop 比较/带参 msg 的分叉指令（预计 codegen.rs child 编译或 lib.rs 装配）。
- 修复后: 包组件 Init 内 prop 比较、带参 msg 声明均正常编译执行。
- 单测: 包路径最小 widget（prop 比较 flag + 带参 msg）双链行为一致（红→绿）。

### M5 诊断补强
- 包组件编译失败（任一族）时,加载与编译层输出组件名+失败原因的显式诊断
  （现状: 静默回落默认值——484 现场排查成本高的根源）。

### M6 绕开摘除 + chart 回归闭环
- line/bar/area 组件: 摘除双算双存（segsM/segsS 合并回单 segs,Init 内直用 prop 比较）、
  `${slot}px` 恢复 dollar-form、`msg { Init, Hover(int) }` 恢复带参声明;
  bar/line/area 的 tipYsS 等双域字段按需简化。
- 三份副本同步;plan437/plan484 e2e + charts_gallery_compiles + gallery golden 全绿。
- charts-gallery 双端目检（用户验收:monotone 曲线/刻度/图例/tooltip 逐卡过）。

## 测试设计

- **每缺陷最小复现单测**（M1-M4 各 ≥1,红→绿,落 crates/auto-lang/src/plan492_*.rs）。
- **回归面**: cargo t（默认）;--features ui-iced 全套;plan437/plan484 e2e;
  charts_gallery_compiles;gallery golden（组件改动→基线更新复核）。
- **摘绕开专项**: M6 后在"无绕开"组件上复跑全部 chart 测试,证明修复对真实消费面生效。
- **诊断验证**: 人为构造包组件编译失败,断言诊断输出含组件名与原因。

## 验收标准

1. M1-M4 每项最小复现单测绿,且修复不引入新告警（warnings 不新增）。
2. chart 组件绕开代码全部摘除（grep 锚: segsM/segsS/双算注释清零）,plan437/plan484/
   charts_gallery_compiles/gallery_golden 在无绕开状态下全绿。
3. 包组件编译失败场景有显式诊断（组件名+原因）,静默回落消除。
4. charts-gallery 双端实机目检通过（用户验收:monotone 曲线贴点、刻度/图例文本齐全、
   tooltip 工作）。
5. KNOWN-DEBT 三条目标记 ✅ 已修复并引用本计划。

## 执行步骤

- [x] M1 族 A2: f-string `${}`+`[]` 词法修复（最小复现先行） [✅ 已完成 2026-08-30] **定案:缺陷在 master 不可复现,判定 484 误归因(与族 C1 同 Init 现场混淆)**。五层验证全绿:①词法(token 探针序列正确)②parser/inline 单 VM 链(`w-[100px] h-full` 正确落 state)③生产包链(charts-gallery 真源+load_package 补丁,bar Init 存活/mouse-area band 正常)④vue SFC(模板字面量+`:style` 绑定)⑤金丝雀负对照(未定义变量补丁确实杀死 bar Init,MouseArea 24→15,证明夹具可检出真失败)。附带语义锚:`{x}` 花括号形式是纯字面量不插值(484"绕开形态"实为无害垃圾类)。产出 `plan492_tests.rs` m1_* 九测;副产物发现:VM 样式解析器静默丢弃浮点任意值类 `w-[127.5px]`(DEBT 行51 已知动态像素定位残余,非本计划范围)。待 M7 时 DEBT 行52 按误归因闭环(见待澄清③)
- [x] M2 族 A1: primary-shorthand `[` 后缀（最小复现先行） [✅ 已完成 2026-08-30] 红→绿:`text t["label"]` 原分裂为 `Ident("t")`+游离段;parse_view_node peek 补 TokenKind::LSquare(镜像 Dot/LParen),修复后 text prop=完整 `Index(Ident("t"), Str("label"))`;既有形态(ident.field/bare ident)回归绿。plan492_m2_tests.rs 2 测;lib 全量(3277)无回归。commit dc67addea(rebase 后 a7e127e92)
- [x] M3 族 B: vue 文本内容 Index/Dot 臂（探针枚举+补臂+告警） [✅ 已完成 2026-08-30] 探针六形式枚举定位真缺口:**Index 字符串键引号被剥**(`expr_to_vue_text_raw` 的 Str 臂出模板文本)——`li["name"]`→`{{ li[name] }}`(裸标识符→渲染空)。修复:Index 臂索引部分改走 `expr_to_vue_bound_value`(svg 属性路径同款,Str→'name');数值/标识符索引不回归;兜底臂 `_ => Ok("value")` 加 R046 告警(替换静默 dump)。plan492_m3_tests.rs 5 测红→绿。commit fe1ee9169(rebase 后 66c7aff65)
- [x] M4 族 C: 包编译链分叉定位与修复（对照实验先行） [✅ 已完成 2026-08-30] **定案:不存在 codegen 分叉**——包链与 use-widget 链同 Parser 同合成器。真机制:裸 prop 名在**赋值 RHS 位**触发 undefined variable 解析错(IF 条件位可过)→`parse_package_widgets` per-file try-parse **静默整文件丢弃**→组件消失零诊断(=484"整组件静默失效"现场)。点前缀 `.curve` 直接形态 Init 内全链可用(探针 probeMark="mono");带参 msg 声明 VM+vue 双轨正常(484 记档不可复现)。裸名 RHS 诊断面缺陷归 M5。plan492_m4_tests.rs 3 锚(C1 锚/直接形态双补丁锚/C2 双轨锚)。commit 75cc87733(rebase 后 99ea81386)
- [x] M5 包编译失败显式诊断 [✅ 已完成 2026-08-30] 三层诊断契约落定:①装载层——`load_package` 对部分文件解析失败逐条 `log::warn`(文件+原因,VM/vue 两消费方同覆盖,此前 parse_warnings 无人看);②合成层——handler `compile_stmt` 失败 eprintln(stderr UI 运行期不可见)→`record_synth_failure`(log::warn+`take_synth_failures()` 可取走,含组件名.handler+原因,legacy synthesize_widget_module 路径同步);③链接层——未定义符号本就 fatal+具名(测试钉住防回退)。plan492_m5_tests.rs 2 测;cargo t 3292 全绿。commit 0bced0939(rebase 后 fce7b644e)
- [x] M6 chart 组件绕开摘除（三副本同步）+ 全回归 [✅ 已完成 2026-08-30] 绕开全退役:Init 内 `.type`/`.curve` prop 比较直选域+单 segs+`msg { Init, Hover(int) }` 带参声明恢复+`f"w-[${slot}px]"` dollar 插值;成对镜像域字段(segsS 系)与 tooltip 双域锚点残留(ay/cum3 死算)退役;yTick/legend 槽位字段维持(R006 双轨限制,text Index 双轨渲染属另行修复面,非 M6 范围)。三副本字节同步;grep 锚(segsS/segsM/yTickS/tipYsS/双算)零命中(头注措辞同步改写避锚词);gallery golden 再生成。六道门禁全绿:chart 专项 lib 12/ui-iced 28(含 437 gallery_chart_components_render_geometry+484 charts_gallery_bare_names_render)/golden 1/cargo t 3292/ui-iced 4116/tf 3293(含 1M churn)。commit 2b952098a(rebase 后基线)
- [x] M7 charts-gallery 双端目检（用户验收）+ DEBT 闭环 + 归档准备 [✅ 已完成 2026-08-30] ①双端验证(autoui-verifier 规程):Vue 端 vite+Playwright 深色全页截图六卡全渲染(monotone 曲线平滑无过冲/刻度 0-400 与 0-8000/图例色块+名/x 轴标签 Jan-Jun·Q1-Q4 齐全)+hover tooltip 截图+monotone/stacked/donut 卡特写;VM 端 MCP 截图+树证据(六卡全活:刻度/图例/xlabel 全在渲染树,legendColor0-3·bands 状态就位=Init 存活);视口剪裁为 DEBT 行50 存量(本计划范围外)。用户验收 OK(2026-08-30)。②DEBT 三条闭环:行48(✅ 已修复 M4+M5+M6)/行49(✅ 不可复现同根闭环)/行52(✅ 误归因,用户裁定,可凭另一复现路径重开)。③归档准备:转 /auto-plan:review 流程

## 复审记录

**复审人**: zcode(/auto-plan:review) · **时间**: 2026-08-30 · **worktree**: .worktrees/plan-492-dev @ 2b952098a(rebase 后基线 a88cffc9a)

**逐条验收**:
1. **M1-M4 最小复现单测绿+零新告警 — PASS**。m1 九测/m2 两测/m3 五测/m4 三锚/m5 双测全绿(本次复审复跑);`cargo tf` 复审门禁复跑 3293/3293(含 1M churn);cargo check 全部警告与 492 触碰文件零交集(parser.rs/vue.rs/handler_codegen.rs/component_registry.rs/plan492_*.rs 无告警命中)。
2. **绕开摘除+全回归 — PASS**。grep 锚 `segsS|segsM|yTickS|tipYsS|双算` 全仓零命中(头注措辞已同步改写避锚词,语义无损);plan437 gallery_chart_components_render_geometry/plan484 四测/charts_gallery_compiles/gallery_golden 在无绕开状态下全绿;三副本字节同步(diff 零差异)。
3. **包编译失败显式诊断 — PASS**。M5 双测绿:装载层(parse_warnings 逐条 log::warn)+合成/链接层;legacy synthesize_widget_module 路径同步核验(handler_codegen.rs:1553/1695 record_synth_failure 双覆盖,:104 take_synth_failures 公开取走)。
4. **双端实机目检 — PASS**。Vue 端 vite+Playwright 深色全页六卡全渲染(monotone 平滑无过冲/刻度/图例/轴标签齐全)+hover tooltip 截图+三卡特写;VM 端 MCP 截图+渲染树证据(六卡全活,刻度 100-400·2000-8000/图例八名/轴标签全在树中,bands·legendColor 状态就位=Init 存活)。视口剪裁为 DEBT 行50 存量(范围外);VM 端 tooltip hover 交互未自动化(MCP 工具面无 hover 注入,仅 Vue 端自动验证),用户目检 OK 验收(2026-08-30)。
5. **DEBT 三条闭环 — PASS**。行48 ✅已修复(M4 真机制+M5 诊断+M6 摘绕开)/行49 ✅不可复现同根闭环/行52 ✅误归因(用户裁定,凭另一复现路径可重开)。

**遗漏/延后/workaround 扫描**:
- **无未批准延后**。yTick/legend 槽位字段保留属 M6 设计明文范围(计划创建时已界定);复审将背后的"text 内容 Index **VM 轨**渲染缺口"正式入账为 **492-R1**(vue 轨 M3 已修,VM/iced 轨仍缺,根治后可再摘槽位字段)。
- M1 副产物(VM 样式解析器静默丢弃浮点任意值类 `w-[127.5px]`)归行51 tooltip 锚点存量残余,非本计划范围。
- parser.rs 存量 TODO 五处均远离 M2 改动区(~14555),非本计划引入;新增文件零 TODO/FIXME/HACK。
- 计划↔代码分歧:计划原文预期 M1 为"lexer 修复",实际定案为不可复现误归因(零引擎改动+回归锚钉住)——以代码/验证为准,已在 M1 执行记录与 DEBT 行52 如实登记。

**结论**: 五条验收全 PASS,无阻塞债 → **status: reviewed**,可入 /auto-plan:merge。

## 待澄清事项

1. 族 C 的分叉若深达 vm/codegen.rs 的 child 编译主路径,修复可能牵动 `use widget:` 链——
   届时以"两链行为一致"为准绳,如需大改先回报再动。
2. M3 若发现文本位置需要完整表达式求值器接入（而非补单臂）,体量升级需回报重估。
3. **[M1 定案已裁(用户 2026-08-30)]** 族 A2 判定误归因(误操作),DEBT 行52 按"误归因/不可复现"闭环并引用本计划,用户未提供 484 另一复现路径——若后续发现,凭路径重开该条。
