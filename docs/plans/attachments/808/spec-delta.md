# PLAN-808 Spec delta（work 草案；merge 正式沉淀）

本文件是 T-07 的 SD 草案素材：canonical Specs **未在本阶段修改**，由 merge
按本文件内容落库。SD-01..04 与计划 §5 表一一对应；AC/证据见
`contract-baseline.md`、`handoff.md` 与 plan808 测试矩阵。

## SD-01 add: `docs/specs/auto-terminal-core/project.md`

```markdown
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

- 每实例持有：`Library`（私有）、NEXT_HANDLE、句柄表（i64→原始指针的
  usize 私有存储）、行文本快照、光标/视口缓存、背压迟滞态、palette 表缓存。
- 34 native 的旧值语义逐项冻结（缺 DLL/无效句柄/可选符号/必须符号），
  见 PLAN-808 T-01 矩阵；`spawn_ex` 符号缺失=0、`backlog_paused`/
  `overflow_count` 符号缺失=0、`palette_color` 符号缺失=内置回退。
- 会话生命期：`free` 调 DLL 后删除句柄/快照；**重复 free no-op**；
  光标/视口缓存保留旧值（旧可观察臂；单调句柄不复用，不串新会话）。
- `shared()` 是进程单例（旧静态句柄表的等价物）；`new()`/`with_candidates`
  提供隔离实例（两 VM 隔离注入；句柄只在所属 service 解析，跨 service
  返回旧缺席值，不会误释放他方指针）。
- 锁序：句柄表临界区不跨 FFI；状态锁短临界区、互不嵌套；不持服务锁
  回调 UI/DLL。DLL 内部每柄串行由 auto-term FFI 保证，core 不复制。

## TerminalHostProvider（34 方法）

- 一 native 一方法（2943..2949、2957..2959、2976..2982、2983..2987、
  2988..2993、2998..3003）；owned 值域，不返回 UI 核心/裸指针/VMValue/
  闭包。
- 默认实现（`EngineService`）：引擎面全在位；输入泵/几何/menu/scroll/
  window 为无 UI 降级（0/0/0/-1、window 1024×768）；config 六项空
  （不新增 autoterm-config 依赖）。
- 装配：VM 实例 provider → 进程默认 provider → 默认共享 service；
  **无 provider ≠ 禁用引擎**（保留旧无 UI 行为）。UI 宿主安装 adapter
  后才有输入/投影/menu/scroll/window/palette 面。

## 不在本域

media asset/session/http、i18n、diff 9915..9917、terminal renderer/UI
注册表、a2r 原生 engine C-ABI 语料。它们各有 owner；本域完成不宣告
R03 整包完成。

## 验证入口

`cargo check -p auto-terminal-core`、`cargo test -p auto-terminal-core`、
`cargo test -p auto-lang --lib plan808`（`AUTOTERM_ENGINE_DLL` 指向真实
构建产物时含真实 PTY 生命周期）、
`cargo test --manifest-path test/fixtures/modularity/terminal-core-consumer/Cargo.toml`、
`bash test/fixtures/modularity/terminal-core-consumer/verify-deps.sh`。
```

## SD-02 modify: `docs/specs/auto-lang/vm/overview.md`

在「现状」列表（PLAN-804 条目后）追加：

```markdown
- terminal 34 native 全族（2943..2949/2957..2959/2976..2982/2983..2987/
  2988..2993/2998..3003）为 terminal 批次（PLAN-808）：core shim 仅
  弹参/push，经 per-VM `auto_terminal_core::TerminalHostProvider` 供值；
  引擎服务/句柄/采样/背压归 `auto-terminal-core`，不再直引 UI 的
  terminal 注册表/theme。缺席=回落进程共享 engine service（不是错误/
  不是空值），保留旧无 UI 驱动行为；UI 宿主安装 adapter 后才有输入/
  几何/menu/scroll/window/palette 面。详见
  [terminal native 宿主能力（PLAN-808）](#terminal-native-宿主能力plan-808)。
```

并在 `## dialog/dnd native 宿主能力（PLAN-804）` 后追加新节：

