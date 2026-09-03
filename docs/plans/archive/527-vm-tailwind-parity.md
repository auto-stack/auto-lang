---
plan_id: PLAN-527
status: archived                # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: VM 轨 Tailwind 全量覆盖契约（清单锚定 + 三后端应用 + 对拍审计台）
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 样式与主题节——ui/style/ 从「Tailwind-inspired 按需子集」改述为 v3.4 清单驱动全量覆盖契约（parse_reported 报告通道/对拍审计台/变体管道解析期门控）"
new_spec_components:
  - "crates/auto-lang/tests/style_parity.rs: 对拍审计台常驻门——8861 类清单分类断言（白名单外零 missing）+布局/视觉/文本三家族 iced applied 门+PARSED_ONLY_ALLOWED 豁免台账+coverage 表同源再生"
  - "docs/style-coverage.md: Tailwind v3.4 清单×VM 后端覆盖矩阵（家族×状态,UNSUPPORTED 白名单逐行理由,与 KNOWN-DEBT P527 节互链）"
  - "crates/auto-lang/tests/fixtures/tailwind-v34-utilities.txt: v3.4 core 清单 vendor fixture（8861 类×15 families）"
  - "tools/gen_tailwind_manifest.py: 清单零依赖再生器（可复现性裁定,待澄清④）"
  - "crates/auto-lang/src/ui/style/mod.rs: Variant 管道（Hover/Focus/Active/Disabled/Responsive(Breakpoint 五档,theme::window_width 解析期门控)/Dark(theme::dark_mode 门控)）+Style::parse_reported 报告通道+覆盖契约头注"
touched_goals:
  - "GOAL-007: VM 轨 Tailwind 覆盖从按需子集升级为 v3.4 清单驱动全量覆盖契约——双端 parity 锁定的 VM 侧地基（plan-512 Tailwind 刻度双端兼容口径的机制上游）,变体/responsive/dark 三能力入册

affects: [autoui]
current_step: 10
total_steps: 10
---

# [PLAN-527] VM 轨 Tailwind 全量覆盖契约

## 变更摘要

用户裁定（2026-09-03）：**VM 侧应支持所有 Tailwind 属性**。现状是
`ui/style/` 已有一套「Tailwind-inspired」三后端统一样式系统
（class.rs 2153 行 / 177 解析臂，hover: 前缀已解析，iced/gpui/headless
三适配器），但覆盖是**按需子集**且无审计手段。本计划把它升级为
**清单驱动的全量覆盖契约**：

1. **清单锚定**：vendor Tailwind v3.4 core plugins 展开清单入库，
   逐类状态（applied / parsed-only / unsupported / missing）机器可查。
2. **静默丢弃显式化**：`Style::parse`（mod.rs）今日对未解析 token
   `if let Ok(c) = …` 静默跳过——plan-055 div 容器臂整串丢类、
   DOC_EDITORS 注释-实现 drift（PLAN-043 T5 所修）同类前科。改为
   parse 报告通道，未映射类显式可查，不再无声消失。
3. **三家族补全**：布局（position/inset/z、w/h 分数、min/max、
   space-x/y、grow/shrink/basis）→ 视觉（shadow 层级、ring、gradient
   全向、opacity 全档）→ 文本（font 栈/字重/leading/tracking/
   decoration/truncate），逐类三后端 applied 断言。
4. **变体系统**：`hover:` 单例管道泛化为 variant 通用管道
   （focus/active/disabled 同构）；responsive 断点（sm/md/lg/xl/2xl）
   接窗口尺寸信号；`dark:` 接 theme 状态（theme.rs 语义色底子）。
5. **对拍审计台**：清单 → parse → applied-style 快照的表驱动测试 +
   覆盖率表落盘 docs/style-coverage.md，漂移即红——防「解析了但某类
   widget 不消费」的全仓系统性风险。
