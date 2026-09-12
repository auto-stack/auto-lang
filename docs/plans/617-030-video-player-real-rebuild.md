---
plan_id: PLAN-617
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 030-video-player-real-rebuild
author: [zhaopuming]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 4                 # r4: VM/Rust 原生播放并入（libmpv DLL 运行时加载），T-15 门控 spike + T-16..T-20 仅 Go 后执行

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui, auto-man/api_gen]
current_step: 0
total_steps: 20
---

# [PLAN-617] 030-video-player-real-rebuild

## 0. 变更摘要

`examples/ui/030-video-player` 目前是一个**外壳真实、内核全假**的播放器：界面渲染完整，
但 `<video>` 元素从未被应用播放过（无 `autoplay`、无 `play()` 调用、无控制绑定），
OSD 的时间/进度/音量/倍速全部是与文件无关的常量，播放队列是写死在 `.at` 里的 5 条远程
URL，`src/back/api.at` 虽已生成却从未被前端引用。

2026-09-12 实证（§4.1）：Vue 端页面加载后 `video.paused === true`、`currentTime === 0`，
只显示首帧；点应用自己的「⏸ 暂停」按钮 `currentTime` 照涨（11.92 → 13.46），证明控件与
媒体元素零绑定；界面显示 `00:45 / 03:45`，而真实文件是 `00:00 / 00:49.4`。

三段式改动：

1. **视觉层（T-02..T-04）**：废掉「深色玻璃 + 渐变 + emoji + 全紫按钮」的 demo 观感，
   重做成**扁平、发丝线分隔、单一强调色 CTA** 的清爽播放器；按 PLAN-571 规则，
   未指定变体的按钮一律不是 primary 填充；emoji 图标全部换成 VM 安全 lucide 闭集图标。
   pac.at 的硬编码 `theme:` 改为可跟随系统（实测确认后落地），深浅两态双端一致。
2. **数据与交互真实化（T-05..T-06、T-09..T-10）**：新增媒体文件服务（**递归**目录扫描 +
   带 Range 的字节流），播放队列由 `E:\Video` **递归**扫描得出（实测 14 个视频、13 个在
   子目录、合计 42.41 GB、单文件最大 5.75 GB），真实文件名/相对路径/大小，
   原先写死的 5 条字面量删除；「打开本地文件」改为真正可用的文件选择；
   目录缺失/文件不可播放有明确错误态，不留黑屏。
3. **真实播放（T-07..T-08）**：为 `video` 元素在 Vue 生成器侧建立**受控媒体契约**
   （状态 → 元素属性的下行同步 + 原生媒体事件 → 状态的上行回灌），使
   play/pause/seek/音量/静音/倍速/上下曲 全部真正作用于 `<video>`；OSD 的时间与进度
   改由 `timeupdate` / `loadedmetadata` 驱动。

**一个必须提前说清的期望管理（r3 已按实测更正）**：我最初断言「Chromium 不支持
Matroska 容器，故 `TV/Loki/S02` 的 6 个 `.mkv` 必播不了」——**这个断言是错的**，
2026-09-12 用真实文件实测推翻（证据见 §4.1「容器/音轨实测」）：

- **容器不是障碍**：`Loki.S02E01…mkv` 的时长（2715.7 s）与 3840×2160 被正常解析，
  `play()` 成功、`currentTime` 推进、`readyState=4`、无错误，
  且 `webkitVideoDecodedByteCount` 达 7,790,503 —— **真实解出了 4K HEVC 画面**
  （截图可辨认为剧集内容）。
- **真正的障碍是音轨**：同一文件 `webkitAudioDecodedByteCount` 恒为 **0**
  （8 MB 与 120 MB 两个窗口各测一次）；而对照组 `龙猫…mp4`（AAC）立即有
  3,662 字节音频解码。Chromium 的 FFmpeg 构建**不含 Dolby Digital Plus
  (E-AC-3/Atmos) 解码器**，故该文件被判为「video-only」。
- **连带后果**：被判为 video-only 后，Chromium 会施加后台省电暂停，
  页面失焦时 `play()` 抛 `AbortError` —— 表现**极像「播不了」**，实为音轨缺失的次生现象。

因此本计划**不承诺「14 个文件都能完整播放」**，而是承诺
「**能播的确实播、播不了或播不好的明确说清楚为什么**」——把逐项真实状态
（含「有画面无声音」这种中间态）提升为一等公民（AC-10/AC-16）。

**VM/iced 端原生播放已纳入本计划，但由门控 spike 决定是否开工**（r4，见 §2.5）：
把界面重做成 VM 安全子集（去掉渐变/`backdrop-*`/`shadow-*`/`transition-*`/`absolute`
等已文档化降级项），并把「无解码能力」显式化为**有信息量的降级态**而非纯黑；
同时 T-13 出选型决策件、**T-15 做带实测数字的 Go/No-Go**。
路径选 **libmpv DLL 运行时加载**（`libloading` 已是本仓依赖，零构建期原生依赖）
而非 ffmpeg FFI；真正的风险不是解码器，而是**帧如何进 iced**
（mpv render API 无 Vulkan/wgpu 后端，且仓内已知每帧新建 `Handle` 会闪烁）。
**Go** → T-16..T-20 实现引擎；**No-Go** → 维持诚实降级 + 债务登记，
617 仍可按 T-01..T-12 完整交付 Vue 端。

**背景与前序**（本仓既有计划链）：
- [PLAN-542](archive/542-030-video-player.md)（已归档）：本示例的建立，交付「外壳 + 模拟状态」。
  本计划**取代**其「模拟视口」与「预置远程 URL 队列」两项决策（§5.3、§5.6）。
- [PLAN-571](archive/571-button-default-variant.md)（已归档）：`default` 变体 = UA 等价、
  未指定按钮**绝不** primary 填充。本计划的按钮纪律直接沿用该规则。
- [PLAN-412](archive/412-layout-gallery.md)：§5 降级矩阵与「换行用 grid 表达」的裁定，
  本计划重做布局时按其白名单取材。
- [PLAN-547](archive/547-image-viewer-pipeline.md)（已归档）：`imagesurface` 媒体票据/rendition
  管线。本计划**只作为 VM 侧未来接入点引用**，不改动它。
- [PLAN-616](616-015-notes-clean-ui-redesign.md)（在途）：同期的「清爽扁平化」姊妹计划，
  本计划的视觉语言、验证手法与门禁分层与之对齐。

## 1. 目标

**目标（GOAL）**
- G1：030-video-player 在 **Vue 端**成为**真实可用**的本地视频播放器——能**递归**扫描
  `E:\Video`（含 `TV/`、`TV/Loki/` 等子目录）形成队列、能播放其中任意**可播**文件、
  拖动进度条真的会 seek、暂停是真的暂停。
- G2：界面达到「清爽简明」：扁平层次、发丝线分隔、尺寸统一的控件、克制的强调色使用，
  并在**深/浅两种主题**下都成立。
- G3：消除所有「看起来能用其实是空的」UI——队列、文件浏览、传输控制、时间显示
  要么真实生效，要么显式标注为不可用。
- G4：为 VM/iced 端的真实播放给出**有依据的技术选型结论**（不实现）。

**非目标（NON-GOAL）**
- N1：**不采用 ffmpeg FFI**（`ffmpeg-the-third`/`rsmpeg`/`rusty_ffmpeg`）。理由见 §2.5：
  它会把原生 FFmpeg 构建依赖强加给 11 个 `ubuntu-latest` CI 任务，且 A/V 同步、音频输出、
  HDR 色调映射仍需自研。VM 端改走 **libmpv DLL 运行时加载**（零构建期依赖，
  且与用户指定的「动态链接库调用」一致）。
  **VM 端实现本身已纳入本计划，但由 T-15 的 Go/No-Go 门控**（r4）。
- N2：不做字幕（`.srt`/`.ass`）、音轨切换、画面比例裁切算法、硬件解码选择器。
- N3：不做音频播放（`020-music-player` 的模拟状态不在本计划范围内）。
- N4：不新增 `027-file-manager` 的「用播放器打开」联动协议（Design 30 §6/§7 属后续阶段）。
- N5：不追求 `E:\Video` 之外的多根目录/媒体库管理。
- N6：**不做真实视频缩略图**（需后端抽帧解码，是本计划之外的独立能力）。
  队列项用文件类型图标 + 文件名 + 大小/扩展名，**不用**假的首帧缩略图。
- N7：不做字幕加载与音轨切换。注：`龙猫…3AUDIO.mp4` 有 3 条音轨，浏览器只会播默认轨，
  该限制记入 README。

**影响范围**：`examples/ui/030-video-player/**`（主）、`crates/auto-lang/src/ui_gen/vue.rs`
（T-07 受控媒体契约）、`crates/auto-lang/src/ui/**` + `crates/auto-man/src/api_gen.rs`
（T-05 媒体文件服务与生成路由）。**不触碰** `crates/auto-lang/src/vm/**` 的 VM 执行语义。

## 2. 架构方案

### 2.1 分层与职责

```
┌─ Vue 前端（.at → a2vue） ─────────────────────────────────────────┐
│  app.at          顶栏 + 骨架 + 消息转发                            │
│  viewport.at     视口 + 受控 <video> + 错误/降级态                 │
│  controls.at     OSD 播控条（传输/进度/音量/倍速）                  │
│  playlist.at     右栏队列（真实扫描结果）                           │
│  player_store.at 播放状态机 + 与 <video> 的受控契约 + 队列加载      │
└───────────────────────────┬───────────────────────────────────────┘
                            │ back.api: scan_media（既有 #[api] 通路）
┌───────────────────────────▼───────────────────────────────────────┐
│  Axum 后端（auto-man/api_gen 生成）                                │
│   /api/media/scan    目录扫描 → JSON（真实文件名/大小/扩展名）       │
│   /api/media/stream  字节流，HTTP 206 + Accept-Ranges             │
└───────────────────────────┬───────────────────────────────────────┘
                            │
┌───────────────────────────▼───────────────────────────────────────┐
│  auto-lang::ui::media_service（新）                                │
│   scan_directory(root) / stream_response(root, token, range)      │
│   根目录解析序：AUTO_MEDIA_ROOT env → pac.at media_root → 默认值    │
└───────────────────────────────────────────────────────────────────┘
```

