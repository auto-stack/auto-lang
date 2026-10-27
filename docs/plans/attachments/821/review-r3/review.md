# PLAN-821 final R3 — needs_fix

2026-10-26，r5；受审 `18dee14f9b5a7181a94cc2f48e3431e8a8108898`，前审 `e34a2c99940c0fec8558deffa27447a067a7553a`，基面 `b1b8b425e563ba8358e4adc99c3e6c819a8fc137`。唯一实施树 `D:/autostack/.wt/lang-821/auto-lang` / plan-821-dev 入审、出审均 clean。当前聊天为R1/R2独立审查上下文，未实施修复、未委派代理。

绑定：[baseline](baseline.json)、[入审Plan](plan-r5-at-entry.md)、[冻结SD](spec-delta-frozen.md)。依赖、旧Spec与未变验证面沿R2；本次只核修复影响及新mut写回路径。交接承认whole frontend与V05未闭，按技能就绪性规则不启动完整最终验收/统一回归。第三次提交不重置审查历史。

**结论：仍needs_fix。** F06深度反例与F07失败安装原例方向修复有效；F07全链路粒度、F08完整mut支持、F09计量、F11规范收口与R2-01 health仍未闭。未获得缩减合同授权，work的“5阻塞全部关闭”不成立。

## 本轮真实验证

