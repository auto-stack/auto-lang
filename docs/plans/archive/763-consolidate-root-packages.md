---
plan_id: PLAN-763
plan_revision: 1
status: archived
feature_name: consolidate-root-packages
created_at: 2026-10-11
updated_at: 2026-10-11
supersedes_spec_components: ["docs/specs/aavm/design/corpus.md", "docs/specs/aavm/design/divergences.md", "docs/specs/aavm/design/file-mapping.md", "docs/specs/aavm/design/idiom-upgrade-prereqs.md", "docs/specs/aavm/design/lib-modularization-map.md", "docs/specs/aavm/design/m3-typecheck-format.md", "docs/specs/aavm/design/method-emission-spec.md", "docs/specs/aavm/project.md", "docs/specs/auto-lang/frontend/plans.md", "docs/specs/auto-lang/runtime/design/ffi-bridges.md", "docs/specs/auto-lang/runtime/overview.md", "docs/specs/auto-lang/ui/architecture.md", "docs/specs/auto-lang/ui/design/aura-pipeline.md", "docs/specs/auto-lang/ui/design/blueprint-tier.md", "docs/specs/auto-lang/ui/design/chart-components.md", "docs/specs/auto-lang/ui/design/tabs-components.md", "docs/specs/auto-lang/ui/overview.md", "docs/specs/auto-lang/ui/plans.md", "docs/specs/auto-man/project.md", "docs/specs/blueprint/contract.md", "docs/specs/blueprint/project.md", "docs/specs/design/repository-layout.md", "docs/specs/goals.md", "docs/specs/overview.md", "docs/specs/parity/project.md", "docs/specs/plans.md", "docs/specs/website/project.md", "docs/specs/widgets/menubar-family.md"]
new_spec_components: []
touched_goals: [GOAL-016]
affects: [aavm, blueprint, parity, website]
current_step: 5
total_steps: 5
---

# [PLAN-763] 根目录第二轮归并与未维护 CI 清理

## 变更摘要
用户已授权检查八个 Actions 工作流，用处不大则删除；其余目录按此前建议合并。本计划落实已有授权。保留历史提交、测试覆盖、标准库与本机配置。

## 目标
Git 根树从 30 项减为 24 项；完整保留五组资产及消费入口。

## 架构方案
auto/ → packages/aavm/；blueprints/ → packages/blueprints/；parity/ → test/parity/；schema/ → docs/schema/；patches/ → crates/patches/。stdlib 独立；逻辑模块名 auto.lib 与 bps 保持不变。隐藏工具目录保持自动发现位置。

## 需求分析与背景调查
依据 docs/specs/overview.md、design/repository-layout.md 及各模块现有实现。七个测试/构建流最近五次均失败，网站部署零次执行。删除八个旧定义，不新增或触发部署；本地验证命令仍可运行。提交历史能确认加入时间和关联计划，不能证明操作者是 AI。

## 详细设计
代码修改与验证在 D:/autostack/.wt/lang-763/auto-lang；主检出仅计划簿记。移动同步 Cargo patch、schema include、AAVM 清单、blueprint 发现与 pac 依赖、parity 仓根推导和脚本。检查活跃仓内与仓外消费者；不修改别的会话检出，不删本机缓存。冻结历史计划/报告保留旧路径；现行 Specs 与入口采用新路径。

### 规范增量
修改 docs/specs/design/repository-layout.md：24 项布局、五处新归属、当前无 GitHub Actions 自动部署/测试，本地门禁继续执行。同步受影响模块现行路径与索引，不改变语言/组件/协议语义。账本按 docs/specs/README.md §5 兼容期回退做保守投影，保留所有无关条目。

## 测试设计
逐文件映射/哈希证明资产保存；扫描旧路径并逐项解释；cargo check -p auto-lang；cargo t、tv、tu、tt、tb 及 docs_gen 对应触面；parity 独立 workspace check/test 与 CLI 冒烟；AAVM 按 AGENTS 跑 taa（遵守重型档主检出单实例纪律）；blueprint 命令与示例生成冒烟；Cargo metadata 确认 patch 来源。按证据区分预存失败，不把未验收条目当通过。tf 合入时检查到期规则。

