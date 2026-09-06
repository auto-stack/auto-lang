# 鸿蒙生态战略（ArkTS 主线；AutoUI 下一主战场）

**日期**: 2026-09-06
**状态**: Draft（骨架，待扩充）
**关联**: [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md)（伞形总战略）、
Design [06](../06-code-generation.md)（a2ark 现状）、GOAL-009（shell 肌肉）、
[c-ecosystem-strategy](c-ecosystem-strategy.md)（Lite 设备交汇）

---

## 0. 定性与排序

AutoUI 在 Web/Desktop 之后的**下一主战场**，优先级高于 Android/iOS。论据：中国市场份额
增量；AI 原生语言与鸿蒙同为新生态，窗口期重合、竞争少；鸿蒙是唯一让 UI 轨、C 轨（Lite
设备）、桌面 shell 轨（GOAL-009）三线交汇的战场。

## 1. 现状（2026-09-06）

a2ark（`ui_gen/ark/`，5 模块，12+ widget 用例）Design 06 定性 Complete，但**停留于旧
AutoUI 形态**（最后实质改动在 Plan 051 时代之前）——当前 shadcn schema / Tailwind 清单 /
store-actions 契约均未对齐，**能否跑通待证**。重启该线的第一个 plan 是 rebaseline：
恢复"能跑通 + 与 vm/vue 双端对拍"，再谈 widget 覆盖扩展。

## 2. 四条路线裁决（合作层原则应用）

| 路线 | 裁决 | 理由 |
|---|---|---|
| **ArkTS 源级发射** | ✅ 主线 | 官方源语言：DevEco 调试/预览/热重载/分发免费继承；ArkTS 为 TS 方言，与 a2ts 共享转译机制 |
| 仓颉语言 | 观察，不做 | 生态/库/工具链成熟度远逊 ArkTS，两者长期并存；等其成为 NEXT 事实默认或出现华为侧合作拉力再评估 |
| ABC 字节码 | 不做 | ArkCompiler 内部 IR：无稳定规范、随 DevEco 漂移、丢失源级生态与调试信息。比"不做 LLVM"（C 战略）更强的否定 |
| 机器码发射 | 不做 | 同上 ×10 |

## 3. OpenHarmony 独立 OS（深开鸿模式）

- **定性：产品发行版决策，不是语言/工具链决策**。技术前提与 ArkTS 线完全同栈
  （OpenHarmony 应用框架即 ArkUI）。
- 推进逻辑：① AutoUI on HarmonyOS rich-device 完整覆盖 → ② GOAL-009 的 shell 肌肉
  （509 Smithay 宿主线练出的"消费帧的宿主"能力）平移 → ③ "OpenHarmony + AutoOS 外壳"
  届时为水到渠成的产品化选项。
- 现在只登记为鸿蒙线**终态方向**，不立项。
- 交汇记录：OpenHarmony Lite（MCU 级设备）接 C 战略消费面二（LVGL/嵌入式线）。

## 4. 待扩充

- [ ] rebaseline plan 范围（旧 12+ widget 用例对当前契约的差距清单）
- [ ] widget 覆盖矩阵与推进顺序（对齐 23 元素 schema 体系）
- [ ] DevEco/API 版本锚定策略（跟进节奏与兼容窗口）
- [ ] vm / vue / arkui 三臂对拍闸门定义（复用 autoui-verifier 范式）
- [ ] Lite 设备与 C 轨嵌入式线的交汇细化
- [ ] OpenHarmony OS 终态方向的成立条件量化（何时升格立项）
