# SPEC — 030-video-player（Plan 542）

> **Purpose**: AutoOS 系统级原生视频播放器（对标 VLC / PotPlayer / IINA / MPC-HC）。
> 沉浸式大视口画布播放 + 自动隐匿 OSD 专业播控条 + 交互式进度寻道 + 多档倍速 + 音量滑块 + 画幅调节 + 可折叠播放队列抽屉 + 全套键盘快捷键。
> **双端支持**：Vue 模式（`auto run`）与 VM/Iced 模式（`auto run -r vm`）。
> **主题**：AutoOS Dark / indigo 默认；root 声明 `dark_mode` bool / `accent_color` str 契约变量，支持五色强调色（indigo, coral, ocean, sage, amber）。

---

## 1. 架构与布局

单文件 App 根 Widget 形态（对齐 020/027/028/029 实证模式），所有状态高内聚于 `src/front/app.at`。

```
+-----------------------------------------------------------------------------------------------+
| ≡ 文件  播放  音频  字幕  帮助              AutoOS Video Player               —   □   ✕   |
+-----------------------------------------------------------------------------------------------+
|                                                                         | 播放队列 (5)   ✕   |
|                                                                         | ------------------- |
|                                                                         | ▶ 01_intro.mp4      |
|                                                                         |   1080P · 03:45     |
|                                                                         |                     |
|                                                                         |   02_architecture   |
|                                                                         |   4K · 12:20        |
|                                [ 视频主画面 / 画布视口 ]                   |                     |
|                              (拖拽视频至此 / 双击全屏)                    |   03_ui_engine      |
|                                                                         |   1080P · 08:15     |
|                                                                         |                     |
|                                                                         |   04_workflow.webm  |
|                                                                         |   1080P · 05:30     |
|                                                                         |                     |
|                                                                         |   05_ambient.mp4    |
|                                                                         |   720P · 01:30      |
+-------------------------------------------------------------------------+---------------------+
|  [===========================●========================================] 04:15 / 12:20 (35%)   |
|  ⏮   ⏪ 5s   ▶ 播放   ⏩ 5s   ⏭   ⏹   🔊 [====●---] 80%   1.25x ▾    🔤 字幕   ⛶ 全屏   ≡ 列表 |
+-----------------------------------------------------------------------------------------------+
```

---

## 2. 数据结构与种子视频

### 2.1 种子视频数据（平行列表/唯一真源）
| id | 标题 | 文件名 | 时长 | 秒数 | 分辨率 | 编码 | 大小 | 渐变背景 |
|---|---|---|---|---|---|---|---|---|
| 1 | AutoOS 2026 Keynote & Intro | 01_intro.mp4 | 03:45 | 225 | 1080P | H.264 | 48 MB | from-indigo-950 via-slate-900 to-purple-950 |
| 2 | Kernel & AutoVM Architecture | 02_architecture.mkv | 12:20 | 740 | 4K | HEVC | 210 MB | from-blue-950 via-slate-900 to-cyan-950 |
| 3 | AutoUI Dual-Backend Engine Demo | 03_ui_engine.mp4 | 08:15 | 495 | 1080P | AV1 | 85 MB | from-emerald-950 via-slate-900 to-teal-950 |
| 4 | Fullstack Parity & Toolchain | 04_workflow.webm | 05:30 | 330 | 1080P | VP9 | 62 MB | from-amber-950 via-slate-900 to-orange-950 |
| 5 | AutoOS Ambient Desktop Visuals | 05_ambient.mp4 | 01:30 | 90 | 720P | H.264 | 24 MB | from-rose-950 via-slate-900 to-pink-950 |

### 2.2 结构体定义
```auto
type VideoTrack {
    id: int,
    title: str,
    file_name: str,
    duration_str: str,
    duration_sec: int,
    resolution: str,
    codec: str,
    size_str: str,
    bg_gradient: str,
}

type AccentItem {
    name: str,
    dot: str,
    bg: str,
}
```

---

## 3. 状态全集 (Model)

