# 30 — 030-video-player 系统原生视频播放器架构与设计

> **状态**：设计文档（v1 定稿；**§4 与 §7 已于 2026-09-12 按 T-15 实测重写**）  
> **日期**：2026-09-04（§4/§7 修订 2026-09-12）  
> **关联**：
> - [PLAN-617](../plans/617-030-video-player-real-rebuild.md)（真实化重做；T-15 门控结论见本文 §4）
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

> **本节已于 2026-09-12 由 [PLAN-617](../plans/617-030-video-player-real-rebuild.md) T-15 门控
> spike 依据实测重写**（原 §4 是未实测的调研性描述，其中「`libmpv-rs` 提供向 OpenGL/wgpu
> 帧缓冲直接渲染接口」一句**不成立**——mpv 的 render API 只有 `opengl` 与 `sw` 两个后端，
> 没有 wgpu/Vulkan 后端）。§4.1 为结论，§4.2 为支撑它的实测数字，§4.3–§4.4 为被排除的路径。

### 4.1 裁定：SW（libmpv 软件渲染后端 + 持久 staging buffer）

| 候选 | 裁定 | 一句话理由 |
|---|---|---|
| **SW**（`MPV_RENDER_API_TYPE_SW` → 自有 `wgpu::Texture`） | ✅ **采用** | 实测两端 4K 实时（时钟 0.997×）且每帧上屏代价 ~0.43 ms，端到端 ~7.3 ms/帧，仅占 24 fps 预算的 17% |
| GL（`MPV_RENDER_API_TYPE_OPENGL` → FBO → wgpu 纹理） | ❌ 排除 | wgpu 无公开的外部内存导入 API；要做只能落到 `wgpu-hal` 乃至裸 Vulkan(`VK_KHR_external_memory_win32`) + 裸 GL(`GL_EXT_memory_object_win32`) 的 unsafe 驱动相关代码，**且做完仍慢于 SW** |
| sidecar（`ffmpeg-sidecar` 子进程 + stdout 管道） | ❌ 排除 | 被 SW 全面支配：多一次跨进程拷贝（4K 33 MB/帧）、A/V 同步与音频输出要自研、增益为零 |
| 放弃（维持今日诚实降级） | ❌ 不采用 | 前提已具备（见 §4.2），没有放弃的理由 |

**通道形状（T-17 据此实现）**：`mpv` 的 SW renderer **直接写进一个持久映射的
`wgpu::Buffer`**（`MPV_RENDER_PARAM_SW_POINTER` 指向映射区），再 `copy_buffer_to_texture`
到一张持久 `wgpu::Texture`。于是「解出的帧 → 屏幕上屏」只剩**一次设备侧拷贝**：

```
mpv (解封装/解码/A-V 同步/色调映射)  ──写──▶  持久映射 staging buffer  ──copy──▶  持久 wgpu Texture
```

三处关键约束（均为硬约束）：
1. **绝不用 iced 的 `image::Handle` 通道**（理由见 §4.5）；
2. 帧必须进 `wgpu::Texture`，**纹理建一次**，绝不每帧新建（实测每帧新建在 4K 下 p95 恶化
   2.5×、离群帧 8/120 —— 这正是 `ui/iced/renderer.rs:2889-2898` 记录的闪烁机理）；
3. 纹理/staging 由自实现的 iced **自定义 shader widget**持有（`iced_widget::shader::Program`
   的 `Pipeline::new(device, queue, format)` 恰好交出 `&wgpu::Device`/`&wgpu::Queue`），
   从而完全绕开 iced 的图像缓存与 atlas。

### 4.2 门控实测数字（AC-18：以下全部为实测，不是估算）

