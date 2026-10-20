# PLAN-815 r2 / M815-A 独立阶段复审 R4

2026-10-20。**outcome: pass（M815-A 阶段）**；review_scope: phase；T-13 / AC-08。所有历史finding已关闭，无新增finding。

这是阶段通过，**不代表整Plan已reviewed或已经发布**。整Plan保持executing，T-14冻结回归门及实际交付/规范/来源收据仍pending。

## 基线与证据复用

- reviewed_commit：`7045e20fa71ba437bc80075f17d9aace755f5abd`。
- repair_base：`bc61501c58d45dde28ee2c739425dbc3c911b38c`；原diff base：`b9f7ab7e1f400073006e096a08b436f933e4a836`；Plan revision=2。
- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，分支 `codex/plan-815-dev`；入场及验证后clean、HEAD稳定。
- reviewer：Codex `/root`，延续R1-R3独立审查上下文，未继承实施会话自检、不创建代理/第二树，不声称未暴露的模型身份。
- [baseline.json](baseline.json)记录全部实现文件/运行输入/主检出Spec与候选哈希、依赖809完整SHA和主检出HEAD。与R3逐文件比较，**唯一变更是 `crates/auto-project-model/src/wire_tests.rs`**；生产实现、其他测试、manifest、锁、构建配置和canonical Spec输入均未变化。
- 真实工具二进制SHA256仍为 `1dc1ec700c7dfdb310e8d61b8250909b6c4549548288daa8de213b69badeb07a`，与R3完全相同。
- 因此复用 [R3报告](../review-r3/review.md) 中已绑定的实际进程/路径/深度/诊断/资源/轻闭包/777证据；这些行为所依赖的源、配置、依赖和输入未变，且本轮diff不改变相关扫描域。本轮仅重跑受影响codec单测与该包健康检查，未机械重启全部95测试或整档。

## F-815-R3-01：CLOSED

核对修复diff与实际断言：

1. helper已经返回完整wire，现直接交给 `request_from_wire`，不再重复包入request。
2. 同一npm_deps/raw两条构造路径各有合法Value控制，必须解码成功；恢复旧二次包装会让这两条控制失败。
3. 六个非法Value控制必须报在 `value` / `value.t` 分支，并且包含各自违反的键规则（t/extra/e/k）；任意的 `request / unknown key wire` Shape不再能满足断言。
4. raw负控同样必须落在value.t并指出t缺失。原有真实进程三位置坏payload→Error→DescribeOk→exit0回归保留。

这完成了原例闭合、合法对照和同族错误分支区分，没有通过删除负控或放宽断言补绿。

实际运行：

| 命令 | 结果 |
|---|---|
| `cargo test -p auto-project-model --features wire --lib wire_tests::` | exit0，**14/14**，包含修复单测、诊断/shape/数值/无损/深度控制；38个非wire测试filtered，不冒称本轮全部52重跑；[wire-tests.log.txt](wire-tests.log.txt) |
| `cargo check -p auto-project-model --features wire --all-targets` | exit0，零warning；[check.log.txt](check.log.txt) |
| `cargo fmt --check -p auto-project-model` | exit0；[fmt.log.txt](fmt.log.txt) |
| `git diff --check bc61501c5...HEAD` | exit0 |

R3原95个reported pass中，受影响的14个wire测试以本轮新结果替代，其余未变的81个定向测试及独立反例证据按上述哈希核对复用。没有重复计为“本轮95个新执行”。

## 全部finding状态

| ID | 当前状态/证据 |
|---|---|
| F-815-R1-01 | CLOSED保持；无损诊断与不可表示路径规则未变，R3独立证据保留；本轮相关wire测试通过 |
| F-815-R1-02 | CLOSED保持；实际Value解析和三位置进程控制未变，本轮修正了其unit假覆盖 |
| F-815-R1-03 | CLOSED保持；数值/版本/窗口实现未变，受影响wire族本轮通过 |
| F-815-R1-04 | CLOSED保持；7组路径双cwd独立证据与实际tool二进制身份未变 |
| F-815-R1-05 | CLOSED保持；精确wire120/frame122完整信封及双读路径证据不变，相关wire测试本轮通过 |
| F-815-R1-06 | CLOSED保持；本轮包级all-targets零warning/fmt通过，其余未变健康证据复用 |
| F-815-R2-01 | CLOSED保持；双侧诊断准入未变，本轮相关codec控制通过 |
| F-815-R3-01 | **CLOSED**；正确fixture、同路径合法对照、目标分支及具名错误断言均实际通过 |

## AC / 任务 / 规范结论

| AC | 阶段结论 |
|---|---|
| AC-01 | A轻闭包、资产登记和来源范围通过；完整UI主体/生产资源迁移仍B pending，不宣称完整AC已通过 |
| AC-02 | A同源codec/字段/路径/无第二求值范围通过；完整UI各生成路径仍B pending |
| AC-03 | A真实describe/validate端点、版本/能力拒绝、错误/日志/帧预算/退出和诚实生命周期范围通过；progress/交互/运行进程寿命等完整B能力不提前声称 |
| AC-04 | A真实consumer/闭包/来源登记通过；792完整UI发行fresh/locked与旧兼容等B范围继续pending |
| AC-05 | 整体最终审查T-09未到入口，不因阶段pass勾选 |
| AC-06 | 整体merge/归档T-10/T-11未到入口 |
| AC-07 | **pass**：本阶段work与必要回归/真实consumer证据齐备 |
| AC-08 | **pass**：T-13绑定本SHA、能力面、规范子集和明确merge-owned pending |
| AC-09 | pending：T-14尚未运行，当前无实际发布/下游放行收据 |

A规范采用 [spec-delta-reviewed.md](spec-delta-reviewed.md)，取代原候选§4中残留的“完整帧120/一概budget_exceeded”概括；保持R3已校对的规范正文与行为规则，仅更新R4绑定提交和阶段通过状态。SD-01-A/02-A/04-A/06-A目标、当前canonical输入及GOAL-003/007真实存在性已核对；SD-03/05和完整B规则不提前沉淀。该文件仅为已审增量，**未修改canonical Specs或ledger**，其冻结hash见final-baseline.json。

原候选/合同另有冻结副本，作为输入历史：[frozen-m815-a-candidate.md](frozen-m815-a-candidate.md)、[frozen-contract-baseline.md](frozen-contract-baseline.md)。后续merge应使用本轮精确已审A增量及其输入范围，不能把历史概括重新当发布事实。

## 路由与pending_closeout_gates

完成T-01/T-02/T-03/T-12/T-13，current_step=5/14；Plan仍 **executing**。本轮不标整体reviewed、不勾T-09、不归档、不清唯一树。

下一步T-14由merge owner在本冻结提交/实际合入输入上串行执行**裸cargo t + 必要新包/协议/独立consumer门**。当前diff无VM/trans/UI生成触面，无tv/tt/tu触发；实际合入若改动代码、依赖、配置或输入，按影响重审/重放。tf仍仅主检出到期单实例批量。

冻结门通过后，完成实际master阶段合入、精确A规范与受管ledger增量、delivery commit主检出祖先/协议与source指纹/consumer解析收据，才能放行816按“已发布M815-A”消费。B仍依既有814/815阶段门继续；阶段pass不代表整个815已交付。

本轮未执行冻结整档、合入、归档、清理或远端写入；上述pending均未冒报通过。以后新增代码不会自动继承此pass。
