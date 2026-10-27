# PLAN-842 独立最终复审 R2：needs_fix

独立复审者 /root/review842：不同子代理上下文，继承了执行对话，但结论从当前合同、已提交差异及独立实际反例重建。计划 r2；工作组 D:/autostack/.wt/lang-842/{auto-lang,auto-ui,auto-man}，分支 codex/plan-842-dev；复审 lang 4e84a8895300731eb94e9cde1737b761745cff39、UI eab3d85c3e7aa0af5ae58c08cdeb6a4d691c5691、Man 8215bf635db76c9f3d1e49ef46a56966a0738d1c、down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，实施树 clean。

比较基面 lang ecc78701124fe6e0ab6f0e262148a1d853ba2af5、UI 4903bef64532750d1597627b1197f3d2cac1effc、Man 9a75d3bab6eaeda961d5dcbf7932258f89e650af。规范和验证配置字节哈希见 [input-hashes.json](input-hashes.json)，受审的正式 SD 和精确提案冻结见 [spec-delta-snapshot.md](spec-delta-snapshot.md)。

语言 master 已前进到 bfefde1a6469e46153e822bcd5b3efde361dc073，新增 824/858 等代码和 auto-lang/project.md 等规范变更；本次只审上面冻结的三仓提交与所列主检出规范，未来并集未审、未验证。合入前须重新核实真实并集及规范冲突，并按改变的代码、配置和输入重审受影响证据。

## F-842-R2-01：自动发现损坏语言源仍生成并覆盖（P1，原合同遗漏）

