# PLAN-842 独立最终复审 R5：needs_fix

独立复审子代理 /root/review842；继承历史交接，但按当前源码、真实产物与独立反例重建结论。计划 revision 2，实施组 D:/autostack/.wt/lang-842/{auto-lang,auto-ui,auto-man}、分支 codex/plan-842-dev；实际 clean HEAD：lang 4ba5b39570a034aa7a262bf3f3a187099dbcdd5d、UI 171454071b8e19e0287f549ceb4f5b2ee87e0087、Man d11660ed74acfe15930e25f2bbcb746f77fcadb0；down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。语言当前主线基面 a547f52c5fbd5a24a203e94051af1e040c808f42 已包含进实施版本；UI 基面 4903bef64532750d1597627b1197f3d2cac1effc、Man 基面 9a75d3bab6eaeda961d5dcbf7932258f89e650af。规范与测试配置 [SHA-256](input-hashes.json)，受审 [SD 快照](spec-delta-snapshot.md)。

## F-842-R5-01：Windows 同目录大小写绕过受管安装身份（P1）

违反 AC-02/04/08、R09/R10/R12 和 R3 诊断的受管 SDK 全文件收据校验/坏源保旧。复制最终真实 Full 安装和正式 UI fixture 到独立探针目录；清除全部来源覆盖，仅指定独立 AUTO_RUST_WORKSPACE，执行复制安装的 auto.exe gen，正控 exit 0。把隔离安装的 sdk/crates/auto-ast/src/lib.rs 加入无害注释，在已有 workspace/project/src/main.rs 追加 sentinel；同一安装的 man-install verify exit 2，确认为受管字节与清单不符（[日志](sdk-case-install-verify.log)）。

指定 AUTO_LANG_ROOT 为复制安装的 SDK 路径、AUTO_UI_SOURCE_ROOT 为合法开发 UI 源。在本机 Windows 文件系统中 SDK 与 sdk 是同一目录，但显式最后一段写大写 SDK 时，真实 auto.exe gen exit 0，覆盖原 app sentinel（[摘要](sdk-case-summary.log)、[日志](sdk-case-generation.log)）；同一坏安装、其余输入相同，最后一段写小写 sdk 时，真实命令 exit 1、诊断具体受管文件不符，旧 app sentinel 保留（[摘要](sdk-lowercase-summary.log)、[日志](sdk-lowercase-generation.log)）。

对既有有效交付设置 AUTO_DELIVERY_SOURCE_ROOT 为同一大写 SDK、合法 UI 源，再运行真实 auto.exe gen --delivery-root：exit 0，受管未登记的 auto-ast 变化被复制进新包，原 app sentinel 被覆盖（[摘要](sdk-case-delivery-summary.log)、[日志](sdk-case-delivery.log)）。未触及任何实际安装或用户数据，以上均为副本。

根因：auto-api-gen/src/source_owner.rs::is_managed_sdk 与 InstalledSdkReceipt::open 先以区分大小写的 basename == "sdk" 检查并早退，未到达当前可执行文件邻接 SDK 的 canonicalize 比较；Windows 对上述两个路径实际指向同一目录。坏受管安装因此被错误分类为普通可修改开发源，不执行冻结收据/复制后校验。修复应以操作系统路径身份识别安装根，覆盖显式语言/UI、AUTO_LANG_CRATE、交付根及邻接和其他受管安装；保留合法无收据普通开发源。不得依靠 Windows 大小写字符串假定源不同。

## F-842-R5-02：path-only 本地包自身的无效版本未检查（P2）

违反 AC-02/04/08、V-02/V-06c 和 R4 诊断对合法 Cargo producer 及坏源保旧的约定。将真实 Full 安装 sdk 复制为没有安装收据和执行文件的独立非受管开发源，正式 UI fixture 以该目录作为 AUTO_LANG_ROOT 和 AUTO_UI_SOURCE_ROOT，独立工作区的真实 auto.exe gen 正控成功。将被 path-only 边引用的 crates/auto-val/Cargo.toml 中 package.version 从 0.1.0 改为语法有效 TOML 字符串 bogus，先为已有 app 追加 sentinel，再执行相同生成：exit 0，旧 app 被覆盖；对生成 workspace 执行 cargo metadata --offline 返回 101，明确报告该 package 版本无法解析（[摘要](dev-version-summary.log)、[Cargo 诊断](dev-version-cargo.log)）。

同一坏开发源用真实 auto.exe gen --delivery-root 对旧交付重试，也 exit 0、覆盖旧 app sentinel，交付内 cargo metadata --offline exit 101（[摘要](dev-version-delivery-summary.log)、[Cargo 诊断](dev-version-bad-delivery-cargo.log)）。预期该已知无效生产者具名失败且旧输出不变。

根因：validate_development_dependencies 中 package_version 仅在某个依赖声明包含 version 约束时调用；如果边只声明 path，即使目标自己的 package.version 非法也跳过版本解析。修复应检查实际图中每个 Cargo package 自身版本合法性，并保留 Cargo 合法的缺省版本、workspace.package.version 继承、重命名、dev 环和注册表离线语义。不要把仅由这次反例覆盖的依赖项直接特判为完整闭包。

## 本轮其他核验与限界

- R4 普通/构建边环、错误 package 名/版本、UI 错名，在两条真实入口已由 [实际门记录](../repair-r4/r8-final-delivery.log) 逐项失败保旧；合法 dev 环/重命名和 workspace 继承由实际 CLI 与 offline Cargo metadata 成功对照。R4 两项原输入的有条件修复闭合，但 F-R5-02 是同一图语义的不同情况。
- R3 受管全 SDK 文件集合、收据、staging 副本和根输入/锁的实测负控及失败恢复已核对；F-R5-01 是 Windows 物理同一来源路径分类的邻接情况。R1/R2 的失败保旧/缺清单控制按具体未变输入复用。
- 独立读取最终 [测试选择摘要](../repair-r4/final-selection-summary.json.txt)，并重新核对四份 raw nextest 列表的大小与 SHA-256，确认真实 21/21 映射目标 listed+selected；绑定本轮语言/UI 提交。此仅 list-only，不冒称整档运行。全变更范围 warning 归因和最终健康检查尚未完成，既有 warning、未测 MPV 视频及 1 ignored 留待复审，不作 pass。
- AC-01/03 本轮未改直接行为，保留既有具体证据；AC-02/04/08 因实际生产入口反例 fail；AC-05 迁移选择定向通过但下次变更后须重绑；AC-06 本轮 needs_fix；AC-07、裸 cargo t、触面档、到期 tf 和实际合入/归档均未开始。SD-04/06 对坏源保旧与受管身份的一致性尚不成立，其余条目按输入保留待最终复审，不将本次记录当作 SD pass。
- 本次是本轮用户授权内第三个也是最后一个自动 work/review 修复循环的最终复审。依据 [修复计数](../repair-r4/diagnosis.md) 和 auto-plan-work 上限，当前 non-pass 后停止自动修复并交接；不得通过重命名 internal 复审继续。没有修改实施代码，也没有运行合入全档。合入必须等待后续明确授权、修复、独立复审通过及冻结门禁。
