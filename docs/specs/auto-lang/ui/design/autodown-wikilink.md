# autodown 编辑壳正文内链（wikilink）激活供给

> PLAN-732 交付（plan-732-dev）。本文是 autodown 编辑壳正文 `[[target]]` /
> `[[target#anchor]]` 从「仅样式」到「可激活」的供给契约——语义消费、命中
> 区间模型、完整点击门、激活出口与双 Str 载荷、origin 形态、兼容矩阵与
> 验证面口径。消费方：jade-edit PLAN-037（其契约冻结件 C-01..06 为对岸
> 验收文本，本文与其逐条对齐、不弱化）。

## 1. 语义消费（U-01 / C-01）

- 上游单源：autodown-core markdown 解析链把 `[[inner]]` 产为带
  `attr "wikilink" = inner 原文` 的 InlineSpan（`markdown_parser.rs`
  convertInlines wikilink 臂）；inner 在解析期整体 trim。代码块/行内代码
  内的 `[[` 不产该 span（parser 层保证——负例回归锁定于
  `plan732_wikilink_negative_code_and_href`）；Markdown 外链（href span）
  不产 wiki 激活。**零 autodown-core / auto-down 仓改动。**
- 编辑壳入口消费：`flatten_inlines`（`ui/autodown_editor/core.rs`）同时读
  `span.attrs`——`attrGetStr(attrs, "wikilink")` 命中即链接 span：
  - `SpanStyle.link = true`（VM 轨样式补齐：wikilink span 无 `Mark::Link`，
    原先渲染纯文本；现走既有 `LINK_COLOR` 链接样式链）；
  - `LinkInterval { lo, hi, target }`（全块字节坐标 + target 原文）随
    `BlockBuf.links` 保留（与 mark 区间同快照口径：本地编辑 snapshot 失配
    即整块失活，直到外部真变化 rebuild 恢复；自回显走 PLAN-057 回声守卫
    不重建）。
- 文本域覆盖：段落/标题/列表/引用/表格 cell 叶（build_walk 全构造点带
  links；fence/math 叶构造点显式弃置）。首块、行内前后文、中文/别名/带
  `.ad` 原文均保留（身份判断四级解析归消费方后端，本层不做）。

## 2. 命中区间模型（K-01 冻结）

- `DocLayout.links: Vec<LinkRegion>`——`LinkRegion { block, lo, hi, target,
  rect }`，rect 为**单 layout-run 段矩形**（`push_link_regions`：行前缀
  偏移平移 + run 内钳制 + `index_x` 字节→像素，`push_byte_range_rects`
  选区同路）。折行链接每 run 一段、各段独立可命中；**整块矩形禁用**
  （判据=段宽显著小于块宽且内含于块域）。
- 产出时机：`render_frame` 布局期随 `DocLayout` 写回（快照匹配期才产）；
  读出经 `AutodownEditorCore::link_regions()`（测试/观察面）。
- 像素阈值：命中=段矩形严格包含，无外加命中带；拖动容差
  `LINK_DRAG_SLOP = 4.0px`（按下点起算累计位移，超阈判拖选）。
- 快照失配/折叠隐藏/只读视图实例：零区间（与样式退化、输入门控同口径）。

## 3. 完整点击门（U-02 / C-02）

门在 core `handle_input`（widget 两事件臂的 DocInput 汇聚点——与生产
共一实现，无第二套 hit-test）：

| 腿 | 行为 |
| --- | --- |
| 按下 | 命中段矩形即登记 `link_pending`（block+lo/hi+target 三重身份 + 按下点 + `single`（多击节律首击）+ `plain`（无修饰键））。**纯观察**：建焦/caret/选区等既有编辑流程不变 |
| 拖动 | 累计位移超 4px → 取消（拖选零激活）；`Drag`/选区路径不受影响 |
| 抬起 | 抬起点命中**同一链接区间**（三重身份 + rect 包含）且 `single && plain` → `DocOutput.link_activated = Some(LinkActivation)`。恰一次 |
| 清除 | 非左键按下、FocusLost、整树 rebuild（外部真变化/换内容）、下一次按下覆写 |

零激活族（测试锁定 `plan732_click_gate_negatives`）：拖选超阈、右/中键、
越界抬起（widget `is_over` 门）、跨链接/非链接抬起、shift 等修饰键按下、
多击节律第二/三击、换内容后陈旧命中、本地编辑暂态失活。

## 4. 激活出口与双 Str 载荷（U-03 / C-01、K-03 冻结）

- `DocOutput.link_activated: Option<LinkActivation>`（DocOutput 由此退
  `Copy`——各臂单次消费）；`DocEditor::publish` 转发到 `on_link` 回调
  （`Shell.publish` 真实路径）。
- 载荷归一（单一拆分点=激活构造处 `split_wikilink_target`）：inner 原文
  按首个 `#` 拆分、两侧 trim；无锚 `target_anchor = ""`（明确空串，非
  None/undefined 字面泄漏）；第二参**不是源块身份**。
- 事件绑定（.at DSL，双形态）：
  - `on "open-wiki-link": .OpenWikiLink`——**完整 kebab 名**（Plan-367
    引号式监听，事件表键 `on"open-wiki-link"`，`split_aura_event_key`
    引号感知 base；与 jade Vue 轨 `on "open-wiki-link"` 同名同位，主
    形态）；
  - `onopenwikilink: .OpenWikiLink`——属性简写（便利形态）。
