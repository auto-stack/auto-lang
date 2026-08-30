---
plan_id: PLAN-493
status: reviewed
feature_name: mention-native textarea——mentions 能力声明与双端实现
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

supersedes_spec_components: []
new_spec_components:
  - ui/aura_view_builder+ui/view+ui/iced/renderer+ui_gen/vue:textarea-mentions-capability（Plan
    493——textarea `mentions:`/`mention_class:` 能力声明：**受限 state-rooted
    名单解析**〔bindings→read_state_as_vec 同步深拷贝；computed/调用链名单
    →None 降级，绝不落入 eval_computed/call_vm_fn——html 降级 UAF 同构雷区
    红线，护栏测锁定〕+ builder 期 `mention_segments` 纯函数段计算〔@\w+
    小写匹配、无 display 替换、覆盖拼接==value 不变式〕+ VM `SpanKind::
    Mention`〔parse "mention" 臂 + blue-600 from_rgb8(37,99,235) 前景；
    iced Format 无 per-span 背景——跨端差异登记〕+ Vue codegen
    backdrop/透明 textarea **兄弟对**发射〔类串自 textarea 类串推导：
    transparent 位换 foreground/删输入面 token/无颜色 token 保底追
    text-foreground/增背板结构 token，幂等〕+ `__autoMentionHtml` helper
    随 script 一次发射〔escape+扫描+span+尾 
，TS/JS 双签名〕+
    mentions/mention_class/height 纳 textarea attr 透传过滤〔height 为
    VM-only 几何契约，防 :height 泄漏 vue 产物〕；活体证据=器具根/子部件
    两形态 blue-600 精确像素 + musk [493-MENTIONS] 段日志）
touched_goals: [GOAL-007]   # AutoUI 跨端视觉一致：mention 高亮双端 parity（声明层抽象提升）
affects: [auto-lang/ui]

current_step: 7
total_steps: 7
---

# [PLAN-493] mention-native textarea——mentions 能力声明与双端实现

## 变更摘要

musk composer 的 mention 高亮此前在 .at 声明层直接使用了 Vue 的实现细节
（backdrop div v-html 高亮层 + text-transparent textarea 叠加技法）——声明层
泄漏实现细节，导致 VM 轨三连断点：①absolute 空背板经 Stack 分层挡死
textarea 点击聚焦（PLAN-051 P2 追加四，C1/C2 单变量实验实证，f811df98b）；
②text-transparent 使输入不可视（P2 追加三回退为可见前景，21bba9b34）；③html
纯文本降级每帧 resolve 经 call 链返回堆引用未 retain，debug RC canary UAF
实锤（3d7f916c6 被同提交回滚，vm/rc.rs:503）。

本计划将 mention 高亮提升为 **textarea 的能力声明**：.at 只声明
`mentions: .mentionNames`（要什么），双端各自实现（怎么摆）——VM 走 Plan 057
已落地的 `View::Textarea.highlight` 原生着色管线（SpanHighlighter 逐段前景
色，单 widget、无叠层）；Vue 由 codegen 在发射点生成 backdrop+透明 textarea
兄弟对（技法整体下沉到实现层）。musk 侧 .at 随之简化为单 textarea 声明。

**复审修正（2026-08-30，逐文件对照代码核查后的三处实质变更）**：

1. **UAF 红线改述（原稿自相矛盾处）**——原稿称"段计算不经 VM call 链，无堆
   引用生命周期问题"，但 `mentions: .mentionNames` 的**名单解析**会走
   `eval_computed → resolve_expr_to_value(Call 臂) → bridge.call_vm_fn`
   （aura_view_builder.rs:6341 起），且 musk 现名单是**嵌套 computed**
   （mentionNames → build_mention_names(.professionsList)，professionsList
   又是 computed）——与被 21bba9b34 回滚的 html 降级完全同构。本计划改为
   **受限解析器**：名单只从 bindings/read_state 同步深拷贝，绝不落入
   eval_computed/call_vm_fn；配套名单契约 = state-rooted（var/store 字段/
   字面量表），computed 名单 = Vue 轨可用（原生响应式）、VM 轨无高亮不崩
   （降级契约，单测锁定）。
