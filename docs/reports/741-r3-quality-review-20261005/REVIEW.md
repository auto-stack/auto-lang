# PLAN-741 r3 合入后独立复审（2026-10-05）

结论：**needs_fix**。已有 HIR→COFF→PE 闭环与前两轮反例修复成立；新反例显示子进程回收和清理错误报告仍未兑现原合同。未改 Rust 实现、canonical Spec、live ledger；依既有用户授权激活同一741，追加 Phase4。

## 基线与独立性

- plan_id=PLAN-741，plan_revision=3，reviewed_commit=`f6a945db849f57789a9e151402e22b8aa57eb7f8`；diff_base=`ed2d00b9076e68fdc41ca59dfa096c753cee5f9e`。
- r3实施 dd89a0156/4857ddb83/510979929/fe2334f55，合并/归档 a0b3750c4/30cb89292/abc6e289f，祖先链已核实。主线随后743文档/工具提交不改本次原型及 root Rust/config；激活起点 `a5f07086b6737959e09bd7d24266143a167131cd`。
- 本会话未参与 r3 实施。自行读源码、核对实际 worktree 和 clean 状态，在 detached `D:/autostack/.wt/lang-741/auto-lang` 重跑测试与自构反例；root 构建使用只读兄弟 auto-down @ fba6563e，无链接。
- rustc 1.98.1，Cranelift0.126.2，windows-sys0.59.0，Windows x64/rust-lld/SDK；lock、配置及 Spec byte-hash 见 [source-manifest.json](source-manifest.json)。当前 link.rs hash 与实施报告不同：fe2334f55统一了备份失败恢复诊断，本轮实际测试包含该改动。
- 原型 workspace 独立、不依赖 auto-lang/auto_val。741 r3 Rust生产面零改；整个区间 root仅有并行计划的 a2r_std.rs `fs.mkdir_all` facade 改动，不能笼统声称整个主线区间零 diff。

## 发现

| ID | 级别 | 已观察到的缺口 | 修复方向/对应合同 |
|---|---|---|---|
| P741-R3-QA-01 | P2 | 正式 run_exe 的60秒路径约60106ms返回 link.deadline，继承管道后代仍存活；最终1秒复现脚本三次均返回后后代存活。代码先 spawn 再加入 Job；Job创建/分配失败默默返回 None，而 reap 在无约束时不 join reader | 受控启动消除约束前的后代创建窗口；约束不可用受控拒绝/等价回收，不能 detach；T-20/26，AC-14/17/21，SD-07 |
| P741-R3-QA-02 | P2 | 暂存 exe 被禁止 DELETE 共享的句柄占用，同时收据暂存路径为目录：publish_artifacts返回 link.receipt，但 owned tmp exe 幸存；诊断只列收据错误，没列清理失败/残留 exe | 收集而非吞掉 remove/rename 错误，原错误+残留路径+实际OS错误；明确提交状态；T-21/27，§5.6/AC-18/22，SD-08 |
| P741-R3-QA-03 | P3 | 已归档 r3 current_step=23/24，T-08仍未勾；T-24已勾且收据确认实际归档/清理，计数不能代表完成任务 | 依据 r1/r3收据归一历史完成记录；新Phase按重开验收/新任务重新计数，最终归档断言；T-22/28，AC-19/23 |

**QA-01 的因果边界：**探针在自身创建的外层 Job 中运行，但这不证明每次内层 Job分配失败。独立 job-probe对活跃父进程曾返回 job_some=true；同场景重跑也出现正确回收：正式路径60020ms、后代已退出，以及快速3次中的2次正确回收；最终入库脚本-ProductionDeadline重跑正式路径60082ms也正确回收，1秒三次仍都留下后代（final-production-reproducer.txt）。这里的真实缺陷是时序相关的资源回收保证，不能把“外层Job必定拒绝分配”写成事实。源码同时暴露启动窗口与静默降级；实际分配失败的细分原因未逐次追踪，不靠这一推断才能成立本次失败结论。

快路径读取执行器是现有 link.rs 的精确副本（只将顶部 //! 改为 //，包装暴露私有函数并传1秒参数）；正式60秒证据**直接调用已编译原型库的公开 run_exe**。产品未插入延时/故障注入。探针仅在自己的 helper 进程设置外层Job与文件共享锁，退出时按精确PID/路径终止自身后代并释放锁；没有全局按进程名杀工具。早期150ms探索易受启动负载/旧PID文件影响，最终脚本清空本次PID文件、使用1秒及5秒外层保护，其结果为准。

QA-02使用真实 PE/COFF 成功基线；旧 exe/obj/receipt字节都不变，用户占位目录幸存，确认为**残留与诊断缺口**，没有伪称旧exe被覆盖。复审释放锁后删除自己的 tmp文件。已有常规收据准备失败无锁反例现 staged_exe_left=false，说明此前常规路径已修。

## 验收与门禁对账

