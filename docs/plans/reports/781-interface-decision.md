# PLAN-781 T-01 子内容锚点接口决策报告（合同冻结）

stage=work | task=T-01 | plan_revision=2 | 阶段 B（T-02~05）以本文为硬前置。
基线：worktree `D:/autostack/.wt/lang-781/auto-lang`，branch `plan-781-dev`，阶段 A 收口于 `c724ae25c`。

## 0. 调查基础与热点对账

- `ui/scroll/`（intent.rs 289 行 / controller.rs 375 行 / state / geometry / managed）：intent 为纯 offset 语义（resolve 只吃 ScrollState 六测量）；controller 注册表承载 bind/prime/enqueue/drain/fold + F-4 预热纪律 + 2026-10-11 offset 来源标记纪律。
- `View::AnchorSlot`（view.rs:638 + ui/anchor_slot.rs，PLAN-063 块锚定）：**零布局 wrapper 先例**——layout 委托子件、布局期记高度入全局注册表。其"layout 期无最终位置，只记高度"的注记是本设计的直接教训来源。
- 阶段 A 已证：iced `Operation` 在布局完成后遍历，bounds 带最终位置（LayoutCollector/ScrollStateReader 即此形态）；读回节拍（prime/drain/心跳 ≤2s）与 content-space 窗口化公式（`window = child − offset + viewport 原点`）已钉死（决策报告 §12 r2）。
- vue 侧：`scroll-pane (controller:)` → `:data-scroll-ctl` attr + `__scrollCtlEl/__sctlTo` JS helper 族（vue.rs:8195/3860-3879）——锚点 helper 完全同构可扩。
- native 目录：scroll 族 9900–9905；9906（code_editor.edit）/9907（Env.track）/9908–9921 全占；**9922/9923 空闲**（下一占用 9930 http.transfer）——实勘定案。
- 在途热点对账：PLAN-778（ui 方言外化）/779（native provider）均为 drafting 草案未执行，无 landed diff 冲突；PLAN-776/742 写面在 vm/ffi/http_server（本阶段不触）。本阶段写面 `ui/scroll/*`、`view.rs`、`aura_view_builder.rs`、`iced/renderer.rs`、`vm/native{,_catalog}.rs`、`ui_gen/vue.rs` 当前无其他 active owner。

## 1. 公共 API 冻结

### DSL 层（双端同形）

- 子元素属性 **`scroll-anchor: "<key>"`**：非空字符串；挂在任意元素上（col/text/button/row…）；由 view-builder 生成零布局 wrapper（见 §4）。pane 侧无新属性——锚点归属**包含它的最近 ordinary scroll-pane**。
- key 作用域 = pane 内；不同 pane 同名 key 不串线（寻址二元组 = (controller handle, key)）。
- 同 pane 内重复 key：**后注册覆盖** + stderr 可观察告警（`scroll-anchor duplicate key` 行）；不失效页面。
- 无 controller 的 pane 上的 scroll-anchor：构建期 stderr 告警 + 属性无操作（不包装、不影响布局）。
- managed content / 虚拟列表 / terminal 锚点：不支持（见 §5）。

### Native 层（id 实勘定案）

| id | canonical 名 | 签名 | 语义 |
|---|---|---|---|
| **9922** | `auto.scroll.to_child` | `to_child(handle str, key str) -> str` | 入队回执（**非完成证明**）：`"queued"` / `"error:<reason>"`；reason ∈ `bad_handle`（非 @scrollctl: 句柄）/ `empty_key` / `unsupported`（managed pane） |
| **9923** | `auto.scroll.anchor_state` | `anchor_state(handle str, key str) -> str` | 状态查询：`"pending"` / `"applied"` / `"invalid:<reason>"` / `"unknown"`（该 (handle,key) 无请求记录）；reason ∈ `not_found`（当前内容无此 key）/ `not_bound`（handle 未绑定/已换绑）/ `unsupported` / `unmeasured`（pane 尚无测量基线且未超时） |

既有 9900–9905 offset 族签名与行为零改动。

### 观测/驱动面（消费 T-11 工具，不新增）

完成证明 = `autoui_scroll_state` 读出 offset 变化（scroll_tick 增加）+ `autoui_visibility` 目标正面积交集；**禁止仅以 progress 或入队回执证明**。

## 2. 状态机与代际语义（冻结）

请求生命周期：`queued → pending →（applied | invalid:<reason>）`，一次性消费：

