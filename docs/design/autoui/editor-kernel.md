# 编辑器内核：rope 单写者版 + 统一 delta + back 分块读（editor-kernel）

> **定性**：需求级/专题设计（Plan 468：slug 不带号，归 `autoui/`）。
> **状态**：draft（2026-09-21，PLAN-673 T-00 立档）——三件（rope 路线/delta 形状/
> 分块读形状）裁定已决；实现批进度见 [PLAN-673](../../plans/673-editor-kernel-rope-delta-chunk.md)。
> **关联**：
> - [Design 08 UI Systems](../08-ui-systems.md)（AURA / 后端矩阵）
> - 供料来源：auto-edit 仓 PLAN-004 T-01 供料包（本仓
>   `docs/plans/attachments/673-674-m1-supply.md` §1/§2/§3）
> - 先例：Plan 413–428 code_editor 全链；Plan 428 fold_map（视图态边界）；
>   Plan 671 T-06（内建单源注册表）；P670-D1（双轨 JSON 逐字节对齐纪律）
>
> **知识分层**：本文回答「编辑器内核为什么这样改、三件形状是什么」；
> 「某次改动怎么做的」见 PLAN-673；「当前实现长什么样」见
> `docs/specs/auto-lang/ui/overview.md` code_editor 段。

---

## 1. 问题陈述

下游消费方（auto-edit）M1 性能轨与 M2 大文件线被三个上游结构性缺口阻塞
（供料包 §1/§2/§3，证据路径以 auto-edit main@872345a 为基）：

1. **每次击键复制全文**：code_editor 的 on_change 载荷是整串文本回读
   （`iced/renderer.rs:25199-25204` 每击键 `code_editor_text(key)` +
   `IcedMessage.input_value: Some(full_text)`）；下游 store 五处以此维护
   字符串镜像。大文件下是结构性死罪。
2. **大文件打开/持有**：文档缓冲存于 cosmic-text `Buffer`（行数组形态，
   非本仓自有结构）；`set_text` 全量重写（1MB 曾 26s，视口 hack 后 4.3s，
   `core/mod.rs:570-574` 在案）；`text()`/`fresh_fold_map`/`find_next`
   全文档遍历。100MB/1GB 语料架构性不可达。
3. **back 整串读**：`File.read_text` 只支持整串交付；大文件模式下 front
   无法"不拥有全文、视口拉取"。

**显式不做**（供料 §1 红线）：协作编辑——无 OT/CRDT/多编辑流合并；
copy-on-write 仅服务快照隔离（后台解析/搜索/diff/保存只读），非多人并发。

## 2. 现状锚点（2026-09-21 worktree 实勘）

| 面 | 锚点 | 关键事实 |
|---|---|---|
| 内核 | `crates/auto-lang/src/ui/code_editor/core/mod.rs` | `CodeEditorCore` 持 `Mutex<SendEditor(ViEditor<'static,'static>)>`（:245/:260，unsafe-Send，访问全程经 Mutex 串行化）；文档在 cosmic-text `Buffer`（`Arc<Buffer>` 共享进 SyntaxEditor，:443-455）；undo 历史归 ViEditor/SyntaxEditor 所有（语言切换已会重置，:516-537） |
| 渲染 | `core/render.rs` + `core/draw.rs` | **渲染契约已是自有件**（Plan 428 P2）：`layout_runs` → owned `TextRun` 片段（`draw.rs:59-68`），iced 侧不持有 live Buffer |
| 全文档遍历点 | `core/mod.rs` | `text()`（:557-560 全行 join）、`fresh_fold_map`（:808-818 每次 fold/帧）、`find_next`（:707-762 逐行 regex）、`content_height`（:1504-1513） |
| 事件流 | `iced/widget.rs` + `ui/view.rs:650-669` | 同步 `shell.publish(f())`，**无事件队列**；`external_dirty`（:286-291）是菜单/工具栏 native 编辑（undo/redo/cut/paste）的重发布信号面（041 Save 陈旧修复 3dd1bcbd4） |
| 内建注册 | `vm/codegen.rs` | 表一 `build_bare_native_intrinsics()`（:498-582，Plan 671 T-06 单源）；**表二 `with_type_store` 内 :896-966 是手工同步副本**（注释自认 "keep in sync"）——单源纪律的真实缺口，新内建须两处同插或顺手改为克隆 |
| File 内建 | `stdlib/auto/file.at` + `vm/ffi/stdlib.rs:337-340` | `File.read_text` 走 stdlib `#[vm]` 声明 + `rust_fn` inventory 注册（非 for_each_native catalog）；aavm 自举引擎 `auto/lib/engine.at:802-809` 有 nat#1000 硬编码臂 |
| a2r 镜像 | `crates/a2r-std/src/fs.rs:22-27` | `read_text` 注释明示镜像契约（368 consumer-mode parity） |