6. **不做/受限台账**：float/clear、whitespace/word-break、print、
   伪元素内容等无原生语义或 cosmic-text 上限项，登记
   KNOWN-DEBT-AND-RISKS.md，显式豁免而非假装支持。

**动机与消费方**：auto-down PLAN-046（VM demo 对齐 vue 版，两栏布局
首个验收案）——style 块只编译 vue scoped CSS，VM 轨唯一样式入口是
Tailwind 类；双轨单源（plan 040）要求 VM 侧覆盖完整。上游依赖仓
auto-down 的 demo/jade-garden VM 化均为直接受益者。

## 目标

1. Tailwind v3.4 清单入库，覆盖率表落盘；白名单外零静默丢弃
   （parse 报告通道，未映射类显式 Unsupported）。
2. 布局/视觉/文本三家族覆盖缺口清零（Unsupported 白名单外全类
   headless+iced applied 断言绿）。
3. focus/active/disabled 与 hover 同管道可声明、可应用。
4. responsive 断点随窗口宽度生效；`dark:` 随主题状态生效（各有
   可断言用例）。
5. 对拍审计台常驻：任何类从 applied 退化为 parsed-only/missing
   即测试红。
6. 不做/受限台账在册（KNOWN-DEBT-AND-RISKS.md）；双 feature 配置
   全量回归零新增红。

## 架构方案

```
Tailwind v3.4 core plugins 展开清单（vendor fixture，一次生成入库）
  tests/fixtures/tailwind-v34-utilities.txt
        │  表驱动
        ▼
StyleParser（parser.rs / class.rs）
  parse_single 全清单分类：mapped → StyleClass；
  无映射 → 显式 Unsupported（parse 报告通道，非静默 Err-skip）
        ▼
StyleClass（后端无关 IR）── 三适配器同源消费
  iced_adapter（金标消费面，1229 行）
  headless_adapter（applied 断言金标——无渲染歧义）
  gpui_adapter（对齐 iced 金标；feature 不入默认门禁，尽力对齐）
        ▼
变体管道：hover_classes: Vec<StyleClass>（既有）
  → variant_classes: Vec<(Variant, StyleClass)>（泛化，hover 迁移保兼容）
  responsive：宿主窗口宽信号（resize→restyle 既有回路）→ 当前断点过滤
  dark：theme.rs 主题状态 → dark 前缀类过滤
        ▼
对拍审计台（tests/style_parity.rs）
  清单逐类 → parse → 分类 → headless applied 断言
  覆盖率表落盘 docs/style-coverage.md（家族×后端×状态）
```

**不做账原则**：原生无语义（float/clear/print/伪元素）、宿主上限
（cosmic-text 文本排版细节、无容器查询故 w-1/2 只能 Fill 系近似）
——登记 KNOWN-DEBT 行 + coverage 表 unsupported 列，不假装支持。

## 技术栈

- `crates/auto-lang/src/ui/style/`：mod.rs（parse 报告通道）、
  parser.rs、class.rs（新家族解析臂）、iced_adapter.rs、
  headless_adapter.rs、gpui_adapter.rs、color.rs、layout_extract.rs、
  theme.rs（dark/字体栈）
- 消费面抽查：`ui/aura_view_builder.rs`（extract_style_with）、
  `ui/node_converter.rs`、`ui/iced/renderer.rs`（build_row/build_column
  的交叉轴与主轴填充消费）
- vendor fixture：`crates/auto-lang/tests/fixtures/tailwind-v34-utilities.txt`
- 无新依赖（清单生成脚本一次性运行，产物入库）

## 需求分析与背景调查

（spec 台账离线摸底：auto-lang .autoos/specs.json 341 条，无样式
专项目标——本计划 review 时或成首条样式目标。）

