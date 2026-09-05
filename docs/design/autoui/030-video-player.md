# 30 — 030-video-player 系统原生视频播放器架构与设计

> **状态**：设计文档（定稿 v1）  
> **日期**：2026-09-04  
> **关联**：
> - [Design 21 / examples-app-track.md](examples-app-track.md)（AutoOS 默认应用矩阵与 App 轨道）
> - [019-video-app](../../examples/ui/019-video-app/README.md)（Web 流媒体门户，对标 Bilibili / YouTube）
> - [020-music-player](../../examples/ui/020-music-player/README.md)（系统音乐播放器）
> - [027-file-manager](../../examples/ui/027-file-manager/SPEC.md)（系统文件管理器，调用源头）
> - [029-photo-gallery](../../examples/ui/029-photo-gallery/SPEC.md)（系统相册图片浏览器）

---

## 1. 背景与产品定位

### 1.1 为什么有了 `019-video-app` 还需要 `030-video-player`？

在现有的 `examples/ui/019-video-app` 中，其产品形态对标的是 **Bilibili / YouTube / 爱奇艺** 这类 Web 流媒体门户平台：
- 具有主页 Feed 推荐、视频分类标签（Tech/Music/Design/Gaming）、点赞/投币、频道与订阅数、相关视频推荐列表等社交与门户属性；
- 它的界面重心是“**内容发现与浏览**”，播放窗口仅是详情页上的一个内嵌模块。

而在桌面操作系统（AutoOS）中，用户最基础、最核心的视频消费场景完全不同：
- 用户通常通过 **文件管理器（`027-file-manager`）** 浏览本机磁盘目录或移动存储；
- 找到视频文件（`.mp4`、`.mkv`、`.webm`、`.mov` 等）后，**双击打开**；
- 此时唤起的是一个**专注、纯粹、低干扰的原生媒体播放器**（对标 **VLC / PotPlayer / IINA / QuickTime / MPC-HC**）。

因此：
1. **保留 `019-video-app`**：作为 Web 视频平台、流媒体门户类 SaaS 应用的高质量 demo；
2. **新建 `030-video-player`**：作为 AutoOS 的默认系统级本地/网络多媒体播放器。

---

## 2. 操作系统视频播放器的核心设计哲学

1. **沉浸式无干扰（Content-First & Chromeless）**：
   - 窗口中心即视频视口，无冗余的网页边栏、无推荐流、无社交评论区；
   - 控制面板（OSD，On-Screen Display）悬浮于画面下部，鼠标静止 2 秒或移出后**自动渐隐**，鼠标移动时平滑浮现；
   - 支持全屏（F11 / F）与画中画（PiP）。
2. **专业级播控与状态反馈**：
   - **交互进度条（Scrubber）**：带悬停时间气泡预览、已缓冲范围（buffer range）、高精度拖拽寻道；
   - **精准时间显示**：当前时间 / 总时长（`00:14:32 / 01:45:00`），支持点击切换为剩余倒计时（`-01:30:28`）；
   - **传输控制**：播放/暂停、前后微调跳转（±5s / ±10s）、上一集/下一集、停止；
   - **多档倍速**：`0.5x` / `0.75x` / `1.0x` / `1.25x` / `1.5x` / `2.0x` 平滑变速；
   - **音量系统**：音量滑块（支持鼠标滚轮微调）、一键静音/取消静音；
   - **画幅比调节**：自动适应（Contain）、拉伸铺满（Fill）、裁切填满（Cover）、原始比例（16:9 / 4:3）。
3. **播放队列抽屉（Collapsible Playlist Drawer）**：
   - 右侧可折叠的播放列表抽屉，展示当前目录或队列中的视频；
   - 显示清晰的分辨率标签（`4K` / `1080P` / `720P`）与时长；
   - 支持多循环模式：顺序播放、单曲循环、列表循环、随机播放。
4. **全套标准键盘快捷键**：
   - `Space` / `K`：播放 / 暂停
   - `←` / `→`：快退 / 快进 5 秒（长按快速扫描）
   - `↑` / `↓`：音量 ± 5%
   - `F`：全屏切换
   - `M`：静音切换
   - `[` / `]`：倍速减慢 / 加快 0.25x
   - `P`：展开/折叠右侧播放队列

---

## 3. 布局与视觉架构 (ASCII Wireframe)

```
+-----------------------------------------------------------------------------------------------+
| ≡ 文件  播放  音频  字幕  帮助              AutoOS Video Player               —   □   ✕   |
+-----------------------------------------------------------------------------------------------+
|                                                                         | 播放队列 (4)   ✕   |
|                                                                         | ------------------- |
|                                                                         | ▶ 01_intro.mp4      |
|                                                                         |   1080P · 03:45     |
|                                                                         |                     |
|                                                                         |   02_architecture   |
|                                                                         |   4K · 12:20        |
|                                                                         |                     |
|                                [ 视频主画面 / 画布视口 ]                   |   03_demo_walk      |
|                              (拖拽视频至此 / 双击全屏)                    |   1080P · 08:15     |
|                                                                         |                     |
|                                                                         |   04_summary.mp4    |
|                                                                         |   720P · 01:30      |
|                                                                         |                     |
|                                                                         | [+ 打开文件/URL]     |
+-------------------------------------------------------------------------+---------------------+
|  [===========================●========================================] 04:15 / 12:20 (35%)   |
|  ⏮   ⏪ 5s   ▶ 播放   ⏩ 5s   ⏭   ⏹   🔊 [====●---] 80%   1.25x ▾    🔤 字幕   ⛶ 全屏   ≡ 列表 |
+-----------------------------------------------------------------------------------------------+
```