2. **既有基建比原稿估计的多（T4/T5 大幅缩编）**——VM 端动态 textarea 臂的
   rich 路径（`highlight_with::<SpanHighlighter>` + `get_textarea_content_rich`
   的 value 逐帧同步与光标保持，renderer.rs:14564-14614/984）与 Vue 端叠加
   层发射点（`textarea_rich_overlay`，vue.rs:8995，经 5488 站点整体替换发射）
   均为 Plan 057 存量。本计划在 VM 侧只需增量 `SpanKind::Mention`（枚举 +
   parse 臂 + 色值）；Vue 侧新增 mentions 分支（复用同一发射站点与 attr_str
   通道）。原稿 T4"动态臂补 highlight_with 消费"是已完成的工作，不再立项。
3. **名单契约 = List<str>、着色 @词原文（无 display 替换）**——musk 现名单
   是 obj map（k_ 前缀防 JS 原型链命中）且 backdrop 渲染时会用显示名**替换**
   命中文本（render_mentions，与编辑器内容= value 的约束冲突，顺带产生
   backdrop/值文本错位与"Agent"重复段边角）。语言层契约为纯名单：命中即把
   `@词` **原文**着色；k_ 前缀技巧与 display 替换一并退役（视觉差与边角收益
   登记于待澄清①）。另增可选 `mention_class:` prop（span 类串）——否则
   codegen 只能硬编码 musk 品牌蓝 hsl(220 90% 56%)，等于把应用样式反向
   泄漏进语言层。

## 目标

1. `.at` textarea 支持 `mentions:`（名单引用或字面量表，List<str>）与可选
   `mention_class:`（span 类串，静态）能力声明，声明层不再出现 backdrop/
   透明叠加结构。
2. VM 轨：mentions textarea 内命中 @词**原生着色**（SpanKind::Mention 前景
   色）；点击聚焦/键盘输入/中文 IME/Enter 发送不回归；mentions 路径**全程
   不经 VM call 链**（构造性 canary 安全，debug 构建保留作回归哨兵）。
3. Vue 轨：mentions textarea 的 emit 产物为 backdrop+透明 textarea **兄弟
   对**（DOM 结构/类串/几何与 musk 迁移前等价），存量无 mentions 的
   textarea 发射零变化（build strict + vitest + style-parity）。
4. musk composer 迁移到新声明：删手写 backdrop 结构、text-transparent、
   backdropHtml/mentionNames 计算属性；名单改 model var 在 handler 上下文
   刷新。

## 架构方案

| 端 | 落点 | 实现 |
|---|---|---|
| 声明层 | .at textarea `mentions:` / `mention_class:` | mentions 值 = state-rooted 名单（var / store 字段 / str 字面量表）；mention_class = 静态类串（缺省 `text-primary bg-primary/10 rounded-[3px] px-[0.2rem] font-medium`） |
| 名单解析 | aura_view_builder `convert_textarea` | **受限解析器**：Ident/自引用 Dot → bindings → read_state → 同步深拷贝 owned `Vec<String>`；str 字面量表 → 逐元素；含任何 Call 形状 → None（VM 无高亮降级，不崩） |
| 段计算 | aura_view_builder | 纯函数 `mention_segments(value, names)`：`@\w+` 扫描，词小写比较 ∈ 名单 → 段 `("@词", "mention")`，其余逐段 `("…", "text")`；**无 display 替换**；覆盖拼接 == value |
| VM | View::Textarea.highlight → renderer rich 路径（存量） | 段 kind "mention" → `SpanKind::Mention`（新增）→ blue-600 前景（无背景 tint 登记）；rich 路径/光标保持/value 同步全用存量 |
| Vue | ui_gen/vue.rs 5488 发射站点 | 遇 `mentions`（state ref）→ backdrop div（`v-html="__autoMentionHtml(text, names, cls)"`，类串自 textarea 类串推导）+ 注入 `text-transparent` 的 textarea **兄弟对**（不包 wrapper）；helper 随 script 一次发射 |
| musk | src/front/mention_input.at | 删 backdrop div/backdropHtml 计算属性/text-transparent；`mentionNames` 改 model var 在 handler 上下文刷新（.Init/.Input）；textarea 加 `mentions:` + `mention_class:` |

## 技术栈

