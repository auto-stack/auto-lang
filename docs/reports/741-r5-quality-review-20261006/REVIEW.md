# PLAN-741 r5 合入后独立复审（2026-10-06）

**needs_fix，仅P3验收与证据问题；r5核心链接清理修复通过。**
reviewed_commit `6baed9bba84016dd8610221a9a0358195444385e`，plan_revision5，diff_base `39a4f9133`（完整SHA见source-manifest），r5合同基线0501b0f27。
修复48652d1e4、独立review/SD09/归档/merge均在祖先链。本会话不参与r5实施；从已提交HEAD建立独立detached检出，重读实际代码/合同/Specs并实跑。没有额外委派代理。
auto-down只读依赖 `fba6563ed2148ce85e68208863159b4ccccac710`；[源指纹](source-manifest.json)、[r5合同快照](reviewed-plan-r5.md.gz)、[规范增量冻结](reviewed-spec-delta-phase5.md)。主检出的SQLite未提交改动排除并保留。

## 核心实现结论

`link_object_staged`的nonzero和executor-error两出口已共同调用discard_own_staged+with_staged_cleanup；NotFound成功、用户目录不删除、真实删除错误不吞掉；publish也用同一helper。原生行为与SD09 canonical一致，没有发现新的核心缺陷。
- `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --offline -- --test-threads=1`：**58 passed +1 ignored**（lib15、cli11、hir_verify9、native_execution11、text_binding11、trace1）；先构建test-support。见[原型日志](prototype-tests.txt)。
- verify-ac-741.ps1 -SkipMainGates：**13/13**，fmt干净、all-targets原型零warning、真实PE/COFF和trace=b,a/12，见[one-shot](one-shot.txt)。
- 当前库非零共享锁反例77ms，正式超时60016ms：原失败+`link.cleanup`+OS32+残留完整路径+NOT committed+恢复办法；旧exe/obj/receipt字节均不变，旧exe exit5，释放锁后清理成功。见[nonzero](exact-nonzero.txt)、[timeout](exact-timeout.txt)。
- 当前公开run_exe持管道后代路径60095ms受控link.deadline；death-watch68ms确认后代不存在，见[public60](exact-public.txt)。未用返回瞬间的存活快照误报泄漏。

## Findings与修复合同

### P741-R5-QA-01 / P3：归档完成计数错误，门禁未检查frontmatter

`status: archived`、current_step34/total_steps35，但35唯一任务全部勾选。final_assertions只检查任务数量/勾选，不对照current_step/total_steps；真实文件输出ALL-ASSERTIONS-PASS。[audit](audit.json)/[原断言](final-assertions.txt)。受影响AC19/23/26、T22/28/33/35。
受控fixture把current_step改为0、total_steps改999或移除current_step仍exit0；有效35/35对照exit0。见[probe脚本](validator_probe.py)、[结果](validator-results.json)。
修复：解析frontmatter中的必需数字/状态/修订，current_step等于完成唯一ID数，total_steps等于实际任务总数；archive必须全部完成，executing允许准确的未完成数。缺字段/错误数字/状态与位置失配/重复勾选冲突必须非零。新增任务后不能把35硬编码成永恒总数。

### P741-R5-QA-02 / P3：active模式跳过实际父目录链接验证

AC26与T33要求实际active、模拟archive和真实archive均验证。实现active只用parent/archive解析原文本，允许7个链接从真实active父目录全部断链仍exit0。[fixture](validator-results.json)保留真实父目录断链列表；注入真正不存在的归档链接控制exit1，证明不是fixture失效。
**当前真实archive的7个链接及README/两模块导航/7个P741指针已通过，不重复报旧归档断链。**
修复：active先验真实文件父目录，再用明确搬移/相对路径转换后的副本模拟最终archive；最终归档后对真实文件复跑。不得拿未转换原文本的模拟检查冒充实际active。激活期ledger archive引用只可明确列暂态；merge检查全部P741 review指针（含P741-7），不能永久写死仅3..6。T22/28/33/35、AC13/19/23/26。