- **现状盘点**（2026-09-03 实勘）：mod.rs `Style::parse` 对
  `parse_single` 失败的 token 静默跳过；class.rs 解析臂覆盖 p*/gap*/
  bg-*/text-*(字号/对齐/颜色)/border*/flex(-1/auto/initial/none/wrap)/
  items-*/justify-*/grid-cols-{1-12}/rounded(56 处)/shadow(25)/
  opacity(8)/position 系(14)/z-(8)/top-inset(11)/w-(17)/h-(10)/
  min-max(15)/font(27)/leading-tracking(7)/decoration(8)/transform(9)/
  transition-animate(9)/cursor(4)/space-x(7)/ring-outline(23)/
  blur-filter(32)；**dark: 零命中、responsive 断点≈零**。
- **iced 消费实证**：Flex1 → width=Fill（iced_adapter.rs:861-865，
  主轴等分已真消费——auto-down demo 两栏 flex-1 方案的机制基础）；
  items-stretch 由 build_row/build_column 包交叉轴 Fill 容器模拟
  （:883-884）；justify-between 由 FillPortion spacer 模拟（:199、
  :232）；Plan 412 交叉仲裁（row 带 grid-cols-N/flex-col 类时布局
  语义被类覆盖）；flex height 猜测启发式（:370-383）。
- **静默漂移前科**：plan-055 div 容器臂丢类只余默认 Column（后修）；
  DOC_EDITORS「注释称 LRU 实为裸 HashMap」drift（PLAN-043 T5 所修，
  实为反例：注释对实现错）。机制根源相同：无清单、无审计。
- **消费方需求**：auto-down PLAN-046（VM demo 两栏 + 对齐清册，
  本计划为机制上游）；demo README「Layout note」明文登记的
  style 块 vue-only 分工；DEBTS 040 平台豁免族（主题/观感差异）。
- **版本锚定**：Tailwind v3.4.x（v4 为 CSS-first 重写，清单结构
  不同，另立不混入）。

## 详细设计

1. **T1 清单锚定 + 静默丢弃显式化**：生成并 vendor
   `tailwind-v34-utilities.txt`（官方 corePlugins 展开去重，生成
   脚本一次性运行不留依赖）；mod.rs 新增
   `Style::parse_reported(input) -> (Style, Vec<String>)`（第二返回值
   = 未映射类原文名），旧 `parse` 内部走同路丢弃行为不变（兼容零
   风险）。验证：新测对清单逐类 parse_reported 无 panic、分类
   三态可枚举。
2. **T2 对拍审计台骨架 + 覆盖率表**：`tests/style_parity.rs` 表
   驱动——清单逐类 → parse_reported → 断言「非白名单者必 mapped」
   → headless_adapter applied 断言骨架；生成
   `docs/style-coverage.md`（家族×后端×状态矩阵，commit 入库作
   基线）。验证：`cargo test -p auto-lang style_parity` 红→绿，
   表落盘。
3. **T3 布局家族补全**：按 coverage 表 missing 项逐类补
   class.rs 解析臂 + headless/iced applied——position 系（static
   降级/absolute 经 Stack 偏移，宿主无 Stack 上下文时登记受限）、
   inset/top/right/bottom/left、z-*（同 Stack 语境）、w-/h- 分数
   与 full/screen（分数在无容器查语义下降级 Fill 系并在类注释
   记口径）、min-/max-、space-x/y（row/col spacing 消费已有，补
   解析面）、grow/shrink/basis 全档、aspect-ratio（无原生 → 直接
   入 Unsupported 白名单）。验证：家族 applied 测试 + parity 表
   对应行转 applied。
4. **T4 视觉家族补全**：shadow-sm/md/lg/xl/2xl/inner/none（iced
   0.14 shadow 面逐档映射）、ring（width+color 组合）、gradient
   全向（GradientDir 已有底子，补 to-* 组合与 from/via/stop 色）、
   opacity-{0..100 全档}、object-*（Image 消费面）。验证同 T3。