## 3. 裁定一：rope 路线三选一 → **(a) 窗口化包装**

| 路线 | 裁定 | 依据 |
|---|---|---|
| (a) 窗口化包装：rope 事实源 + ViEditor 只装视口行 | **选定** | 渲染契约已自有件化（428 P2），iced 不依赖 live Buffer，窗口化无渲染面阻力；rope 接管存储/摘要/delta/保存/搜索/fold 后，全文档遍历点全部 O(log n)；cosmic-text 0.15 与 iced 0.14 单实例约束（Cargo.toml:224-227）原样保留 |
| (b) 引擎替换/内 fork | 否决（本计划内） | 需复刻 shaping/syntect 高亮/hit 测试/光标几何整套，与 cosmic 生态脱钩成本高；仅当 (a) 窗口同步协议实证不可行时重启评估（债务登记） |
| (c) 上游贡献 cosmic-text | 否决 | 节奏不可控；消费方预算门是时间锁定的 |

### 3.1 (a) 线的结构裁定

```
rope（文档事实源：本仓自实现轻量 rope）
  ├─ 摘要层：节点缓存 byte_len / line_count；点-偏移换算 API O(log n)
  ├─ 快照隔离：copy-on-write 快照（后台解析/搜索/diff/保存只读，零锁竞争）
  ├─ 视口物化：cosmic Buffer 只装 [viewport_start, viewport_end) 行
  │   （ViEditor = 视口编辑器；shaping/光标几何/hit 测试保持 viewport-local）
  └─ 事件流：一切编辑写回 rope 时产 delta 入统一队列（§4）
```

- **自实现 rope，不引新依赖**：摘要需自定义指标（字节长/行数/字符点），
  ropey/crop 等现成 crate 不含点-偏移摘要；轻量实现（叶=文本块，节点缓存
  摘要）约数百行，全控且零依赖增量。
- **光标双射**：光标模型仍为 cosmic `(line, byte_index)`，但行号是视口局部
  行；rope 摘要层维护 文档行 ↔ 视口行 ↔ 字节偏移 的稳定双射，窗口平移
  （滚动/seek）保持光标锚定。`gg`/`G` 等全文档 motion 由 rope 行数翻译为
  窗口重定位。
- **undo 范围（M1 显式限制）**：击键编辑的 undo 保持 ViEditor 视口局部
  历史（现状语义的窗口化延续）；跨视口 undo 与 agent 远端写 undo 不进
  M1 范围——rope 级 delta 日志天然是文档级 undo 的素材，列为后续扩展
  （债务登记，review 时入 KNOWN-DEBT）。
- **高亮**：syntect-on-buffer-lines 在视口行上工作（现状机制不动），
  增量重高亮协议不做。
- **第二内核实例** `ui/autodown_editor/core.rs`（小块 per-leaf buffer）
  不在本设计范围。

### 3.2 复杂度契约（AC-05，SD-01 落账素材）

