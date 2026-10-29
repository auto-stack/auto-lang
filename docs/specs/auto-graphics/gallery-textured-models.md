# AutoGraphics Gallery 纹理模型、节点动画与视觉朝向（G10）

Status: delivered（ag-examples PLAN-004 r1，内容a27979788ca835f85e19b1778d9653e83573c3d6）。
本页是内容仓当前消费合同；底层支持域以[textured-model-resources](textured-model-resources.md)、
[scene-assets](scene-assets.md)和[node-animation-and-facing](node-animation-and-facing.md)为准。

## Current content and consumption

G10 is one logical item with exactly g10/textures and g10/animation; total
Gallery inventory is 11 logical items / 20 registered entries. G12, the first
twelve items and the full Gallery/AG milestones remain pending. Both G10
projects select engine-locks/auto-ui-006.json, one Windows x64 native runner
and one complete filtered archive. Textures project bytes remain the delivered
PLAN-003 originals and are reverified under 006, without modifying its history.

Actual engine delivery is auto-ui c681663e6ff392d70471abbec560e9d9daf12622,
1477-file source identity e2994efa1b1ee6d55f79de7bd194bf59c1745c41f60b6c8bbaac7a67197e7243,
SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 /681 frozen files/17 local packages/
479 metadata packages/29 path edges, and the unchanged driver/AGP locks.
Dedicated consumer runner SHA256
93bc1bf5c89989d185b9d660f95386b0ad6fe85675a4d94608f29596608f1720.
Other ten items and five historical lock files retain their prior bytes.
Ordinary Gallery launch emits public --open/--behavior/--behavior-api, using
v1 for textures and node-animation3d for animation; verification adapters do
not appear in ordinary launch.

## Original character and public Auto

Kenney source remains 3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0.
character.glb SHA7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1,
87624 bytes, original shared colormap SHA106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3,
10108 bytes. All four ordinal clips remain intact: static/idle/walk/jump,
8 nodes/44 LINEAR channels/337 channel samples, no skin or conversion.
Every raw runtime identity has complete source attribution/license binding,
including original CC0 asset statement and distinct MIT repository code notice.

Animation scene/6 binds project-local main.at through a content-addressed source.
Initial static clip is visible; real Auto decodes aggregated normal action input,
computes normalized camera-relative four-way movement, jump/gravity, idle/walk/
jump choice, phase seconds at fixed dt, and radians yaw retained at rest.
main-alt doubles only phase speed. Host validates/samples the explicit visual
setter; it does not select clip/yaw or calculate the gameplay trajectory.
Color/shadow consume the same sampled pose, while collision/support/trigger
evaluation retains frozen physical shapes. Pause holds phase; neutral Step
advances one dt; Stop and fresh Run restore the original initial pose.
Loop exact duration wraps and one-shot holds the last key. Old Run/slot-generation
tokens refuse without partially publishing a candidate.

## Budgets and strict readiness

The two projects plus legal alternate image use exactly eight raw identities,
150787 bytes; normal current union without alternate is seven /150508 bytes.
The actual six-closure resident Run includes the alternate image and reserves
32592 pose CPU bytes; total CPU1186261 / expected GPU1159692 bytes.
No project switch erases history/pending/current residency charges. Shared
asset data and separately reserved per-instance pose state remain within
the inherited raw8/32MiB, CPU/GPU64MiB, nodes128/clips16/channels128/samples8192,
expanded primitive/instance limits. Managed96KiB/user256KiB/combined352KiB,
32 shared credits and original cumulative CPU/deadline guards remain unchanged.

Current acceptance binds exactly the two entry IDs, measured 006 runner,
complete item source set and eight SHA-bound license files. Textures retain
their exact nine required oracles, freshly executed under 006. Animation has
eight required oracles: original_four_clips_source_gpu,
phase_main_alt_frozen_control, camera_relative_input_facing,
pose_shadow_physics_independent, pause_step_stop_fresh_run,
stale_identity_last_good, resource_union_budget, save_close_reopen_run.
Wrong/old runner, malformed or missing source/clip/image/license/closure,
source drift and empty/missing/duplicate/extra proofs cannot become Ready.

