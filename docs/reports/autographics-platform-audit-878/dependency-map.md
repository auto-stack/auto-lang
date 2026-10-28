# 2. 依赖图与源码分类（dependency-map）

XP-0 依赖审计。判定词汇与 root 集合见审计合同；输入身份见 [inputs.md](inputs.md)。
原始 metadata JSON（本地）：`D:/autostack/plan-artifacts/plan-878/meta-{agp,driver}-*.json`。

## 1. metadata 解析结果总览

10/10 解析 **complete**（AGP/driver × 5 target，全部 exit 0，无 stderr）。
分级依据合同 §3：cfg 级可解析 ≠ 目标可编译/可运行。

| workspace | lock 条目 | host-win 可达 | wasm 可达 | linux 可达 | android 可达 | ios 可达 |
|---|---|---|---|---|---|---|
| agp（prototype） | 478 | 420 | 192 | 286 | 224 | 243 |
| autographics-auto-driver | 787 | 700 | 449 | 563 | 506 | 523 |

（可达数=resolve nodes；差值即该 target 下被 cfg 剔除的依赖边。）

两 manifest 均无 `[features]`、`[dev-dependencies]`、`[build-dependencies]`；
测试与产品共用同一依赖集。AGP `build.rs` 仅用 std（构建期调用 `git rev-parse`
注入提交指纹——构建机需 git，与目标平台无关）。

## 2. 直接依赖（一级，manifest 声明）

### AGP（agp 0.1.0，14 项）

| 包 | 锁定版本 | 用途 | 显式 feature | 平台分类 | 证据/备注 |
|---|---|---|---|---|---|
| wgpu | 27.0.1 | GPU 渲染 | 默认 | adapter-needed | 与 iced_wgpu 0.14 共享 27.x 实例（T-08 接缝）；wasm 经 web-sys/WebGPU 后端保留在图 |
| winit | 0.30.13 | 窗口/输入 | 默认 | host-only | wasm 上保留（web 后端），android 经 android-activity/ndk 保留 |
| glam | 0.34 | 数学 | 默认 | shared-candidate | f32-first 纯数学 |
| image | 0.25 | 关键帧捕获 | `default-features=false, ["png"]` | adapter-needed | 仅 png 解码；无 zune 压缩面 |
| serde / serde_json | 1 / 1 | 序列化 | derive | shared-candidate | 探针报告与数据交换 |
| sha2 | 0.10 | 内容哈希 | 默认 | shared-candidate | 纯计算 |
| fontdue | 0.9 | HUD 字形栅格化 | 默认 | shared-candidate | 纯 CPU 栅格（glyphon 无 wgpu-27 线的替代选择） |
| iced | 0.14.0 | UI | `["tokio","advanced","unconditional-rendering"]` | host-only（接缝已隔离） | 经 iced_wgpu 共享 wgpu；tokio feature 在 wasm 图中被剔除 |
| iced_wgpu / iced_widget / iced_runtime | 0.14.0/0.14.2/0.14.0 | UI 渲染/控件/运行时 | 默认 | host-only | 库版 crates.io 未打补丁（AGP 侧）；widget 注册表成本见 AGENTS tu 档注记 |
| kira | 0.9 | 音频 | `["ogg"]` | host-only | symphonia/ogg 解码纯软件；后端输出平台相关（alsa/coreaudio/oboe/wasapi） |
| gltf | 1.4 | GLB 模型加载 | 默认 | shared-candidate（解析）/ adapter-needed（上传） | 静态 GLB 支持集不扩张（合同） |

### driver（autographics-auto-driver 0.1.0，11 项）

| 包 | 锁定版本 | 用途 | 显式 feature | 平台分类 | 证据/备注 |
|---|---|---|---|---|---|
| auto-lang-core | path `.sdk-source/crates/auto-lang-core` | 语言 SDK VM/运行时 | — | **adapter-needed（重）** | 见 §4：无条件携带 axum/tokio/reqwest/notify/libloading/stacker/sysinfo 到全部 5 target |
| auto-val | path `.sdk-source/crates/auto-val` | 值/路径原语 | — | shared-candidate | 直接依赖仅 7 项且全轻量（§4） |
| agp | path `../autographics-prototype` | 渲染/场景宿主 | — | （=AGP 分类） | 兄弟 workspace path 依赖 |
| glam 0.34 / serde / serde_json | 同上 | 同上 | — | shared-candidate | |
| gltf | 1.4 | GLB | `["extras"]` | shared-candidate（解析） | extras=JSON 扩展读取 |
| winit 0.30.13 / iced 0.14.0 / iced_wgpu 0.14.0 | 同上 | 窗口/UI | 默认 | host-only | driver 侧 iced 未加 tokio/advanced feature |
| kira | 0.9 | 音频 | `["ogg"]` | host-only | |

## 3. target 条件边（metadata 差异图）

平台适配主体在**传递依赖的 cfg 边**内，第一方 manifest 无 target 表达：

