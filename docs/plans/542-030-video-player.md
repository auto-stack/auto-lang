---
plan_id: PLAN-542
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 030-video-player
author: [Antigravity]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/examples-app-track.md, examples/ui/030-video-player]
current_step: 0
total_steps: 8
---

# [PLAN-542] 030-video-player — AutoOS 原生视频播放器（VLC / PotPlayer 极简形态）

## 变更摘要

在 `examples/ui/030-video-player/` 下新建 AutoOS 原生系统级视频播放器应用（对标 VLC / PotPlayer / IINA / MPC-HC），与现有的流媒体门户平台 `019-video-app`（对标 Bilibili / YouTube）形成清晰的场景区隔：
1. `019-video-app` 继续保留，专注于流媒体门户、推荐流、分类标签、互动点赞与频道体系；
2. `030-video-player` 专注于**操作系统底层媒体播放**：专注无干扰的大视口播放画布、悬浮且自动渐隐的专业级 OSD 控制条（精确寻道 Scrubber、前后 5s 跳转、多档倍速、音量与静音、画幅切换、全屏）、可折叠的右侧播放队列抽屉（带分辨率标签与时长）、本地/网络视频拖拽载入与键盘快捷键支持；
3. 支持 AutoUI 双端（Vue 浏览器模式与 VM/Iced 桌面原生模式），并在 `docs/design/autoui/examples-app-track.md` 矩阵中正式登记。

---

## 目标

1. **清晰的产品定位**：实现独立的系统级播放器 `030-video-player`，作为 AutoOS 默认文件管理器 `027-file-manager` 双击媒体文件的默认关联应用。
2. **专业级 OSD 播控体系**：
   - 交互式进度条（Scrubber）：播放进度指示、已缓冲段展示、悬停时间预览与拖拽跳转；
   - 传输控制：播放/暂停、快退 5 秒（⏪）、快进 5 秒（⏩）、上一首（⏮）、下一首（⏭）、停止（⏹）；
   - 时间计数：支持当前时间/总时长（如 `04:15 / 12:20`）及点击切换剩余时间；
   - 多档倍速调节：`0.5x` / `0.75x` / `1.0x` / `1.25x` / `1.5x` / `2.0x`；
   - 音量系统：音量滑块调节（`0%~100%`）及一键静音/还原；
   - 画幅与显示：自适应（Contain）、拉伸（Fill）、裁剪铺满（Cover）、全屏切换。
3. **播放列表抽屉（Collapsible Playlist Drawer）**：
   - 右侧可一键展开/收起的播放队列，展示当前视频列表、分辨率徽章（`4K` / `1080P` / `720P`）、时长与文件大小；
   - 支持多循环模式（列表循环、单曲循环、随机播放）。
4. **键盘快捷键体系**：
   - `Space`（播放/暂停）、`←/→`（快退/快进 5s）、`↑/↓`（音量 ±5%）、`F`（全屏）、`M`（静音）、`[`/`]`（倍速 ±0.25x）、`P`（切换播放列表）。
5. **双端构建与自动化测试验证**：
   - Vue 模式通过 Playwright E2E 自动化测试；
   - VM / Iced 模式通过 AutoUI MCP 自动化测试。

---

## 架构方案

### 1. 双端执行架构
```
+-----------------------------------------------------------------------------------------------+
| examples/ui/030-video-player/                                                                 |
|   ├── pac.at                 # 工程元数据: scene: "ui", render: "vue", front_port: 3030       |
|   ├── SPEC.md                # 行为规约与状态字典 (与 027/029 一致的单真源规约)              |
|   ├── README.md              # 运行、架构与操作说明                                           |
|   ├── src/front/                                                                              |
|   │   └── app.at             # 核心 App widget (视口 + OSD 浮层 + 播放队列抽屉 + 快捷键)     |
|   └── tests/                                                                                  |
|       ├── smoke.spec.ts      # Playwright 端到端测试 (Vue 模式)                               |
|       └── vm-smoke.mjs       # AutoUI MCP 自动化测试 (VM / Iced 模式)                         |
+-----------------------------------------------------------------------------------------------+
```