The animation adapter reads original JSON/BIN keys and independent slerp/hierarchy
math, never production animation sampling for expected vertices. Source vertices
and stable GPU silhouette probes cover original clips at zero/mid/end/loop/
one-shot and independently phased shared instances. Frozen tracks, wrong yaw,
frozen/disabled shadow and source phase variants are distinguishable. Actual
Gallery bound controller directions are additionally checked against independent
camera/action yaw and GPU masks. Same-input 240-tick visual-on/base-pose runs
preserve positions/camera/events after excluding differing Run IDs. Full last-good
state/pixels survive invalid clip and prepare failure; old Run and recycled slot
generation refuse. Warm mesh/texture uploads do not increase during updates.

## Archive and limits

One filtered g10.zip includes both complete project closures, content-addressed
scripts, all original notices/licenses, selected 006 lock and normal launcher.
Engine consumer/executable are independently supplied. Every ZIP member hash
is checked. Isolated A->B and repaired B run both real Auto/GPU entry gates,
including SaveAs, close, cold reopen and fresh input-driven Run. Structural
inspection alone is insufficient. Fixed inputs are restored after deliberate
bad source/clip/image/closure/license/lock/proof controls and the same entries
recover. Package verifier routes explicitly by the selected fixed 005/006 lock;
unselected G09/G11 ZIPs do not select or suppress G10 checks.

This delivery covers finite original no-skin node animation and visual facing,
not skin/morph/CUBIC, blend trees/root motion, sound/HUD, a complete game,
browser engine, remote publication or production UI integration. Windows x64
windowless actual GPU only; no OS input automation or full-AG pass claimed.
Unchanged engine owner/20-cycle evidence may be reused by exact source impact;
it never substitutes for new Gallery-specific runtime and archive checks.

## Historical PLAN-003 /005 baseline

以下原合同按005历史阶段保留，其中inventory19/单入口/G10b pending、旧runner和旧包统计均为历史，
当前两入口/006身份及Ready规则以上文为准。原textures工程和历史收据没有重写为006证据。

## Historical G10a textured-model contract (PLAN-003 /005)

Status: delivered（ag-examples PLAN-003 r1）。这是内容仓的公开消费合同；底层支持域以 [textured-model-resources](textured-model-resources.md) 和 [scene-assets](scene-assets.md) 为准。

### 内容与阶段

Gallery 新增一个逻辑项 G10、一个必需入口 `g10/textures`；当前 inventory 为 11 项、19 个已登记入口。G10a textures ready；G10b animation/facing、G12、首十二项及全 Gallery/AG 仍 pending。标题、notes、静态 JSON/HTML 均保留这一阶段边界，不用现存唯一入口的 Ready 宣称整个 C10 完成。

普通启动为 `python tools/gallery.py launch g10/textures --engine <fixed-exe>`，只发公开 `--open`、`--behavior`、`--behavior-api v1`。scene/6 加载三件固定 Kenney 原件 platform/coin/flag、共用 colormap，以及一件自制六面 UV GLB/非对称不透明 8×8 PNG。主/alt Auto 只改变合法平移幅度（0.8/1.6）和显隐：tick2 隐藏 flag，tick3 恢复；不提供运行时 UV writer。

自制夹具呈现基准 UV、KHR offset[1.1,0.1]/90° rotation/scale[0.7,0.6]、clamp/repeat、nearest/linear、材质因子和双面背面。第六面 node translation[4.0,1.7,0.35] 在普通工程中实际覆盖后方面板，形成明确近远遮挡。编辑视口可观察其空间关系，验收不靠测试侧另造布局代替内容。

### 来源、闭包与预算

