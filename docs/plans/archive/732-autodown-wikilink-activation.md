---
plan_id: PLAN-732
status: archived            # drafting → executing → execution_done → reviewed → archived
feature_name: autodown 编辑壳正文内链激活供给（jade-edit PLAN-037 上游供料 U-01..06）
author: [agent]
created_at: 2026-10-02
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/autodown-wikilink.md（SD-01：add——内链语义消费/区间命中/激活出口/载荷/origin/兼容矩阵）"
  - "docs/specs/auto-lang/ui/architecture.md（SD-02：modify——ADR-27 内链激活供给裁定）"
touched_goals: []             # 供料/下游谱驱动面（jade-edit PLAN-037 消费解锁），无 goals.md 正式 GOAL-NNN——731/728 先例注记式

affects: [crates/auto-lang/src/ui/autodown_editor/, crates/auto-lang/src/ui/view.rs, crates/auto-lang/src/ui/aura_view_builder.rs, crates/auto-lang/src/ui/iced/]
current_step: 6
total_steps: 6
---

# [PLAN-732] autodown 编辑壳正文内链激活供给

## 0. 变更摘要

jade-edit 仓 [PLAN-037](../../../jade-edit/docs/plans/037-native-wiki-link-navigation.md)（VM 原生正文内链导航，R-03b3）的消费合同要求本仓先交付原生 wikilink 供给。其[需求包](../../../jade-edit/docs/upstream/2026-10-native-wiki-link-supply.md)（U-01..06）与[契约冻结件](../../../jade-edit/docs/plans/attachments/037-native-link-contract.md)（C-01..06，jade 904547d）已冻结验收条款；本 Plan 在本仓独立承接，是五环链从"仅样式"到"可激活"的供给本体。

**关键实勘结论（本 Plan 的地基）**：wikilink 语义载荷**已在模型里**——autodown-core（`D:/autostack/auto-down/autodown/packages/engine/rust`，auto-lang 经 `crates/auto-lang/Cargo.toml:172` path 依赖消费）的 markdown 解析链把 `[[inner]]` 产为带 `attr "wikilink" = <inner 原文>` 的 InlineSpan（markdown_parser.rs:1966 识别→:2869-2876 出 span）。缺口全在 auto-lang 编辑壳链路：`flatten_inlines`（core.rs:132-150）只读 `span.marks`、丢弃 `span.attrs`——目标与样式双双丢失。**因此本供给零 autodown-core 改动、零 auto-down 仓改动、零模型扩展**，纯 auto-lang 编辑壳/视图/降层五环补链。

交付六项：U-01 链接语义与布局命中、U-02 完整点击激活与编辑兼容、U-03 六环回调全链（双 string 载荷）、U-04 来源/世代绑定（禁全局单槽）、U-05 真实事件验证面、U-06 缺省兼容。jade 侧消费（其 T-02 解锁核验、T-03..05 接线）不在本仓。

## 1. 目标

- G-01（U-01）：加载期 `[[target]]` / `[[target#anchor]]` 的目标与锚点在编辑壳内保留至布局命中区；中文/行内前后文/折行/段落·标题·列表·引用文本域可识别；代码块·行内代码字面与普通 Markdown 外链不触发 wiki 语义；不得整块矩形当命中区。
- G-02（U-02）：普通左键完整点击（按下→抬起同链接区间）恰一次激活；拖选、右/中键、越界抬起、非链接、布局/正文变化后的陈旧命中零激活；文字编辑、IME、选区、内部撤销、滚动零回退。
- G-03（U-03）：core 输出→DocEditor.publish→View callback→iced lowering→AURA→DynamicMessage→真实 VM handler 六环全通；载荷两 `Value::Str`（target、target_anchor，无锚空串），首个 `#` 拆分+trim 与 Vue 消费逐值一致；第二参不是源块身份。
- G-04（U-04）：激活绑定实际来源 editor key/挂载代次，结构上禁全局 last-mounted 单槽；双同正文实例各发各来源；失效/卸载/换内容后排队旧激活零冒领。
- G-05（U-05）：验证证据经生产 DocEditor 指针处理与 Shell.publish 真实路径（注入真实 iced 事件、与生产共一实现）；MCP 合成面只作负例/辅助；core 命中→widget/Shell→handler→载荷四层同一次激活贯通并绑定 binary 指纹。
- G-06（U-06）：未声明回调的应用零行为变化（可编辑）；View map/cloning、scroll_sync 包装、oninput/onfocusblock 全保留。
- G-07：解锁回执五件套产出，供 jade T-02 验收（见 AC-07）。

