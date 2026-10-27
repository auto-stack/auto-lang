# PLAN-821 R6 — 授权合同修订后 final pass

2026-10-26；Plan revision **6**；review_scope: **final**。
受审提交 `40f4c594fd6cfdc8d051a980c64f5e57baa67e4a`，diff base
`b1b8b425e563ba8358e4adc99c3e6c819a8fc137`；唯一实施树
`D:/autostack/.wt/lang-821/auto-lang`，分支 `plan-821-dev`，入/出审 clean。
实施代码本轮未变。依赖、Spec、配置及工具版本见 [baseline](baseline.json)。

**结论：代码、验收与拟议 Spec delta final pass；Plan reviewed，8/9。**
T-08/AC-08 及 merge-owned 冻结门禁仍 pending，尚未交付、合入、归档或清树。

## 授权、合同变更及审查身份

用户在本聊天明确选择：**“修订为现有静态类型／借用检查，明确不承诺运行期无效句柄校验（推荐）”**。
这是对 R5 具名 needs_replan 的明确裁定，不是 review 默示豁免。

r5→r6 保留 V-05、AC/T/SD 的稳定 ID；同步 §5.1.6、§5.5、V-05、AC-02/04、
T-03/04/06、SD-01/02/04，以及 contract-decision §5/§12。
原 runtime oracle 被明确替换，旧帧复用、跨 store、generation 回绕/退役的运行期
拦截 **未交付、未执行、不得写成 pass**。其他支持面、资源、兼容及增量合同未降低。
原合同与完成/重开历史均保留：[Plan r5](plan-r5-before.md)、
[决策 r5](contract-decision-r5-before.md)、[SD r5](spec-delta-r5-before.md)、
[任务历史](task-history-through-r5.md)。

当前 Codex 审查聊天延续 R1–R5，未参与受审实现，未委派。
本轮在同一审查聊天按用户裁定修订文档，再从代码及真实断言复核；不声称另有外部
文档审稿上下文，也不把修订后的文档本身当作实现证据。

## V-05 当前可执行合同与真实观察

| 不变量/输入 | 实际入口与观察 | 结果/边界 |
|---|---|---|
| 整数0恰等首个活资源id | `av_owned_821_v05_integer_equal_to_resource_id_stays_integer`；MIR verify→encode→decode→ABC深验→AV，整数运算=I32(0)、arena.live=0 | pass；类型隔离，不是句柄查表 |
| StrView入整数BIN | `av_owned_821_v05_view_into_int_op_rejected_at_verify`；真实MIR门报`mir.bin-type`，不执行AV | pass；此负例是前门拒绝，没有伪称测过runtime guard |
| 活MutRef阻止冲突写 | `av_owned_821_v02_live_mutref_blocks_field_set`；MIR门`mir.mut-conflict` | pass；loan准入规则 |
| 结束借用后同一写合法 | `av_owned_821_v02_borrow_end_releases_exclusion`；完整ABC→AV结果I32(1) | pass；正控防止一律拒绝 |
| BorrowEnd后同槽重新定义 | `av_owned_821_v05_borrow_end_then_redefine_frees_old_exactly_once`；结果41、两次alloc为4/1字节，旧buffer先free、新buffer出口free、各一次、live=0 | pass；实际拥有权/重定义，不测旧token解引用 |
| 多只读view/复制合法，owned或mut token复制、double move拒绝 | 其余3个v02用例：内容与`copy-owned`/`use-after-move`断言 | pass；相关静态合法/非法对照 |

本轮实际执行于实施树：

```powershell
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_semantics av_owned_821_v05_ -- --nocapture
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_semantics av_owned_821_v02_ -- --nocapture
```

**3+5=8 tests pass，0 failed/ignored**，编译未报warning。
[V05 raw](v05-targeted.txt)、[loan raw](loan-targeted.txt)。
没有重跑 aggregate acceptance、性能战役或任一整档。
R4修复触及的6个Rust文件独立rustfmt --check通过，[raw](fmt-scoped.txt)；沿用R5同SHA的cargo check零警告证据。
当前文档链接、9个稳定任务/9个AC/13个V/7个SD及8/9计数核验见[document-checks](document-checks.json)。

代码核验：`contract.rs` 的 Value 变体区分整数/StrView/MutRef；
StrViewSrc 只有 Share/Static，`resolve_view_bytes` 解析其持有的buffer；
`MutRef` opcode 创建 frame/slot/gen token，写入不通过token；ownership分析维护活loan与根冲突。
槽generation当前使用 `wrapping_add`，没有消费它的解引用/回绕退役检查。
因此 SD 不再把“有gen字段/bump”写成“运行期旧句柄已被拒绝”。
一些历史 Rust 注释仍有“generation at use”描述；这是与实现不符的非阻塞注释勘误，
已记债，不能充当r6规范或验收证据，本轮未改代码。

## 既有修复证据复用与最终覆盖

R6与R5是同一代码SHA、同一依赖；实施/兄弟树clean，既有Spec hash未变，验证输入与配置
绑定baseline。R6只改变V05的已授权要求和文档，故复用R1–R5中已独立成立的检查；
不以换审查轮次重复执行未变测试，也不把早期失败收据当成最终通过。

