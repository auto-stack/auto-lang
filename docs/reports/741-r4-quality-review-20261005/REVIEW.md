# PLAN-741 r4 合入后独立复审（2026-10-05）

**needs_fix**。plan_revision4；reviewed_commit `7b9c6948c1465e87a6f60404510c30c712e1c7bc`，base `c865adf66a75099df07113b053eb9d19e92986a3`。实施cc491052f/d43bfcd33、review6fb7a9068、SD沉淀8fdae8c11、archive5690f205c/receipt7b9c6948c均在祖先链。本会话未参与r4实施；重新读实际代码/合同/Specs、独立detached专用检出、自构公开API反例，不依赖执行者pass。

## 本轮真实通过

- 原型53项通过，首次trace因未构建前置test-support库失败；按README构建该库后单独重跑trace1/1通过，合计**54项通过**、1个60秒证据测试默认ignored。没有将首次失败抹掉，见[初跑](prototype-initial.txt)、[trace修正前置后](trace-after-prerequisite.txt)。
- 一键`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-741.ps1 -SkipMainGates` **13/13**，覆盖全族、fmt、all-targets零warning、真实PE/COFF和trace=b,a/12，见[one-shot](one-shot.txt)、[all-targets](all-targets.txt)。使用独立TEMP CARGO_TARGET_DIR；trace支持库仅在专用worktree的ignored target构建，无产品修改。
- **公开run_exe 60秒路径**：60078ms返回link.deadline，持pipe后代在5秒death-watch内确认不存在（首个tasklist查询约35ms）。见[public-deadline](public-deadline.txt)。不把返回瞬间的异步终止快照当作逃逸；旧R3-QA01问题已修复。本次不声称所有OS故障注入都有新证据。
- 原publish共享锁清理、约束失败拒绝等lib11/11绿；bool、bindings、owner、块图及原生执行既有回归绿。原型/工具健康通过，未改生产crates/根workspace。
- 本轮是重复复审：root crates/、Cargo.toml/lock、.cargo/.config和一键脚本相对r4基线零diff；重用明确绑定的本日phase4 check/t/tv及双侧daily失败证据（tv162/162，worktree34红为同源基线36红子集）。**没有新跑或宣称daily全绿**，理由见[root-gate-reuse](root-gate-reuse.txt)、[已归档本轮门禁](../741-phase4-resource-fixes/verification.md)。独立原型发生变化的所有测试/构建和API行为重新运行；不额外跑tf/taa/t3/docs_gen。

## P741-R4-QA-01 [P2] link_object_staged仍吞清理失败

`experimental/ac-core/src/link.rs:519–525`两个错误出口仍`let _ = remove_file(&tmp_exe)`。Phase4详细设计§5.7/T27明确包括link_object_staged及所有清理出口；canonical auto-ac/project.md的SD08承诺实际错误、残留路径、提交状态。此前review将其列为“既有、近不可达 informational”，没有用户批准免除该验收。

公开API反例：[helper源码](link-cleanup-probe.rs)调用当前已编译库`link::link_object_staged`。真实CLI先生成旧PE/COFF/receipt（exe=5）；stand-in linker自行创建部分输出，观察线程取得普通FILE_SHARE_READ|WRITE且不含DELETE的文件句柄，以marker握手后linker非零退出或等待生产60秒截止。无产品故障注入、未改库、没有预造他人文件冒充owned输出。同步真实共享锁模拟AV/扫描器等占用，仅证明该合法IO状态可达，不声称经真实rust-lld复现。

| 路径 | 实测 | 诊断缺口 |
|---|---|---|
| linker exit42 | 101ms、link.failed | staged_exe_left=true，实际remove OS error32，path_in_diagnostic=false、cleanup_in_diagnostic=false |
| executor deadline60s | 60009ms、link.deadline | 同上；只有原超时错误，暂存exe仍在且无清理诊断 |

[非零出口](link-nonzero.txt)、[超时出口](link-timeout.txt)：三件套字节都保持；旧exe仍exit5；释放句柄后清理成功。**不是失败覆盖旧exe，也不是公开执行器仍泄漏后代**；是明确承诺的清理诊断遗漏。

修复：这两个出口收集remove真实错误，和原link.failed/deadline一起返回含完整路径/OS错误/NOT committed/恢复办法的诊断。NotFound已回收、用户目录不删；与publish路径保持一致，不能吞错或改弱SD08。新增永久非零/执行器错误正负矩阵，普通可回收出口零残留、锁受阻可定位，释放后可恢复；真实旧制品保留。