auto-lang（aura_view_builder 受限名单解析与段计算、iced renderer SpanKind::
Mention 增量、ui_gen/vue.rs mentions 发射分支）+ auto-musk（mention_input.at
迁移、mention_helpers.at 增名单 list helper、双轨门禁）。验证环境：debug
增量构建即可——mentions 全程不经 call 链，canary 风险面构造性为零，debug
跑法保留作回归哨兵（canary UAF 属上游 RC 语义债另行立项，见 KNOWN-DEBT）。

## 需求分析与背景调查

- **实证链（2026-08-30，PLAN-051 Phase 2 追加，均在 auto-lang 史）**：
  C1/C2 单变量实验（同语料同帧，语料 examples/ui/p051-min-ta 留档）——
  composer 子树精确复刻（backdrop 在场）即不可输，去 backdrop 即可输；
  absolute 子件经 `is_elevated_view` 分离推入 iced Stack 成上层叠容器，VM
  中 html 不渲染 = 空容器，挡死 text_editor 点击聚焦。空层不入栈修复
  （f811df98b）后点击恢复；透明回退（21bba9b34，a≤0.001 → OnBackground）
  后文字可见，但 mention 高亮丢失（纯文本降级 3d7f916c6 因 RC canary UAF
  被同提交回滚，"该方向需上游 RC 语义先行，留档"）。
- **UAF 机理核对（本计划安全设计依据）**：call_vm_fn（vm_bridge.rs:1050）
  的 C3 retain（:1085-1093）只覆盖 Int≥4M/VmRef 返回值且不配对释放（T11
  债）；Str 返回与嵌套调用链的中转值不受保护——html 降级即在此形态下被
  rc.rs:503 canary 实锤。**结论：builder 期名单解析不得进入 call 链**，
  state-rooted 深拷贝（与列表渲染既有的 materialize 路径同族）是安全面。
- **既有基建核查（本计划直接复用，不重复立项）**：`View::Textarea.highlight`
  （view.rs:291）+ `resolve_highlight_spans`（三态物化先例）+ builder
  `.highlight()` 管线在册；VM rich 路径（renderer.rs:14592-14614：
  `get_textarea_content_rich` value 逐帧同步+echo 光标保持 + `build_span_lines`
  + `highlight_with::<SpanHighlighter>`）；Vue 发射站点（textarea_rich_overlay
  整体替换）与 attr 透传过滤；`extract_state_ref`（裸 `.name`/`self.name` →
  模板 state ref）。
- **musk 现状核对**：mention_input.at:105-108 backdrop div + :115 textarea
  text-transparent；:39-40 计算属性 mentionNames（obj，k_ 前缀）/backdropHtml；
  chats_view.at:261 `professions: ""`（名单实际来自 configs 兜底链）。
  render_mentions/render_mentions_default 是聊天气泡渲染路径，**本计划不动**。
- **弃用路线记录**：html 纯文本降级（每帧 resolve computed 持堆引用，RC
  语义下 UAF）——051 已回滚；本计划的名单来自 state-rooted 深拷贝、段计算
  是 builder 期纯文本扫描，二者均不经 VM call 链。
- **对齐对象**：vue backdrop 的 DOM 结构/类串/几何以 musk 迁移前产物为
  基线（emit 快照比对）；VM 着色以 musk 品牌蓝为近似（跨端差异登记）。

## 详细设计

### 声明形态

```auto
textarea {
    value: .text
    mentions: .mentionNames          # List<str>，mention 命中名单（state-rooted）
    mention_class: "text-[hsl(220_90%_56%)] bg-[hsl(220_90%_56%/0.12)] rounded-[3px] px-[0.2rem] font-medium"   # 可选，缺省主题色系
    oninput: .Input($event)
    placeholder: "..."
}
```

`mentions` 出现即激活双端 mention 实现；不出现 = 普通纯文本 textarea
（存量零变化）。`mention_class` 仅 Vue 侧 span 类串（VM 固定前景色）。

### 名单解析（受限通道——UAF 红线）

`convert_textarea` 对 `mentions` prop 不走全量 `resolve_expr_to_value`（其
Ident 臂含 eval_computed 回退），改用专用受限解析：Ident/自引用 Dot →
bindings → read_state（经 read_state_as_vec 同步深拷贝）；str 字面量表 →
逐元素；其它形状（含任何 Call/computed 链）→ None：VM 轨无高亮（= 迁移前
现状，非回归），不崩。Vue 轨不受限：codegen 经 `extract_state_ref` 取名单名
（computed ref 同为组件级响应式名）——两侧差异属"VM 降级契约"。

