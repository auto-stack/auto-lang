# PLAN-743 r3 合入后独立复审（2026-10-05）

**结论：needs_fix。** 63/63 测试（14.562s）、真实报告严格门、两次生成确定性通过。
上一轮的主要反例确已修；本轮组合路径仍发现三项 P2 工具缺口和一项 P3 归档生命周期遗漏。
用户已授权发现问题时激活同一计划、追加 Phase；本轮只复审/合同，不修 Python、不发布 canonical/live ledger。

## 基线与独立证据

- reviewed_commit：8e8f6f19b96516fdbc9d4c893297fa1757a7aad4；plan_revision=3。
- base_commit：a0f8ddd443418fb32a67d308cbd4a0fd00f74835（r3 激活合同）。
- 落地提交173bf3458/c0c6d71e3/b093471db/b537e6273/a5b68196b/deee10031、归档32e1e8648/清理8e8f6f19b经 Git 历史核实；实现已提交且主检出 clean，743 worktree 已移除。741/742/ABI 检出在途，未修改/删除。
- 独立性：本会话此前起草 r3 修复合同，未参与 r3 Python 实现；结论由当前源码、独立 CLI 副本和新负例重建，不采信 executor 的 R4 pass。
- Python 3.14.2 标准库，PYTHONUTF8=1；Category A，不运行 cargo/t/tv/tf/taa/docs_gen。未改旧源码/native/外仓；tests/fixtures 和生成仅写 TEMP，真实报告未 --write。
- 当前严格门：47 决定全部新鲜、9 受管输入、146 unknown 候选/7 族闭环，exit0。现有 7 resolved 族的真实决定证据均覆盖其路径（audit.json），本轮不将它们误报为当前语义陈旧。
- 重放旧 reproduce.py：63/63、两个生成目录字节相同、归一 HEAD 审计字段后与提交的 manifest 相同、summary 相同；原多行/同名方法反例通过。
- 文档 audit：20 个活动 Markdown 链接无断链；SD-06/07 三段 canonical 与冻结 phase3 提案逐字匹配，旧 R3 摘要消失。报告文本统一 LF；来源原始字节 hash 见下。
- 独立复现：有效 fixture 完整层 exit0；在同一控制层改变被族引用的决定字段、证据覆盖，测试真实 --check --require-decisions（非只调用内部函数）。

## 输入与 Spec 增量指纹

| 来源 | SHA-256 |
|---|---|
| scripts/acc_inventory.py | 799c2bb643c5fd3cd51b576f5bd67c9314817213c69c8f66796b35b47a9f272c |
| scripts/tests/test_acc_inventory.py | ae5f8ae26c66bca79d0247fb93ba62b4ec5cd3487e235e128ac40322d3e62c5b |
| docs/plans/archive/743-acc-bootstrap-hir-contract.md | ee15366ce5ed229ae682baedaa0c198ba472136d2630bb5353a9d5df6865403f |
| docs/specs/auto-acc/project.md | d239e5370ffcdc842a86987e436c70da451c98779cf9f64a7a4071b8da19b628 |
| docs/specs/auto-hir/stage-contract.md | 85989f57c70dd48676932c6fd91415f7b06446faed077feee92a337b021db7b7 |
| docs/design/strategy/auto-acc-bootstrap-contract.md | 676521ab3b9f26ba39798a6c1f3eb78676a6040861055ede15dd0dbb088e5317 |
| docs/reports/743-acc-hir-contract/manual-decisions.json | 3674c687647f88dc7324395038bb883d1ffca04f21d67a34dc54c4118e104a5d |
| docs/reports/743-acc-hir-contract/proposed-spec-delta-phase3.md | a4af013b759a1d35dd7a70b34b202202e28e3c04b4ade204dc9ac47f72217466 |

冻结 delta 全文见 frozen-phase3-spec-delta.md；counterexamples.json、audit.json、replay-results.json、tests.txt 为本轮基线，不覆盖旧报告。

## 上轮六类问题的关闭情况

| 原发现 | 本轮复核 |
|---|---|
| P743-R2-QA-01 | 原本地 IO/print 遮蔽、不同 owner、无 owner 点/管道反例已修；导入/参数遮蔽仍有 R3-QA-03 |
| P743-R2-QA-02 | fixed：同条证据未绑定拒绝，补绑定通过，文件变化 stale；真实人工层均有绑定 |
| P743-R2-QA-03 | 原无/空/不存在决定引用已拒绝；证据适用性仍有 R3-QA-02 |
| P743-R2-QA-04 | 原未被族引用的类型负例受控拒绝；被引用决定的组合路径仍有 R3-QA-01 |
| P743-R2-QA-05 | fixed：canonical R3/正文一致，CLI auto-ac-prototype 同步，SD-06/07 三段匹配 |
| P743-R2-QA-06 | module plans 与链接已修；20/21、T-21 及 P743-3 指针仍有 R3-QA-04 |

## 新发现

### P743-R3-QA-01 / P2：被类型检查淘汰的记录仍进入族引用索引

scripts/acc_inventory.py:741–742、810–819。
第一遍发现 evidence=17 后 continue，但 decisions_by_id 又从原始 decisions 建索引，只检查 id 是 str。
有效控制层中，MD-REVIEW-900 是所有 resolved 族引用的决定；仅把它的 evidence 改为17，严格 CLI 在族适用性检查抛 TypeError（int is not iterable），不是带 ID 的 ERROR/CHECK-FAIL。
63 现有测试的错误字段写入 decisions[0]（module-role、未被族引用），因此漏了这个 consumer 组合路径。
修复：族消费者只读经过类型/语义校验的决定索引，或在结构错误时明确终止适用性检查；无效/缺引用提供定位诊断。不得再索引被淘汰记录，不以通用吞异常/空输出兜底。
新增被 resolved/open 引用的各字段错类型/缺失/重复 ID 组合负例，断言非零且无 traceback；有效完整层绿。
影响 T-17/18、AC-02/10/17/18，canonical 已承诺受控类型拒绝；不需要重定义 schema 语义。

