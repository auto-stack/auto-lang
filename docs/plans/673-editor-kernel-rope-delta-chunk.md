---
plan_id: PLAN-673
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: editor-kernel-rope-delta-chunk
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 ui overview code_editor 缓冲/事件契约, SD-02 分块读内建契约]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/auto-lang/ui/overview.md, docs/specs/auto-lang/runtime/overview.md]
current_step: 0
total_steps: 8
---

# [PLAN-673] 编辑器内核件：rope 单写者版 + 统一 delta + back 分块读

> 来源：auto-edit 上游供料包（2026-09-21 M1 批）
> `docs/plans/attachments/673-674-m1-supply.md` §1/§2/§3（auto-edit 仓
> plan-004-dev @ 86cdfb2 起草，PLAN-004 T-01 产物）。本仓立件承接，走本仓
> plan 流程。工具链实勘基线：`v0.4.2-1631-ge82b95b22`。

## 0. 变更摘要

编辑器内核三件，解锁消费方（auto-edit）M1 性能轨后半与 M2 大文件线：

1. **内核 rope 化（单写者版）**——code_editor 文档缓冲迁 rope+摘要（O(log n)
   编辑定位），**显式不带协作基因**（无 OT/CRDT；主线程写 + 后台只读快照）。
2. **统一 delta 协议**——code_editor 编辑事件从整文本回读改区间增量；人击键
   /agent 结构化写/undo 走同一事件流。
3. **back 分块读**——`File.read_text` 之外增 offset/limit 分块内建（VM host
   + a2r-std 双轨，JSON 形状逐字节对齐——P670-D1 纪律）。

消费方预算门控（auto-edit 战略 §2.1 解锁条件列，供料 §1 动机）：100 MB 文本
打开 ≤1 s、1 GB 可打开、大文件满帧滚动、空闲内存 ≤60 MB——rope 前架构性
不可达（消费方已裁定：rope 前禁对该路径调优）。

## 1. 目标

- **G1（勘定+设计档）**：三件的落点勘定与设计裁定落 `docs/design/` 正式
  设计档（L2 纪律）——尤其 rope 路线三选一（见 §10-1）。
- **G2（统一 delta）**：`code_editor_delta(key) -> {start,end,replacement}`
  事件面（或等价订阅面）+ agent 结构化写端点；人/agent/undo 同流。
- **G3（分块读）**：分块读内建双轨落地，VM/a2r 行为对拍一致。
- **G4（rope 单写者版）**：缓冲编辑 API 复杂度文档化（编辑/定位 O(log n)），
  100 MB 冒烟达消费方预算门。
- **G5（零回归）**：code_editor_* 既有语义零回归（examples 041 与既有测试
  为验收面；消费方矩阵为下游验收）。

**非目标（Non-goals）**：

- **不做协作编辑**——无 OT/CRDT/多编辑流合并；copy-on-write 仅服务于快照
  隔离（后台解析/搜索/diff/保存只读），非多人并发（供料 §1 明确不要）。
- 不动 vue 轨生成面（PLAN-671 域）；不动 a2r codegen 词汇门与 RQ 覆盖集
  （PLAN-674 域）；不动 a2r server 模板（P670-D1 在册）。
- 不在本计划内兑现消费方全量性能数字报表（那是消费方 PLAN-004 ③④ 的
  测量体系；本计划只交付能力+冒烟门）。
- 分块读初期仅 UTF-8（编码参数面留扩展位，不实现多编码）。

## 2. 架构方案

| 件 | 现状锚点（本仓实读） | 目标形态 |
|---|---|---|
| rope | `crates/auto-lang/src/ui/code_editor/core/mod.rs:255` `CodeEditorCore` 持 `Mutex<SendEditor>`，`SendEditor(ViEditor<'static,'static>)`（:245）；引擎 = **cosmic-text 0.15 `vi`+`syntect` features**（`crates/auto-lang/Cargo.toml:227`）——文档缓冲实际存于 cosmic-text `Buffer`（行数组形态），**非本仓自有结构** | 路线三选一（§10-1，T-00 裁定）：(a) 窗口化包装——rope 为事实源、ViEditor 只装视口行；(b) 引擎替换/内 fork；(c) 上游贡献 cosmic-text。摘要（长度/行数/点-偏移）随绳缓存 |
| delta | `code_editor_text(key)` 内建整串回读（`vm/codegen.rs:527/:897/:9525`，Plan 413）；`code_editor_set_text`（core/mod.rs:1762，`last_external` 差分防视图重建覆写）；`revision: AtomicU64`（:283）已在位可复用为增量序号 | 增量事件面（读侧 `code_editor_delta`）+ 结构化写端点（区间替换/插入/删除）；与既有 on_change/on_cursor（`ui/view.rs:633` View::CodeEditor 载荷）同源同流 |
| 分块读 | VM 轨 `File.read_text` → nat#1000 宿主内建（`auto/lib/engine.at:802-808`、`auto/lib/codegen.at:1957-1958`）；a2r 轨 `crates/a2r-std/src/fs.rs:25` 镜像 | `read_text_range(path, offset, limit)`（命名 T-00 定）双轨落地；宿主/a2r-std/（消费方 back 契约面）三处形状一致 |

