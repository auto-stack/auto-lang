# AutoUI 桌面协议 v1（Desktop Protocol v1）

> **版本**：v1（2026-08-29 定稿，Plan 386 Stage 1）。
> **施工图**：[autoshell](../../25-autoshell-dsl-unified-shell.md) §7/§7.1
> （五通道表）；**单源实现**：
> `crates/auto-lang/src/ui/desktop_protocol/`（feature `ui-iced`）。
> **定位**：进程外 App（路线 B）与桌面（compositor 角色）之间的接缝协议，
> 角色对标 Wayland——桌面合成 App 表面，App 上报帧、消费输入。Stage 1 在
> 同进程 loopback 走通本协议（消息/序列化/双端状态机全部真实过线），
> Stage 2 换真 transport（命名管道/共享内存）时**只替换 loopback 层**。

## 1. 版本化

| 版本 | 日期 | 变更 | 关联 |
|---|---|---|---|
| v1 | 2026-08-29 | 初版：五通道消息 + 二进制编解码 + 双端状态机 + loopback 传输 + 462 会话绑定 | Plan 386 S1–S7 |

- 版本常量：`desktop_protocol::PROTOCOL_VERSION = 1`，随每条消息信封头过线。
- **协商规则**：Hello 携带版本；宿主校验不符 → `ProtocolError::VersionMismatch`
  拒收孵化；信封层版本不符 → `CodecError::UnsupportedVersion` 拒收。
- **演进纪律**：只许追加（新变体 tag、新通道号），不许改义/重排既有 tag；
  载荷种类（`DrawList` 的 kind tag）与通道各自独立编号。

## 2. Wire Format（信封）

小端。头部 12 字节定长：

```text
[0..4)   magic   "APDL"（0x41 0x50 0x44 0x4C）
[4..6)   u16     protocol version
[6]      u8      channel（1..=5）
[7]      u8      保留（0）
[8..12)  u32     payload 长度
[12..)   ...     payload（通道内消息：u8 tag + 字段）
```

- 原语：bool=1B(0/1)、String/bytes=u32 长度前缀+内容、Rect=4×f32、RGBA=4×u8。
- 解码终点断言：消息载荷必须恰好耗尽（`TrailingBytes` 拒收——结构漂移哨兵）。
- 零新依赖；`DesktopBus` 载荷沿用既有 `DesktopCommand` 文本记录格式
  （`verb\u{1F}arg`，shell.at 写入侧同源）。

## 3. 五通道

| # | 通道 | 方向 | 消息（tag = 变体定义序） |
|---|---|---|---|
| 1 | 孵化/握手 | app→host→app | `Hello{version,app_name,title,icon,wh,fonts}`(1) → `Welcome{app_id,wid,surface,rect}`(2) → `Ready`(3) |
| 2 | 帧 | app→host（ack 反向） | `FrameReady{wid,frame_id,slot,damage,revision,DrawList}`(4)、`FrameAck{wid,frame_id,slot}`(5)、`BufferAlloc`(1)/`BufferRelease`(2)/`Resize`(3) host→app、`CacheControl{wid,drop_keys}`(6) app→host |
| 3 | 输入 | host→app | (Wid, event) 编码：`PointerMoved`(1)/`Pressed`(2)/`Released`(3)、`KeyPressed`(4)/`KeyReleased`(5)、`CharTyped`(6)、`Scroll`(7)、**`ImePreedit`(8)/`ImeCommit`(9)/`ImeCancelled`(10)** |
| 4 | 控制 | 双向 | host→app：`Close`(1)/`Focus`(2)/`Resize`(3)；app→host：`TitleChanged`(4)/`Notify`(5)/`ExitRequest`(6)/`DesktopBus{record}`(7) |
| 5 | 观测 | 双向 | host→app：`Attach{sink}`(1)/`Detach`(2)；app→host：`Log{level,message}`(3)/`Metric{key,value}`(4) |

### 3.1 帧：共享纹理模拟（Stage 1）→ 共享内存（Stage 2）

- 表面句柄 `surface` 由宿主在孵化时分配；**双缓冲 2 槽**：app 写非前台槽 →
  `FrameReady(slot)` → 宿主合成（翻面）→ `FrameAck` 归还原前台槽。
- `damage` = 脏区（widget 本地坐标；None = 全帧）。`revision` = app 侧内容
  单调版本，宿主缓存键与 413 §7.3 同源。
- 载荷 v1 = `DrawList{clear, ops:[Quad|Text]}`（kind tag=1，扩展位预留给
  Stage 2 的全量 RenderCommand lowering——见 §6 偏差记录①）。
