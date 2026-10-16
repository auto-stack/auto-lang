# PLAN-800 final review R1

- stage: review; review_scope: final; outcome: needs_fix; plan_revision: 1
- Reviewer: 当前独立审核对话，未参与实现；未调用额外 agent，未修实现。
- worktree: D:/autostack/.wt/lang-800/auto-lang; branch: plan-800-dev
- reviewed_commit: 5c1f43b5dfebbc5fdffa891046a343ccd902ef79
- base_commit: b0d6d4aab50bfcb137d55b0f88e9f2bd82b8f644
- current main: 949ae72874dfec562da09839458810c1d195ada7
- dependency: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8
- 实现工作树 escalated git status --short 空；审核结束再次核对 HEAD/status。沙箱内 git/status 无法读取 worktree admin、测试无法写 target，已用获批同树调用完成；不视为代码失败。

## Findings

### F800-R1-01 [P2] SD-01 声称不存在的错误映射（AC-04/05, T-06/07）

`docs/specs/package-resolver/project.md:62-65` 声称 adapter 将 PackagePathNotFound 映射回 AutoManError::FileNotFound；实际 `crates/auto-man/src/resolver.rs:142,156,190` 只有 `if let Ok`，完全忽略核心错误，唯一 map_err 位于读取文件的 Io 转换。核心 lib.rs:61-63 文档、t01-freeze §1/2、t06-work-receipt §2 同样宣称映射存在。私有旧入口已删除，现状无可观察行为回归；但不能将不存在的路径沉淀为规范。按真实边界统一修正文档/收据（旧私有错误已不可达、生产调用仍静默忽略找包失败），或有依据地实现描述的转换；不要求为私有不可观察错误添加无意义代码。此为知识增量准确性缺口，不是新增功能要求。修后冻结新 SD hash。

### F800-R1-02 [P3] 新增核心未通过格式门（AC-05, T-06/07）

`rustfmt --check --edition 2021 crates/auto-package-resolver/src/lib.rs` exit 1；341/406/483/516/549/576 附近存在确定性格式差异，详见 rustfmt.txt。仓库独立审核要求 clean formatting。格式化新增核心/fixture及此次改动范围，避免全仓无关格式改写；提交后复审。不得为此运行整档。

## AC → tasks → evidence

| AC | 结果 | 证据 |
|---|---|---|
| 01 → T02/T05 | pass | core manifest 零 production 依赖；consumer 7/7；verify-deps.sh PASS，仅 core 的 normal/build 闭包 |
| 02 → T01/T03/T04 | pass | 旧/新算法逐臂核对；核心14/14；auto-man resolver:: 38 passed/1预存ignored；独立 counterexamples.rs 通过 |
| 03 → T03/T04 | pass | adapter 全部公开入口保留；唯一 map 在 core；同名覆盖 path/mode 立即可见；ModulePath/ExecutionMode/legacy扫描留 adapter；无复制780/795算法 |
| 04 → T05/T06 | partial | 净搬迁/未迁清单齐；SD 存 F01；792当前已归档，详下 |
| 05 → T07 | fail | F01/F02，独立验收主体已完成，须修复收口后复审 |
| 06 → T08 | pending | merge-owned，未执行合并/归档/整档门 |

测试命令（同一冻结树）：
- cargo test -p auto-package-resolver：14 passed；doc tests 0。
- cargo test -p auto-man --lib resolver::：38 passed/1 ignored。筛串同时选中9条 builder::ninja::resolver 测试，因此不能将38全部称作 Man adapter 测试。原报告总数真实；auto-man lib tests 36 warnings及语言依赖告警来自未改旧代码，本次新增核心/adapter无新告警。
- cargo test --manifest-path test/fixtures/modularity/package-resolver-consumer/Cargo.toml：7 passed。
- Git Bash 显式路径运行 verify-deps.sh：PASS；cargo tree --edges normal,build 仅 consumer → auto-package-resolver。
- python scripts/modularity-audit.py：exit 0，metadata cross_check ok/zero discrepancies；无新禁向边。
- git diff --check b0d6d4aab HEAD：exit 0。
- 独立探针 counterexamples.rs：插入序先目录后下一搜索路径文件、同路径file优先、dependency压过search、std压过同名dependency、exists std缺文件后仍可命中同名dependency、with_extension dotted行为、缺项/未声明完整错误逐字控制，全部通过。初版 oracle 用带斜线字符串join导致Windows期望文本分隔符不同，已修正oracle为逐组件join后重跑通过，无实现变更。

## 当前合流核对

792 已合入（与 work 时点不同）；main rust_ui.rs DELIVERY_CRATES 为10项，不含 auto-man；全部闭包包清单无 auto-man→auto-package-resolver 这条上层依赖。因此本新包不进入792交付闭包，不需要改producer清单或运行不相关重型交付构建；保留merge时同步后metadata核对义务。794接口未触及，799旧AST路径兼容保持。主检出新Spec增量在merge时应按当前版本合并，不能覆盖792新增章节。

## Spec evidence / freeze

- main docs/specs/auto-man/project.md SHA256 AA785B1DBFBCBCF5E52E9075BF6EBF6FFAEE0F9BED7BB51F279D674525D16827
- SD01 frozen sd01-package-resolver.md SHA256 AF3B753425BDC3BB8A3FAFE3A665B7D4723E9438D39E3ABCBE421D356EC99DBD
- SD02 frozen sd02-auto-man.md SHA256 8FCA03939E763C3B44182AE90D2931CD02F0FF571E96EA4A6ABA582EF0AC725D
- metadata路径及 GOAL-003/016保留；未发布canonical/ledger。扫描无未批准延后/实现workaround；F01/02列为本合同待修，不能债务化豁免。

pending_closeout_gates: merge同步/必要受影响复审；冻结裸cargo t、两crate定向+consumer门；Spec/ledger沉淀；guard/归档清理；800%5到期主检出单实例regress。
next: /auto-plan:work 修F01/02并提交，同一工作树送定向复审；不在review修代码或跑整档。
