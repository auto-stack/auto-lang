# 移动生态战略（Android + iOS 统一 Jetpack Compose）

**日期**: 2026-09-06
**状态**: Draft（骨架，待扩充）
**关联**: [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md)（伞形总战略）、
Design [06](../06-code-generation.md)（a2jet 现状）

---

## 0. 已定裁决（2026-09-06 讨论）

1. **Android + iOS 统一基于 Jetpack Compose**：a2jet（`ui_gen/jet/`，11 模块、Material3、
   完整 Android 工程生成）为现成发射器；Compose Multiplatform（CMP）一份发射臂覆盖
   Android / iOS / JVM 桌面。**不做 SwiftUI 臂**——那是全新发射臂 + Swift 工具链 +
   双份 parity 维护。
2. **排序：鸿蒙之后**（鸿蒙分战略 §0 的三线交汇论据）。
3. 重启时同样**先 rebaseline**：a2jet 与 a2ark 一样停留于旧 AutoUI 形态，需先对当前
   shadcn schema / Tailwind / store 契约恢复跑通。

## 1. 风险与假设记录（在案，不改裁决）

- **CMP on iOS 成熟度**：Skia 渲染的原生观感、包体、稳定性。对策：若出现 iOS 高保真
  需求，用**局部原生桥**补缺，不开 SwiftUI 臂。
- **前提假设**："iOS 中国市场份额逐渐下降"——若 Auto 未来确立全球化战略，此前提需重审
  （届时 SwiftUI 权重回升的讨论重新打开）。

## 2. 待扩充

- [ ] a2jet rebaseline 范围与差距清单
- [ ] CMP 版本选型与 iOS 真机验证通道
- [ ] 移动端 parity 闸门定义（vm/vue/compose 对拍）
- [ ] 发布管线（签名、应用市场、CI）
- [ ] 与鸿蒙线的发射器共享面盘点（Material3/ArkUI 的 token 映射）