### 2. 界面与交互布局 (Wireframe)
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
|                                [ 视频主画面 / 画布视口 ]                   |                     |
|                              (拖拽视频至此 / 双击全屏)                    |   03_demo_walk      |
|                                                                         |   1080P · 08:15     |
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

## 需求分析与背景调查

1. **来源背景**：
   - 来自用户对 `examples/ui/019-video-app` 定位的深度复盘与讨论。`019` 属于流媒体内容平台（YouTube/爱奇艺形态），而桌面操作系统基础装机必备的是纯粹的媒体播放器（VLC/PotPlayer/IINA 形态）。
   - AutoOS 默认应用矩阵（[docs/design/autoui/examples-app-track.md](examples-app-track.md) §4）中，媒体播放器矩阵需要补齐专门的系统级播放器。
   - `027-file-manager`（Plan 440）与 `028-launcher`（Plan 464）、`029-photo-gallery`（Plan 537）已落地，`030-video-player` 是多媒体基础工具链闭环的关键节点。
2. **规范契约（C1~C9）**：
   - 遵从 AutoUI 双端变量名即契约（`dark_mode bool = true`、`accent_color str = "indigo"`）；
   - 单文件 App widget 高内聚形态（027/028/029 实证模式），避免过度拆分带来的跨组件通讯开销；
   - 模板内零动态拼接与零方法链调用，所有展示字段均由 handler 预计算准备好。

---

## 详细设计

### 1. 数据结构与种子视频
预置 4 项具有典型特征的媒体项目，构成开箱即用的播放体验：
```auto
type VideoItem {
    id: int,
    title: str,
    path: str,
    url: str,
    duration_str: str,
    duration_sec: int,
    resolution: str,   // "4K" | "1080P" | "720P"
    codec: str,        // "H.264" | "HEVC" | "AV1"
    size_str: str,     // "142 MB"
    cover_color: str,  // 占位预览基色渐变
}
```

### 2. 状态机模型
- **播放状态**：
  - `is_playing bool`：播放 / 暂停
  - `current_id int`：当前正在播放的视频 ID
  - `current_time_sec int`：当前播放秒数
  - `total_time_sec int`：当前视频总秒数
  - `progress_pct int`：进度条百分比（0~100）
  - `volume int`：音量（0~100）
  - `is_muted bool`：静音开关
  - `playback_speed str`：当前倍速（"1.0x"）
  - `aspect_mode str`：画幅比（"contain" | "cover" | "fill"）
  - `loop_mode str`：循环模式（"list" | "single" | "shuffle"）
- **界面控制**：
  - `show_playlist bool`：播放队列侧边栏显隐（默认 true）
  - `show_controls bool`：OSD 播控条显隐（默认 true）
  - `show_speed_menu bool`：倍速下拉菜单显隐
  - `is_fullscreen bool`：全屏状态
  - `time_display_mode str`："elapsed" | "remaining"
  - `status_toast str`：操作提示（如 "音量 85%", "倍速 1.25x", "快进 5 秒"）

### 3. 消息设计 (msg)
- 传输：`TogglePlay`, `StepForward`, `StepBackward`, `NextVideo`, `PrevVideo`, `StopPlayback`
- 寻道：`SeekTo(int)`
- 音量：`SetVolume(int)`, `ToggleMute`
- 倍速：`SetSpeed(str)`, `ToggleSpeedMenu`
- 界面：`TogglePlaylist`, `ToggleControls`, `ToggleFullscreen`, `ToggleAspect`, `CycleLoopMode`, `ToggleTimeMode`
- 队列：`SelectVideo(int)`, `AddLocalFile(str)`
- 主题：`ToggleDarkMode`, `SetAccent(str)`

---

## 测试设计

1. **Playwright E2E 测试 (`tests/smoke.spec.ts`)**：
   - 用例 1：默认初始状态与标题验证（加载默认 01_intro.mp4，时长与进度初始正确）。
   - 用例 2：播放/暂停切换（点击 Play 变为 Pause，状态及指示器更新）。
   - 用例 3：步进寻道（快进 5s，快退 5s，当前时间与进度条实时响应）。
   - 用例 4：倍速切换（从 1.0x 切换至 1.5x，界面徽章高亮）。
   - 用例 5：音量调节与静音切换（静音图标联动，音量条变更）。
   - 用例 6：播放列表抽屉交互（展开/折叠抽屉，点击切换至第二项 02_architecture）。
   - 用例 7：切集切歌联动（Next/Prev 切换，视频标题与时长同步刷新）。
   - 用例 8：主题切换（深色 🌙 ↔ 浅色 ☀，五色 accent 点击响应）。