## P741-R4-QA-02 [P3] 真实归档四处历史报告链接失效

归档Plan行44/168/761/1025仍使用`../reports/...`，从archive目录解析不存在，应为`../../reports/...`。见[audit](audit.json)：8个跨Plan/README/模块导航链接中4处断链。README/两个模块导航和全部P741-* ledger指针正确；unique任务30/30也正确，旧计数缺口已修复，不重复报错。

激活移到plans目录后四个链接暂时恢复，不代表最终归档修复。修复应有实际active/模拟archive和**最终真实archive**链接门（围栏感知）、去重任务ID计数、metadata/README/模块/全部P741历史ledger指针断言，禁止只查文件存在和计数。

## 24项验收与delta对账

| AC | 结论/本轮依据 |
|---|---|
| 01 | pass：独立workspace，受审区间root生产面/配置/脚本零diff |
| 02–04 | pass：text_binding11、hir_verify9、descriptor/unchecked边界检查 |
| 05–06 | pass：native_execution11、trace1及一键真实PE/overflow/顺序 |
| 07 | partial：普通失败保旧制品通过；QA01异常清理诊断不足 |
| 08 | partial：原型与门禁证据真实，但SD08未全兑现 |
| 09–11 | pass：bool、参数映射、body owner既有回归在native/cli族覆盖 |
| 12 | pass（旧常规receipt场景）：cli11/lib11，无扩充结论替代AC22 |
| 13 | pass：README当前check/build路径和13步运行正确，README Plan指针存在 |
| 14–15 | pass：lib截止/大输出等、all-targets无warning/fmt、一键 |
| 16 | pass：旧非法块图/合法loop等hir_verify/native覆盖 |
| 17 | pass（已实测路径）：普通截止/collection/lib、公开60s与death-watch，未复现旧泄漏 |
| 18 | partial：正常事务matrix/恢复诊断已修；pre-publish link出口QA01 |
| 19 | partial：真实30/30、归档/ledger正确；4个历史链接断 |
| 20 | partial：新原型证据和root基线归因保留；最终needs_fix不由历史pass覆盖 |
| 21 | 已覆盖行为pass：受控启动/失败拒绝及公开截止回收成立；未知OS故障场景未额外假写新通过 |
| 22 | fail/partial：原publish锁反例通过；明确要求link_object_staged的两个出口仍吞错 |
| 23 | partial：30/30和指针符合，历史报告链接失败QA02 |
| 24 | partial：健康/原型证据绿、daily旧红单列；SD08及最终链接阻止最终pass |

SD07受控启动/约束失败边界符合观察；SD08 **partial**：publish/restore汇总符合，link_object_staged不符合。冻结[受审SD07/08](reviewed-spec-delta-phase4.md)与[SD09修复提案](proposed-spec-delta-phase5.md)，[hash](delta-hashes.json)。没有新语言能力、延期、debug输出或warning；不能把既有遗漏降为informational。源/Spec/计划hash见[source-manifest](source-manifest.json)。

## 用户授权下的交接

继承用户明确授权，同一[741](../../plans/741-ac-hir-native-core.md)追加r5 Phase5，不取新号、不实现Rust修复、不发布canonical/ledger。T21..24及T27..30重新待验（重复任务行同ID一起重开），其余完成任务保留，当前22/35；新增T31..35、AC25..27、SD09。旧r1..r4记录/销账保留历史，新发现单列债项；派生ledger旧archive指针暂缺在r5 merge恢复，不能宣称已刷新。

复现helper从当前库链接；rustc需要`-L dependency=<prototype target/debug/deps>`、`--extern auto_ac_prototype=<rlib>`及`-L native=<windows_x86_64_msvc-0.52.6/lib>`。CLI先build add-2-3到<scratch>/baseline/old.exe，然后helper `nonzero|timeout <scratch-case> <baseline-dir>`；helper `public-deadline <scratch-dir>`走正式run_exe。所有case目录隔离，结果固定marker握手，释放自己的锁并删除自己的暂存输出；不全局杀进程。完整命令见[reproduce.ps1](reproduce.ps1)。

## 复审检出清理收据

本轮detached D:/autostack/.wt/lang-741/auto-lang与空组已移除，没有创建/删除实施分支；742/743/ABI未动。wt-guard.sh本机缺失，固定Git bash路径不可执行；删除前已验证精确绝对目标、全组ReparsePoint扫描clean和git clean，使用git worktree remove后仅非递归删除空组，沿用r1..r4等价扫描收据。见[cleanup receipt](cleanup-receipt.json)。测试支持库ignored产物随专用树清理，证据已持久保存。
