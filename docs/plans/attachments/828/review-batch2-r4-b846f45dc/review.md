# PLAN-828 第二批 R4：T07-R3-01 窄复审

2026-10-22；review_scope=phase；**needs_fix**。T07-R3-01/P2的旧键迁移原例已关闭，**完整身份不变量仍PARTIAL/OPEN**，沿用原ID，无新增finding。

## 身份与范围

受审树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@b846f45dc10abb580136ff97f0562db7c1f62e41`；base=`3090f2326f3595a69132c80bee3a2891c5f41b76`。只有session manifest key的format1→2和六项永久迁移测试。入口/结束clean，HEAD不变，519项源码/配置/Specs/Design43/主Plan与interface/技能指纹匹配（baseline/final-state）。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。

Plan r3/executing0/11；本Codex独立审核上下文未实施修复、未启agent，不编造未暴露身份。按auto-plan-review核实际代码与独立反例；没有写实施源码、canonical或ledger。T08脚手架兼容重放不作正式性能/IPC/双线内存验收；未merge。

## 已通过的关闭面

旧空边format1键与新required边format2键隔离，旧根仍经artifact id读取且先注册新根后退役旧根。R3 `migration-controls.rs`五项**原样重放5/5**：partial旧store冷自愈、直接重开、完整旧ABC/sidecar可读后重开、同内容编辑persist_error=0、新store幂等正控均通过。新增永久mig6/6，包括旧→编辑→GC→warm恢复值9。

R2 required-root/目录控制5/5、永久t07 8/8继续通过；未撤销required边、严格目录、旧根退役或已通过的帧槽投影/预算/候选机制。

## T07-R3-01 / P2 残项：同一format2下引用图改变仍占同键

**原规则与范围**：F11/T07/V07的缓存miss→冷回退/重建→再恢复，783同layer/key单artifact身份且artifact_id含required边。R3任务单要求身份策略处理同body但不同required闭包，保留缺失/自愈。interface本轮进一步声称“同body不同边集……永不共键”。一次版本隔离只区分旧空边/新required两种配方，未区分**同一新配方下不同required目标**。这是原不变量的残项，不增加缓存语言或性能范围。

**原样可执行控件**：`rebuilt-closure-controls.rs`，两个test构成一正一反。使用纯scalar原源：

```text
fn helper() int { return 7 }
fn main() int { return helper() + 2 }
fn tail() int { return 9 }
```

1. 通过本冻结生产`warmup_persisted`生成全新**format2键**store，保留真实manifest根与required图。
2. 从公开get_artifact的chunk_id定位helper primary sidecar。只在本测试创建的唯一临时store中，canonicalize后断言文件位于该自有root下，移除**一个**blob，模拟已登记的缺块输入；未碰用户cache、WT、DB或其他artifact，没有伪造ABC/HIR。
3. 调用正式store::recover(store,true)，日志完整保留；该fixture下recover计数为0，**不声称recover已经清理了缺行**。
4. 正控保持源不变；反例只把`return 7`改为`return      7`，保持内容语义、函数与调用不变。
5. 在另一个全新reference store冷编反例源码，取真实manifest body，**逐字节断言与原manifest相同**，证明CV与目录全等，非凭自报“同内容”。
6. 调公开restore→正常miss冷回退→真实持久化，检查实际新primary身份与manifest发布结果。

**观察**：未移动源码正控恢复Ok，后续warm admitted3。位置变化反例的primary成功重建：旧id=`art5f39bb8cd39777403b1374a7ed70cdf73880d4e10e9de5b92716e6b2085f0707`，新id=`artef697db300fccd48d213383cd35db14db0df8b5d47c77c41c29683bf5a0fd963`；原始payload重建成功，不是侧边缺失被误当manifest问题。

随后restore返回Err，准确失败在：

```text
A09[entry=d_main,format=2,fp=2933d5f4914d8d43,kind=session-manifest,owner=m_src,producer=plan798/av-probe]
existing: art7b8fe54781c6d3e5c5ba8b11ae81fbc1b9276b34fd5d9e970764fbe40954c91f
attempted: arte776c0c94ad4c4cabe9af0819d2d140efb43e916a9cb5b59215b1cc26b696904
Store(Conflict ...)
```

**期望**：新required目标变化后可发布身份不同的闭包、保留旧row不覆写，cold自愈成功，下一次warm恢复正确值/来源；不是把真实Conflict吞掉或禁persist。正控与失败反例除合法来源位置变化外相同。

**根因已证实**：`session::persist_manifest`键依然只包含producer/owner/kind、固定format2、entry、fp(body)，没有表示实际required集合的变化；shared store按含edges的artifact身份正确拒绝同key第二身份。不是要求store放宽Conflict。fmt1→2确实修好了上一批示例，但没有修同body+新sidecar目标导致不同artifact的问题。

**P2边界**：正常缺失缓存的自愈返回Err；未发现错误guest值被发布或原子性被破坏。沿用T07-R3-01，原例闭合与不变量闭合分别记账。

**关闭oracle**：给实际发布的body＋规范化required闭包一个稳定且可区分的key身份；同集合不同枚举顺序应幂等，不同目标应不同键或通过等效且可证明的兼容机制。不应再仅2→3做固定版本轮换：每个固定版本都能发生同样的primary重建。保留旧format1迁移5＋永久6、原required根GC/用户行/旧根退役；新增本反例应能cold成功→新图注册→GC→第二次warm，数值和来源与冷编相等。对外来/坏版本仍正常拒绝，不删除用户数据。

## 证据与测试

本轮构建9个指定integration targets（`--no-run --message-format=json`），Cargo artifact、精确目标exe/hash/args/exit见cargo-artifacts/target-results。控件从本目录执行：

```powershell
powershell -NoProfile -File ./run-controls.ps1 -Source migration-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source t07-controls.rs -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source rebuilt-closure-controls.rs -AllLibraries
powershell -NoProfile -File ./run-targets.ps1
```

| 定向面 | 实际结果 |
|---|---|
| R3原migration五项 / 新引用图5控制 | 5/5、5/5 |
| 新缺块正反控制 | 1pass/1fail，exit101，对应同一原finding残项 |
| permanent mig/t07/b2/b1 | 6/6、8/8、14/14、11/11 |
| session/contract/t01r/r1/T08脚手架兼容 | 18/18（明确skip两个512）、22/22、4/4、86/86、3/3 |
| sidecar / generation / binding / close / close-adjacent | 6/6、8/8、6/6、6/6、3/3 |
| cache_tier / 实际commit失败组件 | 20/20、1/1 |
| 两改动Rust文件fmt / diff-check / freeze | 0/0/0，519输入一致、clean |

不加总重叠测试。AV/core/ac-core零改动；上轮帧映射binary/unary/三入口、core22等证据仅在相同源码/依赖/配置边界内延续，本轮不冒报再次运行。两512、正式worker/IPC/性能和双线内存仍pending。既有shim/SpanInfo与T08 CloseOp/InitOp告警保持原T09清理责任，未宣称全仓告警为0。

**探针修正**：初次补前提时误以为走repair key，增加的repair存在断言失败。已保留`rebuilt-closure-wrong-repair-premise.log`，纠正为实际primary重建ID变化，最终前提均成立后仍在原manifest Conflict处失败。不会把错误repair前提算产品finding。最初不带该误断言的raw也保留；只有最终源/日志用于结论。

## 收据路由

SD03/04已审来源/绑定/寿命子轴保持；SD05/F11完整持久化兼容仍受原finding残项阻断，SD06/T08正式与F09/512/T09仍由原任务交付。r3目标、支持面与AC不变，未修改canonical/ledger。

下一角色work，按[窄修复任务单](glm-next-round.md)处理required集合实际身份，保留已有通过机制。整体executing0/11、无整份reviewed/merge；最终PASS后的merge-owned裸t＋必要VM/compiler tv/consumer/收尾门pending。本轮未整档/docs_gen/修实现/合入/归档/清树。
