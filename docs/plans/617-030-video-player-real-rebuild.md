---
plan_id: PLAN-617
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 030-video-player-real-rebuild
author: [zhaopuming]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 7                 # r7: T-17 完成（帧上屏通道：持久纹理 + staging 环 + 无闪烁断言）；T-18 起接控制契约

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui, auto-man/api_gen]
current_step: 3
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

- [x] **T-01 基线与实勘**：双端 `before_*` 截图（深浅各一）+ `E:\Video` 文件清单归档；
  把 §4.1 的 grep 判定命令固化成可重跑脚本片段。验证：截图与清单入库。
  [✅ 已完成 2026-09-12] worktree `D:/autostack/.wt/lang-617/auto-lang` @ `092c99f4f`。
  - 脚本：`tests/assert-style.sh`（AC-01/02/04/17 可重跑断言，非注释行判定）。
  - 清单：`tests/media-root-inventory.txt`（**递归 14 文件 / 42.41 GiB**；
    平铺仅见 `caelestia.mp4` 1 个 —— 递归与平铺的差异有了可复算证据）。
  - 截图 4 张：`tests/screenshots/before_{vue,vm}_{dark,light}.png`。
    **注意**：该目录被 `.gitignore:123-126` 有意排除（「AutoUI test screenshots &
    artifacts (local only, do not track binary images in git)」），故按仓库既定策略
    **只作本地证据、不入库**——本任务原写「截图…入库」应据此更正为「入既定本地目录」。
  - **基线断言结果（全部 FAIL，即改造前应有的状态）**：`backdrop-` 6 处、`shadow-` 21 处、
    `rounded-xl/2xl` 15 处、`animate-` 3 处、`hover:scale` 10 处、`flex-wrap` 1 处、
    渐变 `from/via/to` 21 处、emoji **18** 处、编造元数据（`resolution:`/`codec:`）12 处。
  - **T-01 三项实勘发现（影响后续任务）**：
    1. **`AUTO_UI_THEME` 环境变量无效**：pac.at 的 `theme: "dark"` 在优先级链上压过它，
       两次 VM 运行截图 md5 完全相同（`1ec7d3f5…`）。**必须用 `auto run --theme light`**
       （CLI > pac.at，见 `auto run --help`）。→ 实证了 §4.2 P-9，
       **AC-03 的双端浅色验证一律走 `--theme`**，不得依赖 env。
    2. **`auto gen` 会改写 `examples/rust-workspace/`**：在 worktree 内它会
       ① 把 `.cargo/config.toml` 的 `target-dir = "../../target"` 改写成
       `"../../../../../../../../D:///autostack/.wt/lang-617/auto-lang/target"`
       （相对前缀拼绝对路径的畸形产物），② 把 `Cargo.toml` 里既有的
       `"019-video-app-back"` **替换**成 `"030-video-player-back"`（主检出里同一操作是
       **追加**、保留 019）。已 `git checkout` 还原这两个文件。
       → **T-05 必须先解决这一副作用**，否则 worktree 构建目标目录与 019 后端成员均被破坏。
    3. **断言脚本自身的一个静默通过缺陷已修**：emoji 检测最初用 `grep -E`，
       而 `\x{…}` 是 PCRE 语法，`-E` 下**永不匹配**→该检查会永远「通过」（AC-04 形同虚设）。
       改用 `grep -P`（本机 ugrep 7.8.4 支持 PCRE2）后正确报出 18 处。
       **纪律**：新增断言后必须先在违规样本上确认它真的会 FAIL。
- [x] **T-02 视觉系统与骨架**：`app.at` 拆分重写（顶栏 + 主体 + 播控条三段骨架）、
  `style` 配方集中定义、删按强调色 if 链与全部渐变/玻璃/阴影/emoji。
  验证：Vue 截图 = 扁平三段；AC-01/AC-02/AC-04 的 grep 断言通过。
  [✅ 已完成 2026-09-12] worktree @ `a8a98439e`（app.at 1220→980 行）。
  - **断言全绿**：AC-01 七项（backdrop/shadow/rounded-xl/animate/hover:scale/flex-wrap/
    渐变）全 ok；AC-02 两项 ok；AC-04 ok。仅 **AC-17 仍红**（`db.at` 的 12 处编造
    `resolution:`/`codec:`），**属 T-06 范围**，符合本任务「只验 AC-01/02/04」的约定。
  - **实现**：`style` 配方 4 条（`icon_btn`/`seg_btn`/`ctl_btn`/`tab_btn`）；
    18 处 emoji → VM lucide 闭集图标（monitor/info/eye/moon/sun/menu/minus/square/x/
    chevron-left/chevron-right/frame/file，均已核对在 84 名闭集内）；
    删按强调色 if 链、删应用内强调色选择器、删死数据 `current_bg`(16 行) 与
    `bg_gradient`(5 行)；`loop_label` 的 emoji 在 **model 值里**，一并清除。
  - **`auto gen` 通过**，无 error/warn。
  - **VM 渲染验证并修掉三个真实缺陷**（这是本任务最有价值的产出）：
    1. 队列 5 行渲染为**空条**——AURA 树里数据齐全，但 `flex-1`/`shrink-0` 在 VM 端
       不生效（`docs/style-coverage.md` parsed-only 清单），导致「图标列 + flex-1 文本列」
       两个子列塌成 0 宽。**改为单 `col` 承载两行文本**后正常。
    2. 视口高度分配失效、状态条上浮——`video` 在 VM 端是 fallback（零高度），
       给它加 `flex-1` 无效。**改为由真实容器 `col` 承载 flex-1** 后正常。
    3. 进度条渲染到播控条**外面**——`h-14` 容不下「时间行 + 进度行」两层。
       改 `h-16` 后正常。
  - **Vue 截图（一度阻塞，已补齐）**：会话中途 in-app browser 转为不可用——导航 `-3`
    → `browser screenshot activity capture failed for guest` → `browser guest not
    attached (webview not ready)`，`tabs.new()` 亦失败。浏览器恢复后已补拍，
    现 **`after_t02_vue_{dark,light}.png` + `after_t02_vm_{dark,light}.png` 四张齐备，
    双端布局一致**（顶栏 h-12 / 视口 / w-72 队列栏 / 播控条 h-16；队列行有真实
    标题+大小；图标全部渲染；唯一 primary 为「暂停」；进度条位于播控条内）。
    双端差异仅在视口内容：Vue 是 `<video>` 的黑色空画面（mock URL 离线）、
    VM 是 fallback 空容器——两者都由 T-03 的降级面板接管。
  - **操作纪律（本次新增，已写进 §10-15）**：dev server **不跨回合存活**
    （后台任务随回合结束被回收：日志干净、无错误输出、进程消失）。
    故改为**需要截图时在同一回合内临时起、用完即停**；若需长期供人访问，
    用 `DETACHED_PROCESS` 脱离 shell 进程组启动（已验证 200 存活）。
- **T-03 视口组件**：`viewport.at`（video 元素 + 覆盖层 + 错误/降级面板，去 `absolute`）。
  验证：Vue 截图；VM 截图显示降级面板而非纯黑（AC-11 前置）。
- **T-04 队列栏与播控条**：`playlist.at` + `controls.at`（尺寸收敛、单一 primary、
  PLAN-412 §5 白名单内取材）。验证：Vue/VM 双端截图；AC-02 计数复核。
- [x] **T-05 媒体服务与生成路由**（含决策点）：新建 `media_service.rs`，
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

