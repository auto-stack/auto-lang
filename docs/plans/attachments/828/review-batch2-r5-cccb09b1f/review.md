# PLAN-828 R5：T07-R3-01 窄复审 PASS

2026-10-22；`review_scope: phase`；**PASS，T07-R3-01/P2 CLOSED，无新增阻塞finding**。原例与实际引用集合身份不变量均已验证，不再只按固定版本轮换判断闭合。

## 冻结身份和范围

- 同树`D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@cccb09b1f2b77e14cb410811eb4493247e2ed787`，base=`b846f45dc10abb580136ff97f0562db7c1f62e41`；主检出交接簿记f54608491264397f85df546c808c775e798c410f。
- 入口/结束clean、HEAD未变，519项源码/锁/配置/Specs/Design43/主Plan接口/技能输入匹配。baseline/final-state记录完整hash，auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，probe未消费。
- reviewer为本Codex审核上下文，未实施修复、未启代理、不编造未暴露身份；从生产代码和独立输入重建结论。实施代码、canonical/ledger未改。
- 本次只授窄复审阶段PASS；Plan r3/executing0/11，完整T07/F09预算、T08正式worker/IPC/双线测量、T09/整份最终和merge-owned门不由此自动完成。

## 机制与关闭证据

`persist_manifest`把实际`(artifact_id,RefKind)`规范化一次（排序、去重）；同一份规范化数据生成键`required-fp`并重构实际发布edges，原body fp并列保留、format2继续隔离旧配方。当前四种RefKind的往返含义一致。相同body但primary重建目标不同，变成不同key；枚举顺序/重复不再制造同key异artifact。旧row不覆写、不吞Conflict，新根成功注册后才退旧根的顺序保持。

| 关闭轴 | 本轮实际观察 |
|---|---|
| R3旧缓存迁移 | 原`migration-controls.rs`5/5原样通过；partial冷自愈、完整旧ABC/sidecar读后重开、真实同内容编辑、新缓存幂等均正常 |
| R4同format2重建反例 | 原`rebuilt-closure-controls.rs`2/2原样通过；fresh reference仍证明body逐字节相同；旧primary `art5f39...f0707`→重建`artef697...fd963`，restore现在Ok，后续warm admitted3；未移动源码缺块正控保持 |
| 实际publisher顺序/重复 | 私有组件调用生产persist_manifest，逆序＋加入重复边仍同一root/key/artifact；独立读取store实际edges表，精确等于去重后的6条required边 |
| 目标/边种类区分 | 实际publisher加入合法辅助目标后新身份；分别RequiredData/Recomputable/InvalidationFingerprint/Provenance都正确写入，各次身份不同；原row字节保持，未改store授信/Conflict规则 |
| 发布失败 | 缺失RequiredData目标导致真实TargetMissing，旧manifest根保持，旧程序仍可warm admitted3；不把受控拒绝当成功 |
| 新闭包→GC→恢复来源 | 自建缺块store中合法源码位置变化，重建后新root≠旧root而manifest body相同；旧row字节不覆盖；退役全部soft根后真实GC删除3块，required根仍保护新闭包；warm admitted3，DivByZero完整SpanInfo同冷编：func0/start30/end35 |

因此旧例关闭、完整被修复不变量关闭、修复相邻路径无回归，三者分别成立。上轮B2来源映射/owner/调用闭包/预算/事务及T07目录/required根保活的已审结论保持，不要求重做无改动前端或候选机制。

## 独立组件证据的方法与边界

`generate-publisher-controls.py`读取本冻结session.rs，仅将文件开头`//!`文档注释换成`//`以便include；函数体tokens不改，原件/投影hash见`publisher-source.json`。`publisher-injections.rs`在组件内调用**生产私有publisher**，检查真实store结果，而不是只测hash helper；不新增不可信公共构造器、不改WT。三个publisher测试3/3，原组件内另一个测试由filter明确未选，实际commit控制另跑1/1。

目标/RefKind变体为私有publisher身份区分控制，不声称新增了源语言能力。组件投影的unused/dead-code 7项warning是审核装配范围产生的，不归为生产新增警告。

`gc-span-controls.rs`只在测试自建nonce临时store下canonicalize验证后删除一个blob，调用正式recover、restore/cold/persist、GC和再恢复；未接触用户cache或WT。recover在该fixture的清理计数为0，未冒称它已清理所有缺行；实际自愈由生产链完成。

## 执行记录

原WT定向构建：

```powershell
cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_mig --test function_incremental_828_t07 --test function_incremental_828_t08 --test function_incremental_828_b2 --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json
```

从本证据目录复现：

```powershell
powershell -NoProfile -File ./run-targets.ps1
powershell -NoProfile -File ./run-controls.ps1 -Source migration-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source rebuilt-closure-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source t07-controls.rs -AllLibraries
python ./generate-publisher-controls.py
powershell -NoProfile -File ./run-controls.ps1 -Source publisher-controls.rs -Filter publisher_review_ -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source gc-span-controls.rs -AllLibraries
```

Cargo artifact/target-results保留库选择、exe/hash、args、真实exit；所有最终定向调用exit0，没有把zero-selection或此前work日志当本轮PASS。

| 定向面 | 结果 |
|---|---|
| 原migration / 原rebuilt-closure / 原T07 GC目录控制 | 5/5、2/2、5/5 |
| 新实际publisher / 新GC精确来源链 | 3/3、1/1 |
| permanent mig / t07 / b2 / b1 | 9/9、8/8、14/14、11/11 |
| session / contract / t01r / r1 | 18/18（显式skip两个512）、22/22、4/4、86/86 |
| 冻结T08脚手架兼容 | 3/3，仅兼容，不授正式测量PASS |
| sidecar / generation / binding / close / close-adjacent | 6/6、8/8、6/6、6/6、3/3 |
| cache_tier / 真commit失败组件 | 20/20、1/1 |
| 两改动Rust fmt / diff-check / freeze | 0/0/0，519输入一致、clean |

集合有重叠不加总宣传唯一覆盖数。AV/core/ac-core未改；前轮binary/unary/三入口及core证据仅在未变源码、依赖和配置范围延续，本轮不冒称重跑。两512、完整性能/资源门没有执行；不运行整档t/tv/tt/tb/tu/taa/tf或docs_gen。既有shim/SpanInfo、T08 CloseOp/InitOp告警按原T09清理责任保留，不宣称全仓零warning或clippy。

## 规范与路由

SD05/F11的本次持久身份兼容修复通过；SD03/04已审子轴和第一批结论保持。本轮不授F11所有剩余轴/完整SD05、SD06正式worker/IPC或F09完整预算完成，不改canonical/ledger/原r3语义。

**下一责任work：按原Plan进入T-08正式测量与相应第三批责任**，不再为本finding修复往返。保持executing0/11；T07完整512/共享存储预算与T08双线峰值、正式端到端样本及T09规范/清单按原owner完成后，再做第三批/整份最终验收。最终PASS之后才在冻结提交执行merge-owned裸t＋VM/compiler tv/必要consumer及收尾门，通过前不得merge。本次未启动实施/发送GLM消息/合入/归档/清树。