```markdown
## terminal native 宿主能力（PLAN-808）

terminal 34 native（`auto.term.*`，ID/签名/canonical 名不变）供值经
per-VM `auto_terminal_core::TerminalHostProvider`（一 native 一方法，
owned 值域）。VM shim 只弹参/编码/push；DLL 加载、符号契约、会话句柄
表、行/样式采样、光标/视口缓存与背压迟滞归 `auto-terminal-core`
（Cargo 闭包仅 std+libloading，不依赖 auto-lang/UI/man/auto-term）。

装配=779/785/790/794/802/804 同构三级（实例注入 > 构造期进程默认快照 >
兼容 if-absent），但**默认回落不同**：无 provider 时 shim 使用
`auto_terminal_core::default_provider()`（进程共享 engine service），
保留迁移前"无 UI 也能 spawn/rows/backlog"的行为；无 provider ≠ 禁用
引擎。注入不 bump binding_revision/不撤 proof；NI Clone/VM merge 保
provider。

缺席/句柄语义（T-01 冻结）：缺 DLL/无效句柄 spawn=0、rows=[]、
interrupt=-1、is_exited=true、history/backlog=0、cursor/viewport 缓存缺省
(0,0)；free 后光标/视口保留旧缓存（旧可观察臂）；重复 free no-op；
句柄归属 service 实例，跨 service 使用返回旧缺席值且不会误释放原会话。
config 六项保持空集/空串；window 无 UI=1024×768，UI=thread-local override
或 theme；`set_app_window_px` 兼容门面转发 `ui::terminal_provider`。
display_offset 唯一真源仍为引擎，UI 只投影（Mux 契约不变）。

外部 no-UI 宿主可注入自定义 provider；真 no-UI 消费方探针
（test/fixtures/modularity/terminal-core-consumer，独立 workspace）与
真实 DLL 生命周期 driver（`AUTOTERM_ENGINE_DLL`）为验收 oracle。
```

## SD-03 modify: `docs/specs/auto-lang/ui/overview.md`

在「现状（2026-10-18）——dialog/dnd adapter 宿主装配（PLAN-804）」之后
追加：

```markdown
## 现状（2026-10-18）——terminal 域 adapter 宿主装配（PLAN-808）

> terminal 完整子域批次（VM 侧契约见 vm/overview terminal native 宿主
> 能力节；domain 合同见 `docs/specs/auto-terminal-core/project.md`）。

UI 侧 `ui::terminal_provider::TerminalUiProvider` 组合
`auto-terminal-core` 的共享 `EngineService` 与既有 `ui::terminal`
注册表：输入泵/定向排空、pending 几何、menu 载荷、滚轮回灌与窗口投影、
window px thread-local、palette 懒装载全部留在 UI adapter；引擎 DLL/
句柄/采样/背压/颜色位域算法归 core（`TermColor/TermCell` 由 core 单份
定义并在此 re-export，路径不变）。`set_app_window_px` 保留兼容门面。

宿主接线与 779/785/790/794/802/804 同集：`install_ui_host_capabilities`、
`run_file_dynamic_ui_inner`、VmBridge 两构造（全 if-absent）。
`ui_gen/rust.rs` terminal 发射与 `aura_view_builder::convert_terminal`
的 View::Terminal props/事件语义不变；未支持的 a2r 引擎原生绑定如实
记"不在原支持面"，不新增。

**未迁边界（更新）**：`vm/ffi/term_engine.rs` 旧 UI 直引由 49 → 2
（仅窗口像素门面的两处 adapter 委托）；`vm/native.rs` 口径剩余
diff 族 9915..9917（PLAN-703/807）、媒体 image_pipeline
（`vm/ffi/stdlib.rs`）、i18n（`vm/ffi/stdlib.rs::shim_i18n_t` →
`ui::i18n_lookup`）及其他既有族。terminal 34 项清零不等于 R03 整包完成；
媒体/i18n 等仍待各自完整合同。
```

同时把原「未迁边界」行中的 `term/media` 改为 `media`（terminal 已迁）。

## SD-04 modify: `docs/specs/terminal-mux-model.md`

在「虚拟滚动契约」前增加实现 owner 注记，规则正文不变：

```markdown
## 实现 owner（PLAN-808 更新）

display_offset 的唯一真源仍是终端引擎（`autoterm_core.dll`）；
`auto-terminal-core` 持有引擎服务/句柄/采样，UI `ui::terminal` 注册表
与 iced scrollable 只做视图投影。UI 与引擎之间的滚动队列、history/
offset 回写、window anchor、预取窗与回声抑制由
`ui::terminal_provider::TerminalUiProvider` 转发；§V1.10 的
display_offset 单源、坐标映射、双臂回声判别与键入贴底语义逐条不变。
```

## 备注

- 本阶段不修改 `docs/specs/README.md` 索引/`.autoos/specs.json` ledger；
  merge 负责索引、ledger guarded projection 与归档。
- `supersedes_spec_components`/`new_spec_components` 元数据已在计划
  frontmatter（SD-01 new；vm/ui/mux 三项 modify）。