**非目标**：输入期 `[[` 键入的实时解析承诺（B-6 族 owner=auto-down；编辑壳重建面若自然覆盖，如实记录不承诺）；简称 `onopenwikilink` 兼容；保存来源/动作栏判定；caret/reveal 定位；宿主命令桥；全局 Shift；vue/jet/ark/rust 生成器改动；jade 仓任何文件；auto-down 仓任何文件；Musk。

## 2. 架构方案

1. **语义消费在编辑壳入口**：`flatten_inlines`（core.rs:132）扩展为同时读 `span.attrs`——`attrGetStr(span.attrs,"wikilink")` 命中即链接 span：`SpanStyle.link = true`（补齐 VM 轨样式，现 wikilink span 无 `Mark::Link`、渲染为纯文本），并把 `wikilink` 原文（target#anchor 或 target）带入新区间结构。零 parser 改动。
2. **链接区间表进 DocLayout**：沿 `MarkInterval` 同型新增链接区间（byte lo/hi + target 原文），随 `DocLayout.blocks` 落布局（core.rs:406 DocLayout 现无此表）。命中测试复用现有指针→字节映射（widget.rs:234 ButtonPressed 已有 x,y→core 路径）。
3. **完整点击门在 widget 层**：按下/抬起（widget.rs:234/:249 两事件臂）都落在同一链接区间才激活；拖选（按下后移动超阈值/选区展开）、右/中键、越界抬起零激活。阈值 T-01 冻结。
4. **激活出口**：`DocOutput`（core.rs:393）新增链接激活出口（如 `link_activated: Option<LinkHit>`，载荷 target 原文）；`DocEditor::publish`（widget.rs:112）新增转发臂到 `on_link` callback。
5. **视图与降层**：`View::AutodownEditor`（view.rs:767 定义/:2490 map）新增 `on_link: Option<Message>` 字段，map/cloning 随扩；`convert_autodown_editor_native`（aura_view_builder.rs:3880）新增事件消费臂（事件名 T-01 冻结，`open-wiki-link` 全名优先）→ `event_to_message` 变体携两 `Value::Str` 实参（`DynamicMessage::Typed{args}`，Plan 499 M2 mouse-area 坐标实参先例，aura_view_builder.rs:13269 现为空 args）；`build_autodown_editor_generic`（iced/renderer.rs:26228）传递。
6. **origin 即实例**：VM 轨回调天然 per-instance（iced widget 持自己的 callback，无 Vue 轨 opener 全局单槽问题）；载荷与来源在 T-01 冻结公开形态（key 随载荷显式携带 or 结构性实例绑定+反例锁定），双同正文实例与换内容反例为强制证据。
7. **验证面分层**：生产层=iced `Event::Mouse` 注入 `DocEditor::update`（widget.rs:213）全链（与生产共一实现）；core 层负例/辅助=MCP `__mcp_click` 同族合成通道（mcp_server.rs:1855+，现语义=ghost 落点、明示不走 handler 提取通道——**不得改其语义**，wikilink 激活非其职责）。

## 3. 技术栈

Rust（`crates/auto-lang/src/ui/autodown_editor/**`、`view.rs`、`aura_view_builder.rs`、`iced/renderer.rs`）；iced 事件管线与 cosmic-text 编辑核（现有依赖，零新增）；AutoVM 桥载荷走 `DynamicMessage`/`Value::Str`（ADR-24 段执行契约下 handler 可异步，本供给零桥接协议改动）。锚点拆分（首个 `#`+trim）在载荷构造处归一，不依赖 parser。不引入 DOM 私有实例、像素坐标常量、合成整文改写。

## 4. 需求分析与背景调查

### 4.1 授权与消费合同

- 授权：用户 2026-10-02 会话指令「用 auto-plan-new 到 auto-lang 去立项，满足前置条件」——**仅授权本仓立项起草**；实施（work/review/merge）须另行启动；无预算/自动续跑限制。
- 消费方：jade-edit PLAN-037（executing 1/5，其 T-02 解锁门=本 Plan reviewed/delivered + 五项回执）。其契约冻结件 C-01..06 与回执清单为本 Plan AC 的对岸文本；**本 Plan 不得弱化 U-01..06 条款**，做不到报 needs_replan/blocker。
- 上游链事实：autodown-core 为 auto-down 仓产出，本仓 path 依赖（crates/auto-lang/Cargo.toml:172）；模型已带 `wikilink` attr，**无接口缺口、无需 auto-down 供给**（jade 需求包 §3 的预判以实勘收窄）。

### 4.2 五环现状（源码实勘 2026-10-02T13:30Z，SHA256 见 jade 契约 §2）