- Stage 1 的"共享缓冲" = 宿主内存 `SurfaceStore`（每 surface 2 槽）；
  Stage 2 换 OS 共享内存句柄，槽位/翻面/ack 语义不变。

### 3.2 Plan 413 §7 三点落位

| §7 要求 | v1 落位 |
|---|---|
| ① IME 下行（preedit/commit/光标矩形） | `InputMsg::ImePreedit{text,cursor}` / `ImeCommit{text}` / `ImeCancelled`；`EditorFrameSource` 映射到 `EditorInput` 同名输入 |
| ② 字体注册（app 自带字体上传） | `HandshakeMsg::Hello.fonts: Vec<FontBlob{family,data}>`；Stage 1 协议通道就位，真上传随 Stage 2 transport |
| ③ 按行 CacheControl / DirtyRect | `FrameReady.damage`（脏区）+ `FrameMsg::CacheControl.drop_keys`；编辑器键 = `(revision<<16) ^ fold_hidden`（fold 翻转不改 revision 但必须重绘 gutter） |

## 4. 状态机（非法迁移一律拒收）

```text
App  : Detached --connect()--> Handshaking --Welcome(+Ready)--> Active
       Active --Control::Close--> Closing --ExitRequest--> (BufferRelease) --> Detached
       Active --send_exit()--> Closing
Host : Listening --Hello(版本校验)--> (ResolveAndAttach) --activate()--> Active
       Active --ExitRequest--> (ReclaimWindow：wm_remove_win+App 移除+表面释放) --> Listening
```

错误形态（`ProtocolError`）：`WrongState{state,msg}` / `VersionMismatch` /
`ResolveFailed(String)` / `NotActive` / `Codec(_)`。方向错（app 收 Hello、
宿主收 FrameReady 于 Listening 等）与时序错均拒绝，不静默丢弃。

## 5. Stage 2 换 transport 面（本协议的验收遗留兑现点）

| Stage 1（loopback） | Stage 2（真两进程） | 不变量 |
|---|---|---|
| `LoopbackEnd::send/try_recv`（内存字节管道） | `\\.\pipe\autodesk-broker` 命名管道 / memfd | 管道保序 + 字节化过线 |
| `SurfaceStore`（内存 2 槽） | OS 共享内存句柄 + named events | 槽位/翻面/ack 语义 |
| `ResolveAndAttach` → `build_dynamic_component` | spawn-client 双模 exe（`--autodesk-client=<pipe>`） | 孵化三步：身份上报 → 分配 Wid+表面 → 回传句柄 |
| `ReclaimWindow` → 进程内移除 | 进程退出 → 回收虚拟窗（462 Close 语义） | 窗随 App 消亡 |
| `ObserveMsg` 收件箱 | desktop_mcp per-app 端口代理 | 观测通道可代理 |

窗口形态迁移 L1/L3（autoshell §7.1）不在协议 v1 范围——L1 是宿主侧 WM
操作拼装，L3 需要 AutoVM 状态序列化（v2a 快照重启）。

## 6. 偏差与决策记录（对 Plan 386 原文的修正）

1. **帧载荷 v1 = `DrawList`（quad+text run），不含全量 VTree→RenderCommand
   lowering**。原"工作项 Stage 1"的逐 example golden lowering 已被 2026-08-28
   复活重构（计划 §0）取代；全量 lowering 归 Stage 2。依据：`code_editor/
   draw.rs` 头注明言 Stage 1 = "serializes it to quads and text runs"；
   §0 "frame 用内存缓冲模拟共享纹理"。
2. **loopback demo 为 headless 会话对象级验证**（`DesktopSession`/`WmState`
   真实参与孵化/命中/回收；live-iced 渲染器换接 `dynamic_view` → 协议客户端
   留 Stage 2 随真 transport 一起做——I1 评审已证零删除替换可达）。
3. 单客户端 v1：`HostEndpoint` 一次只持一个客户端；多 App 并发复用同一
   协议归 Stage 3（多 App + 形态迁移）。

## 7. 验证（Stage 1 验收）

- 协议测试 31 项（`cargo t desktop_protocol --features ui-iced` 全绿）：
  全消息 round-trip、每通道 golden bytes、坏 magic/版本/未知 tag 拒收、
  状态机全迁移 + 非法迁移、FIFO/线上破坏、462 对象真实孵化/命中/回收、
  计数器 loopback demo 直挂等价（state/frame parity）、编辑器 golden
  （打字/IME/缓存键 + 协议输入与 core 直喂无差）。
- 验收句（计划 §0）："一个 App 的帧/输入/控制经协议通路渲染进 462 虚拟
  窗口，行为与直挂无差" —— `counter_loopback_demo_parity_with_direct_mount`。
