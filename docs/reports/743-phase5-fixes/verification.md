# PLAN-743 Phase 5 修复终验记录（T-33）

> 实施分支 plan-743-dev（基点=激活提交 39a4f9133）；PYTHONUTF8=1，Category A。
> 外部复审反例原文：docs/reports/743-r4-quality-review-20261005/（只读，含冻结 SD-09）。

## Phase 5 门禁结果（实现 HEAD 实测）

| 门 | 命令/方法 | 结果 |
|---|---|---|
| 单元+反例 | `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py` | **110/110 OK**（83 旧 + 27 新：R4-QA-01 六 fixture、R4-QA-03 三组合、R4-QA-02 五形状） |
| 生成 | `--write` ×2 | exit0；manual-decisions.json 字节不变 |
| 严格校验 | `--check --require-decisions` | exit0：47 决定绑定新鲜（5 条 AutoAC stale 经 741-P4 diff 逐条审定后重绑）；146 组/7 族闭环（含 evidence-only 适用性与 validated-only 消费者） |
| 确定性 | 双独立输出 + 仓内制品三方 cmp | BYTE-IDENTICAL |
| 格式 | `git diff --check` | clean |
| 链接 | 围栏感知活动链接扫描（工作树活动态） | 无断链 |
| 范围 | `git diff --name-only 39a4f9133..HEAD` 对 crates/ auto/lib docs/specs/ experimental/ Cargo 探针 | 零触碰（SD-09 为提案，canonical 未动） |
| CLI 残留 | 报告目录裸 ac-probe 扫描 | 0 |

## R4-QA → 修复对照（after 状态）

| 发现 | 修复证据 |
|---|---|
| R4-QA-01 参数遮蔽不完整 | 签名提取覆盖自由 fn + 类型体方法（static 同样）；mut 参数名取 mut 后 token（demo(mut IO Meter)→名=IO）；BARE_NATIVES 判定推迟到 post-classify 且参数/局部绑定遮蔽优先：四反例（mut/method/bare/method-bare）全部不升级 native，usual_parameter 对照仍 unknown、conflict_free 对照仍 native；真实仓宿主正例（List/IO/File/process）不变（commit 739c7aa57） |
| R4-QA-02 manifest 形状崩溃 | --check 容器形状门：顶层非对象/source_identity 非字典/inputs 非列表 → ERROR[manifest-malformed] 受控拒绝、零 traceback；inputs/manual 字节不变；audit-only HEAD 归一行为保留（commit 739c7aa57） |
| R4-QA-03 消费者索引语义 | validated-only 索引扩充：非法 conclusion、证据缺失/未绑定、绑定过期的记录一律不入消费者索引；重复 ID 整组作废（先插入者移除）；族引用被淘汰记录报 family-ref-missing"未通过校验/不存在"（commit 739c7aa57） |
| R4-QA-04 归档链接/断言 | final_assertions r5：33 任务双格式、计数 33/33、收据 key PLAN-743:r5、Markdown 相对链接自实际父目录解析（围栏排除）、P743-* 全指针、canonical（r3 摘要消失+r4 证据约束+r5 绑定优先规则）、严格门（commit efeb03a60）；归档态 6 断链由 merge 归档前统一改 `../../reports/` 后由该断言复核 |
| 并行 5 条 stale | 741 P4 diff（link.cleanup 事务清理/CREATE_SUSPENDED 包含/COMMITTED 状态）逐条审定：五结论全维持，spec-only 重绑附 rationale；源码绑定未触碰 |

## 冻结哈希（实现 HEAD @ 本记录提交）

| 制品 | SHA-256（前 16） |
|---|---|
| proposed-spec-delta-phase5.md（SD-09 冻结提案，复审方交付） | ad3e02d7a6c237f8 |
| manual-decisions.json（r5 经 5 条重审重绑） | 45e04e309fbaeeab |
| acc_inventory.py | 9d315e657a2cd1a0 |
| test_acc_inventory.py | 85c6a63d40fea447 |

## AC-25..28 对账摘要

- AC-25 ✓：非法语义/过期绑定/重复 ID（前后顺序）不入消费者索引（family-ref-missing 定位）；
  受控非零零 traceback；完整合法层 pass。
- AC-26 ✓：mut/free/method/bare 四反例不升级 native；普通参数、导入/局部绑定、合法 owner、
  无冲突宿主正例保留；真实漂移零（13 native 不变），无机械 hash 重刷。
- AC-27 ✓：manifest []/1、source_identity null/[] 受控 ERROR/非零/无 traceback；无输入/
  manual 修改；audit-only HEAD 与漂移检出行为保留。
- AC-28 ✓：110 测试、严格门、确定性、manual/hash/links/diff/范围验证完成；SD-09 冻结
  ad3e02d7 一致；独立 review 待执行（本记录不预写 pass）；final_assertions r5 交付，
  merge 归档收口时执行（33/33、链接、全部 P743-* 指针）。