| 环节 | 事实 |
| --- | --- |
| autodown_editor/core.rs | :393 `DocOutput` 仅五 changed 旗标；:144 `Mark::Link\|Image => style.link=true`；:132 `flatten_inlines` 丢弃 attrs——`wikilink` attr 在此断链 |
| autodown_editor/widget.rs | :112 `publish` 仅 on_change/on_focus；:213 `update` 消费 iced `Event::Mouse` ButtonPressed/Released→`DocInput`（生产指针入口）；:234/:249 左键臂在册 |
| ui/view.rs | :767/:2490 `AutodownEditor` 七字段（key/value/is_final/on_change/on_focus/placeholder/style）无 link callback |
| ui/aura_view_builder.rs | :3880 `convert_autodown_editor_native` 仅消费 oninput/onfocusblock；:13269 `event_to_message_impl` 空 args；:13170 `event_to_message_with` 在册 |
| ui/iced/renderer.rs | :26228 `build_autodown_editor_generic` 无回调传递 |

机制先例：Plan 499 M2 mouse-area 坐标实参经 encode/decode_payload 直达 VM handler 形参（双 `Value::Str` 载荷同法）；mcp-pointer-input.md 合成坐标 `Float(+1e-3)` 编码合同（新增合成通道须同口径）；Plan 057 `__mcp_key` 走 `core.handle_input` 直调=真实按键同构（编辑壳合成事件族先例）。

### 4.3 环境与门禁绑定

- master HEAD=7d50989f7（2026-10-02 实勘）；binary 由外部会话持续重建——**交付批次绑定交付时 binary 版本+SHA256**，jade T-02 有重绑规则（其契约 §1），本 Plan 回执随交付时指纹。
- 运行时依赖契约：ADR-19（Init demand 登记簿）、ADR-24（VM 桥段执行/parked）——handler 派发语义零改动，本供给只是新增一种消息。
- 门禁（AGENTS §2 分级）：开发迭代 `cargo check -p auto-lang` + `cargo t autodown`（scoped）；复审=裸 `cargo t`（全日常面）；触 `ui_gen/**` 才 `cargo tu`（本 Plan 不动生成器，预期零触发）；不触 VM/编译器核心，`cargo tv` 按复审时 diff 触面如实评估。`cargo tf` 非本 Plan 门禁（批量回归档）。
- 规格：ui/architecture.md 现无 wikilink 条目（本 Plan 供 SD-02 ADR）；无 autodown wikilink 局部 spec（供 SD-01 新增）。

## 5. 详细设计

### 5.1 阶段与数据流

```
加载期：src → autodown-core parser → BlockNode(InlineSpan[attrs["wikilink"]])
  → flatten_inlines 扩展：链接 span → (text, mark_ivs, link_ivs[target原文])
  → 布局：DocLayout.blocks[].link_regions（byte 区间×行几何）
点击期：iced Event::Mouse → DocEditor::update → core hit-test
  → 按下/抬起同区间门 → DocOutput.link_activated
  → publish → View::AutodownEditor.on_link → DynamicMessage::Typed
    { event_name: <公开名>, args: [Str(target), Str(target_anchor)] }
  → VM 桥派发 DSL handler（如 jade .OpenWikiLink(str,str)）
```

- 锚点归一：`target#anchor` 在载荷构造处按首个 `#` 拆分、两侧 trim（与 jade spec wiki-link-navigation §1 一致）；无锚 target_anchor=""（空串，非 None 字面泄漏）。
- `target` 为引擎侧已 trim 页面名原文（中文/别名/带 .ad 等不做身份判断——四级解析归 jade 后端）。
- 代码域安全：parser 层已保证代码块/行内代码内 `[[` 不产 wikilink span（负例测试锁定）；`href` span（Markdown 外链）不产 wiki 激活。

### 5.2 T-01 有界原型冻结（四未知，每项一次原型＋一次针对复测）——**已冻结（2026-10-03 work 阶段落定，SD-01 §2..6 为权威文本）**

