# 030-video-player — AutoOS 原生系统视频播放器

AutoOS 默认原生视频播放器（对标 **VLC / PotPlayer / IINA / MPC-HC**）。同一份 AutoUI 源码（`src/front/app.at`）支持 **Vue 模式**（`auto run`）与 **VM / Iced 模式**（`auto run -r vm`）。

与流媒体门户应用（`019-video-app`，对标 Bilibili / YouTube）形成清晰的场景区隔：`030-video-player` 专注操作系统本地与网络媒体文件的沉浸式无干扰播放。

---

## 核心特性

- **大视口沉浸式画布 (Immersive Viewport)**：
  - 纯粹无边框视频播放视口，居中大播放状态指示器；
  - 顶部 HUD 悬浮信息（标题、分辨率 `4K` / `1080P`、编码 `HEVC` / `H.264` / `AV1`、帧率 `60 FPS`、码率）；
  - 实时操作反馈气泡（Toast，如“快进 +5 秒”、“音量 85%”、“倍速 1.5x”）。
- **专业级 OSD 悬浮播控条 (On-Screen Display Controls)**：
  - **交互式进度条 (Timeline Scrubber)**：缓冲条指示、当前时间与总时长（`00:45 / 03:45`）、点击总时长切换为剩余时间（`-03:00`）、快速跳转锚点（`0%` / `25%` / `50%` / `75%`）；
  - **传输控制**：上一首（`⏮`）、快退 5 秒（`⏪ 5s`）、主播放/暂停（`▶ 播放` / `⏸ 暂停`）、快进 5 秒（`5s ⏩`）、下一首（`⏭`）、停止（`⏹`）；
  - **音量调节**：静音切换（`🔊` / `🔇`）、音量加减微调（`-` / `+`）、音量数值显示；
  - **多档倍速**：`0.5x` / `0.75x` / `1.0x` / `1.25x` / `1.5x` / `2.0x` 平滑切换；
  - **画幅比调节**：`16:9` / `4:3` / `铺满`；
  - **循环模式**：`🔁 列表循环` / `🔂 单曲循环` / `🔀 随机播放`；
  - **OSD 显隐控制与全屏切换**。
- **可折叠右侧播放队列抽屉 (Collapsible Playlist Drawer)**：
  - 一键展开/收起播放队列（`≡ 列表 (5)`）；
  - 5 项预置种子视频（4K/1080P/720P），展示视频缩略图、标题、时长、编码与大小；
  - 高亮当前播放项，点击任意队列项立即切换起播；
  - 底部“+ 打开本地视频文件 (Open File...)”按钮（模拟联动 `027-file-manager`）。
- **媒体详细属性弹窗 (Media Properties Modal)**：
  - 查阅当前视频的容器格式、分辨率、视频编码、音频声道、码率与解码管线信息。
- **双端主题与风格契约**：
  - 默认 AutoOS Dark 深色沉浸模式，支持运行时一键切换浅色模式（`🌙 / ☀`）；
  - 5 种系统强调色（`indigo`、`coral`、`ocean`、`sage`、`amber`）实时切换。
- **全套键盘快捷键支持**：
  - `Space` / `K`：播放 / 暂停
  - `←` / `→`：快退 / 快进 5 秒
  - `↑` / `↓`：音量 ± 5%
  - `M`：静音切换
  - `F`：全屏切换
  - `P`：展开/折叠右侧播放队列
  - `[` / `]`：倍速减慢 / 加快

---

## 目录架构

```
examples/ui/030-video-player/
├── pac.at                 # 工程元配置 (scene: "ui", render: "vue", front_port: 3030)
├── SPEC.md                # 单真源规约：数据结构、状态字典与交互逻辑
├── README.md              # 本说明文件
├── src/
│   └── front/
│       └── app.at         # 单文件核心 App widget (视口 + OSD + 抽屉 + 快捷键)
└── tests/
    ├── smoke.spec.ts      # Playwright 端到端测试 (Vue 模式)
    └── vm-smoke.mjs       # AutoUI MCP 自动化测试 (VM / Iced 模式)
```

---

## 运行方式

### 1. Vue 模式（默认浏览器模式）

```bash
cd examples/ui/030-video-player
auto run
```

浏览器访问 `http://localhost:3030`。

### 2. VM / Iced 模式（原生桌面窗口）

```bash
cd examples/ui/030-video-player
auto run -r vm
```

---

## 测试验证

```bash
cd examples/ui/030-video-player

# 1. 运行 Playwright E2E 测试 (需先执行 auto run 或 auto gen)
pnpm exec playwright test

# 2. 运行 VM 模式 MCP 自动化测试 (需在 auto run -r vm 运行下)
node tests/vm-smoke.mjs
```