### P741-R5-QA-03 / P3：反例脚本库版本未绑定，测试数字过期

旧reproduce.ps1用deps目录的第一个libauto_ac_prototype-*.rlib；共享cache内同时存在旧/新库，本次首跑选旧库，输出原来的缺诊断。这不是当前实现回归：[旧脚本输出](reproducer.txt)必须与当前库证据分开。随后使用`cargo build --lib --message-format=json`返回的明确filename编译同一探针，当前nonzero/timeout均通过。[Cargo原始输出](library-artifact.jsonl)/[库与探针指纹](artifact-provenance.json)。
此外r5 verification/merge仍写54项，其lib14+其他43之和也为57；当前实际lib15+其他43=58。旧历史可保留，但当前完成摘要必须据日志更新。
修复：反例采用Cargo package/target/profile和明确filename（或干净的独立target目录且证实唯一），保留源码HEAD/Cargo.lock/依赖/库hash和实际命令；不得任意glob第一项。当前计数由测试日志派生。提供[明确artifact复现脚本](reproduce-exact-artifact.ps1)，同时绑定库/CLI，正常nonzero控制已实跑通过；完整60秒行为本次按同一明确库独立重放。首次脚本在沙箱内因工具链路径发现失败，提升权限后的控制通过，见[受限运行](wrapper-control.txt)/[实机控制](wrapper-control3.txt)，不假写首次通过。AC8/20/24/27、T23/29/34/35。

## 主仓回归与范围

| gate | exit | 耗时 | 证据 |
|---|---|---|---|
| root-check | 0 | 178.034s | [root-check.txt](root-check.txt) |
| root-t | 100 | 674.111s | [root-t.txt](root-t.txt) |
| root-tv | 0 | 6.807s | [root-tv.txt](root-tv.txt) |

root-check有382 warnings，不能宣称全仓零warning；原型健康检查零warning。本次daily观察到37项FAIL/TIMEOUT，其中8项不在r5两份历史失败清单并集；尚未判定是新回归或环境波动，不能一概销为旧flake。另有nextest框架级返回错误则按原日志保留。见[root失败对照](root-failure-comparison.json)。 r5仅独立experimental/ac-core改动；root相对合同0501的a2r_std.rs来自并行计划742，不是741。未单独运行tf/taa/t3/docs_gen；按本仓新分级门禁覆盖check/t/tv（docs_gen测试由mandatory daily alias自行包括）。root失败不以本次P3文档修复冒充已解决。

## 全部AC与delta对账