| # | 未知 | 裁决（原型+复测在案） | 记录处 |
| --- | --- | --- | --- |
| K-01 | 命中映射 | `DocLayout.links` 单 layout-run 段矩形（push_link_regions：行前缀偏移+run 钳制+index_x，选区同路）；折行每 run 一段独立命中；无外加命中带；拖动容差 LINK_DRAG_SLOP=4.0px；快照失配/折叠/只读实例零区间。复测=`plan732_wikilink_regions_semantics`/`_wrapped_regions_split`/`_negative_code_and_href` | SD-01 §2 |
| K-02 | origin 公开形态 | **结构性实例绑定**（per-View-node LinkCallback/per-widget-instance 发布，无全局单槽）；载荷不携 key。复测=`plan732_dual_instance_isolated_activation`+六环语料单实例派发 | SD-01 §5 |
| K-03 | 事件公开名与签名 | 绑定双形态：`on "open-wiki-link"`（完整 kebab，Plan-367 引号式键 `on"open-wiki-link"`——jade Vue 同名，主形态）+`onopenwikilink` 简写；handler 名作 Typed 事件名；双 Str 经 encode/decode_payload。复测=`test_autodown_editor_on_link_message_channel`/`dual_str_payload_crosses_send_boundary` | SD-01 §4 |
| K-04 | 真实事件 seam | `UserInterface::build/update` 生产事件泵（widget.layout→measure→render_frame + update 真实 Event::Mouse/Cursor + Shell 消息），与生产共一实现；MCP `__mcp_click` 语义不变只作负例锚。复测=`plan732_real_iced_events_full_click_publishes_on_link`/`plan732_six_ring_full_click_reaches_vm_handler`/`plan732_mcp_click_ghost_semantics_zero_wiki_activation` | SD-01 §6 |

原型只在本 Plan 专属 worktree（`D:/autostack/.wt/lang-732/auto-lang`）进行；任一项不可达=精确 blocker/needs_replan，不删 U 条款、不降 AC。

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add | docs/specs/auto-lang/ui/design/autodown-wikilink.md | 新增：wikilink span attr 语义消费、链接区间模型、完整点击门、激活出口与双 Str 载荷契约、origin 形态、兼容矩阵、验证面口径 | 内链供给无局部 spec；实现完成后 canonical 化 | AC-01..06 |
| SD-02 | modify | docs/specs/auto-lang/ui/architecture.md | 新增 ADR-27：内链激活供给裁定——零模型扩展（复用 autodown-core wikilink attr）/区间进 DocLayout/激活经 publish 出口/载荷沿 499 M2 实参先例 | 供给机制升格引擎级裁定 | AC-01..07 |

`ui/overview.md` / `plans.md` 回写与 `.autoos/specs.json` upsert 归 merge（规约 §4）。

## 6. 测试设计

| 组 | 场景与证据 | AC |
| --- | --- | --- |
| C 语义/命中 | 加载期英文/中文/别名/带锚链接→链接区间 target 逐值；行内前后文、折行分段、首块；代码块/行内代码/外链 href 零区间 | AC-01 |
| W 点击门 | 按下→抬起同区间恰一次激活；拖选跨区/右中键/越界抬起/非链接/布局变化后陈旧命中零激活；阈值边界 | AC-02 |
| L 全链载荷 | core 命中→publish→View callback→DynamicMessage args=[Str,Str] 四层 known-answer（target/anchor 逐值、无锚空串、第二参非块身份） | AC-03 |
| O 来源 | 双同正文实例各发各来源；卸载/换内容后排队旧激活零冒领（结构性+反例双证） | AC-04 |
| R 真实事件 | iced `Event::Mouse` 注入 `DocEditor::update` 生产路径证据（seam 共一实现，记窗口/布局/binary 指纹）；MCP 合成面负例（`__mcp_click` 零 wiki 激活、ghost 语义不变） | AC-05 |
| X 兼容 | 无 on_link 应用编辑零变化（oninput/onfocusblock/scroll_sync/内部撤销全保）；View map/cloning 扩展后既有视图测试绿；缺省 None 单测 | AC-06 |

命令（worktree）：`cargo check -p auto-lang`；`cargo t autodown`（scoped，随实现面扩 `cargo t iced`）；复审裸 `cargo t` + 触面档评估（§4.3）。jade 侧 probe/e2e 联验归其仓 T-02，本 Plan 只产证据包（回执文件+指纹）。

## 7. 验收标准

| ID | 可观察行为与具体验证／预期 |
| --- | --- |
| AC-01 | C 组全绿：加载期 `[[target]]`/`[[target#anchor]]` 语义保留至 DocLayout 链接区间（中/英/别名/锚点/折行/首块已知答案逐案）；代码块·行内代码字面与 Markdown 外链零区间；VM 编辑器内链接 span 获链接样式（style.link）；命中区非整块矩形 |
| AC-02 | W 组全绿：左键完整点击（同区间按下抬起）恰一次激活；拖选、右/中键、越界抬起、非链接、陈旧命中零激活；编辑/IME/选区/内部撤销/滚动既有族回归绿 |
| AC-03 | L 组全绿：六环全链到达 VM handler，载荷两 `Value::Str`（target、target_anchor），首 `#` 拆分+trim 与 jade spec 语义逐值一致，无锚为空串；事件名与 Vue `on "open-wiki-link"` 兼容（K-03 冻结名）；第二参非源块身份 |
| AC-04 | O 组全绿：双同正文实例各发各来源（载荷可辨或分发可辨）；卸载/换内容后排队旧激活零冒领；无全局 last-mounted 单槽（结构性证据） |
| AC-05 | R 组全绿：真实 iced 事件注入生产路径的可复跑证据（脚本+窗口/布局/binary SHA256 指纹绑定）；`__mcp_click` 零 wiki 激活负例且 ghost 落点语义不变；四层同一次激活贯通 |
| AC-06 | X 组全绿：未声明回调的应用编辑行为逐项零变化（输入/焦点/滚动/撤销/scroll_sync）；View map/cloning 扩展后既有测试零回归；缺省 None 单测在案 |
| AC-07 | 解锁回执五件套产出并随交付提交：①本 Plan revision+delivered commit；②交付时 binary 版本+SHA256；③公开 callback 契约（事件名/双参语义/origin 形态/签名+Vue 兼容证据）；④C/W/L/O/R 已知答案+反例+真实事件证据清单；⑤原生 merged 生产点击可行性说明（jade T-02 联验在其仓执行，本条不冒称已联验）；SD-01/SD-02 落位 |

