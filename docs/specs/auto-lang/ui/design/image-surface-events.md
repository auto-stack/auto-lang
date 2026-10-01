# ImageSurface 运行时事件（loaded/error）

> PLAN-095 T-02 交付（auto-musk-095-dev@fce48632e）。本文是 ImageSurface
> onload/onerror 在 VM/iced 轨的权威行为规则。

## 事件语义

- **loaded**：资产解码终态 Ready **且** 下一拍（其间必有渲染帧消费
  rendition）单次派发。实参按 handler 声明参数数投影：`Some(0)→[]`、
  `Some(1)→[rev]`、`Some(2)→[w,h]`、`Some(3)→[w,h,rev]`；声明数超出
  0..=3 响亮跳过（plan-576 D4 口径，不给 0 形参塞载荷）。
- **error**：终态失败（损坏/缺票/过期/decode 失败）单次派发；`fn(str)`
  声明收原因短文（如 "unsupported image format"）；0 形参不塞载荷。
  永不先 loaded 再对同一请求报失败。
- **单次门**：per 订阅（handler 身份 + src）。重绘/轮询/同 ticket 重复
  构树不重复通知；换 src 或卸载即退订（表项随视图存在性生死），重挂载
  = 新订阅（A→B→A 会再次通知）。
- **无回调组件**零改动兼容（照常显示，不入通知表）。

## 唤醒机制（订阅时门的根因修复）

- 订阅门在 update 时求值，毫秒级 decode 窗口落在两次求值之间会被永久
  错过（空闲应用永不被唤醒）。
- 因此：媒体注册表带**代次计数**（queue/transition/publish/fail 四点
  bump）；renderer 挂**桌面级 50ms 轮询**（`poll_media_wake`），仅代次
  前进时 yield `__media_tick`（空闲零消息，无 update 风暴）；消费接缝在
  update 尾部 sweep（`media_notify_sweep`），快速门带代次比较。
- 不在 view 构建/paint 期重入 handler：派发走 `on_with_input_for`
  合成事件通道（与 __mcp 合成事件同路）。

## 验收锚

`image_surface_contract`（三轨静态合同）+ auto-musk
`canvas-runtime-probe.mjs` media-events 探针（单发/可定位/A→B 零污染）
+ media-soak（有界性）。