- 消息形态：`DynamicMessage::Typed { widget_name, event_name: <handler
  名如 "OpenWikiLink">, args: [Str(target), Str(target_anchor)] }`——
  handler 名作事件名（onfocusblock 同族 codegen 直映）；双 Str 经
  `encode_payload`/`decode_payload`（PAYLOAD_SEP typechar "s"，Plan 499
  M2 mouse-area 坐标实参同路）过 iced Send 边界直达 VM handler 形参
  `.OpenWikiLink(target, anchor)`。无 update 层拦截——真 handler 消费。

## 5. origin 形态（U-04 / C-04、K-02 冻结）

**结构性实例绑定**（框架先验，非公开 origin 字段）：

- 回调闭包 per-View-node 构造（`View::AutodownEditor.on_link:
  Option<LinkCallback<M>>`，map/cloning 随扩），iced widget 实例持有自身
  callback——每个编辑器实例（key）的激活只经自己的 widget 发布自己的
  消息（widget_name 各自携带）。**全链无全局 last-mounted 单槽**（结构
  证据：pending/link 表/回调全部 per-key/per-instance，grep 无静态槽）。
- 换内容/失效：rebuild 清 pending（排队旧激活零冒领）；卸载=无 widget
  收事件。双同正文实例反例（`plan732_dual_instance_isolated_activation`）
  + 六环语料单实例派发（`plan732_six_ring_full_click_reaches_vm_handler`）
  为强制证据。
- 载荷不携带 key（契约第二参=锚点；来源身份由结构保证）。

## 6. 验证面口径（U-05 / C-05、K-04 冻结）

- **生产层 seam**：`UserInterface::build`（widget.layout → measure →
  render_frame 生产布局）+ `ui.update`（真实 `Event::Mouse`
  ButtonPressed/ButtonReleased + `mouse::Cursor`——widget.update 生产事件
  臂）+ `Shell` 消息收集。与生产共一实现；唯一不覆盖 OS 事件循环本体。
  实证：`plan732_real_iced_events_full_click_publishes_on_link`（widget 层）
  与 `plan732_six_ring_full_click_reaches_vm_handler`（全六环：corpus
  app.at 生产 parse → AURA → lowering → 事件泵 → VM handler state 落地，
  语料 `test/ui/plan732_wikilink/`）。
- **MCP 合成面只作负例/辅助**：`__mcp_click` 语义不变（ghost 落点=单次
  MousePressed 建焦，无抬起腿→完整点击门不闭合→零 wiki 激活；负例锚
  `plan732_mcp_click_ghost_semantics_zero_wiki_activation`）。注：MCP press
  后紧跟真实同区间 release 属语义上的完整点击、会激活——不视为通道
  泄漏。二进制指纹由交付回执绑定（jade T-02 按其契约 §1 重绑规则复验）。

## 7. 兼容矩阵（U-06 / C-06）

| 面 | 行为 |
| --- | --- |
| 未声明回调 | `on_link = None` 缺省——应用编辑行为零变化（输入/焦点/滚动/撤销/scroll_sync/placeholder 全保留；X 组回归+缺省单测） |
| View map/cloning | `LinkCallback` newtype（Arc<dyn Fn>），`map_msg` 跨消息类型包装不丢 |
| scroll_sync 包装 | Scrollable 包装臂与裸臂均透传 on_link |
| oninput/onfocusblock | 原装配零改动（convert_autodown_editor_native 仅增臂） |
| feature 降级 | autodown×code-editor 缺一退只读文本（原降级链不变） |
| 输入期行为 | 编辑壳重建模型下键入 `[[x]]` 的解析时机=外部回写 rebuild（B-6 族 owner=auto-down，不承诺输入期实时解析） |
| 非目标 | 保存来源/动作栏判定、caret/reveal、宿主命令桥、全局 Shift、vue/jet/ark/rust 生成器、jade/auto-down 仓文件 |

## 8. 测试索引（worktree `cargo nextest run -p auto-lang --lib --features autodown plan732`）

| 组 | 测试 |
| --- | --- |
| C 语义/命中 | `plan732_wikilink_regions_semantics` / `_negative_code_and_href` / `_wrapped_regions_split` |
| W 点击门 | `plan732_full_click_activates_with_payload` / `plan732_click_gate_negatives` / `plan732_stale_hit_after_content_change` |
| L 载荷 | `plan732_split_wikilink_target_known_answers` / `dual_str_payload_crosses_send_boundary` / `test_autodown_editor_on_link_message_channel`（AURA 环） |
| O 来源 | `plan732_dual_instance_isolated_activation` |
| R 真实事件 | `plan732_real_iced_events_full_click_publishes_on_link` / `plan732_six_ring_full_click_reaches_vm_handler` / `plan732_mcp_click_ghost_semantics_zero_wiki_activation` |
| X 兼容 | `plan732_local_edit_deactivates_until_rebuild` / `test_autodown_editor_on_link_callback_and_map` /（既有编辑器·视图族回归=裸 `cargo t`） |