---

## 4. 底层播放引擎与技术方案分析

### 4.1 Rust 媒体生态与 FFmpeg
在原生桌面操作系统环境中，视频播放的底层标准通常为：
1. **`libmpv-rs` (mpv C 库的 Rust 绑定)**：
   - **优点**：业界最顶级的开源桌面播放核心（IINA、Celluloid、大量 Linux/Windows 播放器均基于此）。
   - 内置极其成熟的音视频同步时钟（A/V sync）、硬解码支持（D3D11VA、DXVA2、NVDEC、VAAPI、VideoToolbox）、libass 字幕渲染管线，并提供向 OpenGL/wgpu 帧缓冲直接渲染接口。
2. **`ffmpeg-next` (FFmpeg 的 Rust 封装)**：
   - **优点**：格式解码全面，底层控制度高。
   - **代价**：需要自行处理音视频时钟同步（音画不同步是自研播放器最容易踩的坑）、自建音频输出队列（结合 `cpal`）以及像素格式转换与 wgpu 纹理拷贝。

### 4.2 AutoUI 双端架构支持路径
AutoUI 支持 **Vue 模式（`auto run`）** 与 **VM / Iced 模式（`auto run -r vm`）**：
- **Vue 模式**：
  - 前端具备现代浏览器强大的多媒体解码引擎（硬件加速 H.264/H.265/AV1/VP9/WebM）；
  - **本地文件即时播放**：支持 HTML5 File API / Drag & Drop 拖入，前端直接通过 `URL.createObjectURL(file)` 零内存拷贝即时播放；
  - **后端 HTTP Range 流式服务**：后端 Axum 提供 `/api/stream?path=...`，返回 `HTTP 206 Partial Content`，使系统内任意路径视频均可无缝拖动进度条。
- **VM / Iced 模式**：
  - Iced 渲染视口负责呈现高精度的播放器 UI 控件、交互反馈、播放队列状态机和 OSD 动画；
  - 画面区域在阶段一提供视口模拟与状态指示，阶段二可通过跨进程/C-FFI 桥接接入原生纹理流。

---

## 5. 数据模型与状态机

### 5.1 数据结构
```auto
type VideoItem {
    id: int,
    title: str,
    path: str,
    url: str,
    duration_str: str,
    duration_sec: int,
    resolution: str,   // "4K", "1080P", "720P"
    codec: str,        // "H.264", "HEVC", "AV1"
    size_str: str,     // "142 MB"
    color: str,        // 占位预览基色
}
```

### 5.2 核心状态变量
```auto
model {
    // 基础主题与契约
    var dark_mode bool = true
    var accent_color str = "indigo"

    // 播放状态
    var is_playing bool = false
    var current_id int = 1
    var current_time_sec int = 45
    var total_time_sec int = 225
    var progress_pct int = 20
    var volume int = 80
    var is_muted bool = false
    var playback_speed str = "1.0x"
    var aspect_mode str = "contain" // "contain" | "cover" | "fill"

    // 界面与交互状态
    var show_playlist bool = true
    var show_controls bool = true
    var show_speed_menu bool = false
    var is_fullscreen bool = false
    var loop_mode str = "list" // "list" | "single" | "shuffle"
    var time_display_mode str = "elapsed" // "elapsed" | "remaining"

    // 播放列表数据
    var playlist []VideoItem = []
    var current_video ?VideoItem = None
}
```

---

## 6. 与系统文件管理器（`027-file-manager`）的集成方案

1. **URL 参数 / 路由直达**：
   - 启动时支持路由查询参数：`http://localhost:3030/?file=/media/demo.mp4`；
   - 播放器启动后自动以该文件为主播放源，并解析其所在目录的兄弟视频文件进入播放队列。
2. **拖拽桥接（Drag & Drop Bridge）**：
   - 视口监听 HTML5 拖放事件（Drop），检测拖入的媒体文件扩展名（`.mp4`, `.mkv`, `.webm`, `.avi`, `.mov`）；
   - 拖入后自动追加至队列顶端并自动起播。
3. **IPC / OS 关联协议（未来演进）**：
   - 在 AutoOS 桌面环境中注册为 `video/*` MIME 类型默认处理程序。

---

## 7. 实施路线图

1. **第一阶段（Plan 542）**：
   - 在 `examples/ui/030-video-player/` 下搭建完整的系统级播放器；
   - 提供 4~6 个预置精选短视频/动态演示流作为内建测试样本；
   - 实现完整的专业 OSD 控制台、时间轴拖拽、倍速选择、音量调节、播放队列抽屉、深浅主题与五色强调色；
   - 支持本地视频文件选择与拖拽播放；
   - 编写完整的双端自动化测试（Playwright E2E + AutoUI MCP VM 测试）。
2. **第二阶段（OS 联动）**：
   - 与 `027-file-manager` 实现联动协议，支持在文件管理器内“打开”直接激活播放器。