**单写者纪律**（全计划适用）：主线程（UI/编辑路径）唯一写者；后台任务经
copy-on-write 快照只读。Mutex/原子量形态沿用 CodeEditorCore 现状（iced
单 UI 线程 + MCP 序列化访问，mod.rs:251-254 既有注释）。

**阶段序**：delta（协议面小、消费方立即受益）→ 分块读（小件随行）→
rope（大头，设计门控后动）。T-00 勘定设计档先行。

## 3. 技术栈

Rust（crates/auto-lang ui/code_editor + vm/codegen 内建面 + crates/a2r-std）；
cosmic-text（现状依赖，去留随 §10-1 裁定）；.at 语料 fixture（tests/）；
大文件冒烟语料（程序化生成，不入库）；对拍纪律参照 P670-D1（VM/a2r JSON
形状逐字节一致）。

## 4. 需求分析与背景调查

**授权记录**：2026-09-21 用户指令（auto-edit 会话任务②）——供料包
`docs/upstream/2026-09-m1-supply.md` 发往本仓立上游计划，建议拆两件
（内核件=本计划；生成器件=PLAN-674）。授权范围 = **立项起草**（drafting）；
执行（work）另行授权。

**消费方证据**（供料包 §1/§2/§3，原件在附件）：

- rope 动机：auto-edit 战略 §2.1 预算列（100 MB ≤1s / 1 GB 可打开〔分块/
  mmap〕/ 满帧滚动 / 空闲 ≤60 MB）；消费方已裁定 rope 前禁对该路径调优。
- delta 现状证据：auto-edit `specs/auto-edit/src/front/editor_store.at` 五处
  以 `code_editor_text(key)` 全量回读维护 `.tabs[i].src` 字符串镜像
  （CtxCut/ActUndo/ActRedo/ActCut/ActPaste，约 L229/350/359/376/391）——
  每击键级操作复制全文，大文件结构性死罪；消费方 store 去文本化（其
  PLAN-005 候选）以本计划 delta 面为前置。
- 分块读现状证据：auto-edit `specs/auto-edit/src/back/api.at` `read_text`
  整串直通；vue/split 轨 ts_adapter 生成 client 同为整串。

**本仓代码实读**（2026-09-21，master@20f79de62）：

- `ui/code_editor/core/mod.rs`（2754 行）：结构见 §2 表；关键既有件——
  `last_external` 差分（:269-273）、`revision`（:283-285）、`external_dirty`
  （:286-291，菜单/工具栏 undo/cut/paste 类编辑的事件重发布通道——delta
  协议须收编此通道，防「native 驱动编辑不产增量」盲区）、folds/fold_map
  （:296-302，视图态）、caret_follow（:303-305）。
- `ui/view.rs:633` View::CodeEditor 载荷（on_change/on_cursor/
  on_context_menu/search…）；:2347 map 转换臂。
- `vm/codegen.rs` intrinsics 两表（:527 与 :897 平名注册，671 T-06 已立
  `build_bare_native_intrinsics` 单源先例——新内建沿用单源纪律）。
- a2r-std `fs.rs:25` `read_text` 注释明示镜像契约（`auto.fs.read_text`/
  `auto.file.read_text`）——分块读沿用同契约形态。

**先例与交集**：413–428 code_editor 全链（ui/overview.md:353 在册）；
Plan 428 折叠/fold_map（视图态边界先例——快照隔离同理）；669 §10-4
（消费方交接模式）；671 T-06（内建单源注册表先例）；P670-D1（双轨
JSON 形状逐字节对齐纪律）。

## 5. 详细设计

### 5.1 统一 delta 协议

