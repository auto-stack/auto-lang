# PLAN-842 R6 修复及最终重新验证

两次连续独立 final 未通过的共同不变量、根因与修复范围在 [修复前诊断](diagnosis.md)，独立反例在 [R6 审查](../review-r6/report.md)。本次用户明确扩展自动循环；这是追加三次中第 2 次 work→final，独立 R7 尚未审。最终源码 lang `a8b4a5bcf53d9be484cc38f028453a4cebbc947f`、UI `bd0e0f38253c5fd0676c8ef729c8c030d372ae82`、Man `9ac33ad871c4e0278b322e9712930ed059029e9c`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；lang 当前主线基面 `a547f52c5fbd5a24a203e94051af1e040c808f42`，三树 clean、相对三仓实际基面 diff-check 通过。

生成入口在 staging 前独立冻结实际选择的语言与 UI 受管 SDK 收据；UI 未显式选择时取默认安装的收据，显式选择另一安装时冻结该安装自己的完整收据。每个 UI crate 的实际 staging 原字节按所选收据的 owner/path/hash 校验后才允许投影，切换前两个收据都复核未变。交付的 UI revision 来自选中 UI 安装自己的冻结收据；普通开发 UI 仍使用自己的 Git 身份。原修复前真实反例与新回归先 [红](r6-red-ui-receipt.log) 后绿，UI 7/7、实际 delivery 1/1、rustfmt/三仓 diff-check/Python 编译通过。

## 真实来源、复制期与恢复

[新的真实 CLI 控制](r10-ui-receipt-controls.log) 使用独立项目和旧交付，对默认、显式同一安装、显式另一份安装三个受管 UI 来源，在目标 compiler 目录出现后、文件复制前修改源，在 staging 文件出现后立刻恢复源；三个进程均拒绝源与收据不符，旧 app sentinel 和清单不变。各自真实日志：[默认](ui-receipt-staged-drift-default.log)、[显式同一安装](ui-receipt-staged-drift-explicit-same.log)、[另一安装](ui-receipt-staged-drift-explicit-other.log)。[另一 UI 收据在复制期间变化](ui-receipt-changed-during-copy.log)也在提交前拒绝并保旧；原源/收据恢复后真实生成与 verifier 通过。永久回归位于 Man `scripts/man-distribution.py::installed_ui_receipt_controls`；其隔离 B 收据采用合成不同 revision 作区分性 oracle，安装文件完整性 verify 仍通过，不把该合成值称为真实 Git 来源。

另以两份**真实版本不同且各自 verify 的安装**做正控：[实际 UI B 来源](r10-real-other-ui-owner.log) `ab81f0406e3401f94f960c60eb87ff460763ded3` 与 producer A 的 UI `bd0e0f38253c5fd0676c8ef729c8c030d372ae82` 不同，真实 delivery 记录 B 的目录和 B 收据版本；随包 verifier 通过。对照脚本 [other-ui-owner-probe.py](other-ui-owner-probe.py)；[普通开发 UI 正控](r10-dev-ui-owner.log) 记录 UI 工作树自身 clean HEAD，并无借用安装 UI 来源。

最终 Full 6806 文件，manifest SHA-256 `253518e544b1a51470a09139df3c00fa7872a33b29d63735fb82c0e0c2a8fc7a`。[真实安装交付门](r10-final-delivery.log) install verify、fresh lock、locked Rust build、生成宿主、R5 SDK alias 五入口/缺收据、path-only 无效包版本、合法 dev 环与重命名、所有受管语言/patch/根输入/锁及 [语言 staging 漂移](r10-language-staged-drift.log)、失败备份和恢复均通过；该门复用前述单独执行且通过的 UI 收据控制，随后重验同一完整安装，再运行其余子项，未宣称重复运行。最终 Vue 真实生成与生产构建通过。

Core 636 文件，manifest SHA-256 `2be8683c4a5aa754b60e5fc79ed48a1cdf9687575e49b3836be165e15978e8fa`，install verify、真实 VM、UI 缺席具名提示、SDK locked check、外部 consumer fresh lock/locked 3/3 通过。语言源码和依赖未改，复用 [R5](../repair-r5/report.md) 中 6/6 来源单元、语言 check、Cargo 版本对照与 driver 18/18。既有 warnings 未清零；复审需区分本计划新增与旧警告。

[最终 list-only 选择摘要](final-selection-summary.json.txt) language 3212、UI 3117、tt 3563、tb 3102；21/21 迁移目标实际 listed/selected，包含每份原始列表 hash；测试 ID 对账重绑当前提交。先前 9 recipe、10 token、2 parser 定向执行的源码和配置无变化可复用；这不是整档运行证据。未测 MPV 硬件播放、原 ignored 及已获用户裁定 v0.7 再测的 jet/ark 构建保持披露。

R7 独立复审、merge-owned 裸 cargo t 和适用触面档、到期批量回归、落地归档均 pending；本文不是复审 pass 或交付收据。
