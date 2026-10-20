# PLAN-815 r2 / M815-A 独立阶段复审 R3

2026-10-20。**outcome: needs_fix**；review_scope: phase（T-13 / AC-08）。

**上一轮四项代码问题均已关闭；当前只剩一项新增回归测试缺陷 F-815-R3-01。** 不把已关闭的代码问题重新标红，也不以测试总数代替测试区分性。

## 冻结范围

- reviewed_commit：`bc61501c58d45dde28ee2c739425dbc3c911b38c`；repair_base：`faea8c9d727df1ab2b16d45fed95c0f345c48180`；原diff base：`b9f7ab7e1f400073006e096a08b436f933e4a836`。
- Plan revision=2；唯一树 `D:/autostack/.wt/lang-815/auto-lang`；分支 `codex/plan-815-dev`。入场/验证后clean、HEAD稳定。
- reviewer：Codex `/root`，延续R1/R2独立审查上下文；未继承实施会话自检、未创建代理或第二工作树，不声称未暴露的模型身份。
- 依赖809已发布，无814 WIP；当前main HEAD、Plan/Spec/候选/配置/锁与实现文件哈希见 [baseline.json](baseline.json)。真实重建二进制身份见 [binary-identities.json](binary-identities.json)：SHA256 `1dc1ec700c7dfdb310e8d61b8250909b6c4549548288daa8de213b69badeb07a`。
- 原候选与合同分别冻结为 [frozen-m815-a-candidate.md](frozen-m815-a-candidate.md)、[frozen-contract-baseline.md](frozen-contract-baseline.md)。审查校对的精确A规范增量见 [spec-delta-reviewed.md](spec-delta-reviewed.md)，哈希在final-baseline.json；未发布canonical/ledger。
- 本轮未修改实现、未合入/归档/清树、未运行任何整档。

## 历史finding关闭表

| ID | 本轮结论 | 依据与覆盖 |
|---|---|---|
| F-815-R1-01 | CLOSED保持 | 原诊断payload/不可表示路径修复保留；本轮诊断成员/defines真实往返正确，路径转换代码未被改动 |
| F-815-R1-02 | **CLOSED** | 缺t、t类型错误、额外键、obj错键、pair缺键均在实际Value分支受控报错；资产/raw/raw-node-props三个真实进程位置均Error→DescribeOk、exit0；永久进程回归也命中真实位置。单独的新增unit测试存在R3-01，不把它误说成解析实现仍失败 |
| F-815-R1-03 | CLOSED保持 | 数值转换未再改动，52项model及原rich数值/窗口测试通过，R2已验证的数值/版本反控保留 |
| F-815-R1-04 | **CLOSED** | 普通相对路径、同盘盘符相对、异盘盘符相对、无盘符根路径以及output/front/dep共7组×两个cwd，报告逐组完全相同；特殊形式明确cwd-qualified-path、不探测cwd；绝对/普通相对的既有控制仍通过 |
| F-815-R1-05 | **CLOSED** | 裸wire119/完整信封121与精确wire120/完整信封122均codec、write、read、parse、typed envelope完整通过；wire121的58层请求在codec拒绝；完整帧121/122通过，123/126两个读入口与writer同限具名拒绝，129由JSON解析器最后防线拒绝 |
| F-815-R1-06 | CLOSED保持 | 三包和独立fixture格式通过，scoped all-targets check零warning |
| F-815-R2-01 | **CLOSED** | 编码侧members/defines通过、future/空field拒绝；解码侧重复两次/三次前缀、空/未知payload均拒绝，真实tool也返回Error；原完整成员字段身份保持 |

深度的当前规则是wire120、完整frame122、解析器128分层，而非四入口各自从不同根计同一个120。该规则实现了原“合规codec请求经真实信封可写可读”的不变量，没有以拒绝全部深度请求补绿。

## F-815-R3-01 [P2，新增测试缺陷] 回归单测把完整wire再次包装，所有负控都在无关入口失败

AC-07/08 的回归验证要求，T-03/T-12；这是本轮新增测试本身的问题，**不是旧四项实现缺陷复活**。

位置：`crates/auto-project-model/src/wire_tests.rs:458..498`，尤其469、477两处调用。

`minimal_request_with_npm` / `minimal_request_with_raw_value` 返回的已经是 `{"wire":1,"request":{...}}` 完整文档，调用方却再次构造：

```text
request_from_wire({"wire":1,"request":helper(...)})
```

因此所有6个坏Value及raw负控均提前在请求第一层返回 `Shape { field:"request", reason:"unknown request key 'wire'" }`，从未进入想测的Value tag解析。断言只要求任意Shape，遂全部通过。

独立区分性控制：将坏Value换成完全合法的 `{"t":"int","v":1}`。

| 构造方式 | 合法Value | 缺t/错t/额外键/obj错键/pair缺键 |
|---|---|---|
| helper返回的正确wire直接解码 | 成功 | 在 `value.t` / `value` 分支具名拒绝 |
| 当前单测的二次包装 | 仍返回 `request / unknown key wire` | 每种都返回同一个无关错误 |

