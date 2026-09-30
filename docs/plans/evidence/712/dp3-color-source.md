# DP-3 探针：mpv SW 输出色彩协商与色偏源定位（T-02）

- 日期：2026-09-30
- 环境：本机 `AUTO_MPV_LIB=D:/autostack/tools/mpv/libmpv-2.dll`
  （mpv-dev-x86_64-20260903-git-69e63f425a），样本 `E:\Video\caelestia.mp4`。
- 探针载体：`crates/auto-lang/tests/mpv_engine.rs` 的
  `probe_color_negotiation_dump` / `probe_color_output_tunable`
  （env 门控，SKIP 语义；决策工件非门禁）。
- 本机无 ffprobe/ffmpeg，媒体元数据改由 libmpv 属性面读取（等效信息源）。

## 探针 1：协商结果 dump

```
video-params（解码面）     = bt.709 matrix / limited range / bt.709 primaries / bt.1886 gamma
video-out-params（输出面） = 同上（pixelformat yuv420p → rgb0 由 render 期转换）
转换选项现值               = sws-scaler lanczos / sws-allow-zimg yes /
                             video-output-levels auto / target-prim auto / target-trc auto
```

## 探针 2：协商可调性差分（同帧 = seek 0.5s absolute+keyframes，time-pos 复核）

| 配置 | 全帧均值 R/G/B |
|---|---|
| [1] 默认协商 | 218.27 / 212.95 / 198.08 |
| [2] pre-init `target-prim=srgb` + `target-trc=srgb` | 218.27 / 212.95 / 198.08（逐位同） |
| [3] `video-output-levels=full`（pre-init 与运行时两试） | 218.27 / 212.95 / 198.08（逐位同） |

运行时设 `target-prim` 经 `mpv_set_property_string` 报
`unsupported format for accessing property`（target-* 是 pre-init 选项面）。

**定谳：SW render 路径完全不吃色彩协商——输出字节恒为源签名传递函数
（SDR = BT.1886 ≈ γ2.4）编码的显示域值。**

## 色偏机制闭环（结合 DP-2）

- 上屏链恒等（DP-2）：屏幕字节 ≡ mpv rgb0 字节 = **2.4 编码域**。
- Chromium `<video>`：YUV→RGB 后按 **sRGB（≈2.2）编码**输出。
- 两端显示同一内容时，VM 中间调系统性偏亮（0.5 显示线性处 ≈ +4/255，
  朝白方向收敛、黑点不动）——与双端像素对照阈值（8/255）同量级。
- 修复（T-03 落地）：`present.rs` shader 把采样值还原字节域后按 2.4 解码、
  交 Srgb 目标按 sRGB 重编码——端到端与 Chromium 字节域对齐；
  像素级回归 `mpv_channel::present_renormalizes_bt1886_bytes_to_srgb_domain`
  （135→~128、0→0、255→255）已绿。

## 对 E-3 实录的判读注记

auto-os 会话实录的"内容白区偏暗偏暖"（251,237,230 vs 铬层白 251,248,242）
**不能**由传递函数错配单独解释（后者方向为偏亮、且白点收敛）——该实录的
取样是"视频帧 vs app 铬层"而非同帧双端对照。T-07 的 AC-03 双端同帧像素
探针（web vs VM 同一帧）是最终裁决：若归一后仍见同量级暖偏，残差只能来自
内容差或 mpv 色度上采样，届时按计划 §10 提交用户裁定。
