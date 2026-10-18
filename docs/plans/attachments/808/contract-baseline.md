# PLAN-808 T-01: 终端宿主域旧臂冻结基线

2026-10-18。本文件是 PLAN-808 的 T-01 交付物：在实施前冻结 34 项 terminal
native、旧引擎 glue 的全部可观察语义、所有权/锁序/线程/feature 矩阵、配置
stub、真实 DLL 身份、旧 oracle 与可重放 driver 入口。后续 T-02..T-07 以本
文件为合同；任何必要偏移必须在本文件登记并给出理由，不能"迁移中顺手修正"。

## 0. 身份与取材

| 项 | 值 |
|---|---|
| 计划/修订 | PLAN-808 revision 1（`status: drafting` 进入 work） |
| 主检出 base | `2046a025b9a445d35a02da2c72a1613259a0cb60`（master） |
| 实施树 | `D:/autostack/.wt/lang-808/auto-lang` |
| 分支 | `codex/plan-808-dev` |
| 工具链 | cargo 1.98.0 / rustc 1.98.0（2026-08-18） |
| 真实 DLL | `D:/autostack/auto-term/target/debug/autoterm_core.dll` |
| DLL SHA256 | `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`（1,615,360 B） |
| auto-term 源 | branch `main` @ `7a0d0e8b718fb260641f0d09ada95a7346771cd3`，working tree clean |
| 构建命令 | `SCCACHE_DISABLE=1 RUSTC_WRAPPER= cargo build -p autoterm-core`（auto-term 主检出；仅写 ignored target，不改源码） |
| 有界探针 | [t01-dll-probe.py](t01-dll-probe.py)，输出 [t01-dll-probe.log.txt](t01-dll-probe.log.txt) |
| 旧 glue 锚点 | `crates/auto-lang/src/vm/ffi/term_engine.rs`（1394 行，49 处 `crate::ui::`） |
| catalog 锚点 | `crates/auto-lang/src/vm/native_catalog.rs` 530–596（binding）、1177–1210（签名） |
| UI 状态锚点 | `crates/auto-lang/src/ui/terminal/mod.rs`（1993 行，terminal 注册表） |
| 视图锚点 | `ui/terminal/iced/widget.rs`（2327 行）、`ui/aura_view_builder.rs:13189`（VM 视图）、`ui_gen/rust.rs:4431`（Rust 生成） |

真实 DLL 探针结论（22/22 导出在位、空句柄哨兵、双会话隔离、自然退出、kill
后退出、free 单次、palette 纯查询）：`verdict: pass`，详见 §7。

## 1. 34 项 native 全矩阵（ID/canonical/签名/旧实现/新 owner）

弹序记法：`.at` 调用参数按左→右压栈，shim 右→左弹（`←` 依次）。所有 ID、
canonical 名、栈形状、返回类型 **不变**；catalog/codegen 无需改。新实现列 =
`auto-terminal-core` 服务方法 / `TerminalHostProvider` 端口方法。

