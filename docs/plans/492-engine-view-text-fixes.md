---
plan_id: PLAN-492
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 引擎正确性专项——包组件编译/text 内容表达式/f-string 插值三族修复
author: [zcode]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/parser, auto-lang/lexer, auto-lang/ui_gen/vue, auto-lang/vm-codegen]
current_step: 0
total_steps: 6
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

- [ ] M1 族 A2: f-string `${}`+`[]` 词法修复（最小复现先行）
- [ ] M2 族 A1: primary-shorthand `[` 后缀（最小复现先行）
- [ ] M3 族 B: vue 文本内容 Index/Dot 臂（探针枚举+补臂+告警）
- [ ] M4 族 C: 包编译链分叉定位与修复（对照实验先行）
- [ ] M5 包编译失败显式诊断
- [ ] M6 chart 组件绕开摘除（三副本同步）+ 全回归
- [ ] M7 charts-gallery 双端目检（用户验收）+ DEBT 闭环 + 归档准备

## 复审记录

（复审时填写）

## 待澄清事项

1. 族 C 的分叉若深达 vm/codegen.rs 的 child 编译主路径,修复可能牵动 `use widget:` 链——
   届时以"两链行为一致"为准绳,如需大改先回报再动。
2. M3 若发现文本位置需要完整表达式求值器接入（而非补单臂）,体量升级需回报重估。