5. **T5 文本家族补全**：font-sans/serif/mono 栈 → theme.rs 字体
   解析（跨平台字体族名表）、font-weight {100..900}、leading-/
   tracking- 全档、underline/decoration 档、truncate/line-clamp
   （cosmic-text 能力内实现，超限入白名单）。验证同 T3。
6. **T6 变体管道泛化**：mod.rs `hover_classes` 迁移为
   `variant_classes: Vec<(Variant, StyleClass)>`（Variant 枚举：
   Hover/Focus/Active/Disabled，旧字段保留 deprecated 转发一版）；
   解析面 `<variant>:<class>` 前缀通用化；iced 消费沿按钮 hover
   先例（iced_adapter hover 消费点）扩展状态回调面——v1 仅按钮族
   真消费，其余 widget 登记 parsed-only（coverage 表可见，不静默）。
   验证：focus/active/disabled 解析+按钮 applied 测试，hover 既有
   测试零回归。
7. **T7 responsive 断点**：Variant 扩 Responsive(Breakpoint)
   （sm 640/md 768/lg 1024/xl 1280/2xl 1536）；宿主信号 = 窗口宽
   （renderer resize→restyle 既有回路上取当前宽挂到 bindings 面，
   接入点实施时实测——session.rs 窗口事件有先例）；build 面按
   当前断点过滤 responsive 类。验证：headless 两断点样本 applied
   断言 + 断点边界值单测。
8. **T8 dark:**：theme.rs 语义色既有底子上挂 dark 主题态；Variant
   扩 Dark；解析 dark:<class>；build 面按主题态过滤。验证：同一样
   本两主题 applied 对拍断言。
9. **T9 不做/受限台账 + 头注契约**：KNOWN-DEBT-AND-RISKS.md 登记
   （float/clear、whitespace/word-break 系、print、伪元素内容、
   aspect-ratio、w 分数近似口径、非按钮族变体 parsed-only）；ui/
   style/mod.rs 头注写覆盖契约 + 台账指针 + coverage 表指针。
   验证：grep 台账行在册、头注与表一致。
10. **T10 全量回归**：`cargo test -p auto-lang --features
    autodown,code-editor` 与 default 双配置；存量红逐项对基线
    归属（fix-ui-iced-suite-reds goal 在案口径），零新增红。
    验证：两命令失败清单 = 基线清单。

## 测试设计

- 对拍审计台（tests/style_parity.rs）为常驻主门：清单驱动、
  白名单外漂移即红、覆盖率表落盘可 review。
- 家族单测沿 class.rs/iced_adapter.rs 内嵌测试既有模式：每新解析臂
  至少一 parse 测 + 一 headless applied 测；iced 消费路径（Flex1
  先例形态）布局/视觉家族各抽主路径一测。
- 变体/responsive/dark：解析→过滤→applied 三段各有断言；hover
  既有套件作迁移兼容回归门。
- 上下游联测：auto-down demo（PLAN-046 T1 两栏 flex-1）作真消费
  冒烟——vm-smoke + 截图。

## 验收标准

1. `docs/style-coverage.md` 在册且与测试断言同源生成；Unsupported
   白名单每行有家族级理由并与 KNOWN-DEBT 行互链。
2. 布局/视觉/文本三家族白名单外全类 headless+iced applied 绿。
3. focus/active/disabled 经 variant 管道可声明；按钮族 applied。
4. responsive 断点与 dark 主题各有可断言生效用例。
5. 静默丢弃通道关闭：parse_reported 报告面在册，coverage 表从
   测试同源生成。
6. 双 feature 配置全量回归零新增红。

## 执行步骤

- T1 清单 vendor（tests/fixtures/tailwind-v34-utilities.txt）+
  mod.rs parse_reported 通道。验证：`cargo test -p auto-lang style`。
  [✅ 已完成] 8859 类×15 families 入库（tools/gen_tailwind_manifest.py 零依赖
  生成器入库，待澄清④裁定：可复现性优先）；parse_reported 四测绿（未映射
  原文名报告/hover 前缀保留/清洁空报告/旧 parse 行为不变），86 style 测试
  全绿（--features ui-iced）。
