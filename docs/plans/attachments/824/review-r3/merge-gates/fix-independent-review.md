# PLAN-824 M1 编译修复受影响独立复审

reviewer `/root/review824_sync`，用户明确授权、独立只读子代理。候选 `b4b8f48b3b3910af5ede773df35fba994f22cb6f`，依赖 auto-ui `4903bef64532750d1597627b1197f3d2cac1effc`、auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`，均 clean。初次冻结 `fdc6f3a0e` 的 [cargo t 日志](cargo-t.log.txt) 含 824 的 `StepOut::dynamic` E0063 及旧 UI 引用已退役 `UiParseContext` 的 179 项错误。

- 824 唯一代码修复是旧 `#[cfg(test)]` 通用判决 helper 的 `dynamic: None`，符合旧非 dynamic case，不改生产协议；[judge-fix.log](judge-fix.log.txt) 5/5、0 ignored（root 有既有 167 条警告）。
- Plan 841 archive 收据确认 UI `4903bef` 与已在候选基线为祖先的 lang `f16bb468b` 配套交付；干净配套 UI 从旧 `b4fe` 切到该交付修订。[ui-dependency-check.log](ui-dependency-check.log.txt) 的 `cargo check --locked -p auto-ui-dialect` 通过；根 Cargo.lock 与主干相同。新版 UI 下正式 824 语料 1/1（实施会话执行）。
- 复审 outcome **pass**，仅限该修复及依赖配对；独立代理只读、未执行测试。旧门禁结果无效，所有合入门禁在修复后重新运行。
- 随后主干 `5b65aa68c` → `ee06acf563440c65a5069200ac32621da538ea45` 仅其他计划簿记，未改代码/依赖/配置/规范；同树重放 17/17 `=`，`b4b8f48b3 → 8da513aefe8885e28ec8716f054be5e8d639800d`，详见 [range-diff](merge-gates/rebase-final-range-diff.txt)。证据按代码、输入、依赖不变复用；门禁只绑定新 SHA。