| ID | canonical | 签名（弹序） | 返回 | 旧实现（term_engine.rs） | 新 owner/方法 | feature |
|---|---|---|---|---|---|---|
| 2943 | `auto.term.engine_spawn` | (cols int, rows int) ←rows, ←cols | Int 句柄 | `engine_spawn`：DLL `spawn(cols,rows,NULL)`；失败=NULL→0；成功后 NEXT_HANDLE++、VIEWPORT=(cols,rows)、CURSOR=(0,0) | service `spawn` | 全 |
| 2944 | `auto.term.engine_write_line` | (handle int, line str) ←line, ←handle | Void | `engine_write_line`：行尾补 `\r\n` 后 `write_input`；无效句柄/DLL 缺席 no-op | service `write_line` | 全 |
| 2945 | `auto.term.engine_rows` | (handle int) ←handle | List | `engine_rows`=`engine_rows_inner(All)`：feed→脏行→cursor 采样→逐行 text+style→**广播**全部 UI terminal 样式→快照文本 | provider `rows`（service text；UI adapter 广播） | 全（UI 样式仅 ui） |
| 2946 | `auto.term.engine_resize` | (handle int, cols int, rows int) ←rows, ←cols, ←handle | Void | `engine_resize`：DLL resize；**不更新 VIEWPORT 缓存**（只有 apply_resize 更新） | service `resize` | 全 |
| 2947 | `auto.term.engine_interrupt` | (handle int) ←handle | Int | `engine_interrupt`：DLL 返回 1/0；DLL 缺席/无效句柄 -1 | service `interrupt` | 全 |
| 2948 | `auto.term.engine_is_exited` | (handle int) ←handle | Bool | `engine_is_exited`：DLL==1；DLL 缺席/无效句柄 **true** | service `is_exited` | 全 |
| 2949 | `auto.term.engine_free` | (handle int) ←handle | Void | `engine_free`：DLL free→HANDLES/SNAPSHOTS 删除；CURSORS/VIEWPORTS **保留旧值**；重复 free no-op | service `free` | 全 |
| 2957 | `auto.term.engine_pump_input` | (handle int) ←handle | Int | `engine_pump_input`：`terminal_drain_all_inputs()` 全量排空后逐键裸写；DLL 缺席/无效仍返回键数 | provider `pump_input`（service `write_keys`） | ui 有输入；no-ui 返 0（队列空） |
| 2958 | `auto.term.engine_cursor_row` | (handle int) ←handle | Int | `geom_of(CURSORS).0`（feed 时采样；缺省 0） | provider `cursor_row`（service cache） | 全 |
| 2959 | `auto.term.engine_cursor_col` | (handle int) ←handle | Int | `geom_of(CURSORS).1` | provider `cursor_col` | 全 |
| 2976 | `auto.term.engine_apply_resize` | (handle int) ←handle | Int | `take_pending_resize()`（任意端）→resize→VIEWPORT 更新；无请求 0 | provider `apply_resize` | ui；no-ui 恒 0 |
| 2977 | `auto.term.engine_viewport_cols` | (handle int) ←handle | Int | `geom_of(VIEWPORTS).0`（spawn 初值，apply_resize 更新） | provider `viewport_cols` | 全 |
| 2978 | `auto.term.engine_viewport_rows` | (handle int) ←handle | Int | `geom_of(VIEWPORTS).1` | provider `viewport_rows` | 全 |
| 2979 | `auto.term.engine_backlog_pending_mb` | (handle int) ←handle | Int | DLL `pending_bytes(h).max(0)/1MB`；缺席/无效 0 | service `backlog_pending_mb` | 全 |
| 2980 | `auto.term.engine_backlog_paused` | (handle int) ←handle | Int | 符号缺失→0；否则 DLL 值；缺席/无效 0 | service `backlog_paused` | 全 |
| 2981 | `auto.term.engine_backlog_take_alerts` | (handle int) ←handle | Int | 进程级**全局**迟滞态：≥2MB 且未 active→1+stderr；≤512KB 解除；否则 0 | service `backlog_take_alerts` | 全 |
| 2982 | `auto.term.engine_backlog_dropped` | (handle int) ←handle | Int | 符号缺失→0；否则 `overflow_count(h).max(0)` | service `backlog_dropped` | 全 |
| 2983 | `auto.term.engine_spawn_ex` | (program str, argv []str, cwd str, cols int, rows int) ←rows, ←cols, ←cwd, ←argv, ←program | Int | `engine_spawn_ex`：CString 化；cwd 空=NULL；DLL 无 `spawn_ex` 符号→0；失败→0；成功同 spawn 缓存 | service `spawn_ex` | 全 |
| 2984 | `auto.term.engine_rows_for` | (handle int, key str) ←key, ←handle | List | `engine_rows_inner(Key)`：样式/窗口只投喂该 key；文本快照同 rows | provider `rows_for`（UI 定向；no-ui 同 rows） | ui 定向；no-ui 文本 |
| 2985 | `auto.term.engine_pump_for` | (handle int, key str) ←key, ←handle | Int | `terminal_core(key)`→`drain_inputs_for`→逐键裸写；缺 key/无 ui → 0 | provider `pump_for` | ui；no-ui 0 |
| 2986 | `auto.term.engine_apply_resize_for` | (handle int, key str) ←key, ←handle | Int | `terminal_take_resize_for(core)`→resize→VIEWPORT；无请求 0 | provider `apply_resize_for` | ui；no-ui 0 |
| 2987 | `auto.term.engine_menu_take` | () | Int | `terminal_take_menu_item_any()`：BTreeMap 键序首个 Some；无→-1 | provider `menu_take` | ui；no-ui -1 |
| 2988 | `auto.term.config_profiles` | () | List | 恒空集 | core 常量 | 全 |
| 2989 | `auto.term.config_default_profile` | () | Str | 恒空串 | core 常量 | 全 |
| 2990 | `auto.term.config_profiles_full` | () | List | 恒空集 | core 常量 | 全 |
| 2991 | `auto.term.config_spawn_program` | (profile str) ←profile | Str | 弹参后恒空串 | core 常量 | 全 |
| 2992 | `auto.term.config_spawn_argv` | (profile str) ←profile | List | 弹参后恒空集 | core 常量 | 全 |
| 2993 | `auto.term.config_spawn_cwd` | (profile str) ←profile | Str | 弹参后恒空串 | core 常量 | 全 |
| 2998 | `auto.term.window_width` | () | Int | ui：thread-local APP_WINDOW_PX.w≥1→w，否则 `theme::window_width()`；no-ui 1024 | provider `window_width` | ui/ no-ui |
| 2999 | `auto.term.window_height` | () | Int | 同上 h；no-ui 768 | provider `window_height` | ui/ no-ui |
| 3000 | `auto.term.engine_write_raw` | (handle int, vt str) ←vt, ←handle | Void | `engine_write_raw`：VT 原样裸写（无补缀）；无效/缺席 no-op | service `write_raw` | 全 |
| 3001 | `auto.term.engine_scroll_pending_for` | (handle int, key str) ←key, ←handle | Int | `terminal_scroll_delta_pending(core)`；缺 key/无 ui 0 | provider `scroll_pending_for` | ui；no-ui 0 |
| 3002 | `auto.term.engine_pend_resize_geom` | (key str, geom str) ←geom, ←key | Int | 解析 `"CxR"`；cols<2 / rows<1 / cols>500 / rows>200 →0；`terminal_pend_resize`（惰性建核+clamp+同值 no-op）→1/0 | provider `pend_resize_geom` | ui；no-ui 0 |
| 3003 | `auto.term.engine_history` | (handle int) ←handle | Int | DLL `history(h).max(0)`；缺席/无效 0 | service `history` | 全 |

