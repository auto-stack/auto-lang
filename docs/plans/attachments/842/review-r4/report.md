# PLAN-842 独立最终复审 R4：needs_fix

独立子代理 /root/review842；继承实施交接，但从实际提交、工作期诊断、原始测试选择清单和独立真实反例重建。计划 revision 2，工作组 D:/autostack/.wt/lang-842/{auto-lang,auto-ui,auto-man}；实际 clean HEAD：lang 729d5a060e1c5e3b7f004f5a7188f27a8831f7d2、UI b6072a664e4b20d49724ad26ce92df23d9374dcb、Man e44e7391df6b17b8e343d685522a95fb28dec759；down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。语言分叉基面含当前主线 a547f52c5fbd5a24a203e94051af1e040c808f42，UI 基面 4903bef64532750d1597627b1197f3d2cac1effc、Man 基面 9a75d3bab6eaeda961d5dcbf7932258f89e650af。规范与配置输入 [SHA-256](input-hashes.json)，受审的 [SD 副本](spec-delta-snapshot.md)。

## F-842-R4-01：普通 Cargo 路径依赖环被当作合法源码（P1）

违反 AC-02/04/08、V-02/V-06c、T-03 坏 producer 显式拒绝及 R3 诊断中对开发源本地 Cargo 闭包的真实正反控承诺。将完整最终 Full 安装的 sdk 复制为没有安装清单或执行文件的普通独立开发源；复制真实生成 UI fixture，环境只显式指定该目录为 AUTO_LANG_ROOT、AUTO_UI_SOURCE_ROOT 和独立 AUTO_RUST_WORKSPACE。真实安装版 auto.exe gen 正控 exit 0。向该开发源 crates/auto-ast/Cargo.toml 的 [dependencies] 追加 auto-lang-core = { path = "../auto-lang-core" }；由于 auto-lang-core 本就普通依赖 auto-ast，形成普通依赖环。在已有 app 末尾设置 sentinel，再运行同一真实命令：实际 exit 0 并覆盖 sentinel，生成 workspace 的 cargo metadata --offline exit 101，明确报告 cyclic package dependency。见 [摘要](summary.log)、[生成](cycle-gen.log)、[Cargo 诊断](cargo-metadata.log)。

使用同一坏开发源和真实安装版 auto.exe gen --delivery-root 对既有有效交付重试：实际 exit 0、旧 app sentinel 被覆盖；所得交付的 cargo metadata --offline exit 101，报告同一环。见 [交付摘要](cycle-delivery-summary.log)、[生成](cycle-delivery-gen.log)、[Cargo 诊断](cycle-delivery-cargo.log)。预期是具名拒绝已确定无法解析的源码，保护旧 app/workspace 或旧交付，而不是成功替换后才暴露 Cargo 错误。

根因：auto-api-gen/src/source_owner.rs::validate_development_dependencies 使用单个 visited 集，对任何重复节点直接返回 Ok；其覆盖测试构造的普通 app/shared 回环也被当成正例，仅证明遍历终止，未对照 Cargo 对依赖图合法性的判断。

控制：将此关系改为 auto-ast 的 dev-dependency 指向已正常依赖 auto-ast 的 auto-lang-core，形成含 dev 边的环，真实生成及 cargo metadata 均 exit 0（[摘要](dev-cycle-summary.log)）。因此不能机械地拒绝所有回边，须保持 Cargo 允许的 dev-dependency 场景；正常路径无网络注册表依赖及合法显式源亦须保持。

## F-842-R4-02：路径依赖包身份及版本未核实（P1）

同一坏源与保旧合同。上述普通 SDK 控制中，把 auto-ast 的 auto-val 依赖路径从 ../auto-val 改指存在的 ../auto-lex：真实 auto.exe gen exit 0 且覆盖原 app sentinel，cargo metadata exit 101 并报告在 auto-lex 路径未找到 auto-val。见 [摘要](identity-summary.log)、[Cargo 诊断](identity-cargo.log)。

仅指定合法实际路径但加入与包版本 0.1.0 不相容的 version = "=9.9.9" 时，真实生成同样 exit 0 并覆盖旧输出，cargo metadata exit 101；见 [摘要](version-summary.log)、[Cargo 诊断](version-cargo.log)。UI 侧同类反例：显式 UI 开发源的 auto-ui-runtime 将 auto-ui-compiler path 指向实际存在的 auto-ast，生成 exit 0 并覆盖旧 app，cargo metadata exit 101；见 [摘要](ui-identity-summary.log)、[Cargo 诊断](ui-identity-cargo.log)。

workspace 继承正反控：SDK workspace.dependencies 给 auto-val 真实路径及 =0.1.0，auto-ast 以 workspace = true 继承，gen 与 cargo metadata 均 exit 0；将被继承路径改指 auto-lex 后，gen 仍 exit 0 且覆盖旧 app，cargo metadata exit 101；见 [摘要](inherit-summary.log)、[Cargo 诊断](inherit-bad-cargo.log)。当前遍历以目标包自身的 package.name 作为验证期望，未将依赖键或显式 package 名和版本约束与该目标比较。

合法重命名控制：依赖声明 auto_val = { package = "auto-val", path = "../auto-val", features = ["serde"] }，生成及 cargo metadata 均 exit 0，见 [摘要](rename-summary.log)。修复须尊重 package 重命名、workspace 继承、有效版本、合法 dev 环、目标条件与既有可用开发来源；不将所有依赖键简单视作真实包名。三份成功 metadata 原始输出较大，保留独立探针工作区，其大小与哈希见 [记录](large-log-hashes.json)。

## 复审覆盖与结论

- 受管 Full 安装 SDK 的全文件预检、原始 staged crate/patch/root 字节验证及事务前 receipt 复核在源码中存在；实际坏内容/缺失/额外、根锁与复制期漂移均已由绑定最终提交的安装驱动逐项执行且旧输出保留。R3 的先前 findings 范围已闭；本轮新 finding 位于新增非受管开发源 Cargo 闭包判断。
- R2 的邻接/其他受管安装缺清单不回落与合法覆盖，R1 的备份失败/恢复，均核对本次真实安装日志；格式按实际三仓基面至 HEAD 的 diff-check 均通过。
- 独立读取最终 [选择摘要](../repair-r3/final-selection-summary.json)：list-only 为 true，绑定当前语言/UI HEAD；language 3212、UI 3117、tt 3563、tb 3102 实际 selected，21 个 841 迁移目标全部 matched。先前独立复核过原始清单和 18 项 driver 正反控；driver/migration manifest 本轮无更改，按输入复用。此证据不代表全档执行通过。
- AC-01/03 未受本轮源码改动的先前行为证据保留；AC-02/04/08 因真实无效开发 producer 被接受并覆盖旧输出 fail；AC-05 测试迁移选择定向通过但重绑后仍须复核，AC-06 本次 needs_fix，AC-07 merge pending。SD-04/06 的坏源保护描述目前不成立；其他 SD 保持待最终影响复审，不出版规范。
- 三次连续独立 final 非通过；在两次时已按技能记录共同不变量/诊断，但修复后新的 Cargo 图语义反例仍失败。按 auto-plan-work 技能三轮自动 work/review 修复循环上限，下一步先做次数核对及诊断/交接，不能将额外 final 复审重命名为 internal 以绕过上限；合入裸 cargo t、触面档、到期批量门及归档均未执行。

本轮没有运行整档或修改实施代码。F-R4-01/02 同属本地 Cargo producer 的图有效性与失败保旧不变量，但各有独立可复现输入和控制，需统一修复并验证所有受影响入口和合法 Cargo 图形。