**关键裁定 R1**：视频字节**不由 `.at` 的 `#[api]` 通路承载**。`#[api]` 生成的是
JSON 端点，无法表达 Range/206 语义；浏览器 `<video>` 对 seek 依赖 Range。因此
流式端点按**既有 `auto_media` 先例**（`examples/rust-workspace/030-video-player-back/src/main.rs:5-32`
的手写 `auto_media` 路由）由生成器发出**原始路由**，服务实现落在 `auto-lang` 内。

**关键裁定 R2**：扫描**不**走 `back/api.at` + a2r→Rust 通路。`crates/auto-man/src/api_gen.rs:381-398`
已文档化 a2r 的多处后端语境缺口（`use api:` 重映射、`List<T>.new`、`&[T]`、
`str`→`String`），用它做真实文件系统遍历风险不可控。扫描与流式同源，一并落在
`media_service`。

**关键裁定 R3**：`video` 的**受控契约进生成器**（§2.3），而不是用 `ref:` DOM 逃生舱。
理由：① `ref:` 生成的引用类型是 `ref<HTMLElement | null>`（`ui_gen/vue.rs:3052-3066`），
而 `HTMLVideoElement` 才有 `play/pause/currentTime`，`npm run build` 会跑 `vue-tsc`
（`crates/auto/src/cmd_vue.rs:1224`）从而在发行构建期报 TS2339——该障碍**有已知小修法**
（见下 Plan-B），故它本身不是决定性理由；② 决定性理由是 `ref:` 方案把播放逻辑写进
命令式 handler，与 AutoUI「状态 → 视图」的模型相悖，且**无法被 VM 端复用**；
③ 受控契约（同一组 prop/event）是 VM 端未来实现解码后**可直接对接**的接口形状，
届时 VM 只需实现同一契约而不必重做界面层。

**Plan-B（降级备选，T-07 若超范围则启用）**：不做受控契约，改为照
**canvas 的既有先例**加一个 `video` 专属类型引用臂（`ref<HTMLVideoElement | null>`，
与 `ui_gen/vue.rs:3052-3066` 的 `__canvas_ref_` 臂同形，Plan 563 已为同一
`vue-tsc` 门禁问题做过一次），示例侧改用 `ref:` + 命令式 handler 驱动元素。
代价：播放逻辑离开声明式模型、VM 端不可复用、`play()` 的 Promise 需额外处理
（handler 体默认非 async）。**启用 Plan-B 必须在 §9 记录理由与债务。**

### 2.2 视觉语言

以 `examples/ui/024-charts`（token-only、零 emoji、VM 安全的样板）与
`examples/ui/031-image-viewer`（应用外壳：标题栏/侧栏/状态栏）为基准：

- **结构**：`h-12` 顶栏（`border-b border-border bg-card`）+ 中部 `row`（视口 `flex-1` +
  `w-72` 队列栏 `border-l border-border bg-card`）+ `h-14` 播控条（`border-t`）。
  发丝线分隔取代悬浮卡片；**不出现** `rounded-xl` / `shadow-*` / `backdrop-*`。
- **按钮纪律**（PLAN-571）：视口内最多 1 个 primary（主播放键）；其余为未指定变体
  （自动落到 `bg-muted border border-border`，见 `crates/auto-lang/src/ui/style/variants.rs:19-56`）
  或 `ghost`。icon 按钮统一 `w-8 h-7`，文本按钮统一 `h-9 px-3 text-sm`，
  传输键统一 `w-9 h-9`。**删除** `app.at:398-408`、`:462-488` 的按强调色分支
  `if` 链——强调色已由 `--primary` / `Color::Primary` 单槽承载。
- **图标**：emoji（🎬⏸▶🔊🔁⚡⛶≡）全部换成 `icon (name: …)`，名字限定在
  VM lucide 字形表闭集内（`crates/auto-lang/src/ui/iced/renderer.rs:5420+`，85 名）。
- **色彩**：只用语义令牌（`bg-background` / `bg-card` / `text-muted-foreground` /
  `border-border` / `bg-primary`）。**不再出现** `from-*`/`via-*`/`to-*` 渐变
  （VM 无 `background-image` 原语，`docs/style-coverage.md` 列为 unsupported）。
- **主题**：五色强调色保留（平台已支持），但**去掉应用内的强调色选择器 UI**——
  它属于系统设置而非播放器；`dark_mode` 开关保留在顶栏。pac.at 的 `theme: "dark"`
  改为系统跟随（可行性见 §4.2 P-9），落地前先实测。

### 2.3 受控媒体契约（本计划的技术核心）

`.at` 作者侧看到的契约（**Vue 端本计划实现；VM 端未来对接，本计划只登记**）：

```auto
video {
    src: .current_url          // 既有：透传
    poster: .poster_url        // 透传
    controls: false            // 透传（自带控件关闭，用自家 OSD）
    preload: "metadata"        // 透传
    playsinline: true          // 透传
    // ── 受控下行（状态 → 元素）──
    paused: .is_playing == false   // false 侧触发 play()，true 侧触发 pause()
    position: .seek_target         // 变化时写 currentTime（带阈值防抖）
    volume: .volume                // 0..100 → 0..1
    muted: .is_muted
    rate: .playback_rate           // float
    // ── 上行回灌（元素 → 状态）──
    ontimeupdate: .OnTime($0)          // $0: float 秒
    onloadedmetadata: .OnDuration($0)  // $0: float 秒
    onplaystatechange: .OnPlayState($0)// $0: bool（由 play/pause 事件合成）
    onended: .OnEnded
    onmediaerror: .OnMediaError($0)    // $0: str
}
```

生成器侧要做的是：识别同时声明了受控下行 prop 的 `video` 元素，为其发出
`ref` + 一次性挂载的控制器（watcher 下行同步 / 原生监听上行回灌），
并在合成调用时**把提取好的值绑定为形参**，从而绕开 `$event` 无法进 handler 体
（`ui_gen/ts_adapter.rs` 零处理）与 `vue_event_param` 只 narrow `.value`/`.checked`
（`ui_gen/vue.rs:15351-15413`）这两处现状限制——**不改动它们**。

**兼容性约束**：未声明任何受控 prop 的 `video` 元素行为必须与今日**逐字节一致**
（仍是裸 `<video :src :class />`），否则会波及 `019-video-app` 等既有示例。

### 2.4 队列数据流

`player_store.at` 的 `Init` → `scan_media()` → 得到 `[]MediaEntry`
→ 赋给 `.playlist`；`.current_index` 指向当前项，`.current_url` 派生为
`/api/media/stream/<id>`。切曲 = 改 `.current_index` + 复位
`.seek_target` / `.is_playing` / `.media_error`。

**扫描是递归的**（实测依据见 §4.1：14 个文件中 13 个在子目录，其中 12 个深达
`TV/Loki/`），因此 `MediaEntry` 必须携带相对路径而不只是文件名：

```
MediaEntry {
    id: str,             // blake3(relative_path) —— 令牌，绝对路径不出后端
    name: str,           // 文件名（用于标题）
    rel_dir: str,        // 相对目录，如 "TV/Loki"（用于分组与副标题）
    relative_path: str,  // "TV/Loki/Loki.S01E01...mp4"
    extension: str,      // 小写扩展名
    bytes: int,          // 真实字节数 → 人类可读 MB/GB
}
```

队列栏**按 `rel_dir` 分组**（`E:\Video` 根下的散文件 + `TV` + `TV/Loki` 三组），
组内按 `natural_sort_key` 排序 —— 否则 `S01E02` 会排在 `S01E10` 之后。

**不谎报字段**：分辨率/编码/码率/时长在不解复用文件的条件下**无从得知**，
故全部删除（今日 `db.at` 里的 `resolution: "4K"` / `codec: "HEVC"` 是凭文件名编的）。
时长由 `<video>` 的 `loadedmetadata` **回灌**；分辨率由 `videoWidth/videoHeight` 回灌；
编解码器由 `MediaCapabilities`/错误事件推断，**推不出就不显示**。这是本计划
「不再假装知道」的核心体现。

### 2.5 VM/Rust 原生播放（r4 新增，spike 门控）

用户裁定「找到可用的 Rust ffmpeg 路径就并进 617」。调研结论是：
**可用的路径存在，但机制不是 ffmpeg FFI，而且真正的风险点不在解码器。**

**候选路径对比（2026-09-12 调研）**

| 路径 | 构建期原生依赖 | 我们能白拿什么 | 我们仍要自己写什么 | 判定 |
|---|---|---|---|---|
| **A. ffmpeg FFI**（`ffmpeg-the-third` / `rsmpeg` / `rusty_ffmpeg`） | **C 工具链 + FFmpeg dev 头文件与导入库**；本仓 **11 个 CI 任务全在 `ubuntu-latest`**，默认档构建会连带要求 `libav*-dev` | 解封装 + 解码 + 像素转换 | **A/V 同步时钟、音频输出（cpal）、HDR 色调映射、丢帧策略**——全是自研播放器的经典坑 | ❌ 不作为首选 |
| **B. libmpv DLL 运行时加载**（`libloading` + mpv render API） | **零**（运行时 `LoadLibrary`，缺失即降级） | 解封装、解码、**硬件解码**（D3D11VA/NVDEC）、**A/V 同步**、**音频输出**、**HDR 色调映射**、libass 字幕 | **帧上屏**（见下） | ✅ **首选，且正好是用户说的「动态链接库调用」** |
| **C. `ffmpeg-sidecar` 子进程** | 零（`auto_download()` 取预编译二进制） | 同上（走 CLI + stdout 管道） | 同 A，外加每帧跨进程边界开销 | 备选/兜底 |

**为什么 B 优于 A**：A 会把「原生 FFmpeg 构建依赖」强加给本仓全部 11 个 Ubuntu CI 任务
（否则该路径在 CI 里永远不被构建 = 永远不被验证）；而 A/V 同步、音频输出、HDR 色调映射
这三件事 ffmpeg **不提供**，必须自研——恰好是 Design 30 §4.1 自己点名的「自研播放器最
容易踩的坑」。libmpv 把这四件事全部封装好了（VLC/IINA/Celluloid 同级的成熟内核），
我们只需解决「帧怎么进 iced」。

**本仓已有的有利条件**（本次实测确认，非推测）：
- `libloading = "0.8"` **已经是 `auto-lang` 的依赖**（`crates/auto-lang/Cargo.toml:134`），
  运行时 DLL 加载**零新增依赖**；
