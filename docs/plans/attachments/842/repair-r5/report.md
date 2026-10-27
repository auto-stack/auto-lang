# PLAN-842 R5 修复与重新验证

用户于 2026-10-27 明确授权扩展自动修复及复审循环、修复 R5 并在通过后执行门禁与落地。[原始诊断](diagnosis.md) 和 [独立 R5](../review-r5/report.md) 为修复前记录。本轮提交 lang `a8b4a5bcf53d9be484cc38f028453a4cebbc947f`、UI `ab81f0406e3401f94f960c60eb87ff460763ded3`、Man `2e69777dba3cbd6cac96d1544bb5866388ead36a`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；lang 主线基面 `a547f52c5fbd5a24a203e94051af1e040c808f42`，三个计划工作树 clean。R6 独立 final 仍 pending。

- SDK 身份使用实际文件系统 canonical identity，三处受管收据入口共用；普通独立开发目录仍不当作安装，大小写敏感文件系统中不同目录不被仅因字符串大小写相近归一。已验证收据为交付来源版本的单一依据；显式 UI 源指向安装本身时使用该 UI 来源，外部 UI 源保留自己的身份。
- 本地 Cargo 图中的每个包自身都检查版本（包括 path-only 和入口自身）；无 package.version 的 Cargo 0.0.0 缺省合法，workspace.package.version 包括工作区根包可继承，合法 dev 环、版本约束、重命名与目标条件维持。原反例新增单元先 [红](r5-red.log) 后 [绿](r5-final-unit.log) 6/6。
- [最终真实 CLI 门](r9-final-delivery.log) 同一 Windows 安装 `sdk`/`SDK` 正控、显式语言根/语言 crate/UI 根/交付语言根/交付 UI 根五条路径坏收据拒绝、旧 app 和清单逐字保留、缺收据拒绝及恢复通过；受管所有文件、根锁、patch、复制期漂移、备份恢复等既有控制重放。合法同 SDK 的 delivery-manifest 两个 source revision 与安装收据一致。
- path-only 包无效版本在真实 workspace 与正式 delivery 两入口拒绝且保旧；[独立 Cargo 对照](r9-version-cargo-oracle.log) 合法生成/metadata 0，坏版本生成 1 且 metadata 101，旧 workspace 不变。可重放 [脚本](version-cargo-oracle.py)；路径环、错包名/版本、UI 错包、dev 环和 package 重命名完整重放。
- 最终 Full 实装 6806 文件，manifest SHA-256 `1f0c02cfdcdd4b366f0dc0b04337bae2c81f8f2e27d0dfc6cec4f406e5705862`；fresh lock、locked build、宿主探针、资源 verify、坏源和恢复通过。[复制期漂移](r9-staged-drift.log) 也保留。Core 636 文件，manifest SHA-256 `0810a68b592c3724109c819f8f00df7a87f4f6c9b1fa7e8380c3a19a42f45b80`，安装 verify、语言 VM、无 UI 报错、SDK locked check 与独立 consumer locked 3/3 均通过。最终 Vue 实际生成及生产构建通过。
- 语言 `cargo check`、来源单元 6/6、UI plan842 测试 7/7、UI 交付单元 1/1、Man check、driver 18/18、三仓完整 diff-check、修改 Rust 文件 rustfmt 和 Python 编译均通过。既有 compiler warnings 仍存在，不宣称全仓零 warning；复审需区分本次新增与基线问题。
- [最终 list-only 选择摘要](final-selection-summary.json.txt) language 3212、UI 3117、tt 3563、tb 3102；21/21 迁移目标实际 selected，含原列表 hash。测试 ID 对账重新绑定当前提交；原 9 recipe、10 token、2 parser 目标与配置未变，沿用先前定向运行证据。list-only 不等于执行全档。MPV DLL/真实播放、既有 ignored、用户批准 v0.7 门控的 jet/ark 实构建保持已披露限界。

独立 R6 审查、merge-owned 裸 cargo t 与触面档、到期批量回归、落地归档均未执行。本报告不是 pass 或交付收据。
