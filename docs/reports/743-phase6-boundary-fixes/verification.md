# PLAN-743 Phase 6 修复终验记录（T-37）

> 实施分支 plan-743-dev（基点=激活提交 bf899da0b）；PYTHONUTF8=1，Category A。
> 外部复审反例原文：docs/reports/743-r5-quality-review-20261006/（只读，含冻结 SD-10
> 825b4ae907ac2c26 与 probe_edges/probe_classification 运行器）。

## Phase 6 门禁结果（实现 HEAD 实测）

| 门 | 命令/方法 | 结果 |
|---|---|---|
| 单元+反例 | `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py` | **128/128 OK**（110 旧 + 18 新：R5-QA-01 三段式、R5-QA-02 双顺序整组作废、R5-QA-03 遮蔽/对照） |
| 生成 | `--write` ×2 | exit0；manual-decisions.json 字节不变 |
| 严格校验 | `--check --require-decisions` | exit0：47 决定绑定新鲜（r5 时点重绑保留）；146 组/7 族闭环 |
| 确定性 | 双独立输出 + 仓内制品三方 cmp | BYTE-IDENTICAL |
| 格式/链接 | `git diff --check` + 围栏感知活动链接扫描 | clean / 无断链 |
| 范围 | `git diff --name-only bf899da0b..HEAD` 对 crates/ auto/lib docs/specs/ experimental/ Cargo 探针 | 零触碰（SD-10 为提案，canonical 未动） |
| 真实漂移 | r5→r6 分类观察对比 | type-qualified 8 / type-construction 25 / native 13 / unknown 687 全部不变（真实仓无参数-本地类型同名冲突） |

## R5-QA → 修复对照（after 状态）

| 发现 | 修复证据 |
|---|---|
| R5-QA-01 缺失绑定文件仍入消费者 | `not f.is_file()` 分支补 record_bad=True：绑定文件删除后该记录退出消费者索引（CLI exit1 + family-ref-missing"未通过校验"）；三段式存在/删除/恢复永久测试锁定（commit bf629ee43） |
| R5-QA-02 错类型同 ID 绕过唯一性 | 全局 ID 注册移到类型门前：可辨识非空字符串 ID 全覆盖；错类型/缺字段记录同样 claim ID → 同 ID 合法记录整组作废（bad-first/bad-last 双序消费 0，family-ref-missing）；非法 ID 类型不 hash 不参与去重（commit bf629ee43） |
| R5-QA-03 绑定让位于本地类型 | _post_classify qualified 与 bare 分支均把 known_bindings/imported 检查提到 local_types/enum/type-construction/BARE_NATIVES 之前：参数名与本地类型同名 → 全 unknown；无绑定对照合法升级保留；真实仓零漂移（commit 9d5bbc85d） |
| 前置 5 条 stale（741 P4/P5） | r5 时点已对 741 P4 diff 逐条审定重绑；741 P5 落地后于 merge 阶段再次逐条审定（结论全维持，rationale 在 manual note） |

## 冻结哈希（实现 HEAD @ 本记录提交）

| 制品 | SHA-256（前 16） |
|---|---|
| proposed-spec-delta-phase6.md（SD-10 冻结提案，复审方交付） | 825b4ae907ac2c26 |
| manual-decisions.json（r5 时点重绑保留） | 45e04e309fbaeeab |
| acc_inventory.py | （本记录提交时实测，见 verification 提交） |
| test_acc_inventory.py | （同上） |

## AC-29..31 对账摘要

- AC-29 ✓：额外绑定文件存在/恢复对照 CLI0 可消费；删除/缺失受控非零且消费者读取 0、
  族覆盖 0；错类型/缺字段同 ID 前/后插整组消费者 0；合法 duplicate 两顺序、非法 ID
  类型、单独坏记录、全部旧负例不退化（126→128 测试）。
- AC-30 ✓：普通/mut、自由函数/方法参数与本地 type 同名时 bare 构造和 qualified 均
  unknown；无冲突本地 type/宿主/当前 owner/原四参数/多行保护正例维持；真实观察零
  漂移，未硬编码数量，未实现 resolver。
- AC-31 ✓：110 旧+18 新、独立反例、真实 strict、双生成/manual 字节/三方一致、
  源指纹/SD-10 冻结 825b4ae9/范围/actual 链接与 38 任务 metadata 有 r6 新证据；
  独立 review 待执行（本记录不预写 pass）；actual archive 38/38 由 merge 收口。
