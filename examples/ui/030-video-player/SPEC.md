# SPEC — 030-video-player

> **Purpose**：真实可播的本地视频播放器示范。数据来自后端对媒体根目录的
> **递归扫描**，播放由浏览器原生 `<video>` 承担并通过**受控媒体契约**由应用状态
> 驱动；界面只显示**实测**值，不显示任何推不出的元数据。
> **双端**：Vue 模式（`auto run`）真实播放；VM/Iced 模式（`auto run -r vm`）
> 无解码能力时给出**显式降级说明**（不是黑屏）。
> **审计线**：PLAN-617（本文件描述实现落地后的真实状态，取代 Plan 542 的
> mock 版描述）。

---

## 1. 架构与布局

四个前置组件 + 一个 store，根 slim 到「顶栏 + 骨架 + 消息转发」；另有一个
手写 Web 组件承担本地文件选择（`use.web component`，k4 先例）：

```
src/front/
├── app.at           根组件：h-12 顶栏 + 中部 row + h-14 播控条；消息转发
├── viewport.at      视口：受控 <video> + 状态条 + 错误面板 + 音轨标注
├── playlist.at      右栏队列：按 rel_dir 分组 + 本地文件选择 + 重扫描
├── controls.at      OSD 播控条：时间/进度段 + 传输键 + 音量/倍速
├── player_store.at  全部状态与业务逻辑 + 受控契约的上行 handler
└── file_picker.vue   手写组件：隐藏 <input type="file">；trigger 计数驱动，
                      ready/pick 两 emit（object URL + 文件名回传 store）
```

```
+-------------------------------------------------------------------------------+
| [icon] Video Player  <当前文件名>                     [主题] [队列显隐]        |  h-12
+------------------------------------------------+------------------------------+
| 标题 / 相对目录 / 真实字节数                    |  播放队列            [X]     |
|------------------------------------------------|  --------------------------  |
|                                                |  根目录                      |
|            受控 <video>（object-contain）       |    caelestia       7.3 MB   |
|            （解码失败 → 错误面板替换纯黑）       |  TV/Loki                    |
|                                                |    Loki.S01E01…    2.3 GB   |
|------------------------------------------------|    …                        |
| 状态  |  画幅  |  时长 <回灌>                   |  [打开本地视频文件]          |
|                                                |  [重新扫描媒体库]            |
+------------------------------------------------+------------------------------+
| 00:12 / 00:49   <状态>   [段×10]  ‹ [播放/暂停] ›    静音 - 80% +  1.0x      |  h-14
+-------------------------------------------------------------------------------+
```

布局纪律（PLAN-617 §2.2 / AC-01）：发丝线分隔、**零**卡片阴影/毛玻璃/渐变；
尺寸收敛为三档（icon `w-8 h-7`、传输 `w-9 h-9`、文本 `h-9 px-3`）；
每视图最多 1 个 primary 按钮（播放/暂停）。

---

## 2. 数据契约

### 2.1 后端（由 `auto-man` 生成，服务实现在 `auto-lang/src/ui/media_service.rs`）

| 端点 | 语义 |
|---|---|
| `GET /api/media/scan` | **递归**索引媒体根；返回 `{entries:[{id,title,name,rel_dir,relative_path,extension,bytes,size_str,video_url}], root_missing}`；`id` 是 `blake3(relative_path)` **令牌**，绝对路径不出后端；`root_missing=true` 表示已配置的根目录不存在（与「目录为空」分开说，AC-10） |
| `GET/HEAD /api/media/stream/:id` | 按 `id` 反查文件并做 **HTTP Range**（206 + `Accept-Ranges` + `Content-Range`；无 Range → 200；越界 → 416）；**惰性分块**，绝不整文件入内存 |

媒体根解析序：`AUTO_MEDIA_ROOT` env（显式，优先）→ pac.at `media_root`
（本示例已写 `E:\Video`；`auto run` 仅在 env 未设时注入子进程）→
无（诚实空库）。
扩展名白名单：`mp4/m4v/webm/mkv/mov/avi`——**只表示「候选」**，能否解码由浏览器定。