## 8. 执行步骤

| 步骤 | 依赖 | 经核实路径/符号 | 交付与验证 | 覆盖 |
| --- | --- | --- | --- | --- |
| T-01 有界原型与公开契约冻结 | 无 | §5.2 四未知；worktree `D:/autostack/.wt/lang-732/auto-lang`（`git worktree add … -b plan-732-dev` 或 `new-wt-group.sh lang-732`） | 每未知一次原型＋一次复测；产出决策工件=SD-01 草案+§5 更新+jade 契约回填建议（其 §6 K-01..04 对应）；不可达记精确 blocker | AC-01..05（契约面） |
| T-02 core 链接语义与布局命中 | T-01 | autodown_editor/core.rs `flatten_inlines`(:132)/`DocLayout`(:406)/新增链接区间表 | C 组测试绿（语义/样式/负例/折行）；`cargo t autodown` 全绿 | AC-01 |
| T-03 激活出口与完整点击门 | T-02 | core.rs `DocOutput`(:393) 增 `link_activated`；widget.rs `update`(:213/:234/:249) 按下抬起同区间门+阈值 | W 组测试绿（激活恰一次/五类零激活）；scoped 绿 | AC-02 |
| T-04 视图扩展与 AURA/VM 全链 | T-03 | view.rs `View::AutodownEditor`(:767/:2490) 增 on_link+map/cloning；aura_view_builder.rs :3880 事件臂+:13269 args 变体；iced/renderer.rs :26228 传递 | L 组四层 known-answer 绿；O 组双实例反例绿；`cargo t` scoped 绿 | AC-03, AC-04 |
| T-05 真实事件验证面 | T-04 | widget.rs `update` iced 注入 seam（K-04 冻结形态）；mcp_server.rs 负例锚(:1855+) | R 组证据包（脚本+指纹）；`__mcp_click` 负例绿 | AC-05 |
| T-06 兼容回归、规范落位与证据包 | T-05 | §6 X 组；docs/specs/auto-lang/ui/design/autodown-wikilink.md（SD-01）+architecture.md ADR-27（SD-02）；AC-07 回执五件套 | X 组+既有编辑器/视图族回归绿；裸 `cargo t` 全绿+触面档如实评估；SD 落位；execution_done → review | AC-06, AC-07 |

进度 6/6；每步完成后在步骤行追加 [✅ 已完成] 证据行（前沿 v2 范式）。