- [x] **T-15 VM 原生播放门控 spike（Go/No-Go，决策件）**：在 `auto-lang` 内做一个最小 spike：
  用**已在依赖中的 `libloading`** 运行时加载 `libmpv-2.dll`，创建 mpv 句柄，
  用 render API 的**软件路径**把一帧 RGBA 送进 iced 视口，并测量
  ① 1080p 与 4K 下可达帧率、② 每帧拷贝耗时与进程内存、③ `Handle` 复用能否消除
  `renderer.rs:2889-2898` 记录的闪烁。同时评估 GL→`wgpu::Texture` 导出路径的可行性。
  参照实现：`ophymx/mpv-engine`。**产出 Go/No-Go 决策件**（含上述实测数字与
  「SW / GL / sidecar / 放弃」四选一结论），落入本文件 §9 与
  `docs/design/autoui/030-video-player.md`。**不得**在没有实测数字的情况下给出 Go。
  验证：spike 代码可跑（可临时存在，Go 后转为 T-16 的基础）+ 决策件含数字。
  [✅ 已完成 2026-09-12] **裁定 Go，通道 = SW**。全文见 **§9.14** 与
  `docs/design/autoui/030-video-player.md` §4（已按实测重写）。
  - 交付：`crates/auto-lang/examples/mpv_spike.rs`（`mpv`/`wgpu`/`selfcheck`/`all` 四个子命令，
    **可跑**）+ `mpv-spike` 特性 + 可选依赖 `iced_wgpu`（零新增编译单元）。
  - 关键数字：4K 端到端 **7.3 ms/帧**（mpv render 6.8 + 上屏 0.43），24 fps 预算占 17%；
    实时播放实测媒体时钟 1080p **0.958×** / 4K **0.997×**；RSS 150 MB / 493 MB。
  - **「Handle 复用消除闪烁」不可表达**：`iced_core::image::Id` 构造器私有 + 稳定 id 命中即
    不再上传 + 大帧必走异步（`MAX_SYNC_SIZE` 2 MB）+ 4K 装不下 atlas（2048）
    → 帧通道结构性绕开 iced 图像系统（三条证据见 §9.14）。
  - **GL 已排除**（wgpu 不公开外部内存导入）；**sidecar 被 SW 支配**。
  - 两处不利事实如实记录：SW 用不了零拷贝硬解（只能 `d3d11va-copy`）；通道 C 有极稀有长尾
    （4K 120 帧中 1 帧 957 ms），处置归 T-17。
  - **附带发现**：`mpv-spike` 特性下 lib 的 `--test` 目标会触发 rustc 1.98.0 ICE
    （`ui/mcp_server.rs:1718`，与 spike 无关）→ spike 落 example 目标，T-16..T-20 照此避开。
- [x] **T-16 native 动态库加载器**（Go 后）：把 `crates/auto-lang/src/ffi.rs:62-145` 的
  `TODO(Plan-212)` 落地为真正的 `libloading` 动态库加载；实现 libmpv 句柄
  生命周期、render context 创建/释放顺序（含 `LC_NUMERIC=C` 等 mpv 已知要求）、
  以及**库缺失时的静默降级**（回落到今日的诚实占位，不 panic、不黑屏）。
  验证：`cargo check -p auto-lang`；库缺失路径的单测。
  [✅ 已完成 2026-09-13] worktree @ `e13e63e59`，全文见 **§9.15**。
  - 交付 `crates/auto-lang/src/ui/mpv/`（`locale`/`loader`/`engine`/`frame` 四模块，
    feature `mpv-native`）+ `ffi.rs` 的真实加载路径 + `tests/mpv_engine.rs`（9 用例）。
  - **销毁顺序钉死在 `Drop`**：`render_context_free` → `terminate_destroy`（render.h:119 记 UB）。
  - **实测两分支**：有 DLL 时真实生命周期 + 真实渲染一帧进我们的缓冲全绿；
    无 DLL 时优雅 SKIP、9/9 通过、exit 0。
  - **实测发现**：`Vec<u8>` 自然对齐只有 1（本机 16），直接交给 mpv 会**静默**掉进
    「整帧拷贝」慢路径（render.h:393-404 要求 64）→ 已编码为 `FrameBuffer` 类型不变量。
  - 门禁 `cargo t`：**零新增红**（20 项全在基线集合内；基线 21 项中
    `ffi_dual_019` 是 P615-D3 并发 flake 本次恰好通过）；+2 测试数即本次新增用例。
  - **构建环境坑（会立刻咬到下一个人）**：`os error 5` 写 `.d` 失败的真因是
    **sccache 缓存 31G/上限 30G** 导致持续 trim；绕过 `RUSTC_WRAPPER= cargo …`。
- [x] **T-17 帧上屏通道**（Go 后）：按 T-15 结论实现帧到纹理的通道，
  **必须绕开每帧新建 `Handle`**（硬约束，见 §2.5 风险点 2）；
  镜像 `image_pipeline` 的既有形状（票据/rendition、latest-wins 代际门、有界缓存）。
  验证：目标分辨率下实测帧率与无闪烁截图。
  [✅ 已完成 2026-09-13] worktree @ 见提交；全文见 **§9.16** 与 design doc **§4.9/§4.10**。
  - 交付 `ui/mpv/channel.rs`（持久纹理 + 3 槽 staging 环 + `VideoLatestWins` 代际门
    + 回收超时丢帧策略 + 分类统计）与 `ui/mpv/present.rs`（WGSL 全屏 blit）；
    新 feature **`mpv-gpu`**（与 `mpv-native` 分开：引擎不碰 GPU，故仍可无 GPU 测试）。
  - 实测（真实 libmpv + 真实片源 + 离屏读回）：1080p **258–385 fps**（p50 0.61–0.67 ms）、
    4K **38.3–38.6 fps**（p50 4.35–4.54 ms）；`textures_created()` 恒为 **1**。
  - **T-15 的那条长尾在正式通道里复现**：4K 30 帧中 1 帧 **329–336 ms**，
    被 3 槽环吸收（0 丢帧、0 空白帧，总时长 0.4→0.78 s）→ §4.6 的长尾风险关闭。
  - 「无闪烁」做成**可证伪断言**（`tests/mpv_channel.rs`）：画面确实上屏 +
    **内容帧之间不得夹空白帧**（正是 renderer.rs:2889 的闪烁签名）+ 帧签名不恒定
    （非陈旧帧）+ 纹理数恒 1。
  - 9 用例全绿（`cargo test --features mpv-gpu --test mpv_channel`）。
  - **未接上的那段**：通道与 blit 都已就位，但**还没接进 VM 的 `video` 元素**
    （`render_support.rs:307` 仍 fallback）——那属 T-18/T-19。
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
15. **已自决（T-01/T-02 实操所得）**：dev server 与该环境的后台任务生命周期。
    实测：后台启动的 `npx vite` 在回合结束后被回收（日志无错误、进程消失、
    `curl` 转 000）。**纪律**：① 需要 Vue 截图时在本回合内临时启动、验证后即停；
    ② 需要长期供人访问时用 `DETACHED_PROCESS`（已实测可存活并返回 200）；
    ③ **不要把「dev server 还开着」写进交付说明**，除非确认它是 detached 启动的。
### 9.5 T-05/T-06 中途记录（2026-09-12，用户要求"先把播放列表改成真实的"）

- **用户现场反馈**：`localhost:3040` 上"列表和播放的视频实际都是假的，播放不出来"。
  属实——T-02 完成后队列仍是 T-01 时的 5 条 mock 字面量，`<video>` 也没有起播路径。
- **T-05 的真实阻塞（重要，影响所有 `crates/**` 任务）**：worktree 内**无法构建 `auto`**。
  `crates/auto-lang/Cargo.toml:110` 的 `autodown-core = { path = "../../../auto-down/autodown/packages/engine/rust" }`
  从主检出可解析（`D:/autostack/auto-down/...` 存在），从 worktree 解析为
  `.wt/lang-617/auto-down/...` → `os error 3`。`--config paths.*` 覆盖对 workspace 成员的
  path dep 无效。**正统解法（已从 618 的组确认先例）**：在组目录放兄弟 worktree
  `D:/autostack/.wt/lang-617/auto-down`（618 就是 `.wt/lang-618/auto-down`，**detached HEAD**，
  无需新分支）。**红线约束**：不得用 junction/symlink（Plan 529 事故），必须真 worktree。
  **代价**：worktree 各自建 target（实测 `.wt/lang-618` 5.9G、`.wt/os-012` 16G），
  首次 `cargo build -p auto` 是全量，需独立时段完成。
- **本轮采取的等效交付**：为不阻塞前端集成，先落 **dev harness** 替代 T-05 的服务端，
  接口形状与 range 语义与 T-05 一致，便于日后原地替换：
  - `tests/local-media-server.py`：`/scan`（递归白名单）+ `/stream/<id>`（**206** +
    `Accept-Ranges` + `Content-Range`，惰性分块 256KB，**不整文件入内存**）+ `/health`；
    端口 8099（**注**：8788 落在 Windows 排除段 8760-8859，绑定报 WinError 10013，
    选端口必须先查 `netsh int ipv4 show excludedportrange protocol=tcp`）。
  - `tests/gen_media_seed.py`：读 `/scan` 把真实条目烘进 `app.at`（**T-06 前端侧内容**，
    T-05 落地后改为运行时扫描）。
- **已实证**（非推断）：`/scan` 返回 **14 条**、`rel_dir` 分组 `["", "TV", "TV/Loki"]`、
  真实大小（7.3 MB / 1.7 GB / 2.3 GB / 4.3 GB…）；`/stream/7` 对 2.2 GB 文件返回
  **206 Partial Content + Content-Range: bytes 0-1023/2198645361**；浏览器实测
  `<video>` 载入 `http://127.0.0.1:8099/stream/1`，`readyState=4`、`duration=49.429`、
  **1920×1080**、无 error，且画面渲染出 `caelestia.mp4` 的真实首帧。