环境：Windows 11 / **NVIDIA GeForce RTX 4060 Ti** / iced 实选 **Vulkan**；
libmpv = `mpv-dev-x86_64-20260903-git-69e63f425a`（`libmpv-2.dll`，120 MB，ffmpeg 静态内链）；
样本 = `Loki.S01E01…1080p.x264.mp4` 与 `Loki.S02E01…2160p.HEVC.HDR.mkv`；
spike 代码 = `crates/auto-lang/examples/mpv_spike.rs`，证据日志 `t15_gate_debug.log`。

**门控 A：libmpv 运行时加载（`libloading`）+ SW 逐帧渲染**（`mpv_render_context_render()` 的
per-frame 代价 = mpv 的缩放/色彩转换/写入我们缓冲的时间；`ADVANCED_CONTROL` +
`mpv_render_context_update()` 生产形状，untimed 求吞吐上界）

| 目标分辨率 | `render()` p50 | p95 | 吞吐上界 | 每帧 RGBA |
|---|---|---|---|---|
| 1080p（H.264） | **0.60–0.68 ms** | 0.74–0.95 ms | 869–989 fps | 8.29 MB |
| 4K（HEVC HDR） | **6.74–6.88 ms** | 10.36–10.71 ms | 90–93 fps | 33.18 MB |

**门控 B：帧上屏每帧代价**（按 60 Hz 节奏节流、预热 10 帧后取样 120 帧）

| 通道 | 1080p p50 / p95 / 离群 | 4K p50 / p95 / 离群 |
|---|---|---|
| **A** 持久纹理 + `queue.write_texture` | 0.67–1.02 / 0.97–1.37 ms / 0 | 3.87–4.17 / 6.91–7.66 ms / 0 |
| **B** 每帧新建纹理（= iced `Handle` 等价） | 1.22–1.27 / 1.98–2.26 ms / 2 | 4.28–4.36 / 18.93–19.49 ms / **8** |
| **C** 持久 staging + `copy_buffer_to_texture`（**采用**） | **0.28–0.29 / 0.76–0.82 ms** / 3 | **0.41–0.44 / 0.86–0.98 ms** / 1 |

**`--release` 复核**（排除「debug 档把 wgpu 代价算高了」这一质疑；同一台机同一命令）：
门控 A 1080p p50 0.66 ms / 4K 6.67 ms（与 debug 一致），门控 B 通道 C **0.132 ms @1080p /
0.244 ms @4K**、`map_async+unmap` **0.043 ms**——**release 只会更好，排序不变**（本管道
的重活都在预编译的 libmpv DLL 与 GPU 驱动里，不在我们的 Rust 代码里）。但**通道 C 的
长尾在 release 下同样出现**（4K max 951 ms）⇒ 它是真实行为，不是 debug 产物，
T-17 必须处置（§4.6）。

`map_async` + `unmap` 一次往返 **0.04–0.14 ms**（环形缓冲换页的固定成本，可被 3 槽环掩盖）。

**实时播放能力**（Python ctypes 探针，同一下载构件的 `libmpv-2.dll`，`ao=null` 给出音频时钟；
与门控 A 同为真实文件、真实解码）

| 场景 | 帧数/时长 | 媒体时钟 | RSS | CPU |
|---|---|---|---|---|
| 1080p H.264 实时 | 233 / 10.0 s（23.3 fps，内容 24 fps） | 0.958× | 150 MB | 5.6 ms/帧 |
| 4K HEVC HDR 实时（`d3d11va-copy` 硬解） | 236 / 10.0 s（23.6 fps，内容 24 fps） | **0.997×** | 493 MB | 54.6 ms/帧 |
| 4K HEVC 纯软解（`hwdec=no`） | 1079 / 8.0 s | — | 1042 MB | — |

> 注：**SW render API 下 `hwdec=d3d11va`（零拷贝）不生效**（实测 `hwdec-current: no`）——
> 软件后端要求帧在系统内存里，故只能用 `d3d11va-copy`（解码后回读）。这不影响判定：
> 4K 24 fps 下即便软解也有 90 fps 上界，实时余量 3.7×。
> **数字对机器负载敏感**：上表取自相对空闲的时段；在有并发负载时复测同一 4K 场景，
> `render()` p50 由 5.7 ms 升到 12.8 ms、CPU 由 54.6 升到 97.5 ms/帧，**而媒体时钟仍为
> 0.999×**——即「能不能实时」这个判定对负载有相当余量（这也是 Go 的一部分依据）。