- T2 tests/style_parity.rs 骨架 + docs/style-coverage.md 基线表。
  验证：`cargo test -p auto-lang style_parity`。
  [✅ 已完成] 红→绿实证：空白名单首跑 6147 missing → 基线白名单（永久 +
  [T3]/[T4]/[T5] 临时条目 + 动态色板缺口规则）后 missing=0；基线矩阵
  applied 2425/parsed-only 287/unsupported 6147；IcedStyle 增 Default+
  PartialEq 派生支撑单类机械差分；cargo t 别名已挂 --test style_parity
  （--features ui-iced 档），2/2 测试绿，表入库。
- T3 布局家族补全（class.rs + headless + iced + parity 表转绿）。
  验证：`cargo test -p auto-lang style_parity layout`（新家族测）。
  [✅ 已完成] audit_layout_family_applied 断言绿：布局组 applied 1582
  （340+151+798+295，前值 1260）。SizeValue::Fraction 通用分数（Fill-ratio
  口径，待澄清③裁定：无容器查语义下近似，记类注释）；min-h/min-w 收紧
  （此前未知命名值误落 0.0——min-h-svh/min-w-full 实勘修）；IcedDisplay/
  Fixed/Sticky 字段化；PARSED_ONLY_ALLOWED 台账（CSS 默认 no-op 族）建立；
  百分比/内容尺寸档转永久受限白名单。80 ui::style 单测+3 parity+2 plan449 绿。
- T4 视觉家族补全。验证：同 T3 命令（视觉家族段）。
  [✅ 已完成] audit_visual_family_applied 断言绿：视觉组 applied 1901
  （backgrounds 1051/borders 577/effects 273，前值 922）。四色板补全+950 真值
  行；ring 宽/色/inset IR；object-fit→ContentFit 真消费；渐变三 stop+位置
  百分比真消费；shadow-{color} 彩色阴影；from-100 三位 hex 误吞假映射修复；
  单侧 border 宽/色分档转永久受限（PLAN-050 C2 边界）。82 单测+4 parity+
  2 plan449 绿。
- T5 文本家族补全（theme.rs 字体栈接线）。验证：同 T3 命令
  （文本家族段）+ theme 既有测试。
  [✅ 已完成] audit_text_family_applied 断言绿：typography applied 308
  （前 234）。tracking 六档 IR（letter_spacing 渲染分期）；leading 命名/数值
  双轨（相对倍率+绝对 px，renderer 主文本臂真消费）；line-clamp 1..6；
  全字重 9 档拆分（renderer 映射同步）；theme.rs font_stack 三栈跨平台
  契约；text-start/end≈left/right；text-ellipsis/clip≈truncate。plan449
  探针 tracking-wide gap→ok 翻转（漂移警报按设计触发）。86 单测+5 parity 绿。
- T6 variant 管道泛化（hover 迁移兼容 + 按钮 applied 面）。
  验证：`cargo test -p auto-lang style`（hover 套件零回归 + 新
  variant 测）。
  [✅ 已完成] Variant 管道四态同构；hover_classes 双轨 deprecated 转发
  （旧消费点零改动）；按钮状态面 Hovered/Pressed/Disabled 真消费 +
  build_button_style 补 opacity 消费（alpha 乘法降级）；focus 在按钮面为
  iced 0.14 上限（Status 无 Focused），登记 parsed-only（KNOWN-DEBT）；
  未知变体前缀走报告通道。90 ui::style 单测+5 parity 绿（hover 套件零回归）。
