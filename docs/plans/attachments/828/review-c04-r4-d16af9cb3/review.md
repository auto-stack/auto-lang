# PLAN-828 C04窄复审：原两例通过，接受字节的失败出口仍漏计

2026-10-22；review_scope=phase（仅C04两子关闭行及受影响相邻出口）。冻结 **d16af9cb3956921c523074ec0a60316a7b79d8a0**；结论 **needs_fix，仅B3-C04/P2的“已接受字节在早退时丢失”残项**。握手子项与成功路径903B重复计量原例均已关闭。C03及其他旧闭合保持，不要求重做机制。

## 身份与范围

- 实施树D:/autostack/.wt/lang-828/auto-lang、plan-828-dev；base=d5a4f3147e336095e0e9d9e6207b67aea1c51d1c；主检出交接18b06c182d6138f080521b958ee062529fce4fdf。
- 入口/结束clean、HEAD未变；baseline/final-state的520项源码/锁配置/Specs/Design43/冻结主Plan接口/技能一致。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。三交付文档与delta冻结并纳入evidence-index。
- 本Codex审核上下文未实施、未启代理，不编造未暴露身份。未改WT实现、canonical/ledger、合入/整档；只写主检出审核附件与簿记。
- 本轮窄修只改formal_worker握手、session计量、coverage数量。执行器/同源publication/身份门未变；仍用当前库重放C03代表性原控，未因旧PASS自动覆盖新代码。

## 已关闭的两个原例

**A握手收据 CLOSED**：send/recv/解析进入同一Result，错误或错误类型设置handshake_failed、跳过编辑、仍返回身份/init/原因/零样本报告。原handshake-controls **4/4**：EOF、坏JSON、意外消息类型均可保存红report；编辑第二次坏JSON保留第一样本。四个实际报告已保存本目录handshake-*。发送错误和接收错误共用同一Err出口，发送分支经源码核对；不冒称4控已实际触发发送错误。spawn失败仍Err的边界保持。

**B成功路径重复计量原例 CLOSED**：新四个成功put的payload173+252+72+406=**903B**，上报也**903B**；no-store结构零盘IO正控保持，原io-controls **2/2**。接受口径写明Ok（含幂等重发布）一次一份，Conflict/失败本次put不计，五处写入口已统一。

但上一轮关闭条件还明确要求：[R3报告](../review-batch3-r3-d5a4f3147/review.md)“相关失败出口不丢已发生计量”，[任务单](../review-batch3-r3-d5a4f3147/glm-next-round.md)“保持已发生计量”。本轮同类早退仍不满足，故完整B子项未关闭；不是新增功能或提高预算。

## 唯一残项：B3-C04/P2——后续put失败会把前面Ok的字节报成0

归属T-07/T-08观测、AC-07/08、原§5.5真实IO与失败观测。**仅证据计量错误，不声称本例程序值或实际store行损坏。**

session.rs将本次已接受字节累积在局部`store_write_bytes`（1512行），只在persist_all_units成功走到末尾（1804行）汇入`self.phase_io`。在此之前，HIR（1654行）、sidecar（1697行）和repair等错误出口直接return Err。先前put已真实返回Ok、逻辑字节按新定义已被接受，但局部计量随早退丢弃。manifest在persist_all_units之后处理，故其失败反而保留了前面的计量。

### 同一新冻结上的区分控制

`io-failure-controls.rs`使用完整冻结session源码投影，**生产函数体tokens不改**；与原IO控制同一个cache旁路记录器仍只委托一次真实put并记录payload长度/Ok状态。仅在指定第n次put的审核边界加入一个不存在的RequiredData目标，使真实ManagedStore返回TargetMissing；之前的put真实成功。该注入代表put返回Err的受控故障，不声称正常sidecar自己会生成坏边，也不要求放宽store准入。

每个fixture是独立fresh临时store；预热后清空审核记录，真实helper7→8、内存编辑ok、persist_error=1。四个出口实际结果：

