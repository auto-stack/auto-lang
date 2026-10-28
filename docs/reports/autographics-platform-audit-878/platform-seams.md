# 3. 平台接缝审计（platform-seams）

XP-0 触面对照审计。对照表（§5.3 of PLAN-878）要求的六类触面逐项给出：观察入口、
已观察事实、分类（shared-candidate / host-only / adapter-needed / unresolved /
hypothesis）、宿主归属与缺口。依赖与文件级证据见 [dependency-map.md](dependency-map.md)。

参照（审计时为 master 工作副本、尚未入 git 的文档，SHA-256 前 16 位，路径供定位）：
`docs/design/autographics/platform-architecture-and-research.md`（跨端设计假设 v3，
`f8fcaa6b778907b4`）、[integration-contract / prototype-profile /
game-actions-and-character2d 等 Specs]（canonical，10 个 SHA 见 877 baseline）。

## 1. 六触面对照

### 1.1 场景 / 身份 / 时钟 / 事件

| 项 | 观察事实（文件/符号） | 分类 |
|---|---|---|
| 身份 | `core/id.rs`：代次分配器（IdAllocator）+ 全局 `static NEXT: AtomicU64`（跨会话唯一 id） | shared-candidate；**hypothesis H-01**：AtomicU64 在 wasm32-unknown-unknown（无 atomics feature）的可用性未测；u64 id→JS Number 需精确编码（设计文档 §2 已要求，无实现） |
| 场景提交 | `core/scene.rs`（scene-change submission）、`core/composition/**`（DAG 调度、ViewId/PortalId、`ContentKind{Iced,Game2d,Scene3d,Canvas2d}` 纯判别） | shared-candidate；composition 明文禁依赖 wgpu/winit/iced（`mod.rs:21`） |
| 时钟 | `core/time.rs`/`runtime.rs`/`demo.rs`：仅 `std::time::Duration`；**core 无墙钟读取**；`Instant::now()` 只在 driver VM 调度（`lib.rs`×6、`script.rs:330`、`squash.rs:575,613`）与 host 层 | core=shared-candidate；driver 调度=adapter-needed；**H-02**：`std::time::Instant` 在 wasm 的可用性/精度未测；Run/tick/代次语义（game-actions Spec）在 core 内为逻辑 tick，无 OS 依赖 |
| 事件 | `core/events.rs`（动作/事件队列）；host 侧 `composition_input.rs` 将 winit 事件翻译为 composition 输入 | 语义=shared-candidate；翻译层=host-only（winit 耦合，wasm 需 web 事件 adapter——边在图，未测） |

### 1.2 renderer / 资产

| 项 | 观察事实 | 分类 |
|---|---|---|
| wgpu 实例 | AGP 与生产 iced_wgpu 共享 wgpu 27.0.1/27.0.4 线（`Cargo.toml:17-21` 注释契约）；driver 经 AGP 传递 | adapter-needed：wgpu 27 有 WebGPU/WebGL（经 web-sys，wasm 图内保留）与 Vulkan/Metal/GLES（android/ios 图内保留）后端——**路线存在 ≠ 本管线可用（U-03）** |
| 表面呈现 | `renderer/target.rs:248,254` `surface.get_current_texture()`（含重试）；surface 生命周期由 host 层管理 | adapter-needed：移动/浏览器 surface 失效/重建路径未测（U-04）；设计文档 §4 device-lost 合同无对应实现 |
| readback | `renderer/capture.rs:61` buffer `map_async`（回调空、轮询等待模式） | adapter-needed：wasm 上 buffer 映射时序未测；同步等待模式在浏览器主线程可能死锁——**同步等待不能由 cfg 自动消失**（合同 §5.3） |
| 2D 管线 | `render2d/**`（sprite/batch，fontdue 字形）+ 1 处 unsafe（bytemuck） | adapter-needed（wgpu 耦合）；字形栅格化本身 shared-candidate |
| 3D 管线 | `render3d/**`（camera/anim/procedural + gltf 上传） | 同上；深度/相机算法纯数学部分 shared-candidate |
| 资产 | `assets/glb.rs`：gltf 解析（纯）→ wgpu 纹理/缓冲上传；文件经 `std::fs` | 解析=shared-candidate；上传=adapter-needed；**fs 读取在浏览器需 fetch adapter（现无接口）**；静态 GLB 支持集不扩张（合同） |

