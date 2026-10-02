# PLAN-732 供给回执（jade-edit PLAN-037 T-02 解锁门对接口径）

日期：2026-10-03。产出：worktree `D:/autostack/.wt/lang-732/auto-lang`（分支
`plan-732-dev`，基面 master@1cabfeb89）。本件是 jade 契约 §4「合格供给回执
清单」五项的对岸供料件；jade T-02 联验（含原生 merged 生产窗口真实点击）
在其仓执行，本件不冒称已联验。

## ① 供给 Plan ID / revision / delivered commit

- PLAN-732 / revision 1（v2 frontmatter，6/6 任务勾选）。
- delivered commit：本 worktree 提交（hash 见计划文件 §9 work 记录；reviewed
  态在 review 阶段落定后由 merge 收口）。

## ② binary 版本 + SHA256（交付时点指纹）

- 交付时点主检出运行 binary：`auto 0.1.0+v0.4.2-2638-g713119788-dirty`，
  SHA256 `2287A53615B39BBFEA2E670FE7F479B98A4B0D6AFD5C0BA127695C2EDEAD3AD2`。
- **诚实边界**：该 binary 由外部会话自主检出重建（D-04 漂移在案），**不含
  本供给代码**；本供给证据绑定 worktree 测试二进制（见 ④命令行）。合并后
  外部重建的新 binary 才携带供给——jade T-02 按其契约 §1 重绑规则以当时
  实际 binary 复验，指纹变化本身不阻塞。

## ③ 公开 callback 契约

| 面 | 冻结值（T-01 裁决） |
| --- | --- |
| .at 绑定 | `on "open-wiki-link": .OpenWikiLink`（完整 kebab 名，主形态）或属性简写 `onopenwikilink:` |
| 事件名 | handler 名（如 `OpenWikiLink`）作 `DynamicMessage::Typed` 事件名；绑定键=完整名 `open-wiki-link` |
| 双参语义 | `(target, target_anchor)`：inner 原文首个 `#` 拆分、两侧 trim；无锚=空串（非 None/undefined 字面）；第二参非源块身份 |
| origin 形态 | 结构性实例绑定（per-View-node callback / per-widget-instance 发布，无全局 last-mounted 单槽）；载荷不携 key |
| handler 签名 | `.OpenWikiLink(target: str, anchor: str)`（VM 形参绑定经 encode/decode_payload） |
| Vue 兼容证据 | 绑定形态与 jade Vue 轨 `on "open-wiki-link": .OpenWikiLink`（app.at:1646）同名同位；Vue 引擎 emit 双参消费（ui_gen vue.rs 引号式多参 handler 锁测在案）；双轨单源事件名 |
| 权威文本 | docs/specs/auto-lang/ui/design/autodown-wikilink.md（SD-01）+ ui/architecture.md ADR-27（SD-02） |

## ④ U-01..06 证据清单（命令 + 已知答案/反例/真实事件）

命令（worktree）：

```
cargo check -p auto-lang                                    # 类型门（默认 feature 集）✅
cargo nextest run -p auto-lang --lib --features autodown plan732 test_autodown_editor_on_link   # 13/13 ✅
cargo t autodown                                            # scoped 78/78 ✅（改动前基线同绿）
cargo nextest run -p auto-lang --lib --features autodown --no-fail-fast   # autodown 档全量：红集与 master 同基线逐名全等（9 预存红 + state_file/plan502_m3 两在案 flake 复跑绿）→ 0 新红 ✅
cargo t（裸，日常档）                                        # 5033 测试：14 预存红与 master 逐名全等 + 2 在案 flake（plan502_m3/plan707 复跑绿）→ 0 新红 ✅
```

| U 条款 | 证据（测试名） |
| --- | --- |
| U-01 语义/命中 | `plan732_wikilink_regions_semantics`（中/英/锚点/首块/标题·列表·引用域、LINK_COLOR 样式、段宽<块宽非整块矩形）/ `_wrapped_regions_split`（折行分段独立命中）/ `_negative_code_and_href`（代码块·行内代码·外链零区间） |
| U-02 编辑与激活 | `plan732_full_click_activates_with_payload`（完整点击恰一次、建焦保留）/ `plan732_click_gate_negatives`（拖选/右中键/跨链接/非链接/shift/双击第二击七负例）/ `plan732_stale_hit_after_content_change`（换内容零冒领）/ `plan732_local_edit_deactivates_until_rebuild`（暂态失活+恢复） |
| U-03 全链载荷 | `plan732_split_wikilink_target_known_answers`（首 # 拆分+trim+无锚空串）/ `dual_str_payload_crosses_send_boundary`（双 Str 过 Send 边界 decode 还原）/ `test_autodown_editor_on_link_message_channel`（AURA 双形态绑定→Typed args 逐值）/ `plan732_six_ring_full_click_reaches_vm_handler`（六环贯通：core→publish→View→lowering→AURA→VM handler state 落地 `wiki_target="目标页" wiki_anchor="锚点甲" wiki_calls==1`+非链接零派发） |
| U-04 来源/世代 | `plan732_dual_instance_isolated_activation`（双同正文实例各发各来源、B 零 pending）+ 结构性证据（SD-01 §5：全链无全局单槽） |
| U-05 真实事件面 | `plan732_real_iced_events_full_click_publishes_on_link`（UserInterface 生产事件泵：真实 Event::Mouse/Cursor→Shell 消息逐值；越界抬起/拖选负例）+ 六环语料（生产 parse 路径 corpus `test/ui/plan732_wikilink/`）；`__mcp_click` 负例锚 `plan732_mcp_click_ghost_semantics_zero_wiki_activation`（ghost 语义不变、零 wiki 激活） |
| U-06 兼容 | `test_autodown_editor_on_link_callback_and_map`（map/cloning/缺省 None）+ 裸 `cargo t` 全日常面零新红（既有编辑器/视图族回归）+ scroll_sync 双臂透传（实现面） |

## ⑤ 原生 merged 生产点击可行性说明

机制上已就绪且经六环语料验证（真实 iced 事件泵 + VM handler state 落地）；
jade T-02 的 merged 生产窗口点击（打开正确目标页）在其仓执行——本供给侧
唯一注意点：`autodown_editor_sync` 首帧 no-op（预存事实，生产动态循环靠
次帧重降层补内容；六环语料测试显式两段降层模拟同节律，见 ADR-27 注记）。
native split 的 F-24-2 单列不属本供给范围。