- **pending 存活**：等待「pane 预热（F-4 六测量基线）+ 当帧锚点注册表可解析」；未预热留队晚一拍（沿 drain/prime 既有节拍，不新增轮询）。
- **applied**：nearest 目标 offset 经既有 scroll_to 落盘通路消费（写臂投影回填 note_controller_offset 纪律保持）。applied 后用户滚动/视图重建**不被旧请求拉回**（请求已消费出队）。
- **绑定代际**：controller 注册表 `bind_controller` 的换绑检测扩充 generation 计数；请求携带入队时 generation，pane 换绑/重绑后旧请求 → `invalid:not_bound` 退场（**不落到新内容**——§5.B.4）。
- **内容代际**：目标 key 解析基于**当帧**锚点注册表（每帧构建期重建）；key 在当前内容消失 → `invalid:not_found`（一次性失败不重试）。字体/宽度变化后的**新请求**自然采用新布局（旧请求已消费/失效，不存在 stale 拉回）。
- 请求表有界：同 (handle,key) 重复入队 → 覆盖旧请求状态（重新 pending）。

## 3. nearest 算法（冻结公式，与阶段 A oracle 同源）

输入：目标 wrapper 的 content-space rect `(tx, ty, w, h)`（pane 坐标，operate 期实测）；pane 测量 `(offset_y, viewport_h, content_h)`：

```
若 h > viewport_h（目标高于视口）   → target_offset = ty            （顶对齐，起始区域可见）
否则若 ty + h > offset_y + vh（底出） → target_offset = ty + h − vh  （底对齐）
否则若 ty < offset_y（顶出）          → target_offset = ty           （顶对齐）
否则                                  → target_offset = offset_y     （已正面积相交，不动）
clamp [0, content_h − viewport_h]（geometry::clamp_offset）
```

单轴（Y）冻结为 v1；X 轴与双轴 pane 列为不支持面（`to_child` 对显式 X 绑定 pane 返回 `error:unsupported`）。

## 4. 实现映射（T-02/T-03/T-04 的接线合同）

| 层 | 改动 |
|---|---|
| `ui/view.rs` | 新变体 `View::ScrollAnchorMark { key: Arc<str>, child: Box<View<M>> }`（AnchorSlot 同构零布局 wrapper） |
| `ui/aura_view_builder.rs` | `scroll-anchor` prop 提取（extract_string_with 同款）→ 包装子树；空/类型错 = 解析告警不包装 |
| `ui/iced/renderer.rs` | wrapper widget（AnchorSlot 式）：operate 期以合成 id `scrollanchor:{key}` 调 `operation.container(Some(&id), bounds)` 上报**最终位置 bounds**；drain 期（controller 排空点旁）`AnchorBoundsCollector`（container 钩子收 `scrollanchor:` 前缀）+ ScrollStateReader 读数 → nearest → 并入既有 finals scroll_to 通路 |
| `ui/scroll/controller.rs` | 请求表 `(handle, key) → {generation, state, enqueued_at}`；bind 换绑检测挂 generation++；drain 联动解析与状态回写 |
| `ui/scroll/intent.rs` | **不改**——锚点请求走 controller 注册表独立队列，解析为绝对 offset 后复用 scroll_to；intent 纯 offset 语义不污染 layout 依赖 |
| `vm/native_catalog.rs` + `vm/native.rs` | 9922/9923 注册 + shims（`#[cfg(feature="ui")]` 同族，无 ui 降级臂返回 error:unsupported） |
| `ui_gen/vue.rs` | `scroll-anchor` → `data-scroll-anchor` attr；helpers `__sctlToChild(k, key)`（pane 内 `[data-scroll-anchor]` 定位 + `scrollIntoView({block:'nearest'})`）与 `__sctlAnchorState`（Vue 轨 applied 即回执后 DOM 就绪查询）；native sig 映射 + golden 同步 |
| `ui/scroll/mod.rs` | 头注记「刻意不落地的 ScrollAnchor」改记已落地形态与本文指针 |

## 5. 支持边界与非目标（冻结）

- 仅 ordinary scroll-pane（须有 controller——to_child 按 handle 寻址）；嵌套 col/row 包装子树允许（bounds 实测）。
- managed content / virtual-list / terminal 锚点：`error:unsupported` 显式拒绝。
- X 轴/双轴 pane、持久化 ScrollAnchor、跨 pane/跨页锚点、虚拟化内容的按需实例化定位：v1 非目标。
- Vue parity 以 DOM `scrollIntoView({block:'nearest'})` 等价承载，语义等价性由 T-05 双端几何驱动验收（几何 oracle 容差内），不以实现细节对齐。
- 既有 controller/offset API 与 2026-10-11 回声/抖动修复纪律零回归（T-02 单测面覆盖）。

## 6. 验收映射（V-01~06 → 合同条款）

V-01（真实定位/正面积）→ §3 公式 + §1 观测面；V-02（寻址/原始行号 key/col 包装）→ §1 key 语义；V-03（时序/换章/换绑代际）→ §2 代际；V-04（负例：空/缺/重复/越界）→ §1/§2 reason 集；V-05（重排/已可见不跳/过高起始可见）→ §3 三分支；V-06（旧 offset API 回归 + managed unsupported）→ §5。