计数核对：2943–2949=7，2957–2959=3，2976–2982=7，2983–2987=5，
2988–2993=6，2998–3003=6 → **34**。catalog 34 条与 shim 34 个一一对应，
无静默差额。

## 2. DLL 符号契约（必须/可选/未绑定）

真实 DLL 导出 22 个符号；旧 glue 使用其中 19 个。冻结分类：

| 类 | 符号 | 旧行为 |
|---|---|---|
| **必须**（`.expect`） | `spawn`, `write_input`, `feed_ready`, `take_dirty_rows`, `cursor`, `row_text`, `row_style`, `scroll`, `scroll_offset`, `history`, `resize`, `interrupt`, `is_exited`, `free`, `pending_bytes` | 符号缺失 = 调用线程 panic（不是返回值） |
| **可选** | `spawn_ex`（缺失→0）、`backlog_paused`（缺失→0）、`overflow_count`（缺失→0）、`palette_color`（缺失→内置表） | 静默降级 |
| **未绑定**（auto-lang 原支持面之外） | `kill`, `set_palette`, `shell_pid` | 34 native 不暴露；不新增绑定（T-02 不扩大支持面） |

`kill` 仅被 a2c/a2r C-ABI 测试直接消费（`test/a2c/18_c_interop/004_engine_face`），
不经 auto-lang native；本计划不改这些语料。

### 2.1 真实 DLL 探针（T-01 bounded probe）

[t01-dll-probe.py](t01-dll-probe.py) 以 ctypes 直接消费 DLL（不经 auto-lang），
覆盖**身份/导出/空句柄哨兵/双会话/退出/free**，`verdict: pass`：

- 22/22 导出在位；`palette_color(0,0,0)=0xE8E8E8`，非法 scheme=`0xFFFF_FFFF`。
- 空句柄（NULL ptr）：`is_exited/interrupt/pending/paused/overflow=-1`，
  `history/offset=0`，`cursor/row_text/feed_ready=-1`，`free/kill` no-op。
- Session A：`spawn_ex cmd.exe /C echo PLAN808_PROBE_A` → 行文本含 marker、
  `is_exited=1`（自然退出）、`history=0`、`offset=0`、`cursor ret=1 (1,0)`、
  `resize(100,30)` 无崩溃、`free` 单次。