- **桌面 OS 后端族**（wayland/x11/winapi/objc2/metal/alsa/coreaudio/zbus/…）：
  host 上 228（AGP）/251（driver）个包仅在桌面可达。
- **wasm32-unknown-unknown**：`web-sys`/`wasm-bindgen` 保留在图（wgpu WebGL/WebGPU、
  winit web 后端路线存在）；`tokio`、`rustls` 从 AGP/driver 图剔除（iced tokio
  feature 与 reqwest tls 边被 cfg 收缩）。AGP 图 420→192。
- **android**：`android-activity`/`ndk`/`ndk-sys`/`jni`/`oboe`（音频）保留；
  `web-sys` 剔除。AGP 图 420→224。
- **ios**：`objc2-*`（core-text/video 等）保留；`openssl` 剔除（reqwest 默认
  native-tls 边收缩，`rustls` 保留）。AGP 图 420→243。
- **异常边（如实记录）**：`wasm-bindgen` 在 android/ios 图中仍可达（非 wasm 专用
  cfg 收缩不彻底的传递边，源自 web 栈传递依赖）；driver 图中 `openssl` 在
  win/linux/android 可达、ios 剔除——`reqwest` 默认 TLS 面按 target 变化，
  是 SDK HTTP 栈跨端不确定性的直接证据。

完整逐包 presence 矩阵：`plan-878/analysis-{agp,driver}.json`（本地）。

## 4. SDK（.sdk-source）闭包

- 17 本地包中 **15 个在 driver resolve 图内且全部可达**：
  `a2r-std, auto-ast, auto-atom, auto-bindgen, auto-cache, auto-diagnostics,
  auto-lang-core, auto-lex, auto-macros, auto-media-core, auto-parser-api,
  auto-terminal-core, auto-text-core, auto-val, shim-metadata`。
- **不在闭包**：`auto-test-case`、`vendor/autodown-test-parser`（测试专用包，
  产品 driver 不引用；仅 SDK 内容锁内在场）。
- `stdlib/` 非 cargo 包（.at 源文件集合，随 SDK 分发），无依赖边。
- **auto-lang-core 直接依赖 67 项**，宿主服务栈（重）：`axum, tokio, reqwest,
  hyper, tungstenite(tokio-tungstenite), notify, libloading, stacker, sysinfo,
  dirs, rustyline, csv, tar, flate2, blake3, chrono, …`。
  **关键观察：这批边在全部 5 个 target（含 wasm32-unknown-unknown）的 resolve
  图中都存在**——SDK manifest 未做任何 target 门控。即静态图上 wasm 也"携带"
  axum/libloading/stacker/sysinfo；这些 crate 在 wasm/移动 target 是否可编译
  为未测假设（多数默认不可），XP-1 必须首先探明。
- **auto-val 直接依赖 7 项**：`ecow, glob-match, indexmap, normalize-path,
  regex, serde, thiserror`——纯逻辑/纯数据形状，shared-candidate。
- driver 运行时语义依赖 SDK VM 形态：`create_vm_from_source` +
  `AutoVM/ParkedSegment/CpuSliceBudget/SegmentOutcome/AutoTask`
  （`src/lib.rs:1-10`，driver 以 CPU 片段预算+停靠段调度 .at 行为脚本）。

## 5. 第一方源码分类（222 文件全闭合）

分类词汇/三对照见合同 §4。方法：9 类宿主服务标记扫描（winit/wgpu/iced/kira/
std::fs/path/thread/process/sync/time/net/env/windows-api/libloading/spawn/
sleep/atomic/cfg/unsafe）+ 全部共享候选逐文件人工复核 + 触面抽查读源。
计数：**shared-candidate 22、vm-harness 1、adapter-needed 16、host-only 73、
frozen-oracle 2、build-tool 1、structure 1、evidence-tool 106**（tests+research 探针）。

要点（代表行，全表见文末附录 A）：