| 操作 | 现状 | rope 后 |
|---|---|---|
| 文本定位（字节→行/点） | O(n) 行走 | **O(log n)** |
| 区间编辑（结构化写） | O(n) 全量 set_text | **O(log n)** 定位 + O(k) 块改写 |
| 长度/行数摘要 | O(n) 遍历 | **O(1)**（根缓存） |
| delta 产出差集 | 全串 diff | 视口局部 diff O(视口) + 全量事件 O(1) 条 |
| 后台搜索/保存读出 | 持锁全文档 | 快照零锁 O(n) 只读（与主线程编辑并发） |
| 整串 `code_editor_text` | O(n) join（保留兼容） | O(n)（兼容面不优化，文档注明增量面为推荐路径） |

## 4. 裁定二：统一 delta 协议形状

### 4.1 偏移口径

**UTF-8 字节偏移**（与分块读同口径；消费方 back 面友好）。所有端点必须
是 char boundary——内核保证自产增量合法；写面入参违反时**报错不静默**
（不替你猜边界）。

### 4.2 读面：`code_editor_delta(key) -> str`（JSON）

```json
{"revision": 42, "deltas": [{"start": 128, "end": 136, "replacement": "rope"}]}
```

- **形状恒定**：无变更返回 `{"revision": N, "deltas": []}`——消费方解析面单一，
  空数组即"无操作"。
- **revision**：复用内核既有 `AtomicU64`（每次文本变更 +1，:283-285），
  即增量序号。响应携带**消费后水位**。
- **消费语义**：destructive read——调用即取走并清空自上次调用以来的队列；
  同水位重读返回空 deltas。消费方按 revision 单调校验，**跳变 = 漏窗**
  （消费方兜底：整串 `code_editor_text` 重同步；本面不提供历史重放）。
- 返回经 VM 栈槽为 JSON **字符串**（沿 `read_lines` serde_json 先例）。

### 4.3 写面：`code_editor_edit(key, start, end, replacement)` 单 API 三形态

| 形态 | 参数形态 |
|---|---|
| 插入 | `start == end && replacement 非空` |
| 删除 | `replacement == ""` |
| 替换 | 其余 |

- 与击键（ViEditor 内部编辑）、undo/redo、cut/paste **同一条 delta 队列**
  （供料 §2：人与 agent 的写在内核层无差别）。
- **击键差集计算**：视口局部 before/after 行窗 diff（编辑固有局部性）；
  undo/redo 与 set_text 允许全文档 diff（低频，文档化）。
- **set_text 增量**：整串差分简化为单条全量 delta `{start:0, end:old_len,
  replacement:new_text}`；`last_external` 防重建覆写语义不变（:269-273）。
- **external_dirty 收编**：native 驱动编辑（undo/redo/cut/paste 埋点
  :1803/:1810/:1835-1837）推 delta 队列后**保留** mark_external_dirty——
  delta 是数据面，external_dirty 是 widget 重发布信号面，双通道并存
  （防 041 式 Save 陈旧回归）。

### 4.4 注册落点（单源纪律）

`code_editor_delta` / `code_editor_edit` 走 `for_each_native!` catalog
新增 nat#（code_editor 族 2910-2933 后续段）：
`native_catalog.rs`（目录+常量生成）→ `codegen.rs` 表一 → **表二同步**
→ `vm/native.rs` shim（feature 双形态）→ `last_expr_type` 返回类型臂。
**顺手修单源缺口**：验证两表等价后，表二（:896-966）改为克隆表一。

## 5. 裁定三：back 分块读形状

### 5.1 命名与注册

`read_text_range(path, offset, limit) -> str`（JSON）。注册**沿
`File.read_text` 先例**（stdlib `#[vm]` 声明 + `rust_fn` inventory，
非 for_each_native catalog）：`stdlib/auto/file.at` + `file.vm.at` +
`vm/ffi/stdlib.rs` 实现；a2r-std `fs.rs` 镜像 `pub fn`。
**P670-D1 纪律**：两侧用同一 serde 序列化形状，JSON 逐字节一致；
对拍测试断言逐字节相等。初期仅 UTF-8（编码参数面留扩展位，不实现多编码）。