- Session B：交互 `cmd /K` → `write_input` 后 marker 可见 → `resize(100,30)`
  行数≥24 → `interrupt=0`（字节降级路径）→ `scroll ±3` offset 非负 →
  `kill` 后 `is_exited=1` → `free`。
- Session C：`cmd /C echo PLAN808_PROBE_C`，句柄与 B 不同；B 行不含 C marker，
  C 行不含 A marker（**双会话不串线**）→ `free`。
- 结论：DLL 指针寿命 = spawn→free；free 后不得再触同一指针；旧 glue 的
  `ptr_of` 表删除语义与 DLL 契约一致。provider 替换不得让旧句柄路由到
  另一个 service（R1 起由进程全局句柄命名空间 + per-VM owner 历史保证，
  见 §3.1/§5）。

## 3. 所有权、锁序、线程与 feature 矩阵

### 3.1 所有权

| 状态 | 旧 owner | 新 owner |
|---|---|---|
| `Library`/DLL 符号查询 | `term_engine::LIB`（OnceLock） | `auto-terminal-core::EngineService`（每实例 OnceLock；Library 私有） |
| NEXT_HANDLE（i64 命名空间） | 进程静态 | **进程静态**（core 全局 atomic；两隔离 service 不重号，R1） |
| HANDLES（i64→ptr） | 进程静态 | service 实例会话表（每会话 = 私有指针 + 操作门 + 退役位，R1） |
| provider 历史（handle→owner 路由） | 无（旧 glue 进程静态唯一） | per-`NativeInterface` 的 `terminal_owners`（强 Arc 历史；按 `owns_handle` 反查，R1） |
| SNAPSHOTS（handle→行文本） | 进程静态 | service 实例 |
| CURSORS / VIEWPORTS | 进程静态 | service 实例（free 后**保留旧值**，冻结旧臂） |
| BACKLOG_STATE（全局迟滞） | 进程静态 | service 实例（默认共享 service = 进程全局） |
| UI `TERMINALS` / 输入队列 / pending resize / scroll delta / menu 载荷 / window_store / 选中/光标状态 | `ui::terminal` | **仍在 UI**（adapter 消费，不搬 UI 注册表） |
| `APP_WINDOW_PX` thread-local | `term_engine` | 具名 UI view adapter（`ui::terminal_view` 或 `ui::terminal_provider`）；`term_engine::set_app_window_px` 保留兼容门面 |
| palette 内置表/缓存 | `ui::terminal` | **仍在 UI**（`terminal_effective_palette` 不变）；引擎 palette 查询经 service `palette_color` |
| FFI cell 颜色位域解码 | `term_engine::decode_style_color`（ui） | **一份**：core `decode_style_color`；UI 侧仅结构转换 |
| 行显示宽度 `char_width`/`row_width` | `ui::terminal` | **仍在 UI**（纯显示规则；本计划不搬 renderer） |

**默认共享 vs 隔离**：`EngineService::shared()` 是进程单例，作为无 provider
时的默认回落；UI adapter 持有同一 `Arc<EngineService>`，因此 UI 安装前后
spawn 的句柄仍由同一会话表解析（旧跨调用/共享行为保留）。`EngineService::new()`
可创建隔离实例（测试/两 VM 隔离注入）。**句柄命名空间是进程全局的**，两
隔离 service 不会发出同号句柄；VM 保留曾安装 provider 的历史，带 handle
的 shim 先按 `owns_handle` 反查原 owner——provider 替换后旧句柄仍路由到
原 service，不会误释放新 service 的会话（R1-F01）。旧 owner 在最后一个
会话释放后被剪枝；其末引用 Drop 时释放剩余会话（R1-F04）。

### 3.2 锁序（实施必须遵守）

1. `service.lib` OnceLock 初始化 → 不持其他 service 锁。
2. 会话表锁只做查询/插入/摘除，**不跨会话门/FFI/UI**；每个会话有独立
   操作门 + 退役位（R1-F03）。
3. `free`：表锁内摘除 → 释放表锁 → 取会话门（等待在途操作）→ 置退役 →
   调 DLL free **恰好一次**。在途/后继 lease 在门内复查退役位，绝不使用
   已释放指针。