违反 AC-02/04、V-02/V-04、R10 与 T-03 工作区坏源保旧。复制 r1-final-full 完整安装到独立探针目录；复制生成 UI 的正式 fixture，清除 AUTO_LANG_ROOT、AUTO_LANG_CRATE、AUTO_DELIVERY_SOURCE_ROOT、AUTO_UI_SOURCE_ROOT 和 AUTO_UI_TOOL_BIN，只将 AUTO_RUST_WORKSPACE 指向独立目录。先运行复制安装的 auto.exe gen 得到有效旧输出；在生成 project/src/main.rs 追加独立 sentinel；再将复制安装中 sdk/crates/auto-lang/Cargo.toml 写成无效 TOML “ [package ”，再执行 auto.exe gen。预期具名拒绝并保留原输出；实测 exit 0、覆盖旧 sentinel 并报告生成成功。[摘要](implicit-malformed-summary.log)、[原始输出](implicit-malformed-generation.log)。根因：auto-api-gen/src/rust_workspace.rs::compute_auto_lang_rel_path 仅显式根使用新完整性检查，安装自动发现只检查路径存在。

邻接实际反例：安装语言 crate 缺席时，CWD 旁的语言根只有 auto-lang、缺 auto-lang-core，仍 exit 0 并选中该残缺根、覆盖旧输出；CWD 直接发现的 auto-lang 清单损坏也被接受。分别见 [兄弟根](sibling-partial-summary.log)、[CWD 直接根](direct-malformed-summary.log)。两探针显式指定合法 UI 源，以隔离语言源选择。修复应覆盖实际选择的安装、direct、sibling 和编译期候选及有效安装、显式开发源控制；R1 的显式根单测尚不能证明这些分支。

## F-842-R2-02：安装缺源时静默借开发源替换旧产物（P1，原合同遗漏）

违反 AC-02/04/08、V-02/V-06c、R09/R10，以及现行 generated-ui-distribution 的“不回退主检出”失败合同。将独立 Full 安装中 sdk/crates/auto-lang 重命名至旁边备份而不删除；清除来源覆盖，对同一正式 fixture 的有效旧交付执行复制安装的 auto.exe gen --delivery-root。实际 exit 0，交付清单 producer.source_root 指向 D:/autostack/.wt/lang-842/auto-lang 开发树，旧 app sentinel 被覆盖；预期缺闭包时具名失败且旧产物不变。[摘要](delivery-missing-installed-summary.log)、[原始输出](delivery-missing-installed-generation.log)。原因是 rust_ui.rs::delivery_producer_root 顺序查找 find(valid)，跳过失效邻接 SDK 而落到 CWD 开发源；同一安装的工作区入口也回落并覆盖，见 [摘要](missing-installed-workspace-summary.log)。

项目放在系统临时目录、安装语言源缺失、仅指定有效 UI 根时，工作区会静默指向构建时固化的 D:/autostack/.wt/lang-842/auto-lang/crates/auto-lang，见 [编译期回落](compiled-fallback-summary.log)。应在受管安装失效时拒绝暗中借其他源，同时保留合法显式覆盖与非安装开发发现的合同。

区分性对照：安装语言源存在，故意损坏 auto-ast/Cargo.toml 时，实际交付 exit 1、报告 TOML 解析错误，旧 app sentinel 与清单均保留；见 [日志](delivery-bad-toml-control.log)。不将此正确拒绝误报为缺陷。

## F-842-R2-03：冻结提交差异格式不洁（P3，验收健康缺口）

语言仓对实际起始基面运行 git diff --check ecc78701124fe6e0ab6f0e262148a1d853ba2af5 HEAD，exit 1：test/fixtures/modularity/plan842-retirement/check.py:136: new blank line at EOF。clean 工作树上仅执行 git diff --check 会遗漏已提交差异。UI 和 Man 从实际起始基面至 HEAD 各 exit 0。修复后须核验完整提交差异；受影响 T-04/T-08。

## 验收和风险覆盖

| 范围 | 复核和结论 |
|---|---|
| AC-01、R01–R07、R14–R16 | 核对冻结矩阵、外部旧入口编译拒绝、新 API 和版本证据，以及相应代码差异；未发现本轮新反例。按实际输入保留定向证据，不冒充已通过最终复审。 |
| AC-02、R09–R10 | fail：F-R2-01/02 覆盖安装、CWD direct/sibling、编译期和正式交付分支。显式源 R1 控制本轮 2/2 通过（[日志](workspace-replay.log)），但它未覆盖上述分支。 |
| AC-03、R08 | 核读独立宿主 consumer 的默认、显式、无 UI、线程和退出以及删注册 red/恢复 green；现有必要单份登记有证据，此轮不冒充全平台新运行。 |
| AC-04 | fail：安装缺源仍错误更换旧输出。R1 的事务恢复测试本轮 2/2 通过（[日志](recovery-replay.log)）；旧安装升级证据仅按相同输入复用。 |
| AC-05 | partial：已核对测试 ID 清单、版本和迁移方案；F-R2-03 和后续新源码会影响格式及来源绑定。 |
| AC-06 | fail：独立最终复审 needs_fix；既有 warning 在日志内，尚无全变更范围零未处理 warning 的证明。 |
| AC-07 | merge-owned 未开始；语言主检出已前进，并集需另行审验。 |
| AC-08 | fail：真实 Full 安装缺语言闭包却错误回落并替换交付；坏 TOML 的直接拒绝正控已核对。 |

SD-01/02/03/05 的退出、保留、身份及版本提案与核读矩阵/代码/证据大体一致，未对未触及模块重跑全套；SD-04/06 的来源错误拒绝与失败保护尚不成立，不能作为已实现规范沉淀。来源修复、重验及并集后仍需重新核对 SD-01..06 与目标规范。另核读投影三态、未知 schema/安装回滚、777 Core 隔离和真实 fresh/locked 输出；未运行全档或修改实施代码。

修复所有 finding 后固定新提交，跑有区分性的真实坏源、合法源和旧产物字节控制，重新绑定 T-03/04/07/08/09，再独立复审；裸 cargo t 和适用触面档仍为 review pass 后的 merge-owned 门禁。