- **本轮两个自伤缺陷（已修，记录以免重犯）**：
  1. 用 Python `!r` 生成 `.at` 字符串 → 产出**单引号**，而 `.at` 把单引号当 **char 字面量**
     → `Lexer(UnterminatedChar)`。必须在生成器里显式双引号 + 转义。
  2. 写了 `"\u2014"` → `.at` **没有 `\u` 转义**，同样 `UnterminatedChar`。要用字面字符。
  修好后 `auto gen` 通过（GEN_EXIT=0）。
- **尚未完成（不当作已完成）**：
  1. `src/back/db.at` 仍含 12 处编造 `resolution:`/`codec:`，**AC-17 仍红**（前端已不消费，
     但未退役）。
  2. `controls: true` 是 **interim**：用户可用浏览器原生控件起播，但应用自己的 OSD
     仍是 mock（T-07/T-08 用受控契约替换）。
  3. 布局小疵：视口内容偏下、页面出现左侧滚动条（`h-screen` + 内容溢出），待 T-03/T-04 收拾。
  4. 头部默认值已改指真实首项，但"改后"截图未重拍（本轮截图是改前那一版）。

### 9.6 VM 端播放能力实测确认（2026-09-12，用户提问后）

- **问题**：Vue 端已能真实播放，VM 端能否播放？后台 ffmpeg 库适配做了没有？
- **实测（非推断）**：`auto run -r vm --theme dark` + MCP 快照 + 截图，worktree @ `f34d0f944`。
  - **AURA 渲染树里根本没有视频节点**：全树提及 video/image/surface/media 的 18 处，
    全部是 lucide 图标的 `[Image]` 占位、`"Video Player"` 文案与一行
    `onclick: .SelectVideo(item.id)`；**没有 `video` / `imagesurface` / media 节点**。
  - **截图**：视口区**纯黑**。真实队列（14 条、真实文件名与大小）、头部
    `caelestia / 7.3 MB`、Toast、`00:00 / 00:00` 时钟**都正常**——因为那些只是数据；
    唯独**画面没有任何渲染路径**。
  - 状态条仍写着「已暂停 · 点击视口或按空格播放」，但它**没有可播的东西**。
  - 证据图：`tests/screenshots/vm_cannot_play_evidence.png`（本地 gitignored）。
- **结论**：**VM 端不能播放视频**，且这不是本计划的缺陷——`video` 在 iced 侧仍为
  `render_support.rs:307` 的 fallback、`element_coverage.rs:449` 的 `NotYet`（§4.2 P-11/P-16）。
- **ffmpeg 适配进度：零。一行都没写。** 且需澄清一个前提——本计划**刻意没有选 ffmpeg FFI**
  （§2.5 裁定）：它会要求全部 11 个 `ubuntu-latest` CI 任务装 `libav*-dev` + C 工具链，
  而 A/V 同步、音频输出、HDR 色调映射 ffmpeg 都不提供、仍需自研。计划选的是
  **libmpv DLL 运行时加载**（`libloading` 已在本仓依赖中，`ffi.rs:62-145` 已有预留位置）。
- **VM 链各任务状态**：T-13（选型决策件）未做；**T-15（Go/No-Go 门控 spike）未开始**；
  T-16..T-20 为条件任务、未开始。且 T-15 所需的 spike **也要编译 Rust**，
  同样受 §9.5 记录的 worktree 构建阻塞（缺兄弟 `auto-down` worktree）影响。

### 9.7 更正：为什么 UI demo 的构建会牵扯 auto-down（用户质疑后查证）

- **用户质疑**：「视频播放器 demo 为什么会需要 auto-down 的 worktree？这俩应该没有依赖关系。」
  **质疑成立**：030-video-player 自身**零** autodown/markdown 用法（`grep` 为空）。
- **真实链条**（逐环实证）：
  1. 要落 T-05/T-07 必须构建 **`auto` 二进制**；
  2. `crates/auto/Cargo.toml:26` → `default = ["ui-iced", "python", "autodown"]`
     —— **autodown 在 auto 的默认特性里**（Plan 040 裁定①）；
  3. `autodown = ["auto-lang/autodown"]` → `crates/auto-lang/Cargo.toml:110`
     `autodown-core = { path = "../../../auto-down/autodown/packages/engine/rust", optional = true }`；
  4. 该相对路径从主检出解析为 `D:/autostack/auto-down/...`（存在），
     从 worktree 解析为 `.wt/lang-617/auto-down/...`（不存在）→
     `failed to load manifest for dependency autodown-core`。
- **更正我先前的两处不准确表述**：
  1. 我上一轮说这份依赖"从没被启用"——**错**。`auto` 的默认特性就启用它。
  2. 我说"cargo 只是要读 manifest"——方向对但不够准：实测（最小复现）
     `cargo metadata --no-deps` **通过**、`cargo metadata` 与 `cargo build` **失败**，
     即失败发生在 **resolve 阶段**；且 `--no-default-features` **也不能规避**
     （EXIT=101，同样报错），因为 workspace 解析要覆盖所有成员的可能特性选择。
     **结论：无法用特性开关绕过。**
- **根因定性**：这是 **Plan 040 的假设 × Plan 529 的布局**之间的缺口，不是 demo 的需求。
  `crates/auto/Cargo.toml` 自己写着：「autodown 提为 default——auto-down 是旗舰消费方
  （**path 依赖恒在本机检出**）」——即"构建发生在主检出"这一前提。
  Plan 529 引入 worktree 布局后，该前提在工作树内不成立。
- **既有解法（仓库现状）**：`auto-down` 的 worktree 列表显示
  **618 / os-012 / os-013 / auto-down-063 全部建了兄弟 worktree**，
  其中 618 用 **detached HEAD**（无需新分支）。故这是仓库的既有惯例解法。
  **红线**：只能用真 worktree，不得用 junction/symlink（Plan 529 事故）。
- **三条可选路径**（需用户裁定）：
  (a) **建兄弟 worktree**（既有惯例；成本 1.2G 检出 + 首次全量构建）；
  (b) **按 Plan 040 的原假设走**：Rust 侧改动在主检出构建与验证
      （与"实现只在 worktree"的纪律冲突，但与该 plan 的明文前提一致）；
  (c) **根治**：让跨仓 path 依赖在两种布局下都可解析（单独立项，属仓库基建）。

### 9.8 裁定：Rust 侧改走主检出（方案 b）—— 用户 2026-09-12 决定

- **用户裁定**：采用 §9.7 的 **(b)** —— `crates/**` 的改动在主检出**编写、构建与验证**；
  `examples/**` 的改动继续在 worktree `D:/autostack/.wt/lang-617/auto-lang`。
  **这是对 AGENTS.md「实现只在 worktree」纪律的一次显式、经批准的偏离**，
  理由是 Plan 040 的明文前提（"path 依赖恒在本机检出" = 构建发生在主检出）
  与 Plan 529 的 worktree 布局冲突，而该冲突的根治（§9.7 (c)）属独立基建立项。
- **可行性已实测**：
  1. 主检出 `cargo metadata` **EXIT=0**（worktree 里同一条命令失败的 resolve 步骤在此通过）；
  2. `cargo build -p auto` 增量 **33.17s** 完成（`Finished dev profile`）。
     → **这把 (b) 的成本从"首次全量构建（数十分钟量级）"降到"增量几十秒~数分钟"**，
     是本决策成立的关键数据；worktree 方案则无法复用该缓存。
- **实测踩到的坑（务必记住，否则每次构建都会失败）**：
  `auto run -r vm` 被 `proc.terminate()` 终止时会**留下孤儿子进程**，
  它持续占用 `target/debug/auto.exe`，导致下一次 `cargo build -p auto` 报
  `failed to remove file ...\target\debug\auto.exe / 拒绝访问 (os error 5)`。
  **构建前必须先 `tasklist | grep auto.exe` 并 `taskkill //F //PID <pid>`。**
  （本次即因此先失败一次，杀掉 PID 25496 后 33s 通过。）
- **落地方式**：Rust 改动直接落在主检出（即 master 工作树），随本计划一并提交；
  worktree 分支 `plan-617-dev` 保留 examples 侧改动；折入时两者在 master 汇合。

### 9.9 T-05 完成（2026-09-12，主检出 @ `2763657fb`）

