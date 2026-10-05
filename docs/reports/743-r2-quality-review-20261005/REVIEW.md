# PLAN-743 r2 合入后独立复审（2026-10-05）

**结论：needs_fix。** 41/41 Python 测试、当前真实报告严格门、两次生成确定性通过。
r2 对上一轮具体反例的修复有效；新增反例仍暴露调用分类、证据绑定、unknown 闭环和字段类型校验缺口。
canonical 阶段摘要与修订正文不一致；归档收尾有状态/链接遗漏。不得用原 pass 覆盖这些失败。
用户此前明确授权发现问题时重新激活同一 743 并追加 Phase；本轮只复审和更新合同，不实施修复。

## 基线、独立性与验证范围

- reviewed_commit: 5983f8aeedeae2dc5769f9a0820eec4b722d4c6d；plan_revision=2。
- base_commit: c6e4e568994941883b8c1b6b19dff703feb081cd（r2 激活合同）。
- 交付链 6859e984f/89c5850c9/5b1719591/fde60c2c9/8b44ae855/4ca002099/4bec7575c/fdded7af6/5983f8aee 在主线；源码及冻结提案直接核验，不采信 executor 的 R3 pass。
- 复审期间主线前进到 ed2d00b9076e68fdc41ca59dfa096c753cee5f9e，差异只有其它工作线 crates/a2r-std/src/sqlite.rs（NOTES-001）；743 源码、受管输入、报告和 Spec 均无差异。本报告不把该改动归到 743。
- 本会话参与过 r2 合同修订，未参与 Python/设计修复实现。已交付 worktree 被清理；读取已提交主检出，在独立 TEMP 输出/fixture 副本重放，不向真实报告 --write，不改变实现或 canonical Spec/live ledger。
- Python 3.14.2 标准库；无网络/外仓依赖；Category A，不运行 cargo/tv/tf/taa/docs_gen。
- 重放旧 reproduce.py：41/41（20.383s）、两个 --write 字节一致、归一审计 HEAD 后与已提交 manifest 相同、summary 相同；当前 --check 通过。另独立 --check --require-decisions：47 条新鲜、9 输入、146 unknown 候选闭环，exit0。
- 初次重放的 Windows 子进程输出使用本地编码，UTF-8 reader 报错；重跑显式 PYTHONUTF8=1 后以上结果成立。仅保留后次 tests/结果。部分旧 --check 摘要中文有本地编码乱码，exit/结构化观察不受影响；严格门另行 UTF-8 重放。
- 范围检查：r2 六条实施提交只涉及工具/测试/报告/设计；旧源码、根 Cargo、741 profile 未被本计划修改。
- 18 个活动本地 Markdown 链接审查（围栏不检查），两个归档计划链接失效。SD-04/05 canonical 修改正文与冻结提案逐字匹配；匹配不代表未修改的旧摘要已同步。

## 输入和 Spec 增量冻结指纹

| 来源 | SHA-256 |
|---|---|
| scripts/acc_inventory.py | 8be8c39952887304c126b3fe6eec6bb53c0f44ea05555548857a442bf6d99250 |
| scripts/tests/test_acc_inventory.py | 1f8bde2f77b0272f20c5168a4f609f55d1a7d5d958955cedd250367b384e06bc |
| docs/plans/archive/743-acc-bootstrap-hir-contract.md | f0133243cd33f20bb60aa4540fa194829198252d68ab7677a5d9b59aa3134ffe |
| docs/specs/auto-acc/project.md | 21cd68db2764c0bf52f4a71dfa8f648f99a75cf7b3309b4fadbd31b131606275 |
| docs/specs/auto-hir/stage-contract.md | 6e4d0cded8886196d707fe21e755d62ad5548fcf87a054f06ea17bbd5b820248 |
| docs/design/strategy/auto-acc-bootstrap-contract.md | 676521ab3b9f26ba39798a6c1f3eb78676a6040861055ede15dd0dbb088e5317 |
| docs/reports/743-acc-hir-contract/manual-decisions.json | 3674c687647f88dc7324395038bb883d1ffca04f21d67a34dc54c4118e104a5d |
| docs/reports/743-acc-hir-contract/proposed-spec-delta-phase2.md | a963adfbd6b2488772f9830c7f6ea279f1f5eecf9417fd1ba25f0f743f7cbf6b |

