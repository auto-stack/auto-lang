# PLAN-651 三态支持矩阵（T-00 产物）

> block 类型 × {view, edit, streaming} × 双实现（autodown-engine TS ↔ autodown-core
> Rust + auto-lang VM 壳）。语料 = autodown demo corpus（`demo/src/content.ts`）+
> jade 实际文档集（`jade-garden/front/e2e/.runtime/workspace/wiki/`，10 文件）。
> 盘点日期 2026-09-18；证据为 file:line 实测（关键红格经主会话复核）。

## 0. 判定口径（Q-1 裁定）

| 态 | 格 = 绿的标准 |
| --- | --- |
| view | 同语料双侧渲染结构对拍一致（结构锚，非像素）；已登记的平台豁免（如 math/mermaid 图形渲染 web-only，PARITY #9）记 **绿(豁免)** 并注 DEBTS/豁免编号 |
| edit | 聚焦→编辑→回写 roundtrip **模型无损**（行为锚：parse(emit(doc)) ≡ parse(doc)）；TS 侧显式 v1 裁定（如 query/embed 冻结预览不可编辑）为双侧对齐基准 |
| streaming | 三态机语义一致：未闭合构造→段落字面降级 / 开放 fence 族→loading 骨架 / 闭合→终态（语料锚，tri-state fixtures 双侧同源） |

双格都按"各自已裁定的豁免"一致时记绿(豁免)；单侧缺裁定记差异格进红项清单。

## 1. 类型集合（069 manifest 23 起点 + 语料实测增删）

- 模型 17 BlockType（TS block-model.ts:386-404 ≡ Rust block_model.rs:326，双侧同构）。
- **TODO/DOING/DONE/NOW/LATER/Priority A-C 不是独立 kind**：ListItem 字面文本
  标记（TS tasks.ts 正则 + slash 插文本；Rust parser 不识别标记、随 ListItem
  文本自然保形）。069 的 23 类型 → 矩阵按 17 kind + 2 特殊行（任务标记态、
  Image 行内 span）展开。
- **Image 无块 kind**：inline span（Mark.Image），双侧一致。
- 语料增删：jade 文档集无 callout/details/math/mermaid/query/embed（demo corpus
  全覆盖）；jade 高频 = heading(19)/bullet(~20)/wikilink 行内(13)/**块锚 ^id(27)**/
  任务列表(8)/表格(1)/fence-rust(1)；heading `{#block-*}` IAL 为 jade 前端字面
  约定，parser 不吞（字面文本，编辑天然保形，不进红项）。

## 2. 矩阵

图例：✅绿 · 🟡部分(注) · ❌红 · 🕳️不适用 · (豁)=已登记豁免。
TS 侧 = autodown-engine（engine/src）；Rust 侧 = autodown-core（engine/rust）+
auto-lang VM 壳（ui/autodown_editor/core.rs，下记 VM）。核心证据缩写：
P=markdown_parser.rs S=serializer.rs C=core.rs V=autodown_render.rs。

