# auto-terminal-core（终端宿主域核心）

> **Status**: implemented
> 路径：`crates/auto-terminal-core`
> 来源：PLAN-808（R03 终端子域完整迁移）

## 职责

终端宿主域的非 UI 核心：owned 行/cell/颜色/几何/快照合同、运行期
`autoterm_core.dll` 引擎服务（库解析、符号契约、会话句柄表、行采样、
光标/视口/背压状态、palette 查询）与 34 方法 `TerminalHostProvider`
端口。VM 只编解码；UI 宿主实现 view/input/palette/window adapter。

## 依赖边界

- Cargo 闭包仅 `std` + `libloading`（钉住：
  `test/fixtures/modularity/terminal-core-consumer/verify-deps.sh`）。
- **禁止**依赖 auto-lang / UI / auto-man / auto-term；DLL 是运行期资源，
  不建立 Cargo 边（无环铁律）。
- DLL 搜索序（冻结旧臂）：`AUTOTERM_ENGINE_DLL` → 宿主 exe 同目录 →
  exe 祖先 `target/{debug,release}`；缺件 `spawn=0`、其余按无效句柄
  no-op，不因部署缺件崩溃 VM。

## Value contract（owned，无 UI/VM/裸指针）

- `TerminalColor{Default,Indexed(u8),Rgb(u8,u8,u8)}`、
  `TerminalCell{ch,fg,bg}`、`TerminalGeometry{cols,rows}`、
  `TerminalSnapshot{lines,cells,cursor,history,display_offset}`。
- `decode_style_color`：引擎 `(kind<<24)|value` 位域唯一解码实现
  （0=Default、1=Indexed、2=RGB、其他=Default）。
- 几何护栏常量：`MIN_RESIZE_COLS=2`、`MAX_RESIZE_COLS=500`、
  `MAX_RESIZE_ROWS=200`；`PALETTE_SLOTS=18`。

## 引擎服务（EngineService）

- 每实例持有：`Library`（私有）、会话表（每会话 = 私有原始指针 + 操作门 +
  退役位）、行文本快照、光标/视口缓存、背压迟滞态、palette 表缓存。
- **句柄命名空间 = 进程全局**（旧 glue 单计数器语义）：两隔离 service 不会
  发出相同 handle 值；handle 是进程内不透明会话 ID。
- 34 native 的旧值语义逐项冻结（缺 DLL/无效句柄/可选符号/必须符号），
  见 PLAN-808 T-01 矩阵；`spawn_ex` 符号缺失=0、`backlog_paused`/
  `overflow_count` 符号缺失=0、`palette_color` 符号缺失=内置回退。
- **会话生命期（R1 修复）**：每次 FFI 操作在会话门内取得 lease；`free`
  先从会话表摘除（新操作不再可见），等待在途 lease 结束，置退役位并
  调用 DLL free **恰好一次**；并发/重复 free 是 no-op，且绝不会以已释放
  指针再次进入 DLL。不同会话各有独立门，互不阻塞。表锁不跨门/FFI/UI。
- `Drop for EngineService` 在 `Library` 字段析构前释放全部活会话；隔离
  service 末引用释放不遗留 PTY/子进程。默认 shared service 仍按旧语义
  跨 VM 共享会话，VM drop 不承诺杀死共享会话。
- 光标/视口缓存保留旧值（旧可观察臂；单调句柄不复用，不串新会话）。
- **释放后缓存保持（R2/R3/R4）**：`EngineService::free` 在会话退休前把
  cursor/viewport 纯值发布到**进程级 retired 缓存**（**无界**——旧
  `CURSORS`/`VIEWPORTS` 静态表从不淘汰，仍可查询的冻结值不得因无关
  退休被淘汰，R4-02）；VM shim
  另在 free 时复制进 per-VM 表。缓存查询顺序 = per-VM retired 表 → 活
  owner → 仍持有 retired 缓存的旧 owner → **进程级 retired 缓存** → 当前
  provider。因此释放者 VM、其他 VM、直接 `EngineService::free` 的消费者
  都可在 owner 被剪枝/销毁后读取冻结值，且不依赖“提前读过一次”；只存
  纯值，不持旧 service、不触已释放 DLL 指针。