新报告文本统一 LF 入库；源文件的原始字节 SHA-256 仍以上表为准。冻结提案全文：frozen-phase2-spec-delta.md（不覆盖原 r1/r2 资料）。counterexamples.json/audit.json/replay-results.json/tests.txt 固定为此次基线。

## 上轮问题的复核

| 原发现 | 本轮结果 |
|---|---|
| P743-QA-01 | fixed：跨行 pretend 不泄漏、结束后真实代码与行号正确，原 fixture 翻转 |
| P743-QA-02 | 原 Meter.new/x.len、CG.new/Ar.new 已修；保证仍受下面 R2-QA-01 挑战 |
| P743-QA-03 | 原空绑定/缺证据/空决定负例拒绝、scan-only 分开；保证仍受 R2-QA-02..04 挑战 |
| P743-QA-04 | 设计正文 const-fold v2/运行期 trap/X9 已修；canonical 旧 R3 摘要仍有 R2-QA-05 |
| P743-QA-05 | fixed：A2 分 HIR/源码；A8/A9 owner/两候选能力前置已对齐 |
| P743-QA-06 | fixed：真实 47 决定绑定新鲜，8 决定/9 绑定逐项重审理由在案，不是机械换 hash |
| P743-QA-07 | T-05/06 已修；最终 T-14/索引/链接仍有 R2-QA-06 |

## 新发现与修复要求

### P743-R2-QA-01 / P2：同名证据不是当前接收者/owner 证据

scripts/acc_inventory.py:337–340、363–366、394–423。
namespace/native 优先于本地声明；_post_classify 仅用全模块方法名集合，把其它 owner 的方法也升为 implicit-self-method。
shadowed_host.at 明确声明 type IO.read_line 与 fn print，调用却均 native-runtime；
different_owner.at 只有 P.next，Q.go 的 .next() 也被认成 implicit-self-method；
unrelated_dot_pipe.at 无当前 type owner 的裸点/管道 next 也被升级。counterexamples.json 有完整观察。
这违反 r2 AC-09/§5.6 的本地 owner/当前作用域优先规则；现有同名保护测试还把无 owner 的形态升级当正确。
不要求完整名字解析器；能证明本地/当前 owner 才升级，有本地或导入冲突则不得猜宿主；不能证明时保留 unknown。
补当前 owner 正例、其它 owner/同名宿主/无 owner/管道负例；重新生成清单并逐项核准新增 unknown。

### P743-R2-QA-02 / P2：证据存在却未绑定，变化不触发失效

scripts/acc_inventory.py:682–705：evidence 检查文件存在，bound_input_hashes 另行遍历，二者没有包含关系检查。
合法 fixture 决定追加 review-evidence.txt:1，但绑定只含原 token.at。严格 --check 在证据改动前后都 exit0。
本轮当前真实 47 决定的 evidence 全有绑定（audit.json=空缺口），所以不把现有清单误报为已陈旧；问题是工具会对未来不完整决定提供错误的新鲜保证。
按同一决定核验每份证据文件有其当前 SHA-256 绑定；外部于扫描 9 输入、但在仓内的证据同样受控。
证据和绑定身份一致；缺绑定明确诊断 ID/path，已绑定证据变化仍走 stale；不以放宽规则或统一 hash 覆盖来通过。
影响 T-09/12、AC-02/03/10/13、SD-04。

### P743-R2-QA-03 / P2：resolved unknown 族没有决定仍宣称闭环

scripts/acc_inventory.py:744–758：resolved 只检查 disposition；ref 为空时不报错。
从有效 fixture 的所有 resolved families 删除 decision 后，严格 --check 仍 exit0/“覆盖闭环通过”。
已有 families 与真实 MD-501..507 正常引用不否认；负例证明 gate 的闭环保证不成立。
resolved 必須显式引用真实、适用的决定；open 必須有 owner/probe/work_package，owner 与 decision 的语义分开。
至少校验必要引用/字段、记录状态与族的对应关系；补缺失/空/不存在/不适用引用、有效 resolved/open 正例；不把任意存在 ID 当语义闭环证据。
影响 T-09/12、AC-03/10/13、SD-04。

### P743-R2-QA-04 / P2：字段存在校验代替类型校验，错误输入抛 traceback