### 1.3 窗口 / UI / 输入 / IME

| 项 | 观察事实 | 分类 |
|---|---|---|
| 窗口 | `host/**`（winit event loop、surface_runtime、iced_surface） | host-only：桌面 winit 逻辑属宿主；wasm/android 后端依赖边在图但 host 层未抽象（直接持有 winit 类型） |
| UI 接缝 | iced 0.14 离屏接缝：公共 runtime API `UserInterface::{build,update,draw}`（`Cargo.toml:42-44` 注释 + `host/iced_spike.rs`、`iced_surface.rs`） | host-only（当前形态）；与 DOM 非"直接纹理来源"——设计 §3：DOM 子树不能自动成为可采样纹理 |
| IME/组合输入 | `host/composition_input.rs:480-586`：`#[cfg(windows)]`/`#[cfg(not(windows))]` 成对分支（产品代码唯一成对平台分支） | host-only；非 windows 分支在场=有初步隔离意识，但 wasm/移动 IME 无路线 |
| 输入语义 | `core/composition/input`（InputOutcome 语义层）与 winit 键盘映射分离（`driver/src/host.rs`、`squash_host.rs` 用 `winit::keyboard`） | 语义层 shared-candidate；映射层 host-only；touch/pen/软键盘映射无实现（XP-4 范围） |

### 1.4 OS / 文件 / 线程

| 项 | 观察事实 | 分类 |
|---|---|---|
| 文件 | driver `authoring/storage.rs`（kernel32 `MoveFileExW` 直链+非 windows 硬错误桩）、`authoring/assets.rs:106`（reparse point 检测，windows 分支内嵌）；AGP host 层 `std::fs`×16 | host-only；**移动/浏览器存储 provider 无接口**（设计 §4 存储/下载行无对应实现）；作者存储在非桌面明确不支持（代码自述） |
| 线程/同步 | driver 测试面 spawn/进程/env 重度使用（evidence-tool，106 文件）；产品面：AGP host 线程×3、同步原语集中 host/renderer | 产品同步原语=host-only/adapter-needed；**evidence-tool 面在非桌面整体不可用（U-06），不阻塞产品闭包** |
| 动态库 | driver 闭含 `libloading`（经 SDK，无门控） | unresolved（U-01）：SDK 宿主栈跨端可编译性未测 |
| 环境变量 | driver 测试面 env×95（工具面）；产品面少 | 工具面不迁；产品面 env 读取点在 wasm 无对应物，需 provider（清单在 dependency-map 附录） |

### 1.5 音频 / 生命周期

| 项 | 观察事实 | 分类 |
|---|---|---|
| 音频 | kira 0.9（`["ogg"]`）：symphonia 软解（纯）+ 平台后端（wasapi/alsa/coreaudio/oboe） | 解码 shared-candidate；输出 host-only；**H-03**：移动端音频会话/用户激活/挂起约束未测（U-05） |
| device 失效 | 现状：原生故障保帧合同（原生 830 证据）；设计 §4 要求"保逻辑状态/资源身份，不承诺像素" | **缺口**：device 重建/旧代次隔离在代码中无实现；跨端不能沿用原生保帧承诺 |
| 挂起/恢复 | 无后台暂停/恢复处理代码；driver VM 有 ParkedSegment 停靠语义（逻辑层） | 逻辑停靠 shared-candidate；墙钟债务/事件迟到规则无实现（XP-4） |

### 1.6 构建 / 消费 / 工具

| 项 | 观察事实 | 分类 |
|---|---|---|
| 构建脚本 | AGP `build.rs`（git 指纹注入，构建机需 git；目标无关） | build-tool |
| 消费链 | `tools/prepare_sdk.py`（git archive 重建 SDK）、`package_consumer.py`、`verify_source.py`（cargo metadata 闭包断言：target 不逃逸、17 本地包、29 path 边、forbidden 零、AGP 锁身份保留） | evidence-tool：Python/Git 工具链，非产品路径；其身份/导出假设是迁仓触面（见 migration-readiness） |
| SDK 身份 | `research/source-lock.json`（681 文件）+ `baseline.json.txt`（pin 7b987ca8）+ `sdk-Cargo.lock.txt` | 身份机制健全（本审计 T-01 已重放）；**pin 对象在 auto-lang 已不可达，仅保留副本存在**——迁仓/长期保存风险 |
| 工作区形态 | 两 workspace 各自 `[workspace]`+锁；driver 排除 `.sdk-source`（`Cargo.toml:1-2`） | 迁仓时保持该形态可零改动消费（详见 migration-readiness） |

