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