4. `snapshots`/`geoms`/`cursor`/`backlog` 各自短临界区；互不嵌套。
5. UI adapter 顺序固定：**先 service 调用（锁已释放）→ 再 UI 注册表锁**。
   UI 代码（widget/dynamic）从不回调 service；无反向锁序。
6. **NI owner 锁纪律（R2）**：`terminal_owners` 锁内只做 Arc 列表
   快照/摘除/合并，**不得**调用 `owns_handle`/`live_handle_count`/
   `cached_cursor_viewport` 或析构 provider。查询锁外执行并以 generation
   变化重试；剪枝锁外求值/析构并把并发新增合并回；槽位替换锁内只换值、
   旧 provider 锁外析构。四类重入与并发 set/query/prune 有永久控制。
7. 不持 service/registry 锁回调 UI 或 DLL（防重入锁死）——旧 glue 已有的
   纪律原样保留。
8. DLL 内部每柄锁（auto-term `engine_lock`）与 service 门无交叉持有；
   service 门保护的是 DLL 锁不覆盖的 free 生命周期窗口。

### 3.3 线程亲和

- provider 实现 `Send + Sync`；service 方法可从任意线程调用；DLL 每柄串行。
- UI `TERMINALS`/FOCUS 等均为 Mutex/原子；iced widget 与 VM shim 可跨线程。
- `APP_WINDOW_PX` 保持 **thread-local**（旧语义；renderer 线程写、VM 读）；
  本计划不改同步调用为异步排队。
- 真实 DLL 可能持 PTY/子进程/读线程；`free` 语义以 DLL 为准（T-06 观测）。

### 3.4 Feature 矩阵

| 构建 | terminal 行为 |
|---|---|
| `--features ui`（含 ui-iced 宿主） | 安装 UI adapter：输入泵/定向/几何/menu/scroll/window/palette/view 投影全部在位；`rows` 广播样式，`rows_for` 定向 |
| 无 `ui` | 仅 core 默认 provider：spawn/write/rows(文本)/resize/interrupt/free/backlog/history 在位；pump/apply_resize/menu/pend_resize/scroll_pending 值降级（0/-1）；window 1024×768；config 空 |
| `ui-iced` 独立 | widget 编译面不变；provider 不因 ui-iced 增加 |
| `auto-terminal-core` | 无 feature；normal/build/dev 相同；Cargo 闭包只含 std + libloading |

### 3.5 配置六项（冻结）

`config_profiles`→`[]`；`config_default_profile`→`""`；
`config_profiles_full`→`[]`；`config_spawn_program`/`config_spawn_argv`/
`config_spawn_cwd` 弹参后→`""`/`[]`/`""`。**不新增 autoterm-config 依赖、
不实现配置功能**（旧 PLAN-025 B 部 stub 语义）。

## 4. 旧可观察 oracle（逐项）

### 4.1 缺 DLL / 缺符号

| 调用 | 结果 |
|---|---|
| `spawn`/`spawn_ex` | 0（spawn_ex 另有"符号缺失=0"） |
| `write_line`/`write_raw`/`resize`/`free` | no-op |
| `pump_input`/`pump_for` | 返回排空键数（队列已排空丢弃，不回灌） |
| `rows`/`rows_for` | `[]` |
| `interrupt` | -1 |
| `is_exited` | true |
| `cursor_row/col`、`viewport_cols/rows`、`history`、`backlog_*` | 缓存/0（见 §1） |
| `apply_resize`/`apply_resize_for`/`pend_resize_geom`/`scroll_pending_for`/`menu_take` | 0/0/0/0/-1 |
| `window_width/height` | 1024/768（no-ui）；ui 读 theme/override |
| palette | 内置回退表（UI） |

### 4.2 句柄生命期

- spawn 失败（DLL spawn 返回 NULL / spawn_ex 符号缺失 / CString 失败）→0，
  不分配句柄、不写 CURSORS/VIEWPORTS。
- 成功：**进程全局** NEXT_HANDLE 单调递增（旧 glue 单计数器语义；两隔离
  service 不重号）；VIEWPORT=(cols,rows)、CURSOR=(0,0)。
- `free`：会话表内摘除 → 等会话门（在途操作结束）→ 置退役 → DLL free
  **恰好一次** → SNAPSHOTS 删除；CURSORS/VIEWPORTS 保留旧值（旧臂；单调
  全局 ID 不复用，不会串到新会话）。