## 2. DOM / Canvas / 3D 边界结论（V-04）

对照设计文档 §2/§3 的分层表，AG 当前实现与浏览器候选的关系：

1. **DOM overlay 与可采样 UI 是两种能力**：AG 的 iced 离屏接缝（`UserInterface` 公共 API）属于"可离屏绘制的 UI adapter"候选路线（设计 §3 第 4 行）；DOM 子树（Vue 组件）不能直接成为纹理来源——当前代码无任何 DOM 桥，无违反，也无实现。
2. **renderer 可用 ≠ Auto 程序可执行 ≠ 完整 UI 可交互**：三个独立结论（设计 §2）。本审计证据：renderer 路线=adapter-needed（U-03）；Auto 执行=driver VM 闭包含 SDK 宿主栈（U-01，最重缺口）；UI 交互=winit/iced host-only（XP-2 范围）。静态审计不输出"WASM 已通过"。
3. **u64/JS Number 与 f32 桥接**：`output.rs` 的 Packet（u64 session/sequence、f32 values）与 `core/id.rs` 的 u64 代次正是设计 §2 要求精确编码的承载面；当前无任何 JS 序列化层，跨端时须显式编码（不检查转 Number 会静默截断 >2^53 身份）。
4. **整帧一次发布**：原生同设备 UI→3D→UI 证据（830）不能抵扣 DOM 域；composition 的 commit 语义（whole-state swap，`composition/mod.rs:15-17`）是逻辑层合同，物理原子呈现跨域不成立（设计 §3 末段）。

## 3. 移动原生与移动 Web 各自缺口

| 路线 | 依赖边在图 | 静态可见缺口 | 未测假设 |
|---|---|---|---|
| 移动原生（android/ios） | winit(android-activity/ndk/objc2)、wgpu(Vulkan/Metal)、kira(oboe)、音频边 | AGP host 直接持有 winit 桌面形态；storage 硬错误桩（ios/android 无作者存储）；无 surface 生命周期/挂起恢复处理；driver 测试面（进程/env/端口）不可用 | U-01 SDK 栈、U-04 surface、U-05 音频会话；NDK/Xcode 工具链本机未装（未编译） |
| 移动 Web/WebView | web-sys/wasm-bindgen 在 wasm 图保留 | SDK 宿主栈无门控携带（U-01）；Instant/AtomicU64（H-01/H-02）；fs/env 无 provider；DOM/Canvas 桥不存在 | WebView 能力差异按设计 §1 不以名称推断；XP-4 另测 |

移动 Web/WebView 不用 target triple 替代（合同 §3）：上表中"移动 Web"列是浏览器宿主
约束推演，非 target 解析结论。

## 4. 共享候选清单（可共享部分 / 平台特有部分 总表）

**shared-candidate（22 文件 + auto-val + SDK 纯语义段）**：core 语义族（id/scene/
events/time/runtime/demo/composition/**）、driver script.rs/output.rs/squash.rs、
fontdue 字形栅格、gltf 解析、serde/sha2 数据面、SDK auto-val。
共同前置：H-01/H-02 原子与时钟、u64/f32 编码合同、无文件系统访问（现状满足）。

**adapter-needed（16 文件 + SDK 宿主栈）**：render2d/render3d/renderer/assets
（wgpu 资源上传/表面/readback）、driver VM 调度（Instant 计时→注入时钟）、
SDK auto-lang-core（axum/notify/libloading/stacker/sysinfo——最重，需逐项
provider 化或裁剪）。

**host-only（73 文件）**：AGP host/**、两仓 bin、driver authoring/host/squash_host/
consumer（winit/iced/kira/线程/进程/fs/env）。

**unresolved**：U-01..U-08（dependency-map §7）。

**非支持声明（XP-0 收口即有效）**：本审计不宣称任何非 Windows 平台可运行；
作者存储非 Windows 明确不支持（代码内声明）；移动原生与移动 Web 均 require
XP-1..4 探针后才可谈能力。