- `shared()` 是进程单例（旧静态句柄表的等价物）；`new()`/`with_candidates`
  提供隔离实例。跨 service 使用未绑定句柄返回旧缺席值；VM 装配按
  原 owner 路由已暴露句柄（见下节），不会误释放他方会话。

## TerminalHostProvider（34 native + 3 lifecycle helpers）

- 一 native 一方法（2943..2949、2957..2959、2976..2982、2983..2987、
  2988..2993、2998..3003）；owned 值域，不返回 UI 核心/裸指针/VMValue/
  闭包。**三个非 native 生命周期辅助**（不改变 native 栈形状）：
  `owns_handle(handle)->bool`（当前是否持有活会话）、
  `live_handle_count()->usize`（活会话数）、
  `cached_cursor_viewport(handle)->Option<TerminalRetiredCache>`
  （活或 retired 句柄的 cursor/viewport 值），供 VM owner 路由、剪枝与
  free 后缓存保持。
- 默认实现（`EngineService`）：引擎面全在位；输入泵/几何/menu/scroll/
  window 为无 UI 降级（0/0/0/-1、window 1024×768）；config 六项空
  （不新增 autoterm-config 依赖）；三个生命周期辅助读会话/缓存表。
- 装配：VM 实例 provider → 进程默认 provider → 默认共享 service；
  **无 provider ≠ 禁用引擎**（保留旧无 UI 行为）。UI 宿主安装 adapter
  后才有输入/投影/menu/scroll/window/palette 面。
- **provider 替换与旧句柄（R1/R2）**：`NativeInterface` 保留本 VM 曾
  安装的 provider 历史（强 Arc）。所有带 handle 的 shim 先按
  `owns_handle` 反查原 owner，再回落当前 provider / 共享 service；因此
  切换 provider 后旧句柄仍读/写/释放到原服务，不会误释放新服务的同号
  会话。旧 owner 在最后一个会话释放后被剪枝；若其为本 VM 唯一强引用，
  service Drop 随即释放剩余资源。
- **锁纪律（R2/R3/R4）**：owner 查询、剪枝、槽位替换**不得在 NI 锁内执行
  provider 方法或析构**。查询在锁内只快照条目列表，锁外调用
  `owns_handle` 并以 generation 重试覆盖并发 set/prune；**剪枝在锁外
  求值期间保持可见列表完整**（快照不摘除）；**R4-01 复活协调**：每次
  重装与每次经 spawn shim 的会话发布 bump 条目 revival 计数，剪枝仅当
  判定 stamp 未变才在短写锁内移除（锁内零 provider 调用）——复活者
  从未离开可见列表，无"移除后重加"窗口；槽位替换锁内只换值，旧
  provider 锁外析构。四类重入回调/析构、
  并发 set/query/prune 的**每次查询**、剪枝窗口内的真实读/free、零→活
  状态转换（stale 零判定 + 重装/真实 spawn 交错）有永久控制。

## 不在本域

media asset/session/http、i18n、terminal renderer/UI
注册表、a2r 原生 engine C-ABI 语料。它们各有 owner；本域完成不宣告
R03 整包完成。（diff 族 9915..9917/9921 已由 PLAN-807 迁出，见
SD-03 与 canonical 2026-10-19 PLAN-807 节，不列本域残边。）

## 验证入口

`cargo check -p auto-terminal-core`、`cargo test -p auto-terminal-core`、
`cargo test -p auto-lang --lib plan808`（`AUTOTERM_ENGINE_DLL` 指向真实
构建产物时含真实 PTY 生命周期）、
`cargo test --manifest-path test/fixtures/modularity/terminal-core-consumer/Cargo.toml`、
`bash test/fixtures/modularity/terminal-core-consumer/verify-deps.sh`。
