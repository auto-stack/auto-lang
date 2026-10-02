# PLAN-731 T-02..T-05 受控谱+双谱对照报告（2026-10-02，R1 修正版）

> R1 F-1/F-2 修正：基线谱=T-00 态（7454e4c13=插桩无缓存）二进制全档重跑
> 归档（本文件全部数字直出自 ladder-{baseline,after}[-release].jsonl
> 在档文件，可 `python ladder.py --label <x> --out <p>` 复跑）；此前引用
> 的 7.06/38.66 等为首次运行控制台数字（跨跑波动注记：s5 P50 6.24-7.06、
> 滚动尾帧 P95 13.88-38.66 两轮实测——首轮含 900px 首步整形突发）。

## 1. T-05 双谱对照（改前/改后，同一阶梯负载：041-auto-edit VM 轨，30 键×3 相位+25 滚动步）

### debug（ladder-baseline.jsonl → ladder-after.jsonl，文件实算）

| 档.相位 | s5 P50 | s5 P95 | ce_fold_scan 均值/帧 |
|---|---|---|---|
| 5kb.type | 0.49 → 0.44（-10%） | 0.62 → 0.54 | 0.047 → ~0.001ms |
| 100kb.type | 2.04 → 0.53（-74%） | 2.34 → 0.59 | 1.20 → 0.001ms |
| 1mb.type | **6.60 → 0.81（-88%）** | 8.98 → 1.13 | **6.32 → 0.001ms（-99.98%）** |
| 1mb.scroll | **8.67 → 0.79（-91%）** | **13.88 → 1.49** | 7.82 → 0.001ms |
| 1mb.typenl（024 协议形） | 6.62 → 0.80（-88%） | 12.86 → 1.44 | 6.90 → 0.001ms |
| 100kb.scroll / typenl | 1.42→0.49 / 1.25→0.50 | 1.95→0.59 / 2.10→0.65 | 0.87/0.81 → 0.001ms |

### release（ladder-baseline-release.jsonl → ladder-after-release.jsonl）

| 档.相位 | s5 P50 | s5 P95 | ce_fold_scan |
|---|---|---|---|
| 100kb.type | 0.24 → 0.13ms | 0.36 → 0.30 | 0.10 → 0ms |
| 1mb.type | **1.16 → 0.36（-69%）** | 1.30 → 0.79 | 0.78 → 0ms |
| 1mb.scroll | 1.25 → 0.42 | **2.35 → 0.94** | 0.84 → 0ms |
| 1mb.typenl | 1.08 → 0.34 | 1.40 → 0.56 | 0.80 → 0ms |

### S1-S4 零回退门（frozen ②——文件实算，R1 F-2 修正）

- **代码事实**：S1-S4 打点与实现路径零改动（diff 仅 frame_segments 增
  s5 字段+renderer 加根包装；S1-S4 note 调用点未触碰）。
- **谱面**（type 帧 segsum 均值）：5KB 4.62→4.42（-4.2%）；1MB
  8.55→8.88（**+3.8%**）；100KB 6.93→4.41（-36.3%，改善向）。分段分解
  （1MB）：s1/s2 恒定（0/0.15ms），波动主源=s3a（MCP 同步块，跨跑
  2.86-2.95ms）与 s3b（4.01-4.07ms）的帧样本构成差异。**无系统性
  回退**（最大劣化 +3.8%@1MB，同带内）。
- **视觉零变化（frozen ①）**：golden 三形 type/scroll/resize 改前/改后
  0.00% 全绿。
- **判定口径不换（frozen ③）**：本件零测量定义变更（只增 s5 字段）。

## 2. T-04 泵率谱（release 改后，distinct present/秒+孤儿帙）

| 相位 | pump/s | 事件数 | 孤儿帙 | 段帧数 |
|---|---|---|---|---|
| 100kb.type | 2.47 | 10 | 32 | 12 |
| 1mb.type | 1.46 | 6 | 37 | 19 |
| 1mb.scroll | 0.41 | 3 | 17 | 20 |
| 1mb.typenl | 1.69-2.14 | 7-9 | 34-36 | 43 |

**归因注记（AC-05 受控臂）**：泵消费率 ≪ 实际呈现率——相位墙钟（30 键 60ms
节拍 ≈1.8s 完成）与 begin 行数对照证明呈现随键入节奏发生，而 `__frame_pump`
消息（listen_raw RedrawRequested→异步回环）消费滞后/合并——**present=-1
孤儿行=测量配对面（帧分段滞留 prev 被覆写），非呈现丢失**。024 下游
"present=-1 掉泵直落"同机理（其 ~108ms 帧时为真实 S5 工作慢——本件已清偿
其上游成本面）。泵率通道（pump_per_sec/orphan_fallthrough_frames）已入谱
脚本与 JSONL，下游重判件可直接消费。

## 3. T-02 layout 增量（AC-03 受控臂）

- 勘定（T-00-survey §4②）：iced 0.14 无脏子树/memo 公开界面；Element 无
  Clone、view() 契约恒重建。**实测成本**：s5_layout release 0.02-0.05ms/帧
  （全相位全档）——非瓶颈。
- 替代案成效（Q-1 裁定路线）：T-01 fold 缓存为主力（§1 谱）+受控重建谱
  （fall-through 帧 s3b+s4 release ~0.7ms——泵消息驱动的重建成本受控）。
- golden 三形 0.00% 全绿（§1）。

## 4. T-03 组件实例缓存（AC-04 受控臂）

- `ce_widget_new` 实测非成本：render_dynamic_view 编辑器臂（含
  CodeEditor::new+set_text diff）debug 0.17ms/rebuild（code_editor 臂
  4136µs/39）。024 的"重建→全量重整形"关联=同帧伴随（重建帧必过
  draw→fold_scan+整形），插桩定谳非因果（T-00-survey §3.4）。
- 编辑器态本就 registry 跨帧（TEXTAREA_CONTENTS 模式）——「实例缓存」在
  iced 即时模式语义下无可实施面；重建计数受控谱=code_editor 臂 n（每重建
  帧一次，值恒定）。
- **046 memo 正交性**：本件零触碰 memo_deps/ui_epoch——`cargo t memo`
  92/92 绿（零扰动实证）。

## 5. 预存红对账（T-06 前置）

`projector_counter_layout_and_hits` 在纯净 master 基面（7d50989f7）同
FAIL——预存红族，非 731 引入（对照 worktree 实录）。