Kenney 源固定 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`，三原件及共图共 58,860 原始字节、SHA 不变；完整上游 README 的图像/模型 CC0 声明和代码 LICENSE 的 MIT 原文分别保留。每个运行身份有 `licenses/<sha>.LICENSE`，原件记录作者、commit、Git blob、长度/SHA、依赖和许可归属；自制 GLB/PNG/alt 为 CC0，标准库 Python 制作源、glTF2 float32/u16 与 PNG RGBA8/zlib9 配方一并交付。

正常工程为六个原始身份、62,884 字节；含 alternate PNG 后为七个身份、63,163 字节。重复保存的原件来源副本仍是相同身份。实际 Host asset union CPU 1,089,221 / expected GPU 1,094,312 字节；图像计入共享预算，几何与 renderer/target 开销分别记录。current/history/pending 总并集仍受 8 身份/32MiB raw、64MiB CPU/GPU 限制，不能为每个 closure 分配免费额度。

PNG 是模型闭包依赖，不套用 G09 sprite/帧表规则。每个原始 GLB root、ordered image source/SHA/长度/尺寸及 closure ID 受检；同 root 换合法 PNG 是不同 closure，实际共享 GPU store 中画面不同且恢复原 closure 后原图一致。路径逃逸、坏图、缺图/许可、错边、源漂移、错锁和空/缺失/重复/额外 oracle 均不得显示 Ready。验收精确绑定 20 个产品源文件与 7 份逐 SHA 许可；旧十项、四把旧锁和旧 acceptance 字节保持。

支持域继承 005：opaque single/double sided、baseColorFactor×sRGB、UV0/KHR transform，nearest/linear LOD0；原件 TANGENT 先检但不用于法线贴图。没有完整 PBR/mipmap/透明3D/GLB 动画、节点朝向能力、声音/HUD、游戏、生产 UI 或远端发布。

### 固定引擎与验收

按项锁 `engine-locks/auto-ui-005.json` 固定 auto-ui delivery `351919c76b4ef1d940fc7b8120254e321c41cba2`；source identity 1456 项，SHA `1e0aa531dd3ee2c966c4155059d4edd7858f495cc549bc3703879dc6c8f5efbb`；SDK pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`（681 文件/17 本地包、479 依赖包/29 本地边）。两 workspace 锁保持 005 已交付字节。独立构建的 Windows x64 runner SHA `844181264efe57684ede24e0eefc9b78cf5401922720be9179e0725c19e6dd4c`。每次验收同时核冻结源和实际参与构建的 SDK 副本；实际 SDK 篡改在 Cargo 构建前拒绝。

`verify_g10a.py` 的九个必需 oracle 通过真实 Storage→Host→Auto→GPU 验证原件与换图敏感性、六面 24 个独立 CPU/GPU ROI（最大误差1、容差2）、KHR 缺失/冻结 UV/无纹理、双面/深度、main/alt/冻结 Auto、同根异图共享缓存、双阶段候选失败保整帧/像素及恢复、Pause/Step/Stop、新 Run 和保存关闭搬迁重开。普通场景的四个重叠 ROI 从当前相机及几何独立投影求得；移除近面改变每个 ROI，反序绘制保留结果。tick2 实际隐藏像素与初态不同，tick3 恢复 tick1；重开后同样覆盖隐藏/恢复。删除显隐写入的真实源码反例必须运行失败。

单项 ZIP 为 42 个文件，122,138 字节，SHA `50ac023e7aa02ff964a92be98b23eb6d2d7f7e93a7385c8407e5ec964a690f07`。包内含过滤 catalog、普通启动工具、新锁、资源闭包、完整来源许可与小型 acceptance；引擎固定 consumer/EXE 独立提供。manifest 每个成员字节均核 SHA。隔离 A→B 和损坏修复后的 B 各通过九项真实 Run/保存/关闭/重开；十一类资源/许可/边/源/锁/oracle 控制拒绝，精确恢复再 Run。`--only g10` 只要求 G10a runner/consumer，未选中的 G09/G11 ZIP 不影响路由；被选中的入口仍需自己的 runner，不静默跳过。

验证平台为 Windows x64、无窗口实际 GPU；未执行 OS 键鼠自动化或声称 OS 人工走查。底层未变 005 的 20 Run/owner 证据明确复用，全 AG 归独立里程碑。

Provenance: [ag-examples PLAN-003](../../../../ag-examples/docs/plans/archived/003-autographics-gallery-textured-models.md)，独立 final R2 通过，内容 delivery/reviewed commit `4816807835b484653c2c31b6fbfff9907aef1ebf`；[auto-ui PLAN-005](../../../../auto-ui/docs/plans/archived/005-autographics-textured-models-and-uv.md)。规范为已验证交付的权威知识，计划和本地原始回执保留追溯信息。

## Current delivery provenance

[ag-examples PLAN-004](../../../../ag-examples/docs/plans/archived/004-autographics-gallery-node-animation-and-facing.md),
r1, independent final R2 pass at a98ca21ca4bd2ecf271b5b663bd11821feb3bc0b;
linear equivalent content delivery a27979788ca835f85e19b1778d9653e83573c3d6.
Current ZIP55 members (54 manifest-bound files), SHA256
e7428355adaf6411d53d27bf6cffefab0c048200fcecdab192ac53975835a99b.
Frozen gate independently validates current identity/archive and revision-bound
reuse of the unchanged original GPU/physical/lifecycle/offline scenarios.
Independent R2 adds66 state controls,8 standalone bad-proof refusals and2 actual
publication rollback controls. Full AG/Gallery and G12 remain separate milestones.
