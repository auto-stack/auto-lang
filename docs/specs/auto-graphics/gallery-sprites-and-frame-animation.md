# AutoGraphics Gallery 精灵与帧动画（G09）

Status: delivered（ag-examples PLAN-002）。本文记录 Gallery G09 的公共消费合同；引擎能力仍以 [sprites-and-image-resources](sprites-and-image-resources.md)、[game-actions-and-character2d](game-actions-and-character2d.md) 与现有场景/相机 Specs 为准。

## Gallery 内容

- G09 是一个逻辑项，含两个必需入口：`g09/atlas`（图集采样）和 `g09/character`（2D 角色动作）。交付后 ag-examples 为 10 个逻辑项、18 个必需子入口；G10/G12 和全 Gallery/AG 里程碑仍未完成。
- 两入口分别使用 scene/5 项目、PNG 纹理和 `sprites2d` 公共行为 API。普通打开/运行经 `tools/gallery.py` 的公开 CLI 传入受检 runner、场景和 Auto 源；不依赖 checks、probe 或私有 Host 开关。
- Atlas 使用三个非对称 32×32 帧，呈现不透明颜色、alpha/透明区、层序、按 tick 选帧及水平翻向；alt 源在相同 tick 改变帧节奏。
- Character 使用五格姿态图（idle、两格 run、jump、fall）。Auto 源依据动作输入、垂直运动和真实 grounded 结果选择姿态；左右运动决定 UV 翻向，停止后保留朝向。alt 只改变 run 节奏。示例验证角色绘制与实际角色运动一致，不代表完整平台游戏。
- 两张运行图均为仓内自制 CC0-1.0 RGBA8 PNG，带确定性标准库制作配方、帧清单和完整许可文本；每个场景保存自己的资源字节。图片 SHA、许可、帧范围、尺寸和消费源进入严格验收身份。坏尺寸/格式/帧、缺失许可、源或 PNG 漂移及错 runner 均不得继续显示 Ready。

## 固定消费引擎与目录边界

G09 按项锁为 `engine-locks/auto-ui-004.json`。它固定 auto-ui PLAN-004 交付 `e0fce13810f9e1af28182950d62cfce8722aa30a`，以及已含 PLAN-008 精确 PID 探针路径排除修复的 auto-ui master `0261ae9dacf786beb6dceb630be6af267f242625`。身份清单的规则只排除 `research/authoring-native-<PID>.agscene` 精确路径模式；编辑/删除受管源、任意未列文件或错误 pin 仍拒绝。原路径和 fresh consumer 均已复验：SDK 681 文件/17 个本地包，依赖闭包 479 包/29 条本地路径边/零禁止依赖；consumer identity 为 1431 项，SHA-256 `f59502b49d40338e3b2c8b24f332afb449bca4ef06dcc0b60afab6be7d9cfdeb`。锁定 Windows x64 runner SHA-256 `c960febaf741e4a896d48be13ad62c8ebea257523c72160e4b97347726d847c4`。

身份锁只作用于 G09；旧九项、三个原有引擎锁及各自 acceptance 保持原字节。G09 验收源绑定 `scene.agscene`、Auto 主/alt 源、PNG、清单与许可，不因生成一份新报告而接受旧源身份。

## 验收与交付边界

- `tools/verify_g09.py` 经固定 PLAN-004 consumer 无窗口执行实际 Auto/GPU，再以独立 PNG 解码期望核对帧像素。V-01..05 共 18 个必需 oracle 覆盖目录/身份、透明与半透明、帧/flip、角色姿态、方向/旧帧/错误空中姿态对照、生命周期与失败恢复。
- 最终验收为 18/18；旧 idle 像素冒充 run、wrong-air-run 和反向朝向的隔离负控按预期失败。`tools/verify_packages.py --only g09` 对同一单项 ZIP 在隔离位置 A、B 均以真实 Auto/GPU 通过 18/18，并拒绝 PNG、许可或按项锁篡改；精确恢复后通过。
- ZIP 含 31 个文件、75,997 字节，SHA-256 `66858a09c6fa85978f51feddcbf15c571aa664a47db9ac5a5e507a329bdd86de`。验证范围为 Windows x64；未运行 OS 键鼠自动化。完整 Gallery/AG 回归作为后续里程碑，不以本项结果代替。
- 本工作交付只增加 Gallery 内容与其消费验证；不新增或改变 SDK、renderer、Auto 语言服务、生产 AutoUI 集成，也不交付敌人、计分、完整关卡、声音/HUD 或远端发布。

Provenance: ag-examples PLAN-002（最终代码提交 `d1b4f9baf6f83575af4c669efa9b17f507d9003d`；交付 Plan 归档于该仓 `docs/plans/archived/002-autographics-gallery-sprites-and-frame-animation.md`），独立终审 r4。固定 engine lock、identity、runner 与实测在 Plan 的 T-01/T-02/T-05/T-06/T-08 收据中记录。身份生成器修复来源：auto-ui PLAN-008，master `0261ae9dacf786beb6dceb630be6af267f242625`。
