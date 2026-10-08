# PLAN-745 r1 第七轮最终独立复审：pass

日期2026-10-08；review_scope=final，plan_revision=1，review_round=7。
实现HEAD **bb8cb4b4330d505059ea20e7bfe561a93c1051f3**；上一轮522b98a4c27d2064049e188d386f6958721b3a90；原代码基线eea7a80618d4d5e75749d666b7609a9a87925bbd。
入口主检出05d2e1c8464f6d0b2366a040dbd99e29d7eac666，v0.6-dev；唯一登记专属树D:/autostack/.wt/lang-745/auto-lang，branch plan-745-dev；组内只读auto-down依赖fba6563ed2148ce85e68208863159b4ccccac710。
原规划及历轮复审Codex上下文，独立于GLM实施/修复上下文；没有代做产品修复、另建树、委派或合入。Plan读取主检出，代码及测试在原专属树；提交、依赖、配置与Specs稳定，最终双方clean。绑定见[baseline.json](baseline.json)、[final-baseline.json](final-baseline.json)，证据指纹见[artifact-manifest.json](artifact-manifest.json)。

## 上轮三项发现全部清偿

| 发现 | 当前独立验证 | 结论 |
|---|---|---|
| P745-R6-QA-01 二元实参逗号误拒 | M01 positional与M02 named真实native42；M06嵌套192、M07异构bool5、M15末实参15；M17/M18均23/stderr ba；H09恢复111；M14纯括号内逗号仍受控拒绝 | 清偿，无额外括号workaround |
| P745-R6-QA-02 未闭合帧被接受/错误发布 | M03/04/05/08/09/11/12/13与M19/20在原token处text/source.syntax拒绝；M21实际check1/build1，旧exe/obj/receipt三件SHA逐件相同 | 清偿，没有丢调用、Checked或错误发布 |
| P745-R5-QA-02 块嵌套支持边界 | K02的999层native1、K04的1000层check0、H08的1001层check0；新增N17的1000层if真实native1、N18的1500层if-else真实native1、N20的2048层while真实native9；N19的20000层if到Checked，N21/22畸形20000层受控拒绝 | 剩余块子项清偿，原finding整体关闭 |

证据：[parser-results.json](parser-results.json)、[parser-extra-results.json](parser-extra-results.json)、[malformed-sentinel-results.json](malformed-sentinel-results.json)、[resource-results.json](resource-results.json)、[novel-results.json](novel-results.json)、[block-results.json](block-results.json)。输入.at、结果与三制品指纹随附件保存；临时PE/obj清理后仍保留链接收据与逐件SHA。

## 新块帧栈独立检查

本轮修复不只是移除计数护栏：parser的OpenBlock/BlockCont、AST/Resolved Block Drop、resolve词法/循环栈、typecheck、adapter规划/发射与终止分类memo、verify数据流帧、native控制流发射均显式迭代。核对IfThen/IfElse的先后与作用域关闭、父帧索引、初始化分支合流、循环mark截断、最近循环目标、native跳转/封块/返回路径；没有借原生后端补类型或绕过verify。

独立新增N01..N24 **24/24通过**：同级if合流、部分return/双分支return、嵌套循环break/continue、shadow初始化先读旧绑定、分支局部越域/重名拒绝、退出循环后stray目标拒绝、returning-body loop真实native7及其死语句拒绝、未执行分支trace/溢出不提前触发、未执行分支仍检查类型、深合法与畸形树释放、混合loop/if/while状态和分支赋值。既有C/D控制及位置断言亦保持。

永久deep_regression已覆盖999/1000/1001/5000/20000层if、5000层if-else/while、深未闭合块；source_native新增M组与1001层真实PE，source_cli新增未闭合调用三件SENTINEL，source_frontend固化逗号/未闭合正反code-token矩阵。
“无上限”按合同解释为没有人为固定嵌套计数排除，迭代路径保持支持；实测规模如上，不是无限内存或任意规模性能保证。r1语义没有缩域或新增需求，plan_revision保持1。

## 门禁、反控制与限制

