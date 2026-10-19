# PLAN-809 R9：needs_fix（代码finding已闭，仅正式独立交付验收待补）

2026-10-19；review_scope=final，重点复核R8修复和最后验收项。**F809-R8-01已关闭，原Ninja真实产物要求保持关闭；当前唯一开放finding为F809-R2-06正式独立交付验收。** 本轮无新增代码finding，不授予reviewed。

## 冻结基线与范围

- plan_revision=1；reviewed_commit=`390d02acc40b2141293730be7eee2b2f95113ec1`；prior=`2568f92d21468294c21b95089d20bde93ac3c0dc`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 原树`D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/结论前实现及兄弟依赖clean，HEAD未漂移，diff --check过。
- reviewer_identity：延续独立复审的Codex上下文，未实施本轮修复、未另开agent、不虚构模型/会话ID。
- [基线/哈希](review-r9-baseline.json.txt)、[送审Plan](review-r9-plan-input.md)、[冻结SD](review-r9-spec-delta.md)、[未变生产代码/依赖](review-r9-reuse-inputs.json.txt)。仅写主检出簿记、debt及附件，未改实施/发布Specs或ledger/合入/归档。

## F809-R8-01：CLOSED

当前helper使用split_once解析环境行、按ASCII大写匹配PATH/INCLUDE/LIB/LIBPATH，并在临时注入环境后检查是否真正能定位cl；不再仅凭SDK若干变量将缺编译器误判成成功分支。普通vcvars输出Path键及行尾空行不再导致原例失败。

独立重放[当前提交的原集成测试源码](review-r9-submitted-integration.rs)，追加与R8**相同**的[产物运行正控](review-r8-product-control.rs)；[重放脚本](review-r9-ninja-replay.ps1)以rustc --test编译，并链接既有生产库，打印源码/rlib SHA256。生产源码、manifest/锁与测试配置未变，范围核对见reuse-inputs；未把此命令称为cargo test或整档回归。

为避免追加控制先准备MSVC环境而掩盖原用例问题，分两次新进程执行：

1. 记录启动时`INITIAL_CL_IN_PATH=False`；先只执行R8失败的原用例v05_memory_and_cargo_real_serialized，**1/1通过**，真实Ninja Linking main.exe成功，无外层vcvars初始化。
2. 再开测试进程运行原8项及1项独立产物执行控制，**9/9通过**。Ninja实际编译/链接，生成9728字节main.exe实际exit0；真Cargo/Vue成功和失败传播原控制同时通过。

[完整日志](review-r9-ninja.log.txt)、[新产物身份/环境收据](review-r9-ninja-product-receipt.json.txt)。产物控制打印前缀与目录名沿用R8，但本轮确实重新编译测试源码、构建新PID目录中的产物并执行，未借旧exe作为结果。两次执行合计不等于10个不同测试：原失败用例在组合中重复，distinct为8项原测试+1项控制。

仅本轮变化的测试文件rustfmt通过。已审核范围内没有新增告警/格式/debug或生产语义问题；不把此次定向验证表述为全仓健康检查。

## F809-R2-06：OPEN，唯一剩余必需验收

最新Plan末行仍明确：“唯一余项：独立交付正式验收（fresh独立构建+重定位--locked重建，792 driver V01/V02形态定向真跑收据）”。本轮未提供新正式收据，可见809非review附件仍仅contract、handoff、旧777收据和`t06-792-delivery-regression-receipt.txt`；默认scratch-consumer目录不存在。

该t06摘要仍是9项delivery内容单测，不包含正式CLI请求路径的独立fresh构建、搬迁后不带target的locked重建及相应负控。R7已按实际测试和权威Spec核对两门区别，本轮不重复把内容单测计作正式交付门，也不重开已归档792。缺口归AC04/V06、T07/08，属于原work-owned合同，不是最终merge整档门。

因此不再启动一轮完整终验来重复确认已知pending，不凭本轮MSVC修复通过或之前的绿数推定正式交付通过。若提交可验证的完整原证据复用依据，亦须绑定受影响代码、输入、依赖、锁和配置；当前没有这类新依据。

## 复用与验收路由

本轮仅改集成测试的MSVC准备逻辑；生产core/请求读取/sidecar及静态准入行为均未变。R6/R7已验生产反例、core/model、轻consumer和777证据按各自范围及历史复用条件保留；未改记为本轮新跑。新helper则已用普通环境的新进程实际重放原红例及真实产物控制，没有依赖R8预初始化环境的正控结果。

SD-01..04候选及四条Spec路径/GOAL-003元数据未变，原sidecar规范冲突保持关闭，见冻结SD与当前源哈希；无新增规范/算法冲突。正式交付未来验收不能写成已完成。没有发布canonical知识或ledger。

AC01..03已审规则/请求支持矩阵保持pass；AC04 partial（仅正式独立交付未验）；AC05 partial（代码修复已通过，最终验收闭合证据待齐）；AC06 merge-owned pending。

outcome=needs_fix；status=executing；T01..06保留完成，T07正式独立交付和T08最终证据/交接收口仍开放，T09未完成、T10/11待merge；current_step=6/11。F809-R8-01关闭，当前没有未处理的代码finding。

next=work在同一树完成正式独立交付fresh真实build、不带target重定位后的cargo build --locked，以及受影响负控/来源身份检查，留下绑定当前代码、输入/依赖/锁和实际命令退出结果的完整收据，更新当前handoff后再审。不批准延期，也不把该验收转归merge以提前pass。

pending_closeout_gates：最终code/acceptance/Spec pass后，由merge在冻结提交执行裸cargo t、必要包及实际触面档、最终777/792并集；规范/ledger/归档、已合入接口SHA交接及wt-guard clean后清理仍pending。本轮无whole-tier t/tv/tt/tb/tu/taa/tf、正式driver/777或合入。