- 重复/并发 free：只有摘除成功者获得 free token；其余 no-op（不触 DLL、
  不误删新会话）。
- free 后读：`rows=[]`；`is_exited=true`；`cursor/viewport` 返回旧缓存值；
  `history/backlog=0`；等待中的 lease 看到退役位后返回缺席值。
- provider 替换：句柄按 VM 记录的 provider 历史反查原 owner；旧 owner
  保留到最后一个会话释放（R1-F01/F04）。
- **free 后缓存保持（R2-F02）**：free 经 VM shim 时，owner 被剪枝/销毁前
  把 cursor/viewport 值复制进 per-VM retired 表；缓存查询序 = retired 表
  → 活 owner → 仍持有 retired 缓存的旧 owner → 当前 provider。复制纯值，
  不持旧 service、不触已释放 DLL 指针；两种 replace/free 顺序、重复 free、
  未知句柄均按冻结语义（未知=0）。
- service 末引用 Drop：在 Library 析构前逐会话 free（R1-F04）。
- interrupt 不 free；is_exited 不 free；测试/driver 必须在截止后 kill+free。

### 4.3 采样与滚动

- `rows` 每拍：feed_ready → (Key 臂先排滚动队列) → take_dirty_rows(cap64，
  忽略输出) → cursor 采样（返回 1 才更新）→ 逐行 `row_text`（cap 512，
  `need<0` 停）→ `row_style`（cap 1024，返回 -1 视为 0 样式）→ 快照文本替换。
- 文本解码：UTF-8 lossy，长度 = min(need-1, 511)。
- 样式位域：`kind=v>>24, value=v&0x00FF_FFFF`；1→Indexed(u8)，2→Rgb，
  其他→Default。**算法只在 core 一份**。
- scroll 队列：`delta!=0` → DLL scroll(delta) → 读 display_offset/history →
  UI `set_scroll_offset/set_history/set_window_anchor(hist-offset)`；
  delta≠0 时 `terminal_mark_view_bound`；随后预取窗 N=24（上/下，
  `scroll(+n)` 采样后 `scroll(-n)` 恢复；`feed_window_for` 绝对 id）。
- display_offset 唯一真源 = DLL；UI 仅投影（Mux §V1.10 不变）。
- 输入：`write_line` 补 `\r\n`；`write_raw` 原样；pump 逐键裸写（无补缀）。

## 5. 新架构落点（T-02..T-05 施工合同）

```
crates/auto-terminal-core/          # 新 crate；std + libloading
  src/lib.rs                         # re-export
  src/types.rs                       # TerminalColor/TerminalCell/Geometry/Cursor/Damage/BacklogAlert
  src/engine.rs                      # EngineService：LIB/符号/handles/snapshots/cursor/viewport/backlog
  src/provider.rs                    # TerminalHostProvider (34 方法, owned 值域)
  src/consumer_probe.rs?             # 轻 consumer 测试辅助（可选）

crates/auto-lang/src/
  vm/host_capabilities.rs            # terminal provider 进程默认槽（779 模式）+ re-export core 端口类型
  vm/native.rs                       # NativeInterface terminal 槽（Clone 保留）
  vm/engine.rs                       # AutoVM 构造期快照 + 实例 set/get
  vm/ffi/term_engine.rs              # 仅 shim：34 弹参/push；无 DLL/无 UI 业务
  ui/terminal_provider.rs            # UI adapter：输入/几何/menu/scroll/window/palette/view 投影
  ui/terminal/mod.rs                 # 保留注册表/视图状态；TermColor/TermCell 从 core re-export
  lib.rs                             # install_ui_host_capabilities 增加 terminal adapter（if_absent）
  ui/session.rs                      # set_app_window_px 调用点保持（门面不变）
```

- **端口位置**：`TerminalHostProvider` 定义在 `auto-terminal-core`（owned
  值域，无 UI/VM 类型/裸指针/闭包）；`auto-lang` 依赖 core 并 re-export。
  VM shim 只做弹参、调用 provider、push 结果。
- **默认回落**：shim 解析序 = 原 owner（按句柄）→ 本 VM 实例 provider →
  进程默认 provider → `EngineService::shared()`（内置非 UI 实现）。
  **无 provider 不等于禁用引擎**，旧"无 UI 也能 spawn/rows"行为保留。
