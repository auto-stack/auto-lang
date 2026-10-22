# auto-graphics — prototype profile

> Status: experimental（PLAN-829 实测事实；代码在 `autographics-dev` 分支，
> 见 [project.md](project.md)）。以下为单机（RTX 4060 Ti / Vulkan / 驱动
> 617.14 / DPI 2.0）probe 实测值，不外推跨平台。

## Supported surface（已验证）

- Window（winit 0.30）+ 离屏 RGBA8-sRGB target，同一 wgpu 27.0.1
  device/queue；最小化暂停呈现；resize 仅重配 target。
- 2D：正交 quads/circles、alpha blend、fontdue 文本（Breakout HUD 33px、
  squash HUD 32px）。
- 3D：正交/透视相机（z 0..1）、深度、光照网格、2048² 方向阴影贴图、
  无纹理 GLB（gltf 1.4）节点变换烘焙、程序化环境。
- 音频：kira/cpal；一次性 SFX + 循环流。
- 固定步长时钟：1/60s（Breakout）、1/120s（squash 物理）；冻结动作流+
  种子确定性回放（与独立参考实现 bit-exact）。

## Resource/update/lifecycle 合同

- 对象/资源身份 session 域 + 代际；失效/复用 ID 拒绝；跨 session 迟到
  事件拒绝。
- 有界身份键缓存：未改资源零重上传（warm 后 created-counter 平坦；
  3D 实测 uploads/frame 0）。
- 局部编辑只触碰具名对象（touch-set + revision 追踪）。
- 失败分类：zero_size_target / missing_file / bad_shader / bad_model_parse /
  stale_identity / cache_full / resource_released / surface_lost；失败不
  覆盖 last-good 帧。
- 重启/关闭：双游戏各 20× 重启基线稳定（真实游戏资源在环：GLB mesh store
  建后平坦、循环音频跨 retry、按 session 缓存）；视图增删由 drop-witness
  派生（mem::forget 目标必红）；关闭中加载等待 abort 回调 ack；teardown
  事实在 device 销毁**之后**采样。运行时不自建 owned worker 线程
  （iced spike 输入回退的短命 helper 线程已记录）。

## Replay/measurement 方法

- Canonical 回放：冻结流+种子在 `testdata/`；golden trace 由独立参考实现
  生成；回放按冻结容差比对（位置 1e-3 / 1e-2；事件精确）。
- 计量：sim+encode+submit 的 CPU wall、P50/P95、cold/warm、双视图；
  **GPU 计时不可用**（无 timestamp query）——必须如实标注，不得编造。
  原始报告在 `scratch/829/`（worktree，gitignored）+ 主检出附件。
- 验证器：门断言于独立状态（事件重计、哈希、device token、session id）；
  skip 即失败（sabotage 演示在案）。

## Profile 差异（已知，未复刻）

手柄/右键输入、Bevy stepping 工具、gl_compatibility 分支、生产 AutoUI。
无跨平台或竞品性能结论。

## PLAN-830 扩展（组合 Surface/Portal 与嵌套输入，Windows 单窗实测）

> 代码同样在 `autographics-dev`（830 落点 `764e87559`），master 仅知识文档。
> 证据目录：`docs/plans/attachments/830/work-r6/merge/`（release 报告，fingerprint
> `764e87559cff`）。

**已验证（在 829 单窗 probe 之上新增）**

- 组合模型：Surface（有自身状态的 iced UI / 2D canvas / 3D 场景）+ View +
  Portal（父 View 到子 Surface 的挂载）。有限无环多层组合，两层方向均实测：
  UI→3D→UI（S-01）、3D→UI→2D（S-02）；S-03 为同一 Squash 内容两个 Portal。
- 离屏 UI 作为被采样内容：每 Surface 一个 `Rgba8UnormSrgb` 采样目标
  （`RENDER_ATTACHMENT|COPY_SRC|TEXTURE_BINDING`），预乘 over 合成；2D 挂载用
  scissor 裁剪、3D 平面用挂载 host Surface 的透视相机；DAG 顺序保证先产出后被采样。
- 嵌套输入：窗口→视图→挂载链（2D 仿射 / 3D 射线+UV）→子内容像素；捕获绑挂载
  路径；焦点为唯一真实编辑目标（含合批与取消三态语义）；真实 caret 的 IME 锚点
  随挂载链投影；真实 OS 中文 IME 组合/提交已取证。
- 单设备归属对 **三种内容 adapter**（Iced/2D/3D）逐一校验；无法证明归属的
  raw-device 内容 fail closed（`DeviceUnverifiable`）。
- 计量：2D portal 每次 composite 真实写 160B uniform、3D 平面 128B（两条组合
  管线独立计数，另计内容 adapter）；验收要求分管线精确、总量严格等于分项之和，
  并可用 V-08 钩子证明“省略计数”会被门禁识别。PortalQuad 的 160B 是真实
  `write_buffer` 传输长度（有效字段 24B + 对齐/未用区），上传计量按 160B，
  不按 24B 折算。
- 内存事实：owned 离屏目标 payload = Σ`w*h*4`（按真实尺寸，排除驱动对齐/开销）
  与进程工作集分开报告；GPU/驱动驻留**不可测**（wgpu 27 无可移植查询）。

**边界（本 profile 未支持/未外推）**

- 仅 Windows 单原生窗口、平面（Flat2d/Plane3d）挂载；无曲面/任意网格 Portal、
  无跨进程 GPU 共享、无多窗口焦点。
- 3D 平面按声明序画家式合成（无逐平面深度遮挡）。
- iced 内容与 3D 场景每帧重绘（dirty 跳过仅 2D canvas）；稳态零 GPU→CPU
  readback（仅验收捕获）。
- iced 内部上传与驱动侧内存不可测；GPU timestamp 不可用（`gpu_timing_available
  = false`），不编造。
- 候选窗逐像素模板比对不在本 profile 证据范围；Null 剪贴板；组合管线创建尚未
  安装 wgpu error scope（缺口具名保留）。
- 生产 AutoUI/RQHost/Forge 接线、AutoShader/CAP、I1–I3 仍 pending。
## Proposed PLAN-833 Auto source profile (review/merge pending)

Windows x64 single-window keyboard/mouse; original GPU/Iced/stock widget/font/audio resource versions. Two retained thread-confined SDK instances; two Portal mounts share the first instance and advance it once per fixed tick. Auto output/event/scalar limits, slice/round budgets and error/close behavior are explicit in [auto-source-driver.md](auto-source-driver.md). GPU pixels/3D hashes, source changes, golden replay, actual OS-input focus control and reset/owner witnesses supplement the inherited 829/830 evidence; work self-check is not final acceptance or merge. Main Plan receipts bind exact device, compile/pump timings, memory/upload/target/pipeline counts. GPU timestamps are unavailable. Full engine, Auto Squash, production RQHost/AutoUI and Shader/CAP stay pending.

PLAN-833 R1 repair: content-upload accounting includes TextRenderer camera/glyph writes (64B + 48B/glyph, empty glyphs omit instance write). Counts follow child renders, not mount count; five glyphs add 304B / two rewrites. All quad/text resources must independently match the runtime at admission and each frame before any allocation/frame/GPU side effect. See auto-source-driver.md and the current repair receipt for exact controls and revised measurements.