- **交付**：
  - `crates/auto-lang/src/ui/media_service.rs`（新，429 行）：`index_directory`
    （**递归**、扩展名白名单、**不跟随 symlink/junction**、MAX_DEPTH=32 兜底）、
    自然排序（`S01E02` 在 `S01E10` 前）、`blake3(relative_path)` 令牌 id
    （绝对路径不出后端）、`resolve_root`（explicit → `AUTO_MEDIA_ROOT`）、
    `parse_range` / `StreamPlan`（200/206/416 语义）、`content_type`、`entry_path`。
  - `crates/auto-man/src/api_gen.rs`：新增 `MEDIA_SERVICE_HANDLERS` 常量并发出
    `/api/media/scan`、`/api/media/stream/:id`；生成的 back crate 加
    `tokio-util`（`io` feature）用于 `ReaderStream` **惰性分块 256KB**；
    索引用 `std::sync::OnceLock` 每进程建一次（**零新增依赖**）。
- **验证（全部实机，非推断）**：
  - `cargo t media_service` → **7 passed / 0 failed**（递归+白名单、自然序+rel_dir、
    令牌不泄路径、缺根报错不 panic、Range 全表面、状态/长度、content-type）。
  - `cargo test -p auto-man --lib` → **285 passed / 0 failed**（生成器未回归）。
  - 生成后端在主检出构建通过（54s 首建 / 2.8s 增量）并实跑：
    `/api/media/scan` → **14 条**、`rel_dir = ['', 'TV', 'TV/Loki']`、
    最大 `Loki.S02E06…mkv` = **5,751,764,883 B**；
    `Range: bytes=0-1023` → **206** + `content-type: video/x-matroska` +
    `accept-ranges: bytes` + **`content-range: bytes 0-1023/5751764883`**；
    越界 → **416**；未知 id → **404**。
- **顺带发现的仓库既有缺陷（重要）**：生成的 back crate 解析 **axum 0.7.9**
  （`examples/rust-workspace` 是独立 workspace，有自己的锁；主仓是 0.8.9）。
  axum 0.7 的参数语法是 **`:id`**，写成 `{id}` 会被当**字面量**且**静默 404**。
  实测对照：静态路由 200 / `/api/media/stream/{id}` 404 / `/api/media/stream/:id` 200。
  → **既有的 `/api/__auto/media/{id}/{revision}` 路由就是这个 bug，一直 404**
  （即 Plan 547 的图像 rendition URI 在该形状下取不到文件）。本计划已在自己的路由上
  改用 `:id` 并在代码注释里标注；**修既有那条路由属独立事项**，登记为债务候选。
- **T-05 决策点落定**：
  1. route 发出条件：**无条件发出**（`generate_main_rs` 签名里没有 root_dir，
     按前端源码判断需要改签名与全部调用点）。代价是多 2 条惰性路由 + 1 行依赖；
     **登记为债务**（待出现第二个媒体应用时再做按需门控）。
  2. `tower-http` **不加 `fs` feature**——改用 `tokio-util::io::ReaderStream`，
     语义更直白且不牵动既有依赖的 feature。
  3. **不直接复用 `image_viewer.rs`**（它在 `auto-man`，生成的后端 crate 够不着；
     且它面向 `img` 标签）。改为在 `auto-lang` 内**镜像其形状**实现。
- **尚未接线**：前端此刻仍指向 Python harness（8099）。切到真后端 =
  把 `gen_media_seed.py` 的产物换成运行时 `Http.get("/api/media/scan")`，
  并让 `video_url` 用 `id` 拼 `/api/media/stream/<id>`（属 T-06 收尾）。

### 9.10 裁定回退：恢复 worktree-only（用户 2026-09-12 决定，取代 §9.8）

- **用户裁定**：「我们还是用独立的 worktree 去做修改吧」——**取代 §9.8 的 (b)**。
  理由（§9.10 的实测观察支撑）：主检出正在被**多计划共用**，
  未提交改动会产生提交裹挟风险。
- **落地：按 auto-plan-work 的「Dependency worktree | Same group」建兄弟 worktree**
  （§9.7 方案 (a)，仓库既有惯例）：
  - `git -C D:/autostack/auto-down worktree add D:/autostack/.wt/lang-617/auto-down --detach`
    （**detached**，无需新分支——`auto-down-dev` 已被 `.wt/auto-down-063` 占用；
    照 618 先例）；
  - **关键验证**：worktree 内 `cargo metadata` **EXIT=0** —— §9.5/§9.7 记录的
    `autodown-core` 跨仓 path 解析失败**已消除**。即 worktree 现在可自洽构建。
  - **红线遵守**：用的是真 worktree，**没有**用 junction/symlink（Plan 529 事故）。
- **与已提交工作的关系**：T-05 的 Rust 改动已在 master 提交（`2763657fb`），
  `git merge master` 已并入 `plan-617-dev`，故 worktree 同时具备示例侧（T-01/02/06）
  与 Rust 侧（T-05）改动。**此后所有 `crates/**` 改动一律在本 worktree 内编写与构建。**
- **构建**：worktree 无 target 目录 → 首次为**全量构建**。已以 `DETACHED_PROCESS`
  脱离 shell 进程组启动（`cargo build -p auto`，pid/日志见会话记录），
  使其**跨回合存活**（本环境后台 Bash 任务会随回合结束被回收，§10-15）。

### 9.11 观察：主检出已被多计划共用（本次实测，非推测）

- 会话开始时的系统快照显示 `crates/` 下**干净**；会话进行中出现 **19 个脏文件**
  （`ui/iced/renderer.rs`、`ui/terminal/*`、`vm/native*.rs`、`ui_gen/rust.rs`、
  `terminal_pixel_*.png` 金标、`auto-man/src/rust_ui.rs` 等），时间戳
  13:01/13:03/13:24/13:57 均落在本会话窗口内，且有**新增测试文件**
  `terminal_input_tests.rs` —— 属实质代码改动，非构建产物。
- 这些改动**不在任何 worktree 中**（`.wt/os-012`、`.wt/os-013` 对应文件干净）。
- **推断（非证据）**：很可能其它计划也撞上了同一个「worktree 无法构建」的限制，
  因而同样采取了「Rust 改动落主检出」的变通。**能证明的是"有人直接改了主检出且不在
  worktree 里"，不能证明是谁或动机。**
- **风险**：多写者向同一主检出写未提交改动 → **提交裹挟**；枢纽文件
  （`ui/mod.rs`、`api_gen.rs`、`ui_gen/vue.rs`）尤甚。
  **本计划纪律**：提交一律**逐文件显式 add** 并先 `git diff` 确认只含本计划内容，
  **禁用 `git add -A`**。
- **建议（供决策，不在本计划内实施）**：§9.7 (c) 根治跨仓 path 依赖的价值因此上升；
  属仓库基建，跨所有计划的开发方式，应单独立项。

### 9.12 状态对账与账目补齐（2026-09-12，用户询问"现在什么状态"后）

- **发现的账目偏差**：`current_step` 与实际不符——§9.5/§9.9/§9.10 的散文记录里
  T-05 早已完成，但任务勾选未翻转。**本轮补齐：T-05 标 [x]，`current_step: 2 → 3`**
  （已完整交付的：T-01、T-02、T-05；T-06 为进行中，见下）。
- **构建阻塞确认解除（决定性验证）**：worktree 内
  `target/debug/auto.exe --version` → `auto 0.1.0+v0.4.2-402-g31655b73e`，
  **hash 与 worktree HEAD 一致**，证明该二进制确由 worktree 源码构建。
  → 「worktree 无法构建」已终结，`crates/**` 的活此后全部在 worktree 内闭环。
  （副作用记录：本人脚本的 `open(log,"w")` 与 cmd `>>` 抢同一文件导致
  `BUILD_OK` 标记未写成——**仅在日志里留下 2 行 `另一个程序正在使用此文件`，
  构建本身成功**。下不为例：完成标记改写到**另一个**文件。）
- **T-06 实际状态：进行中（半成品，勿当完成）**：
  - 已做：递归扫描出的 **14 条真实条目**（真实文件名/大小/相对目录）已进队列，
    浏览器实测可播放（`readyState=4`、`duration=49.429`、1920×1080）。
  - **未做**：`app.at` 里仍有 **15 处 `127.0.0.1:8099`** —— 队列是
    `tests/gen_media_seed.py` 把扫描结果**烘死**进源码的，**前端仍挂在 Python
    harness 上**；T-05 已验证的 Rust 后端（206/`content-range`）**尚未被前端使用**；
    `db.at` 未退役（AC-17 仍红）。
  - → 收尾 = 运行时 `Http.get` 取列表 + `video_url` 由后端给出 + 删两个 Python 脚本。
