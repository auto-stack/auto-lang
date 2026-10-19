# PLAN-809 R8：needs_fix（真实Ninja已闭，独立交付与测试自举待闭）

2026-10-19；review_scope=final，聚焦R7两项及本轮新增测试。**原F809-R2-04的真实Ninja成功/产物可用要求已关闭；F809-R2-06正式独立交付验收仍开放。另发现修复新增的测试环境自举回归F809-R8-01。** 未合入、归档或宣称交付完成。

## 基线

- plan_revision=1；reviewed_commit=`2568f92d21468294c21b95089d20bde93ac3c0dc`；prior=`dca409c94000011016be0fd8c5b5ae9a1baa7ff1`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 原树`D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场、重放后及续接中断时实现/依赖clean，HEAD未漂移，diff --check通过。
- reviewer_identity：此前独立复审的Codex上下文，未参与本轮实现，无新agent、不虚构模型/会话ID。复审中断前验证已完成，续接仅核对证据与基线并记录；没有把其他Plan的交接混入809。
- [基线/哈希](review-r8-baseline.json.txt)、[送审Plan](review-r8-plan-input.md)、[SD冻结](review-r8-spec-delta.md)、[未变运行代码/依赖输入](review-r8-reuse-inputs.json.txt)。仅写主检出复审簿记及附件，未改实现或live Specs/ledger。

## 原两项闭合

| finding | 结果 | 独立证据 |
|---|---|---|
| F809-R2-04 三Builder真实效果矩阵中的Ninja成功产物 | **CLOSED** | 正确初始化MSVC环境后，Pac::new→set_port(win32)→resolve→build→真实NinjaBuilder执行[1/2]编译、[2/2]链接；新增独立产物控制实际执行main.exe返回0。原提交8项管理测试也全部通过 |
| F809-R2-06 正式独立交付fresh/重定位locked | **OPEN** | 当前Plan最新work行及用户澄清都明确仍待执行；未提供新正式收据位置。可见附件仍只有9项delivery内容单测摘要，默认scratch-consumer目录不存在 |

原sidecar对象/嵌套属性/优先级/缺源传播等代码finding保持关闭。本轮唯一代码diff是修改集成测试的环境准备，全部生产代码、依赖manifest/锁和测试档配置与R7未变；故限定复用R6/R7既有已验行为，未重跑或重记那些旧测试。

## 本轮真实Ninja产物收据

[产物收据](review-r8-ninja-product-receipt.json.txt)、[完整成功日志](review-r8-msvc-control.log.txt)：

- 输入为当前提交成功fixture同值C代码：`int main(void){return 0;}`，同样的pac项目和win32端口。
- 在完整VS2022/MSVC vcvars环境下调用实际生产Pac/NinjaBuilder；不是fake runner、内存文本断言或仅独立cl编译。
- 产物`target/review-809-r8-product-3864/build/main.exe`，9728字节，SHA256=`6cba5e3631329cf6549ac72e8ab0f40a2418d858bc1f7e54f5c8839aa6ff8455`；实际运行exit0。
- 同一测试二进制在已准备MSVC环境下9/9通过：当前提交原8项集成测试＋本轮1项独立产物执行控制；含真Cargo/Vue成功失败、真实Ninja成功及此前的失败控制。

这是所要求的Ninja成功实证，因此不再因下面的环境自举问题否认已经成功的生产构建。二进制路径在工作树清理后可能消失，输入、哈希、运行结果和原始日志已保存在主检出附件。

## F809-R8-01（P2，新增验收测试回归）：MSVC环境变量大小写导致自举漏掉Path

AC-05，T-08/09；影响`crates/auto-man/tests/plan809_management_core.rs:684..694`及:605的可用性判定。

实际vcvars dump输出键名是`Path=`，新helper只接受大小写敏感的`PATH=`。独立观测：INCLUDE/LIB/LIBPATH三键命中，**Path exact_submitted_match=False**。helper又用`env.len() >= 2`返回Some，:605即使找不到gcc/clang/cl，也把这个Some映射成空PathBuf，误判成功分支可执行。结果SDK变量被导入，编译器路径却没导入。

在本轮普通入口环境重放，Ninja执行到`cl.exe /c ...`后CreateProcess失败“找不到指定的文件”；原`v05_memory_and_cargo_real_serialized`和新增独立产物控制均失败，总计7过2红，exit101。原8项中是7过1红，新增控制另1红。见[红轮日志](review-r8-ninja.log.txt)。这不是纯源码推测，也不是机器没有VS——vswhere/vcvars均实际可用。

单变量正控：只在测试进程的父侧按Windows环境名大小写无关规则导入同一vcvars结果，再运行**同一个已编译测试二进制**，原8项＋产物控制全部通过，生成程序实际exit0。原测试文件和生产库没有改动，未用新算法掩盖红轮；见[环境正控脚本](review-r8-msvc-control.ps1)和成功日志。

修复：环境键按Windows语义匹配，尤其必须识别实际Path；在可用性准入时验证所需编译器/工具路径，而不能用“任意两项SDK环境存在”或空PathBuf代替。针对Path大小写输入和缺PATH/不完整vcvars输出加可区分控制；修复后在未预设VS PATH的普通入口重放。无需为此跑整档。此为本轮新增测试helper的回归，不是新增产品需求或生产Ninja算法缺陷。

## 重放方法及归因限制

[重放脚本](review-r8-ninja-replay.ps1)读取当前提交的集成测试源码（同一份[原集成测试快照](review-r8-submitted-integration.rs)已冻结），仅在文件末尾增加[1项产物执行控制](review-r8-product-control.rs)，用rustc --test重新编译并链接工作树既有生产rlib，日志打印原测试源码及rlib SHA256。原8项用例正文未改。测试按单线程运行。

此方法直接重编当前测试源码、调用实际已构建生产库，并非运行一个身份不明的旧test exe；本轮生产源码完全未变，复用现有生产库适用于该变化范围。**没有把此命令说成cargo test或整档cargo t**。先保留普通环境红轮，再用同一exe做环境正控，避免重编差异混淆因果。

唯一变更Rust文件rustfmt通过；运行代码/依赖与R7哈希相同，见reuse-inputs。未修改测试实现、发布规范、运行整档、正式792 driver或777。现有正式交付缺口已明确，未再启动完整最终验收来重复确认该已知pending。

## 独立交付仍缺什么

仍需`test/fixtures/modularity/generated-ui-consumer/driver.py`的正式CLI/请求路径验收或合同内等价真跑：V01不借开发锁/target的fresh解析与真实build、V02不携带target重定位后`cargo build --locked`、受影响的资源/源身份及失败控制。应记录输入、依赖、锁、代码和实际命令/退出结果。

附件中`t06-792-delivery-regression-receipt.txt`仍是9项内容单测，不能替代上述真实交付门；此为R2以来的原开放项，不是本轮新要求。用户本轮已澄清该项计划后续执行；未重开已归档的792，也不将work-owned验收转记到merge以提前pass。

## AC / SD / 路由

AC01..03已审规则/请求支持矩阵保持pass；AC04 partial（Ninja实际效果已闭，正式独立交付仍缺）；AC05 fail（新增测试自举红轮及闭合收据未齐）；AC06 merge-owned pending。

SD候选正文未改，三形态sidecar及已审核心/效果边界保持一致，四个Spec路径/GOAL-003元数据保留并冻结。没有新生产语义冲突；不得把独立交付未来步骤写成已完成。未发布canonical Specs或ledger。

outcome=needs_fix；status=executing；**T06生产接线与真实Builder效果本轮完成**，保留T01..05；T07正式独立交付、T08测试自举/收口仍开放，T09未完成、T10/11待merge；current_step=6/11。历史Ninja pending记录保留，以本轮实际关闭表为准。

next=同树work修复R8-01及其准入控制，完成R2-06正式fresh/重定位locked/受影响负控并留下绑定收据，校准当前handoff后提交复审。不能把普通环境红轮改记成功，亦无需重复否认已验证的Ninja产物。

pending_closeout_gates：最终代码/验收/Spec pass后，merge在冻结提交统一执行裸cargo t、必要包及实际触面档、最终777/792并集；规范/ledger/归档、已合入接口SHA交接及wt-guard clean后清理仍pending。本轮未授予合入资格。