| 触面 | 文件/符号 | 分类 | 关键证据 |
|---|---|---|---|
| 场景/身份语义 | `prototype/src/core/{id,scene,events,composition/**}` | shared-candidate | `core/mod.rs:5` 与 `core/composition/mod.rs:21` 明文禁令"不得引用 wgpu/winit/Iced 类型"（PLAN-829 §2）；composition 的 `ContentKind::Iced` 仅为枚举判别值（`Copy`，无 iced 类型） |
| 身份分配 | `core/id.rs:7,16` `static NEXT: AtomicU64` | shared-candidate×（**wasm 风险单列**） | 全局代次分配器用 AtomicU64；wasm32 无 atomics feature 时 64 位原子可用性未测 |
| 时间语义 | `core/{time,runtime,demo}.rs` | shared-candidate | 仅 `std::time::Duration`（数据类型）；core 无 `Instant::now()` 读取墙钟 |
| VM 调度 | `driver/src/lib.rs` | vm-harness | `Instant::now()` ×6 处 CPU 片段预算计时；AtomicU64 会话计数（`script.rs:12`）——wasm 上 Instant/AtomicU64 可用性未测（hypothesis） |
| 游戏逻辑 | `driver/src/squash.rs`、`prototype/src/games/**`（除 host 接线文件） | shared-candidate / host-only 混合 | squash.rs 纯 glam+atomic+Instant；games/squash/live.rs 含 winit 键盘接线 |
| 2D/3D 渲染 | `prototype/src/{render2d,render3d,renderer}/**` | adapter-needed | wgpu 耦合（16 文件）；`renderer/target.rs:248` `surface.get_current_texture()`（表面呈现假设）、`renderer/capture.rs:61` buffer `map_async` readback（回调被忽略的轮询模式）；wgpu 本身有 wasm/移动后端，但资源上传/readback/表面生命周期是未测假设 |
| 资产 | `prototype/src/assets/glb.rs` | adapter-needed | gltf 解析（纯）+ wgpu 纹理上传 + `std::fs` 读取（fs 需宿主注入或 fetch 适配） |
| 桌面宿主 | `prototype/src/host/**`（iced_spike/iced_surface/surface_runtime/composition_input/gates/cases）、`driver/src/{host,squash_host,consumer}.rs`、`bin/**`、`authoring/**` | host-only | winit×15、iced×15、kira×3、线程/进程/fs/env；`composition_input.rs:480-586` 与 `gpu_foundation.rs:766-818` 为仅有的产品 cfg(windows) 对（测量/IME 路径，均有非 windows 分支或属探针） |
| 作者存储 | `driver/src/authoring/storage.rs:594-616` | host-only | windows 直链 kernel32 `MoveFileExW`（unsafe extern）；**非 windows 分支是硬错误桩**（"asset persistence profile is Windows"）——移动端作者存储无路线声明 |
| 传输协议 | `driver/src/output.rs` | shared-candidate | 纯数据 Packet/Object（u64 session/sequence、f32 值、[f32;4] 背景）——正是 u64/JS Number、f32 桥接要求的承载面 |
| 冻结样本 | `prototype/testdata/breakout/*.rs` | frozen-oracle | Bevy 冻结参照实现，仅对拍用 |

## 6. 三对照核验（合同 §4 强制项）

1. **仅测量 native 不判死 core**：`host/cases/gpu_foundation.rs` 的
   `#[cfg(target_os="windows")]` 全部位于 GPU 探针/测量路径（host-only 目录），
   core/ 语义模块 0 个平台 cfg（仅 `#[cfg(test)]`）——core 的 shared-candidate
   判定不因此受损。✔
2. **无关键词 ≠ shared**：driver 闭包内 SDK 栈（axum/notify/libloading/stacker/
   sysinfo）源码无平台关键词，但经 metadata 证明其依赖边在全部 target 无门控
   地存在——driver 的运行时闭包**不能**因此标 shared（§4 关键观察）。✔
3. **纯数据不过度判 native**：`output.rs`、`core/composition/**`、`auto-val`
   均为隔离纯数据/纯逻辑路径（输入输出类型可序列化、无宿主调用），判
   shared-candidate 并附未测桥接假设（u64 代次、f32、Duration）。✔

## 7. 未解析边（unresolved 清单）

| # | 边 | 为何 unresolved | 后继 |
|---|---|---|---|
| U-01 | auto-lang-core 宿主栈（axum/tokio/reqwest/notify/libloading/stacker/sysinfo/rustyline）在 wasm/android/ios 的可编译性 | manifest 无门控；未编译（合同禁止） | XP-1 编译探针 |
| U-02 | `std::time::Instant` 与 AtomicU64 在 wasm32-unknown-unknown 的可用性/精度 | std 行为随 rustc 版本变化；未编译 | XP-1 最小 .rs 探针 |
| U-03 | wgpu 27 wasm 后端（WebGPU/WebGL）对本管线（readback/纹理格式/render target）兼容性 | 有依赖边（web-sys 在图），无运行证据 | XP-2 渲染探针 |
| U-04 | winit 0.30 android-activity 生命周期与 AGP host 假设（surface 丢失/暂停恢复） | 依赖边在图，运行未测 | XP-3 |
| U-05 | kira 0.9 移动音频（oboe/AVAudioSession）与用户激活约束 | 后端在图，运行未测 | XP-3 |
| U-06 | driver 测试面（spawn 子进程/env/网络端口）在非桌面 target 的整体不可用范围 | 属 evidence-tool，未逐文件展开（不属产品闭包） | 不阻塞 XP-1 |
| U-07 | 第三方 registry 源码未逐包读缓存核销 | 合同允许按图分类；关键边（wgpu/winit/kira/reqwest）已知平台结构引用上游文档，未实测 | 按探针逐个消解 |
| U-08 | 本机无 aarch64-linux-android / aarch64-apple-ios 工具链（合同禁下载） | 移动 native 的编译级事实全缺；`--filter-platform` 解析仅证 cfg 可解 | XP-4 立项前补装工具链后定向复验 |

