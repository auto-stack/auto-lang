# PLAN-828 第二批 R3 修复复审

2026-10-22；review_scope=phase；**needs_fix：上轮三项已闭合，新增一项 P2 持久化回归 T07-R3-01**。不重新打开已通过的来源投影/目录校验/required根保活机制，也不授整份reviewed/merge。

## 冻结身份

- 原实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@3090f2326f3595a69132c80bee3a2891c5f41b76`，入口及最后核查clean，HEAD未变，518项源码/配置/规范/接口/技能输入结束匹配。
- review diff base `cfa67eeceb4555e55b978354d3fd7ab508e80664`，包括已冻结T-08脚手架`8be89bf5d208f6f36f65aa22b79eea0a9a5953ff`及本次修复；修复直接基面8be89bf5d。未reset或排除并发改动。
- Plan r3/executing0/11，主Plan与interface冻结为`plan-input.md`、`interface-input.md`；baseline/final-state记录Specs、Design43及依赖/配置hash；auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，probe未消费。最终`matches_baseline=true`。
- reviewer为本Codex审核上下文，未实施修复、未启新agent，未编造模型/会话身份。独立读生产机制、重放旧输入并补相邻路径；没有改实施源码、canonical或ledger。
- 范围是R2三项修复与受影响恢复/执行路径；T08冻结脚手架3项仅兼容重放，正式worker/IPC全矩阵及性能/双线内存仍归后续，不据此授T08 PASS。

## 三项关闭

| Finding | 机制与本轮执行证据 | 结论 |
|---|---|---|
| B2-C03/P2残项 | 删除payload裸func号全局表，在真正发生trap的帧执行槽经ModuleView/UnitView投影。原两个同slot=1的真实单元现在helper来源159..164；R1重排/真实编辑例继续绿；二元溢出和一元负号溢出分别159..173、185..190，普通/units别名/steps→resume均只映射一次；旧pin/fuel/trap退出和不可变身份保持 | CLOSED，当前scalar所有span-bearing trap出口已检查 |
| T07-R2-01/P2 | format==1、行必解析、无重复、目录集合精确相等；原重复掩盖缺项/未知format999均拒；永久合法重排正控通过，matched/absent/rejected自愈保持 | CLOSED，当前v1目录语义边界 |
| T07-R2-02/P2 | ABC/sidecar store-internal，manifest RequiredData edges保护实际幸存primary/repair行；先注册新根再退役superseded；原退役soft根后真实GC仍warm admitted3；旧载荷回收/用户独立根字节不变永久控制通过 | 原required图/根保活与回收输入CLOSED；新增旧行身份升级回归单列下项 |

原R1 link8、execution4、R2 projection1/T07控件5/lifetime2全部**原样**重放通过。新增trap控件1 test含两种trap与三入口通过。原owner/调用闭包/步数预算及第一批关闭保持。

## T07-R3-01 / P2：新增required边改变artifact身份，却未隔离旧manifest键

**原规则**：T-07/F-11/V-07恢复与冷回退/自愈，C02-04/AC05既有共享存储身份不可覆盖或吞Conflict；兼容/错版本可受控miss后回冷，不能因本修复让合法旧行阻断新持久化。R2任务单明确要求保留partial旧store和自愈路径。此项是**本次required边修复引入的回归**，不是新增缓存语言或强制旧工件免验证采纳。

**定位**：`experimental/ac-av-probe/probe/src/session.rs:1621`起，manifest key仍为旧producer/owner/kind=session-manifest/**format=1**/entry/**fp(body)**；1639起写入新required_edges。shared store的`store/mod.rs:531`将edge集合计入artifact_id，且同layer/key只能对应同一artifact身份，合法返回Conflict。该冲突不是环境/SQLite故障。

**重放输入**：`migration-controls.rs`。使用实际生成的合法manifest body/envelope，按cfa源代码原配方写入**同format、同key、同body、空edges**的旧row并注册原root；完整闭包控制还复制真实合法ABC/sidecar原payload，用旧producer的无边＋Some(project) soft根方式写入。全程公共API，无改DB、无伪造opaque凭证；内容版本从公开manifest读，不碰私有字段。fixture复刻旧producer的确切配方，**未声称运行了旧二进制**。`legacy-producer-manifest.txt`及reviewed.diff保留配方对照。

**期望**：旧行可继续只读重验使用，或按新键/版本隔离后冷回退、自愈；新required闭包发布成功，已有旧行不覆盖且不吞冲突。新缓存正常重开/同内容编辑仍保持。

**实际，4个失败出口对应一个根因**：

1. 旧manifest存在但所需单元缺失的partial store：`restore`到cold fallback后尝试persist，返回 **Err("persist manifest: ... Conflict")**，不能自愈。
2. 同样旧row上直接`warmup_persisted`也返回相同Err。
3. **完整旧ABC/sidecar闭包**先经`restore`真实通过、`restore_units_admitted=3`（证明fixture是可读的合法旧缓存），随后重开`warmup_persisted`仍Conflict。
4. 从完整旧闭包warm restore后，真实在helper的`7`前插3空格，`edit.ok=true`、`reuse_hit:helper=1`；持久化却 **persist_error=1**，未发布新required图。运行安装未回滚，不误报源码/运行结果被损坏。

固定例key为`A09[entry=d_main,format=1,fp=2933d5f4914d8d43,kind=session-manifest,owner=m_src,producer=plan798/av-probe]`，旧artifact=`art58dcbe1e4ff484e0e734520c7891ea685940bd1249b7e02041a06ace95bf0afd`，新required图尝试=`art4c26cd9aeec62b01b5356aab66b425089540d12a033b031d2432743131f78031`。日志保留原始错误。

**合法相邻控制**：新修复生成的required manifest重开与同内容编辑全部正常，persist_error=0；新store原五个GC/格式控制全绿。故不是普遍不支持空白编辑、ABC非法、缺少函数或普通幂等重开失败。

**严重度/边界**：P2，阻断已有缓存上的持久化构造/冷自愈，或使成功编辑长期丢持久化能力；已有ABC/HIR重验和内存安装语义未失守，不报P1。无需回滚已通过的required边/源映射/严格目录机制。

**关闭oracle**：保持新图真required根与root发布顺序，明确新的持久身份/版本策略，使同body但不同required闭包不再占用同旧key而冲突。必须覆盖完整旧闭包、partial旧闭包冷fallback、warm读后同内容编辑、新缓存幂等重开、随后真实GC与再恢复；旧row不覆盖、不吞Conflict、不关闭持久化掩盖。如果采用换键/格式，读旧→冷→写新→再恢复需有永久断言，不能只在全新临时目录验证。

## 执行结果与复现

构建（原WT）：

```powershell
cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_b2 --test function_incremental_828_t07 --test function_incremental_828_t08 --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json
```

在本证据目录：

```powershell
powershell -NoProfile -File ./run-targets.ps1
powershell -NoProfile -File ./run-controls.ps1 -Source migration-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source projection-controls.rs
powershell -NoProfile -File ./run-controls.ps1 -Source t07-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source trap-controls.rs
```

运行器按本次Cargo artifact选择精确库，目标二进制/hash/args/exit见target-results；不是用work旧日志授PASS。migration **1pass/4fail，exit101**；其余控件exit0。

| 定向选择 | 本轮结果 |
|---|---|
| b2 / t07 / b1 / session | 14/14、8/8、11/11、18/18（显式skip两个512） |
| contract / t01r / r1 | 22/22、4/4、86/86 |
| 冻结T08脚手架兼容 | 3/3，非正式测量/IPC全验收 |
| 原link / execution / projection / T07 / lifetime | 8/8、4/4、1/1、5/5、2/2 |
| 新trap出口组合 | 1/1（binary＋unary/三入口） |
| 旧sidecar / generation / binding / close / close-adjacent | 6/6、8/8、6/6、6/6、3/3 |
| core lib / 真commit失败组件 / cache_tier | 22/22、1/1、20/20 |
| migration原型至最终 | 最终1pass/4fail；初次误用私有resident_content_version未编译，已改公开manifest读取，装配日志单列不作产品finding |
| 九文件fmt / diff-check / 最终freeze | 0 / 0 / 0，入口结束clean且518输入一致 |

集合重叠不汇总为唯一测试数；两项512负载未跑，本轮不授完整T07旗舰/资源验收。没有运行整档t/tv/tt/tb/tu/taa/tf、docs_gen或merge。告警：shim未用文档注释和probe SpanInfo为既有；冻结T08脚手架的CloseOp/InitOp未用导入仍待原T09告警清理，未宣称全仓warning=0或clippy通过。

## 知识与下一责任

SD03/04/05当前受测来源/目录/required闭包子轴可关闭，但F11完整恢复/持久化兼容仍被T07-R3-01阻断，不授整份delta完成。SD01/02第一批与原B2已闭保持；SD06正式worker/IPC、F09双线峰值、512受影响最终与T09完整规范落文仍归原owner。本轮不改r3语义、canonical或ledger。

下一角色work，仅修新的manifest身份升级回归；[任务单](glm-next-round.md)给出同树边界与永久关闭矩阵。再次冻结后受影响面复审。Plan继续r3/executing0/11，最终PASS后merge-owned冻结裸t＋VM/compiler tv/必要consumer/收尾门pending。