**VM 端消费同一后端**（T-10）：VM 的 `Http` 通道支持**相对 URL 按基址展开**——
设 `AUTO_HTTP_BASE=http://127.0.0.1:8330` 后，VM 与浏览器消费同一个后端。
取 JSON body 的**双端同源配方**是 `json.to_value(Http.get_json(path))`：VM 侧
get_json 返回 body 字符串（reqwest，经基址展开）、`to_value` 解析成 Value（与
`#[api]` 改写同型）；Vue 侧 get_json 映射为 `fetch().json()`、to_value 为恒等。
**不要写裸 `Http.get`**——它在 VM 返回响应句柄（Plan 446 契约，配
`.status()/.body()` 访问器），不是解析后的 body。基址展开覆盖
get/post/put/delete/patch/json/request/auth 臂；download/stream/SSE 臂未覆盖。

### 2.2 受控媒体契约（作者面，两端同名同单位）

```auto
video {
    src: .store.current_url      // 透传
    controls: false              // 透传（自带控件关闭，用自家 OSD）
    preload: "metadata"
    playsinline: true
    paused: .store.is_playing == false   // 受控下行（false → play()，true → pause()）
    position: .store.seek_target         // 秒；**目标值变化才 seek**
    volume: .store.volume                // 作者面 0..100（Vue 侧换算成元素 0..1）
    muted: .store.is_muted
    rate: .store.playback_rate           // float；<= 0 被忽略
    ontimeupdate: .OnTime($0)            // $0: float 秒
    onloadedmetadata: .OnDuration($0)    // $0: float 秒
    onplaystatechange: .OnPlayState($0)  // $0: bool（由 play/pause 合成）
    onended: .OnEnded
    onmediaerror: .OnMediaError($0)      // $0: str（真实 error 文案）
    onaudiotrack: .OnAudioTrack($0)      // $0: bool 音轨可用性（AC-16b）
}
```

**onaudiotrack 语义**（仅 Vue/Chromium 发射）：元素**实际播放超过 1s** 后读
`webkitAudioDecodedByteCount`（Chromium 专有探针）——为 0 ⇒ `false`（该文件的
音轨编码，如 Dolby Digital Plus，无浏览器解码器），界面必须**主动标注**
「音轨不受支持」；>0 ⇒ `true`（界面无声不标，有声也不标）。探针不存在的
浏览器**不调用 handler**（不主张任何结论）；VM/mpv 端无此事件（mpv 自带
Dolby 解码，不存在该问题）。换片复位一次性探针标志。

**兼容边界**：未声明任一受控下行 prop 的 `video`，生成结果与引入契约前**逐字节
一致**（仍是裸 `<video :src :class />`）——`019-video-app` 等既有示例零影响。
生成器实现位置：`crates/auto-lang/src/ui_gen/vue.rs`
（`try_generate_controlled_video_html` / `video_script_block`）。
VM 侧的同一份契约在 `crates/auto-lang/src/ui/mpv/contract.rs`。

### 2.3 不谎报

分辨率 / 编码 / 码率在不解复用的前提下**无从得知**，因此：
- 数据模型里没有这些字段（旧的 `resolution:"4K"` / `codec:"HEVC"` 字面量已随
  `src/back/db.at` 退役）；
- 时长只来自 `loadedmetadata` 回灌，当前时间只来自 `timeupdate` 回灌；
- 音轨只有一条**实测**信息：`onaudiotrack` 探针（§2.2）回灌的可用性——
  解不出时标注「音轨不受支持」，解得出时什么都不说；没有第三种声称。

---

## 3. 状态机（player_store.at）