- **并发态势更新**：主检出 `crates/` 脏文件 **19 → 22**，且主检出 `auto.exe`
  在 **17:15** 被重写 —— **另一 agent 此刻正在主检出构建**。本人严守逐文件 add。
- **服务全部离线**（:3040 Vite / :8099 harness / :18080 后端），用户所看 demo 未运行。

### 9.13 阶段 1 折入（2026-09-12，用户要求；master `3546f9567`）

- **折入内容**：`plan-617-dev` → master（`--no-ff`，6 文件 / +513 −614）。
  覆盖 T-01（基线实勘）、T-02（扁平清爽骨架）、T-05（媒体服务与生成路由）、
  T-06（前端运行时扫描队列，删 Python 脚手架）。
- **折入前门禁**：worktree 内 `cargo t --no-fail-fast`。
  **存在较多红，但经判定均为预存，非本分支引入**：
  1. `plan492_m4_tests::…c2_param_msg_declaration_both_tracks_alive`
     —— **以还原法隔离证明**：把本分支唯一的 `vue.rs` 改动 `git checkout master --` 还原后
     **该测试仍红**；在主检出（无本分支改动）亦红。
  2. `ui::app_registry::tests::scan_examples_ui_curation_set`
     —— 断言 `left` 含 `031-image-viewer` 而期望集不含；系 commit `9c6c27e86`
     （031-image-viewer 进桌面）**未同步该清单**的既有漂移。
  3. 其余红域（`ui::layout::tests::*` 12+、`aura_view_builder`、`desktop_protocol::coverage`、
     `lucide_icon_coverage_manifest_all_hit`、`ffi_dual_019_dep_layout_invariants`）
     与本分支改动**领域不相交**（本分支只动：handler/Init 的 async 生成、媒体路由、
     新建 media_service）。
- **诚实标注的局限**：**未能建立「相对 master 的零新增红」这条硬基线**——因为
  主检出同时被另一 agent 持有 38 个未提交文件，取不到干净基线。故本次折入的结论是
  「已检查的红均可解释为预存」，**不等同于**已证明零新增红。
  （计划仍为 `executing`，非终态；最终 review 需在干净基线上重跑门禁。）
- **并发安全**：折入前后主检出的 38 个脏文件**始终未被我提交或修改**；
  合并文件与它们**零交集**（折入前已 `comm -12` 核对）。本计划提交一律逐文件显式 `add`。
- **worktree 保留**：`D:/autostack/.wt/lang-617/auto-lang`（分支 `plan-617-dev`）
  与兄弟 `D:/autostack/.wt/lang-617/auto-down` **继续保留**用于后续阶段
  （T-03/04/07..15 与 VM 线）。折入后分支已落后 master，后续开工前需再 `merge master`。
- **下一阶段焦点（用户指定）**：**VM 版能否启动并播放** → T-15 门控 spike
  （libmpv DLL 运行时加载；Go/No-Go 必须带实测数字）。

### 9.14 T-15 门控 spike 完成——**裁定 Go**（2026-09-12，实测件）

- **结论：Go，通道 = SW（libmpv 软件渲染后端 + 持久 staging buffer → 自有 `wgpu::Texture`）。**
  完整决策件（含四选一理由与风险）已写入
  [`docs/design/autoui/030-video-player.md`](../design/autoui/030-video-player.md) §4。
  **T-16..T-20 可以开工。**
- **取得 libmpv（本机原本一无所有）**：下载
  `shinchiro/mpv-winbuild-cmake` 的 `mpv-dev-x86_64-20260903-git-69e63f425a.7z`
  （sha256 `fac135c6…80faef`），用 `bsdtar`（libarchive 支持 7z，本机无 7z CLI）解出
  `libmpv-2.dll`（120 MB，ffmpeg 静态内链）+ `include/` + `libmpv.dll.a`。
  **不入库、CI 不依赖**（运行时依赖而非构建期依赖）；解析序 = `AUTO_MPV_LIB` → exe 同目录 → 系统路径。
- **AC-18 的实测数字**（环境：Win11 / RTX 4060 Ti / iced 实选 Vulkan；样本 1080p x264 与 4K HEVC HDR；
  两个探针**互相独立**：Python ctypes 求实时能力，Rust spike 求每帧代价，同一份 DLL）：

  | 指标 | 1080p | 4K HEVC HDR |
  |---|---|---|
  | mpv `render()` per-frame（`ADVANCED_CONTROL` 生产形状） | **0.60–0.68 ms** (p95 0.74–0.95) | **6.74–6.88 ms** (p95 10.4–10.7) |
  | 吞吐上界（untimed） | 869–989 fps | 90–93 fps |
  | **实时播放**（`ao=null` 音频时钟） | 23.3 fps / 内容 24 fps，时钟 **0.958×** | 23.6 fps / 内容 24 fps，时钟 **0.997×** |
  | 进程 RSS（实时） | 150 MB | 493 MB（纯软解峰值 1042 MB） |
  | CPU | 5.6 ms/帧 | 54.6 ms/帧（`d3d11va-copy`） |
  | 上屏每帧代价（**采用通道 C**） | **0.28–0.29 ms** p95 0.76–0.82 | **0.41–0.44 ms** p95 0.86–0.98 |
  | 对照：通道 A 持久纹理 + `write_texture` | 0.67–1.02 ms | 3.87–4.17 ms |
  | 对照：通道 B 每帧新建纹理（= iced `Handle` 等价） | 1.22–1.27 ms，离群 2/120 | 4.28–4.36 ms，p95 **18.9–19.5 ms**，离群 **8/120** |

  **端到端（4K）≈ 6.8 + 0.43 = 7.3 ms/帧**，占 24 fps 预算 **17%**、60 fps 预算 **44%**。
  **`--release` 复核**（排除「debug 把 wgpu 代价算高了」的质疑）：门控 A 1080p 0.66 / 4K 6.67 ms
  （与 debug 一致）；门控 B 通道 C 更好——**0.132 ms @1080p / 0.244 ms @4K**，
  `map_async+unmap` 0.043 ms；**排序不变**（重活都在预编译的 libmpv 与 GPU 驱动里，不在我们的
  Rust 代码里）。但**通道 C 的长尾在 release 下同样出现**（4K max 951 ms）⇒ 真实行为，
  非 debug 产物，T-17 必须处置。
  证据日志：`t15_gate_debug.log` / `t15_gate_release.log`（本地，含完整输出）。
- **「Handle 复用能否消除闪烁」的答案是：该问法本身不成立**（这是本轮最有价值的架构发现）。
  三条源码级证据：① `iced_core::image::Id` 的构造器与字段**全私有**，仓外无法构造
  「同 id、不同像素」的 Handle；② 每帧新 id 必走 `load_image` 未命中，而同 id 命中即
  **不再上传**（`cache.rs:94-101`）——两条路都不通；③ 大帧必然走异步慢路径
  （`MAX_SYNC_SIZE = 2 MB` 而真实帧 8.29/33.18 MB）+ 4K 装不下 atlas（`MAX_SIZE = 2048`）。
  故帧通道**结构性绕开** iced 的图像系统（自持纹理 + `iced_widget::shader::Program`），
  闪烁成因随之被移除。**本仓既有的 `cached_media_handle`（P547）对静态图有效，对帧流
  必然不命中**——那条修复不能直接搬到视频上。
- **GL 路径已评估并排除**（不是「难」而是「在本仓架构下不可达」）：mpv 的 GL 后端要一个
  当前 GL 上下文，而 iced 走 Vulkan；两 API 共享纹理需 `VK_KHR_external_memory_win32` +
  `GL_EXT_memory_object_win32`，而 **wgpu 不公开外部内存导入**（只有 `wgpu-hal` 层
  `create_texture_from_hal`/`as_hal` 这类 unsafe 接口），且即便写通仍慢于已实测的 SW 通道。
  sidecar 则被 SW 全面支配（多一次跨进程 33 MB/帧拷贝 + A/V 同步与音频输出要自研）。
- **两处不利于 SW 的实测事实（如实记录）**：① SW 后端**用不了零拷贝硬解**
  （`hwdec=d3d11va` 实测 `hwdec-current: no`，只能用 `d3d11va-copy` 回读），4K 纯软解 RSS 达 ~1 GB；
  ② 通道 C 有**极稀有长尾**（4K 120 帧中 1 帧 957 ms、1080p 中 3 帧 242 ms），T-17 须以 2–3 槽
  staging 环 + 单帧丢弃处置，长尾定位是 T-17 的第一件事。