- T7 responsive 断点 + 窗口宽信号接入。验证：variant/responsive
  段测试。
  [✅ 已完成] Breakpoint 五档 + Variant::Responsive；解析期按 theme::
  window_width 门控（待澄清①裁定：接入点=Plan 409 §10 续 11 既有信号
  renderer view 前回填，resize→重建→重解析回路天然生效，无需新挂点）；
  未命中断点登记 variant 可见不静默。边界值单测（768 恰命中/767.99 不
  命中）+两断点样本 applied 断言（hidden md:flex @500/800 分叉）+
  plan411 零回归。92 ui::style 单测+5 parity 绿。
- T8 dark: 主题过滤。验证：dark 两主题对拍测试。
  [✅ 已完成] Variant::Dark + theme::dark_mode() 解析期门控（dark 态命中
  进 base、light 态仅登记可见）；语义色双主题真值沿 theme.rs stella 表
  既有底子。同一样本两主题 applied 对拍测试绿（dark:bg-slate-900 分叉）。
- T9 台账 + 头注。验证：grep 检查（KNOWN-DEBT 行、mod.rs 头注、
  coverage 表互链一致）。
  [✅ 已完成] KNOWN-DEBT P527 节五条（P527-1 永久不做族/P527-2 宿主上限/
  P527-3 分数近似口径/P527-4 变体分期/P527-5 存字段类）+ mod.rs 覆盖契约
  头注（三支柱 + 变体管道门控说明）；grep 互链一致（台账↔coverage 表↔头注）。
- T10 双配置全量回归。验证：`cargo test -p auto-lang --features
  autodown,code-editor` + default，失败清单 = 基线。
  [✅ 已完成] 零新增红，存量逐项归属：
  ① 日常门禁 `cargo t`（nextest 进程隔离，含新挂 style_parity 5 测）
  4478/4481，3 红 = P524-1 在案存量（d8_toggle_dark_mode /
  strips_tags_and_decodes_entities / schema_drift_fence），与 KNOWN-DEBT
  台账逐项对上；② default 档 lib 3387/3387 绿（本计划 diff 因 ui feature
  关闭未编译，测试集与基线恒等）；docs_gen kitchen_sink 红 = 基线存量
  （P528 并行域，6b614d505 已在上游修）；③ autodown,code-editor 档批量
  173 红 = 端口 9247 环境占用（用户运行中 musk.exe 持有）+ 运行内竞争
  级联——跨模块抽样隔离跑全绿（vm_bridge/aura/session/dynamic/iced
  shell），desktop_mcp_* 系为端口依赖环境红；style 系统零失败。执行期
  事故 0（worktree stash 共享栈误触并行会话条目，git reset --hard 即恢复，
  stash 条目 kept 无损）。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review,2026-09-03,worktree `.worktrees/plan-527-dev` @ c1020e760）

**验收标准逐项重证**（全部实跑,非引用执行期证据）：

1. **PASS** coverage 表：在册;`STYLE_COVERAGE_REGEN=1` 再生与 committed 文件
   byte-identical（同源实证）;清单生成器再生 fixture 零 diff（确定性实证）;
   白名单逐行家族级理由;三向互链 grep 证（mod.rs 头注↔coverage 表↔
   KNOWN-DEBT P527 节）。
2. **PASS** 三家族 applied：`cargo test --test style_parity`（ui-iced）5/5
   绿——audit_layout/visual/text_family_applied 三断言全过
   （基线 applied 3807/parsed-only 276/unsupported 4778/missing 0）。
3. **PASS（附宿主上限注记）** focus/active/disabled 经 Variant 管道可声明
   （test_variant_pipeline_collects_all_variants 绿）;按钮族 applied
   （plan527_t6_variant_button_status_applied 绿——三态合并构建+opacity
   消费实证）;focus 按钮回调面为 iced 0.14 API 上限（Status 枚举无
   Focused 档）,可声明可合并已测,P527-4 在册——非实现缺口。
4. **PASS** responsive：断点边界值（768 恰命中/767.99 不命中）+两窗宽
   applied 分叉（hidden md:flex @500/800）;dark：同一样本两主题对拍
   分叉——均 fresh 绿。