- **owner 路由（R1）**：VM `NativeInterface` 保存曾安装 provider 的历史；
  带 handle 的 shim 先 `terminal_owner_for(handle)`，再回落当前/共享。
  旧 owner 在最后一个会话释放后剪枝（可触发 Drop）。
- **会话生命周期（R1）**：core 每会话门 + 退役位；free 表内摘除→等门→
  退役→DLL free 一次；`Drop for EngineService` 在 Library 析构前清空活
  会话。句柄命名空间进程全局唯一。
- **UI adapter**：`TerminalUiProvider { service: Arc<EngineService> }`；
  `install_process_default()` 用 `if_absent` 补缺（显式默认恒胜，779 纪律）。
  UI adapter 的组合方法（rows/rows_for/pump/apply_resize/menu/pend/scroll/
  window）先调 service，后调 UI 注册表。
- **palette 装载**：保持旧"首次引擎调用时懒查询"；UI adapter 在首次
  engine 调用后把 `service.palette_tables()` 写入 `terminal_palette_load`
  （UI 缓存单源不变；无 DLL 时内置回退）。
- **cell/color 单份**：`TerminalColor`/`TerminalCell` 定义在 core；
  `ui::terminal::TermColor/TermCell` 改为 `pub use`（路径不变，iced/widget/
  tests 零改动）。UI adapter 的结构转换只复制 `ch/fg/bg`，不重复位域算法。
- **几何常量**：`MIN_RESIZE_COLS=2`、`MAX_RESIZE_COLS=500`、
  `MAX_RESIZE_ROWS=200` 移到 core；UI re-export 同名常量。
- **catalog/codegen**：ID/canonical/`List`/Int/Bool/Void 全部不动；
  `binding_revision` 不 bump（provider 注入不改绑定身份）。
- **a2r/生成 Rust**：`ui_gen/rust.rs` 的 terminal 发射（View::Terminal
  props/oninput/onmenu/onkeydown/scheme/scroll/geometry）与
  `aura_view_builder::convert_terminal` **不改语义**；它们消费同一 UI view
  合同。未支持的 a2r 引擎绑定（原生 `term.*` 调用）如实登记"不在原支持面"，
  不新增。
- **`set_app_window_px`**：保留在原路径作为门面，转发到具名 UI view adapter
  （thread-local 所有权迁出 term_engine）；`ui/session.rs` 三处生产调用零改。
- **禁用**：core 不依赖 auto-lang/UI/auto-man/auto-term；UI adapter 不返回
  UI 核心/裸指针/VMValue/闭包；不新增 terminal 之外的媒体/i18n 迁移。

## 6. 边缘清单（本计划范围外，保留实际路径与 owner）

### 6.1 本计划清除

- `vm/ffi/term_engine.rs` 49 处 `crate::ui::` 直引 → 0（业务迁 core/UI adapter）。
- `vm/native.rs`/`native_catalog.rs` 中 terminal 34 项的 provider 化（shim
  保留同名符号）。
- UI palette/window/input 的 VM 直连由 adapter 承接（不是删除 UI 状态）。

### 6.2 保持不动（其他 owner）

| 残边 | 位置 | owner/退出阶段 |
|---|---|---|
| 媒体 asset/image_pipeline | `vm/ffi/stdlib.rs`（30 处 image_pipeline/MediaAsset 引用；`image_ticket_*`/`media_response_for_vm_request`） | R03 媒体完整子域（后续独立合同）；本计划不碰 |
| i18n | `vm/ffi/stdlib.rs::shim_i18n_t` → `ui::i18n_lookup` | R03 i18n 后续合同；本计划不碰 |
| diff 9915–9917 | `vm/native.rs`/`ui_gen`（PLAN-703/807） | 807 文本/diff |
| a2r direct frame arms | `a2r_std.rs:1231/1237` | 既有说明（ui/overview 未迁边界）；不因本计划扩大 |
| editor/dialog/dnd/scroll/clipboard | 已迁 provider（779/785/790/794/802/804） | 不动 |
| terminal UI 注册表/输入队列/选中/菜单/滚动投影 | `ui/terminal/mod.rs`、`iced/widget.rs`、`ui/dynamic.rs:2587`（split 载荷侧车） | 仍属 UI 视图/输入 owner；本计划只改消费路径 |
| a2r/a2c 原生 engine C-ABI 语料 | `test/a2c/18_c_interop/004_engine_face`、`test/a2r/27_c_abi/*` | 不在 34 native 支持面；不改 |