**音频：Vue 端解不了的那条轨，在 VM 侧实测可解**（这是用户要求 VM 播放的主要动机）。
单独探针实测 `Loki.S02E01…mkv`（`ao=null`，只读 mpv 自报属性）：

```
audio-codec-name : eac3
audio-params     : {"samplerate":48000,"channel-count":6,"channels":"5.1(side)","format":"floatp"}
audio-bitrate    : 768000          aid: 1        AUDIO_RECONFIG 事件: 3 次
```

即 **Chromium 判为「video-only」的 Dolby Digital Plus 音轨，libmpv 内链的 ffmpeg 正常解码**；
**Atmos 对象元数据如预期坍缩为 5.1**（已实测确认，不是推断）。HDR 色调映射亦由 mpv 承担
（质量未单独实测，T-18 验收时看画面）。音频输出用 mpv 自带的 `ao`，**T-18 不引入 `cpal`**。

**端到端每帧预算**（4K）＝门控 A `render()` 6.8 ms ＋ 门控 B 通道 C 0.43 ms ≈ **7.3 ms**
→ 24 fps（41.7 ms 预算）占 **17%**；60 fps（16.7 ms 预算）占 **44%**。

### 4.3 为什么不是 GL（FBO → `wgpu::Texture` 导出）

调研结论是**在本仓架构下不可达**，而非「难」：

1. mpv 的 GL 后端要求我们提供一个**当前生效的 OpenGL 上下文**
   （`MPV_RENDER_PARAM_OPENGL_INIT_PARAMS` 含 `get_proc_address`），而 iced 在本机走的是
   **Vulkan**——两套图形 API 之间要共享纹理，必须走外部内存互操作
   （Vulkan 侧 `VK_KHR_external_memory_win32`，GL 侧 `GL_EXT_memory_object_win32`）；
2. **wgpu 不公开外部内存导入**：`wgpu::Device` 只有 `create_texture_from_hal` /
   `Texture::as_hal` 这类 **`wgpu-hal` 层的 unsafe 接口**（要拿到 hal 的 `vk::Image` 得自己
   用 `ash` 建），没有面向应用的 external-memory API；
3. 即便全部写通，它还要多一趟「GL 上下文渲染 → 外部内存 → Vulkan 导入」，
   **不见得比已实测的 SW 通道（0.43 ms/帧）快**，却引入了驱动相关性与大量 unsafe 代码。

先例 `ophymx/mpv-engine` 的 GL 路径成立，是因为它的宿主就是 GL（不是 wgpu-first）。
故 GL 路径列为**已评估并排除**，不留待办。

### 4.4 为什么不是 sidecar

`ffmpeg-sidecar` 走 `ffmpeg.exe` + stdout 管道拿 rawvideo：每帧多一次**跨进程**拷贝
（4K 33 MB/帧，且 pipe 无法零拷贝），并且 A/V 同步、音频输出、HDR 色调映射要自研
（§4.1 原描述里 ffmpeg 不提供的正是这三件事）。libmpv 已把这些全部封装好，故 sidecar
**被 SW 通道全面支配**，只在前者被证伪时才作为兜底——而它没有被证伪。

### 4.5 帧上屏为什么必须绕开 iced 的图像通道（三条源码级证据）

这是 T-15 最重要的架构发现。**「复用 `Handle` 以消除闪烁」在本 API 下不可表达**：