### 3.1 播放状态
- `is_playing bool`：播放状态（true = 播放中，false = 暂停）
- `current_id int`：当前视频 ID（默认 1）
- `current_title str`：当前视频标题
- `current_file str`：当前文件名
- `current_res str`：当前分辨率
- `current_codec str`：当前编码
- `current_bg str`：当前画布渐变色
- `current_time_sec int`：当前播放秒数（初始 45）
- `total_time_sec int`：总秒数（初始 225）
- `current_time_str str`：当前时间字符串（"00:45"）
- `total_time_str str`：总时间字符串（"03:45"）
- `time_display_fmt str`：组合时间文案（"00:45 / 03:45"）
- `progress_pct int`：进度百分比 0~100（20）
- `volume int`：音量 0~100（80）
- `is_muted bool`：静音状态（false）
- `playback_speed str`：当前倍速（"1.0x"）
- `aspect_mode str`：画幅模式（"contain" | "cover" | "fill"）
- `aspect_label str`：画幅标签（"自适应 16:9" | "裁剪铺满" | "拉伸填满"）
- `loop_mode str`：循环模式（"list" | "single" | "shuffle"）
- `loop_label str`：循环标签（"列表循环" | "单曲循环" | "随机播放"）
- `time_mode str`："elapsed" | "remaining"

### 3.2 界面交互状态
- `show_playlist bool`：播放队列显隐（默认 true）
- `show_controls bool`：OSD 控制条显隐（默认 true）
- `show_speed_menu bool`：倍速下拉菜单显隐（默认 false）
- `show_info_modal bool`：媒体详细信息弹层（默认 false）
- `is_fullscreen bool`：全屏模拟开关（默认 false）
- `toast_msg str`：操作提示浮层文本（如 "快进 +5 秒"、"倍速 1.5x"）
- `show_toast bool`：操作提示显隐（默认 false）

### 3.3 主题与风格契约
- `dark_mode bool = true`
- `accent_color str = "indigo"`
- `accents []AccentItem`：5 色强调色（indigo, coral, ocean, sage, amber）

---

## 4. 消息与行为 (Messages & Handlers)

| 消息 (Msg) | 参数 | 行为说明 |
|---|---|---|
| `Init` | — | 初始化种子列表，设置默认播放第一项 |
| `TogglePlay` | — | 切换播放/暂停，更新 `is_playing` 与 toast |
| `StepForward` | — | 快进 5 秒，更新进度与时间文案 |
| `StepBackward` | — | 快退 5 秒，更新进度与时间文案 |
| `NextVideo` | — | 切换至下一视频，重置当前播放时间为 0 |
| `PrevVideo` | — | 切换至上一视频，重置当前播放时间为 0 |
| `StopPlayback` | — | 停止播放并复位时间至 0 |
| `SeekPercent(int)` | pct | 跳转至指定进度百分比（0~100） |
| `SetVolume(int)` | vol | 调节音量（0~100），若静音则自动解除 |
| `ToggleMute` | — | 切换静音，静音时显示 🔇 |
| `SetSpeed(str)` | spd | 设置倍速并关闭下拉菜单 |
| `ToggleSpeedMenu` | — | 展开/折叠倍速下拉选择菜单 |
| `TogglePlaylist` | — | 展开/折叠右侧播放队列抽屉 |
| `ToggleControls` | — | 显隐 OSD 控制栏（模拟鼠标移动/静止） |
| `ToggleFullscreen` | — | 切换全屏模拟视口 |
| `ToggleAspect` | — | 轮转画幅比（contain → cover → fill → contain） |
| `CycleLoopMode` | — | 轮转循环模式（list → single → shuffle → list） |
| `ToggleTimeMode` | — | 切换总时间与剩余时间显示 |
| `SelectVideo(int)` | id | 选择指定视频并立即开始播放 |
| `ToggleInfoModal` | — | 打开/关闭视频详细媒体元数据弹窗 |
| `DismissToast` | — | 关闭状态通知浮层 |
| `ToggleDarkMode` | — | 切换深浅主题 |
| `SetAccent(str)` | color | 切换当前强调色 |

---

## 5. 键盘快捷键映射

- `Space` / `K`：`TogglePlay`
- `ArrowLeft`：`StepBackward` (-5s)
- `ArrowRight`：`StepForward` (+5s)
- `ArrowUp`：增加音量 5%
- `ArrowDown`：降低音量 5%
- `KeyM`：`ToggleMute`
- `KeyF`：`ToggleFullscreen`
- `KeyP`：`TogglePlaylist`
- `BracketLeft`：减慢倍速（0.25x 步长）
- `BracketRight`：加快倍速（0.25x 步长）