| 失败位置 | 之前实际Ok的逻辑字节 | 上报字节 | 判据 |
|---|---:|---:|---|
| 第1次ABC失败 | 0 | 0 | 正控通过 |
| 第2次HIR失败 | **173**（ABC） | **0** | 失败 |
| 第3次sidecar失败 | **425**（ABC+HIR） | **0** | 失败 |
| 第4次manifest失败 | 497（ABC+HIR+sidecar） | 497 | 正控通过 |

io-failure-controls.log **2过2败、退出101**，记录实际TargetMissing错误、全部调用长度/接受状态及persist_error。源、限定注入方法与hash见generate-io-failure.py / io-failure-source.json / session-io-projection.rs。审核投影的unused/dead-code告警来自仅使用部分session API，不归生产新增告警。

两次失败不是两个新finding，而是同一个“成功末尾才结账”的共因。错误put本身应计0，但不能抹掉早先Ok的173/425B。不能通过将整个失败编辑都定义成0来放宽已冻结的接受口径。

### 关闭oracle及最小修正边界

在任何可能早退前保留此前已接受字节，或用统一退出结账机制；覆盖ABC/HIR/sidecar主行/repair的同类错误出口，不只给本次反例加特殊分支。保持每个Ok一次一份、失败/Conflict本次put零计。上述四个出口全绿，原903/no-store两控保持，握手4控与C03代表性控制保持即可。不要改store身份/根回收、执行器、publication或预算。

## 本次验证

| 定向面 | 实际结果 |
|---|---|
| 原handshake / 原IO | 4/4、2/2 |
| 新部分成功后失败IO | 2/4（唯一阻塞） |
| C03原judge / consumer-copy / identity / hit | 6/6、2/2、3/3、2/2 |
| 新执行器trace四入口及旧pin | 1/1，原C03证据保持 |
| 官方T08（含正式30轮） | 5/5；成功raw/report在formal-official |
| 原wrongvalue失败落盘控制 | 1/1，报告/raw先落盘后门断言，文件在formal-failed-* |
| permanent mig/t07/b2/b1 | 9/9、8/8、14/14、11/11 |
| session/contract/t01r/r1 | 18/18（显式skip三512）、22/22、4/4、86/86 |
| close继承控制 | 96/96，include链与r1有重叠，不加总宣传 |
| 两改动Rust fmt / diff-check | 0/0，生产构建零告警 |

正式30样本本次P50/P95/P99=226.9941/300.5848/312.3631ms，init1305.374ms；worker峰值210898944B、父侧22396928B；原798同口径双线预算硬断言通过（日志A=201、B=0整数MiB）。这不能覆盖store_attached=true的计量失败出口，正式战役无store的数值绿不能替其关闭。

本轮不再运行512或整档，不把work的303.8s当本次独立结果。未改变且已闭的core/存储机制证据在明确输入边界延续；长负载/整份AC09/10仍按T-10对最终冻结核验。SD01/原C05接缝更正保持；SD05/06“真实IO/完整失败证据”的整体完成声明仍受此C04残项限制，全部仍proposed，不沉淀canonical。coverage_t08=5校准通过。

## 复现与下一步

在冻结WT按cargo-artifacts所列定向目标重建后，本目录执行：

```powershell
python ./generate-io-control.py
powershell -NoProfile -File ./run-controls.ps1 -Source io-controls.rs -Filter measurement_ -AllLibraries
python ./generate-io-failure.py
powershell -NoProfile -File ./run-controls.ps1 -Source io-failure-controls.rs -Filter accepted_io_ -AllLibraries
```

首组退出0，第二组当前退出101。实际库/exe身份、输入、store错误、原始日志完整保存，未借实施者TEMP验证台结论。

下一责任work **只修接受字节在早退时的保留**，新冻结后再作此单项窄复审；通过后T-10。握手和C03不再重开。Plan保持r3/executing0/11；整份最终pass后的merge-owned裸t+tv/必要触面consumer、tf到期主检单实例、Spec沉淀/归档/guard均pending；本轮不授权合入。