- 且已有承载位置：`crates/auto-lang/src/ffi.rs:62-145` 就是为动态库加载预留的
  `libraries: HashMap<String, libloading::Library>`，其 `TODO(Plan-212)` 明确写着
  「Implement real C library loading via libloading」——**本计划的 native 加载器正好落地在此**；
- 与仓内既有的 `video` 降级链（`render_support.rs:307`、`element_coverage.rs:449`）天然衔接：
  DLL 缺失 → 保持今日的诚实占位，不崩、不黑屏。

**真正的风险点（因此需要 spike 而不是直接排任务）**：
1. **mpv render API 只有 OpenGL 与 Software 两个后端，没有 Vulkan/wgpu 后端**
   （`libmpv/render.h`）。而本机 iced 实际选中的是 **Vulkan**（实测日志）。于是只有两条路：
   - **软件路径**：mpv 把 RGBA 写进内存缓冲 → 我们拷进 iced 纹理。简单，但每帧一次
     CPU 拷贝：1080p 约 8 MB/帧、**4K 约 33 MB/帧**，目标分辨率下能否实时**未经验证**；
   - **GL 路径**：FBO → `wgpu::Texture` 导出。性能好，但要 iced 走 glow/EGL 互操作，
     与 wgpu-first 的现状有落差（`iced-rs/iced#468` 记录了同一困境）。
2. **帧上屏还有一个仓内已知的坑**：`ui/iced/renderer.rs:2889-2898` 的注释明确写着
   `Handle::from_rgba` 每次都会 mint 新的 `Id::unique()`，**每帧新建 Handle 会造成纹理
   缓存未命中与重上传，与帧时钟竞争产生可见闪烁**。视频帧流必须绕开这条路径
   （复用 Handle / 直写纹理），这是硬约束而非优化项。
3. **先例存在，可降低风险**：`ophymx/mpv-engine` 即「libmpv 嵌入核心」，
   其 iced widget 两种路径都做过（SW 读回 RGBA、GL FBO 导出为 `wgpu::Texture`），
   可作 spike 的参照实现。
4. **本机没有任何 mpv/libmpv**（实测：Program Files 下无 `mpv.exe` / `libmpv-2.dll`），
   故 spike 必须先把「获取 libmpv 构建」纳入，且 **CI 绝不可依赖它**。

**据此的结构**：T-15 是**门控 spike**（产 Go/No-Go 决策件 + 实测帧率/内存数据），
T-16..T-20 是**仅在 Go 之后执行**的引擎实现。这样既满足用户「并进 617」的要求，
又不会把未经验证的假设写成必然发生的工程量——这是 r3 那次「未实测就断言」教训的直接应用。

## 3. 技术栈

| 层 | 技术 | 既有/新增 |
|---|---|---|
| 前端 UI | AutoUI `.at` → Vue 3 + Tailwind + shadcn | 既有 |
| 播放内核 | 浏览器原生 `<video>`（H.264/H.265/AV1/VP9 硬解） | 既有（能力在浏览器，接线是新增） |
| 受控契约 | `ui_gen/vue.rs` 生成器扩展 | **新增** |
| 后端 | Axum 0.7 + tokio | 既有 |
| 字节流 | 手写 206/Range（或 `tower-http` `fs` feature 的 `ServeFile`） | **新增**（`tower-http` 目前只开 `cors`，见 `examples/rust-workspace/Cargo.toml:58`） |
| 目录扫描 | `auto-lang::ui::media_service`（新模块，`std::fs::read_dir`） | **新增** |
| VM 播放（r4） | **libmpv DLL 运行时加载**（`libloading` 已在依赖）+ mpv render API（SW 或 GL 互操作）；备选 `ffmpeg-sidecar` | **新增**（可选 feature，T-15 门控） |

## 4. 需求分析与背景调查

### 4.1 现状实勘（2026-09-12，master `859c31710`）

- **应用形状**：`src/front/app.at` 单文件 1193 行；`src/back/api.at`（`playlist`/`get_track`）
  与 `src/back/db.at`（5 条**字面量** `VideoTrack`）已存在且已生成后端 crate
  `examples/rust-workspace/030-video-player-back/`，但**前端从未 `use back.api`**。
- **播放是假的**（本次实证，浏览器实测）：
  - 页面加载 3s 后：`video.attributes === ["class","src"]`、`paused === true`、
    `currentTime === 0`、`readyState === 4`、`duration === 49.429`、`videoWidth === 1920`。
    即：**文件被加载、首帧被渲染，但从未起播**。
  - 点击应用自己的「⏸ 暂停」：`currentTime` 11.92 → 13.46，`paused` 恒为 `false` ——
    控件与媒体元素零绑定。
  - 界面显示 `00:45 / 03:45`，真实文件 `00:00 / 00:49.4`；居中大字「● 正在播放
    (60 FPS 硬件解码)」在 `paused === true` 时依然显示。
  - 生成的 `gen/front/vue/src/App.vue:988` 只有
    `<video :class="…" :src="current_video_url" />`；全文件搜不到 `autoplay` / `.play(` / `ref`。
- **`file://` 直接播本地文件不可行**（本次实证）：页面源 `http://localhost:3030` 下
  设 `src = "file:///E:/Video/caelestia.mp4"` 得到
  `MEDIA_ELEMENT_ERROR: Media load rejected by URL safety check`（error code 4）。
  → 本地文件**必须经 HTTP 提供**（§2.1 裁定 R1 的依据）。
- **`E:\Video` 实况**（2026-09-12 递归实测）：

  ```
  E:\Video\                        14 个视频文件，合计 42.41 GiB
  ├── caelestia.mp4                    7.7 MB   H.264（Vue 可播）
  ├── Captures\                        （无视频，仅 desktop.ini）
  ├── RustDesk\                        （空）
  └── TV\
      ├── 龙猫(...).My.Neighbor.Totoro.1988...CHS-UUMp4.mp4   1.81 GB  H.264，3 音轨
      └── Loki\                        12 个文件
          ├── Loki.S01E01..E06.*.mp4  ×6   12.98 GiB   合计  H.264
          └── Loki.S02E01..E06.*.mkv  ×6   27.74 GiB   合计  HEVC 2160p HDR / DDP5.1 Atmos
  ```

  **平铺扫描只能看见 1 个文件（7.7 MB），递归扫描才能看见 42.41 GiB** —— 这是 r2
  修订的原因，也是「队列必须按 `rel_dir` 分组」的来源。
  仓内 `examples/**` 资产中**无**视频文件，故演示根目录只能指向本机路径。
- **容器/音轨实测（2026-09-12，r3 更正）**：用一个只回前 N 字节的本地 HTTP 服务
  把真实文件喂给页面，实测结论如下（**推翻了我先前的「Matroska 不支持」断言**）：

  | 探针 | `Loki.S02E01…mkv`（HEVC 4K HDR + DDP5.1 Atmos） | `龙猫…mp4`（H.264 1080p + AAC，对照组） |
  |---|---|---|
  | `canPlayType('video/x-matroska')` | `"maybe"`（该 API 宽松，**不可作判据**） | — |
  | `MediaSource.isTypeSupported('video/x-matroska')` | `false` | — |
  | 元数据解析 | ✅ duration 2715.7 s / 3840×2160 | ✅ 1920×1080 |
  | `play()` | ✅ succeeded | ✅ succeeded |
  | `currentTime` 推进 | ✅（8 MB 窗口下推进并渲染） | ✅ 5.77 s |
  | `webkitVideoDecodedByteCount` | ✅ **7,790,503**（真实解出 4K HEVC 画面，截图可辨内容） | ✅ 2,658,289 |
  | `webkitAudioDecodedByteCount` | ❌ **0**（8 MB / 120 MB 两次均 0） | ✅ **3,662**（AAC 正常） |
  | `error` | 无 | 无 |

  **裁定**：① 容器（Matroska）**不是**障碍；② 障碍是 **Dolby Digital Plus
  (E-AC-3/Atmos) 音轨**——Chromium 的 FFmpeg 构建不含该解码器；③ 音轨缺失使媒体被判为
  「video-only」，进而触发后台省电暂停，失焦时 `play()` 抛 `AbortError`
  （`"video-only background media was paused to save power"`），**表象酷似「播不了」**。
  **未测**：HDR10 色调映射质量（单帧无法判定）；其余 MKV/编解码器组合。
- **VM 端**：`video` 在 `crates/auto-lang/src/ui/render_support.rs:307-310` 为
  `TagSupport::fallback(…, "media component not implemented")`，
  `crates/auto-lang/src/aura/element_coverage.rs:449` 为 `QueueStatus::NotYet`；
  实测 VM 视口为**纯黑**（仅剩模拟覆盖层），且该降级**不产生任何运行时警告**。
- **视觉问题清单**（用于 AC-01/AC-02/AC-04 的可验证判定）：
  `backdrop-blur` ×4、`drop-shadow` ×7、渐变 `from-/via-/to-` ×6、`shadow-inner`、
  `rounded-xl`/`rounded-2xl` 多处、`text-[Npx]` 任意值 ×7、`flex-wrap`、`hover:scale-105`、
  `transition-*`/`animate-fade`、emoji 18 处、`absolute` ×3；按钮尺寸至少 5 种不同组合
  （`px-2 py-1` / `px-2.5 py-1` / `px-3 py-1.5` / `w-8 h-8` / `w-9 h-9`），且
  `app.at:398-408`、`:462-488` 存在按强调色分支的冗余 `if` 链。

### 4.2 DSL/VM 能力探针（决定设计形状的硬证据）