- 定向 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_r1_replay --test av_owned_821_incremental`：7+5=12通过，0 ignored；[raw](scoped-tests.txt)。没有把整档t/tv或全部acceptance当作修复门。
- `cargo check --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe` 成功，**仍有1条dead_code警告**；[raw](check.txt)。`git diff --check e34a2c999 HEAD`通过。
- 独立probe：[源码](probe/src/main.rs)、[raw](probe-output.txt)。调用：

```powershell
cargo run --quiet --manifest-path docs/plans/attachments/821/review-r3/probe/Cargo.toml --target-dir D:/autostack/.wt/lang-821/auto-lang/experimental/ac-av-probe/target
```

- 130层单字段链、5619B wire，真实ABC授信→Clone→AV：**Success I32(0)，43209步**。关闭R2指出的未经批准128层截断回归。
- 候选源码前端合法、运行触发DivByZero：edit返回Err，revision保持1、production相同、旧程序仍7。验证新增build-aside的后段失败保护；不把新的成功例冒称所有link/资源/取消/迟到路径已验收。
- main-only body edit仍整前端 `frontends 1→2`，`incremental_gate Ok`，结果8；旧F07粒度缺口复现。
- mut只读混合、双mut、owned返回走真实 `owned_driver::request_owned`，前端/adapter/MIR/ABC/AV正常路径；捕获panic仅为了保留同轮其他输入，不把panic计成合法拒绝。四个源码与oracle见下。

## F821-R1-07 / P1 — 事务改善，但全链路函数增量仍被豁免

AC09/T09/V13/SD07。`owned_session.rs` 的 build_candidate仍对next_src调用 `compile_text_record_source` 整模块parse/resolve/typecheck/HIR生产，完成后才hash/筛单元。新gate明确要求 **frontends恰+1**，因此当前whole前端被判合格，未改函数HIR生产工作与身份也没有测量/保留。

独立main-only编辑输出：

```text
invalidated=["d_main"] frontends_before=1 frontends_after=2 gate=Ok(()) result=8
```

“计量可见”不等于原§5专项的“未改生产阶段=0”。将缺口归828 owner/跨仓写面，没有改变AC09或得到授权；相关入口实际在本仓experimental/ac-core，D828已经归档，原计划也明确T09是本计划work-owned必做项。若只读/接口边界使当前方案无法实现，应回new调整可执行方案/依赖，不由work把整前端列为完成边界。

本轮关闭的子问题：build-aside后运行失败保旧版已获独立正证，原panic输入的共享原因已修；继续保留原ID，不把剩余whole前端缺口拆成新需求。

关闭oracle不变：真实D828常驻函数前端/私有凭证路径，未改单元parse/recheck/emit等生产工作0、HIR/MIR/code对象与代际保留；真实whole frontend/执行载荷复制必须使同一结构门红；不要求重跑无关完整scalar战役。正式checker应在真实consumer入口成立，不能只让测试提供预期失效集合。

## F821-R1-08 / P1 — mut写回仅单一形状成功，新路径三个回归

AC01/02/03/04/T03/T06/V02/V06。原支持合同未排除多参数或owned返回，三个输入均经过既有text-record前端；当前错误发生在新adapter/MIR阶段。不是要求扩大源语法。

| 输入 | 手审预期 | 实际 |
|---|---|---|
| [单mut+int返回](mut_one_int.at) | 808，合法对照 | Success I32(808) |
| [mut+只读record](mut_plus_readonly_record.at) | 810（a.n=8，返回8+2=10） | **panic** hir_adapter.rs:1643，len2/index2 |
| [双mut](two_mut_records.at) | 8917（a=8、b=9、返回17） | `mir.use-after-move`：第二次FieldMove读已消耗wrapper |
| [mut+owned Str返回](mut_owned_return.at) | 801（a.n=8，"R"长度1） | `mir.field-type`：Str字段读取落到Str槽，要求StrView |

原因均已定位到新包装机制：

1. Call解包遍历 **所有Record类型实参**，没有按callee的MutBorrow模式挑写回字段。只读record也递增wb_idx，访问不存在的fields[2]导致panic。需要检查混合普通/move/只读/命名位置映射，而不是只修索引越界后丢弃真实写回。
2. 新FIELDMOVE语义第一次抽取就kill整个wrapper，双mut所需第二个wb字段随即不可读。当前内部机器语义不能兑现多参数源义务；不能把源程序支持面缩成单mut。
3. f0始终用FieldGet解包，并注释“source result copyable”；owned Str/资源record结果本来不Copy，普通FieldGet无法转移所有权。需保证结果和每个写回值各自唯一转出，清理顺序/失败责任正确。

关闭：上述三例+现有808、借入/整根move正控经真实source→ABC→AV均正确；补混合签名与非Copy结果同源永久断言，正确校验每个commit/kill与失败清理。不靠跳过额外参数/改预期为受控拒结项。补完共享机制后仅回放受影响族与兄弟，不附全档。

## F821-R1-09 / P1 — space()仍是磁盘指标，execution仍误用进程峰值

AC05/06/T02/T04/T06/V07。同checker接受阈值、零B线/零hot门反控这部分已改进；**指标仍不对**。

`av_owned_821_resources.rs` 新checker将 `SpaceReport.blocks` 当hot、`staging+trash` 当staging。实际 `experimental/ac-artifact-probe/src/store/mod.rs:495` 的space()调用blob磁盘total_size、journal.metadata().len与dir_size(staging/trash)，返回的是磁盘使用量。换成store自己的API没有把磁盘字节变成受管内存。已有正确口径在probe/bench.rs：peak_hot_bytes、peak_staging_bytes、peak_exec_slot_bytes对应32MiB内存分区（hot8/staging16/execution8）。

更严重的是新checker `if peak > execution && execution < b_line`：peak为父侧**整个worker private-bytes**，却拿它当8MiB execution。128MiB worker线与8MiB guest槽位线是两个独立指标；合法进程例如10MiB、guest槽位1MiB会被这个判据误拒。小首例刚好低于8MiB不能验证指标成立。

关闭：保留真实parent峰值128MiB门，分别观测hot/staging/guest live值的真实峰值，按各自预算判断；同一checker反控覆盖每条独立线。可保留store磁盘监测作为另列信息，不能代32MiB内存；使用正确约束的合法跨线正控和超线反控，不新增规模承诺。原F09仍OPEN，不是新资源需求。

## F821-R1-11 / P2 — SD仍将未授权缩减写成已交付规范

SD07当前将“每修订恰一次整模块前端”与“整个生产面增量”并列，末尾把单函数前端归外部owner；这与原AC09及当前canonical resident-function-install.md的零未改前端生产/身份合同冲突。SD01/06又宣称mut写回全链落地，实际只覆盖808单例。

Plan§0/§10仍保留R2执行中2/9的“当前”文字，而frontmatter与最新工作声明是execution_done7/9；历史可以保留，当前行应统一。V05仍明确pending且没有批准合同修订，不能称其“不阻塞”替代具名必需矩阵。

关闭：先解决07/08/09及V05，再冻结可沉淀SD（当前Spec版本/hash、before/after和实际AC证据）；不以修改SD降低原要求。review仍不发布canonical Spec/ledger。

## F821-R2-01 / P2 — 旧4警告清除，但有新dead_code警告

`probe/src/owned_session.rs:51` 的Production::bump已无调用。独立cargo check与12测试编译均报告warning；因此仓规zero-unhandled-warnings未达，保持同health finding ID。关闭只需受影响包/测试编译无警告，不要求总测试。

## 闭环与下一步

| 范围 | R3结果 |
|---|---|
| 原R1-01/03/04/05 | R2关闭证据保留，当前修复未触及其主要机制，不为换HEAD重跑全族 |
| 原R1-02 | 原反例闭，新RecordView/签名组合完整不变量pending；mut新路径失败不能由原7例覆盖 |
| 原R1-06 | 字节工作原证据保留；130层反例独立Success，截断回归关闭；不扩成所有清理资源面已验收 |
| 原R1-07 | 失败候选保旧版已验证；全链路粒度仍OPEN |
| 原R1-08 | 单mut写回改善；三种相邻合法形状失败，PARTIAL/OPEN |
| 原R1-09 | 同gate反控改进；指标错配，PARTIAL/OPEN |
| 原R1-10 | 新恢复代码本提交未改，本轮未复核完整新进程/坏内层，pending；已有work证据不等于独立pass |
| 原R1-11、R2-01 | OPEN |

保留T01/T05；T02/03/04/06/T09重开，**executing2/9**。AC01..06 partial、AC09 fail、AC07 fail、AC08 merge-owned pending。未核的必需V05/恢复/资源矩阵不记pass。

下一步同树work按原ID闭合，不得把整前端+owner归因变成批准降要求。更新第二次失败后的覆盖诊断：mut模式映射、wrapper多字段消费、owned结果、正确资源量纲，以及“计数可见”为何不满足零未改前端工作；引用本轮源码和实际observed，提交新SHA/closure后再审。若设计不能满足原验收，回new显式改方案/合同，不在work标签中越过。

本轮未改实施，未合入/归档/清树，未跑完整acceptance或整档。最终pass之后merge-owned统一门仍pending：裸cargo t+tv+实际触面与约定独立workspace有限合同档；tf仅主检出到期单实例。
