# PLAN-743 Phase 3 修复终验记录（T-20）

> 实施分支 plan-743-dev（基点=激活提交 a0f8ddd44）；PYTHONUTF8=1，Category A。
> 外部复审反例原文：docs/reports/743-r2-quality-review-20261005/（只读）。

## Phase 3 门禁结果（实现 HEAD 实测）

| 门 | 命令/方法 | 结果 |
|---|---|---|
| 单元+反例 | `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py` | **63/63 OK**（41 旧 + 22 新：QA-01 四 fixture 测试+正例修正、QA-02/03/04 九类） |
| 生成 | `--write` ×2 | exit0；manual-decisions.json 字节不变（--write 不触人工层） |
| 严格校验 | `--check --require-decisions` | exit0：47 决定绑定新鲜；146 unknown 组 ↔ 7 族双向闭环（含新适用性规则） |
| 确定性 | 双独立输出 + 仓内制品三方 cmp | BYTE-IDENTICAL（HEAD 仅审计） |
| 格式 | `git diff --check` | clean |
| 链接 | 围栏感知活动链接扫描 | 无断链；计划文件 4 处 `../reports/`→`../../reports/` 后模拟归档路径全部可解析 |
| 范围 | `git diff --name-only a0f8ddd44..HEAD` 对 crates/ auto/lib docs/specs/ experimental/ Cargo 探针 | 零触碰 |
| CLI 残留 | 报告目录 ac-probe 裸引用扫描 | 0（acceptance-matrix×5、inventory MD-205、run-context 均为 auto-ac-prototype，历史处注记更名） |

## R2-QA → 修复对照（after 状态）

| 发现 | 修复证据 |
|---|---|
| R2-QA-01 分类越权 | shadowed_host：print=local-fn、IO.read_line=type-qualified（本地遮蔽宿主）；different_owner：Q 体 .next=unknown；unrelated_dot_pipe：自由函数点/管道=unknown；当前 owner 正例（P 体 .next/.pipe）=implicit-self；真实仓 CG.new/Ar.new 仍 type-qualified、parser P 体隐式 self 保留（commit 45fe4fb21） |
| R2-QA-02 证据未绑定 | decision-evidence-unbound 定位 ID/path；补绑定通过；证据内容变化→decision-stale（测试三段式锁定，commit a25a30722） |
| R2-QA-03 族闭环无引用 | resolved 族缺/空/不存在/不适用（kind 或 conclusion 不符或证据不覆盖路径）引用全部拒绝（family-ref-missing/-inapplicable）；有效 resolved/open 正例通过 |
| R2-QA-04 类型错误 traceback | 9 个参数化类型负例（kind/conclusion/evidence/bound/note/family path/names/disposition/owner）全部受控 ERROR[...] 定位、stderr 无 Traceback |
| R2-QA-05 canonical 摘要冲突 | SD-07 提案同步阶段摘要 R3 为两分规则（X9/trap 保留）；CLI 全量更名 auto-ac-prototype |
| R2-QA-06 生命周期 | 历史链接改归档可解析（4 处）；plans.md 743 行=r3 executing（激活提交已更）；ledger P743-3 活动指针可解析、P743-4 归档指针留 merge 对账；T-14 按 r2 收据补勾在案 |

## 冻结哈希（实现 HEAD @ 本记录提交）

| 制品 | SHA-256（前 16） |
|---|---|
| proposed-spec-delta-phase3.md（SD-06/07 冻结提案） | a4af013b759a1d35 |
| manual-decisions.json（r3 无内容变更，仅复检确认） | 3674c687647f88dc |
| source-manifest.json | 5725b5a6319a5cf3 |
| acc_inventory.py | 799c2bb643c5fd3c |
| test_acc_inventory.py | ae5f8ae26c66bca7 |

## AC-15..20 对账摘要

- AC-15 ✓：三 fixture 负例类别正确 + 当前 owner/CG.new/Ar.new 正例绿；不靠固定数量断言。
- AC-16 ✓：同条 evidence 每文件有准确绑定；缺绑定定位拒绝、内容变化 stale、非扫描输入证据受控；真实层严格门绿。
- AC-17 ✓：9 类型负例受控拒绝无 traceback；合法数据不误拒绝（63 测试含全部正例）。
- AC-18 ✓：resolved 引用存在+适用校验；open 三要素强制；真实 unknown 全有去向且保持可研究态。
- AC-19 ✓：SD-07 提案消除摘要/正文冲突；CLI/锚点/候选对齐 auto-ac-prototype；canonical 留 merge。
- AC-20 ✓：T-14 历史补勾有 r2 收据依据；r3 进度 14/21→（本 Phase 后）20/21；链接模拟归档可解析；Category A 全门禁通过。
