# 030-video-player

真实可播的本地视频播放器示范（Vue：`auto run`；VM：`auto run -r vm`）。

> 本示例在 PLAN-617 里做过一次**真实化重做**：此前是「外壳真实、内核全假」
> ——`<video>` 从未被播放过、OSD 全是常量、队列是写死的字面量。现在队列来自
> 后端对媒体根的递归扫描，播放由受控契约驱动真实元素，界面只显示实测值。

## 跑起来

```bash
# 指向真实媒体根（也可以由 pac.at 的 media_root 提供）
export AUTO_MEDIA_ROOT='E:\Video'     # Windows: $env:AUTO_MEDIA_ROOT='E:\Video'

auto run          # Vue 模式：真后端（axum）+ Vite
auto run -r vm    # VM/iced 模式（原生播放需另开 feature，见下）
```

VM 模式想看到**真实画面**（而不是降级面板）：

```bash
cargo build -p auto --features mpv          # 打开 mpv-native/mpv-gpu/mpv-widget
AUTO_MPV_LIB=<dir-with-libmpv-2.dll> auto run -r vm
```

未开该 feature 或找不到运行库时，视口给出显式降级说明
（「本后端未启用原生播放」），**不留黑屏**。

## 能做什么（都是真的）

- **队列**：递归扫描媒体根的白名单文件（`mp4/m4v/webm/mkv/mov/avi`），
  按相对目录分组、组内自然排序（`S01E02` 在 `S01E10` 之前）；
  条目显示真实文件名与真实字节数。
- **播放**：`<video>` 真实加载并播放。起播/暂停、seek、音量、静音、倍速、
  上下曲**全部作用于元素属性**（不是只改界面文字）。
- **时间/进度**：由 `timeupdate` / `loadedmetadata` 回灌驱动；没有预置常量。
- **错误可见**：解码/加载失败时视口用错误面板替换纯黑，文案来自 `MediaError`。
- **深浅主题**：顶栏开关真实切换（会同步 `<html>` 的 `dark` 类）。

## 不做什么（如实说明）

- **不显示推不出的元数据**：分辨率/编码/码率在不解复用时无从得知，
  数据模型里根本没有这些字段。时长/画面尺寸只来自元素回灌。
- **不声称有声**：Matroska 容器能解，但 Dolby Digital Plus (E-AC-3/Atmos)
  音轨不被 Chromium 的 FFmpeg 构建支持，那类文件表现为**有画面无声音**
  ——界面不会假装有音频信息。
- **「打开本地视频文件」尚未接线**（计划内 T-09）：当前按下会如实提示
  「本地文件浏览尚未接线」，不做「已模拟打开」这种不实陈述。
- **VM 端媒体库为空**：VM 的 `Http.get` 需要绝对 URL，而扫描接口是相对路径；
  VM 因此显示空态。属平台侧缺口（已登记债务）。

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