## 验收标准
- AC-01：八流程的来源、触发条件、远端记录与删除理由有持久报告；八定义删除，无自动发布替代。
- AC-02：五目录无丢件；Git 根树 24 项，stdlib 与隐藏共享配置保留。
- AC-03：编译、blueprint/schema/AAVM/parity 入口与 Cargo patch 消费新位置；依赖命名及资产语义保持。
- AC-04：验证、复审、规范投影证据绑定实际提交；预存问题独立记录。
- AC-05：线性合入、归档、guard 清理专属工作树；保护其他会话及本机缓存。

## 执行步骤
- [x] T-01：冻结八工作流来源与 GitHub 记录，形成 docs/reports/763-ci-audit.json。
- [x] T-02：迁移五目录，同步消费者，形成资产映射核验。
- [x] T-03：完成作用域验证与冒烟，保存结果和失败归因（taa 5 红全分诊：P735-R1/REG-2 已登记 2 枚 + AAVM 3 枚经基线 716ca62db 同滤串对照同红同形态=预存，763-baseline-taa.{json,log}；bp 冒烟 2 红为调用 cwd 工件、项目 cwd 复跑双绿，763-smoke-recheck.json；预存债登记 KNOWN-DEBT P763-R1/R2）。
- [x] T-04：提交实现，以 auto-plan:review 从产物重建验收结论（同会话复审，明确限制），审定 Spec delta。（212f222fb+62a7cf47a；outcome pass，见复审记录）
- [x] T-05：auto-plan:merge 完成规范/账本、ff-only 合入、归档与 guard 清理。（delivery 1202d2062，见合并收据）

## 复审记录
待执行（T-04）。基面 4aa461d1c0765436ab974817ee4ad7ebb8ba719c；外部依赖在工作树创建后记录。

T-03 失败归因收口（2026-10-11）：全档 taa 5 红=2 已登记（plan730_commit_target_matrix=P735-R1 时序敏感、ffi_dual_019=REG-2 并行 flake）+3 AAVM 预存（基线 worktree 716ca62db 同滤串 `cargo taa -E 'test(test_aavm2_goldens_check) | test(test_aavm2_compile)'` 3/3 同红同错误形态：goldens 同 file:line 同 `INVALID_BYTECODE: 60`、compile 双腿同 E0433 `auto_lang` 未解析——血统 db4e5954b，见 763-baseline-taa.json）。t 档首轮 shell_pack_hash 一次性 flake（后续 taa/daily-final 双绿）；daily-final 两 chart 红=762 在册/同族 flake（plan484_024 为 738/739/740 归档在册负载 flake 族）。冒烟收口见 763-smoke-recheck.json（bp 双绿、catalog 零差异、aavm.at 直跑红=master 预存 P763-R2）。