- **一个必须记住的构建期发现（rustc ICE）**：打开 `mpv-spike` 特性后编译 **lib 的 `--test`
  目标**会触发 rustc 1.98.0 的 ICE（`collect_and_partition_mono_items`，查询栈指向
  `ui/mcp_server.rs:1718` 的 iterator chain，**与 spike 代码无关**）；同特性下 `--lib`（rlib）
  与 `example` 目标均正常。故 spike 落为
  `[[example]] name = "mpv_spike" required-features = ["mpv-spike"]`（默认档/CI 不编译）。
  **T-16..T-20 的测试须照此避开 lib 测试目标。**
- **交付物（worktree 内，`crates/auto-lang`）**：
  `examples/mpv_spike.rs`（可跑：`mpv` / `wgpu` / `selfcheck` / `all` 四个子命令；
  `selfcheck` 证明缺库走 `Err` 而非 abort，即 AC-20 的降级入口）；
  `Cargo.toml` 新增 `mpv-spike` 特性 + 可选依赖 `iced_wgpu`（**零新增编译单元**——它本就是
  iced 默认渲染器的依赖）与 `[[example]]` 声明。
- **交付物（worktree 内，`scripts/mpv_spike/`）**：AC-18 的「实时播放能力」与「进程 RSS」
  两类数字来自 Python ctypes 探针（Rust example 只覆盖每帧渲染/上屏代价），
  已一并收编入仓以保证可复现：`sw_probe.py`（实时/逐帧/内存，口径见其 README）、
  `audio_probe.py`（Dolby 音轨实测）、`README.md`（前置/用法/口径说明）。
  DLL 路径经 `AUTO_MPV_LIB` 覆盖；**`libmpv-2.dll` 本身不入库、CI 不依赖**。
- **数字对机器负载敏感（如实记录）**：上表取自相对空闲时段。有并发负载时复测同一 4K 场景，
  `render()` p50 5.7→12.8 ms、CPU 54.6→97.5 ms/帧，**媒体时钟仍 0.999×** ⇒
  「能否实时」这一判定本身有相当余量。
- **降级路径已实证**（AC-20 前置）：`Library::new` 对不存在的库返回 `Err`（`LoadLibraryExW failed`），
  解析不出即返回 `None` 交回调用方走今日的诚实占位，**不 panic、不黑屏**。
- **Vue 端那条「解不了」的音轨在 VM 侧实测可解**（这原是用户要求 VM 播放的主要动机）：
  单独探针实测（`ao=null`，只读 mpv 自己报告的属性）`Loki.S02E01…mkv` 得到
  **`audio-codec-name: eac3`**、`audio-params: {"samplerate":48000,"channel-count":6,
  "channels":"5.1(side)","format":"floatp"}`、`audio-bitrate: 768000`、`aid: 1`、
  AUDIO_RECONFIG 事件 3 次 —— **即 Chromium 判为 video-only 的那条 Dolby 音轨，
  libmpv 内链的 ffmpeg 正常解码**。**Atmos 对象元数据如预期坍缩为 5.1**
  （实测确认，非推断）；HDR 色调映射由 mpv 承担（未单独实测质量，T-18 验收时看画面）。
  音频输出 mpv 自带（`ao` 为本机设备），故 **T-18 不引入 `cpal`**。
- outcome: **Go**；next: **T-16**（native 动态库加载器，落 `ffi.rs:62-145` 的 `TODO(Plan-212)`）。
- **门禁 `cargo t` 实跑（worktree @ `b621edda0`）**：`4812 tests run: 4791 passed, 21 failed,
  109 skipped`（47.7s）。**21 项红全部可解释为预存/环境，且本改动不可能影响它们**——
  **完整代码 diff vs master 只有两行式内容**：`Cargo.toml`（+20 行：可选依赖 `iced_wgpu`、
  `mpv-spike` 特性、`[[example]]` 声明）与**新增的** `examples/mpv_spike.rs`；
  **`crates/auto-lang/src/**` 零改动**（`git diff master --stat -- crates/auto-lang/src/` 为空），
  `Cargo.lock` 未变动（→ 既有特性的解析结果逐字节不变）。故 lib 测试目标的行为不可能改变。
  逐项归因：
  1. `plan492_m4_tests::…c2_param_msg_declaration_both_tracks_alive` —— §9.13 已用**还原法**
     隔离证明为预存；
  2. `ui::app_registry::tests::scan_examples_ui_curation_set` —— 实测输出 `left` 含
     `031-image-viewer`、期望集不含，与 §9.13 记录的 `9c6c27e86` 清单漂移**逐字一致**；
  3. `ui::layout::tests::*`（12 项）与 `ui::desktop_protocol::coverage::tests::…`、
     `ui::aura_view_builder::plan055_…`、`lucide_icon_coverage_manifest_all_hit`、
     `ffi_dual_019_dep_layout_invariants` —— 均在 §9.13 已列的**红域**内（桌面布局/协议域），
     与本计划的改动领域不相交；
  4. **`ui::iced::renderer::tests::external_config_poll_hot_apply_loopsafe`（§9.13 未逐条列出的一项）
     经实测定为并发 flake，非本改动引入**：该测试读写**真实用户配置路径**
     （`desktop_config::desktop_config_path()`），全档并发时被其它进程写回
     （实测失败值是 `theme_source` 从测试写入的 `"manual"` 变回 `"system"`）；
     **隔离复跑 3/3 全绿**。与 §9.11 记录的「主检出被多计划共用」同一成因。
  - **诚实标注的局限（与 §9.13 同）**：主检出仍被另一 agent 持有未提交改动，**无法取到干净基线**，
    故本次结论是「已检查的红均可解释、且改动面为零源代码」，
    **不等同于**已证明「相对干净基线零新增红」。

### 9.15 T-16 完成——native 动态库加载器与 libmpv 引擎生命周期（2026-09-13）

- **交付**（worktree `plan-617-dev` @ `e13e63e59`）：新增 `crates/auto-lang/src/ui/mpv/`
  （feature **`mpv-native`**，只挂 `ui`、**不牵 iced**，故可独立编译与测试）：
  - `locale.rs` —— `LC_NUMERIC` 必须是 `"C"`。这不是可选项：`client.h:147` 是硬前提，
    且同 header 把它列为 `mpv_create()` 返回 NULL 的原因之一（`:479`）。用 **CRT 自己的
    `setlocale`**（零新增依赖——本仓没有 `libc`），并且**只动 `LC_NUMERIC`**、不用
    `LC_ALL`（那会连带改掉调用方的其它分类）。LC_NUMERIC 的**数值平台相关**，
    按 MSVC/ucrt(4) 与 glibc/musl(1) 分别写死并注明依据，不凭印象。
    为什么要主动做：Rust std 从不调 `setlocale`，但进程内其它 C 库（本仓有 oniguruma、
    sqlite3）可能调 `LC_ALL` 把它一起改掉。
  - `loader.rs` —— 解析序（`AUTO_MPV_LIB` → exe 同目录 → 无）与符号表。**显式路径存在
    才采用，不存在就直接降级为 `None` 而不回落**——否则「路径指错了」会表现为
    「莫名用了另一个版本的运行库」，比直接降级更难排查。符号在加载期**一次性**解析成
    裸函数指针（`MpvSymbols`），从而绕开 `libloading::Symbol<'lib, T>` 的生命周期约束。
  - `engine.rs` —— 句柄与 render context 的生命周期，**本任务的核心**。销毁顺序钉死在
    `Drop` 里：`mpv_render_context_free()` → `mpv_terminate_destroy()`；反序是 UB
    （`render.h:119` 原文 "If this doesn't happen, undefined behavior will result"）。
    另两条同源约束照做：`render.h:111`（**先建 context 再 loadfile**，否则 video 初始化
    失败或退回自建窗口 VO）与 `:114`（一个 core 同时只允许 1 个 context——重复建返回
    错误）。「同线程 create/free」只在 API < 1.105 存在（`render.h:96-107`），故按运行时
    `mpv_client_api_version()` 判断，旧版跨线程释放时打 warning 但**仍按正确顺序释放**
    （不释放就销毁 core 是 UB，比旧版线程风险更严重）。
  - `frame.rs` —— 帧目标缓冲，把 mpv 的 **64 字节对齐**要求（`render.h:393-404`）
    编码进类型。**这条是 T-16 的一个实测发现**：`Vec<u8>` 的自然对齐只有 1
    （本机实测 16），把 `vec.as_mut_ptr()` 直接交给 mpv 会**静默**掉进「整帧拷贝」慢路径
    ——T-15 的 spike 里已经踩过一次，所以这次把它变成类型不变量（`FrameBuffer::as_target`
    是唯一无需 `unsafe` 的入口）。
