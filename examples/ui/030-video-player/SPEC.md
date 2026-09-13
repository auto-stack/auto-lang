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

四个前置组件 + 一个 store，根 slim 到「顶栏 + 骨架 + 消息转发」：

```
src/front/
├── app.at           根组件：h-12 顶栏 + 中部 row + h-14 播控条；消息转发
├── viewport.at      视口：受控 <video> + 状态条 + 错误面板
├── playlist.at      右栏队列：按 rel_dir 分组 + 底部管理键
├── controls.at      OSD 播控条：时间/进度段 + 传输键 + 音量/倍速
└── player_store.at  全部状态与业务逻辑 + 受控契约的上行 handler
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
| `GET /api/media/scan` | **递归**索引媒体根；返回 `{entries:[{id,title,name,rel_dir,relative_path,extension,bytes,size_str,video_url}]}`；`id` 是 `blake3(relative_path)` **令牌**，绝对路径不出后端 |
| `GET/HEAD /api/media/stream/:id` | 按 `id` 反查文件并做 **HTTP Range**（206 + `Accept-Ranges` + `Content-Range`；无 Range → 200；越界 → 416）；**惰性分块**，绝不整文件入内存 |

媒体根解析序：`AUTO_MEDIA_ROOT` → pac.at `media_root` → 无（空态）。
扩展名白名单：`mp4/m4v/webm/mkv/mov/avi`——**只表示「候选」**，能否解码由浏览器定。

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
}
```

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
- 界面**不出现**任何音轨/编码的声称（无「AAC Stereo 48kHz」这类文案）。

---

## 3. 状态机（player_store.at）

`Load` → `Http.get("/api/media/scan")` → `playlist` + `groups`（按 `rel_dir`
去重）→ `SelectIndex(0)`。切曲 = 改索引 + 复位 `seek_target`/`current_time`
+ 复位时长（**同索引重选不复位时长**——见 §4 的坑）。

上行 handler 直接驱动界面：`OnTime`/`OnDuration`/`OnPlayState`/`OnEnded`
（放完自动 `Next`）/`OnMediaError`（置 `media_error`，视口显示错误面板）。
`Next`/`Prev` 是 **cycle_index** 语义（末项再下一首回首项）。

---

## 4. 已知边界与坑（实测记录）

| 事项 | 事实 |
|---|---|
| VM 端无解码 | VM 视口渲染「本后端未启用原生播放（构建时未开 `mpv-widget`）」的显式降级面板；打开原生播放需 `cargo build -p auto --features mpv` + `AUTO_MPV_LIB` |
| VM 端媒体库为空 | VM 的 `Http.get` 直接把 URL 交给 reqwest，而 `/api/media/scan` 是相对路径 → VM 里显示空态。属平台侧缺口（登记为债务） |
| MKV 可解、Dolby 不可解 | Matroska 容器 Chromium 能解（实测 `Loki.S02E01….mkv` 画面 3840×2160）；但 E-AC-3/Atmos 音轨不被 Chromium 的 FFmpeg 构建支持，表现为**有画面无声音**。界面不声称有声 |
| Playwright 自带 Chromium | **不含 H.264/AAC/HEVC 专有编解码器**，`videoWidth` 恒 0；本目录的 e2e 因此强制 `channel: 'chrome'` |
| 重选当前项会清掉时长 | `duration` 只由 `loadedmetadata` 回灌，src 不变时不再触发；故 `SelectIndex` 对「同索引」只做重播（seek 回 0），不清时长——否则进度条（前置条件 `duration > 0`）会永久失效 |
| 主题变量必须在根 widget | `dark_mode`/`accent_color` 放在 **store** 里时，切换只改根 div 的 class，而 `<html class="dark">`（pac.at `theme` 注入）仍在 `:root` 上给着深色变量 → 切浅色无反应。放根 widget 才会拿到生成器的主题运行时（含 html class 同步） |

---

## 5. 验证

| 层 | 命令 | 覆盖 |
|---|---|---|
| 生成器单测 | `cargo t vue_gen`（`test_controlled_video_contract_sfc` / `test_uncontrolled_video_is_byte_identical`） | 契约生成形状 + 兼容性约束 |
| Vue e2e | `cd tests && pnpm exec playwright test`（需 `AUTO_MEDIA_ROOT` + `auto run`） | T1 队列递归/分组、T1b 自然序、T2 起播、T3 暂停、T4 seek、T5 音量静音倍速、T7 上下曲、T9 错误面板、T11 逐项真实状态 |
| VM 冒烟 | `AUTO_BIN=<auto> node tests/vm-smoke.mjs` | 起窗 + `View::Video` 在树 + 无 mock 残留 + 面板显隐 |
| 后端单测 | `cargo t media_service` | 递归/白名单/自然序/令牌/Range |
