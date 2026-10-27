# PLAN-857 独立最终复审 R3

- `stage=review`、`review_scope=final`；独立子agent `/root/review_857`，未参与实施；Plan revision 1。受审 `4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837`，修复前 `e20b95e175f9184e13c00c78efa7b63349165276`；同一工作树 `D:/autostack/.wt/lang-857/auto-lang` / `plan-857-dev`。复审末HEAD一致、工作树源码clean。
- **结论：needs_fix。** R2原报的缺Home、真实partial、单侧错runner及像素/截图等8个负控与真实正控独立重放均通过，但相邻身份链出现两个可复现错误成功，须在work修复后复审。真实代码锁及已记录的真实当前producer结果未被更改或判为错误。
- [baseline.json.txt](baseline.json.txt) 记录提交、SDK/锁、运行器、规范输入、日志和摘要SHA；[spec-delta.md](spec-delta.md) 为冻结规范增量。所有负控都是临时复制的收据，不修改源码或原始证据。原始数据位于 `D:/autostack/plan-artifacts/plan-857/review-r3/`。

## 已通过的复核与范围

- 同一比较器和实际调用方 `tools/test_default_3d_baseline.py`：缺报告、真实partial、错runner、缺截图、错来源、错856基线、截图收据不匹配、逐像素差异均非零且无成功摘要；实际856/当前默认与完整Home正控通过。独立执行输出 `controls/results.json`。这是R2原反例及诊断中列明负控的闭合，不覆盖下面两个新增兄弟变异。
- 另测错误当前commit、SDK、单侧锁值、缺收据、phase33、漏case、报告收据hash及基线来源身份，均在具名门非零拒绝；见 [additional-controls.py.txt](additional-controls.py.txt) 和 `additional-controls/results.json`。
- 冻结producer runner `77b40869...` 的收据绑定 `4d9fa8e...`、来源身份 `f99c2bd...`；856 runner `05372dfc...`、原始真实Home 34阶段、截图/报告/收据hash、两锁和SDK通过核对。源文件477项及SDK681项/479包/17 local/29 edges独立验证通过；修复交接26个证据文件哈希均匹配。
- 相对 `e20b95e...` 仅来源清单和3个Python验证工具变化，Rust/AGP、两锁、SDK、场景及配置不变。因此R2独立重放的R1三项行为反例/正控、144点拾取及126点轴回投、855基准与857 8条性能/owner证据按R2复用；不因新HEAD重跑无关测试或完整门。

## F-857-R3-01 [P2] 两侧收据的锁文件键可同时错绑普通文件并产出成功摘要

- **原合同**：AC-01/06、T-05/06要求856/857精确两锁与SDK身份、失败传播及可核验摘要；R2修复说明声称检查两锁。这里是身份判断的新兄弟路径，不要求数字签名。
- **实际反例**：复制真实的856 baseline和当前producer runner收据、实际完整Home报告/截图，只将两份收据 `locks` 同时替换为工作树中 `autographics-auto-driver/README.md` 和 `autographics-prototype/Cargo.toml` 的真实SHA；其余内容、冻结exe与源码未改。运行实际比较器和旧/新80帧窗口程序，返回0并打印 `PASS`，summary 的 `locks` 写入这两个普通文件。具体输入与输出 `forged-lock-names/`、`forged-lock-output/summary.json`、`identity-replay/wrong-lock-keys-comparison/summary.json`，SHA见baseline；[可重放脚本](identity-controls.py.txt)。当前和856来源清单内的实际两份Cargo.lock仍是正确的，不能把此发现描述为锁文件实际被篡改。
- **原因**：`tools/check_default_3d_baseline.py` 的 `prepare` 遍历旧收据给出的任意键，对应比较当前收据与工作树文件hash，仅验证旧map长度为2；并未验证键就是driver与AGP两个 `Cargo.lock`，也未与各自冻结来源清单中的两锁字段对账。单侧改一个值会被拒，但双侧一致的错误键会被接受并作为锁身份写入摘要。
- **最小关闭条件**：对规范化后的确切两个锁路径逐一校验其原始文件hash与两份冻结来源身份及runner收据；拒绝双侧同错、遗漏、重复/路径别名、单侧错误，同时保留正确两锁/当前来源正控。只运行比较器/调用方相关定向控制，不重复整档。

## F-857-R3-02 [P2] 旧运行器能被收据标记为当前提交而通过比较器

- **原合同**：R2诊断要求报告、截图、runner与来源/提交相互绑定；Plan允许复用不变运行时代码，但须显式提供覆盖源码、依赖、配置和输入的复用证明，不能把旧构建直接改称新提交。
- **实际反例**：旧 `e20b95e...` 的真实producer runner SHA `bc18c3ae...` 及其真实34阶段Home报告/PNG均已保留在 `repair-r1/committed-native/`，旧收据明确记录该提交；当前 `4d9fa8e...` 的producer runner SHA为 `77b40869...`。在干净当前checkout中调用实际 `m2_evidence.record_runner`，传入旧的 `bc18...` 可执行文件，工具仍把当前source identity及 `code_commit=4d9fa8e...` 写入新runner收据。负控复制旧提交真实通过的Home报告/PNG并重新组装本地测试receipt，实际比较器执行856与该旧runner后 `exit 0`，summary称 `current_commit=4d9fa8e...`、runner却是 `bc18...`；见 `relabelled-old-report/`、`relabelled-old-comparison/summary.json` 和 [可重放脚本](identity-controls.py.txt)。该样本明确是复审负控重组收据，不是当前提交新产生的Home证据。
- **边界**：两提交间Rust运行时源码确实不变，但binary及AGP内嵌Git元数据不同；本发现是无复用证明时的错误归属，不能据此断言运行时功能不同或既有真实 `77b...` 正例无效。尝试重新实际启动旧runner原生探针时被其他进程遮挡，在phase0退出；此环境失败保留 `old-runner-native-driver.log.txt`，未记为功能故障或新实机通过。
- **原因**：`tools/m2_evidence.py::record_runner` 对调用方提供的任意exe归档后，独立读取当前checkout HEAD和source identity直接写入同一收据；`check_default_3d_baseline.py::prepare` 仅验证这些自报字段和文件hash相互一致，不核该exe实际构建来源或明确批准的跨提交复用证明。两阶段串联后可把旧exe及其真实旧报告标为当前提交并成功。
- **最小关闭条件**：冻结producer的二进制与构建输入/提交有可核对的产生或复用依据；若跨提交复用，独立校验并记录实际旧提交、输入/依赖/配置不变性及范围，不把旧二进制写成新提交构建。增加“旧runner+真实旧Home报告但无复用证明”拒绝与实际当前runner正控，检查调用方失败传播；无需引入数字签名或声称不可能的全局防篡改。

## 诊断、验收和后续

R2确实修复了原始单个错误/缺失case；新失败来自只校验收据间自洽却未绑定字段含义与真实构建来源的共享不变量。[R3诊断](diagnosis-r3.md)列出双侧一致错误、旧binary误标和正控矩阵。已有有效像素正例和行为修复可继续保留，不能因两条身份负控绿不了而回滚或重跑无关内容。

AC-02/03/04/05本轮复用的实现与已验证输入不变；AC-01/06的比较器身份链未闭，AC-07不通过。SD-01..04为待merge增量，当前Spec输入和hash见baseline。完整双端/受影响AGP及原新native-owner-content门仍由merge负责、pending；未修改实现、运行整档、合并或清理工作树。
