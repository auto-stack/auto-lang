# PLAN-743 Phase 4 修复终验记录（T-26）

> 实施分支 plan-743-dev（基点=激活提交 1d8fba15c）；PYTHONUTF8=1，Category A。
> 外部复审反例原文：docs/reports/743-r3-quality-review-20261005/（只读）。

## Phase 4 门禁结果（实现 HEAD 实测）

| 门 | 命令/方法 | 结果 |
|---|---|---|
| 单元+反例 | `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py` | **83/83 OK**（63 旧 + 20 新：R3-QA-01 消费者组合、R3-QA-02 hash/subject 双旁路、R3-QA-03 四 fixture/正例、类型参数化 9 例） |
| 生成 | `--write` ×2 | exit0；manual-decisions.json 字节不变 |
| 严格校验 | `--check --require-decisions` | exit0：47 决定绑定新鲜；146 组/7 族闭环（含 evidence-only 适用性） |
| 确定性 | 双独立输出 + 仓内制品三方 cmp | BYTE-IDENTICAL |
| 格式 | `git diff --check` | clean |
| 链接 | 围栏感知活动链接扫描 | 无断链 |
| 范围 | `git diff --name-only 1d8fba15c..HEAD` 对 crates/ auto/lib docs/specs/ experimental/ Cargo 探针 | 零触碰（SD-08 为提案，canonical 未动） |
| CLI 残留 | 报告目录裸 ac-probe 扫描 | 0 |

## R3-QA → 修复对照（after 状态）

| 发现 | 修复证据 |
|---|---|
| R3-QA-01 消费者组合 traceback | decisions_by_id 只收通过全部类型/语义校验的决定（validated-only 索引）；MD-REVIEW-900 evidence=17 组合路径 → ERROR[decision-malformed] MD-S-900 定位 + family-ref-missing"未通过校验"，零 traceback（测试锁定） |
| R2-QA-02 旁路 | evidence-only 适用性：hash-only 与 subject-only 两个旁路负例均 family-ref-inapplicable（各自隔离的专用决定测试）；真实七族 evidence 均覆盖族路径，严格门绿（无需盲换） |
| R3-QA-03 导入/参数遮蔽 | import_shadow：4 个 qualified 全部 unknown-receiver、print=imported-symbol；parameter_shadow：IO(参数).read_line=unknown；let/var 绑定同名同此；无冲突宿主正例（List/IO/File/process）不变；真实仓观察零漂移（13 native/687 unknown 不变） |
| R3-QA-04 生命周期 | 重复 R4 标题去重（1 处）；两个起草态标题标注历史（计划文件）；final_assertions.py 交付（归档 27/27、全部 P743-* 指针、P743-3 archive 对账、plans.md 行、canonical、严格门七类断言，merge 收口执行）；P743-3 指针随重开恢复可解析、P743-4/5 留 merge 统一对账（合同既定） |

## 冻结哈希（实现 HEAD @ 本记录提交）

| 制品 | SHA-256（前 16） |
|---|---|
| proposed-spec-delta-phase4.md（SD-08 冻结提案） | f8924d0a8266976d |
| manual-decisions.json（r4 无内容变更） | 3674c687647f88dc |
| acc_inventory.py | 074a32853df94b7f |
| test_acc_inventory.py | 9f9b3681c2b5c029 |
| final_assertions.py | a8e629f87fa84b3d |

## AC-21..24 对账摘要

- AC-21 ✓：被引用决定的类型/缺字段/重复 ID 组合受控拒绝（定位 ID/族/原因），零 traceback；有效完整层绿。
- AC-22 ✓：hash-only/subject-only 旁路负例拒绝；evidence+binding 三段式通过；真实七族及完整层严格绿。
- AC-23 ✓：导入/参数/let-var 绑定同名不误判 native；未确定身份保持 unknown；无冲突宿主、当前 owner、既有修复正例不退化；真实观察零漂移，未硬编码数量。
- AC-24 ✓：T-21 历史补勾（激活提交）、T-22..26 进度真实；83 测试/确定性/严格门/链接/范围证据齐全；SD-08 冻结 f8924d0a 待独立 review 后 merge；final_assertions.py 交付待 merge 执行（27/27 与全部指针断言）。
