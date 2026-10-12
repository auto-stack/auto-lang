# PLAN-725 T-00 勘定记录：键入帧五段成本链分段剖析

工作树 `D:/autostack/.wt/lang-725/auto-lang`（branch plan-725-dev，基 96c876cea）。
基准负载 `examples/ui/041-auto-edit`（menubar/toolbar/tree/tab/code_editor/状态栏
全 chrome——下游帧档工作负载同形）；MCP fixture 通道装载 fixture 文件
（AUTOUI_TEST_FIXTURES 写 `auto_open_path` + 触发 App.Tick→ConsumeOpen）；
驱动 `autoui_type` 单字符 30 键 60ms 节拍（下游 stage_frame 同协议）；
读回 stderr `[P725-FRAME]` 分段行（`ui::frame_segments` 探针，AUTO_FRAME_BENCH
同门）。谱 JSONL：`ladder-baseline.jsonl`（改前）/`ladder-after.jsonl`（改后）。

## 1. 改前基线谱（master@96c876cea + 探针，2026-10-02）

| 档 | segsum P50/P95 (ms) | s1 | s2 | s3a | s3b | s4 | builds |
|---|---|---|---|---|---|---|---|
| 5KB (~120 行) | 11.68 / 14.90 | 0 | 0.19 | 5.19 | 3.28 | 2.33 | 2 |
| 100KB | 24.04 / 36.85 | 0 | 0.14 | 3.86 | 2.80 | **19.64** | 2 |
| 1MB | 126.76 / 159.13 | 0 | 0.07 | 6.15 | 3.14 | **120.98** | 2 |

（分段=typed 帧均值；segsum=S1..S4 之和；S5=layout+draw 未插桩=total−segsum
残差口径，见 §5。）

## 2. 四项勘定

### ① Element 缓存 put-then-take 快道命中率 → **死缓存（永不命中）定谳**

- 证据（代码内三重）：写点 `renderer.rs` view() 尾部 `put(Some(result))` 后
  **立即 `take().unwrap()` 返回**（iced 要求 owned Element——无法既保留又
  返回）；另两写点恒写 None。入口态恒空 → 快道 `take()` 分支永不进入。
  代码注释自证：`__frame_pump` 臂 :18005「不等 Element 缓存 take 空后的
  fall-through 帧」。
- 推论：**每 view() 调用（含非脏帧）都付全树 Element 重建**（cached_
  converted_view 深克隆 + render_dynamic_view）。孤儿帧行（present=-1）实测
  其成本：改前 5KB ~1.8ms/帧、1MB ~101-120ms/帧。
- 处置（T-04）：修复=不可能（iced 所有权模型）；死写移除（直接返回+勘定
  注释）；快道保留为防御面。非脏帧成本经 ③ 根因修复后降至 ~1.3-2.4ms
  （改后孤儿帧行）。

### ② G-3 载荷形态择路 → **静态消费判定形（非 lazy/delta 通道形）**

- 实测：MCP `autoui_type` 驱动走模型层直写（action channel），**on_change
  闭包不触发**（全程 s1=0）——下游帧档（同驱动协议）的 110ms 里 S1 分量
  为零；S1 仅真实键盘路径付费。
- 消费面实勘：`on_with_input_for` 对 input_value 的消费点恰三处且**全部
  静态可知**：(a) payload 内 `$event` 前缀实参（事件串静态）；(b)
  input_state_map 双向绑定（`scan_node_for_inputs` 只扫 input/textarea/
  Input 标签——code_editor/autodown_editor 恒不注册）；(c) 空 payload 首实参
  （`.Edit(str)` 契约）。
- 择路：**构造期静态判定**（`dynamic::input_payload_consumed`）——三点均
  不命中 → 消息 `input_value: None`，闭包跳过 `code_editor_text` O(doc)
  物化。较 lazy 通道形（IcedMessage 类型改造 ~70 构造点）改动面小两个量级；
  契约保真：判定命中臂逐字面保持旧发射。保守语义：绑定表缺席=携带
  （宁缺勿错——判定只允许把白运的全文变没）。
- 同族：autodown_editor on_change 闭包同款改造；textarea/Input 臂的
  input_value 来自 iced 事件回调形参（无主动物化）——零改动，scope 注记。

### ③ G-4 脏域化形态择路 → **根因不在 builder 域：S4 随文档尺寸线性爆炸**

- 实测分段：s3a/s3b 与文档尺寸**无关**（~5ms/~3ms 恒定）；**s4 线性放大**
  （2.3→19.6→121ms）。计划书「110ms 纯结构成本非文本量」的假设在当前
  master 二进制上不成立（下游旧二进制 0.4.2+b50dce902 时代的归因可能为真，
  现势 master 已含 716/721 系优化）。
- 臂级钻取（`[P725-ARMS]`/`[P725-ARM]` 探针）：code_editor 臂 94-95ms
  @1MB；臂内再切分：`ce_set_text_search` 67µs、`ce_widget_new` 5µs、
  `ce_hosted` 0µs、**`ce_scrollable` 95375µs**、`ce_wrap_debug` 38µs。
