# mpv 门控 spike 探针（PLAN-617 T-15）

这两个 Python 探针是 [PLAN-617](../../docs/plans/617-030-video-player-real-rebuild.md)
**T-15 门控决策件**中「实时播放能力」与「音频解码」两类数字的来源。
Rust 侧的每帧代价探针是 `crates/auto-lang/examples/mpv_spike.rs`。

用 Python 而不是 Rust 是有意的：AC-18 要的三类数字里，「真实播放能否跟上媒体时钟」
与「进程 RSS」需要**长时间真实播放**，用 ctypes 直接驱动 `libmpv-2.dll` 最快；
Rust 侧则专门量「每帧渲染/上屏代价」。两者跑的是**同一份 DLL**，结论互相印证。

## 前置

1. 一个 libmpv 构建（`libmpv-2.dll`）。本仓**不收录**该二进制（它是运行时依赖，
   不是构建期依赖，CI 也不得依赖它——AC-20）。Windows 上可取
   [`shinchiro/mpv-winbuild-cmake`](https://github.com/shinchiro/mpv-winbuild-cmake)
   的 `mpv-dev-x86_64-*.7z`（内含 `libmpv-2.dll`）。
   解压工具：本机无 7z CLI 时可用 `C:\Windows\System32\tar.exe`（bsdtar 支持 7z）。
2. 设 `AUTO_MPV_LIB` 指向该 DLL；视频样本路径按下方变量覆盖。

```bash
export AUTO_MPV_LIB='D:\path\to\libmpv-2.dll'
```

## sw_probe.py — 实时播放 / 逐帧渲染 / 内存

```bash
# 4K 实时：媒体时钟应 ≈1.0x（跟不上就 <1.0）
python scripts/mpv_spike/sw_probe.py "<video.mkv>" 3840 2160 \
    --seconds 10 --hwdec d3d11va-copy --ao null --advanced --no-block --start 00:05:00

# 吞吐上界（untimed，mpv 跑满）
python scripts/mpv_spike/sw_probe.py "<video.mkv>" 3840 2160 --seconds 8 --untimed
```

关键量：`-> effective fps`、`render() call` 分位、`media clock ... x realtime`、
`RSS`、`CPU ... ms CPU/frame`。

**口径说明（读数字时必看）**：

- `--ao null` 保留 mpv 的**音频时钟**，因此是**真实播放节奏**；省略 `--ao` 会关掉音频，
  mpv **脱离时钟自由跑**，此时的高 fps 是吞吐上界而**不是**实时能力。
- `--advanced` 用生产形状（只在 `mpv_render_context_update()` 报告新帧时才 render）；
  `--no-block` 关掉 mpv 为对齐显示时间而做的等待，使测出的 `render()` 时间是**纯
  缩放/色彩转换/写入**代价，而不是等待。
- **SW 后端用不了零拷贝硬解**：`--hwdec d3d11va` 实测不生效（`hwdec-current: no`），
  只有 `d3d11va-copy`（解码后回读）可用。

## audio_probe.py — Dolby/Atmos 音轨是否真能解

```bash
python scripts/mpv_spike/audio_probe.py      # 可用 AUTO_SPIKE_VIDEO 覆盖样本
```

读 mpv 自己报告的属性（不做推断）：预期 `audio-codec-name: eac3`、
`audio-params` 为 5.1(side)/48000，即 **Chromium 判为 video-only 的那条轨在 mpv 侧可解**；
Atmos 对象元数据会坍缩为 5.1。
