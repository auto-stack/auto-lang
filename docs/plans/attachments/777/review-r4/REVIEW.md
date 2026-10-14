# PLAN-777 第四轮独立 final review

结论：**pass**。本审面没有新的阻断finding；R1–R3全部合同内findings闭合。Plan进入reviewed / 5/6，T-06 merge仍pending，尚未交付/合入/归档。

## 审面、独立性和输入

- plan_revision=1；review_scope=final；worktree `D:/autostack/.wt/lang-777/auto-lang` / branch `plan-777-dev`。
- reviewed_commit=`5e1794bcc6f1b811a2b8d7a224d12fac9997f829`；base=`17bc8431457848b040bffa5c261e13b057771559`；冻结基线提交=`677055420203db3bc3a868a2dca697f8167361dd`，其base_revision=`d6e9479124d7aa11a92d5c63cc3df30874c2af31`。
- 主检出=`92d0ed6267254e8ed7cea556f8886eb92c442ecf`；组内auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；Python3.14.2/Cargo1.98.0。
- 本评审会话参与规划及前轮审查，未参与实现；直接从提交、原反例及独立组合验证重建结论，没有另建agent/树。
- Spec目标源SHA256=`3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b`。SD-01具体持久规则已冻结在 `spec-delta-reviewed.md`，与当前canonical布局无冲突；本轮不发布Spec/ledger。
- 受管输入完整hash、Cargo.lock、HEAD、branch和实施树status在运行前后与判定前复查一致；依赖提交/manifest核查见 `fingerprints.jsonl`。实施树干净。所有变异及硬链接仅在自有临时副本，真实实施树无代码改写或链接创建。

本轮只改脚本、不改配置或声明边。没有重冻基线是合理的：锚定manifest/配置未变，config_sha一致，真实strict无staleness/config-drift；基线重冻并不是每次脚本修复的必要条件。

## finding闭合与修复组合

| finding | 本轮核验 |
|---|---|
| F777-R1-01 多target表复用 | 原例通过，build/dev+workspace继承别名组合仅Windows build UI边判红 |
| F777-R1-02 workspace继承path基点 | 原例和上述组合通过，Cargo声明oracle一致 |
| F777-R1-03 输出覆盖输入 | 配置/manifest/源码别名控制均拒绝且字节不变；只读不变量闭合于已约定输入面 |
| F777-R1-04 optional激活冒充存量 | 原例通过；完整声明键仍包含optional/scope/dep_key，无新豁免 |
| F777-R1-05 ownership遗漏 | 27行/7组及6条补登记路径通过，owner/退出阶段与调研一致 |
| F777-R1-06 外部SDK误红 | 合法SDK通过；scope-only旧例与生产方向规则正控分开，不用旧例代替真实配置检查 |
| F777-R2-01 外置auto-ui映射遗漏 | 实际生产域表/规则注入：UI、Man禁向exit2，合法SDK exit0；v10组合通过 |
| F777-R2-02 外部manifest覆盖 | 原例exit3/输入hash不变；config/外部manifest以samefile比较身份 |
| F777-R3-01 tracked源码硬链接漏防 | 原矩阵4/4；源码硬链接exit3/输入hash不变，新增身份比较遍历全部tracked输入与读取的锁 |

对 `_identity_hit` 新路径独立选择额外3个组合：有效未受管Cargo.lock的根外硬链接、受管非Rust资产的根外硬链接、已存在但不属于输入的独立报告。前两项exit3/原hash不变，后一项exit0/报告有效/源码不变；避免以“一律拒绝已有外部输出”伪证修复。见 `identity-combinations.py` / `.jsonl`。有效锁仅在临时fixture中通过cargo generate-lockfile --offline生成，生产锁未改变；无编译/测试。

此前连续失败的诊断已对账：工作从具体旧例扩展为域/输入身份矩阵，本轮身份比较确实接入所有tracked类别与读取的锁。未发现未批准延期、删验收项或新增豁免。文本候选不是完整语义解析、未知外包仍unclassified、锁对照gap可告警等原有边界继续明确，不外推到探针/完整拆仓。

## AC→task→evidence

| AC | 结果 | task与验证 |
|---|---|---|
| AC-01 | pass | T01/T04；help、真实双扫同字节、2052个受管Rust/945894物理行、报告revision/hash/method、全输入前后不变、身份正反控制 |
| AC-02 | pass | T02/T03/T04；旧7控、R2补充5控，窄豁免/撤销/过期/宽豁免、optional激活、生产跨根方向与合法SDK对照；36声明边与cargo metadata --locked零差异 |
| AC-03 | pass | T02/T03；27行/7必需组/路径核对、资产consumer归属及启发式候选分类限制检查 |
| AC-04 | pass | T05；独立提交/原反例/组合/Specdelta/遗漏健康检查及判定前指纹复查 |
| AC-05 | pending / merge-owned | T06；实际Spec/模块导航/ledger/index沉淀、合流、归档、批回归到期判定、wt-guard clean后移除树 |

独立验收：self-test **15/15**；旧replay **7/7**；R2 supplemental **5/5**；R3 input-matrix **4/4**；新identity combinations **3/3**，所有控制脚本整体exit0。原始命令/退出码/输出hash保存在 `primary-runs.jsonl`、controls/supplemental/input-matrix/identity-combinations.jsonl。

真实双observe字节SHA256=`6ce5bb4ae986d869ce38954c6e8a768f6f920007c4838d107afb95b81993cb16`（评审保存的Windows文本）；strict exit0/pass/0违例/2grandfathered，ownership coverage无缺组。配置、边集不变，Rust规模保持一致。`git diff --check`通过；Python脚本真实执行无警告/无非预期debug输出。没有Cargo编译或Rust测试。

## 冻结规范增量与后续门禁

SD01已审查pass，新增入口/口径/输入保护/声明边/责任与baseline持久规则；不把目标仓写成已拆。supersedes_spec_components=[docs/specs/design/repository-layout.md]，new_spec_components=[]，touched_goals=[]（无新增产品目标）。若merge时该Spec或代码/依赖有新变化，核对受影响delta/evidence，不能直接复用旧pass。

**Category A：无merge-owned Rust t/tv/tu/docs_gen整档门禁。** T06实际沉淀/合流/归档/投影与guard清理仍pending；批回归是否到期由merge按现行规则判定。最终review pass不代表已合入或交付。merge如改动审计代码/配置或锚定输入，复查受影响审面并运行新的真实strict/观察证据。

下一步 `/auto-plan:merge`，使用该冻结提交与SD01；本次用户请求仅review，本轮不merge、不归档、不清理树、不发布canonical Spec/ledger。781/782的其他WIP保留。