| AC | 状态 | 本轮证据 |
|---|---|---|
| AC-01 | pass（限定741） | 独立workspace/依赖；741无root生产改动，区间并行facade变化已单列 |
| AC-02 | pass | text_binding11/11，四设计文本/enum/重复/往返 |
| AC-03 | pass | hir_verify9/9及旧owner/binding反例拒绝、CheckedModule私有构造 |
| AC-04 | pass | descriptor/未知项/无VM回退核查；块图结构门正确 |
| AC-05 | pass | native_execution11/11、实际PE/COFF、one-shot真实build/run |
| AC-06 | pass | overflow=70，trace b→a/12及能力拒绝 |
| AC-07 | partial | 常规CLI/失败不覆盖通过，强资源回收/恢复诊断未完整兑现 |
| AC-08 | partial | 原型绿，主仓daily红且本轮SD-06复核失败，不能总pass |
| AC-09 | pass | bool-local/bool-return旧反例build0，native结果正确 |
| AC-10 | pass | valid swap check0/native9；invalid check1及消费关系拒绝 |
| AC-11 | pass | shared/盗用body正常verify拒绝，不panic |
| AC-12 | pass（原常规场景） | 旧最终收据失败回滚反例与cli全绿；异常清理扩展由AC-18判断 |
| AC-13 | pass（r3归档快照） | README当前check/build命令one-shot13步复现、归档文件存在；旧r1脚本故意调用ac-probe exit2属历史探针，不是当前README缺陷 |
| AC-14 | partial | 所有产品子进程走执行器，大输出/一般挂起通过；硬回收保证有反例 |
| AC-15 | pass | all-targets无warning、fmt；探针最终编译无warning |
| AC-16 | pass | 自环/断开SCC check/build定位拒绝，结构验证先于数据流；合法嵌套/loop/native测试通过 |
| AC-17 | fail | 普通1秒/正式60秒回收测试通过，额外正式60秒及最终快速探针后代存活 |
| AC-18 | fail | 普通故障矩阵绿，真实共享锁导致清理失败未列残留路径/清理错误 |
| AC-19 | partial | archived/r3与归档指针正确；23/24与历史任务收据不一致 |
| AC-20 | partial | 新原型/脚本/check/tv证据充分；daily既有失败和上述两行为缺口阻止最终pass |

- 原型 `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked -- --test-threads=1`：**52 passed +1 ignored**；单独 `--lib -- --ignored`：**1 passed，60.03s**。all-targets check/fmt通过；见 prototype-*.txt/normal-60s.txt。
- `verify-ac-741.ps1 -SkipMainGates`：**13/13**；前两轮复现脚本已新跑，旧bool/binding/owner/block/普通receipt反例翻转，见 one-shot.txt/prior-r*-reproduce.txt。
- root `cargo check -p auto-lang` exit0，**382既有生产warning**（原型零warning）；`cargo tv` **162/162**。
- root `cargo t`首次fail-fast：1298通过、1失败，3672未跑；补 `cargo t --no-fail-fast`：**4971全跑，4935通过、35失败、1超时，1534 skip**。唯一超时ffi_dual_018单独再跑 **1/1通过，2.532s**，符合负载敏感。完整失败清单见 summary-root-t-all.txt；警告/基线族THR-D3/D4及旧阶段报告存在，**没有宣称35红逐个完成同基线归因，也没有把required daily gate写绿**。此次741原型不链接生产crate，不能将root所有旧失败算成741新增缺陷。
- 无tf/ta/taa/t3/docs_gen专项执行；`cargo t`按仓规自然包含docs_gen二进制，未额外启动docs_gen生成专项。
- 所有原型/主仓构建均在独立review worktree；main只写本轮文档。日志入库副本统一LF、去除行尾空白与多余空结尾；root长日志保留完整失败段/结束摘要与原日志hash/大小，重现命令明确；代码/Spec指纹和行为证据入库，清理worktree后可追溯。

## 规范增量核对/遗漏扫描

SD-05：pass，块图包含边可达性+入口/唯一入边足以推出无环，非法结构阻止递归数据流；独立样例/测试支持。
SD-06：**fail/partial**，canonical project.md承诺“任一出口回收自有后代/reader、不以detach换快返回”和真实IO清理错误可定位，与本次反例不符。此前 informational降级记录不是用户批准放宽AC-17。

当前Spec原文冻结在 auto-*-spec-snapshot.md，原始byte-hash在manifest；下一修复的SD-07/08冻结于 [proposed-spec-delta-phase4.md](proposed-spec-delta-phase4.md)，SHA见spec-delta.sha256。未发布canonical增量、不改live ledger。未发现新语言子集/平台/源码adapter的越界实现或未批准延期；无新增产品debug输出，原型零warning/fmt绿。两个P2为既有承诺缺口，不以债务销账当完成。

## 交接

用户此前明确授权：若仍有问题，激活同一741、记录修复方案为新Phase；本次继承该范围。PLAN-741:r4/executing，Phase4 T-25..30、AC-21..24；恢复T-20..24的验收待办，既有实现/历史pass/收据不删除。本轮不实施，不分配新ID。

live ledger现仍指向archive/741，活动文件移动后该指针为**待merge刷新**；复审技能禁止修改live ledger，不能伪称已同步。README及模块plans导航回写active，已有debt中741链接随移动修正；其余历史/ledger指针由T-28和后续merge按生命周期协调。

## 复审检出清理收据

2026-10-05：本机D:/autostack/wt-guard.sh仍缺失（r1..r3既有偏差）；
按既有收据的等价方案，确认精确组路径D:/autostack/.wt/lang-741，
PowerShell全树ReparsePoint扫描clean、auto-lang/auto-down双方git clean后分别git worktree remove，
最后仅删除空组目录（非递归Remove-Item）。detached复审组已移除，没有创建/删除实施分支，未碰742/其他worktree。
关键证据已入本报告目录；下一work需从包含r4合同的v0.6-dev重建plan-741-dev。