`ui/dynamic.rs:2587` 的 `terminal_drain_all_inputs()` 是 split 轨 UI 内部
载荷侧车（UI 输入队列 owner），不构成 VM→UI 生产直连；本计划保留。

## 7. 旧 oracle 与 driver 入口

- **T-01 driver/oracle**：`t01-dll-probe.py`（ctypes 直驱真实 DLL），
  身份/导出/句柄/多会话/退出/free 的原始证据见 `t01-dll-probe.log.txt`。
- **T-03/T-06 必须**：在 `crates/auto-lang/src/tests/plan808_terminal_domain.rs`
  重建 Rust 入口，消费**实际 DLL**（`AUTOTERM_ENGINE_DLL` 指向 §0 产物），
  含截止时间与超时后 kill+free；缺 DLL 环境显式 `pending`，不得以 fake
  代替或静默 skip 当绿。
- **旧 term/Mux 定向反例**：沿用
  `ui/terminal/mod.rs` 的 `per_key_*`/`menu_item_any`/`input_queue_fifo`/
  `resize_request_pends`/`p025_scroll_render_tests` 等既有断言；迁移后
  这些测试必须保持绿（UI 状态 owner 未变）。
- **独立 consumer**：T-06 建立
  `test/fixtures/modularity/terminal-core-consumer/`（独立 workspace，
  只依赖 `auto-terminal-core`），证明无 auto-lang/UI checkout 可消费。

## 8. 有界决策与未决项

| 项 | 决策 | 依据 |
|---|---|---|
| provider 端口定义位置 | `auto-terminal-core` | core 合同单份；VM 只编解码 |
| 默认回落 | 共享 `EngineService`（非 raise） | 保留无 UI 引擎行为 |
| 句柄命名空间 | **进程全局唯一**（core 静态计数器） | 两隔离 service 不得重号；owner 路由可判定 |
| 句柄 owner 路由 | per-VM provider 历史 + `owns_handle` 反查 | provider 替换后旧句柄仍归原 owner（R1-F01） |
| 会话并发/释放 | 每会话门 + 退役位；free 摘除→等待→一次 DLL free | DLL 锁不验证存活；防双 free/在途释放（R1-F03） |
| NI owner 回调 | 锁内只快照/摘除/合并；provider 方法与析构锁外 | 四类重入死锁不可接受（R2-F01） |
| retired 缓存 | shim free 复制 cursor/viewport 到 per-VM retired 表 | 切换/剪枝后仍读冻结值且不持旧 service（R2-F02） |
| service 末引用 | `Drop` 逐个 free（Library 析构前） | 独立 service 不泄漏 PTY/子进程（R1-F04） |
| free 后 CURSORS/VIEWPORTS | 保留旧值 | 旧臂可观察值冻结；单调句柄不复用 |
| backlog 迟滞 | 每 service 全局（默认=进程全局） | 旧静态语义 |
| palette | UI 缓存 + core 懒查询 | 旧 DLL 单源/内置回退 |
| cell/color | core 单份定义，UI re-export | 算法单份 |
| window px | UI thread-local + 门面 | 旧线程亲和 |
| a2r 引擎绑定 | 明确不新增 | 原支持面之外 |
| 媒体/i18n | 不迁 | 其他 owner |

**T-01 阻塞判定**：无。真实 DLL 在位且探针通过；FFI 寿命与多会话隔离已
确认；provider 替换路由由"全局句柄命名空间 + per-VM owner 历史 + 会话
lease/退役 + service Drop"冻结（R1 补强），T-02
实施后以两 VM 隔离测试验证。

## 9. 计数与证据

- 34/34 native 已映射；catalog 签名表 34 条；shim 34 个；无新增/删除 ID。
- term_engine 旧 UI 直引 49 处；目标 0（T-07 复核）。
- 真实 DLL 探针 34 项检查全 PASS；DLL SHA256/源 HEAD/工具链见 §0。
- 后续收据：T-06 `attachments/808/` 测试与 driver 日志；T-07 `handoff.md`。