### 5.2 返回形状与边界语义

> **PLAN-687（2026-09-22）实现精化**：envelope 形状/字段序/边界回退
> 语义零变；`fs::read` 全文+切片改 **File seek+精确窗读**
> （`[offset−3, end+1)` 余量窗 + metadata 总长，IO O(limit)——多块
> 扫描不再每块全量读盘）。**流式语义收窄**：UTF-8 校验从全文一次性
> 收窄为块区（非法字节在所在块读取时报 total:-1，未读区不提前暴露）
> ——VM shim 与 a2r-std 双轨同形（P670-D1 纪律延续），parity 测试
> 零回退。另：消费侧装载主链已改 `code_editor_load_file` 端点
> （nat#9908，auto-edit PLAN-007 供料回流）——分块读保留为流式
> 消费面。

```json
{"text": "...", "total": 1234567, "next_offset": 4096}
{"text": "", "total": 1234567, "next_offset": null}   // 到达 EOF
{"text": "", "total": -1, "next_offset": null}         // 错误形状
```

| 情形 | 语义 |
|---|---|
| 正常读 | `total` = 文件字节数；`text` = 自 `offset` 起 **≤ limit 字节** 的 UTF-8（截到 char boundary，**绝不半字符**）；`next_offset` = `offset + text 字节数`；读到文件尾时 `next_offset = null` |
| `offset >= total`（空文件同） | EOF 形状：`text ""` + `next_offset null` + `total` 实际值 |
| **短读防护** | 只有 `offset + text.len() >= total` 才允许 `next_offset = null`——磁盘尾短读不会被误读为 EOF（消费方判 EOF 只看 `next_offset == null`） |
| IO 错误（文件不存在等） | 沿 `read_text` unwrap_or_default 先例：**错误形状** `total = -1`（消费方以 `total < 0` 判错） |
| `offset < 0` 或 `limit <= 0` | 参数错误 → 错误形状 `total = -1` |

### 5.3 涟漪面（显式登记）

- **aavm 自举**：`auto/lib/engine.at` 需加新 nat# 臂（沿 nat#1000 模式
  :802-809）+ `auto/lib/codegen.at` 发射臂——按 AGENTS.md §AAVM 作用域
  映射，engine.at 终段执行器改动 → `cargo taa aavm2_m5`，review/fold 前
  裸 `cargo taa` 兜底。
- **ts_adapter/vue client**：M1 **不发射**。natives.d.ts 层是 fail-fast
  声明层（vue 轨对这些内建无运行时）；消费方 M1 性能主路径 = VM/iced +
  a2r merged。vue/split 轨需要真实分块端点时另立计划（ts_adapter 改写先例
  在册）。

## 6. SD 锚点（T-00 钉死）

| SD | 锚点 |
|---|---|
| SD-01 | `docs/specs/auto-lang/ui/overview.md` code_editor 全链段（413–428 在册处）：补缓冲架构契约（rope+摘要、单写者+快照隔离、§3.2 复杂度表）+ 编辑事件契约（§4 统一 delta 流） |
| SD-02 | `docs/specs/auto-lang/runtime/overview.md`：line 22 内建函数表段落之后新增「File 内建契约（VM/a2r 双轨）」小节（现状该 spec 未细化 File 面，小节为新增而非改写） |

## 7. 债务与风险（review 时入 KNOWN-DEBT 候选）

1. (a) 线 undo 范围限制（§3.1）——视口局部 undo；rope 级文档 undo 为后续扩展。
2. `code_editor_text` 整串面保留不优化——兼容面，推荐路径为 delta。
3. 分块读 vue/split client 不发射（§5.3）——消费方该轨需要时另立计划。
4. cosmic-text `Cargo.toml:224-227` 注释的 `Weak<Buffer>` 理由已随 428 失效
   （该 handoff 已移除），注释应随本计划顺手更正。
5. 若 (a) 窗口同步协议（光标双射/窗口平移）实证不可行，重启 (b) 评估。