### 段计算（builder 期，纯函数）

`mention_segments(value, names)`：`@\w+`（\w=[A-Za-z0-9_]，对齐
mention_is_word_char）扫描，词小写比较 ∈ names → 段 `("@词原文","mention")`；
裸 `@` 与其余字符逐段 `("…","text")`。无 display 替换。不变式：段文本顺序
拼接 == value。names 空 → 单段 text。

### VM 渲染

`renderer.rs` 增量三点：`SpanKind::Mention` 枚举变体；`parse_span_kind`
"mention" 臂；`span_kind_to_format` → blue-600 前景（from_rgb8(37,99,235)，
≈ hsl(220 90% 56%)，无背景 tint——iced Format 无 per-span 背景，跨端差异
登记）。rich 路径其余全用存量。

### Vue 发射

发射站点前判 `mentions` prop（`extract_state_ref` 取名，`value` 同法；任一
非裸 state ref → R013 警告 + 回退普通发射）。命中则整体替换为**兄弟对**
（不包 wrapper——absolute 子件包进无固有高度 wrapper 会塌陷，锚点
relative 容器归 .at 自理）：

1. backdrop div：`v-html="__autoMentionHtml(<value>, <names>, '<cls>')"`，
   类串自 textarea 最终类串推导——删 `text-transparent`/`caret-*`/`resize-*`/
   `outline*`/`focus:*`/`disabled:*` token，`text-transparent` 位换
   `text-foreground`，删后无 text 颜色 token（仅字号）时保底追加
   `text-foreground`；增 `pointer-events-none overflow-hidden
   whitespace-pre-wrap break-words`（幂等）；
2. textarea 本体：复用既有 `attr_str` + 类串注入 ` text-transparent`（幂等）；
3. `__autoMentionHtml(text, names, cls)` 随 script 一次发射（TS/JS 双签名）：
   HTML 转义 → `@\w+` 扫描（词小写 ∈ names，无 display 替换）→ 命中包
   `<span class="<cls>">@词</span>` → 尾加 `\n`（换行几何对齐）。

attr 透传过滤追加 `mentions`/`mention_class`（执行期追加 `height`——VM-only
几何 prop 防 `:height` 泄漏 vue 产物）。无 `mentions` 的 textarea 走既有发射。

### musk 迁移

- mention_input.at：删 backdrop div、计算属性 backdropHtml/mentionNames、
  textarea 类串 text-transparent；model 增 `var mentionNames []Value = []`；
  .Init 与 .Input 增 `.mentionNames = build_mention_name_list(.professions,
  .store.configs)`（handler 上下文安全路径；configs 异步到达静置缺口登记
  待澄清④）；textarea 增 `mentions:` + `mention_class:`（传现行 hsl 串）+
  `height: 72`（执行期补——VM 轨编辑面多行）。
- mention_helpers.at：增 `build_mention_name_list`（复用兜底链，扁平化
  id+name）；`build_mention_names`（obj）保留（气泡路径）；
  `render_input_mentions` 删除（孤儿）。
- chats_view.at 无改动。

## 测试设计

- auto-lang `plan493_*` 前缀：段计算四断言（命中/裸 @ 与大小写/空名单/
  覆盖拼接）；builder 集成（var 名单→段拼回=value；**computed 名单→
  highlight 空、不崩**；字面量表）；类串推导单测；vue 两形态快照（有
  mentions → backdrop 兄弟对 + helper 恰一次；无 → 逐字节一致）。
- 实机：VM composer 真键盘四点 + 017-chat 冒烟 + vue playwright 9/9。
- musk 门禁：build strict + vitest + style-parity + vm-link-probe +
  first-run。

## 验收标准

1. mention_input.at 无 backdrop div、无 text-transparent、无 backdropHtml
   计算属性；textarea 带 `mentions: .mentionNames`（var）+ `mention_class:`。
2. VM 实机：composer 打字可见、@mention 词着色、点击聚焦、Enter 发送闭环。
3. Vue 实机：composer 高亮在、几何与迁移前一致（span 覆盖收敛为 @词原文
   ——待澄清①裁定的有意差异除外）。
4. 双轨门禁全绿。

## 执行步骤

- [ ] **T1** 段计算红测。
      [✅ 已完成] 红态实证：4×E0425 cannot find function `mention_segments`。
