# PLAN-797 final 就绪复核 R2

2026-10-16（Asia/Shanghai）；**needs_fix / executing 5/9**。这是 R1 修复后的就绪复核，未进入完整代码验收，不计作第二次完整代码验收失败。没有新增实现 finding；F-797-R1-03 独立确认关闭，F-797-R1-02 部分关闭，F-797-R1-01 仍 OPEN。

## 基线

- stage: review；review_scope: final（readiness only）。reviewer: 当前 Codex 复审聊天，未实施本分支、未创建代理，具体模型/session 身份未另行暴露。
- plan_id: PLAN-797；plan_revision: 1；主检出 `32743f07b89c130f8834941583511f7fe45230a3`。
- 入审主检出 Plan SHA256: `207076DAF5A01EB724586ACABADF1E68F6DB0DED2F559E9EA8FCEA3F7542EAF8`。
- worktree_path: `D:/autostack/.wt/lang-797/auto-lang`；branch: `plan-797-dev`；reviewed_commit: `635053f74aa49f89d74ffef6db9c70df7c363c06`。
- base_commit / merge-base(master, plan-797-dev): `5dcf4e3c2d5266cfa4ebf1444d6d7e154305d3c4`；R1 commit: `defcc8eaa1fdcfa619feda99721d0b93c25035fb`。
- `git worktree list --porcelain` 确认同一实施树；入审及收尾 `git -C ... status --short` 均空，收尾 HEAD 未变。只读 Git/metadata 经工具批准在现有树核对，未修改实施代码。
- R1→R2 diff：新增锁文件、验收脚本、报告/Spec 草稿/并行审计；唯一既有 Rust 变化是 emit.rs 删除一个多余 mut。ac-core/783 路径依赖、manifest、原有测试均未变；其 tree 身份保持 R1 报告所录值。依赖解析的独立验证见 F03。

## 旧 finding 闭环

| finding / AC / tasks | 本轮证据 | 结论 / 下一步 |
|---|---|---|
| F-797-R1-03 / AC-01–05 依赖基线 / T-01 | HEAD 中 Cargo.lock blob `598833812a800c9d71574b5cf345f8583c082b88`；实际 `git hash-object` 与其相同；SHA256 `C7EC8DD5A8B49B0DDF510C5B470FC5C6DA873B2AB66B73009CCACED32F93ABC6`，与 R1 本机锁相同。`cargo metadata --manifest-path .../Cargo.toml --locked --offline --format-version 1` 最终 exit 0，解析 120 packages、Cranelift codegen 0.126.2。 | **CLOSED**。锁交付/冻结解析结果已补齐；保留 T-01 完成。没有把 work 的 check/test 绿直接当作本轮代码验收。 |
| F-797-R1-02 / AC-06/07 / T-07 | HEAD 已有 `scripts/verify-autoabi-797.ps1`、`docs/reports/plan797/{freeze-proposal,support-matrix,r1-repair-baseline}.md`、`docs/plans/attachments/797/{spec-delta,source-parallel-audit}.md`。spec-delta 开头明确草稿、待 T-06 重写实际通路并冻结，T-07 行仍未勾选。 | **PARTIAL / OPEN**。文件缺失子项关闭；正式通路证据、V-01–11 终收口、支持/SD/freeze 终稿仍待完成。 |
| F-797-R1-01 / AC-05/06 / T-06 | semref.rs 保持 `autoabi-semref-snapshot/0` / `temporary-797-local`；`.case.ad:3–10` 仍明确 execution pending。Plan §8/§9 和修复基线报告同样承认 T-06 未实施。796 HEAD `d85f89fc7584d40146d026e3c79fe0db18b2a78e`、788 HEAD `720df159c656c1acc8ac32e94458060a03f5f39b` 对当前 master 的 ancestry 检查均 exit 1；两计划均 executing 5/9、5/10。 | **OPEN**。D-796/D-788 未满足；实际合入/Spec 完整后在同树完成真实语义绑定、快照退出、正式 AutoDown/兼容反控及性能复测，再做 T-07。 |

metadata 初次在默认 RUSTC_WRAPPER 下 exit 101：sccache 启动服务遭 Windows 端口排除、os error 10013，尚未解析依赖。仅在该次进程临时清空 RUSTC_WRAPPER（finally 恢复），相同 locked/offline 命令重试成功。不改 Cargo 配置、不启动测试，不把环境错误算产品回归，也不隐去失败。

## AC 覆盖与验收脚本

- AC-01–04：本轮仅完成依赖冻结/文件完整性复核；完整 contract/layout/native/resource/cache 行为仍未独立认证。R1 没有独立测试通过证据可供“复用为 final pass”；work 历史证据保留。
- AC-05：**partial，完成性 fail**。正式 adapter 的硬周转与全阶段资源证据尚无。
- AC-06：**fail**，F01/F02 仍在合同内，不能靠登记债务批准延期。
- AC-07：**partial**，已做就绪复核，T-08 不勾选；AC-08：merge-owned **pending**。
- 新脚本 `-Scope acceptance` 注释限定为依赖门前的 probe 测试和材料存在性；`-Scope closeout` 明确是 dry checklist，写死 V-09 partial/V-10 pending，最后的 `ALL PASS` 与 exit 0 不能解释成 AC-05/06 或 final review 通过。完整 final review 前仍须补齐实际合同要求的验收；本轮不运行该脚本重复整套 probe 验证。
- `git diff --check <R1> HEAD` exit 0。没有运行 Rust check/test、C 编译/PE、性能或整档。完整健康检查及前五项高风险路径矩阵留待具备就绪条件后验收；没有声称零其他缺陷。

## Spec 输入与草稿身份

本轮读取的 `spec-delta.md` 已提交于受审 HEAD，SHA256 `B74BDD3552E407078878833844D3A081E528E96667249CE7BF98ACC45E7AB5EF`。这是已冻结身份的**草稿**，不是已批准的最终 delta；本轮不改 canonical Specs/ledger。

主检出 canonical 输入 `auto-ac/{project,data-resource-contract,artifact-contract}.md`、`auto-cache/project.md`、`auto-test-case/project.md` 的 SHA256 分别仍为 R1 报告列出的 `0040541B…`、`85B8074B…`、`B4378653…`、`7A48BA94…`、`FC37B5EC…`，逐文件重新计算一致。`docs/specs/goals.md` 当前 SHA256 `EC5170B6B769E33F24735E35541A071EE6E499F09D5BD328F47BF64B6F1E2D7D`，已不同于 R1；终稿必须核对当前输入，不能盲用草稿中“hash 以 R1 报告为准”。保留 supersedes/new paths 与 `touched_goals: []`，最终知识增量/目标影响审查 pending，未批准语义合同变更。

## 路由

next: **work**。维持 `executing 5/9`，T-01–05 完成标记保留，T-06/07 未完成，T-08/09 pending。先按原有依赖门完成 T-06，再收口 T-07；不要把补齐可独立材料等同于“实施完毕”。遵循 auto-plan-review 的 “Known unfinished work-owned criteria or required oracles return to work” 入口规则，不再重复完整 final-review 周期来确认已承认的未完项。

pending_closeout_gates: 最终代码/验收/Spec review 通过后，merge 冻结提交执行 ABI 完整套件/真实 C 三腿与受影响 796/788 定向兼容；根 cargo t 和 tv/tt 等按最终实际 diff 定界，tf 为主检出到期批量。合入、Spec/ledger、归档和 guard 清理全部 pending。本轮只写共享 Plan/复审报告/债务记录，不修代码、不合入。