| AC | 结果 | 任务 | 证据/结论 |
|---|---|---|---|
| AC-01 | pass | T01/31 | 独立 workspace/Cargo 依赖与范围 diff；旧生产无741归因改动 |
| AC-02 | pass | T02/03 | text_binding11；四文本/双槽/enum/反例 |
| AC-03 | pass | T04/19 | hir_verify9+cli11；CheckedModule 私有/块图及拒绝矩阵 |
| AC-04 | pass | T03/05 | descriptor/verifier/native capability 源码和测试，无回退 |
| AC-05 | pass | T05/06 | native_execution11 + one-shot真实COFF/PE |
| AC-06 | pass | T06/07 | trace1 + one-shot b,a/12/溢出trap |
| AC-07 | pass | T07/21/32 | cli11/事务矩阵/两错误出口锁负例 |
| AC-08 | partial | T08/23/34/35 | 核心证据通过；QA03验收证据需整理 |
| AC-09 | pass | T09 | bool-local/return原生测试通过 |
| AC-10 | pass | T10 | bindings有效/错配原生及拒绝回归 |
| AC-11 | pass | T11 | owner/shared-body拒绝测试 |
| AC-12 | pass | T12/21/27 | publish/restore矩阵与真实旧制品保留 |
| AC-13 | partial | T13/22/28/33 | 当前README/归档链接可达；活动态漏检QA02 |
| AC-14 | pass | T14/20/26 | lib15资源回归+当前库正式60秒探针 |
| AC-15 | pass | T15 | one-shot fmt/all-targets原型零warning |
| AC-16 | pass | T19 | hir_verify9+cli块图拒绝，未改verify源码 |
| AC-17 | pass | T20/26 | 公开60s=60095ms，death-watch68ms后代不存在 |
| AC-18 | pass | T21/27/32 | publish矩阵/真实锁/旧三件套不变/释放恢复 |
| AC-19 | fail | T22/28/33 | 归档frontmatter34/35与35完成任务不符QA01 |
| AC-20 | partial | T23/29/34 | 新实跑原型58+1ignored及root结果；QA03 |
| AC-21 | pass | T26 | 受控启动/约束失败/lib回归+公开60s |
| AC-22 | pass | T27/32 | 共享discard_own_staged/publish与两出口诊断实测 |
| AC-23 | fail | T28/33 | 当前计数遗漏和活动态验证缺口QA01/02 |
| AC-24 | partial | T29/34/35 | 核心门禁通过；生命周期/证据收口不足 |
| AC-25 | pass | T32 | nonzero77ms/timeout60016ms；link.cleanup完整/旧exit5 |
| AC-26 | fail | T33/35 | 实际archive7链接可达，但metadata错误且active真父目录未验 |
| AC-27 | partial | T34/35 | 当前库明确绑定重放通过；旧脚本/测试计数QA03 |

原SD01..09保留；SD09的现有持久规范与当前代码通过。本次Phase6无需新的编译器规范增量，[r6影响说明](proposed-spec-delta-phase6.md)，supersedes/new/touched均[]。不改canonical或live ledger。root基线已有红，AC20/24的整体partial源于本轮QA证据/生命周期缺口，不声称所有既有红由741造成。

重放时使用此reviewed_commit的干净snapshot检出（不能拿后续r6活动合同替代r5输入）：`python -B validator_probe.py <r5-snapshot-root> <new-temp-fixture-dir>`；`powershell -NoProfile -ExecutionPolicy Bypass -File reproduce-exact-artifact.ps1 -RepoRoot <r5-snapshot-root> -CargoTargetDir <prototype-target> -ScratchRoot <new-temp-dir> -ProductionDeadline`。准备当前原型CLI和test-support库；Windows实机工具链可见。最终断言原命令见final-assertions.txt，frozen原实现见reviewed-final-assertions.py。

## 状态与下一步

继承用户此前“有问题激活同741并新增phase”的明确授权，r6 Phase6仅修复验收器/文档/证据，不改Rust产品代码。原r5 pass/merge保留历史，复审结论不继承到r6。
重开T22..24、T28..30、T33..35（包括同ID所有重复行），新增T36..38；26/38完成。T21/27/32和进程/原生核心任务保持完成。
next `/auto-plan:work`：T36最终断言与生命周期、T37证据版本/计数、T38独立review及最终归档门。仅文档验证变更不重跑cargo大档；若核心/依赖/配置变动，再按触面门禁执行。本次不实施这些修复，不merge产品。

## 本次复审检出清理与文档核验

仅本次detached auto-lang/auto-down检出：精确HEAD/Git clean、14,145/1,174条目扫描均零ReparsePoint；git worktree remove后空组目录已移除，无分支删除，742/743/ABI保留。[清理收据](cleanup.json)。规定guard脚本缺失、WSL VHDX不存在，使用既有741 §10授权的等价精确路径扫描。活动计划10条真实相对链接、报告链接、唯一38任务/完成26/同ID状态一致、README/两模块741导航均已核对；live ledger未改，激活暂态明确留最终归档。

冻结r5合同以gzip保存原始字节（解压SHA对应source-manifest）；原始控制台表格日志保留Windows换行及输出空格，格式门针对合同/代码/文档，原始日志不作排版修改。