### P743-R3-QA-02 / P2：hash/subject 绕过“证据覆盖族路径”

scripts/acc_inventory.py:818–821。
canonical auto-acc/project.md、冻结 SD-06 和错误文案均要求 evidence 覆盖族路径，但代码把 bound map 或 subject 覆盖作 OR 替代。
有效 unknown-resolution/resolved 决定的 evidence 只指向注册宿主 lib.rs（不含任何族路径），追加族路径 hash 后，所有族仍严格 exit0。
另一控制仅把 subject 写成目标族路径，证据仍不覆盖，也严格 exit0（其它族保持具名 open，前置完整）。
hash 绑定证明文件新鲜，subject 命名对象；两者不能代替审定的证据引用。当前真实 7 族证据均覆盖，不否认已提交结论的有效性。
修复：适用性按明确 evidence 文件集合判断，再要求同条 binding 新鲜；移除两条 OR 兜底，不把 canonical 改宽来接受本负例。补“仅 hash / 仅 subject”拒绝和真正 evidence+binding 正例。
影响 AC-03/10/18、T-18、SD-06；无需新增调查结论。

### P743-R3-QA-03 / P2：qualified native 判定漏导入/参数遮蔽

scripts/acc_inventory.py:258–264、360、376、414–417。
imported_symbols 仅用于裸调用；_post_classify 没有导入或可见变量信息，只排除 local_types 后查宿主白名单。
import_shadow.at：use other: IO, List, File, process, print；裸 print 是 imported-symbol，四个 qualified 调用却 native-runtime。
parameter_shadow.at：IO 明确是 Meter 参数，IO.read_line() 仍 native-runtime，违背变量接收者 unknown 规则。
这已在 r3 AC-15 的“本地及导入同名证据优先”范围内；不是要求完整 resolver。
修复：把可证明导入/局部参数/绑定冲突纳入 native admission；无证明时保持 unknown，有限词法扫描遇到不能确定的作用域不得猜宿主。补导入/参数/局部变量正反例，不改变真正无冲突宿主正例及已修 owner 规则。
影响 AC-01/02/03/09/15、T-16；SD-08 可澄清执行政策，不放宽原约束。

### P743-R3-QA-04 / P3：最终阶段和历史指针仍未收口

归档 metadata archived/r3、current_step20/total_steps21；T-21 无完成标记，末段还称 T-15..21 待执行；实际 review/merge/clean 收据已完成。
ledger P743-3.file 仍指向不存在的 docs/plans/743-acc-bootstrap-hir-contract.md；P743-4/5 归档路径有效。audit.json 是递归读取现有 ledger 的结果，无写入。
修复合同明写最终21/21且所有历史指针须可解析，这项此前未闭环。module plans 已 archived、20链接已通过，不重复报告为坏。
本轮按真实收据补历史 T-21=完成，再另列 r4 待办；最后 merge 对全部 P743-* 指针、task/counter、active/archive 状态做自动断言，不仅检查新 P743-5。重复 R4 标题、旧阶段“尚未实施”语句应标明起草历史以免误读，不篡改历史 verdict。
影响 AC-08/20、T-21；本轮不修改 live ledger，P743-3 指针随重开暂恢复、P743-4/5 暂待下次 merge 统一对账。

## 全 AC 对账与路由

| AC | 结果 | 证据 |
|---|---|---|
| AC-01/02/03 | partial | 覆盖/确定性/完整表成立；导入分类、组合类型/适用性保证不足 |
| AC-04/05 | pass（合同域） | canonical R3/运行 trap/例示/阶段边界一致，无 pass 实现声明 |
| AC-06/07/12 | pass（合同域） | 10 锚点与两候选能力/前置/状态/代际/估算保留，CLI 已同步 |
| AC-08 | fail | 本轮独立复审仍有未闭环事项 |
| AC-09/15 | partial | 原同名/owner 反例已修，导入/参数遮蔽失败 |
| AC-10/17/18 | fail | 原负例大部分已修；新组合类型与证据适用性失败 |
| AC-11/19 | pass | SD-06/07 文本落地匹配、摘要与正文不再冲突 |
| AC-13 | pass（当前真实清单） | 47 决定 hash 新鲜、7 真实 resolved 族 evidence 覆盖；工具通用保证的失败另见 AC-18 |
| AC-14/20 | fail | 收尾21/21/P743-3指针未一致 |
| AC-16 | pass | 同条 binding 缺失/补全/变化三段式真实 CLI 行为正确 |

转同一 PLAN-743 r4 / Phase 4，P743-R3-QA-01..04；旧 r1/r2/r3 交付/复审保留，不新增计划号。
修复仍只工具/合同与簿记；不改旧 compiler/native/741/742/ABI，不开展 ACC 实现。

## 复现

新目录（避免覆盖冻结资料），PYTHONUTF8=1：

```powershell
$env:PYTHONUTF8='1'
python -B docs/reports/743-r3-quality-review-20261005/reproduce.py D:/autostack/auto-lang <fresh-output-dir>
python -B docs/reports/743-r3-quality-review-20261005/audit.py D:/autostack/auto-lang <fresh-output-dir>
python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py -v
python -B scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions
```

生成确定性与旧扫描反例另用 docs/reports/743-quality-review-20261005/reproduce.py，独立新目录。