- [x] T-01 [✅ 已完成] 四未知原型+复测全过（§5.2 冻结表）；SD-01 草案=worktree `docs/specs/auto-lang/ui/design/autodown-wikilink.md`；jade 契约回填建议=事件名/双参/origin 与其 C-01..C-04 冻结值逐条对齐无需修订（本仓证据见回执 ③）。
- [x] T-02 [✅ 已完成] flatten_inlines 三元组（text/marks/links）+BlockBuf.links 全构造点+DocLayout.links+push_link_regions；C 组 3 测绿（`plan732_wikilink_regions_semantics`/`_negative_code_and_href`/`_wrapped_regions_split`）。
- [x] T-03 [✅ 已完成] DocOutput.link_activated（退 Copy）+link_pending 完整点击门（三重身份+single+plain+4px slop+rebuild/FocusLost/非左键清除）+MouseReleased 补坐标；W 组 3 测绿（`plan732_full_click_activates_with_payload`/`plan732_click_gate_negatives` 七负例/`plan732_stale_hit_after_content_change`）。
- [x] T-04 [✅ 已完成] View::AutodownEditor.on_link（LinkCallback newtype+map/cloning+scroll_sync 双臂）+aura `on "open-wiki-link"`/`onopenwikilink` 双形态绑定+renderer 三降层臂；L/O 组+`test_autodown_editor_on_link_message_channel`+`dual_str_payload_crosses_send_boundary`+六环语料 `plan732_six_ring_full_click_reaches_vm_handler`（VM handler state 落地）全绿。
- [x] T-05 [✅ 已完成] K-04 seam=UserInterface 生产事件泵（`plan732_real_iced_events_full_click_publishes_on_link`：真实 Event::Mouse→Shell 消息逐值+越界/拖选负例）；MCP `__mcp_click` 语义零改动+负例锚绿（`plan732_mcp_click_ghost_semantics_zero_wiki_activation`）。
- [x] T-06 [✅ 已完成] X 组（`plan732_local_edit_deactivates_until_rebuild`/`test_autodown_editor_on_link_callback_and_map`）+裸 `cargo t` 日常档 5033 测红集与 master@26a5af68e 逐名全等（14 预存红+plan502_m3/plan707 两在案 flake 复跑绿，0 新红）；`--features autodown` 档全量红集与 master 同基线全等（9 预存红+2 flake，0 新红）；SD-01/SD-02 落位（ADR-27）；AC-07 五件套回执 `docs/reports/p732-wikilink-supply-receipt.md`（binary 指纹 v0.4.2-2638-g713119788-dirty/2287A536…AD2+诚实边界注记）。触 `ui_gen/**` 生成器=零（tu 不触发）；VM/编译器核心零触及（tv 不触发——renderer/aura 为 UI 面非 VM 桥派发语义改动，派发链零改动）。

## 9. 复审记录

- stage: new
- plan_id: PLAN-732 / plan_revision: 1
- outcome: pass（drafting handoff——立项完成，work/review/merge 未授权未开始）
- 任务/验收覆盖：6 任务覆盖 7 验收/2 SD；T-02..06 逐级依赖 T-01 冻结产物，路径/符号全部源码实勘（§4.2 行号级）。
- 关键裁定：零 autodown-core/auto-down 改动（wikilink attr 已在模型，markdown_parser.rs:2869-2876 实勘）；载荷沿 Plan 499 M2 实参先例；MCP `__mcp_click` ghost 语义保持不变、只作负例锚；不做输入期解析承诺（B-6 族如实维持）。
- next: work——用户授权后在专属 worktree `D:/autostack/.wt/lang-732/auto-lang`（分支 `plan-732-dev`）从 T-01 开始；jade-edit PLAN-037 的 T-02 解锁门在本 Plan reviewed/delivered + AC-07 回执后核验。

### work 阶段收口记录（2026-10-03）

- stage: work | PLAN-732 / plan_revision: 1 | outcome: **pass**
- code_commit: `10abc4500`（worktree `D:/autostack/.wt/lang-732/auto-lang` @ `plan-732-dev`，基面 master@1cabfeb89；auto-down 兄弟 detached@895f8d0 零改动零消费）
- task_ids: T-01..T-06 全勾（6/6，证据行见 §8）
- evidence:
  - 实现：core.rs（flatten_inlines 三元组/BlockBuf.links/DocLayout.links+push_link_regions/link_pending 完整点击门/DocOutput.link_activated+split_wikilink_target）、widget.rs（on_link+publish 转发+release 坐标）、view.rs（on_link 字段+LinkCallback+map/cloning）、aura_view_builder.rs（双形态绑定臂）、renderer.rs（generic/convert/VM 三臂）、autodown_render.rs/mod.rs（构造点/导出）。
  - 测试：13 个新测试 + 语料 `test/ui/plan732_wikilink/`（C3/W3/L3/O1/R3/X2 + 六环语料）——`cargo nextest run -p auto-lang --lib --features autodown plan732 test_autodown_editor_on_link` 全绿。
  - 门禁：`cargo check -p auto-lang` ✅；`cargo t autodown` 78/78 ✅；裸 `cargo t` 日常档红集与 master@26a5af68e 逐名全等（14 预存红+2 在案 flake 复跑绿，0 新红）；`--features autodown` 档全量同基线全等（9 预存红+2 flake，0 新红）；tu/tv/tt 零触发（不动生成器/VM 编译器/transpiler）；fmt 差异仅预存 examples/、零残留调试输出。
  - 规范：SD-01 新增 + SD-02（ADR-27）追加；AC-07 五件套回执 `docs/reports/p732-wikilink-supply-receipt.md`（含交付时 binary 指纹与诚实边界注记）。