2. **AutoUI MCP VM 模式测试 (`tests/vm-smoke.mjs`)**：
   - 驱动 VM 实例，验证窗口正常创建、控件树 snapshot 正常、点击 Play/Pause 及切换视频正常响应。

---

## 验收标准

1. `examples/ui/030-video-player/` 下全部源码创建完毕（`pac.at`, `SPEC.md`, `README.md`, `src/front/app.at`）。
2. `auto gen` 与 `auto check` 顺利通过，无编译/转译报错与死循环。
3. Vue 模式（`auto run`）下前端正常启动（`http://localhost:3030`），界面视觉呈现专业级 VLC/PotPlayer 深色极简质感。
4. VM 模式（`auto run -r vm`）下原生窗口正常渲染，OSD 控制条、进度条、播放列表抽屉排版稳健无越界。
5. Playwright E2E 8 项测试全部通过。
6. AutoUI MCP VM 测试用例顺利跑通。
7. `docs/design/autoui/examples-app-track.md` 矩阵更新，将 `030` 明确为 `030-video-player` 系统视频播放器。

---

## 执行步骤

### Task 1: 创建基础工程规约与配置文件
- 文件：`examples/ui/030-video-player/pac.at`, `examples/ui/030-video-player/SPEC.md`
- 操作：定义工程元信息（`scene: "ui"`, `render: "vue"`, `front_port: 3030`），书写完整的状态字典、数据结构与交互规约。
- 验证：文件存在且语法格式正确。

### Task 2: 编写核心播放器 Widget (`src/front/app.at`)
- 文件：`examples/ui/030-video-player/src/front/app.at`
- 操作：实现单文件 App widget，包含视频画布视口、OSD 悬浮播控条、交互进度条、倍速菜单、音量滑块、可折叠播放列表抽屉、主题切换与快捷键。
- 验证：`cargo run -p auto -- gen examples/ui/030-video-player`

### Task 3: 编写应用说明文档
- 文件：`examples/ui/030-video-player/README.md`
- 操作：记录特性列表、架构说明、Vue 与 VM 双端运行方式、测试指引。
- 验证：文件存在且包含完整的运行说明。

### Task 4: 双端生成与类型/语法检查
- 命令：`cargo run -p auto -- check examples/ui/030-video-player` 与 `cargo check -p auto-lang`
- 操作：确认 codegen 输出的 Vue SFC 与 TS 代码语法完备，无未解决引用。
- 验证：检查通过无报错。

### Task 5: 编写与运行 Playwright 端到端测试
- 文件：`examples/ui/030-video-player/tests/smoke.spec.ts`
- 操作：覆盖 8 个关键用例（播放/暂停、快进快退、倍速、音量/静音、切换视频、列表展开折叠、主题切换）。
- 验证：`pnpm exec playwright test` 测试通过。

### Task 6: 编写与运行 AutoUI MCP VM 测试
- 文件：`examples/ui/030-video-player/tests/vm-smoke.mjs`
- 操作：编写基于 AutoUI MCP 协议的 VM 模式自动化测试脚本。
- 验证：`node tests/vm-smoke.mjs` 测试通过。

### Task 7: 更新应用示例轨道设计文档
- 文件：`docs/design/autoui/examples-app-track.md`
- 操作：在 §4 默认应用矩阵与 §5 填洞路线中登记 `030-video-player`。
- 验证：文档表格更新并对齐。

### Task 8: 复审与状态流转
- 文件：`docs/plans/542-030-video-player.md`
- 操作：核对所有任务完成证据，生成变更复审记录，流转至 `reviewed`。
- 验证：计划内全部复审项通过。

---

## 复审记录

（待执行完成后按 /auto-plan:review 填写）

---

## 待澄清事项

无（需求、设计方案与技术路径已通过前序讨论完全对齐）。