- **读侧**：`code_editor_delta(key)` 增量面——返回自上次消费以来的区间
  增量 `{start, end, replacement}`（字节或字符偏移口径 T-00 定，倾向
  UTF-8 字节偏移与分块读同口径）；以 `revision`（既有 AtomicU64）为增量
  序号，消费方按序号续读。无变更返回空/无操作。
- **写侧**：结构化写端点（区间替换/插入/删除三形态，单 API 收敛）——
  人击键（ViEditor 内部编辑）、agent 结构化写、undo/redo（含
  `external_dirty` 通道的 native 驱动编辑）**同一条增量事件流**（供料 §2
  期望形态：人与 agent 的写在内核层无差别）。
- **兼容**：`code_editor_text` 整串回读保留（既有消费方零破坏），文档
  注明增量面为推荐路径。

### 5.2 back 分块读

- 内建 `read_text_range(path, offset, limit) -> {text, total, next_offset}`
  （返回形状 T-00 定型；EOF 判据明确）；VM host（nat# 注册沿单源表）+
  a2r-std 镜像 + （如契约面需要）ts_adapter client 发射三处一致。
- 初期仅 UTF-8；offset 单位与 delta 口径统一。越界/短读语义显式（不静默
  截断成成功空串——防消费方把短读当 EOF）。

### 5.3 内核 rope 化（单写者版，设计门控）

- 路线三选一（§10-1）：T-00 设计档裁定，裁定前不写实现代码。
- 无论何线，交付物含：**摘要层**（根缓存总长/总行数/点-偏移换算，
  O(log n) 定位 API 文档化）；**快照隔离**（copy-on-write 或等价，后台
  只读任务零锁竞争）；**视口物化**（布局/渲染只触可见区间——与 428
  fold_map 的视图态边界同纪律）。
- 冒烟门：100 MB 程序化语料打开+滚动（预算数字以 T-00 勘定后的本机基线
  为准记录，不预设通过线——通过线归消费方 ④ 计划定标）。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md（code_editor 段） | before：code_editor 全链描述（413–428 在册）无缓冲复杂度契约与事件增量契约 / after：缓冲架构契约（rope+摘要、单写者+快照隔离、复杂度表）+ 编辑事件契约（统一 delta 流：人/agent/undo 同流、revision 序号、整串面兼容注记） | 内核件是消费方 M2 大文件线的架构前置，契约须进 spec 防散记失联 | AC-02/03/06 |
| SD-02 | add | docs/specs/auto-lang/runtime/overview.md（File 内建面；具体锚点 T-00 钉死） | before：仅整串 read_text / after：分块读内建契约（形状/EOF/越界语义/双轨一致纪律） | 新内建契约落账；P670-D1 对齐纪律引用 | AC-04/06 |

## 6. 测试设计

- **delta**：fixture（.at 语料）断言增量序列——击键/结构化写/undo 三来源
  各产正确区间增量；连续消费按 revision 续读；无变更空返回。
- **分块读**：VM/a2r 双轨对拍（同文件同参数 JSON 逐字节一致）；EOF/越界/
  短读/空文件四边界 fixture。
- **rope**：100 MB 冒烟（生成语料，记录打开/滚动/保存耗时与峰值内存——
  收据入计划，不设通过线）；既有 code_editor 单测全绿。
- **回归**：`cargo t`（日常档）；改动触 VM/编译器面按 AGENTS.md §AAVM
  作用域映射加档；examples 041-auto-edit VM 轨冒烟。
- **通用性静态门**：修复面 grep 零消费方特定标识符（auto-edit/
  editor_store/tabs 等名单）。

## 7. 验收标准

- **AC-01（勘定+设计档）**：T-00 设计档在 `docs/design/` 落盘并注册
  `00-intro.md`；rope 路线三选一有裁定与依据；delta/分块读形状定案。
- **AC-02（统一 delta）**：`code_editor_delta` 读面 + 结构化写端点落地；
  击键/agent 写/undo（含 external_dirty 通道）三来源同流，fixture 断言
  增量序列正确；`code_editor_text` 兼容保留。
- **AC-03（快照隔离）**：后台只读任务（解析/搜索类）与主线程编辑并发
  形态下无锁竞争死等/无数据竞争（fixture 或测试证明快照语义）。
- **AC-04（分块读双轨）**：VM host + a2r-std 落地，双轨对拍逐字节一致，
  四边界 fixture 绿。