| 类型 | TS-view | TS-edit | TS-stream | core-parse | core-serialize | VM-view | VM-edit | VM-stream |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Heading | ✅ builtin H1-6 | ✅ RichTextHost level | ✅ 行构即完整 | ✅ ATX+setext P:1248/1409 | ✅ S:334 | ✅ V:420 | ✅ Leaf(Heading) C:2640 | ✅ |
| Paragraph | ✅ | ✅ | ✅ | ✅ P:1416 | ✅ S:435 | ✅ V:936 | ✅ 主编辑面 | ✅ |
| Fence | ✅ widget 面板 | ✅ CodeBlockWidget | ✅ 唯一真 open 态 | ✅ P:1164（未闭→loading） | ✅ S:322 **含语言** | ✅ V:428 | ❌R1 **emit 丢语言** | ✅ |
| Blockquote | ✅ | ✅ 容器家族 | ✅ | ✅ P:1323 lazy | ✅ S:257 | ✅ V:511 | ✅ Seg::Quote | ✅ |
| ListBlock/ListItem | ✅ | ✅ +任务勾选动词 | ✅ | ✅ P:1367/1505 | ✅ S:275 | ✅ V:535 | ✅（有序规则未接线→R6） | ✅ |
| Table | ✅ | ✅ 七动词 | ✅ 唯一 stream 槽 | ✅ P:1351+1703 | ✅ S:216 含 align/IAL | ✅ V:594 | 🟡R7 cell 可编辑；无行列动词(已登记"另立")；**emit 丢 align/IAL** | ✅ pre-table ghost |
| ThematicBreak | ✅ | 🟡 降级 preview（slash 可插） | ✅ | ✅ P:1262 | ✅ S:410 | ✅ V:922 | 🟡 Raw 固化只读（同 TS 降级口径） | ✅ |
| Callout | ✅ widget | ✅ 容器+AttrHost | ✅ 未闭→字面 | ✅ P:1269 | ✅ S:358 | ✅ V:630 | ✅ Seg::Callout 往返 | ✅ |
| Details | 🟡D1 面板依赖 EngineEditor 加载 | ✅ summary/open 动词 | ✅ | ✅ P:1295 | ✅ S:367 | ✅ V:699 ▸/▾ | 🟡R5 **折叠不可交互**(恒 ▸)；summary/open 往返 ✅ | ✅ |
| WikilinkBlock(块级) | ❌R8 死 kind（parser 不产） | ❌R8 | ✅ 随容器 | ❌R8 块级构造不存在 | ✅ S:375 | ✅(手工构造可达) | ❌R8 catch-all 丢语义 | 🕳️ |
| QueryBlock | 🟡D1 widget 完整但 editor 侧注册 | ❌R9 冻结预览（v1 裁定） | ✅ | ✅ P:1309 | ✅ S:422 | 🟡 "未求值"面板(求值归宿主→(豁) §10.5) | ❌R2 **catch-all 空段，回写丢 query** | ✅ |
| BlockEmbed | 🟡D1 同上 | ❌R9 同上 | ✅ | ✅ P:1314 | ✅ S:425 | 🟡 占位面板(装载归宿主→(豁)) | ❌R2 **空段，回写丢 src** | ✅ |
| Mermaid | 🟡D1 | ✅ 源码编辑器+预览 | ✅ open=通用 fence loading | ✅ P:1191 | ✅ S:428 ```mermaid | 🟡 源码面板 web-only(豁 PARITY#9) | ❌R3 **emit 丢围栏→降级段落** | ✅ |
| MathBlock | 🟡D1 | ✅ 源码+katex 同屏 | ✅ 未闭→字面 | ✅ P:1226 | ✅ S:431 `%{ }%` | 🟡 源码面板 web-only(豁) | ❌R4 **emit 丢 `%{ }%`→降级段落** | ✅ |
| 任务标记态(TODO 等) | 🟡 字面标记+t 助手死代码(R10) | ✅ 字面编辑 | 🕳️ 随 ListItem | ✅ 字面 | ✅ 字面 | ✅ 字面 | ✅ 字面 | ✅ |
| Image(行内 span) | ✅ img+破图回退 | 🟡R11 宿主内降纯文本(v1) | 🕳️ | ✅ 行内 | ✅ | ✅ 行内 | 🟡R11 同口径 | ✅ |
| **块锚 `^id`**(横切) | ✅ anchor attr+ensureBlockAnchor | ✅ 模型保形 | ✅ | ✅ P:2787 extractAnchorBlock | ✅ S:442 withIdSuffix(heading/leaf/容器内) | n/a | ❌R-ANCH **emit 丢锚**(jade 27 处高频) | ✅ |

D1（TS 结构性注记，全 5 类型：Details/Math/Mermaid/Query/Embed）：view 面板注册
在 EngineEditor 模块加载侧（EngineEditor.vue:261-264），纯渲染消费者
（MarkdownRender/StreamingRenderer 直用）降级 unknown-node；测试被迫
`void EngineEditor` 补注册（stream-tri-state.test.ts:17-19）。渲染层已注册的
Fence/List/Table/Callout/Blockquote 无此问题。

## 3. 红项清单与批次定价

### T-01 第一批（高优：VM 编辑回写数据/结构丢失；jade 语料定价）

| # | 红项 | 位置 | 闭合设计 |
| --- | --- | --- | --- |
| R1 | Fence emit ` ``` ` 裸发丢语言（jade 有 rust fence） | C emit_seg:2834 | 发射读 `BlockBuf.syntax`（已存 language），` ```{lang} `；对齐 S:322 fenceMd |
| R2 | QueryBlock/BlockEmbed 编辑=空段，回写丢 attr | C catch-all:2762 | 进 `Seg::Raw(canonical)` 冻结源行——`$query(..)`/`$embed(src: "..")` 与 S:422-427 同形；Raw 不可聚焦=TS 冻结预览裁定对齐，emit 原文保形 |
| R3 | Mermaid emit 丢 ```mermaid 围栏→降级段落 | C catch-all | 闭合 mermaid 进 Fence 族叶（kind=Fence, syntax="mermaid"），编辑面 mono 源码（TS 编辑=源码编辑器对齐；图形预览维持 web-only 豁免），emit 走 R1 通道 |
| R4 | MathBlock emit 丢 `%{ }%`→降级段落 | C catch-all | 新 `LeafKind::Math`（mono 源码叶）；emit `%{\n..\n}%`（S:431 同形）；Enter/Backspace/行首规则同 Fence 守卫 |
| R-ANCH | 块锚 `^id` emit 丢失（jade 27 处；[[page#锚]] 断链） | C 无 anchor 通道 | `BlockBuf.anchor` 随 build_walk 收 anchor attr；emit 段 Leaf(Paragraph/Heading) 尾补 ` ^id`（S:442 同形，含 catch-all 降级臂）；Enter 拆分锚随头块、合并保头锚（v1 语义，测试钉死） |

验收锚：逐项行为测试 + `t651_closure_corpus` 双跑锚（语料 parse→emit→parse
幂等 + emit 字面断言）；TS 侧同语料经既有 serializer-roundtrip 全绿即为基线
（本批 TS 零改动）。

### T-02 第二批（余项：交互差距/发射保真）

| # | 红项 | 位置 | 设计 |
| --- | --- | --- | --- |
| R5 | Details 编辑面折叠不可交互（恒 ▸） | C:3377 绘制 | 消费 open attr（▸/▾），点击发 toggle 消息写 seg.open→重建；对齐 TS marker 翻转一步 undo |
| R6 | 行首规则缺口：`1. ` 有序未接线、`#` 仅到 h3（TS slash 可达 h6）、`---`/任务标记不触发 | C:2974 | 按语料频率接线：`1. ` 有序、`#`→h6 对齐 TS 输入规则面 |
| R7 | Table emit 恒 `---` 丢 align/IAL | C:2923 | Seg::Table 携 align 列表 + 表 IAL（cols/rows）随 emit 还原（S:93/194 同形）；行列增删动词维持"另立"登记 |

### DEBTS 提案（经用户裁定转债务，本计划不闭合）

| # | 项 | 提案理由 |
| --- | --- | --- |
| R8 | WikilinkBlock 块级死 kind（双侧一致：parser 不产、仅 serializer 认识） | 双侧对齐的模型卫生项：退役该 kind 或补块级语法，属模型层裁定，非编辑器闭合 |
| R9 | TS Query/Embed 编辑面=冻结预览（v1 显式裁定 host-controller.ts:281-282） | 已是双侧对齐后的共同基线（VM T-01 R2 对齐此裁定）；增强=query attr 就地编辑另行立项 |
| R10 | TS tasks.ts 轮换/优先级/日程助手零消费方 | 疑留 Vue 应用侧接线；engine 内死代码，清理或接线归 auto-down 侧 |
| R11 | Image 编辑宿主内降纯文本（双侧同口径 v1） | 行内层选区/富编辑 = 路线图行内层计划（EDITOR-CONTRACT §9 路线图注记） |
| D1 | TS Details/Math/Mermaid/Query/Embed view 面依赖 EngineEditor 加载 | 结构性（widget 依赖 editor 数据通道 loader）；改注册层级动导入图，收益=纯渲染消费者；对拍不暴露数据丢失 |
| (豁) | VM math/mermaid 图形渲染、query 求值/embed 装载宿主桥 | 已登记豁免（PARITY#9 / PLAN-041 T7 / §10.5 平台面），维持 |

## 4. Q-2 落地通道裁定

T-01/T-02 修复面 100% 在 auto-lang VM 壳（core.rs）→ **本计划 auto-lang 主通道**；
autodown-core(auto-down) parse/serialize 全绿**零改动**；TS 侧零改动（基线绿）。
auto-down worktree 仅承担 T-03 的 gallery RC-E 联动小改（component-gallery
units.mjs + App.vue + pages + VM twin）。Q-3 未触发（无新工具链级硬骨头）。

## 5. 对拍 gate（T-03 落库形态）

- Rust 臂：core.rs `t651_*` 测试组（行为锚 + closure corpus 幂等锚），`cargo t` 可跑。
- TS 臂基线：engine 既有 stream-tri-state/serializer-roundtrip/editor __tests__
  全绿即为对拍基准（本计划不改 TS）；golden 双跑既有 parse_parity 金标机制不动。
- 常驻化：矩阵本文件 + 测试组命名 `t651_` 前缀可检索；jade gallery RC-E 单元
  状态转"对拍 gate 在库"（auto-down 侧联动提交）。