scripts/acc_inventory.py:659–705。
合法 JSON 的 kind=[]、conclusion=[]、bound_input_hashes=[...]、evidence=17 分别触发 TypeError/AttributeError。
counterexamples.json 保存真实 CLI stderr；exit1 不是带决定 ID 的受控拒绝。
修复须先验证完整字段类型和元素类型，再做集合/迭代/hash检查；包括决定、families、路径/hash/ID/状态/引用。
非法数据返回有定位的 ERROR[...]，无 traceback；新增参数化负例，不使用吞异常后空结果的兜底。
影响 AC-02/10、T-09；不扩为通用 schema 引擎。

### P743-R2-QA-05 / P2：canonical 阶段摘要仍是旧 effect 规则

docs/specs/auto-hir/stage-contract.md:16–18 仍写“R3 effect 只能收窄”，
同文件新“保守 effect 方向”节及设计正文 R3(a)/(b) 已要求分析精化与语义双向保持分开。
SD-05 冻结提案未覆盖旧摘要，所以 merge 逐字落地仍留下冲突。影响 AC-04/05/11。
新 delta 必须同步摘要/正文/例示，且保留运行期 trap/X9；本轮不直接修改 canonical 行为。
acceptance-matrix/桥待验证问题仍用旧 ac-probe，而 741 r2 已是 auto-ac-prototype；核对实际命令与待建源码状态，避免后续照文档无法执行。

### P743-R2-QA-06 / P3：归档生命周期/导航未闭环

归档 frontmatter archived/r2，却 current_step=13/total_steps=14，T-14 仍 [ ]，其 review/merge/clean 收据已齐。
auto-acc/plans.md、auto-hir/plans.md 的 743 行还写 executing/r2；live ledger P743-3 的 file 指向不存在的活动文件。
归档计划两个 ../reports/... 链接实际指向 docs/plans/reports/...，均不存在（audit.json 精确行号）。
按收据纠正历史 T-14 并保留事实；本次新 Phase 另列待办，最终 merge 对状态/路径/索引/ledger/链接逐项检查。
本轮不修改 live ledger；重开后 P743-3 的旧活动指针恢复可解析，P743-4 的归档指针暂待下一次 merge 对账。
影响 AC-08/14，不能靠归档文件有 archived 就宣布所有元数据一致。

## 验收对账与 Spec 路由

| AC | 本轮结果 | 依据 |
|---|---|---|
| AC-01 | partial | 9 输入/8 模块/18 use 与确定性保持；调用分类仍有新缺口 |
| AC-02 | partial | 41 测试、正常/严格 CLI 绿；错误字段未受控拒绝，证据新鲜门不完整 |
| AC-03 | partial | 主体/能力/迁移表齐全，真实决定当前新鲜；分类/决定闭环保证不足 |
| AC-04/05 | partial | 设计正文/const-fold/trap 修复有效，canonical R3 摘要冲突 |
| AC-06/07 | pass（合同域） | 10 锚点、两候选单向前置、A2/A8/A9 状态、代际与估算完整；旧 CLI 名需同步 |
| AC-08 | fail | 本轮独立复审发现未闭环事项 |
| AC-09 | partial | 原反例已修，当前 owner/同名宿主负例失败 |
| AC-10 | fail | 新绑定/引用/字段类型负例失败 |
| AC-11 | partial | 设计正文正确，canonical 摘要仍旧 |
| AC-12 | pass | 两候选/锚点能力边界与 owner 无环且不抢编号 |
| AC-13 | partial | 真实决定语义重审/新鲜成立；通用闭环保证受新反例挑战 |
| AC-14 | fail | 归档 T-14/counters/索引/链接未一致 |

SD-04/05 正文与冻结提案一致；需要补 SD-06/07，而不是把现有规范改成接受错误实现。
所有新事项转 PLAN-743 r3 / Phase 3；旧 pass/merge 作为历史保留。
不实施优化器、不扩大 native 子集、不改 741 或外仓；无需新的用户产品裁定。

## 复现

在新输出目录（必须不存在；避免覆盖冻结证据）：

```powershell
$env:PYTHONUTF8 = '1'
python -B docs/reports/743-r2-quality-review-20261005/reproduce.py D:/autostack/auto-lang <fresh-output-dir>
python -B docs/reports/743-r2-quality-review-20261005/audit.py D:/autostack/auto-lang <fresh-output-dir>
python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py -v
python -B scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions
```

旧反例与确定性另外用 docs/reports/743-quality-review-20261005/reproduce.py，独立新输出目录。
--write 只在 scratch，真实人工决定与旧历史报告均不覆盖。