| AC | Tasks / V | 当前证据及结论 |
|---|---|---|
| AC-01 | T02/03/06；V01/10 | pass；首例MIR/ABC/AV/宿主结果释放与真实recipe已有R1基线；源码支持缺口由R5永久回放/独立808/810/8917/801及mut循环9关闭；跨unit view正证保持 |
| AC-02 | T03/04；V01/02/05/06/11/12 | pass；R1–R5终态/逃逸/loan/来源、字段重建与前缀重叠修复证据；r6 V05定向正反控8例。r5 runtime要求是获批替换，不伪称原oracle通过 |
| AC-03 | T03/04；V02/03/04/06/11/12 | pass；Aggregate/FieldSet提交后kill、事务失败保源、所有出口/host释放原证据保留；R5新owner重置/父子重叠与mut多形状重放通过，合法130层Clone通过 |
| AC-04 | T03/04/05；V04/05/08/10/11/12 | pass；非法状态/错型/loan前门、分态退出；R5真实新worker合法恢复及自洽坏内层深验拒绝2/2；r6仅静态V05，不含runtime句柄能力 |
| AC-05 | T04/06；V03/07/11/12 | pass；原分配故障/±1B/稀疏分析/字节fuel证据有效；R4独立四线同gate反控及合法跨线控制；R5 RecordView载荷hot=9,431,319B≥独立下界9,068,791B>8MiB，漏计关闭 |
| AC-06 | T02/05/06；V08/09/10 | pass；R1基线兼容/身份证据与R5新进程补核；源码/正式recipe成功/结果和ledger sabotage、worker故障收据不受r6影响；本轮补核实际必需case选择器控制，见下 |
| AC-09 | T09；V13四子case | pass；scalar既有D828基线保持；R5 7增量测试+独立probe确认未改前端零生产、对象/代际保持、失败安装旧结果7、连续变长1234→8及107→108，正式退化门反控证据保留 |
| AC-07 | T07；SD01..07 | pass；本报告最终逐AC/V/SD核验；原F01..11、R2-01无阻塞残项；绑定r6与冻结SD |
| AC-08 | T08 | **pending，merge-owned**；未执行冻结整档、合入/规范发布/归档/guard清理 |

V01/02/03/04/06/07/08/09/10/11/12/13分别由上表与R1–R5原始证据覆盖；
V05仅按当前五个属性核验。分支/恢复等case依其适用路径验收，不把两个新worker用例
扩大成任意类型/状态组合全部重测，也不把typed fixture能力冒称新增源码支持。

主要原始/关闭证据：[R1](../review-r1/review.md)、[R2](../review-r2/review.md)、
[R3](../review-r3/review.md)、[R4](../review-r4/review.md)、[R5](../review-r5/review.md)。
R5本轮代码已有**21定向测试**及独立probe通过；它们与本轮8例分开计数，未说全部族在本轮重跑。

### V-10 汇总与遗漏检查

读取当前脚本的 RequiredIds/失败透传，以及正式parent/worker的实际断言。
既有 `av_owned_821_negative_oracle_fails` 对真实fixture分别篡改result/ledger期望，
对应owned-result/owned-frees两轴红；worker exit9/extra-frame/wrong-hash为Error，
这些已执行证据不受本次文档变化影响。

本轮[validator-probe.ps1](validator-probe.ps1)通过AST加载实施树**实际Test-Selected函数**，
验证合法1例接受，以及0例、漏必需名、近似名、ignored、重复结果、重复必需ID均拒绝，
**7 controls pass**，[raw](validator-probe.txt)。这是实际选择器对合成日志的敏感性测试，
不冒称再次执行真实worker或aggregate。Invoke-Scope按子脚本exit传播失败，所用必需名均能
对应现有测试；本轮没有通过编辑一个平行checker来证明它自己。

## 最终 Spec delta 与findings闭环

目标版本/hash与存在性见baseline，冻结副本 [spec-delta-reviewed.md](spec-delta-reviewed.md)。

| Delta | before/after核验 | 结论 |
|---|---|---|
| SD01/02 | 两个add目标当前不存在；owned类型/操作/loan/字段状态及r6 V05边界与当前MIR/ABC/AV一致 | pass；不发布canonical |
| SD03 | 当前core/data/827恢复合同上增加owned A21身份和重验；R5新worker与自洽坏内层补证 | pass |
| SD04 | 当前profile目录增owned并明确语言/运行期非目标，不改旧profile | pass |
| SD05 | 现recipe架构追加owned注册/独立父侧判决；沿实际auto-lang-core test_cases owner，无新执行器 | pass |
| SD06 | HIR既有语义消费到owned，borrow/mut按实际adapter路径；原782多参move关联缝不冒领为后端新能力 | pass |
| SD07 | D828基础增加owned真单body前端、事务发布、坐标平移及真实heap载荷计量；无整前端常态降级承诺 | pass |

F821-R1-01..10的具体缺口/修复回归依R5及前序有效证据CLOSED；
F821-R1-11在本轮准确合同/SD冻结后CLOSED；F821-R2-01 health保持CLOSED。
原V05 runtime缺口以**用户批准的合同替换**解决，不记“原runtime已实现”。
遗漏/延后/workaround扫描未发现剩余合同内阻塞。保留既有782源域债，
借入record旧占位债已被RecordView实现关闭，不能继续列成当前未完成支持。

规范影响frontmatter已核：五个modify、两个add，精确路径有效；touched_goals=[]，
有限实验扩面不核销生产/AAC/AAV目标。主检出仅计划簿记，未改canonical Spec/ledger。

## Merge交接（尚未执行）

冻结代码与r6合同，merge须先在同树运行**裸cargo t + cargo tv + 独立workspace完整有限合同档**，
按实际diff补适用触面。门禁红必须归因；代码修复则复审受影响内容，旧pass不覆盖新SHA。
`cargo tf`仅主检出批量档、到期单实例。通过后才合入、沉淀SD/ledger、归档，最后先
`wt-guard.sh` clean再移除worktree/分支/空组。当前没有运行这些操作。
