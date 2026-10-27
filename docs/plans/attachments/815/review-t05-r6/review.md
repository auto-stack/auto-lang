# PLAN-815 T-05 R6 / T-06 / T-08 定向独立复审（2026-10-22）

**needs_fix；executing10/17。** R5四项中R1-01（Unix）、R4-04（配置单源）、R5-01（旧入口语义）CLOSED；R4-05的取消/EOF挂起与假completed原例已闭，但非取消的捕获失败传播仍PARTIAL/OPEN，按当前剩余影响降P2。新增F-815-T05-R6-01/P2：PM验收矩阵误拒合法bun回退。当前两项，不新增验收范围，非T-09完整B终审。

## 基线与范围

- code **c6fdaaaf1ecc2799d5ad225db3922a121c90c383**；handoff HEAD **bb4f0a1caeb252ccb82d722659ee827703b82fc1**，其后相对code无生产差异；R5基面10e4e1cc1a840132513317424afa7bba4dec5262。唯一树`D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`，开始clean；[初始输入](baseline.json.txt)/[终核](final-baseline.json.txt)绑定代码、锁、配置、Plan r3、当前Specs与M814-B已发布祖先93a25c924d1beafab130588bad4d1041dc86e48c。
- reviewer=Codex `/root`，延续独立审查上下文，有早期合同/账本参与限制，未参与实施；使用`/auto-plan:review`从原始输入/生产状态重建，不新建agent或第二树。autoui-verifier仅复用原有MCP客户端state/snapshot，未作双端视觉声明。
- 七个净改Rust文件全部读审与格式检查。实际重建bin SHA256 **c95748c80b512215d89d6c987d3b8b708f697d102a24a62333f7666051c42b61**，与交接一致。[修复原文](frozen-fixes.md)、[handoff](frozen-handoff.md)、[Unix实施证据](frozen-unix-evidence.md)、[当期SD核验](spec-delta-reviewed.md)冻结；未改canonical/live ledger。

## 四项复审关闭表

| Finding | R6裁定 | 独立证据与边界 |
|---|---|---|
| R1-01 / P2 Unix验收 | CLOSED | 本机WSL Ubuntu登录shell可见rustc1.97.1/cargo；对当前冻结树实际`cargo test --locked --offline -p auto-ui-tools --lib --target-dir /home/visus/plan815-unix-target -- proc_host::tests::unix_reader proc_host::tests::captured_drain proc_host::tests::vm_session`，离线编译24.64s、四项4/4（0.84s）。持管道真停/Stopped+自然EOF、捕获取消/自然EOF、每请求env与fatal捕获均通过，原日志unix-independent.log.txt。不声称Linux图形/全库通过。 |
| R4-04 / P2 主pac第二真源 | CLOSED（原不变量修复） | 原package-source两例：wire npm+disk npm/pnpm均实际执行npm；真实无字段请求、disk npm、私有仅bun回退环境实际执行bun，证明Some(None)分支未走legacy磁盘。三入口guard设置/清理、工具侧不走CACHED_PM，源码已读；新测试误拒另编号R6-01。 |
| R5-01 / P2 旧公开入口语义 | CLOSED | 独立legacy-build探针真实到Jet exit7，REVIEW_LEGACY_RESULT:Ok(())；新legacy_build_semantics Jet/Ark两例2/2；协议原exit7与exit0无APK仍具名failed。Jet/Ark/VSCode strict仅托管形态，旧/新结果边界源码一致。 |
| R4-05 / 原P1 捕获生命周期 | 原例CLOSED；完整不变量PARTIAL/OPEN，剩余P2 | 私有非owned holder持管道时cancel后约290ms返回cancelled，holder释放前宿主EOF exit0，原无限等待/释放后completed已闭。新超时/截断可停返回，但调用方丢弃结束原因，最终completed/error=None，详下文。 |

R4其余已闭目录、Multi、VM上下文/缺DLL与卫生边界按当前重放保持；本轮没有将原正确关闭项回滚。T-07完整发行/资源与T-09最终AC仍未完成，故无整Plan pass。

## 当前两项

### F-815-T05-R4-05 / P2 — 捕获失败终态被吞成“没有设备”，最终completed

**AC-03/V-03，T-05/06；原reader生命周期/具名失败完整不变量继续开放。** 不是再指称cancel挂起；新增五分类不等于消费者已传播分类。R5报告关闭oracle明确要求读取错误不得当完整结果、真停止或有界具名失败，未新增规模承诺。

- 重放`adb-drain-exits.py`，复用原`adb-held-pipe.py`的真实generate→run_ui→Jet Gradle→owned ADB→caller drain入口。只有Gradle/ADB是私有受控OS负控，不作为真实Android成功证明。自然EOF正控及原取消对照在adb-held-pipe.json.txt。
- 超时例：ADB将stdout写端DuplicateHandle给本次私有非owned holder、自己正常退出，dup=true/非零handle明确到达；不发cancel。**约10.6s得到RunOk.completed，error=None，holder仍活。** 源码10s界限已触发，但无具名超时/截断收据。
- 截断例：ADB输出约1.1MiB（超过新增1MiB cap）后退出；**约346ms得到RunOk.completed/error=None**。受控程序可能因读端关闭触发BrokenPipe/非零退出，原始stderr保留；无论该进程退出码，捕获已非完整Eof，不能把原因抹成“无设备”。两例完整回执/帧/fixture和stderr见[adb-drain-exits.json.txt](adb-drain-exits.json.txt)。
- 原因已确认：proc_host::drain_captured_stdout确实返回BoundOverrun/Truncated等分类；jet.rs:952-969在wait后只检查cancel，随后把`matches!(end,Eof) && exited_ok && ...`变成bool，异常分类/等待错误均丢失，has_device=false后打印手工指引并Ok，map_flow无取消时自然completed。并非推测helper没有返回，也不是要求杀非owned holder。
- 关闭：非Eof捕获结束、读取错误、异常等待/子进程退出应按已约定合同得到结构化具名失败/准确残留说明；自然EOF完整空列表可保持合法“无设备”语义，取消仍cancelled。对受影响owned树收尾/重用下一请求与五分类各自代表对照补齐。不能仅在私有enum里命名、客户端却看到completed。若界限触发后owned ADB本身还活，后续wait也须有实际收尾边界（本轮两例ADB已退出，不声称实测了该sibling）。