## 附录 A：第一方源码逐文件分类表（222 行）

| 文件 | 行数 | 分类 | 观察证据（标记） |
|---|---|---|---|
| `autographics-auto-driver/src/output.rs` | 226 | shared-candidate | —（无宿主标记） |
| `autographics-auto-driver/src/script.rs` | 378 | shared-candidate | atomic, std-time |
| `autographics-auto-driver/src/squash.rs` | 640 | shared-candidate | atomic, glam, std-time |
| `autographics-prototype/src/core/composition/input.rs` | 465 | shared-candidate | —（无宿主标记） |
| `autographics-prototype/src/core/composition/mod.rs` | 1060 | shared-candidate | iced, wgpu, winit |
| `autographics-prototype/src/core/demo.rs` | 206 | shared-candidate | cfg, glam, std-time |
| `autographics-prototype/src/core/events.rs` | 109 | shared-candidate | cfg |
| `autographics-prototype/src/core/id.rs` | 199 | shared-candidate | atomic, cfg, std-sync |
| `autographics-prototype/src/core/mod.rs` | 21 | shared-candidate | wgpu, winit |
| `autographics-prototype/src/core/rng.rs` | 74 | shared-candidate | cfg |
| `autographics-prototype/src/core/runtime.rs` | 235 | shared-candidate | std-time |
| `autographics-prototype/src/core/scene.rs` | 372 | shared-candidate | cfg, glam |
| `autographics-prototype/src/core/time.rs` | 116 | shared-candidate | cfg, std-time |
| `autographics-prototype/src/games/breakout/mod.rs` | 5 | shared-candidate | —（无宿主标记） |
| `autographics-prototype/src/games/breakout/sim.rs` | 425 | shared-candidate | glam |
| `autographics-prototype/src/games/mod.rs` | 4 | shared-candidate | —（无宿主标记） |
| `autographics-prototype/src/games/squash/mod.rs` | 8 | shared-candidate | —（无宿主标记） |
| `autographics-prototype/src/games/squash/sim.rs` | 628 | shared-candidate | glam |
| `autographics-prototype/src/render3d/anim.rs` | 100 | shared-candidate | cfg |
| `autographics-prototype/src/render3d/collision.rs` | 172 | shared-candidate | glam |
| `autographics-prototype/src/render3d/procedural.rs` | 135 | shared-candidate | cfg |
| `autographics-prototype/src/renderer/witness.rs` | 92 | shared-candidate | atomic, cfg, std-sync |
| `autographics-auto-driver/src/lib.rs` | 452 | vm-harness | cfg, std-path, std-sync, std-time, vm-sdk |
| `autographics-prototype/src/assets/glb.rs` | 179 | adapter-needed | glam, gltf, std-fs, std-path |
| `autographics-prototype/src/assets/mod.rs` | 7 | adapter-needed | gltf, wgpu |
| `autographics-prototype/src/render2d/mod.rs` | 6 | adapter-needed | fontdue, wgpu |
| `autographics-prototype/src/render2d/text.rs` | 499 | adapter-needed | fontdue, unsafe, wgpu |
| `autographics-prototype/src/render3d/camera.rs` | 174 | adapter-needed | cfg, glam, wgpu |
| `autographics-prototype/src/render3d/editor_grid.rs` | 243 | adapter-needed | glam, std-sync, wgpu |
| `autographics-prototype/src/render3d/mod.rs` | 20 | adapter-needed | wgpu |
| `autographics-prototype/src/render3d/pipeline.rs` | 928 | adapter-needed | glam, iced, std-sync, unsafe, wgpu |
| `autographics-prototype/src/renderer/capture.rs` | 194 | adapter-needed | image-crate, wgpu |
| `autographics-prototype/src/renderer/composition/mod.rs` | 666 | adapter-needed | glam, wgpu |
| `autographics-prototype/src/renderer/device.rs` | 153 | adapter-needed | atomic, serde, std-sync, wgpu |
| `autographics-prototype/src/renderer/error.rs` | 96 | adapter-needed | std-path, wgpu |
| `autographics-prototype/src/renderer/mod.rs` | 80 | adapter-needed | atomic, serde, std-sync, wgpu |
| `autographics-prototype/src/renderer/pipeline.rs` | 458 | adapter-needed | glam, std-sync, unsafe, wgpu |
| `autographics-prototype/src/renderer/resources.rs` | 248 | adapter-needed | cfg, serde, wgpu |
| `autographics-prototype/src/renderer/target.rs` | 269 | adapter-needed | atomic, std-sync, wgpu, winit |
| `autographics-auto-driver/src/authoring/assets.rs` | 778 | host-only | atomic, cfg, glam, gltf, serde, std-env, std-fs, std-path, std-sync, std-time, windows-api |
| `autographics-auto-driver/src/authoring/behavior.rs` | 1043 | host-only | serde, std-env, std-fs, std-path, vm-sdk |
| `autographics-auto-driver/src/authoring/behavior/assembly.rs` | 478 | host-only | cfg, vm-sdk |
| `autographics-auto-driver/src/authoring/behavior_project.rs` | 213 | host-only | serde, std-path, std-sync |
| `autographics-auto-driver/src/authoring/document.rs` | 449 | host-only | serde |
| `autographics-auto-driver/src/authoring/game_input.rs` | 113 | host-only | cfg |
| `autographics-auto-driver/src/authoring/hierarchy.rs` | 170 | host-only | —（无宿主标记） |
| `autographics-auto-driver/src/authoring/host.rs` | 541 | host-only | glam, std-path, std-sync |
| `autographics-auto-driver/src/authoring/host/commands.rs` | 510 | host-only | std-path |
| `autographics-auto-driver/src/authoring/host/navigation.rs` | 213 | host-only | glam |
| `autographics-auto-driver/src/authoring/host/navigation3d.rs` | 262 | host-only | glam |
| `autographics-auto-driver/src/authoring/host/panel.rs` | 501 | host-only | iced |
| `autographics-auto-driver/src/authoring/host/play_adapter.rs` | 404 | host-only | std-path, std-time |
| `autographics-auto-driver/src/authoring/host/viewport.rs` | 1074 | host-only | glam, iced, std-time, winit |
| `autographics-auto-driver/src/authoring/interaction.rs` | 398 | host-only | glam, std-time |
| `autographics-auto-driver/src/authoring/measurement.rs` | 30 | host-only | serde, std-time |
| `autographics-auto-driver/src/authoring/mod.rs` | 19 | host-only | —（无宿主标记） |
| `autographics-auto-driver/src/authoring/play.rs` | 1189 | host-only | atomic, cfg, serde, std-path, std-sync, std-time |
| `autographics-auto-driver/src/authoring/preview.rs` | 282 | host-only | std-time |
| `autographics-auto-driver/src/authoring/session.rs` | 508 | host-only | atomic, std-sync |
| `autographics-auto-driver/src/authoring/spatial.rs` | 106 | host-only | atomic, glam, serde |
| `autographics-auto-driver/src/authoring/storage.rs` | 766 | host-only | atomic, cfg, std-env, std-fs, std-path, std-process, unsafe, windows-api |
| `autographics-auto-driver/src/authoring/tagged.rs` | 155 | host-only | std-sync |
| `autographics-auto-driver/src/authoring/view2d.rs` | 163 | host-only | glam |
| `autographics-auto-driver/src/authoring/view3d.rs` | 329 | host-only | glam |
| `autographics-auto-driver/src/bin/authoring/cli.rs` | 459 | host-only | std-env, std-fs, std-path, std-process |
| `autographics-auto-driver/src/bin/authoring/probes.rs` | 289 | host-only | std-env, winit |
| `autographics-auto-driver/src/bin/authoring_probe/assets.rs` | 349 | host-only | glam, winit |
| `autographics-auto-driver/src/bin/authoring_probe/character2d.rs` | 230 | host-only | winit |
| `autographics-auto-driver/src/bin/authoring_probe/hierarchy.rs` | 328 | host-only | winit |
| `autographics-auto-driver/src/bin/authoring_probe/mod.rs` | 555 | host-only | cfg, glam, std-process, unsafe, winit |
| `autographics-auto-driver/src/bin/authoring_probe/motion.rs` | 203 | host-only | —（无宿主标记） |
| `autographics-auto-driver/src/bin/authoring_probe/play.rs` | 882 | host-only | cfg, glam, std-env, std-path, winit |
| `autographics-auto-driver/src/bin/authoring_probe/segment.rs` | 138 | host-only | —（无宿主标记） |
| `autographics-auto-driver/src/bin/authoring_probe/spatial.rs` | 561 | host-only | glam, std-env, winit |
| `autographics-auto-driver/src/bin/authoring_probe/sweep.rs` | 155 | host-only | —（无宿主标记） |
| `autographics-auto-driver/src/bin/authoring_probe/viewport2d.rs` | 248 | host-only | glam, winit |
| `autographics-auto-driver/src/bin/authoring_probe/viewport3d.rs` | 527 | host-only | glam, std-env, std-path, std-process, winit |
| `autographics-auto-driver/src/bin/autographics-authoring.rs` | 740 | host-only | cfg, std-env, std-fs, std-path, std-process, std-sync, std-time, unsafe, windows-api, winit |
| `autographics-auto-driver/src/bin/autographics-game-probe.rs` | 161 | host-only | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/src/bin/autographics-m2-baseline.rs` | 239 | host-only | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/src/consumer.rs` | 113 | host-only | glam |
| `autographics-auto-driver/src/host.rs` | 550 | host-only | glam, kira, std-time, winit |
| `autographics-auto-driver/src/squash_host.rs` | 487 | host-only | cfg, glam, kira, std-sync, winit |
| `autographics-prototype/src/bin/graphics-probe.rs` | 73 | host-only | std-env, std-path, std-process |
| `autographics-prototype/src/bin/iced_spike.rs` | 7 | host-only | iced, wgpu |
| `autographics-prototype/src/games/squash/live.rs` | 274 | host-only | cfg, winit |
| `autographics-prototype/src/host/cases/breakout.rs` | 1225 | host-only | kira, serde, sleep, std-fs, std-path, std-sync, std-thread, std-time, winit |
| `autographics-prototype/src/host/cases/consumer.rs` | 569 | host-only | iced, std-env, std-fs, std-path, std-process, thread-spawn, wgpu, winit |
| `autographics-prototype/src/host/cases/frame_oracle.rs` | 100 | host-only | image-crate, serde, std-fs, std-path |
| `autographics-prototype/src/host/cases/gpu_foundation.rs` | 989 | host-only | cfg, std-fs, std-path, std-sync, std-time, winit |
| `autographics-prototype/src/host/cases/iced_shared_device.rs` | 484 | host-only | atomic, iced, std-fs, std-path, std-sync, wgpu |
| `autographics-prototype/src/host/cases/lifecycle.rs` | 693 | host-only | kira, std-fs, std-path, std-process, std-sync, std-time, wgpu, winit |
| `autographics-prototype/src/host/cases/metrics.rs` | 445 | host-only | serde, std-env, std-fs, std-path, std-process, std-sync, std-time, wgpu |
| `autographics-prototype/src/host/cases/mod.rs` | 69 | host-only | iced, std-path |
| `autographics-prototype/src/host/cases/negative.rs` | 390 | host-only | glam, std-env, std-fs, std-path, std-time |
| `autographics-prototype/src/host/cases/risk_slice.rs` | 694 | host-only | glam, gltf, std-fs, std-path, std-sync, winit |
| `autographics-prototype/src/host/cases/shared_runtime.rs` | 474 | host-only | std-path, std-sync, winit |
| `autographics-prototype/src/host/cases/squash_game.rs` | 1364 | host-only | glam, kira, serde, sleep, std-fs, std-path, std-sync, std-thread, std-time, winit |
| `autographics-prototype/src/host/cases/surface_compose.rs` | 1175 | host-only | glam, iced, serde, std-fs, std-path |
| `autographics-prototype/src/host/cases/surface_ime_os.rs` | 1175 | host-only | glam, iced, serde, std-fs, std-path, std-process, std-sync, std-time, winit |
| `autographics-prototype/src/host/cases/surface_input.rs` | 1315 | host-only | glam, iced, serde, std-env, std-path, std-sync, std-time, winit |
| `autographics-prototype/src/host/cases/surface_lifecycle.rs` | 1182 | host-only | iced, serde, std-path, std-time, wgpu |
| `autographics-prototype/src/host/cases/surface_offscreen.rs` | 601 | host-only | iced, image-crate, std-env, std-path, std-sync, std-time, winit |
| `autographics-prototype/src/host/cases/surface_scenarios.rs` | 1201 | host-only | glam, iced, image-crate, serde, std-fs, std-path, std-time |
| `autographics-prototype/src/host/composition_input.rs` | 597 | host-only | cfg, glam, iced, std-process |
| `autographics-prototype/src/host/gates.rs` | 114 | host-only | —（无宿主标记） |
| `autographics-prototype/src/host/iced_spike.rs` | 1756 | host-only | atomic, cfg, glam, iced, image-crate, sleep, std-env, std-fs, std-path, std-process, std-sync, std-thread, std-time, wgpu, winit |
| `autographics-prototype/src/host/iced_surface.rs` | 603 | host-only | iced, std-time, wgpu, winit |
| `autographics-prototype/src/host/iced_surface_probe.rs` | 1073 | host-only | atomic, glam, iced, sleep, std-env, std-fs, std-path, std-process, std-sync, std-thread, std-time, wgpu, winit |
| `autographics-prototype/src/host/mod.rs` | 15 | host-only | winit |
| `autographics-prototype/src/host/report.rs` | 634 | host-only | iced, serde, std-env, std-fs, std-path, std-time, wgpu, winit |
| `autographics-prototype/src/host/surface_runtime.rs` | 1392 | host-only | glam, iced, wgpu |
| `autographics-prototype/testdata/breakout/breakout.rs` | 422 | frozen-oracle | thread-spawn |
| `autographics-prototype/testdata/breakout/stepping.rs` | 276 | frozen-oracle | cfg, thread-spawn |
| `autographics-prototype/build.rs` | 36 | build-tool | std-path, std-process |
| `autographics-prototype/src/lib.rs` | 19 | structure | wgpu, winit |
| `autographics-auto-driver/src/main.rs` | 536 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-thread, std-time, winit |
| `autographics-auto-driver/src/research/game_runtime.rs` | 728 | evidence-tool | serde, std-path, std-time |
| `autographics-auto-driver/src/research/mod.rs` | 2 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/asset_feasibility.rs` | 243 | evidence-tool | glam, iced, std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/authoring_assets.rs` | 1598 | evidence-tool | atomic, cfg, glam, std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/authoring_behavior.rs` | 344 | evidence-tool | std-fs, std-path, std-time |
| `autographics-auto-driver/tests/authoring_behavior_host.rs` | 315 | evidence-tool | glam, std-env, std-fs, std-path, std-sync, std-time |
| `autographics-auto-driver/tests/authoring_document.rs` | 37 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/authoring_feasibility.rs` | 148 | evidence-tool | glam, iced, std-env, std-fs, std-process |
| `autographics-auto-driver/tests/authoring_hierarchy.rs` | 676 | evidence-tool | glam, std-fs, std-path, std-process, std-sync |
| `autographics-auto-driver/tests/authoring_host.rs` | 371 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync |
| `autographics-auto-driver/tests/authoring_interaction.rs` | 402 | evidence-tool | glam, std-env, std-fs, std-process, std-sync |
| `autographics-auto-driver/tests/authoring_play.rs` | 361 | evidence-tool | std-time |
| `autographics-auto-driver/tests/authoring_play_host.rs` | 562 | evidence-tool | sleep, std-env, std-fs, std-process, std-thread, std-time |
| `autographics-auto-driver/tests/authoring_probe_exit.rs` | 184 | evidence-tool | std-fs, std-path, std-process, std-sync |
| `autographics-auto-driver/tests/authoring_session.rs` | 155 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/authoring_storage.rs` | 116 | evidence-tool | cfg, std-env, std-fs, std-path, std-process, windows-api |
| `autographics-auto-driver/tests/authoring_viewport_feasibility.rs` | 135 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/behavior_project_document.rs` | 120 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/behavior_project_host.rs` | 480 | evidence-tool | glam, std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/behavior_project_session.rs` | 165 | evidence-tool | std-path |
| `autographics-auto-driver/tests/behavior_project_storage.rs` | 287 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/behavior_source_identity.rs` | 52 | evidence-tool | std-path |
| `autographics-auto-driver/tests/breakout_replay.rs` | 122 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/character2d_api.rs` | 426 | evidence-tool | std-path |
| `autographics-auto-driver/tests/character2d_host.rs` | 724 | evidence-tool | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/character2d_samples.rs` | 232 | evidence-tool | std-env, std-fs, std-path |
| `autographics-auto-driver/tests/editor_viewport2d.rs` | 537 | evidence-tool | glam, std-env, std-fs, std-process |
| `autographics-auto-driver/tests/editor_viewport3d.rs` | 863 | evidence-tool | glam, std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/game_input.rs` | 262 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/game_runtime_feasibility.rs` | 666 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/graphics_loop.rs` | 505 | evidence-tool | std-fs, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/hierarchy_feasibility.rs` | 159 | evidence-tool | iced |
| `autographics-auto-driver/tests/m2_baseline.rs` | 68 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/motion_api.rs` | 77 | evidence-tool | std-path |
| `autographics-auto-driver/tests/motion_cli.rs` | 104 | evidence-tool | std-path, std-process |
| `autographics-auto-driver/tests/motion_feasibility.rs` | 125 | evidence-tool | std-time |
| `autographics-auto-driver/tests/motion_geometry.rs` | 217 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/motion_host.rs` | 278 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/motion_owners.rs` | 282 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/motion_project.rs` | 81 | evidence-tool | std-env, std-fs, std-path, std-process |
| `autographics-auto-driver/tests/motion_scan.rs` | 122 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/numeric_probe.rs` | 19 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/output_admission.rs` | 76 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/repair857_buttons.rs` | 50 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/repair857_clip.rs` | 141 | evidence-tool | glam |
| `autographics-auto-driver/tests/repair857_visible.rs` | 156 | evidence-tool | glam |
| `autographics-auto-driver/tests/scene_play_feasibility.rs` | 130 | evidence-tool | glam, std-time |
| `autographics-auto-driver/tests/scheduler_lifecycle.rs` | 61 | evidence-tool | std-time |
| `autographics-auto-driver/tests/sdk_probe.rs` | 69 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/segment_api.rs` | 202 | evidence-tool | std-path |
| `autographics-auto-driver/tests/segment_budget.rs` | 300 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/segment_cli.rs` | 84 | evidence-tool | std-path, std-process |
| `autographics-auto-driver/tests/segment_counterexamples.rs` | 92 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/segment_derive.rs` | 59 | evidence-tool | std-path |
| `autographics-auto-driver/tests/segment_feasibility.rs` | 87 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/segment_geometry.rs` | 143 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/segment_host.rs` | 185 | evidence-tool | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/segment_owners.rs` | 249 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/segment_project.rs` | 81 | evidence-tool | std-env, std-fs, std-path, std-process |
| `autographics-auto-driver/tests/spatial_api.rs` | 208 | evidence-tool | std-fs, std-path |
| `autographics-auto-driver/tests/spatial_cli.rs` | 83 | evidence-tool | std-path, std-process |
| `autographics-auto-driver/tests/spatial_feasibility.rs` | 358 | evidence-tool | glam, std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/spatial_geometry.rs` | 118 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/spatial_host.rs` | 167 | evidence-tool | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/spatial_owners.rs` | 237 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/spatial_project.rs` | 81 | evidence-tool | std-env, std-fs, std-path, std-process |
| `autographics-auto-driver/tests/spatial_samples.rs` | 62 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/squash_admission.rs` | 154 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/squash_clock_input.rs` | 178 | evidence-tool | std-sync |
| `autographics-auto-driver/tests/squash_feasibility.rs` | 138 | evidence-tool | glam, std-time |
| `autographics-auto-driver/tests/squash_graphics.rs` | 362 | evidence-tool | glam, sleep, std-sync, std-thread, std-time |
| `autographics-auto-driver/tests/squash_numeric_replay.rs` | 131 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/squash_profile.rs` | 76 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/squash_stability.rs` | 83 | evidence-tool | std-fs, std-process, std-time |
| `autographics-auto-driver/tests/squash_supplements.rs` | 165 | evidence-tool | glam |
| `autographics-auto-driver/tests/squash_visual_oracle.rs` | 239 | evidence-tool | glam, std-fs, std-time |
| `autographics-auto-driver/tests/support/motion.rs` | 79 | evidence-tool | std-fs, std-path |
| `autographics-auto-driver/tests/support/segment.rs` | 84 | evidence-tool | std-fs, std-path |
| `autographics-auto-driver/tests/support/sweep.rs` | 85 | evidence-tool | std-fs, std-path |
| `autographics-auto-driver/tests/sweep_api.rs` | 64 | evidence-tool | std-path |
| `autographics-auto-driver/tests/sweep_cli.rs` | 103 | evidence-tool | std-path, std-process |
| `autographics-auto-driver/tests/sweep_derive.rs` | 59 | evidence-tool | std-path |
| `autographics-auto-driver/tests/sweep_feasibility.rs` | 121 | evidence-tool | std-time |
| `autographics-auto-driver/tests/sweep_geometry.rs` | 322 | evidence-tool | —（无宿主标记） |
| `autographics-auto-driver/tests/sweep_host.rs` | 205 | evidence-tool | std-env, std-fs, std-path, std-process, std-time |
| `autographics-auto-driver/tests/sweep_owners.rs` | 274 | evidence-tool | std-env, std-fs, std-path, std-process, std-sync, std-time |
| `autographics-auto-driver/tests/sweep_project.rs` | 81 | evidence-tool | std-env, std-fs, std-path, std-process |
| `autographics-auto-driver/tests/sweep_samples.rs` | 133 | evidence-tool | std-path, std-time |
| `autographics-auto-driver/tests/user_behavior_feasibility.rs` | 255 | evidence-tool | glam, std-fs, std-path, std-time, vm-sdk |
| `autographics-auto-driver/tests/viewport3d_decision.rs` | 82 | evidence-tool | glam |
| `autographics-auto-driver/tests/viewport3d_depth_probe.rs` | 214 | evidence-tool | glam |
| `autographics-prototype/tests/asset_resources.rs` | 96 | evidence-tool | glam, std-sync |
| `autographics-prototype/tests/breakout_contract.rs` | 395 | evidence-tool | serde, std-fs, std-path, std-time |
| `autographics-prototype/tests/cache_contract.rs` | 137 | evidence-tool | —（无宿主标记） |
| `autographics-prototype/tests/composite_gate_contract.rs` | 481 | evidence-tool | iced |
| `autographics-prototype/tests/delivery_closure.rs` | 113 | evidence-tool | iced, std-fs, std-path, std-process |
| `autographics-prototype/tests/iced_seam_contract.rs` | 54 | evidence-tool | iced, wgpu |
| `autographics-prototype/tests/prototype_contract.rs` | 475 | evidence-tool | glam |
| `autographics-prototype/tests/squash_contract.rs` | 973 | evidence-tool | glam, serde, std-fs, std-path, std-time |
| `autographics-prototype/tests/surface_contract.rs` | 1450 | evidence-tool | —（无宿主标记） |
| `autographics-prototype/tests/surface_review_r2.rs` | 521 | evidence-tool | iced |
| `autographics-prototype/tests/surface_review_r3.rs` | 406 | evidence-tool | iced |
| `autographics-prototype/tests/surface_review_r4.rs` | 236 | evidence-tool | iced |
| `autographics-prototype/tests/threed_contract.rs` | 273 | evidence-tool | glam, std-path |
| `autographics-prototype/tests/verifier_contract.rs` | 212 | evidence-tool | glam, std-fs, std-path, std-time |