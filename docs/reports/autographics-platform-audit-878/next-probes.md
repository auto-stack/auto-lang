# 4. 后继探针建议（next-probes）

XP-1 的具体最小输入、触面 owner、正负 oracle 与规模建议。依据：XP-0 审计事实
（[dependency-map](dependency-map.md) / [platform-seams](platform-seams.md)）。
**本文件不创建探针、不预设通过**；XP-1..5 仍需各自正式 Plan。877 Gallery 未交付
内容只作候选引用（G01/G02 几何研究资产可等价地用当前冻结 scene/模型替代），XP-1
不依赖 877 完成。

## 1. readiness 总判定

| 维度 | 判定 | 阻塞项 |
|---|---|---|
| audit_complete | **complete**（覆盖集逐项分类闭合：222 第一方文件 + 10/10 metadata + SDK 闭包 15/17 + auto-ui 触面） | 无 |
| next_probe_readiness | **limited** | R-01..R-04（下表），每项均有绕行或缩小范围方案 |

`limited` 而非 `ready` 的原因：驱动语义执行的最重未测假设（SDK 宿主栈跨端可编译性，
U-01）无法在静态审计内消解，且本机无 android/ios 工具链；但这不阻塞 XP-1 的
**render-only** 首步与两项零成本预探（P-0a/P-0b）。

## 2. XP-1 具体入口建议

### P-0 预探（零代码、静态可完成，可先于 XP-1 立项）

| ID | 内容 | 入口 | oracle | 成本 |
|---|---|---|---|---|
| P-0a | SDK 栈裁剪面核对：对 auto-lang-core 67 直接依赖逐项标注"VM 运行必需/宿主服务/可选"，产出 wasm 候选裁剪清单 | `.sdk-source/crates/auto-lang-core/Cargo.toml` + metadata 图（plan-878 已留全量 JSON） | 每依赖一行 必需/可裁/未知；未知项进 U-01 探针 | 半天 |
| P-0b | Instant/AtomicU64 wasm 事实核对：查 rustc 1.98 std 对 wasm32-unknown-unknown 的 Instant/64 位原子支持文档与源（`library/std/src/time.rs` 等），落成事实注记 | 本机 rustc 源/文档 | 每项：可用/不可用/条件可用 + 出处 | 1 小时 |

### P-1 render-only 双端（XP-1 首步，建议正式 Plan 范围）

- **输入来源**：当前冻结 scene（AGP `core/scene` + `assets/glb.rs` 支持的静态 GLB
  子集）+ 一个最小 .agscene/.at 驱动输入；**不引入游戏 Rust 分支、不复制第二套实现**
  （设计 §6 XP-1 行）。
- **候选资产**：877 G01/G02 几何（若届时已交付）或 AGP `testdata`/`assets` 内已有
  静态模型；资源规模沿用小场景（128 节点/32 credits/250ms 预算不重命名为移动保证）。
- **正 oracle**：两端最终内容的独立几何/颜色区域/深度/命中 oracle（不要求全图 hash
  一致）；资源身份一致（GLB 内容哈希）。
- **负 oracle**：少画一个 mesh/不更新 viewport 必须被发现（设计 §6 判别行）。
- **出口分类**：仅原生成功=render-only 不认领 Auto 执行；两端成功才可进 P-2。
- **触面 owner**：wgpu 实例/surface（`renderer/target.rs`）、资源上传（`assets/glb.rs`）、
  readback（`renderer/capture.rs`）——即 adapter-needed 16 文件族。
- **资源规模建议**：单窗、单一后端优先（WebGPU，WebGL 降级不默认承诺）；构建包体/
  启动/CPU/内存可测范围逐项记录。

### P-2..P-4 接口（XP-2/3/4 只列研究接口，不创建）

- **XP-2（Vue/Canvas 宿主）**：bridge 接口=单一逻辑状态 + 布局/焦点/输入桥；
  负控=DOM 文字焦点时游戏不得响应同键、漏 DPI、重复事件。前置=P-1 两端成功。
- **XP-3（3D 挂 UI）**：两候选（DOM 投影 overlay vs 可采样 UI adapter——后者已有
  iced 离屏接缝雏形）；判别=前景几何遮挡在两候选下的不同结果；逐项测遮挡/clip/
  命中/IME/可访问性。
- **XP-4（移动两路线）**：输入=touch/多指 cancel/软键盘/旋转/safe area；生命周期=
  前后台/内存压力/surface 或 device 重建（platform-seams §1.5 缺口表）；低能力
  拒绝/降级显式分类。前置=P-1 成熟；本机工具链（NDK/Xcode）需先补装（U-08）。

## 3. 阻塞与限制清单（readiness=limited 的依据）

| ID | 阻塞 | 影响 | 绕行/消解 |
|---|---|---|---|
| R-01 | U-01 SDK 宿主栈跨端可编译性未测 | Auto 执行腿（VM 驱动语义）在 wasm/移动不可承诺 | P-0a/P-0b 先缩小面；XP-1 首步限 render-only |
| R-02 | 本机无 android/ios 工具链（合同禁下载） | 移动 native 编译级事实全缺 | XP-4 立项前补装；XP-1/2 不受影响 |
| R-03 | 无浏览器运行证据（合同禁浏览器） | WebGPU 路线=依赖边存在，仅此 | XP-1 P-1 探针本体 |
| R-04 | 877 Gallery 未交付 | G01/G02 资产候选可能缺 | 用当前冻结 scene/模型等价替代（已内建） |

XP 后继**不被本 Plan 自动启动**：XP-1 立项需各自 needs-analysis/合同；本文件仅是
研究接口与事实起点。