| # | 探针 | 结论 | 出处 |
|---|---|---|---|
| P-1 | `video` 是否映射为真 HTML 元素 | 是，`map_tag("video") → "video"` | `ui_gen/vue.rs:8095` |
| P-2 | 非保留 prop 是否透传 | 是，无按标签白名单；`controls/autoplay/muted/loop/poster/playsinline/preload` 均可发 | `ui_gen/vue.rs:6259-6445` |
| P-3 | `schema/aura.at` 的 `video` prop 声明 | `props: []` 且 `backends: { web: "none", iced: "fallback" }`；前者使 shadcn 注册表不认领该标签（故走裸元素臂），后者与 render_support 的 fallback 互相印证；`props: []` 另使 S001 校验跳过（**typo 也静默通过**） | `schema/aura.at:2153-2161`、`ui_gen/validators.rs:938-940` |
| P-4 | 任意原生事件是否可转发 | 是，`base_event_to_dom` 的 strip-`on` 兜底（`:14603`）；`ontimeupdate`→`@timeupdate` 等 | `ui_gen/vue.rs:14548,14603` |
| P-5 | `$event` 能否进 handler 体 | **否**，`ts_adapter.rs` 零处理；`$event` 只在事件实参位合法 | `parser.rs:16622-16628` |
| P-6 | 事件实参的属性 narrow | 只 narrow `.target.value` / `.target.checked` | `ui_gen/vue.rs:15351-15413` |
| P-7 | `ref:` DOM 逃生舱 | 可用（028-dom-escape 为常驻能力测试），但引用类型为 `ref<HTMLElement\|null>`；`HTMLVideoElement` 无特例 | `ui_gen/vue.rs:3052-3066`、`ui_gen/ts_adapter.rs:836-840` |
| P-8 | 自定义 Vue 组件导入 | 可用：`use.web component X from "src/front/x.vue"` | `examples/capability-tests/k4-ports-forwarding/src/front/ports/symbols.web.at:14` |
| P-9 | pac.at `theme:` 能否写 `auto` | **否**，`THEME_PREFS = ["dark","light"]`，`auto` 被当未知值忽略；OS 跟随仅在「env 链解析不出值」时对独立 VM 窗生效（**待实测**） | `ui/style/theme/mod.rs:200`、`ui/iced/renderer.rs:11246-11277` |
| P-10 | `#[api]` 前后端通路 | 前端 `use back.api:` + 裸调用；Vue 由 `auto run` 自启 Rust 后端；VM 需 `--no-merge`（默认合并模式编译为 `warn_api_noop`） | `examples/ui/031-image-viewer/src/front/app.at:4`、`auto-man/vue.rs:5378-5392`、`vm/codegen.rs:8206` |
| P-11 | 既有文件服务端点 | 仅 `/api/__auto/media/{id}/{revision}`，**图片专用**（扩展名白名单 + rendition worker），`media_http_response` **无 206/Range** | `ui/image_pipeline.rs:1207-1260`、`:960-968` |
| P-12 | 生成路由可否定制 | 可——`auto_media` 即为生成器发出的手写路由先例 | `examples/rust-workspace/030-video-player-back/src/main.rs:5-32` |
| P-13 | 后端可依赖的 HTTP 栈 | `axum 0.7` + `tower-http 0.5`（**仅 `cors` feature**） | `examples/rust-workspace/Cargo.toml:56,58` |
| P-14 | a2r→Rust 用于文件遍历 | **高风险**，`api_gen.rs:381-398` 文档化多处后端语境缺口 | `crates/auto-man/src/api_gen.rs:381-398` |
| P-15 | 仓库既有媒体解码依赖 | **无**：全仓 `Cargo.toml`/`Cargo.lock` 无 ffmpeg/gstreamer/symphonia/rodio/cpal 任何一条 | 全仓扫描 |
| P-16 | iced 已开 feature | `["tokio","image","svg","advanced","canvas"]`；无帧/视频能力 | `crates/auto-lang/Cargo.toml:177` |
| P-17 | 是否有「按真元素类型声明 ref」的既有先例 | **有**：canvas 走 `ref<HTMLCanvasElement \| null>` 独立臂，注释明写理由为「width/height 属性在 HTMLElement 上不存在，vue-tsc 门禁」（Plan 563） | `ui_gen/vue.rs:3052-3066` |
| P-18 | 是否有可复用的递归索引先例 | **有且近乎同形**：`image_viewer.rs` 的 `index_directory` 已实现**递归收集** + `relative_path`（`\`→`/` 归一）+ **`natural_sort_key` 自然排序** + **blake3 哈希 id 令牌** + `redact_path` 路径脱敏 + `cycle_index` 上下曲循环。`MediaEntry` 可直接复用该形状，只换扩展名白名单 | `crates/auto-man/src/image_viewer.rs:11,225,253-264,266-283,284,291` |
| P-19 | 索引模块该放哪个 crate | 生成的后端 crate 依赖 `auto-lang`（`features=["ui","image-pipeline"]`）而**不依赖 `auto-man`**，故可被生成路由调用的服务必须落在 `auto-lang`（与 `image_pipeline::media_http_response` 同处）；`image_viewer.rs` 在 `auto-man` 只作**形状模板** | `examples/rust-workspace/030-video-player-back/Cargo.toml` |

### 4.3 被测试钉住的既有语义（不可默默改）

- `video` 元素在**未声明受控 prop** 时，生成结果必须保持裸 `<video :src :class />`
  （§2.3 兼容性约束）——`019-video-app` 与 widgets-gallery 可能依赖。
- 五个强调色名与色值（`indigo/coral/ocean/sage/amber`）是平台契约
  （`ui/style/theme/mod.rs:202`、Design 22 §7.2），本计划**不动**，只删应用侧的 if 链。
- `dark_mode` / `accent_color` 两个魔法变量名的语义与生成器特判
  （`ui_gen/vue.rs:4255-4300`）不动。

### 4.4 契约与授权

- **已授权范围**：用户明确要求「①UI 重新设计（清爽简明 + 深浅主题）②mock 转真实
  （播放/文件浏览/队列，队列从 `E:\Video` 扫描）③先完善 Vue 端真实播放，
  VM 端另行讨论」，并要求输出**详细改进计划**。允许读取 `E:\Video`。
- **未授权/未指定**：无预算或自动延续额度说明；VM 端解码实现被明确排除在本轮之外。
- **需用户裁决**：本计划是否维持「单计划」形态（见 §10-1）；`E:\Video` 作为示例
  默认根目录是否可接受（见 §10-2）。

## 5. 详细设计

### 5.1 文件与组件形状

```
examples/ui/030-video-player/
├── pac.at                    改：theme 跟随系统、新增 media_root（实测后定）
├── README.md                 重写：真实能力边界 + 运行方式 + VM 端限制
├── SPEC.md                   重写：真实数据契约与状态机
├── src/front/
│   ├── app.at                重写：顶栏 + 骨架 + 消息转发（当前 1193 行单文件拆分）
│   ├── viewport.at           新：视口 + 受控 video + 错误/降级态
│   ├── controls.at           新：OSD 播控条
│   ├── playlist.at           新：右栏队列（真实扫描结果）
│   └── player_store.at       新：播放状态机 + 队列加载 + 时长回灌
├── src/back/
│   ├── api.at                改：删字面量契约中的谎报字段（resolution/codec/duration_str）
│   └── db.at                 删：字面量数据退役（改由 media_service 扫描）
└── tests/
    ├── smoke.spec.ts         重写：真实播放断言（currentTime 递增 + 帧变化）
    └── vm-smoke.mjs          改：VM 端降级态断言

