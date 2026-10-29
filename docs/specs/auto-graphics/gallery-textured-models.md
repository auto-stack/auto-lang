# AutoGraphics Gallery 纹理模型与 UV（G10a）

Status: delivered（ag-examples PLAN-003 r1）。这是内容仓的公开消费合同；底层支持域以 [textured-model-resources](textured-model-resources.md) 和 [scene-assets](scene-assets.md) 为准。

## 内容与阶段

Gallery 新增一个逻辑项 G10、一个必需入口 `g10/textures`；当前 inventory 为 11 项、19 个已登记入口。G10a textures ready；G10b animation/facing、G12、首十二项及全 Gallery/AG 仍 pending。标题、notes、静态 JSON/HTML 均保留这一阶段边界，不用现存唯一入口的 Ready 宣称整个 C10 完成。

普通启动为 `python tools/gallery.py launch g10/textures --engine <fixed-exe>`，只发公开 `--open`、`--behavior`、`--behavior-api v1`。scene/6 加载三件固定 Kenney 原件 platform/coin/flag、共用 colormap，以及一件自制六面 UV GLB/非对称不透明 8×8 PNG。主/alt Auto 只改变合法平移幅度（0.8/1.6）和显隐：tick2 隐藏 flag，tick3 恢复；不提供运行时 UV writer。

自制夹具呈现基准 UV、KHR offset[1.1,0.1]/90° rotation/scale[0.7,0.6]、clamp/repeat、nearest/linear、材质因子和双面背面。第六面 node translation[4.0,1.7,0.35] 在普通工程中实际覆盖后方面板，形成明确近远遮挡。编辑视口可观察其空间关系，验收不靠测试侧另造布局代替内容。

## 来源、闭包与预算

Kenney 源固定 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`，三原件及共图共 58,860 原始字节、SHA 不变；完整上游 README 的图像/模型 CC0 声明和代码 LICENSE 的 MIT 原文分别保留。每个运行身份有 `licenses/<sha>.LICENSE`，原件记录作者、commit、Git blob、长度/SHA、依赖和许可归属；自制 GLB/PNG/alt 为 CC0，标准库 Python 制作源、glTF2 float32/u16 与 PNG RGBA8/zlib9 配方一并交付。

正常工程为六个原始身份、62,884 字节；含 alternate PNG 后为七个身份、63,163 字节。重复保存的原件来源副本仍是相同身份。实际 Host asset union CPU 1,089,221 / expected GPU 1,094,312 字节；图像计入共享预算，几何与 renderer/target 开销分别记录。current/history/pending 总并集仍受 8 身份/32MiB raw、64MiB CPU/GPU 限制，不能为每个 closure 分配免费额度。

PNG 是模型闭包依赖，不套用 G09 sprite/帧表规则。每个原始 GLB root、ordered image source/SHA/长度/尺寸及 closure ID 受检；同 root 换合法 PNG 是不同 closure，实际共享 GPU store 中画面不同且恢复原 closure 后原图一致。路径逃逸、坏图、缺图/许可、错边、源漂移、错锁和空/缺失/重复/额外 oracle 均不得显示 Ready。验收精确绑定 20 个产品源文件与 7 份逐 SHA 许可；旧十项、四把旧锁和旧 acceptance 字节保持。

支持域继承 005：opaque single/double sided、baseColorFactor×sRGB、UV0/KHR transform，nearest/linear LOD0；原件 TANGENT 先检但不用于法线贴图。没有完整 PBR/mipmap/透明3D/GLB 动画、节点朝向能力、声音/HUD、游戏、生产 UI 或远端发布。

## 固定引擎与验收

按项锁 `engine-locks/auto-ui-005.json` 固定 auto-ui delivery `351919c76b4ef1d940fc7b8120254e321c41cba2`；source identity 1456 项，SHA `1e0aa531dd3ee2c966c4155059d4edd7858f495cc549bc3703879dc6c8f5efbb`；SDK pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`（681 文件/17 本地包、479 依赖包/29 本地边）。两 workspace 锁保持 005 已交付字节。独立构建的 Windows x64 runner SHA `844181264efe57684ede24e0eefc9b78cf5401922720be9179e0725c19e6dd4c`。每次验收同时核冻结源和实际参与构建的 SDK 副本；实际 SDK 篡改在 Cargo 构建前拒绝。

`verify_g10a.py` 的九个必需 oracle 通过真实 Storage→Host→Auto→GPU 验证原件与换图敏感性、六面 24 个独立 CPU/GPU ROI（最大误差1、容差2）、KHR 缺失/冻结 UV/无纹理、双面/深度、main/alt/冻结 Auto、同根异图共享缓存、双阶段候选失败保整帧/像素及恢复、Pause/Step/Stop、新 Run 和保存关闭搬迁重开。普通场景的四个重叠 ROI 从当前相机及几何独立投影求得；移除近面改变每个 ROI，反序绘制保留结果。tick2 实际隐藏像素与初态不同，tick3 恢复 tick1；重开后同样覆盖隐藏/恢复。删除显隐写入的真实源码反例必须运行失败。

单项 ZIP 为 42 个文件，122,138 字节，SHA `50ac023e7aa02ff964a92be98b23eb6d2d7f7e93a7385c8407e5ec964a690f07`。包内含过滤 catalog、普通启动工具、新锁、资源闭包、完整来源许可与小型 acceptance；引擎固定 consumer/EXE 独立提供。manifest 每个成员字节均核 SHA。隔离 A→B 和损坏修复后的 B 各通过九项真实 Run/保存/关闭/重开；十一类资源/许可/边/源/锁/oracle 控制拒绝，精确恢复再 Run。`--only g10` 只要求 G10a runner/consumer，未选中的 G09/G11 ZIP 不影响路由；被选中的入口仍需自己的 runner，不静默跳过。

验证平台为 Windows x64、无窗口实际 GPU；未执行 OS 键鼠自动化或声称 OS 人工走查。底层未变 005 的 20 Run/owner 证据明确复用，全 AG 归独立里程碑。

Provenance: [ag-examples PLAN-003](../../../../ag-examples/docs/plans/archived/003-autographics-gallery-textured-models.md)，独立 final R2 通过，内容 delivery/reviewed commit `4816807835b484653c2c31b6fbfff9907aef1ebf`；[auto-ui PLAN-005](../../../../auto-ui/docs/plans/archived/005-autographics-textured-models-and-uv.md)。规范为已验证交付的权威知识，计划和本地原始回执保留追溯信息。