stage: review | PLAN-763 | r1 | outcome: pass（同会话复审，明确限制：执行会话即复审会话，结论自产物重建——基线 worktree 实证 + 仓内登记核对 + 程序化资产核验，不采信执行者叙述） | reviewed_commit: 62a7cf47a（实现 212f222fb + T-03 证据 62a7cf47a，工作树 clean） | base_commit: 716ca62db | dependency_revisions: auto-down 兄弟 detached@895f8d0（零代码触面）；基线对照 worktree 716ca62db 已 guard-clean 移除 | spec_inputs: docs/specs/design/repository-layout.md@edb983653 等 31 份投影（763-spec-delta.json 冻结） | acceptance_results: AC-01 pass（763-ci-audit.json 8 流全字段：definition/sha256/added/recent_runs/latest_jobs/total_runs/decision；.github 不在 24 项根树；author_caveat 诚实记录归因限制）；AC-02 pass（asset-map 396 项：343 git blob 逐字节同 + 53 content_changed 全为路径引用更新（抽验 pac.at 仅注释路径、语义不变）；5 旧根目录零残留；stdlib/.cargo/.config/.agents/.autoos 保留；根树=24（committed-audit 复核））；AC-03 pass（check/t/tv/tu/tt/tb/docs_gen 绿或红全归因；patch_contract=iced_widget→crates/patches 实解析；parity 独立 workspace build+test+list 绿；bp list/show/gen 双示例绿——项目 cwd 复跑实证 ../../packages/blueprints 解析；schema include 经 schema_drift+docs_gen 绿）；AC-04 pass（证据绑定 212f222fb/62a7cf47a；预存独立记录=KNOWN-DEBT P763-R1/R2 + P735-R1/REG-2 引用）；AC-05 待 merge 执行（线性合入/归档/guard 清理为 T-05 动作，非复审缺陷） | findings: F-763-R1(low,非阻塞)=bp 冒烟必须从项目 cwd 调用（dep 相对路径按进程 cwd 解析），已记录 763-smoke-recheck.json；F-763-R2(low,非阻塞)=aavm 预存红 3+1 枚登记 P763-R1/R2（血统 db4e5954b/master 同错），清偿归 aavm 域另档，不阻塞（aavm 无实用面裁定，守护=CI+ta）；零未批准延期/零 workaround（aavm-smoke 换 scratch 测试经 master 同错实证正当） | evidence: docs/reports/763-{ci-audit.json,asset-map.json,committed-audit.json,cargo-metadata.json,gates.json,daily-final.json,baseline-taa.json,smoke-recheck.json,links.json,spec-delta.json,rust-change-scope.json,external-consumers.json}（均入 62a7cf47a；日志按仓惯例 gitignore、JSON 带 sha256 锚定）；门禁复用理由=daily-final 后仅 docs 提交（212f222fb..62a7cf47a 零代码 diff） | next: T-05 auto-plan:merge（含 tf 批量档到期检查——计划号 763%5≠0，按 >48h 规则查 .last-batch-regression.json）

## 待澄清事项
无；已有授权。先审计仓外消费者，保护其他会话检出。

执行基面 716ca62db；专属工作树 lang-763/auto-lang，auto-down detached@895f8d0。八流程审计与396资产迁移记录在开发工作树 docs/reports/763-*。

stage: merge | PLAN-763 | r1 | outcome: pass | completion_kind: delivered | prepared: 62a7cf47a（reviewed，工作树 clean）+31 份 spec 投影在支（763-spec-delta.json 冻结） | landed: master=plan-763-dev=1202d2062（git merge --ff-only 零 merge 提交；rebase 旧→新映射 212f222fb→4707b9742、62a7cf47a→1202d2062，git range-diff 双 = 等价证明；落地 diff rename 检出 100%） | ledger_refreshed: .autoos/specs.json 兼容期手工投影（README §5 通道，758 先例）——designs/repo-layout 同目标更新（24 项现状+PLAN-763:r1+source-sha256+commit:1202d2062）+ 新增 reports/PLAN-763-deposit、tests/repo-consolidation-verification、reviews/PLAN-763-review（832→835 项，无关条目全保留）；索引再生 scripts/spec-index.py 26 projects 零漂移 | archived: docs/plans/archive/763-consolidate-root-packages.md（status: archived，T-05 勾选，本收据） | cleaned: 待清理步回填 | tf 到期检查: 不到期——763%5=3 非 ÷5；收据 2026-10-11T07:41Z（<48h）covered 761，其后仅 L0 fix-nowrap-shrink 与本计划落地，无 ÷5 计划 | 生产工件: 主检出 target/debug/auto.exe（PATH 消费面）重建回执待回填

### spec-sync 回写记录
- docs/specs/design/repository-layout.md：Current-state 30→24 项、五归位表、CI 审计与维护规则（PLAN-763 节）。
- 31 份模块 spec 投影（aavm/auto-lang/auto-man/blueprint/parity/website/widgets/plans/goals/overview/INDEX，清单见 763-spec-delta.json）。
- KNOWN-DEBT-AND-RISKS.md：P763-R1（aavm compile 腿双 E0433 + goldens INVALID_BYTECODE 预存，基线实证）/ P763-R2（aavm.at 单文件直跑 master 同错）。
- goals.md GOAL-016 行同步（CI 删除后未启用 Actions）。
