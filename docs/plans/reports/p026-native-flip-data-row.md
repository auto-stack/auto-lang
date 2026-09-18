# P026 native queue 覆盖翻转数据行（T-06，§5.1 D3 口径）

日期：2026-09-18 ｜ 基线：plan-026-dev @ 5a560adc1（025 landed master
e352437b0）｜ 仪器测试：`native_flip_coverage_data_row`
（crates/auto-lang/src/ui/desktop_protocol/coverage.rs）

## 仪器

examples/ui 全量 app `.at`（src/front/*.at 全文拼接）→ 解析提取
`widget App`（缺省首个 WidgetDecl）→ `VmBridge::new_from_decls`（空绑定）
→ `AuraViewBuilder::build`（VM 轨运行时 aura→View 构造器，与 a2r
codegen 同以"降级到 View IR"为口径——§5.1 D1/D1' 定案后两轨 View 层
同构）→ `scan_native_view × judge(native_queue_set)`。

编译级验证另由 e2e 真 exe 承担（p026_native_display_arm，T-07）。

## 终版数据行（2026-09-18）

- **overall：Covered 16/35 = 45.7%**
- **judged（剔除仪器桶）：Covered 16/21 = 76.2%**
- **阈值（T-01 D3 定案）：≥95% 且缺项全在册 not-yet**
- **裁定：两口径均未达阈值 → 维持 native `auto` = independent（不翻）**

### 判定桶

| 桶 | 数量 | 样本 |
|---|---|---|
| Covered | 16 | 001 002 003 004 005 006 007 008 010 012 013 014 019 020 022 023 |
| NotCovered（语义缺项在册） | 5 | 009(opacity-50) 018(fixed+hidden+z) 021(sticky+top+z) 024(style-grid) 041(hidden+popover) |
| parse-fail（仪器桶） | 9 | 015 016 017 029 030 031-image-viewer 031-paint 043 044 045 中 9 项（多文件/use-import 装载面——单源拼接仪器限制，非 native 判定） |
| bridge-fail（仪器桶） | 3 | 011 026 027（store/import 桥构造依赖） |
| no-widget（非 app） | 1 | stylekit（样式库目录） |

（逐样本行以 `native_flip_coverage_data_row` 每次运行的 eprintln 数据行
为准。）

## 缺项面（全在册 not-yet，无一静默）

| 缺项 | 语义 | 归册 |
|---|---|---|
| opacity-* | alpha 合成无 DrawOp 通道 | 解释态 target_set 同 not-yet |
| fixed/sticky/top-/z-index | 定位族 | native 块流静态帧无定位通道 |
| hidden | display:none 语义 | 需节点跳过臂（ramp v3 候选） |
| 样式版 grid（display:grid/grid-cols 类） | 类驱动网格 | View::Grid 变体臂不覆盖 style 路径（ramp v3 候选） |
| popover（View::Popover 变体） | 悬浮层 | shell a2r 设计 S1 范围（§10-③ 拆分裁定） |

## 翻转点（已备未启）

`resolve_native_frame_mode`（client_entry.rs）已升级为**扫描制观测**：
Auto → 真扫描，观测行携带逐 App 缺项清单（原 v1 恒定文案退役）+
queue-covered 命名；**裁决保持 Auto→Pixels 不变**。数据门达标
（ramp v3 复测 ≥95%）时 Covered 臂改返 `FrameMode::Commands` 即完成
翻转（one-line）——desktop-protocol-v1.md §1.8 翻转点随注。

## 附带收口（T-06 样式降级放行批）

解释态 target_set 同款保真边界（非静默扩权，逐类随注
`Coverage::native_queue_set`）：overflow-（含 x/y 轴族）/min-w-/min-h-/
leading-（PLAN-527 typed LineHeight 可达）/flex/block/underline 族/
cursor-/outline-/transition/antialiased/shrink-/whitespace-/relative/
tracking-/backdrop-（518 G8 冻结词汇）/字重族补全（SemiBold/Light/
ExtraLight/ExtraBold/Thin）。

025 防漏钉样式样本换防：underline（026 放行）→ opacity-50。