- **AC-05（rope 冒烟+复杂度）**：编辑/定位 API 复杂度文档化（O(log n)）
  落 spec（SD-01）；100 MB 冒烟收据（耗时/内存实测记录）入计划。
- **AC-06（零回归+落账）**：cargo t 绿（预存红在案除外）；examples 041
  冒烟绿；grep 门过；SD-01/SD-02 落账；消费方复跑通知发出（669 §10-4
  模式）。

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 意图 | AC | 验证命令/预期 |
|---|---|---|---|---|---|---|
| T-00 | 勘定+设计档：rope 路线三选一裁定（cosmic-text 依赖去留）、delta/分块读形状定案、SD-02 spec 锚点钉死；落 `docs/design/<NN>-editor-kernel.md` 并注册 00-intro | — | docs/design/ 新档 + 本计划回填裁定 | L2 设计先行 | AC-01 | 设计档在册 + §10 三问闭环 |
| T-01 | delta 读侧：`code_editor_delta` 内建（单源注册表接入）+ revision 续读语义 | T-00 | vm/codegen.rs 内建面 + core/mod.rs 增量队列 | 读面 | AC-02 | fixture：增量序列断言 |
| T-02 | delta 写侧+同流：结构化写端点（区间替换/插入/删除）+ external_dirty 通道收编进增量流 | T-01 | core/mod.rs 写路径 + set_text 侧 | 写面+同流 | AC-02 | fixture：三来源同流断言 |
| T-03 | 分块读内建：VM host `read_text_range` + a2r-std 镜像 + ts_adapter client 面（按 T-00 定案） | T-00 | vm/codegen.rs + a2r-std/src/fs.rs + ts_adapter.rs | 双轨内建 | AC-04 | 双轨对拍逐字节一致 + 四边界 fixture |
| T-04 | rope 实现批一：摘要层（长度/行数/点-偏移）+ 定位 API + 快照隔离 | T-00 | core/ 新模块（路线随裁定） | 内核地基 | AC-03/05 | 单测：定位/快照语义 |
| T-05 | rope 实现批二：视口物化 + 全链接线（set_text/编辑路径/保存读出）+ 100 MB 冒烟收据 | T-04 | core/ + iced 适配层 | 全链 | AC-05 | 冒烟收据入计划 §9 |
| T-06 | 回归+grep 门：cargo t / 041 冒烟 / 通用性 grep | T-01..T-05 | — | 零回归 | AC-06 | cargo t 绿 + grep 零命中 |
| T-07 | spec 落账+交接：SD-01/SD-02 落盘 + 消费方复跑通知（669 §10-4 模式） | T-06 | docs/specs/ + §9 记录 | 收口 | AC-06 | spec 回读 + 通知留档 |

## 9. 复审记录

- 2026-09-21 · stage: new · r1 起草 · 授权=用户指令（auto-edit 供料包发车
  拆两件，本件=内核件；供料原件附件 `673-674-m1-supply.md` §1/§2/§3）·
  outcome: **pass（待执行授权）** · next: **work**（授权后自 T-00 起；
  §10-1 rope 路线为开工前最大待裁项）。

## 10. 待澄清事项

1. **rope 路线三选一**（阻塞 T-04/T-05，T-00 裁定）：(a) 窗口化包装——
   rope 事实源 + ViEditor 只装视口行（改动面最小，双引擎并存的复杂度）；
   (b) 引擎替换/内 fork（自由度最高，工程量与 cosmic-text 生态脱钩成本）；
   (c) 上游贡献 cosmic-text（最正统，节奏不可控）。供料不预设；T-00 勘定
   Buffer 实际瓶颈（set_text 全量 / 布局重排 / 保存读出）后裁定。
2. **delta 偏移口径**（T-00 定）：UTF-8 字节偏移（与分块读同口径、消费方
   back 面友好）vs 字符/行-列偏移（编辑器语义友好）。倾向字节偏移。
3. **分块读返回形状**（T-00 定）：`{text,total,next_offset}` 三件套 vs
   简化两件；EOF 判据（next_offset==null vs total 比较）。
4. **预算数字归属**：100 MB ≤1s / 1 GB / ≤60 MB 为消费方产品预算；本仓
   冒烟只记实测收据不设通过线（通过线归消费方测量计划定标）——上游如需
   自设门，review 时定。
5. **消费方复跑时点**（不可控）：本计划 G5 零回归为上游侧验收；消费方
   矩阵复跑按 669 §10-4 模式 merge 后通知，不阻塞本计划终态。