1. **`Id` 无法复用**——`iced_core::image::Handle::from_rgba` 每次都 `Id::unique()`，而
   `Id` 的构造器（`Id::unique` / `Id::path`）与元组字段**全是私有的**
   （`iced_core-0.14.0/src/image.rs:209-228`），仓外无法构造「同 id、不同像素」的 Handle。
   于是只剩两条路，**都不通**：每帧新 id（= 缓存永远未命中），或同 id（= 永远显示陈旧帧，
   因为 `cache::load_image` 命中即**直接返回、不再上传**，`image/cache.rs:94-101`）。
2. **大帧必然走异步慢路径**——`upload_raster` 对 `image.len() >= MAX_SYNC_SIZE (2 MB)`
   的图像走 **worker 线程上传**（`cache.rs:229-267`），而真实的视频帧是 1080p **8.29 MB** /
   4K **33.18 MB**，**全部超过阈值** ⇒ 每帧都要经历「异步上传 + 至少一帧延迟」的节奏，
   这正是 `ui/iced/renderer.rs:2889-2898` 记录的闪烁机理。
3. **4K 帧连 atlas 都装不下**——atlas `DEFAULT_SIZE = MAX_SIZE = 2048`
   （`iced_wgpu-0.14.0/src/image/atlas.rs:13-14`），3840×2160 的帧必须**碎片化跨多个 layer**，
   每帧「分配 → 分片上传 → 释放」的扰动远大于一张静态图片。

**旁证（本仓已有先例，但只对静态图成立）**：`renderer.rs:2889` 的 `cached_media_handle`
（P547 闪烁修复）靠「asset id + **像素 `Arc` 指针同一性**」复用 Handle——视频帧的像素
`Arc` 每帧都是新的，**该缓存对帧流必然不命中**，所以那条修复不能直接搬到视频上。

结论：`video` 的帧通道必须是**与 iced 图像系统无关的自有纹理**。这同时解释了为何
AC-19 的「无闪烁」是可判定的：一旦不再经过 Handle/atlas，闪烁的成因就被结构性移除。

### 4.6 遗留风险与 T-17 的约束

| 风险 | 实测依据 | T-17 的处置 |
|---|---|---|
| ~~通道 C 有极稀有的长尾~~ **已在 T-17 处置**（4K 120 帧里 1 帧 957 ms、1080p 里 3 帧 242 ms；release 档同样出现 ⇒ 非 debug 产物） | 门控 B 离群计数 | **已落地**：**3 槽 staging 环**（避开「映射中的缓冲不可提交」）+ **回收超时即丢帧**。T-17 在正式通道里复现了该长尾（4K 30 帧中 1 帧 **329–336 ms**）并被环吸收（0 丢帧、0 空白帧）。见 §4.9 |
| 通道 A/B 在**有并发负载时尾部急剧恶化**（release 构建占着 GPU 时实测：A 4K p95 43 ms/12 离群、B 4K p95 217 ms/16 离群；C 仍 p95 0.76 ms） | 同机对照运行 | 进一步支持选 C；但说明上屏通道对 GPU 争用敏感，T-17 需在真实负载下复测 |
| SW 后端**用不了零拷贝硬解**（只能 `d3d11va-copy`），4K 软解内存可达 ~1 GB | 门控 A（`hwdec-current: no`）与 Python RSS 表 | 目标分辨率下调（4K 降采样到视口尺寸再上屏，`SW_SIZE` 即视口尺寸）可显著降低 `render()` 与内存 |
| **本机无 mpv/libmpv**，CI 更不能依赖 | 取得构件前实测 | 运行库解析序（`AUTO_MPV_LIB` → exe 同目录 → 系统路径）+ 缺失即降级（`mpv_spike selfcheck` 已证 `Library::new` 返回 `Err` 而非 abort） |
| 该特性会**让 lib 的 `--test` 目标触发 rustc ICE** | 见 §4.7 | spike 与后续测试一律落在 example/integration 目标 |

### 4.7 一个必须记下的构建期发现（rustc ICE）