crates/auto-lang/src/ui_gen/vue.rs        改：受控媒体契约（T-07）
crates/auto-lang/src/ui/media_service.rs  新：扫描 + Range 流（T-05）
crates/auto-lang/src/ui/mod.rs            改：注册新模块
crates/auto-man/src/api_gen.rs            改：生成 /api/media/* 路由（T-05）
examples/rust-workspace/Cargo.toml        改：tower-http 视需要加 fs feature（T-05）
```

### 5.2 骨架与顶栏（app.at）

```auto
col {
    style: "w-full h-screen flex-col bg-background text-foreground overflow-hidden"
    row {                                   // 顶栏 h-12
        style: "h-12 shrink-0 items-center gap-3 px-4 border-b border-border bg-card"
        icon (name: "film", size: 18, style: "text-primary")
        text "Video Player" { style: "text-sm font-semibold tracking-tight" }
        col { style: "flex-1" }
        button { onclick: .ToggleDarkMode; style: icon_btn
            if .store.dark_mode { icon (name: "sun", size: 16) }
            else { icon (name: "moon", size: 16) }
        }
        button { onclick: .TogglePlaylist; style: icon_btn
            icon (name: "list", size: 16) }
    }
    row { style: "flex-1 min-h-0"          // 主体
        Viewport(...)                       // flex-1
        Playlist(...)                       // w-72 border-l
    }
    row { Controls(...) }                   // h-14 border-t
}
```
按钮样式用 PLAN-607 风格配方（`style` 命名配方）集中定义，避免 1193 行里的重复串。

### 5.3 视口（viewport.at）

- 正常态：`video { src: .store.current_url, paused: …, position: …, volume: …,
  muted: …, rate: …, ontimeupdate: .OnTime, onloadedmetadata: .OnDuration, … }`，
  外层 `bg-black` 容器 + 居中缓冲/错误指示。
- **降级/错误态**（VM 端与解码失败共用）：一个 VM 安全的信息面板显示真实文件名、
  大小、以及「本后端暂无视频解码能力」/错误原因，**不再留纯黑**。
- 覆盖层：播放/暂停大按钮与点击视口切播的交互保留，但**去掉** `absolute` 定位
  （VM 降级）与 `pointer-events-none`，改为常规流式布局 + 不遮挡。

### 5.4 播控条（controls.at）

`row` 三段：左=时间（`current_time_str` / `duration_str`，由回灌状态格式化）、
中=进度条（`progress` + `onclick` 计算 seek 目标写入 `.seek_target`）、
右=音量（`-`/数值/`+`/静音）、倍速（`0.5/0.75/1.0/1.25/1.5/2.0`）、上一首/播放暂停/下一首。
按 PLAN-412 §5：**不用 `absolute`**；进度条用 `row` + 宽度百分比填充表达。
按钮统一 `w-9 h-9`（传输键）与 `h-9 px-3`（文本键），主播放键唯一 primary。

### 5.5 媒体服务与路由（T-05）

- `crates/auto-lang/src/ui/media_service.rs`（新；P-19 决定其位置，P-18 决定其形状）：
  - `index_directory(root) -> MediaIndex`：**递归**遍历（双栈/显式递归，**不跟随
    symlink/junction** 以防环），扩展名白名单过滤，产出
    `MediaEntry { id, name, rel_dir, relative_path, extension, bytes }`。
    形状、排序与脱敏**照搬 `crates/auto-man/src/image_viewer.rs` 的已验证实现**：
    `collect_files` 递归（`:253-264`）、`natural_sort_key` 自然排序（`:225`,`:273`）、
    相对路径 `\`→`/` 归一化、`id = blake3(relative_path)`（`:281`）、
    `redact_path`（`:284`）。**换的只有扩展名白名单**：
    `mp4 / m4v / webm / mkv / mov / avi`（白名单是「候选」，真实可播性由
    浏览器决定，见 AC-16 的逐项错误态）。
  - `stream_response(root, id, range_header) -> HttpResponse`：以 **`id` 反查
    `relative_path`**（绝不接受请求方传入的路径，杜绝任意文件读取），实现
    206 + `Accept-Ranges` + `Content-Range`；无 Range 头退化为 200。
    **必须惰性分块**：单文件最大 5.75 GB，**禁止** `fs::read` 整文件进内存，
    用 `tokio::fs::File` + `ReaderStream`（或 `tower-http` 的 `ServeFile`）流式转发。
  - 人类可读大小格式化（B/KB/MB/GB）——今日 `db.at` 只到 MB 且是编的。
  - 根目录解析序：`AUTO_MEDIA_ROOT` env → `pac.at` `media_root` → 内置默认。
- `crates/auto-lang` 的 `Cargo.toml`：若走 `ServeFile` 路线需给 workspace 的
  `tower-http` 加 `fs` feature（现仅 `cors`，P-13）；该选择在 T-05 决策并记录。
- `crates/auto-man/src/api_gen.rs`：按 `auto_media` 先例（P-12）发出
  `GET/HEAD /api/media/scan` 与 `GET/HEAD /api/media/stream/{id}`；仅在应用
  声明了媒体需求时发出（判定条件在 T-05 决定并记录），避免污染全部 43 个后端 crate。

### 5.6 store 与 handler 清单（player_store.at）

状态：`playlist []MediaEntry`、`current_index int`、`current_url str`、
`is_playing bool`、`seek_target float`、`volume int`、`is_muted bool`、
`playback_rate float`、`current_time float`、`duration float`、`media_error str`、
`media_unplayable bool`、`root_missing bool`、`groups []str`（由 `rel_dir` 去重得出）。
handler：`Init`（递归扫描）、`SelectIndex(int)`、`TogglePlay`、`SeekTo(float)`、
`OnTime(float)`、`OnDuration(float)`、`OnPlayState(bool)`、`OnEnded`、`OnMediaError(str)`、
`VolUp/VolDown/ToggleMute/SetRate(str)`、`Next/Prev`（索引用 `cycle_index` 语义循环）、
`OpenLocalFile`、`Rescan`。
**纪律**：内建/原生调用只在根 handler，store handler 内不调（沿用 041/043 的既有纪律）。

### 5.7 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/overview.md`（媒体元素节） | 新增「`video` 受控媒体契约」：受控下行 prop 清单（`paused`/`position`/`volume`/`muted`/`rate`）+ 上行事件（`ontimeupdate`/`onloadedmetadata`/`onplaystatechange`/`onended`/`onmediaerror`）及其语义；声明「未声明受控 prop 时行为不变」的兼容边界；标注 Vue 已实现、iced 仍为 fallback | 该契约是本计划的接口核心，且是 VM 端未来对接的形状，必须进规范否则下个计划会重新发明 | AC-05..AC-09 |
| SD-02 | add | `docs/specs/auto-lang/ui/overview.md`（媒体服务节） | 新增「媒体文件服务」：`index_directory` **递归**语义（不跟随 symlink）与 `MediaEntry{id,name,rel_dir,relative_path,extension,bytes}` 契约（`id` 为 blake3 令牌，绝对路径不出后端）、`/api/media/scan` / `/api/media/stream/{id}` 语义、Range/206 契约与**惰性分块**（不得整文件入内存）、根目录解析序（env → pac.at → 默认） | 该服务是示例之外可复用的平台能力；安全边界（不得任意读文件）与大文件内存约束必须成文 | AC-05, AC-10, AC-15 |
| SD-03 | modify | `docs/specs/auto-lang/ui/overview.md`（示例示范段） | 030-video-player 的示范描述从「沉浸式大视口 + 预置远程 URL 队列」更新为「扁平发丝线布局 + 真实目录队列 + 受控 `<video>`」 | 该示例是本仓的视频类权威示范，实现变了规范描述必须跟 | AC-01, AC-05 |
| SD-04 | add | `docs/specs/auto-lang/ui/overview.md`（已知约束节） | 记录三条已知约束：① iced 端 `video` 为 fallback 且**静默降级**（无运行时警告）；② 本仓尚无任何视频解码依赖；③ **Vue 端的真实能力边界经实测**：Matroska 容器**可**解（推翻「Chrome 不支持 MKV」的常见误判），但 Chromium 的 FFmpeg 构建**不含 Dolby Digital Plus (E-AC-3/Atmos) 解码器**，此类文件表现为「有画面无声音」并被判为 video-only（失焦时 `play()` 抛 `AbortError`）。规范须要求界面**如实标注音轨不可用**，不得假装有声 | 本计划实证了静默降级、容器可解、音轨不可解三项事实；不落文会让后续计划重复踩，并会诱发「MKV 播不了」与「所有文件都能播」两种相反的错误预期 | AC-10, AC-13, AC-16 |
| SD-05 | add（**仅 Go 后**） | `docs/specs/auto-lang/ui/overview.md`（媒体元素节）+ `schema/aura.at` 的 `video.backends.iced` | 把 iced 端 `video` 的支持级别从 `fallback` 提升为可用，并登记：native 播放为**可选 feature**、运行库解析序（如 `AUTO_MPV_LIB` → 系统路径）、缺失时的降级行为；同时记录「mpv render API 无 Vulkan/wgpu 后端，故帧上屏走 SW 或 GL 互操作」这一架构约束 | 该能力一旦落地即为平台规范的一部分，且「可选 + 可降级」的边界必须成文，否则会被误当作默认能力 | AC-19, AC-20 |

无 spec 影响的改动需说明理由：`examples/**` 的视觉与数据改动不产生规范增量；
`ui_gen/vue.rs` 的受控契约实现细节属实现而非规范，规范只记「作者可见的契约」（SD-01）。

## 6. 测试设计

### 6.1 分层门禁（按 AGENTS.md §Change-Scoped Verification Gate）

| 层 | 手段 | 覆盖 |
|---|---|---|
| 编译/校验 | `auto gen` 无 error；VM 启动无 `panicked` | 每步 |
| 生成器单测 | `cargo check -p auto-lang` + `cargo t vue_gen`（受控契约生成断言） | AC-05..AC-09 |
| Rust 单测 | `cargo t media_service`（扫描白名单 / Range 解析边界：合法/非法/越界/无头） | AC-05, AC-10 |
| Schema/文档门禁 | 若改动 `schema/aura.at`（`video` 的 props 声明）→ `cargo test -p auto-lang --test docs_gen` | AC-12 |
| 场景（Vue/E2E） | `pnpm test`（`tests/smoke.spec.ts` 真实播放断言） | AC-05..AC-10 |
| 场景（VM/MCP） | `python .agents/skills/autoui-verifier/scripts/test_vm_mcp.py` → 降级态断言 | AC-10, AC-11 |
| 视觉 | `test_vue_playwright.mjs` + `test_vm_mcp.py` 双端截图（深浅各一组） | AC-01..AC-04 |
| 全局回归 | 触及 `crates/**` → 收尾跑 `cargo t`；**不触及** `vm/**`，按 §AAVM Tier 零触发 `cargo taa` | AC-13 |

### 6.2 新增/改写的场景测试

`tests/smoke.spec.ts` 重写为**真实播放**断言（这是本计划的核心回归网）：

- **T1 队列真实且递归**：队列条目数 == `E:\Video` **递归**枚举到的白名单文件数
  （基线：**14**）；且**必须包含嵌套项**——断言存在 `rel_dir === "TV/Loki"` 的条目
  （基线 12 个）；断言根目录 `caelestia.mp4` 与 `TV/龙猫…mp4` 均在列。
  **平铺实现会得到 1 条，此断言必红**，这就是 r2 的回归网。
- **T2 起播**：点队列中**可播**的第 1 项（`caelestia.mp4`，7.7 MB）→ 等 `loadedmetadata`
  → 断言 `video.videoWidth > 0` 且 `video.paused === false`；两次采样 `currentTime` 严格递增。
  **测试只允许点小文件**，禁止在 e2e 里加载 GB 级样本。
- **T3 暂停真实**：点暂停 → `video.paused === true` 且 `currentTime` 在 1s 内不变；
  再点播放 → `paused === false`。**这是今日必红的断言**（现状点按钮不影响元素）。
- **T4 seek 真实**：点进度条 50% 位置 → `currentTime` 落在 `duration * 0.5 ± 5%`。
- **T5 音量/静音/倍速真实**：`video.volume` / `video.muted` / `video.playbackRate`
  随操作变化（断言元素属性，不是断言界面文字）。
- **T6 时间显示真实**：OSD 显示的总时长 == `duration` 的格式化值（±1s）；
  当前时间随播放前进。**否定**今日的 `00:45 / 03:45` 常量。
- **T7 上下曲**：下一首 → `currentSrc` 变化且为**下一条目的 `id`** 对应的流地址；
  末项再按下一首回到首项（`cycle_index` 语义）。
- **T8 本地文件浏览**：选择本地文件后 `video.src` 为 blob/流地址且可播放。
- **T9 错误态**：构造不可播放源 → 出现错误文案，视口非纯黑。
- **T10 VM 降级**：VM 模式下视口显示文件名与「无解码能力」提示（非空黑）。
- **T11 逐项播放状态被诚实报告**（AC-16）：选中 `TV/Loki/S02E01…mkv` → 断言
  (a) 画面维度被解析为 3840×2160；(b) 界面标注「音轨不受支持（Dolby）」；
  (c) **不出现**「正在播放（含声音）」这类不实陈述。
  断言依据 r3 实测的预期结果（画面可播、Dolby 音轨不可解码），
  **不断言「全部可播」也不断言「一律播不了」**。
- **T12 Range 与大文件**（AC-15）：对 `TV/Loki/S02E01…mkv`（4.29 GB）发
  `Range: bytes=0-1023` → 断言 `206` + `Content-Range` 正确；断言服务端进程
  内存**不随文件大小增长**（不整文件读入）。此测在 **Rust 单测 / curl 层**做，
  不放进浏览器 e2e。

## 7. 验收标准

- **AC-01 扁平视觉**：Vue 与 VM 双端均为「`h-12` 顶栏 + 视口 + `w-72` 队列栏 + `h-14` 播控条」
  的发丝线扁平结构。验证：双端截图；`grep -nE 'backdrop-|shadow-|rounded-(xl|2xl)|animate-|hover:scale|flex-wrap' src/front/*.at`
  无非注释命中；渐变 `from-|via-|to-` 零命中。
- **AC-02 按钮纪律**：控件尺寸收敛为 3 档（`w-8 h-7` icon / `w-9 h-9` 传输 / `h-9 px-3` 文本）；
  每个视图 primary 按钮 ≤ 1；未指定变体不出现在 primary 填充中；按强调色分支的 `if` 链已删。
  验证：截图 + `grep -c 'bg-primary' src/front/*.at` 计数复核。
- **AC-03 双主题**：`dark_mode` 双端切换生效；浅色态下无「深色残留」（白底黑字可读、
  边框可见）。pac.at 的主题跟随行为**实测后**记录（P-9 待实测）。验证：双端深浅各一组截图。
- **AC-04 图标一致**：UI 无 emoji；图标名全部在 VM lucide 85 名闭集内；
  VM 截图中不出现空图标色块。验证：
  `grep -nE '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]' src/front/*.at` 无非注释命中。
- **AC-05 队列来自真实目录且递归**：队列由 `E:\Video` **递归**扫描得出，条目数等于
  递归枚举到的白名单文件数（基线 14），**包含 `TV/Loki/` 下的嵌套条目**（基线 12），
  队列按 `rel_dir` 分组、组内自然排序（`S01E02` 在 `S01E10` 之前）；条目标题与副标题
  来自真实文件名与相对目录；`src/back/db.at` 的字面量数据已退役。
  验证：spec T1 + 与 `find` 对拍（`find E:/Video -type f -iname '*.mp4' -o -iname '*.mkv'`）。
  **平铺实现只能得到 1 条，故本 AC 可判定递归与否。**
- **AC-06 真实起播**：选中条目后 `<video>` 真实加载并播放（`paused===false`、
  `videoWidth>0`、两次采样 `currentTime` 递增）。验证：spec T2（**今日必红**）。
- **AC-07 控件真实作用于媒体**：播放/暂停/seek/音量/静音/倍速/上下曲全部落到元素属性。
  验证：spec T3/T4/T5/T7（**今日必红**）。
- **AC-08 时间与进度真实**：OSD 时间/进度由 `timeupdate`/`loadedmetadata` 驱动，
  与 `duration`/`currentTime` 一致；常量 `03:45` 不再出现。验证：spec T6 + `grep`。
- **AC-09 文件浏览真实**：「打开本地文件」真正可选本地视频并播放。验证：spec T8。
- **AC-10 错误可见**：目录缺失 / 文件不可播放 / 后端不可达 均有明确文案，无静默黑屏；
  `/api/media/stream` 对越界路径返回 4xx 而非读出文件（安全断言）。验证：spec T9 + Rust 单测。
- **AC-11 VM 端显式降级**：VM 视口显示真实文件名与「本后端无解码能力」提示，
  且该限制写进 README/SPEC。验证：VM 截图 + 文档 diff。
- **AC-12 契约同步**：`SPEC.md`/`README.md` 与实现一致，无 mock/预置远程队列表述残留；
  `video` 元素在未声明受控 prop 时生成结果与今日一致（兼容性回归断言）。验证：
  `cargo t vue_gen` + `grep -n 'commondatastorage\|BigBuckBunny' src/` 零命中。
- **AC-13 无新增回归**：`cargo t` 相对基线零新增红；`cargo check -p auto-lang` 干净；
  无 `crates/auto-lang/src/vm/**` 改动（故不需 `cargo tv`/`cargo taa`）。
- **AC-14 VM 播放决策件**：产出一份技术选型结论（FFmpeg 系 / Media Foundation /
  GStreamer / 纯 Rust 的对比与推荐路径、许可与分发影响、与 `imagesurface` 的接入点），
  落入 `docs/design/autoui/030-video-player.md` §4 更新。验证：文档存在且结论可执行。
- **AC-15 大文件流式安全**：单文件最大 5.75 GB，服务端**不得整文件读入内存**；
  `Range: bytes=0-1023` 返回 `206` 且 `Content-Range` 正确；无 Range 头返回 `200`；
  越界/畸形 Range 返回 `416` 或安全降级（不是 panic、不是读出文件）。
  验证：Rust 单测（`cargo t media_service`）+ spec T12 + `curl -r 0-1023` 对拍。
- **AC-16 逐项真实播放状态（含「有画面无声音」中间态）**：每个队列项的播放结果必须
  **按实际测得**呈现，不得一律乐观或一律悲观。`TV/Loki/S02` 的 6 个 `.mkv`
  在 Vue 端预期为「**画面可播、Dolby 音轨不可解码**」（r3 实测），此时界面必须
  明确标注「音轨不受支持（Dolby Digital Plus）」并**不得**假装有声；
  真正无法解出的文件（如有）才显示错误态。验证：spec T11 + 截图。
  **本 AC 明确否定「所有文件都能完整播放」这一不成立的期望**，
  也否定「MKV 一律播不了」这一同样不成立的期望（r3 更正）。
- **AC-17 不显示无法得知的元数据**：分辨率/编码/码率在未解复用时**不得显示**；
  今日 `db.at` 的 `resolution:"4K"` / `codec:"HEVC"` 等凭文件名编造的字段全部删除；
  时长/分辨率只显示来自 `loadedmetadata` / `videoWidth` 回灌的真实值。
  验证：`grep -nE 'resolution|codec|bitrate' src/back/*.at src/front/*.at` 零命中
  （除回灌来源外）+ 截图。
- **AC-18 VM 原生播放门控决策（Go/No-Go）**：T-15 产出一份**带实测数字**的决策件——
  至少含 1080p 与 4K 下的可达帧率、每帧拷贝耗时、进程内存、以及
  「SW / GL / sidecar / 放弃」四选一的明确结论与依据。
  **未实测数字即不算通过**（本 AC 是对 r3 教训的制度化）。
  验证：决策件文本 + spike 代码可跑。
- **AC-19 VM 端真实播放（仅 Go 后适用）**：VM/iced 端能用**同一份 `app.at`** 真实播放
  本地视频文件（`paused`/`seek`/`volume`/`rate` 均作用于播放内核），
  且 `video` 从 iced `fallback` 提升为可用；帧率不低于 T-15 决策件承诺的值，
  无 `renderer.rs:2889-2898` 描述的闪烁。验证：VM/MCP 实机 + 双端同源截图。
  **No-Go 时本 AC 转为「维持诚实降级并在规范/债务中记录」的等值验收。**
- **AC-20 门控不污染默认档与 CI**：native 播放为可选 feature；
  默认档 `cargo t` 在**不安装 ffmpeg/mpv** 的机器上干净；
  全部 `ubuntu-latest` CI 任务**不含**系统级媒体包安装步骤仍全绿；
  运行库缺失时走降级而非 panic。验证：`cargo t` + CI 配置 diff + 缺库单测。

## 8. 执行步骤

> 计划获批后：先在 master commit `.next-id` 与本骨架，再建 worktree
> `git worktree add D:/autostack/.wt/lang-617/auto-lang -b plan-617-dev`。
> 全部代码改动在 worktree 内执行；本文件的勾选与 frontmatter 翻转留在 master。
> 每步完成后在下方追加 `[✅ 已完成]` 证据行。

- **T-01 基线与实勘**：双端 `before_*` 截图（深浅各一）+ `E:\Video` 文件清单归档；
  把 §4.1 的 grep 判定命令固化成可重跑脚本片段。验证：截图与清单入库。
- **T-02 视觉系统与骨架**：`app.at` 拆分重写（顶栏 + 主体 + 播控条三段骨架）、
  `style` 配方集中定义、删按强调色 if 链与全部渐变/玻璃/阴影/emoji。
  验证：Vue 截图 = 扁平三段；AC-01/AC-02/AC-04 的 grep 断言通过。
- **T-03 视口组件**：`viewport.at`（video 元素 + 覆盖层 + 错误/降级面板，去 `absolute`）。
  验证：Vue 截图；VM 截图显示降级面板而非纯黑（AC-11 前置）。
- **T-04 队列栏与播控条**：`playlist.at` + `controls.at`（尺寸收敛、单一 primary、
  PLAN-412 §5 白名单内取材）。验证：Vue/VM 双端截图；AC-02 计数复核。
- **T-05 媒体服务与生成路由**（含决策点）：新建 `media_service.rs`，
  **照搬 `crates/auto-man/src/image_viewer.rs` 的递归收集 / 自然排序 / blake3 id /
  路径脱敏形状**（P-18），实现 `index_directory`（**递归**，不跟随 symlink）与
  `stream_response`（`id` 反查 + Range/206 + **惰性分块**，禁整文件读入）；
  在 `api_gen.rs` 按 `auto_media` 先例发出 `/api/media/scan` 与
  `/api/media/stream/{id}`。**决策点**：路由发出条件（按需 vs 全量）、
  `tower-http` 是否加 `fs` feature、以及是否直接复用 `image_viewer.rs`
  而非新写——结论记入本文件 §9。
  验证：`cargo check -p auto-lang`；`cargo t media_service`（含 T12 的 Range 边界）；
  `curl -r 0-1023` 对 4.29 GB 的 `TV/Loki/S02E01…mkv` 得 `206`。
- **T-06 前端接线真实队列**：`player_store.at` 的 `Init` 调 `scan_media()`
  替换 `app.at:99` 的字面量；队列栏按 `rel_dir` 分组渲染；`src/back/db.at` 退役；
  `api.at` 删全部凭文件名编造的字段（`resolution`/`codec`/`size_str`/`duration_str`/
  `bg_gradient`）。
  验证：spec T1（递归断言，基线 14 条 / 12 条嵌套）；AC-17 的 grep 零命中。
- **T-07 受控媒体契约（生成器）**：`ui_gen/vue.rs` 实现 §2.3 的下行同步与
  上行回灌；**未声明受控 prop 时输出不变**；补生成器单测（含兼容性回归用例）。
  验证：`cargo t vue_gen`；生成产物 diff 仅限目标元素。
  **若范围失控**：改走 §2.1 Plan-B（canvas 同形的类型引用臂 + `ref:`），
  并在 §9 记录理由、代价与债务。
- **T-08 真实播放接线**：view/controls 接 T-07 契约；`OnTime`/`OnDuration`/
  `OnPlayState`/`OnEnded`/`OnMediaError` handler 落地；OSD 改由回灌状态驱动。
  验证：spec T2/T3/T4/T5/T6/T7 全绿（**今日必红项转绿是本计划的核心证据**）。
- **T-09 真实文件浏览**：`OpenLocalFile` → 文件选择 → 可播放地址；浏览器能力不足时
  给出明确降级文案而非假按钮。验证：spec T8。
- **T-10 错误与空态**：目录缺失/不可播放/后端不可达三态；`Rescan` 可用。
  验证：spec T9；Rust 单测覆盖越界路径拒绝。
- **T-11 契约与文档同步**：重写 `SPEC.md`（真实数据契约/状态机）、`README.md`
  （真实能力边界 + VM 限制 + 运行方式 + `AUTO_MEDIA_ROOT` 说明）；落 SD-01..SD-04。
- **T-12 双端验证与归档**：`cargo t` 零新增红；双端深浅截图入库；
  VM/MCP 降级态断言；`pnpm test` 全绿或明确列出仍红项与归因。
- **T-13 VM 播放选型决策件**：把 §4 研究与外部调研（FFmpeg 系 `ffmpeg-the-third`/
  `video-rs`/`rsmpeg`、Windows Media Foundation（`windows` crate 已在依赖中）、
  GStreamer、纯 Rust `rust_h264`/`openh264`；`symphonia` 仅有音频）整理为
  `docs/design/autoui/030-video-player.md` §4 的更新，含推荐路径与许可/分发影响、
  以及与 `imagesurface`/`ImageSurface` 的帧流接入点与「每帧新建 `Handle` 会抖动」
  的已知约束（`ui/iced/renderer.rs:2889-2898`）。**不实现**。
- **T-14 债务登记**：把本计划自决或未解的取舍写入 `docs/plans/KNOWN-DEBT-AND-RISKS.md`。

- **T-15 VM 原生播放门控 spike（Go/No-Go，决策件）**：在 `auto-lang` 内做一个最小 spike：
  用**已在依赖中的 `libloading`** 运行时加载 `libmpv-2.dll`，创建 mpv 句柄，
  用 render API 的**软件路径**把一帧 RGBA 送进 iced 视口，并测量
  ① 1080p 与 4K 下可达帧率、② 每帧拷贝耗时与进程内存、③ `Handle` 复用能否消除
  `renderer.rs:2889-2898` 记录的闪烁。同时评估 GL→`wgpu::Texture` 导出路径的可行性。
  参照实现：`ophymx/mpv-engine`。**产出 Go/No-Go 决策件**（含上述实测数字与
  「SW / GL / sidecar / 放弃」四选一结论），落入本文件 §9 与
  `docs/design/autoui/030-video-player.md`。**不得**在没有实测数字的情况下给出 Go。
  验证：spike 代码可跑（可临时存在，Go 后转为 T-16 的基础）+ 决策件含数字。
- **T-16 native 动态库加载器**（Go 后）：把 `crates/auto-lang/src/ffi.rs:62-145` 的
  `TODO(Plan-212)` 落地为真正的 `libloading` 动态库加载；实现 libmpv 句柄
  生命周期、render context 创建/释放顺序（含 `LC_NUMERIC=C` 等 mpv 已知要求）、
  以及**库缺失时的静默降级**（回落到今日的诚实占位，不 panic、不黑屏）。
  验证：`cargo check -p auto-lang`；库缺失路径的单测。
- **T-17 帧上屏通道**（Go 后）：按 T-15 结论实现帧到纹理的通道，
  **必须绕开每帧新建 `Handle`**（硬约束，见 §2.5 风险点 2）；
  镜像 `image_pipeline` 的既有形状（票据/rendition、latest-wins 代际门、有界缓存）。
  验证：目标分辨率下实测帧率与无闪烁截图。
- **T-18 VM 侧播放控制与契约对齐**（Go 后）：mpv 侧实现 §2.3 的**同一套受控媒体契约**
  （`paused`/`position`/`volume`/`muted`/`rate` 下行 + 时间/时长/状态上行），
  使 `.at` 应用**无需分叉**即可在两端工作；音频输出由 mpv 承担（不引入 cpal）。
  验证：同一份 `app.at` 在 VM 端能真实播放、seek、调音量。
- **T-19 `video` 元素支持级别提升**（Go 后）：把 `video` 从 iced `fallback` 提升为
  可用（更新 `render_support.rs` 与 `schema/aura.at` 的 `backends.iced`，
  保持「schema 为权威」的既定关系）；若 Go 条件不满足则**不动**，维持 fallback 并
  在规范中记录原因。
  验证：`cargo t` + `cargo test -p auto-lang --test docs_gen`（触及 schema）。
- **T-20 特性门控与 CI 保真**（Go 后，**硬要求**）：native 播放必须是**可选 feature**
  （沿用 `ui-iced`/`python` 的既有模式），默认档与 **全部 11 个 `ubuntu-latest` CI 任务
  在不安装 ffmpeg/mpv 的情况下保持全绿**；缺失运行库时走降级路径。
  验证：默认档 `cargo t` 干净；CI 配置 diff 中**不出现** apt 安装 ffmpeg/mpv 或
  系统包安装步骤；`AUTO_MPV_LIB` 等解析序文档化。

**依赖**：T-01 → T-02/T-03/T-04 → T-05 → T-06 → T-07 → T-08 → T-09/T-10 → T-11 → T-12；
T-13/T-14 与主链并行。
**VM 链（r4）**：T-15（门控）→ **Go 才** → T-16 → T-17 → T-18 → T-19 → T-20。
**No-Go 时**：T-16..T-20 不执行，改为把 T-13 决策件的结论 + T-15 的实测数字
一并写入 `docs/design/autoui/030-video-player.md`，VM 端维持诚实降级并在
`KNOWN-DEBT-AND-RISKS.md` 登记；此时 617 仍可按 T-01..T-12 完整交付 Vue 端。

## 9. 复审记录

### 9.1 /auto-plan:new 起草移交（2026-09-12）

- stage: new；Plan ID: PLAN-617；plan_revision: 1。
- 依据：§4.1 现状实勘（**浏览器实测**：`paused=true`/`currentTime=0`、点按钮 `currentTime`
  照涨、`file://` 被 URL safety check 拒绝）+ §4.2 的 16 项能力探针（含 4 项本次新增实证）。
- outcome: pass（在 §4.4 记录的授权范围内可直接进入 work）。
- next: work — 从 T-01 起执行；changed task/acceptance IDs：全部为新建
  （AC-01..AC-17 / T-01..T-14）。
- 需要用户裁决的事项：无（§10-1/§10-2 已在 r2 由用户裁定，见下）。

### 9.2 r2 修订移交（2026-09-12，用户裁定后）

- **用户裁定**：① 维持**单计划**形态；② 接受 §10-2 的方案 (a)（`pac.at` 写
  `media_root`，env 可覆盖）；③ **扫描必须是递归的**（`E:\Video\TV\` 下有视频）。
- **据此的语义修订**（`plan_revision: 1 → 2`）：
  1. 扫描从平铺改为**递归**，`MediaEntry` 增加 `rel_dir` / `relative_path`，
     队列**按目录分组**、组内自然排序 —— 依据是递归实测：平铺只能看见 1 个文件，
     递归看见 14 个 / 42.41 GiB（§4.1）。
  2. T-05 改为**照搬 `image_viewer.rs` 的已验证索引形状**（P-18/P-19），
     而不是新写一套递归逻辑；`media_service` 落在 `auto-lang`（P-19）。
  3. 新增 AC-15（大文件惰性分块 + Range 边界）与 AC-17（不显示无法得知的元数据）；
     新增 spec T11/T12；AC-16 把「不可播容器」提升为明确验收项。
  4. 新增非目标 N6（不做真实缩略图）与 N7（不做字幕/音轨切换）。
  5. SD-02/SD-04 的规范文本随之扩写（递归语义、`MediaEntry` 契约、Matroska 约束）。
- **未变**：三段式结构、三个关键裁定 R1/R2/R3、Plan-B、任务编号 T-01..T-14、
  AC-01..AC-14 的既有内容（只做增补）。
- outcome: pass（在 §4.4 授权 + r2 用户裁定范围内可直接进入 work）。
- next: work — 从 T-01 起执行。

### 9.3 r3 修订移交（2026-09-12，实测更正 + VM 路线裁定）

- **触发**：用户问「mkv 格式是 chrome 浏览器不支持对吗？那么我们就只在 vm/rust 后端
  支持播放它们吧」，并推测「ffmpeg 应该支持这个格式」。
- **实测更正（本次最重要的记录）**：我先前的「Chromium 不支持 Matroska，`.mkv` 必然
  播不了」是**错的**。用一个只回前 N 字节的本地 HTTP 服务把真实文件喂进浏览器实测：
  `Loki.S02E01…mkv` 的时长/3840×2160 正常解析、`play()` 成功、`currentTime` 推进、
  `webkitVideoDecodedByteCount = 7,790,503`（**真实解出 4K HEVC 画面**，截图可辨内容）、
  无 error；而 `webkitAudioDecodedByteCount` 在 8 MB 与 120 MB 两个窗口下**均为 0**，
  对照组 `龙猫…mp4`（AAC）为 3,662。**结论：容器可解，音轨（Dolby Digital Plus /
  E-AC-3 / Atmos）不可解**；音轨缺失使媒体被判为「video-only」，进而触发后台省电暂停
  （失焦时 `play()` 抛 `AbortError`），表象酷似「播不了」。
  完整表格见 §4.1「容器/音轨实测」。
- **据此的语义修订**（`plan_revision: 2 → 3`，只动受该事实影响的条目）：
  1. §0 的期望管理段与 §4.1 的对应条目**重写为实测结论**；
  2. **AC-16 重写**：从「不可播容器被报告」改为「逐项真实播放状态」，
     明确纳入「**有画面无声音**」这一中间态，并要求界面标注音轨不受支持；
  3. spec **T11 重写**为断言实测预期（画面可解 + Dolby 音轨标注），
     既不断言全部可播、也不断言一律播不了；
  4. **SD-04 重写**：规范须记录「容器可解 / Dolby 音轨不可解」两面事实，
     并明确「不得假装有声」；
  5. §10-12 改为「用户已裁定 MKV 完整播放归 VM 路线」+ **更正其理由**
     （真实障碍是音频编解码器而非容器），并给出 VM 路线的三处技术要点
     （Atmos 元数据会丢 / HDR 需色调映射 / 4K HEVC 需硬解）与
     「工作量不在解 MKV，而在 A/V 同步 + 音频输出 + 帧上屏」的判断；
  6. 新增 §10-14：VM/Rust 原生播放引擎**应作为独立 L2 线立项**（先设计文档后拆 Plan），
     617 维持 Vue 端范围，T-13 决策件作为该设计文档的输入。**待用户确认。**
- **未变**：三段式结构、裁定 R1/R2/R3、Plan-B、任务编号 T-01..T-14、
  其余 AC-01..AC-15/AC-17、递归扫描设计（r2）。
- outcome: **blocked → 待用户对 §10-14 表态**；若确认「617 维持现状、VM 另立 L2」，
  则本计划即可进入 work。
- next: work（从 T-01 起）或 new（为 VM 引擎起草设计文档 + 新 Plan）。

### 9.4 r4 修订移交（2026-09-12，VM 原生播放并入 617）

- **用户裁定**：「原生播放引擎如果找到了 rust 的对应 ffmpeg 仓库且能用
  （VM 通过动态链接库调用；Rust 直接调用），那就加到计划 617 吧」。
- **调研结论（附本地实测）**：
  1. **可用的路径存在**，但最优不是 ffmpeg FFI，而是 **libmpv DLL 运行时加载**——
     正好对应你说的「动态链接库调用」。本仓 `libloading = "0.8"` **已在依赖中**
     （`crates/auto-lang/Cargo.toml:134`），且 `crates/auto-lang/src/ffi.rs:62-145`
     已有为动态库加载预留的结构（`TODO(Plan-212)`），故**零新增依赖**即可落地。
  2. **ffmpeg FFI 被排除**：本仓 **11 个 CI 任务全在 `ubuntu-latest`**，
     采用 FFI 会把 `libav*-dev` + C 工具链变成默认档构建要求；且 A/V 同步、
     音频输出、HDR 色调映射 ffmpeg 不提供，仍需自研（Design 30 §4.1 已点名这些坑）。
     `ffmpeg-sidecar`（子进程）列为备选兜底。
  3. **真正风险不在解码器，在帧上屏**：mpv render API 只有 OpenGL/Software 两个后端，
     **没有 Vulkan/wgpu**，而本机 iced 实选 **Vulkan**（实测日志）；软件路径每帧 CPU
     拷贝在 4K 下约 33 MB/帧、能否实时**未验证**；且仓内 `ui/iced/renderer.rs:2889-2898`
     明确记录「每帧新建 `Handle` 会造成缓存未命中与可见闪烁」。本机**没有任何 mpv/libmpv**。
  4. 先例 `ophymx/mpv-engine` 的 iced widget 已同时做过 SW 读回与 GL FBO→`wgpu::Texture`
     两条路，可作参照。
- **据此的结构**（`plan_revision: 3 → 4`）：
  1. 新增 §2.5「VM/Rust 原生播放」：三路径对比表（A FFI / B libmpv DLL / C sidecar）、
     本仓有利条件、四个风险点、以及「为何 spike 门控」；
  2. 新增 **T-15 门控 spike（Go/No-Go，必须带实测数字）** 与 **T-16..T-20**
     （加载器 / 帧上屏 / 控制契约对齐 / `video` 支持级别提升 / 特性门控与 CI 保真），
     T-16..T-20 **仅在 Go 后执行**，并写明 No-Go 的等值交付；
  3. 新增 **AC-18（决策件必须含实测数字）/ AC-19（VM 端真实播放，仅 Go 后适用）/
     AC-20（默认档与全部 CI 不因该能力变红）**；
  4. 新增 **SD-05**（仅 Go 后：`video` 的 iced 支持级别提升 + 可选特性/降级边界）；
  5. §1 的 N1 由「不实现 VM 播放」改为「**不采用 ffmpeg FFI**，改走 libmpv DLL」；
     §3 技术栈表补 VM 播放行；§0 摘要改写对应段落；
  6. §10-14 关闭：**已裁定并进 617**，以 spike 门控方式。
- **未变**：Vue 端全部条目（T-01..T-14、AC-01..AC-17、递归扫描、裁定 R1/R2/R3、Plan-B）。
- outcome: pass（在 §4.4 授权 + r2/r3/r4 用户裁定范围内可进入 work）。
- next: work — 从 T-01 起；**T-15 可与 Vue 主链并行启动**（互不阻塞），
  但 T-16 起必须等 T-15 判定。

## 10. 待澄清事项

1. **已裁定（r2，用户）**：**维持单计划形态**。三处改动（示例视觉 / 生成器受控契约 /
   后端媒体服务）同属一条「把 030 做成真播放器」的交付链，拆分会让「真实播放」的验收
   切成两半。风险由 T-13/Plan-B 与分层门禁兜住。
2. **已裁定（r2，用户）**：接受方案 **(a)** —— `pac.at` 写 `media_root`
   （本机为 `E:/Video`），`AUTO_MEDIA_ROOT` env 可覆盖，README 说明这是本机路径。
   替代演示目录仍不存在（仓内无视频资产），故该路径会进仓库。
   **实施注记**：README 必须写明「此路径为你本机目录，换机器需改 `AUTO_MEDIA_ROOT`」，
   且根目录不存在时走 AC-10 的空态而非报错崩溃。
3. **已自决：视频字节不走 `#[api]`**。`#[api]` 只能生成 JSON 端点，无法表达 Range/206，
   浏览器 seek 依赖 Range。故按 `auto_media` 先例走生成器手写路由（§2.1 裁定 R1）。
4. **已自决：扫描不走 a2r→Rust**。`api_gen.rs:381-398` 文档化的多处后端语境缺口
   使真实文件系统遍历风险不可控，改由 `auto-lang` 内新模块承担（裁定 R2）。
5. **已自决：受控契约进生成器而非用 `ref:` DOM 逃生舱**。`ref:` 生成 `ref<HTMLElement>`，
   媒体 API 会导致 `vue-tsc` 构建失败，且与「状态 → 视图」模型相悖、无法被 VM 端复用
   （裁定 R3，§4.2 P-7）。
6. **已自决：不实现 VM 端解码**（用户明确要求单独讨论），但把静默黑屏改为有信息降级态，
   并出 T-13 选型决策件。§4.2 P-11/P-12/P-15 已证「无任何解码依赖、既有媒体路由是图片专用」。
7. **已自决：删应用内强调色选择器 UI**。强调色是系统级设置（os-config 已达），
   应用内五色 picker 既造成「全紫」观感又属越权；改为只保留 `dark_mode` 开关，
   `accent_color` 仍作为契约变量由 pac.at/os-config 驱动（保留变量，不删）。
8. **已知偏差（登记为债务候选）**：受控契约的 `onplaystatechange` 不是原生 DOM 事件，
   需由 `play`/`pause` 两个原生事件合成；若合成引入额外渲染循环，按 T-14 登记债务。
9. **待实测（P-9）**：pac.at 的 OS 主题跟随。`THEME_PREFS` 只认 `dark`/`light`，
   `auto` 会被静默忽略；去掉 `theme:` 键后是否真能跟随系统**必须先实测**再决定写法，
   未经实测不得写进 README 承诺。
10. **已自决（含备选）**：受控契约 vs `ref:` 逃生舱。首选受控契约（裁定 R3，理由为
    声明式模型 + VM 可复用）；若 T-07 的实际范围超出单个小改动，则启用 §2.1 Plan-B
    （照 canvas 先例加 `ref<HTMLVideoElement | null>` 臂 + 命令式 handler），
    并在 §9 记录理由与债务。**该切换不需要用户裁决**，因为两条路径都满足 G1/G3，
    差别仅在架构优雅度与 VM 可复用性。
11. **已解决（r2）**：样本不足的顾虑不再成立——递归扫描下有 14 个文件 / 42.41 GiB，
    分组、上下曲、自然排序都能充分验收。**但反转出新的纪律**：e2e 测试
    **只允许点 `caelestia.mp4`（7.7 MB）**，其余样本仅用于列表渲染与 Range 头
    （`curl`/Rust 单测）断言，**禁止**在浏览器里加载 GB 级文件。
12. **已裁定（r3，用户）：`TV/Loki/S02` 这 6 个 `.mkv` 的「完整播放」（含 Dolby 音轨）
    归 VM/Rust 路线**，Vue 端只做诚实降级。**但据 r3 实测更正了理由**——不是
    「Chrome 不支持 MKV」（容器其实能解、4K HEVC 画面能渲染），而是
    **Chromium 不带 Dolby Digital Plus 解码器**。
    **对 VM 路线的含义**：用户「基于 ffmpeg 的 PotPlayer 能播」的前提是成立的
    （本机 PotPlayer 确实随包 `Module/FFmpeg60/61/62`），但要注意三件事：
    ① ffmpeg 能**解** E-AC-3，但 **Atmos 对象音频元数据会丢失**（只能得 5.1）；
    ② 该文件是 **HDR10**，需额外做色调映射（`zscale`/`tonemap`）否则画面发灰；
    ③ 4K HEVC 软解很吃 CPU，本机 RTX 4060 Ti 的 **NVDEC/D3D11VA 硬解**才是可行路径。
    真正的工作量不在「解 MKV」（这一点 ffmpeg 一句话就够），而在
    **A/V 同步 + 音频输出 + 帧上屏**——这正是 VM 端今天**完全没有**的部分
    （§4.2 P-15/P-16：仓库零媒体依赖，`video` 为 fallback）。
    故该项**不是「给 VM 加个 MKV 支持」**，而是「为 VM 建整套原生播放引擎」，
    与 617 分开立项（见 §10-14）。
13. **待实测（r2 新增）**：递归扫描在**符号链接/junction 环**上的行为。Windows 下
    `E:\Video` 目前无此类链接，但实现必须显式不跟随（`symlink_metadata` /
    `file_type().is_symlink()` 判定），并在 T-05 的单测里构造一个环验证不无限递归。
14. **已裁定（r4，用户）**：**VM/Rust 原生播放引擎并进 617**，以 **T-15 门控 spike**
    方式落地（Go 才执行 T-16..T-20）。路径选 **libmpv DLL 运行时加载**而非 ffmpeg FFI，
    理由与风险见 §2.5、修订记录见 §9.4。
    **提醒**：Go 之后本计划规模会显著增大（新增 6 个任务、3 条 AC、1 条 SD，
    并首次引入一个可选的原生媒体依赖与一条全新的验证链），
    review 时若认为体量已越过单计划上限，可依 §9.4 的切分点把 VM 链拆为独立的
    617B（Vue 链保持 617A 不变、编号与内容均不改）。