- [ ] **T2** 段计算实现。验证：`cargo test -p auto-lang --features ui-iced
      --lib plan493_` 绿。
      [✅ 已完成] 4/4 绿（commit f632d0817）。注：aura_view_builder 在
      `ui-interpreter`（⊆ui-iced）feature 后，命令须带 --features ui-iced。
- [ ] **T3** mentions 受限解析接线 + 三集成测。
      [✅ 已完成] 7/7 绿；textarea(9)/highlight(10) 家族回归绿（commit
      a9a90297c）。
- [ ] **T4** VM `SpanKind::Mention` + parse 臂 + 色值；kind 流通/覆盖契约
      单测。
      [✅ 已完成] 测试绿；renderer 家族 stash 前后失败集一致（6-7 预存
      污染，0 新增；commit fea25c618）。
- [ ] **T5** vue mentions 发射分支（兄弟对+类串推导+helper 一次发射+
      attr 过滤），快照测两形态+推导单测。
      [✅ 已完成] 11/11 绿；ui_gen:: 全模块 708 绿（commit 995c98058 +
      2dc4975ac 保底着色/TS 注解续修）。
- [ ] **T6** musk 迁移 + 双轨门禁。
      [✅ 已完成] 勘误：musk 活跃前端=gen 轨（web/ 已冻结退役），门禁实跑
      =worktree auto.exe `auto build`（vue-tsc+vite 绿）+ gen `npx vitest
      run` 23 绿 + style-parity 0 新增红（12 条 border-t/b 为 051 登记基线
      先在）+ vm-link-probe PASS（VM_LINK_LANG_ROOT 指 worktree）+ first-run
      alive reds=0（worktree 工具链）。生成物核验：backdrop 兄弟对类集与
      迁移前等价（musk commit 969b922）。环境注记：vendor/@autodown/engine
      dist 为 gitignored 本地产物，新 worktree 需自主复制。
- [ ] **T7** 实机验收+收尾。
      [✅ 已完成·活体部分受阻如实登记] 自动化全绿：--lib 全量 4090 过/7
      败（stash 差分=6 renderer 污染+1 stage3 flake，均先在零新增）；017-
      chat VM 冒烟 A-E 绿+playwright 9/9（worktree release 二进制）。musk
      VM 活体：登录→composer 打字可见（vtree+白像素双证）→Enter/按钮均
      消息入列；**draft 清空被先在债阻断**（.send 链垃圾引用崩溃，双仓
      master A/B 同现——auto-lang master 二进制×musk master .at 复现
      Invalid object ID，非 493 回归，登记 musk KD-493）；@蓝色着色活体
      像素终验 pending（机制链 T3/T4 单测绿；VM 实例静默退出+登录链抖动
      阻断，复验手段 AUTO_DEBUG_MENTIONS=1 已埋，commit 6d18f12d5）。
      用户实测反馈"composer 变 input 样子"=VM text_editor 缺省 Fixed(30)
      （h-full/min-h 为 CSS-only，先在）——已补 musk .at `height: 72`
      （vue 不消费+上游 attr 过滤防 :height 泄漏，musk commit 8c2d36b）。
      截图存档 tmp/（composer 白像素证据 screen5；493-review 归档随复验补）。

## 复审记录

**复审（2026-08-30，/auto-plan:review，verify-don't-trust 全项重跑）**

diff 对照：`git diff d827f1d8c..6d18f12d5` = 3 文件 +673/-6（aura_view_builder
/renderer/vue.rs），与计划范围逐一对应，无越界文件、无计划内任务无 diff 项。

逐验收标准判定：
1. **PASS**——mention_input.at 活代码无 backdrop div/text-transparent/
   backdropHtml（2 处命中均注释），textarea 带 `mentions: .mentionNames`
   （model var）+ `mention_class:` + `height: 72`。
2. **PASS（机制四层闭合；musk 现场像素一项 pending，见⑥）**——打字可见
   （vtree+白像素双证）；聚焦/type 通路实证；Enter/按钮消息均入列（draft
   清空=先在债⑤，双仓 master A/B 实证非 493 回归）；@着色：单测 11 绿 +
   musk 活体段日志（names=18，段正确）+ **同二进制器具活体 blue-600 精确
   像素**（根部件与子部件两形态均着色，avg (36,98,234)≈#2563eb）+ 057
   存量 highlight 路径同证健康（emerald/purple）；musk composer 现场像素
   因 VM 实例静默退出/登录链抖动未捕获（~6 次尝试，含一次环境变量遗漏的
   无效扫描）。