给 `auto-lang` 打开 `mpv-spike` 特性后编译 **lib 的 `--test` 目标**会撞 rustc 1.98.0 的
ICE（`collect_and_partition_mono_items`，查询栈指向 `ui/mcp_server.rs:1718` 的 iterator
chain，**与 spike 代码无关**）；同一特性下 `--lib`（rlib）与 `example` 目标均正常。
故 spike 落在 `[[example]] name = "mpv_spike" required-features = ["mpv-spike"]`，
默认档与 CI 完全不编译它（AC-20）。**T-16..T-20 的测试须照此避开 lib 测试目标**。

### 4.8 libmpv 的取得与分发

本机无任何 mpv/libmpv（实测）。spike 用的是
[`shinchiro/mpv-winbuild-cmake`](https://github.com/shinchiro/mpv-winbuild-cmake) 的
`mpv-dev-x86_64-20260903-git-69e63f425a.7z`（31 MB，解出 `libmpv-2.dll` + `include/` +
`libmpv.dll.a`；sha256 `fac135c6…80faef`），**不入库**——它是运行时依赖，不是构建期依赖，
故 CI 无需安装任何系统媒体包（AC-20）。分发形态与许可影响：libmpv 为 **LGPLv2.1+**
（本次构件的 ffmpeg 为静态内链，若未来跟随发行版分发需按 LGPL 提供重链接能力）——
**本计划不分发 DLL**，只定义解析序与降级行为，故该问题留给真正要做发行的那件事。

### 4.9 T-17 落地：帧上屏通道（实测数字与长尾复现）

通道已按 §4.1 的形状实现（`crates/auto-lang/src/ui/mpv/channel.rs` + `present.rs`，
feature `mpv-gpu`；引擎部分 `mpv-native` 单独门控，不碰 GPU）：

```
mpv SW renderer ──直接写──▶ 持久映射 staging（3 槽环）──copy_buffer_to_texture──▶ 持久 wgpu 纹理 ──全屏 blit──▶ 目标
```

**实现里三条值得记住的结论**：

1. **行距取 256 对齐**，一个 stride 同时满足两边：wgpu 的
   `copy_buffer_to_texture` 要求 `bytes_per_row` 是 `COPY_BYTES_PER_ROW_ALIGNMENT`
   (256) 的倍数，mpv 要求 64 的倍数（`render.h:393-404`）——256 是 64 的倍数。
   实测常用宽度（1920/2560/3840 等）本来就落在 256 上，故无 padding 开销。
2. **片元着色器强制 `alpha = 1.0`**：mpv 的 `"rgb0"` 第 4 字节是**未初始化垃圾**
   （`render.h` 原文 "the '0' component contains uninitialized garbage"）。若把它当
   alpha 用，画面会随机变半透明甚至整帧「消失」——**那就是闪烁**本身。
3. **背压一律丢帧，不阻塞**：取不到空闲槽时定向回收（`poll` 只等**该槽**的
   submission，不串行化整个 GPU），回收带超时（默认 4 ms），**超时即丢帧**。
   视频场景里丢一帧远好于冻结界面，且纹理始终保留上一帧内容，故丢帧**不会**
   产生空白。

**实测（`tests/mpv_channel.rs`，真实 libmpv + 真实片源 + 离屏读回）**：

| 通道尺寸 | 源 | 端到端帧率（含逐帧读回/上屏） | 上屏代价 p50 | p95 | max（离群） | 丢帧 |
|---|---|---|---|---|---|---|
| 1920×1080 | `caelestia.mp4` | **258–385 fps** | **0.61–0.67 ms** | 0.85–0.98 ms | 6.8–55.4 ms（2/60） | 0 |
| 3840×2160 | `Loki.S02E01…4K HEVC HDR` | **38.3–38.6 fps** | **4.35–4.54 ms** | 5.57–5.65 ms | **329–336 ms（1/30）** | 0 |

**那条长尾在正式通道里复现了**：4K 下 30 帧中 1 帧 `copy_buffer_to_texture` 耗时
**329–336 ms**（T-15 门控 B 量到的是 957 ms，同一现象）。这说明 §4.6 记录的
长尾不是 spike 的测量假象。**而 3 槽环把它吸收掉了**——`reclaim_timeout=0`、
无丢帧、无空白帧，表现只是那一轮的总时长从 0.4 s 变成 0.78 s。这正是选 3 槽
（一槽在被 mpv 写、一槽在 GPU 拷贝、一槽空闲可取）的意义。

**「无闪烁」的可证伪判定**（写在 `tests/mpv_channel.rs` 里，不是靠眼看）：
逐帧上屏后读回像素，断言 ① 画面确实上了屏（非全黑）；② **没有「内容帧之间夹
空白帧」**——这正是 `renderer.rs:2889` 描述的「画面消失又出现」的闪烁签名；
③ 帧签名不恒定（纹理在更新，不是命中缓存后的陈旧帧）；④ 全程
`textures_created() == 1`（**硬约束：纹理只建一次**，这条是结构性反闪烁）。

端到端帧率含逐帧读回（真实播放器不会做），故是**保守下界**；对照 24 fps 片源，
1080p 有 10× 以上、4K 有 1.6× 余量。

### 4.10 T-18：§2.3 受控媒体契约在 mpv 侧的落地

`crates/auto-lang/src/ui/mpv/contract.rs` 把 §2.3 的契约实现到 mpv 上
（`mpv-native` 门控，不需要 GPU）。**共享边界取「作者面的状态值」**：`volume` 是
**0..100**（Vue 侧生成器翻成元素的 0..1；mpv 的 `volume` 本就是 0..100，故 VM 侧恒等），
`position`/时长是 float 秒，`rate` 是倍速 float。换算函数留在 `contract.rs` 作单一出处。

属性映射（全部走 mpv 的**属性**接口，读写对称）：

| 契约字段 | mpv 属性 | 说明 |
|---|---|---|
| `paused` | `pause`（flag） | 作者写 `paused: .is_playing == false` |
| `position` | `time-pos`（double） | 秒；写它就是绝对 seek |
| `volume` | `volume`（double） | 0..100，与契约同单位 |
| `muted` | `mute`（flag） | |
| `rate` | `speed`（double） | `<= 0` 一律忽略（0 倍速会冻住播放） |
| 上行 `ontimeupdate` | `time-pos` | 变化超 0.25s 才回灌（流量而非精度考虑） |
| 上行 `onloadedmetadata` | `duration` | 首次可知时**恰好发一次** |
| 上行 `onplaystatechange` | `pause` | **合成**事件（§2.3 明写它不是原生 DOM 事件），边缘触发 |
| 上行 `onended` / `onmediaerror` | `END_FILE` 事件的 `reason` | `EOF` → Ended；`ERROR` → MediaError |

**`src` 连数据通路也不需要分叉**：mpv 自带网络栈，`loadfile` 既收本地路径也收
`http(s)://`，故后端给 Vue 的 `/api/media/stream/<id>` 在 VM 侧原样可用。

**三处必须写对的细节**（都在模块文档与测试里）：① `position` 是「目标变化时 seek」，
不是「与当前位置不同就 seek」——后者会让播放在前进时每帧触发 seek 从而把播放钉死在
目标点；② 下行必须差量（视图每帧重建）；③ 换片与 seek 让 `generation` 前进，
供 T-17 的 `VideoLatestWins` 作废在途旧帧。

**实测**（真实 libmpv，10/10 绿；缺库整体 SKIP）：seek 19.77s → 落点 **19.77s**；
起播后时间前进、暂停后 0.6s 内不动；`volume/mute/speed` 读回校验；`rate=0` 被忽略、
`volume=300` 夹到 100；`LoadedMetadata` 恰好一次且等于 mpv 的 `duration`；EOF 回灌 `Ended`
而不误报错误；音频实测 **`current-ao=wasapi`、`codec=aac`、48k/stereo**，且全仓
`Cargo.toml` **无任何音频输出依赖**——**音频确实是 mpv 的职责，没有引入 cpal**。

**一处 T-16 遗留缺陷（已修，记录以免重犯）**：`wait_event()` 原先只判「指针是否为 NULL」，
但 **mpv 超时返回的是有效指针 + `MPV_EVENT_NONE`**，故 `while let Some(ev) = wait_event(..)`
形式的抽取循环会死循环。T-16 的 `wait_first_frame_event` 恰好有 20s 截止兜底所以没显形，
T-18 的 `poll()` 一上来就把它触发了。**教训**：「当时测试通过」不等于「没有缺陷」。

**一处限制（登记为债务 P617-D8）**：mpv 在 `END_FILE/ERROR` 上不总是给错误码（实测
`error == 0`，`mpv_error_string(0)` 得到「success」）。现以「哪个源加载失败」为主信息；
要拿到精确原因需捕获 mpv 日志（`mpv_request_log_messages`）。

### 4.11 T-19：`video` 提升为可用（接线形状与「差一步」的边界）

`video` 的 iced 渲染面落在 `crates/auto-lang/src/ui/mpv/widget.rs`，feature
**`mpv-widget`**（= `mpv-gpu` + `ui-iced`）。它把 §4.9/§4.10 的两段接成一个
**自定义 shader widget**：

```text
Program::draw      → Primitive{ id, 目标尺寸, 本帧下行值 }     （纯数据 → Send+Sync 自然成立）
Primitive::prepare → [thread_local 取引擎] apply 下行 / poll 上行 → channel.with_frame
Primitive::draw    → 用管线里那张持久纹理画满 bounds
```

**引擎为什么住 thread-local**：`Primitive`/`Pipeline` 在 native 上要求
`Send + Sync`，而 `MpvEngine` 含裸指针、**刻意** `!Send`（`mpv_wait_event` 只允许
一个线程调用）。故引擎不进 Primitive/Pipeline，而放进由 widget id 索引的线程本地
注册表——widget 与渲染都在 iced 主线程，正是引擎被创建与使用的线程。
**万一被换线程调用，表现是「查不到运行时 → 不画」，降级而非 UB。**

接线三层：① `View` 新增 `Video` 变体（与 `ImageSurface` 同形，**不携带消息**——
上行由渲染面按帧采集，故各后端消息重映射臂平凡）；② `aura_view_builder` 的
tracked/untracked 两条 dispatch 各有 `video` 臂；③ iced renderer 的 `View::Video` 臂
（有 feature 走 widget，无 feature 渲染**诚实降级面板**而非黑屏）。

配套状态同步：`schema/aura.at` 的 `iced: fallback → partial` 且 **`props` 由 `[]`
改为声明**（打开 S001 校验；此前 typo 静默通过）；`render_support.rs` 记 partial 并
写明三项限制；`element_coverage.rs` 的 `video` **仍为 not-yet**——那张表描述的是
**queue/投影臂**（`video` 不在 `Coverage::target_set`），理由改成「差投影不差渲染」。

**帧由谁驱动**：iced 只在有重绘时 `prepare`，而视频要持续出帧。widget **不自带定时器**，
复用应用既有的 tick（`tick_interval_ms()` → `iced::time::every`）。
**应用不声明 tick，视频就停在首帧**——这条写进了模块文档与规范注记。

**AC-19 还差的那一步**：`crates/auto` 目前没有 `mpv-widget` 特性透传，故默认
`auto run -r vm` 看到的是降级面板。要在 VM 窗口里真看到画面在动，需补该透传
（T-20 的特性门控）并让应用声明 tick。**通道就位 ≠ 默认可用**，这一点在计划 §9.18
里写清楚了，不以「已达成」自居。

### 4.12 T-20：实机收口（AC-19 达成）与它抓到的三个真缺陷

`video` 现在**在 VM 窗口里真的会动**：验证语料 `test/ui/plan617_video_vm`
（`video` + `paused: false` + `position: 8.0` + `timer { FrameTick (every_ms: 16) }`）
实机 1080p 播放 `caelestia.mp4`，seek 生效（`time-pos` 由 8 递增到 12.63s）。
启用方式一行：`cargo build -p auto --features mpv`（默认不开，理由见下）。

**三个只有实机才能发现的缺陷**（`cargo t` + `docs_gen` + `video_contract` 三套门禁全绿却依然存在）：

1. **`convert_view_messages` 的 `_ => Empty` 兜底静默吃掉了 `video` 节点。**
   VM 动态路径是 `View<DynamicMessage>` → `convert_view_messages` →
   `View<IcedMessage>` → `into_iced`。T-19 只加了 `map_msg_with_arc` 与 `into_iced` 的臂，
   漏了这一处 ⇒ 播放面在 VM 里恒为 `Empty`。而 **MCP 快照走 `vnode_converter` 另一条路**，
   快照里 `[Video] caelestia.mp4` 看起来「节点就在树里」——**假绿**。
   该函数的注释里已记 Grid/MouseArea/select 三次同类坑，这是第四次（见债务 P617-D10）。
2. **widget 从没建 mpv render context** ⇒ `has_new_frame()` 恒假、渲染恒失败，
   而 mpv 侧一切正常（time-pos 在走、duration 已解析）。正是 `render.h:111`
   「先建 context 再 loadfile」的次序要求被违反。
3. **忘了 `channel.advance(gen, seq)`** ⇒ 通道的 `VideoLatestWins` 每帧判 `DroppedStale`，
   纹理停在初始全零，表现为「mpv 在播、纹理全黑」。

**教训（已登记为债务 P617-D11）**：涉及「新 `View` 变体要一路走到渲染」的改动，
**必须有一次真起窗看画面的验证**——编译通过 + 契约单测不能替代它。
本轮三个缺陷全部落在那条缝里。

**feature 门控**：`mpv-native`/`mpv-gpu`/`mpv-widget` 三层（`auto` 侧有同名透传与 `mpv` 别名），
**默认不开**——打开后任何带 `video` 的示例都会真解码并打开音频设备
（对 019-video-app 这类示例是行为变化）。**零构建期原生依赖**（运行时 `LoadLibrary`），
故 11 个 `ubuntu-latest` CI 任务**不需要任何系统媒体包**（实测 `grep` 零命中、
`cargo build -p auto` 本机 1m21s 通过）。

### 4.13 尚未接上的那一段

**VM 链已全部收口**（T-15..T-20）：`video` 在 VM 端真实播放已实机验证（见 §4.12）。
仍未接上的是 **Vue 链**（示例侧 T-03/T-04 的视口与播控条、T-07/T-08 的受控契约生成器与
真实播放接线、T-09..T-14），以及「默认可用」——native 播放**刻意是显式开启的 feature**
（`--features mpv`），默认走诚实降级面板。

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
3. **第三阶段（真实化重做，PLAN-617）**：
   - Vue 端转真实：递归扫描 `E:\Video` 成队列 + 后端 Range/206 字节流 + `video` 受控媒体契约；
   - **VM/iced 端原生播放**：路径已由 T-15 门控 spike 裁定为 **SW**（libmpv DLL 运行时加载
     + 持久 staging buffer → 自有 `wgpu::Texture`，见 §4）。**Go** 之后按 T-16（加载器与
     降级）→ T-17（帧上屏通道）→ T-18（受控契约对齐）→ T-19（`video` 支持级别提升）→
     T-20（特性门控与 CI 保真）实施；**No-Go 时**维持诚实降级并登记债务——按 §4.2 的实测，
     本次判定为 **Go**。
