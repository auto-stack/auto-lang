
### 2026-10-27 独立最终复审 R17

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/验收覆盖审读，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: a26b8e71c3760e0538cb654d7c39e5a548c3f104 | bookkeeping_commit: 7fe3e2fc8 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: f1d31632b1e625deb125142864e1ad032cb9870d

F-822-R16-01 CLOSED：decode后强制profile id4再verify_abc，通用旧profile入口不变；独立运行cache_compat八控8/8绿，包括原core混入拒绝、同形scalar collections正控及R14/R15错配/诚实恢复。actual profile→family/version由decode合法tuple检查，owner重盖章已核，types_fp/entry/snapshot各有真实内容比较；未发现本次恢复修复新回归。

继续核对历次保留pending的AC-09/V-13子case3，发现 F-822-R17-01(P1，原必需验收证据缺口，非R16引入；AC-09/T-09/T-06，SD-06)：原§5全链增量及V-13明确要求“实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控”。现有822 incremental全文件只有signature_change_precise覆盖接口由i32→bool的link拒绝、body_edit_identity覆盖beta本体更新/调用语义和旧snapshot保活；未提供collections能力的实际layout事实变化及host授权事实变化正反控，script incremental必需清单亦无这两轴。独立运行signature_change_precise=1/1绿，只能证明签名轴，不能代表V-13子case3全覆盖。共享828 source helper/body测试不自动覆盖822新增集合layout，字符串含authorize的manifest/重复身份测试也不是host grant变化oracle。

覆盖核验方法（可复现，无新规模承诺）：审读 probe/tests/av_collections_822_incremental.rs（202 body/263 signature/303 forced/343 baseline等8测）、probe/src/collections_session.rs及scripts/verify-av-collections-822.ps1必需清单；搜索相关共享function_incremental_828*.rs的layout/grant/authoriz/helper/capabil引用并检查实际语义。session仅接submitted MIR，默认ExecLimits/CancelFlag执行，没有本计划授权变化的测试输入/观察；此点只确认当前验收缺口，未据此声称已复现越权或错误layout重用。受检新layout/host授权的生产接线是否缺失需work定向调查；不以未验证的处方当根因。

关闭oracle：对既定真实collections consumer，提供actual layout相关依赖变化的合法/不兼容控制（精确失效或受控拒绝，prior snapshot/值/清理、未影响单元身份均可观察）；提供输入required capabilities与当前host grants分开的实际变化控制（合法允许、撤销/不足拒绝，失败不安装，恢复合法事实仍可用），或定位已存在且适用于本profile/当前baseline的等价耐久证据。保既有signature/body/forced控制。不能仅增测试名/改清单或以scalar campaign代替。如当前架构不能满足原承诺，回new显式合同修订，不在work默默标N/A。仅要求受影响定向验收，不跑整档或重放无关性能矩阵。

acceptance_results: AC-06本轮R14..16恢复身份面pass（其他兼容证据保原边界）；AC-09 partial/fail（子case1/2/4沿既有证据，子case3签名/body有证据但layout/授权未闭）；AC-07 fail；AC-01..05沿既有证据边界，AC-08 merge-owned pending。SD-03本轮恢复绑定主张可接受，SD-06完整增量依赖主张待关闭F-R17-01；其余SD保持原边界，无canonical/ledger写入。冻结SD hash见input-hashes，touched_goals空理由沿计划有限实验扩面保留。

next: 同树work补F-822-R17-01调查及必需oracle，T-09/T-06重开，executing5/9，其余任务保留（T-05已关闭，不再重开）；定向提交再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门仍pending，tf仅主检出批量。未修实现、整档/aggregate acceptance、合入、归档或清树。