- **缺失即降级（AC-20 的落地）**：`MpvEngine::new()` 返回
  `MpvUnavailable::{NoLibrary | LoadFailed | CreateFailed | InitFailed | RenderContextFailed}`，
  全程不 panic、不黑屏；`MpvEngine::is_available()` 供渲染层廉价探测以便决定降级。
  **刻意区分「本机没有运行库」（正常分支）与「显式指定却加载失败」（缺陷）**。
- **`ffi.rs` 落地 `TODO(Plan-212)` 的加载部分**：`register_c_function` 现在真的
  `LoadLibrary`，**失败是硬错误而非静默 no-op**（旧实现 `log::info!("Would load ...")`
  假装成功，问题会在日后以「莫名错误结果」的形式浮现）；新增 `with_library` 作用域访问器
  （`Symbol` 借用库，无法外传，故用闭包收口）与 `loaded_libraries()`。
  **明确未做**：任意 C 签名的实参编组仍属 Plan-212 未完成范围，本次**未**随附（已登记债务）。
  与 `ui/mpv` 的**刻意反差**：那里的缺库是「本机没这个能力」→ 降级；这里的缺库是
  「有人点名要它」→ 报错。
- **测试（9 用例，集成目标而非 lib 单测——避开 lib `--test` 的 rustc ICE，见 P617-D1）**：
  `crates/auto-lang/tests/mpv_engine.rs`。**两分支都实测**：
  - **有 DLL**：`libmpv: …libmpv-2.dll (client API 2.5)`，真实生命周期（建 context →
    拒绝重复建 → Drop → 再建）与**真实渲染一帧进我们的缓冲**（320×180，断言渲染后缓冲
    非全零）全绿；
  - **无 DLL**（`env -u AUTO_MPV_LIB`）：优雅 SKIP、**9/9 通过、exit 0**、不 panic。
  解析序的纯函数形式（`resolve_library_with`）让解析用例**不碰进程环境**，避免测试间竞态。
  `ffi::tests` 更正 2 例 + 新增 1 例：旧用例传 `target/hal.dll`（一个**从不存在的路径**）
  却断言成功——只因加载当时是 no-op；现改用**必然可加载**的路径（测试可执行文件自身），
  并新增「不可加载库必须报错且不留半状态（函数未注册 / id 未推进 / 库未记录）」。
- **门禁 `cargo t`**：`4814 run / 4794 passed / 20 failed / 109 skipped`（55.0s）。
  基线（§9.14）为 21 项，本次 **20 项且完全落在基线集合内**——差异的那一项
  `ffi_dual_019_dep_layout_invariants` 是 `KNOWN-DEBT` 记载的 **P615-D3 并发 flake**
  （本次恰好通过）。测试数 +2 即本次新增的两例。
  → **零新增红**，且本计划自己的新增用例全绿。
- **构建环境踩坑（重要，会立刻咬到下一个人）**：`cargo` 报
  `error writing dependencies to …deps\<crate>-<hash>.d: 拒绝访问 (os error 5)`，
  一次构建里几十个 crate 同时失败。**逐层排除**：不是权限（我用 shell 手写同名文件成功、
  手动单跑 rustc `--emit=dep-info,metadata` 到同一目录也成功）、不是沙箱
  （关掉沙箱同样失败）、不是 target 目录损坏（**换全新 target 目录同样失败**）、
  不是孤儿进程（已清理上一会话残留的 lang-617 `cargo run` + :8330 后端，无效）。
  **真因：sccache**。`SCCACHE_DIR=D:\autostack\.sccache` 已 **31 G**，而
  `SCCACHE_CACHE_SIZE=30 G` → 超出上限、持续 trim，硬链接/写入竞态就表现为 `os error 5`。
  **绕过**：`RUSTC_WRAPPER= cargo …`（清空 wrapper，本次所有验证均在此环境下取得）。
  **根治**：清理该缓存目录或调大 `SCCACHE_CACHE_SIZE`——但它被多个计划共用，本计划不擅自改动。
- outcome: pass；next: **T-17**（帧上屏通道：持久 staging 环 + `copy_buffer_to_texture`，
  并处置 T-15 记录的长尾）。T-16 已把「帧 → 我们的内存」这一段打通（`FrameBuffer` +
  `render_sw_frame`），T-17 要接的是「我们的内存 → wgpu 纹理」。

### 9.16 T-17 完成——帧上屏通道（持久纹理 + staging 环 + 无闪烁断言）（2026-09-13）

- **交付**：`crates/auto-lang/src/ui/mpv/channel.rs` + `present.rs`，新 feature **`mpv-gpu`**。
  与 `mpv-native` **分开门控**是有意的：引擎（libmpv+libloading）不碰 GPU，故
  `mpv-native` 在没有 GPU/无窗口的环境里仍能独立编译与测试；通道与 blit 才要 wgpu。
  `mpv-spike` 现为 `["mpv-gpu", "ui-iced"]`。
- **通道形状**（T-15 §4.1 的裁定落地）：
  `mpv SW renderer →（直接写）持久映射 staging（3 槽环）→ copy_buffer_to_texture → 持久 wgpu 纹理 → 全屏 blit → 目标`。
- **实现里三条值得记住的结论**：
  1. **行距取 256 对齐**，一个 stride 同时满足两边：wgpu 的 `copy_buffer_to_texture`
     要求 `bytes_per_row` 是 256 的倍数，mpv 要求 64 的倍数（256 是 64 的倍数）。
     实测常用宽度（1920/2560/3840）本就落在 256 上，故无 padding 开销。
  2. **片元着色器强制 `alpha = 1.0`**：mpv 的 `"rgb0"` 第 4 字节是**未初始化垃圾**
     （`render.h` 原文 "the '0' component contains uninitialized garbage"）。若当
     alpha 用，画面会随机变半透明乃至整帧「消失」——**那就是闪烁本身**。
  3. **背压一律丢帧、不阻塞**：取不到空槽时**定向**回收（只 `poll` 该槽的
     submission，不串行化整个 GPU），回收带超时（默认 4 ms），**超时即丢帧**。
     纹理始终保留上一帧内容，故丢帧**不会**产生空白。
  4. 代际门镜像 `image_pipeline.rs` 的 `MediaLatestWins`（同一 `(generation, seq)`
     二元组 + `accept_*` 返回 bool 并计丢帧），并比它多一处：**渲染前判一次、
     上屏前再判一次**——渲染期间用户可能已经 seek。
- **实测（`tests/mpv_channel.rs`，真实 libmpv + 真实片源 + 离屏读回）**：

  | 通道尺寸 | 源 | 端到端帧率（含逐帧读回） | 上屏代价 p50 / p95 | max（离群） | 丢帧 |
  |---|---|---|---|---|---|
  | 1920×1080 | `caelestia.mp4` | **258–385 fps** | **0.61–0.67 / 0.85–0.98 ms** | 6.8–55.4 ms（2/60） | 0 |
  | 3840×2160 | `Loki.S02E01…4K HEVC HDR` | **38.3–38.6 fps** | **4.35–4.54 / 5.57–5.65 ms** | **329–336 ms（1/30）** | 0 |

  端到端数字含逐帧读回（真实播放器不会做），故是**保守下界**；对照 24 fps 片源，
  1080p 余量 10× 以上、4K 余量 1.6×。
- **T-15 的那条长尾在正式通道里复现了**：4K 30 帧中 1 帧 `copy_buffer_to_texture`
  耗时 **329–336 ms**（T-15 门控 B 量到 957 ms，同一现象）⇒ 它不是 spike 的测量假象。
  **而 3 槽环把它吸收掉了**：`reclaim_timeout=0`、无丢帧、无空白帧，表现只是那一轮
  总时长 0.4 s → 0.78 s。这正是选 3 槽（一槽在被 mpv 写、一槽在 GPU 拷贝、一槽空闲）
  的意义。**§4.6 的长尾风险由此关闭。**
- **「无闪烁」做成了可证伪断言**（不靠眼看，写在 `tests/mpv_channel.rs`）：
  逐帧上屏后读回像素，断言 ① 画面确实上了屏（非全黑）；② **没有「内容帧之间夹
  空白帧」**——这正是 `renderer.rs:2889` 描述的「画面消失又出现」的闪烁签名；
  ③ 帧签名不恒定（纹理在更新，不是命中缓存后的陈旧帧）；④ 全程
  `textures_created() == 1`（**纹理只建一次**，结构性反闪烁）。
- **一个测试设计上的教训（已修正并留痕）**：我最初写「背压必然丢帧」的断言，
  结果红了——因为小尺寸拷贝毫秒级完成、环根本顶不满，而 T-15 那条长尾是**驱动
  偶发停顿、无法按需复现**。改成断言**策略与上界**（每次调用只能返回「已提交」或
  「按策略丢帧」，且总耗时必有界），并额外断言 `reclaim_waits > 0`（证明回收路径
  确实被走到）。「一定丢帧」那种断言只能靠运气通过，是坏测试。