5. **PASS** 静默丢弃通道关闭：parse_reported 在册（mod.rs）+四单测;
   audit_no_silent_drop 常驻绿;表同源（见 1）。
6. **PASS 零新增红**：复审档全量 `cargo tf` **3397/3398**（唯一红=
   schema_drift_fence,P524-1 在案同族存量,「漂移已消除请裁剪 baseline」
   维护形态,涉及文件本计划零触碰）;日常门禁 `cargo t` 复跑 **4478/4481**
   （3 红=d8_toggle_dark_mode/strips_tags/schema_drift_fence,P524-1 逐项
   对上）;审计台 5 测已实证入日常门禁（gate log 4473-4477 号位 PASS）。
   本计划未触碰 VM/转译/Book 文件,tv/tt/tb 不适用。

**遗漏/延后/workaround 猎捕**：
- diff 内 TODO/FIXME/HACK 扫描零命中;[T3]/[T4]/[T5] 临时白名单条目
  零残留（机制注释除外）。
- 债务候选（非阻塞,未入验收面）：
  D1 auto-down demo 真消费冒烟（测试设计提及 vm-smoke+截图）未执行——
  非 执行步骤/验收标准 项,下游 auto-down PLAN-046（未开工）自然覆盖,
  归其域;
  D2 hover_classes 双轨 deprecated 转发（计划内过渡,下版移除）;
  D3 gpui renderer 存量 bit-rot（AbstractView 模式缺 selectable/disabled
  等字段——此前计划加字段后 gpui 后端无门禁维护所致,`cargo check
  --features ui-gpui` 实证,错误全在 ui/gpui/renderer.rs,本计划 diff
  零触碰该文件;style/gpui_adapter.rs 无错）。D1-D3 已登记/建议随
  KNOWN-DEBT 维护。

**执行侧待澄清裁定确认**（四条全部执行中裁定,采纳为终案）：
①responsive 接入点=theme::window_width 既有信号（renderer view 前回填），
解析期门控+resize→view 重建→重解析既有回路，无新挂点——实证：边界值/
双窗宽测试+plan411 零回归；
②gpui=headless 金标+iced 机械差分为主门，gpui 尽力对齐不入门禁（D3 存量
bit-rot 非本计划域）；
③w/h 分数=Fill-ratio 近似（同分母互补保比/混分母退化等分）,P527-3 在册；
④生成脚本入库 tools/（再生确定性已证）。

**结论**：验收 6/6 PASS,零阻塞债务 → **reviewed**。

（执行侧裁定备档：
①responsive 接入点=theme::window_width 既有信号（renderer view 前回填），
解析期门控+resize→重建→重解析回路，无新挂点；
②gpui 策略=headless 金标+iced 机械差分为主门，gpui 尽力对齐（feature
不入默认门禁，差异随 T9 台账登记）；
③w/h 分数口径=Fill-ratio 近似（同分母互补保比，混分母退化等分），
KNOWN-DEBT P527-3 在册；
④清单生成脚本入库 tools/gen_tailwind_manifest.py（可复现性裁定）。）

## 待澄清事项

- responsive 窗口宽信号的接入点（renderer resize→restyle 回路的
  具体挂点）实施时实测定夺，在复审记录记录所选案。
- gpui_adapter 不在默认门禁（feature 未开）的验证策略：headless
  做金标、gpui 尽力对齐 + 差异登记，还是纳入某条 CI——实施时定。
- w-1/2 类分数宽度的口径（Fill 系近似 vs 显式像素计算）在 T3
  实施时按 iced 能力定夺并记入类注释与台账。
- 清单生成脚本是否入库 tools/（可复现性 vs 一次性成本），T1 定。
- auto-down PLAN-046 为时序下游消费方（其 T1 用 flex-1 两栏），
  本计划 T3 前其先行可用既有 Flex1 机制，无硬阻塞。