- blockers: 无（四未知全冻结、七验收全有证据、无未闭合澄清）。
- 实现期微调（均在授权范围内，未弱化任何 AC）：①完整点击门落 core handle_input（计划 §2.3 文案为「widget 层」——实际落点是 widget 两鼠标臂的 DocInput 汇聚点，语义同为「按下/抬起两事件臂都落同一链接区间才激活」且与生产共一实现，SD-01 §3 在案）；②`DocInput::MouseReleased` 补 x,y 字段（iced ButtonReleased 本体不带位置，widget 层取 cursor.position——计划未预见的小扩展，MCP 拖拽合成臂同步适配）；③首帧行为发现：`autodown_editor_sync` 槽位缺席时 no-op（预存事实，非本计划引入），六环语料测试显式两段降层模拟生产次帧节律（ADR-27 注记）。
- next: review（/auto-plan:review）——复审门=裸 `cargo t` 已跑（红集对拍在案）+触面档评估已记录（零触发）；review 后 merge 收口回执终稿指纹。

### review 阶段记录（2026-10-03，实施同会话复审——按技能要求从工件重建裁决，独立性受限已在节首声明）

- stage: review | plan_id: PLAN-732 | plan_revision: 1 | outcome: **pass**
- reviewed_commit: `10abc4500e024291f424337f47cd7cc819717b90`（worktree clean，无未提交实现改动）| base_commit: `1cabfeb89fe1d276bcca0c8e36a87a7d304ed191` | dependency_revisions: auto-down 兄弟 detached@895f8d0（零改动零消费）| spec_inputs: ui/architecture.md（ADR 日志至 ADR-26）、jade 契约冻结件 037-native-link-contract.md（对岸验收文本）
- 验证重跑（本基线）：`cargo nextest run -p auto-lang --lib --features autodown plan732 test_autodown_editor_on_link` = **15/15 绿**；裸日常档（--lib+schema_drift+docs_gen+component_registry_test, ui-iced, no-fail-fast）红集与 master@26a5af68e 基线 **diff 逐名全等（exit=0，14 预存红，本次零 flake 触发）**；diff 扫描零 TODO/FIXME/workaround/unimplemented（「autodown」含「todo」子串的 73 处误报已甄别）；fmt 差异仅预存 examples/。
- acceptance_results: AC-01 ✅（C 组 3 测：语义/样式 LINK_COLOR/非整块矩形/折行分段/负例）/ AC-02 ✅（W 组：恰一次+七负例+越界/拖选 widget 层+陈旧命中；既有族回归=日常档逐名全等）/ AC-03 ✅（L 组 known-answer 三层+六环语料到 VM handler state 逐值+Vue 同名绑定经真实 parser 装配验证）/ AC-04 ✅（双实例隔离+结构性 grep 零全局槽+pending 为 per-core 字段 core.rs:713）/ AC-05 ✅（UserInterface 生产事件泵+__mcp_click 零改 diff+负例锚+回执指纹）/ AC-06 ✅（map/cloning/缺省 None 单测+scroll_sync 双臂+零新红）/ AC-07 ✅（五件套回执在案，不冒称已联验）。
- findings: 无阻塞项。两条非阻塞观察：F-NB1 `autodown_editor_sync` 首帧 no-op 预存事实已登记（ADR-27/§10，潜在改进归后续）；F-NB2 work 记录「13 测」为 filter 组合计数口径，复审重跑 15/15（含六环语料+payload mod）为本基线权威数。
- spec delta 冻结：SD-01 `docs/specs/auto-lang/ui/design/autodown-wikilink.md` SHA256 `8bf534bccb162b23a3ece0caf5696d0e54714b288dd01ddb6bed4b8fbcf282ae`（131 行，add）；SD-02 `ui/architecture.md` ADR-27（+7 行，modify，@10abc4500）。文本=当前行为与持久裁定（非执行日记），与 ADR-25/26 格式同构。frontmatter：supersedes=[]（新供给面无替代）、new_spec_components=SD-01+SD-02、touched_goals=[] 带书面解释（731/728 先例）。
- evidence: worktree 仓内工件（测试源码/语料/规范/回执均随 10abc4500 落 git，merge 后于 master 持久）；命令摘录见上。binary 指纹 v0.4.2-2638-g713119788-dirty/2287A536…AD2（回执含不含本供给代码的诚实边界）。
- next: merge——合入 master、specs.json/索引回写、归档、worktree 组清理；merge 后外部重建 binary 由 jade T-02 按其契约 §1 重绑复验。

## 10. 待澄清事项