### F-815-T05-R6-01 / P2 — 新PM验收矩阵误拒支持的bun自动回退

**AC-02/V-02、验收oracle有效性，T-05/06；本轮新增测试回归。** 产品在该配置下正常，本项针对用来证明单源不变量的断言。不得通过删除bun支持或改变产品正确回退来让测试变绿。

- 重放`pm-fallback-environment.py`。先定位当前冻结包测试binary，独立进程只跑`pkg::tests::tool_pm_state_matrix`：继承环境exit0为正控；另一个私有PATH包含bun.cmd而无pnpm、私有空USERPROFILE/HOME、不写用户全局配置。该环境属于代码明确支持的global/detect回退，未增加产品平台要求。
- 实测：合法bun回退环境同一具名测试exit101，pkg.rs:472断言`resolved != Bun`失败（Bun==Bun）；继承环境同一binary通过。真实工具同环境、**wire无pkg、disk pkg=npm实际选择bun**，证明产品已正确跳过磁盘，不能把这当成R4-04重新失败。[完整双环境测试和真实工具对照](pm-fallback-environment.json.txt)。
- 原因已确认：测试将磁盘pkg设bun，无字段请求允许外部链也合法解析为bun，却按“解析值不相等”判断来源。之后的`if !command_exists("pnpm")`注释/print在assert之后，无法豁免合法bun；来源不同可以得到相同值。原单源规则已满足，这个assert本身不是正确oracle。
- 关闭：固定/隔离测试自身外部来源，用相同外部环境下改变/移除主pac、或明确观察执行来源的正反控证明磁盘独立性；同时保留bun合法回退、pnpm优先、显式请求/缺字段、串行guard恢复与错误退出。断言失败时cwd/状态也要可靠恢复，不能污染同进程后续用例；具体测试编排由work处理。复跑当前有效两个环境与真实工具正控，不把环境差异泛称flake。
- 探针首版对SystemRoot环境变量键大小写作了错误假设，在设置隔离环境前KeyError退出；修正后才得到上述有效exit101与真实工具bun结果。首次布置错误不计产品finding，也不替代有效结果。

## 验证账、规范与后续

- Windows根定向**79/79（1既有DirectChildOnly负控LEAK）**，额外PM矩阵/两个legacy/五个实际生成取消VM用例**8/8**，Man兼容/媒体/请求缝**6/6**、独立consumer **5/5**：98个不同Rust用例在继承环境通过。另Unix **4/4**，按平台单列，不将跨平台重复ID算新独立测试。合法bun环境的PM矩阵则明确1红，不能宣称全部验收绿。
- 七个冻结Rust格式全部相等，代码diff-check通过；runtime235/ui-tools22既有诊断基线保持。真实二进制与T-08指纹一致。R4/R5原例通过不覆盖本轮两项新输入；helper只作生命周期/失败负控，真实SDK ABI与标准MCP state/snapshot功能正控保持。
- 旧verify-targeted脚本内fmt写死历史SHA，其首轮fmt.json.txt只作历史重放产物，当前格式以按bb4f0a1ca完整SHA运行的fmt-current.json.txt为准；可复用脚本已改到R6范围。原始日志/命令保留，未借错基线制造格式finding。
- Unix实施收据此前称“无工具链”实际是非登录PATH与登录shell差异；本轮登录shell直接复跑成功，证据不足旧结论已关闭。实施者另披露旧real_tree测试Windows命令导致Linux可移植性问题；本轮未将其跑成通过，也不因“旧测试”自动批准延期。完整T-07/Linux覆盖须对账，四项定向pass只覆盖其明确的reader/capture/env/fatal边界。
- T-08源/bin、取消双终态、单源PM/legacy/Unix正文原缺口均有进展；客户端仍缺非取消捕获失败披露，新PM验收oracle仍红，正文需据修复再对齐。当前T-08准备稿partial，依赖T-07的完整777/792并集/fresh/locked/资源/来源审计与T-09最终发布性核验不在本轮完成范围。见[逐SD核验](spec-delta-reviewed.md)。
- 未整档t/tv/tt/tb/tu/taa/tf/docs_gen；未修实施代码、改canonical/live ledger、合入、归档或清树。A/L与T-04及未变既闭结论保留。Plan r3 executing10/17，T-05/T-06/T-08继续未勾；原唯一树work修齐两项并补代表分类/合法环境对照，提交新SHA再审。最终pass后仍有merge-owned冻结裸t+实际触面档/并集与真实发布收口，不提前放行M815-B消费。
