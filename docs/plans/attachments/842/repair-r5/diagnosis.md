# PLAN-842 R5 连续非通过诊断与交接（修复前快照）

> 用户随后已明确扩大循环上限；R5 修复及重验证见 [修复报告](report.md)，仍待独立复审。

最终独立 R5 对三次自动修复后的来源重新执行实际反例，仍有失败；本次用户授权的自动 work/review 循环 R2→R3、R3→R4、R4→R5 已用满 3/3。依据 [auto-plan-work](C:/Users/zhaop/.codex/skills/auto-plan-work/SKILL.md) 的自动循环上限，现暂停自动代码修复与复审，不运行复审后冻结门、合入、规范沉淀或归档。需用户明确扩展循环上限或另定处理方式。

## 共同不变量及根因

1. **受管来源身份不能因同一物理路径的不同表示降级**。Windows 同一安装的 `sdk` 与 `SDK` 是同一目录。隔离的真实 Full 安装中篡改受管 `auto-ast/src/lib.rs` 后，安装 verify exit 2；显式小写 `sdk` 的真实生成 exit 1 且保留旧 app，同一目录仅改为 `SDK` 则 gen exit 0 并覆盖 sentinel；正式交付同样 exit 0，覆盖旧 app 并把不符收据的字节复制到新包。`source_owner.rs` 中三处按 `file_name() == "sdk"` 进行大小写敏感短路（`installed_ui_source`、`is_managed_sdk`、`InstalledSdkReceipt::open`），身份判断先于邻接安装判定；单测和早期真实反例仅用规范小写路径，漏掉 Windows 同目录别名。见 [工作区摘要](r5-review-sdk-alias/summary.log)、[小写对照](r5-review-sdk-alias/lowercase-summary.log)、[交付摘要](r5-review-sdk-alias/delivery-summary.log)。
2. **非受管依赖闭包内每个本地包必须有 Cargo 接受的有效版本**。将真实普通开发源码 `auto-val` 的包版本改为 `bogus`，依赖只写 path、不写版本约束时，真实工作区生成和正式交付均 exit 0 并覆盖旧输出，但生成物 `cargo metadata --offline` exit 101；当前 `package_version` 仅在依赖项有 `version` 约束时调用。见 [工作区摘要](r5-review-dev-version/summary.log)、[Cargo 对照](r5-review-dev-version/bad-cargo.log)、[交付摘要](r5-review-dev-version/delivery-summary.log)。

## 后续修复设计与验证要求

- 针对平台真实文件身份及安装判定统一所有相关入口，避免仅对一处字面字符串打补丁；同时保留真实独立开发源、大小写敏感目录环境与已有受管清单缺失行为。受管无效/缺失/篡改收据在 workspace、显式 UI 源及正式交付所有可达入口先拒绝，再保障旧产物逐字不变。
- 对每个访问到的本地 package 解析有效版本，包括 path-only、包本身和 workspace 继承；无版本的 Cargo 合法默认、semver 约束、别名、合法 dev 环和原有 target/workspace 场景继续以 Cargo offline 正反控对照。两入口各用真实 CLI 反例、旧输出保护、恢复验证；重跑受影响的最终安装来源与现有受管闭包/复制期验证并重绑三仓提交。
- 完成后仍需独立 final 审查全部受影响 AC/SD 与真实输入，得到针对新冻结提交的 pass 后才能执行 merge-owned 门与归档；本诊断不是实现或复审 pass。