- ~~D-01 K-01..04 原型未决~~ **已闭合（2026-10-03）**：四未知全冻结（§5.2 冻结表；SD-01 §2..6 权威文本）；与 jade 契约 §6 同表对应，冻结值与其 C-01..C-04 验收条款逐条对齐，无需回填修订。
- ~~D-02 事件公开名~~ **已闭合**：`open-wiki-link` 完整 kebab 名（引号式绑定 `on "open-wiki-link"` 主形态 + `onopenwikilink` 简写）；未改签名未用别名——无需回填 jade 契约 C-01。
- D-03 VM 轨样式 parity：**已供给**——wikilink span 补 `style.link`（LINK_COLOR 既有链），观感沿主题族既有 link 样式未另立视觉验收（AC-01 内测锁定）。
- D-04 binary 外部重建漂移：回执已绑定交付时指纹（v0.4.2-2638-g713119788-dirty / 2287A536…AD2）+ 诚实边界注记（该 binary 不含本供给代码；jade T-02 按其契约 §1 重绑规则复验）。**开放项（归属 merge/后续）**：合并后外部重建的新 binary 指纹由 jade T-02 自行重绑，本 Plan 不锁全局版本。
- D-05 输入期行为：**如实记录**——编辑壳重建模型下键入 `[[x]]` 后解析时机=外部真变化 rebuild（自回显走 PLAN-057 回声守卫不重建，暂态失活为既有语义）；不进承诺、不冒称 B-6 已解（owner=auto-down）。六环语料测试两段降层注记在 ADR-27。
- **新增登记（work 期发现，非本 Plan 修域）**：`autodown_editor_sync` 在 core 槽位缺席时 no-op——生产动态循环靠次帧重降层补内容（首帧空窗）。潜在改进（首帧直接建核同步）归后续计划评估，不改变本供给语义。

### merge 阶段收据（2026-10-03，PLAN-732:r1）

- **prepared**：reviewed 基线=review pass@10abc4500e（基 1cabfeb89f，依赖 auto-down 兄弟@895f8d0 零改动）；冻结 Spec delta=SD-01 SHA256 `8bf534bc…282ae`+SD-02 ADR-27；canonical 投影目标=ui/design/autodown-wikilink.md（新）、ui/architecture.md（ADR-27）、ui/plans.md（732 行）、ui/overview.md（注记）、.autoos/specs.json（P732-1/2/3）。projection-only descendant `f7e758e27`。
- **landed**：master 基线前移（733/731 projection+archive 落地）→ rebase：代码 commit `10abc4500`=（range-diff 全等）`7bf7dc179`；投影 commit specs.json/ui-plans.md 双追加冲突调和（实录：首次调和丢数组逗号致 JSON 无效，git 链未拦已 rebase——确定性重建自 master 版+P732 追加并 amend 修复，P014-1 双份为 master 预存态非本次引入）；rebased 态复验=SD-01 哈希恒等+ADR-27 在册+plan732 族 15/15 绿。wt-guard clean → 主检出 `git merge --ff-only` → **master tip=`12fa9fd100c02640bfe99813b55977bcadaea9be`（=delivery commit，无 merge commit）**；轨后主检出冒烟 plan732 族 15/15 绿。
- **ledger_refreshed**：`.autoos/specs.json`（git-tracked，worktree 投影 commit 内落盘）——P732-1（designs→docs/specs/auto-lang/ui/design/autodown-wikilink.md）/P732-2（architecture→ui/architecture.md ADR-27）/P732-3（reviews→本归档件）；主检出回读三条各恰一次、JSON 有效（754 项）。索引再生 `python scripts/spec-index.py`（INDEX.md 内容零变化=仅项目级索引）。
- **archived**：`docs/plans/archive/732-autodown-wikilink-activation.md`（git mv）+ frontmatter `status: archived`；completion_kind: **delivered**。
- **部署观察**：生产面 artifact——主检出 debug `auto.exe` 为外部会话持续重建产物（merge 前 v0.4.2-2638-g713119788-dirty 不含本供给）；合并后需外部重建方含 PLAN-732（jade T-02 按其契约 §1 重绑规则以当时 binary 复验——回执 ② 诚实边界在案）。release 二进制/web bundle（auto build）无独立消费面（本供给为 VM 轨 UI 库改动，jade 联验未开始），登记待外部会话重建。
- **批量回归到期判定（fix-test-tiering）**：732 非 5 整除；`docs/plans/.last-batch-regression.json` 回执 2026-10-03 00:22（PLAN-733 merge 写入，<48h）——**未到期**，732 合并不触发 `/auto-plan:regress`。

## spec-sync 回写记录（v1 惯例）

- 2026-10-03 merge：SD-01 canonical 落 `docs/specs/auto-lang/ui/design/autodown-wikilink.md`（新，SHA256 8bf534bc…282ae）；SD-02 落 `ui/architecture.md` ADR-27；`ui/plans.md` +732 行；`ui/overview.md` +PLAN-732 注记；`.autoos/specs.json` upsert P732-1/2/3；spec-index 再生。
