# PLAN-828 C04单项窄复审 PASS

2026-10-22；review_scope=phase；冻结 **a030427fe15dc043871d3253101f209653cabd9c**。**PASS，B3-C04/P2 CLOSED，无新增finding**。原例与失败出口的完整接受计量不变量均已闭合；C01/C02/C03/C05及前两批已闭机制保持。可进入T-10整份最终验收；本收据不是整份最终PASS或合入许可。

## 冻结与审核边界

- 同树D:/autostack/.wt/lang-828/auto-lang，plan-828-dev；base=d16af9cb3956921c523074ec0a60316a7b79d8a0。主检出入口HEAD=f6b99189ac03bb16cdc8e85de61c54e20084e9bf（包含用户交接簿记540f6834f）。
- 入口、结束clean，HEAD/依赖未变；baseline/final-state的520项源码锁配置/Specs/Design43/冻结主Plan接口/技能hash一致。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。三交付文档/Spec delta及所有原始控制保存在本目录，hash见evidence-index。
- 本Codex审核上下文未实施、未启代理，不编造未暴露身份。未改WT源码、canonical/ledger，未整档、合入或清树。
- 唯一实现差异为session.rs的计量落点，27行差异；不改变put参数/顺序、错误处理、根/身份/回收/执行器或publication。检查完整diff及五个写入口，未只按测试总数判闭合。

## 机制与原控闭合

局部store_write_bytes累加器和成功末尾统一汇总已删除。ABC、HIR、sidecar主行、sidecar修复的Ok点即时更新self.phase_io；manifest仍在其put成功后即时记账。Conflict解析幸存行和失败本次put零计，返回Ok的幂等重发布仍按既定逻辑payload接受口径计一次。

因此任何后续Err都不会撤销已经写入phase_io的前序接受计量；是共同落点修复，不是HIR/sidecar两个特殊补丁。相关早退之后仍由原persist_error记录错误、内存版保持原有发布语义。

使用上轮**同一生成器/同输入/同真实store故障/同oracle**，生产session函数体tokens未改，旁路记录器仅逐次委托真实put并记录payload与接受状态。四个真实TargetMissing故障出口全部转绿：

| 指定失败位置 | 实际此前接受字节 | 修复后上报 | 结果 |
|---|---:|---:|---|
| ABC第1写 | 0 | 0 | PASS |
| HIR第2写 | 173 | 173 | PASS（原0） |
| sidecar第3写 | 425 | 425 | PASS（原0） |
| manifest第4写 | 497 | 497 | PASS |

io-failure-controls.log **4/4**、真实退出0；旧903/no-store控制 **2/2**。源码/限定注入/实际错误/hash见io-source.json、io-failure-source.json及对应源。故障注入代表真实put的通用Err出口，不声称正常sidecar会自行生成坏边。审核投影9项unused/dead-code警告属于装配范围，生产定向构建零告警。

成功路径fresh四个Ok payload仍是173+252+72+406=903、只计一次；失败put本身不计、此前已发生计量保留。修复不改逻辑payload与envelope/SQLite/WAL物理开销的边界，也没有更改资源上限。

## 本轮定向证据

| 面 | 独立结果 |
|---|---|
| 原逐写故障 / 原IO | 4/4、2/2，0/173/425/497及903逐值相等 |
| 原握手/中断收据 | 4/4，ready前EOF/坏JSON/类型错及编辑中坏JSON报告保持 |
| 原judge / 实际run内副本 / identity | 6/6、2/2、3/3；合法命中绿，副本/缺目录/不配对序号红 |
| 四个Program入口与旧pin trace | 1/1 |
| 官方T08含正式30轮 | 5/5，一次独立调用通过，raw/report在formal-official |
| permanent mig/t07/b2/b1 | 9/9、8/8、14/14、11/11 |
| session/contract/t01r/r1 | 18/18（显式skip三512）、22/22、4/4、86/86 |
| 改动Rust格式 / diff-check / freeze | 0/0/0，520输入一致、clean |

正式本次P50/P95/P99=**212.6478/219.4917/221.9354ms**，init1300.2778ms；worker峰值211775488B、父侧22392832B。官方同798双线硬预算通过（日志A=201/B=0，显示整数MiB）；只代表本次冻结负载采样，不扩成普遍工程性能承诺。

未重跑512、core全套或整档。本次只移动观测计量，不改变512断言依赖的存储/执行/拥有关系；不拿work旧测试数冒充独立复现。未变core/正式worker/T08测试/配置与依赖的既有证据在明确边界延续，当前相关控制已用新库重放。最终长负载/兼容性/整份AC/V/SD证据由T-10统一核版本与必要补测；裸t+tv/实际触面consumer及tf到期主检单实例仍merge-owned。

## 实施所报单次战役失败的证据限制

实施记录承认一次战役失败后重跑全绿，但没有保留最初失败的具体诊断，当前只有“机器负载”解释。因此本轮**不独立确认该归因**，也不把三次重跑视作最初失败不存在。当前独立正式战役一次5/5且完整raw可核；计量修复的原故障与相邻出口另有直接区分证据。

此历史记录不另立C04阻塞finding；T-10应如实保留该限制，按本冻结实际证据核正式门。后续每次战役使用独立证据目录，若再红先保存诊断/完整raw并归因，不覆盖失败后仅报告重跑绿；不要求追造已经丢失的历史日志。

## 复现

冻结WT按cargo-artifacts列出的定向targets构建后，在本证据目录：

```powershell
python ./generate-io-control.py
powershell -NoProfile -File ./run-controls.ps1 -Source io-controls.rs -Filter measurement_ -AllLibraries
python ./generate-io-failure.py
powershell -NoProfile -File ./run-controls.ps1 -Source io-failure-controls.rs -Filter accepted_io_ -AllLibraries
```

两组实际退出0；方法与前红例保持，未依赖实施TEMP验证台。生产库/worker/test exe绑定Cargo实际artifact路径与hash，原始日志保存。C03和握手重放源、生成器也全部保留。

## 路由与Spec

B3-C04的握手、成功/失败计量、先落盘后断言、正式阶段观测等已审子项现全部关闭；第三批已登记finding均闭。SD01/05/06相关完成声明不再受C04缺口阻塞，但完整Spec delta仍待T-10逐项核对，不在本轮给整份delta授最终PASS。coverage_t08=5更正保持，canonical/ledger未动。

Plan保持r3/executing0/11，下一责任 **review/T-10**。以a030427fe为唯一冻结根汇总前批与本收据，做整份AC/V/C/SD及遗漏/未授权延期核对，明确证据复用/补测边界；若代码变化须重新冻结受影响面。不在本次窄复审中自动改reviewed或合入。T-11 merge门禁及沉淀/归档/guard仍pending。
