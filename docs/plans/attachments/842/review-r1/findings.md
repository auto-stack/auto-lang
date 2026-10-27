# PLAN-842 final review R1：needs_fix

审查在实施会话中，不能称独立。以下均对已提交的 lang `2de6176a9e573d04bbb72122996e70b90dd70288`、UI `533417e1c5d0bd171c83aeeae8b0dfdec783a41c`、Man `3f09779b2a3d4ce31f00d105f3f18520accf02b9` 和实际 Full 安装 CLI 复现。

## F-842-R1-01：不完整显式 UI 源被成功接受

T-03 / AC-02。先以已安装 Full `auto.exe gen` 为真实 fixture 建立有效工作区；再将 `AUTO_UI_SOURCE_ROOT` 指向仅有空目录 `crates/auto-ui-runtime`、缺少 `Cargo.toml` 的独立来源，再次生成。期望具名拒绝且旧清单保持；实际 exit 0，工作区 Cargo.toml 内容改变并指向无清单的来源，见 [摘要](partial-ui-summary.log) 和 [输出](partial-ui-generation.log)。根因是 `auto-api-gen::source_owner::ui_crate_source` 对显式/组内源只检查目录存在。需要覆盖缺失、损坏和错误包身份等邻接情况，以及合法来源正控。

## F-842-R1-02：失败重试提前回收唯一完整备份

T-03/T-07 / AC-04、AC-08。复制一个通过验证的真实交付作为 `.delivery.delivery-old-999999-1` 并添加事务标记；原交付根保持受支持清单但将 app/Cargo.toml 损坏，令真实项目 sidecar 引用缺失文件后用已安装 Full CLI 重试。期望生成失败时完整恢复备份仍在；实际 exit 1，受损根不变但完整备份已被删除，见 [摘要](recovery-summary.log) 和 [输出](recovery-generation.log)。根因是生成前 GC 仅用根清单 schema 判断完整性，先删除旧备份，再在 staging 失败；成功落位后另有 GC。须将回收限于本次成功提交后，并验证失败/成功及未知版本/外来目录控制。

修复后重新绑定受影响来源和真实安装交付，复审必须重新评估；无全档运行，最终独立验收和 merge 门均未通过。

## F-842-R1-01 邻接：不完整显式语言源

在修复 UI 来源期间以新安装 CLI 验证同一合同：`AUTO_LANG_ROOT` 指向含 `crates/auto-lang/Cargo.toml` 但清单语法损坏的独立来源（组内 UI 来源合法）。生成仍 exit 0 且改写工作区清单，未具名诊断；见 [摘要](partial-lang-summary.log) 和 [输出](partial-lang-generation.log)。同一修复须覆盖语言源、其工作区实际引用的包及合法正控。
