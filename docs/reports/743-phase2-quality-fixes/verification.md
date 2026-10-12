# PLAN-743 Phase 2 修复终验记录（T-13）

> 实施分支 plan-743-dev（基点=激活提交 c6e4e5689）；Category A，未跑 cargo/docs_gen。
> 复审反例原文与复审基线见 docs/reports/743-quality-review-20261005/（只读）。

## §6 Phase 2 门槛命令结果（全部在实现 HEAD 实测）

| 门 | 命令 | 结果 |
|---|---|---|
| 单元/反例 | `python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py` | **41/41 OK**（18 旧 + 23 新：QA-01 五例、QA-02 五例、QA-03 十三门 + 分类政策更新） |
| 生成 | `--write` | exit0；9 受管输入全记录 |
| 严格校验 | `--check --require-decisions` | exit0：47 决定绑定新鲜；严格门覆盖闭环（输入 9 / unknown 候选 146 组 ↔ 7 族双向闭环） |
| 确定性 | 双独立输出目录 + 仓内制品三方 `cmp` | BYTE-IDENTICAL（HEAD 仅审计字段，免疫） |
| 格式 | `git diff --check` | clean |
| 链接 | 围栏感知活动链接扫描（排除冻结提案正文的未来文件引用） | 无断链 |
| 范围 | `git diff --name-only c6e4e5689..HEAD` 对 crates/ auto/lib auto/aavm docs/specs/ experimental/ Cargo 探针 | 零触碰 |
| 旧保护回归 | 注释/字符串伪代码、注册表漂移、缺文件、篡改 manifest、人工结论过期、输出不安全 | 全部保持（18 旧测试含） |

## QA → 修复对照

| QA | 修复证据 |
|---|---|
| QA-01 跨行泄漏 | mask 状态机跨行保持字面量态（转义续行/EOF 兜底/char 换行恢复三态分立）；pretend 反例零泄漏；engine.at 314/333 识别为合法跨行（无异常、非不确定）；五新测试锁定 |
| QA-02 分类越权 | native 仅限 NATIVE_NAMESPACES；变量接收者一律 unknown；Meter.new/CG.new/Ar.new=type-qualified（真实仓断言）；dot/pipe 隐式 self 仅经本地方法集升级；五新测试锁定 |
| QA-03 人工层门 | schema/必填/唯一 ID/合法结论/非空绑定/证据存在全校验；--require-decisions 严格门 + unknown_families 双向闭环 + open 族 owner/probe/work_package 强制；scan-only 明示非完成态；十三新测试锁定 |
| QA-04 trap 边界 | 合同 eval.const-fold v2：运行期语境溢出一律保持运行期 trap，comptime 域出外；R3 分析/语义两分（双向禁止）；X9 反例入册 |
| QA-05 锚点/候选 | A2 HIR/源码分态；A8 owner=运行时首批候选、A9 owner=聚合能力线（未编号）；依赖单向无环声明 |
| QA-06 输入新鲜度 | 8 决定/9 绑定逐条审定（维持结论+具体依据）后重绑；MD-407/506 按 QA-01 修正更新；新增 MD-507 变量接收者政策；严格门绿 |
| QA-07 簿记 | r2 合同激活时已校正 T-05/06 历史勾选；本 Phase 进度见计划 §8 |

## 冻结哈希（实现 HEAD @ 本记录提交）

| 制品 | SHA-256（前 16） |
|---|---|
| proposed-spec-delta-phase2.md（SD-04/05 冻结提案） | a963adfbd6b24887 |
| manual-decisions.json | 3674c687647f88dc |
| source-manifest.json | b06d576845c6dd1b |
| auto-acc-bootstrap-contract.md | 676521ab3b9f26ba |
| acceptance-matrix.md | 5b2360c39930e218 |
| next-work-packages.md | da6806c312d56007 |
| inventory.md | bd9fbc1829fc1419 |

## AC-09..14 对账摘要

- AC-09 ✓：多行/同名/真实仓断言逐项有测试与 manifest 证据。
- AC-10 ✓：13 个严格门负例 + scan-only 区分 + --write 不触人工层（既有字节不变验证）。
- AC-11 ✓：X9 新反例 + eval.const-fold v2 + R3 两分 + 实现状态边界；无优化器实现。
- AC-12 ✓：10 锚点全保留；A2 分态；A8/A9 具名 owner；候选无环；不占新号。
- AC-13 ✓：9 绑定逐条审定重绑（附 741 r2/r3 差异依据）；严格 --check 当前真实报告绿；
  Spec 落地后的再次重绑留给 merge（合同 §5.6.6）。
- AC-14 ✓：T-01..06 历史勾选 r2 合同已校正；本记录 + 计划 §8 进度真实；冻结哈希在案。