`Load` → `json.to_value(Http.get_json("/api/media/scan"))`（双端配方，§2.1）→
三态如实落地：
**catch**（后端不可达 → `library_error` 文案）/**`root_missing`**（根目录不存在
→ 专文案）/**entries 空**（根目录可读但没有白名单文件）→ 正常态则
`playlist` + `groups`（按 `rel_dir` 去重）→ `SelectIndex(0)`。
切曲 = 改索引 + 复位 `seek_target`/`current_time`/`audio_unsupported`
+ 复位时长（**同索引重选不复位时长**——见 §4 的坑）。

上行 handler 直接驱动界面：`OnTime`/`OnDuration`/`OnPlayState`/`OnEnded`
（放完自动 `Next`）/`OnMediaError`（置 `media_error`，视口显示错误面板）/
`OnAudioTrack(false)`（置 `audio_unsupported`，状态条显示音轨标注）。
`Next`/`Prev` 是 **cycle_index** 语义（末项再下一首回首项）。

**本地文件**（T-09）：`LocalPick` → `local_pick_ready` 真则 `pick_seq += 1`
（file_picker.vue 的 watch 打开系统文件对话框）→ 选中后 `LocalPicked(url, name)`
把 **object URL** 直接当 `current_url`（blob: 是浏览器一等媒体源），换片语义
与 `SelectIndex` 一致；`local_pick_ready` 假（VM 端组件不渲染、ready 事件
永不来）→ 诚实降级文案，不做假动作。`LocalPickReady` 由组件 `onMounted`
发来，是「当前后端有文件选择能力」的实测信号。

---

## 4. 已知边界与坑（实测记录）

| 事项 | 事实 |
|---|---|
| VM 端无解码 | VM 视口渲染「本后端未启用原生播放（构建时未开 `mpv-widget`）」的显式降级面板；打开原生播放需 `cargo build -p auto --features mpv` + `AUTO_MPV_LIB` |
| VM 端媒体库（T-10 已收口） | VM 的 `Http` 现支持 `AUTO_HTTP_BASE` 基址展开：先起生成后端（`cargo run --manifest-path examples/rust-workspace/030-video-player-back`，媒体根经 env/pac.at 解析），再 `AUTO_HTTP_BASE=http://127.0.0.1:8330 auto run -r vm` —— VM 队列与 Vue 端同一份真实数据。未设基址时仍是诚实空态文案 |
| MKV 可解、Dolby 不可解 | Matroska 容器 Chromium 能解（实测 `Loki.S02E01….mkv` 画面 3840×2160）；但 E-AC-3/Atmos 音轨不被 Chromium 的 FFmpeg 构建支持，表现为**有画面无声音**。播放 >1s 后探针回灌，界面**主动标注**「音轨不受支持（Dolby Digital Plus 无浏览器解码器）」（AC-16b） |
| 本地文件选择是 Web 端能力 | `file_picker.vue` 依赖浏览器 File API；VM/iced 端该组件不渲染（`use.web component` 退化为 0×0 占位），`local_pick_ready` 恒 false，按钮点击给出降级文案——这是**能力边界**，不是缺陷 |
| Playwright 自带 Chromium | **不含 H.264/AAC/HEVC 专有编解码器**，`videoWidth` 恒 0；本目录的 e2e 因此强制 `channel: 'chrome'` |
| 重选当前项会清掉时长 | `duration` 只由 `loadedmetadata` 回灌，src 不变时不再触发；故 `SelectIndex` 对「同索引」只做重播（seek 回 0），不清时长——否则进度条（前置条件 `duration > 0`）会永久失效 |
| 主题变量必须在根 widget | `dark_mode`/`accent_color` 放在 **store** 里时，切换只改根 div 的 class，而 `<html class="dark">`（pac.at `theme` 注入）仍在 `:root` 上给着深色变量 → 切浅色无反应。放根 widget 才会拿到生成器的主题运行时（含 html class 同步） |

---

## 5. 验证

| 层 | 命令 | 覆盖 |
|---|---|---|
| 生成器单测 | `cargo t vue_gen`（`test_controlled_video_contract_sfc` / `test_uncontrolled_video_is_byte_identical` / `test_controlled_video_audio_probe_sfc`） | 契约生成形状 + 兼容性约束 + 音轨探针 |
| VM 基址单测 | `cargo t http_base_url_resolution`（`vm/ffi/stdlib.rs`） | 相对 URL 展开/绝对 URL 透传/env 缺省行为 |
| Vue e2e | `cd tests && pnpm exec playwright test`（需 `AUTO_MEDIA_ROOT` + `auto run`） | T1 队列递归/分组、T1b 自然序、T2 起播、T3 暂停、T4/T4b seek、T5 音量静音倍速、T7 上下曲、**T8 本地文件浏览**、T8b 静态文案、T9 错误面板、**T10 重扫描**、T11 逐项真实状态（含音轨标注） |
| VM 冒烟 | `AUTO_BIN=<auto> node tests/vm-smoke.mjs` | 起窗 + `View::Video` 在树 + 无 mock 残留 + 面板显隐 |
| 后端单测 | `cargo t media_service` | 递归/白名单/自然序/令牌/Range/缺根不 panic |
