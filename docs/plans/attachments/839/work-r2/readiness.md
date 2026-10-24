# PLAN-839:r1 repair readiness — e44644e1f

**Work pass，execution_done8/10；最终独立复审与merge门pending。** 唯一实施树 `D:/autostack/.wt/lang-839/auto-lang`，分支 `plan-839-dev`，当前提交 **e44644e1fa03c126801caecfe84a8b91054ee09c**，修复前 e58828ca64b7e26ce984faf850fb3b1d0887517f，整体基面37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c。仅6个driver文件改变，HEAD clean，实施写入暂停；SDK/AGP/两锁/生产写面不变。用户已明确授权本修复提交后复审，并授权独立审查代理。

## 原finding闭合

| finding | 不变量与触面 | 修复和永久断言 | 实测与边界 |
|---|---|---|---|
| F-839-R1-01/P2 | 四种窗口取消在Starting期间必须保留，完整初始化后Paused，无自动tick；30s初始化/2s后续、Stop/Fault/新Run、实际Editor/GPU/VM owners不变 | `pause_after_start`只保存意图，状态继续Starting，原30s配置/queries/prepare完整完成后accept直接Paused；Stop/Fault清意图。Host新增2case，inline deadline新增2case | 原审查输入4/4绿，四边沿全部Paused/tick0、后台帧后仍tick0且保原因；正常启动、显式Resume、repeat拒绝/新按键移动、neutral Step、启动中Stop、错profile/prepare失败→owner释放→新Run、3s初始化合法/31s两阶段失败/后续Step3s失败全过。单窗可信源范围，未承诺同步编译抢占。 |

先添加永久Host用例，修复前实际失败 `Running != Paused`：[red状态日志](before-fix-state.log.txt)。初版更早在取消reason为空处失败也保留，未计pass。修复后 [play6+Host7](play-tests.log.txt)、[deadline3](deadlines.log.txt)、[原R1客户端同输入4](original-review-replay.log.txt)通过。没有通过删取消规则、切成2s初始化、放宽期限或skip输出过门。

## 当前验收和实际数量

| AC / V / task | 当前证据 |
|---|---|
| AC01 / V01 / T01,08 | SDK680 / closure479 /17local/29edges及两锁不变；source278全部验证，更新identity raw c8f6a39383036d380fcfbeb6acca41dc33b88efe1cd85c965d9a611e6d25835c，LF d989596db99b7df7ffe4ffe7fd342325555c672225a0002041a6ec948d59e266。[source](source.log.txt)、[结构收据](receipt.json.txt)。 |
| AC02 / V02,08 / T03,05 | 本轮Host7完整指纹/dirty+redo+未存asset/文件sentinel/busy/Close Cancel、停止恢复/Save重开通过；新四边沿每轮原Editor及两canvas hash不变。 |
| AC03 / V03 / T03,04 | 实际Auto模型6、12坏包/改源→GPU、原late scalar query故障对照及play Host7通过；准入/Auto样例/renderer未被修改。 |
| AC04 / V04,05 / T04 | 原600 trace/128/空图/gap/Backlog/step/stale/tickmax和新启动取消、期限兄弟断言全部过。 |
| AC05,06 / V05..08,10 / T04..07 | 原四边沿和重入/恢复原例闭合，producer7原生/consumer7原生全部按命名oracle通过；完整positive27phase、6像素状态、Step实际+1、UI输入/V、blur、Stop恢复和新Run、Save/搬迁独立进程重开。 |
| AC07 / V01,09 / T03,08 | 真实20轮owner/mesh Arc/targets/pipeline/upload不变及最后owner断言通过；保留VM和重传实际故障101/101，恢复0。[owner](owners/results.json.txt)。旧profiles/AGP/资源/金样无diff；完整双游戏矩阵按合同仍merge-owned。 |
| AC08 / V10,11 / T07,08 | SD01/03明确Starting取消完整边界及期限，README/handoff刷新；source/consumer当前验证和准确命令见收据。其他规范路径/GOAL007/008元数据保持；未写canonical/ledger。 |

Producer本轮**39个不同永久定向case、0failed/ignored**：play6+Host7+deadline3+completion4+旧Host7/interaction7/session4/scheduler1。owner重复对照不另加总；原review客户端另4通过。check-all-targets/fmt/diff-check/locked定向bin build通过；SDK继承146 warnings，新增0。未跑完整driver/AGP或root整档。

## 源码副本和原生输入前提

现有 `consumer-work839` 仅刷新5个变化的冻结输入加identity（[精确刷新](consumer-refresh.json.txt)），其余273个源/资产输入、SDK680/两锁及自己target保留；未拷producer产物。当前source/四实际来源反控、all-target check、play13+deadline3、locked bin build与**完整native7**通过。[consumer16](consumer-tests.log.txt)、[期限](consumer-deadlines.log.txt)、[当前native7](consumer-native-pass/results.json.txt)。原源输入是否匹配由完整278/SDK校验保证，不能只用刷新列表代替。

原生失败尝试全部保留，不当pass：producer初次phase24新Run未进入，Stop后出现两新leaf，来源未确证；consumer初次V之前实际cursor-left自动Paused、一次fixture空图导致探针index panic/缺report、另一次phase2新Run未进且增加group。只读监测随后确认实际物理cursor移动及foreground从测试进程切走，观察的Run也正确Paused/tick0：[监测](consumer-native-observed/physical-inputs.json.txt)、[实际状态](consumer-native-observed/report.json.txt)。仅这次监测确证外部输入，不倒推早期额外对象事件的来源。

用户明确提供约30秒不操作桌面的窗口后，同一冻结代码/同一二进制重跑，consumer7全部通过（[idle运行](consumer-native-idle.log.txt)）。没有改OS取消条件、脚本时钟、verifier、像素oracle或被测场景过门；当前非以盲重试替代分析。先前缺报告的尝试照实为失败；受控成功及原有原生交互前提不能被外推为任意桌面同时操作下的自动验收保证。

## Handoff

Work-owned AC01..08通过；开放合同内代码finding、必需覆盖gap、未决决策均为空。T01..08全部完成，execution_done8/10。下一角色为用户授权、实施历史之外的独立审查代理；本readiness不是final review。原R1为internal历史，最终独立复审尚未授pass。

T09 pass后仍需完整merge-owned冻结driver/AGP test+build、16AGP cases、原双游戏/835..838原生正反/partial/reopen、新play/source/owner/fresh-consumer及counter AND content，然后实际autographics-dev合入、规范/ledger/归档、fresh guard清理和batch到期收据。未合入、未沉淀、未归档/清树。保留实施树和源码副本。