| 验证 | 实际结果 | 证据 |
|---|---|---|
| ac-core locked all-targets check/fmt | PASS/PASS，零warning | ac-check/ac-fmt原始日志 |
| ac-core全测试 | **114 passed / 1 ignored**，exit0 | ac-test.stdout/stderr |
| 741一键 -SkipMainGates | **13步通过** | verify741日志 |
| 745一键 | **13步/74例通过** | verify745日志 |
| 独立74源码 | **74/74**；21真实PE/53准确stage-code拒绝 | [corpus-results.json](corpus-results.json)、corpus/** |
| strict ACC inventory | exit0，**47决定新鲜**，9输入/146 unknown族闭环 | inventory日志 |
| 实际当前库Bundle变异 | 有效control通过；同I32越域local实际reverify报uninitialized-read；永久删除let/错type变异亦通过 | [bundle-results.json](bundle-results.json)、ac-test |
| 执行器正常/真实失败 | Rust原样helper4路通过；PS **9/9**，实际约束失败/读错误6及wrapper传播拒绝，所有已观测子进程无幸存，迟到输出与UTF8完整 | [helper-results.json](helper-results.json)、[csharp-control-results.json](csharp-control-results.json) |
| 事务/输出 | 坏source/link/entry与M21旧三件不变；永久真实publish回滚通过；20000 trace完整收集 | [supplemental-results.json](supplemental-results.json)、ac-test |
| 历轮健壮性 | 4096非连续括号正反、100000连续括号、20000调用正反、4096混合调用native4097、长链到Checked保持 | crash/novel/calls/new-results |
| root check | PASS；生产既有382warning | root-check日志 |
| cargo t --no-fail-fast | **4942/4972 passed，30 failed，1534 skipped**，exit100 | root-t日志 |
| cargo tv --no-fail-fast | **158/162 passed，4 failed，6330 skipped**，exit100 | root-tv日志 |

根日常30个失败名全部落在历轮40个历史唯一红名集合，新增名0；tv仍4个历史semver失败。生产crates、根Cargo/lock、.cargo/.config相对745原基线无diff；实验workspace独立，没有根接线。本轮归因依据历史逐名结果及路径隔离，**不等于根门禁全绿、严格同源码main对照或仅靠计数证明零回归**。[daily-attribution.json](daily-attribution.json)保留边界。这些根历史失败不作为745已通过AC的替代证据，也不列为本轮新增合同内finding。

全部9门禁串行重跑，未跑无触面的tf/taa/tt/tb/tu/docs_gen单独档。3个日常测试生成的已核对普通叶文件已清理，未修改生产代码/测试。入口与终审Git只读核验使用提升权限，以避开沙箱路径访问限制，未改Git配置。健康/遗漏/延后/workaround扫描：实验编译与fmt无问题，没有新TODO/调试输出/固定深度护栏；旧限制已由实际代码与正反回放清偿，未以债务登记批准延后。

## AC、规范增量与路由

| AC | 结论 | 任务/证据 |
|---|---|---|
| AC-01 | pass | T01/03/05，数值报告、精确I32/Bool、range/width/char排除与runtime trap |
| AC-02 | pass | T02/03/05，74族支持/拒绝、C/D/M/N及深输入full-consumption控制 |
| AC-03 | pass | T03/04/05，生成Atom→真实bind→全span投影→verify→Checked；真实Bundle变异与741拒绝矩阵 |
| AC-04 | pass | T02/03/05，精确code/token/UTF8/CRLF、合成bool/while及新未闭合token |
| AC-05 | pass | T05，21原始PE及独立M/N native退出码、命名/求值序、branch/loop/recursion/trap |
| AC-06 | pass | T04/05，显式profile、0/1/2、entry/capability/user-shadow、共用finish_build、事务与进程控制 |
| AC-07 | pass | T01/06，SD01..04经核实；strict47新鲜，完整宽度/char仍open |
| AC-08 | pass | T07/08，本轮逐AC独立复验、健康/门禁/遗漏审计及稳定绑定 |
| AC-09 | pending | T09，merge所有权，未执行 |

冻结提案[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md) SHA256 **cd027eae6395f8903f41f1623c20af49d67ec5fdf09631b30f52b8f5485bcdb2**，与当前实现提案一致；四份canonical Specs、四份验证配置、HEAD与依赖无漂移。
SD01新增source profile及受控支持边界；SD02仅本profile的S0接入，HIR三重身份/Schema/唯一Checked入口不扩面；SD03补A1–A3与块支持证据；SD04仅MD408/409追加profile证据，required conclusion/绑定保持，完整int/char open，不重开GOAL-017。元数据精确目标路径已就绪，touched_goals=[]有范围说明。提案review定稿，canonical与live ledger留T09。

下一步可进入 **T-09 /auto-plan:merge**；本轮只完成最终复审，状态 **reviewed 8/9**，revision1，T01..08完成、T09待完成。入口frontmatter/文字称7/9而任务表仍留四个修复项空勾，本轮依据实际验收恢复T02/05/06/07并完成T08，统一计数；不虚构先前复审通过。
merge仍须授权并实际执行：v0.6-dev整合、SD/MD注释与消费者新鲜度复验、Specs/ledger/index/导航、当日verify745重跑及批回归到期判定、归档9/9、wt-guard clean后的原树/分支/空组清理。没有预勾这些closeout步骤，历史债务与每轮发现记录保留。