也就是说，当前unit的“拒绝成功”断言连合法输入也会通过，不能守住声称的同族负控。现有真实进程tagless三位置回归是有效的，但不代替此unit声称覆盖的错类型/错键/额外键分支。

修复范围很小：直接将helper完整返回值交给request_from_wire，或把helper改为返回request字段并保持调用方一致；在相同构造路径加入合法值必须成功的控制，并断言错误来自目标value/tag分支，不能接受任意Shape。保留现有有效的真实进程回归。无需重写实现或扩大测试档位。

证据：[独立codec日志](independent-codec.log.txt) 的7条unit-oracle-control（包含合法输入），[可直接重放的probe](probe/src/main.rs)；源码定位及错误文本均绑定当前SHA。review没有修改实现测试。

## 本轮实际执行与证据

| 检查 | 结果 |
|---|---|
| `cargo test -p auto-tool-protocol -p auto-ui-tools -p auto-project-model --features auto-project-model/wire` | exit0，52 model +18 protocol +4 resources +17 process =91；[targeted.log.txt](targeted.log.txt)。其中R3-01指出的一条unit虽显示pass，其所声称负控覆盖无效 |
| 指向重建真实tool的独立fixture测试 | exit0，4/4；[fixture.log.txt](fixture.log.txt)，现有定向测试共95个reported pass |
| `cargo check -p auto-ui-tools -p auto-tool-protocol -p auto-project-model --features auto-project-model/wire --all-targets` | exit0、零warning；[check.log.txt](check.log.txt) |
| 三包fmt + 独立fixture fmt | 均exit0，fmt.log / fixture-fmt.log |
| Git Bash verify-deps.sh | exit0，轻闭包无Man/lang/VM/gen/media大包；[dependency-tree.txt](dependency-tree.txt) |
| 777 strict | exit0，pass / 0 violations / 2 grandfathered /75 edges；[audit-summary.json](audit-summary.json) |
| `git diff --check b9f7ab7e1...HEAD` | exit0 |
| R2原共享回放 | [r2-process-replay.jsonl](r2-process-replay.jsonl)；[r2-codec-replay.log.txt](r2-codec-replay.log.txt)在旧future-field的unwrap处停止，因当前编码侧正确拒绝，exit101是旧probe不适应新语义，不能称整份回放通过 |
| 新独立回放 | [codec probe](probe/src/main.rs)覆盖合法/非法fixture、精确完整信封边界、两读路径、诊断两侧准入；[process probe](independent_process_probes.py)覆盖7组路径/三位置坏payload/流继续/诊断拒绝/手工深度帧。实际结果见[日志](independent-process-results.jsonl)，再由[逐项断言收据](process-assertions.json)核对，所有进程均有10秒截止 |

已闭合的R1普通shape、数值、不可表示路径、日志/版本/能力/EOF/输出超限等未受本轮改动影响的证据，结合当前定向包测试与来源差异复核保留；不机械重复全量或把已闭finding再造为新项。资源登记源与生产embed未变化，4项资产测试也本轮重跑通过。未运行裸t/tv/tt/tb/tu/taa/tf。

## AC与规范结论

- AC-01/04：A轻闭包、来源登记、真tool/consumer、资源未提前迁移有证据；完整UI主体/792发行fresh/locked/旧兼容仍B pending，不作为本轮新缺陷。
- AC-02/03：A当期的实际行为与四项代码修复通过；原B范围不宣称完成。
- AC-07：实现验收通过部分保留，T-03同族回归的区分性仍有R3-01，当前work交接未完整。
- AC-08：T-13 needs_fix，不勾选。AC-05/06/09分别等待整体final或merge，未到阶段。

规范检查没有再要求新能力。本轮已在review产物中校对出精确A增量，修正候选§4残留的“完整JSON120/一概budget_exceeded”和将测试计数写入规范等概括；采用其合同§4已经明确的120+2分层和已观察的错误分类。见 spec-delta-reviewed.md 的SD-01-A/02-A/04-A/06-A。它描述当前真实实现与阶段来源；canonical目标版本/hash见baseline，GOAL-003/007及完整B元数据不删，SD-03/05完整退役规则仍B。此为未发布候选，不能绕过本轮non-pass。

## 路由与证据复用

Plan保持executing；确认T-01预算/完整封装已完成、T-02保留；T-03与依赖的T-12因唯一R3-01继续未完成，current_step=2/14；T-13不勾选。修复交接不要再宣称旧四项代码问题仍未解决。

下一步可严格限定为修复上述unit输入构造/正负控断言，在原树提交新SHA。若实现、依赖、配置及既有反例输入保持不变，下轮可按哈希复用本轮四项代码关闭、真实进程、深度/路径及规范证据，只重放该测试及受影响的定向检查、基线健康/Spec一致性；不必重新跑整阶段矩阵，更不触发整档。

pending_closeout_gates：阶段pass后T-14串行冻结裸cargo t+必要新包/协议/外部consumer门，以及实际master交付、A规范/ledger/来源发布收据。当前没有VM/trans/UI生成触面，不加tv/tt/tu；合入输入改变再判。tf仅主检出到期单实例批量。未合入/归档/清树，仍不能放行下游按“已发布A”消费。
