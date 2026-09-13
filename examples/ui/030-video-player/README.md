# 030-video-player

真实可播的本地视频播放器示范（Vue：`auto run`；VM：`auto run -r vm`）。

> 本示例在 PLAN-617 里做过一次**真实化重做**：此前是「外壳真实、内核全假」
> ——`<video>` 从未被播放过、OSD 全是常量、队列是写死的字面量。现在队列来自
> 后端对媒体根的递归扫描，播放由受控契约驱动真实元素，界面只显示实测值。

## 跑起来

```bash
# 媒体根已在 pac.at 写死为 E:\Video（用户 r2 裁定方案 (a)）；
# env 可覆盖：AUTO_MEDIA_ROOT 优先于 pac.at。
export AUTO_MEDIA_ROOT='E:\Video'     # 可选；Windows: $env:AUTO_MEDIA_ROOT='E:\Video'

auto run          # Vue 模式：真后端（axum）+ Vite
auto run -r vm    # VM/iced 模式（原生播放需另开 feature，见下）
```

VM 模式想看到**真实画面**（而不是降级面板）：

```bash
cargo build -p auto --features mpv          # 打开 mpv-native/mpv-gpu/mpv-widget
AUTO_MPV_LIB=<dir-with-libmpv-2.dll> auto run -r vm
```

VM 模式想看到**真实媒体库**（队列与 Vue 端同一份数据）——先起生成后端，
再给 VM 一个 HTTP 基址（T-10 起支持相对 URL 展开）：

```bash
cargo run --manifest-path examples/rust-workspace/030-video-player-back
AUTO_HTTP_BASE=http://127.0.0.1:8330 auto run -r vm
```

未开 mpv feature 或找不到运行库时，视口给出显式降级说明
（「本后端未启用原生播放」），**不留黑屏**；未设 `AUTO_HTTP_BASE` 时
媒体库显示诚实空态文案。

## 能做什么（都是真的）

- **队列**：递归扫描媒体根的白名单文件（`mp4/m4v/webm/mkv/mov/avi`），
  按相对目录分组、组内自然排序（`S01E02` 在 `S01E10` 之前）；
  条目显示真实文件名与真实字节数。
- **播放**：`<video>` 真实加载并播放。起播/暂停、seek、音量、静音、倍速、
  上下曲**全部作用于元素属性**（不是只改界面文字）。
- **时间/进度**：由 `timeupdate` / `loadedmetadata` 回灌驱动；没有预置常量。
- **本地文件**（Web 端）：「打开本地视频文件」打开系统文件对话框，
  选中即用 object URL 播放——不经过后端、不经过队列（T-09）。
- **音轨标注**：播放超过 1s 仍解不出音频字节（Dolby Digital Plus 等
  Chromium 无解码器的编码）时，界面**主动**标注「音轨不受支持」（AC-16b）；
  解得出时不说任何音频信息。
- **错误可见**：解码/加载失败时视口用错误面板替换纯黑，文案来自 `MediaError`；
  后端不可达 / 媒体根目录不存在 / 目录为空 三种空态文案各不相同（T-10）。
- **深浅主题**：顶栏开关真实切换（会同步 `<html>` 的 `dark` 类）。

## 不做什么（如实说明）

- **不显示推不出的元数据**：分辨率/编码/码率在不解复用时无从得知，
  数据模型里根本没有这些字段。时长/画面尺寸只来自元素回灌。
- **不声称有声**：Matroska 容器能解，但 Dolby Digital Plus (E-AC-3/Atmos)
  音轨不被 Chromium 的 FFmpeg 构建支持，那类文件表现为**有画面无声音**
  ——界面不假装有音频，并按上节的探针实测给出反向标注。
- **本地文件选择仅 Web 端**：`file_picker.vue` 依赖浏览器 File API；
  VM/iced 端按钮点击给出「仅 Web 端可用」的降级文案，不做假动作。
- **VM 端 `AUTO_HTTP_BASE` 覆盖面**：get/post/put/delete/patch/json/request/
  auth 臂支持相对 URL 按基址展开；download/stream/SSE 臂**未**覆盖
  （这些原生的 URL 惯例是绝对地址）。

## 验证

```bash
# 生成器侧契约与兼容性（在仓库根）
cargo t vue_gen

# Vue 端真实播放 e2e（需先 auto run + AUTO_MEDIA_ROOT）
cd tests && pnpm install && pnpm exec playwright test
#   注意：e2e 强制使用本机真实 Chrome（Playwright 自带 Chromium 不带
#   H.264/AAC，videoWidth 恒为 0），见 tests/playwright.config.ts 注释。

# VM 端诚实降级冒烟
AUTO_BIN=<path-to-auto> node tests/vm-smoke.mjs
```

截图（双端、深浅）在 `tests/screenshots/`：
`after_t08_vue_{dark,light,playing,mkv}.png` 与 `after_t08_vm_degrade.png`。