3. **PASS**——gen 轨 vue-tsc+vite 绿；emit 结构/类集与迁移前等价（含
   text-foreground 保底）；span 覆盖收敛为 @词原文=待澄清①既定裁定。
4. **PASS**——cargo tf 3286/3286 全绿（合并 master 后）；tv 2 败
   （aavm2_m4 语料/cookbook channel）master 同败=存量基线零新增；ui_gen
   708 绿；plan493_ 11/11；017-chat VM 冒烟 A-E+playwright 9/9；musk 四
   门禁+probe+first-run 全绿（执行期记录）。

遗漏/延后/workaround 扫描：
- 遗漏：无（任务-diff 一一对应）。
- 延后：display 替换移除（①既定裁定）、VM 无 bg tint（③）、静置刷新缺口
  （④）——均计划内登记在案。
- Workaround/新发现债候选：①**iced textarea 内容键碰撞**——两 textarea 同
  无 on_change 且 placeholder 等长时 key 同为 `__textarea_0`，内容缓存互串
  （复验器具实证：TA2 显示 TA1 内容；musk 不受影响——oninput 键唯一）；
  建议 key 混入 path/序号，登记上游。②AUTO_DEBUG_MENTIONS 调试打印保留
  （env 门控，[VM-CALLFN] 先例）。③musk worktree auto-musk-dev-2 未折回
  musk main（消费方验证已过，按 musk AGENTS 应尽快折回清理）。

结论：**reviewed**（标准 2 的 musk 现场像素一项以四层机制证据+环境性受阻
登记放行；若需像素级终验再跑待澄清⑥手续，不构成代码性缺口）。

## 待澄清事项

1. **display 替换移除的视觉差（默认执行"移除"）**：新契约为着色 @词原文
   ——常态视觉差 = 着色范围从"显示名整段"收敛为"@词段"，并消除 backdrop/
   值文本错位与多词名"Agent"重复段边角。若裁定必须保留替换，vue helper
   需名单改映射表（VM 不受影响），属二期。聊天气泡路径不动、保留替换。
2. **mention_class 默认值**：缺省 `text-primary bg-primary/10 rounded-[3px]
   px-[0.2rem] font-medium`（musk primary=紫≠现蓝，须显式传 hsl 串保零
   视觉差）。默认值可按类串可用性微调，登记不阻塞。
3. **iced Format 无 per-span 背景**：VM mention 仅前景色（无 bg tint），
   跨端视觉差登记。
4. **名单静置刷新缺口**：configs 异步到达后、未再输入前名单 var 不刷新
   （装饰性损失；正规解挂上游 RC 语义债）。
5. **（执行期发现）composer `.send` 链垃圾引用崩溃——先在债**：Enter/
   按钮发送消息均入列，但 draft 清空前的 handler 链在两仓 master 上均可
   复现崩溃（child `Invalid object ID 18446744071562067969` / parent
   `unknown_obj:4000041 无 .push` 两形态抖动；A/B：auto-lang master
   release×musk master .at 同现）。051 期按钮路径曾实证 draft 清空——
   期间 auto-lang master 有新提交介入，定位归上游 VM 债（musk KD-493
   登记），本计划不修。
6. **（执行期发现）@着色活体终验 pending**：机制链（受限解析→段计算→
   SpanKind::Mention→blue-600）单测全绿；活体窗口像素终验被 VM 实例
   静默退出（KD-048-a，本 session 观测部分实例 <30s）+登录链抖动阻断。
   复验：`AUTO_DEBUG_MENTIONS=1 auto run --render=vm` 看 `[493-MENTIONS]
   names=N ... segs=[...]` 段产出+窗口 @词蓝色像素。**复审期补证（2026-08-30
   晚）**：musk 活体段日志已取（names=18/段正确）；同一 worktree 二进制上
   器具（examples/ui/p493-color-check，根+子部件两形态）@assistant 字形
   avg=(36,98,234)=blue-600 精确命中，057 存量 highlight 同帧健康——机制
   定案；musk 现场像素终验仍待一次环境配合（登录链抖动/静默退出）。