- **门禁 `cargo t`**：`4814 run / 4793 passed / 21 failed / 109 skipped`。21 项
  **全部落在基线集合内**（19 项稳定红 + 两项已记录的并发 flake：
  `ffi_dual_019_dep_layout_invariants`〔P615-D3〕与
  `external_config_poll_hot_apply_loopsafe`〔§9.14 实测定为并发 flake，隔离复跑 3/3 绿〕）。
  测试数仍为 **4814**（本任务新增的用例在 `required-features = ["mpv-gpu"]` 的
  独立目标里，不进默认档）→ **AC-20 保持**，且零新增红。
- outcome: pass；next: **T-18**（VM 侧播放控制与 §2.3 受控媒体契约对齐，音频由 mpv 承担）。
  **交接提醒**：通道与 blit 管线刻意**不依赖任何 iced widget 类型**（只依赖 wgpu），
  就为了让 T-19 能直接把它搬进 `iced_widget::shader::Program` 的
  `Pipeline`/`Primitive`——`Pipeline::new(device, queue, format)` 恰好交出
  `&wgpu::Device`/`&wgpu::Queue`，`Primitive::render` 交出 `&mut CommandEncoder`
  与 `&TextureView`。**但截至 T-17，`video` 元素仍是 `fallback`**：通道就位 ≠ 已接上。

## 11. 新会话开工须知（Handoff，2026-09-12）

> 本会话很长了，以下是把「不读完整 §0–§10 也能安全接手」所需的操作要点集中在此。
> 一句话状态：**Vue 端已可用（真实队列 + 真实播放）；VM 端仍完全不能播放（无渲染路径）。**
> 用户指定的下一焦点：**VM 版能否启动并播放** → **T-15**。

### A. worktree 布局（关键，勿重新踩坑）
- 实现 worktree：`D:/autostack/.wt/lang-617/auto-lang`，分支 `plan-617-dev`。
- **兄弟依赖 worktree：`D:/autostack/.wt/lang-617/auto-down`（detached HEAD）——必须存在。**
  缺它则 worktree 内 `cargo` **连 resolve 都过不去**（`crates/auto-lang/Cargo.toml:110` 的
  `autodown-core` 跨仓 path 依赖解析失败）。修复命令：
  `git -C D:/autostack/auto-down worktree add D:/autostack/.wt/lang-617/auto-down --detach`
- **红线**：不得用 junction/symlink 代替（Plan 529 事故）；必须是真 worktree。
- 折入（§9.13）后 `plan-617-dev` 落后 master → **开工前先 `git merge master`**。

### B. 构建
- worktree 内**可自洽构建**（已验证 `auto --version` 的 hash 与 worktree HEAD 一致）；增量 **5–13 秒**。
- **构建前必清孤儿进程**：`auto run -r vm` 被 `terminate()` 时会留子进程占住
  `target/debug/auto.exe`，导致 `failed to remove file … os error 5`。
  先 `tasklist | grep auto.exe` → `taskkill //F //PID <pid>`。
  **注意**：机器上常有**别的计划**的 auto.exe（主检出 / auto-os / lang-618/619）——
  动手前先按 ExecutablePath 确认归属，别误杀。本计划上一会话就留过一个
  lang-617 的 `cargo run`（`examples/rust-workspace/030-video-player-back`）+ :8330 后端。
- **`os error 5` 的另一个（更常见的）真因是 sccache**，不是孤儿进程：
  `SCCACHE_DIR=D:\autostack\.sccache` 已 **31G** 而 `SCCACHE_CACHE_SIZE=30G` →
  超限持续 trim → 硬链接/写入竞态 → 一次构建里几十个 crate 同时报
  `error writing dependencies to …deps\<crate>-<hash>.d: 拒绝访问 (os error 5)`。
  **判别**：换个全新 target 目录**同样**失败 ⇒ 与 target 状态无关；
  **绕过**：`RUSTC_WRAPPER= cargo …`（本次 T-16 全部验证在此环境下取得）。
  根治需清理缓存或调大上限（被多计划共用，未擅自改）。
- 门禁 `cargo t`。**注意基线是红的且不干净**（§9.13：无法取到干净基线，因为主检出被并发占用）。

### C. 运行
- `AUTO_MEDIA_ROOT='E:\Video' auto run` → 真后端 :8330 + Vite :3030。
- **`pac.at media_root` 尚未接通**，只认该环境变量（债务）；未设时 `/api/media/scan`
  返回 `[]`（诚实空态）而非报错。
- 后台任务不跨回合存活（§10-15）：要长期可访问须 `DETACHED_PROCESS` 启动。

### D. 提交纪律（重要）
- **主检出有另一个 agent 在并发写**（本会话期间脏文件 19→41 且仍在涨）。
- **一律逐文件显式 `git add <path>`；禁用 `git add -A`**；提交前 `git diff` 确认仅含本计划内容。
- 计划书（勾选/frontmatter/§9.x）**留在 master 主检出**，不要只改 worktree 的副本。

### E. 任务状态
- **已完成并折入 master `3546f9567`**：T-01、T-02、T-05、T-06。
- **已完成**：**T-15（门控 spike，裁定 Go，§9.14）**、**T-16（native 加载器 +
  引擎生命周期 + 降级，§9.15）**——均已提交（worktree `b621edda0` / `e13e63e59`）。
- **未完成**：T-03（`viewport.at` + VM 有信息降级面板）、T-04、T-07（受控媒体契约）、
  T-08、T-09、T-10、T-11、T-12、T-13、T-14、**T-17..T-20**。
- **VM 链进度**：T-16 ✅ → **T-17（下一步）** → T-18 → T-19 → T-20。

### F. T-15 已完成的门控定义与结论（全文见 §9.14 与 design doc §4）
- 路径与通道：**libmpv DLL 运行时加载；通道 = SW**（软件渲染后端 + 持久 staging buffer →
  自有 `wgpu::Texture`）。**明确不选 ffmpeg FFI**（理由见 §2.5），**GL 已评估排除**
  （wgpu 不公开外部内存导入），**sidecar 被 SW 支配**。
- **已实测（AC-18 满足）**：4K 端到端 7.3 ms/帧（24 fps 预算 17%）；实时播放媒体时钟
  1080p 0.958× / 4K 0.997×；RSS 150 MB / 493 MB；每帧上屏代价 0.28 ms @1080p / 0.43 ms @4K。
- **`Handle` 复用问法不成立**（三条源码级证据）→ 帧通道必须绕开 iced 图像系统。
- **T-16 的起点**：spike 的解析序与 `selfcheck` 降级路径已可复用；
  `crates/auto-lang/src/ffi.rs:62-145` 的 `TODO(Plan-212)` 是落地位。
- **T-17 的第一件事**：定位并处置通道 C 的稀有长尾（4K 120 帧中 1 帧 957 ms），
  用 2–3 槽 staging 环（映射中的缓冲不可提交）+ 单帧丢弃。
- **spike 复现**：`AUTO_MPV_LIB=<...>\libmpv-2.dll cargo run -p auto-lang
  --features mpv-spike --example mpv_spike -- all`；**DLL 不入库**，需按 design doc §4.8 自取。

### G. 已知坑与债务（细节见对应 §）
- **axum 参数语法**：生成的 back crate 解析 **axum 0.7**（`examples/rust-workspace` 是独立
  workspace，有自己的锁）。参数是 **`:id`**，写 `{id}` 会被当字面量**静默 404**；
  **既有 `/api/__auto/media/{id}/{revision}` 路由正是此 bug**（§9.9）。
- `NextVideo`/`PrevVideo` **仍是 mock**（10 处硬编码假 URL）→ 属 T-08。
- `db.at` 未退役 → **AC-17 仍红**。
- `controls: true` 是 **interim**；应用自身 OSD 尚不驱动播放（T-07/T-08）。
- 媒体路由**无条件发出**（未按需门控，登记为债务）。
- 主检出被多计划共用（§9.11）：建议单独立项根治跨仓依赖，或把"新建 worktree 时同时建
  兄弟 `auto-down`"标准化。

### H. 本会话结束时的残留（接手前先看清）
- `:8330` 后端可能仍在运行（`auto run` 的残留子进程）；`:3030` Vite 已随回合结束被回收。
- worktree 可能有 1 个未提交文件（`auto gen` 产物 / rust-workspace 改动）——
  **开工前先 `git status` 并处理**，不要直接叠加改动。