- **根因链定谳**：`iced::widget::scrollable::new` → `enclose()` →
  `content.as_widget().size_hint()`（iced 0.14 构造期急切求尺寸提示）→
  CodeEditor `size()` hosted 臂 → `core.content_height()` →
  **`fresh_fold_map()` 把全文档逐行物化成 `Vec<String>`**（core/mod.rs:1078，
  Plan 673 注释自认 O(n)）——每次 Element 重建（每帧、脏与非脏皆然）执行。
- 处置（T-03/T-04 本体）：`content_height()` 无折叠快道（folds 空 →
  total×line_height，rope 摘要 O(1)）——与慢路径逐值等价（空折叠集的
  退化形态），单测锁定（`content_height_fold_fastpath_equivalence`）。
  有折叠时维持 O(n) 慢路径（折叠开关稀疏，正确性不变）。
- **memo 域扩面裁定：不扩**。定量门已由上述修复达成（§4）；builder 域
  剩余 s3b ~3ms（5KB 档），memo/keyed-for 扩面（PLAN-045/046/047 机制在库）
  属下游 app 侧 opt-in 消费面（SD-01 注记），且 P721-R1 读侧机理未定谳前
  version_fast 快道受 frozen⑤ 约束——本件零扩面代码交付，判定记录在档。

### ④ S2/S5 占比定谳

- **S2（VM 段）非瓶颈**：0.07-0.33ms/帧（标量写 handler）——Q-4 裁定输入：
  解释器段定向优化**无需立项**。
- **S5（layout+draw）**：未插桩（iced 无公开钩子）；present 配对（帧泵序障
  代理）受 bounds 回路竞争不稳定（配对率 3-4/30），total 口径弱。1MB 改前
  谱偶发 ~100ms 残差帧（装载后首键——编辑器内核首帧 shaping 域，plan 非
  目标在册债）。处置：残差口径注记；占比如需精测定谳另立剖析件。

## 3. 改后终态谱（全修复：T-01 单帧单建 + T-02 载荷判定 + content_height 快道 + T-04 死写移除）

| 档 | segsum P50/P95 (ms) | s2 | s3a | s3b | s4 | builds | vs 基线 |
|---|---|---|---|---|---|---|---|
| 5KB | 5.30 / 10.59 | 0.13 | 0.88 | 3.12 | 1.62 | **1** | P50 −55% |
| 100KB | 5.88 / 10.70 | 0.14 | 0.94 | 3.33 | 1.72 | **1** | P50 −76% |
| 1MB | 8.07 / 10.01 | 0.14 | 2.95 | 3.48 | 1.68 | **1** | P50 −94% |

- **builds=1**：单帧单建断言成立（MCP 活跃脏帧 view_with_debug_gated 恰一次）。
- **尺寸缩放消除**：100KB≈5KB（5.88 vs 5.30）；1MB 残差 +2.8ms（s3a 的
  vtree 转换随编辑器节点值串增长——量级可忽略）。
- 孤儿帧（非脏 fall-through）s4 降至 ~1.3-2.4ms/帧。

## 4. 定量门定参（AC-04 冻结值）

- **冻结**：5KB 档键入→present **S1..S4 分段和 P95 ≤ 16.7ms**（@60Hz 上游
  基准 app VM 轨，本谱口径）。改后实测 P95=10.59ms **PASS**（改前 14.90
  亦在界内——基线谱显示当前 master 的 5KB 档结构性成本已低于预算，下游
  110ms 时代差距来自旧二进制+重 handler 负载；本件交付把**尺寸缩放面**
  清零并双倍余量）。
- **大文件档**：键入成本谱不随文档尺寸线性放大（1MB/5KB P50 比 ≤1.6），
  **PASS**（8.07/5.30=1.52）。分段可分离内核 O(n) 债（首帧 shaping 残差）
  归因在册（editor-kernel「现状限制」），管线段增量形成立。
- present 配对率注记：泵序障代理受 bounds 回路竞争（§2④），total 口径以
  segsum 为准入——保守方向（segsum 不含 layout+draw，实际帧总时长 ≥ 谱值）。

## 5. 探针通道口径（frame_segments，常驻门控）

- `AUTO_FRAME_BENCH=1` 同门（与 frame_bench 单一门语义）；门关零分支零写入。
- `[P725-FRAME] begin/present/s1/s2/s3a/s3b/s4/builds/dirty`——prev/cur 双槽
  配对（泵序 begin→present）；present=-1 = 无泵呈现帧（fall-through 成本面）。
- `[P725-ARMS]`/`[P725-ARM]`：Element 臂累积/单发钻取行（勘定遗留面，同门）。
- 复跑：`python docs/plans/evidence/725/ladder.py --label <tag> --out <path>.jsonl`
  （同负载同脚本——复审复跑面；fixture 幂等再生成）。
