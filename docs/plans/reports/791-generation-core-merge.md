# PLAN-791 合入沉淀收据（生成核心与 Auto 可执行模板适配拆分）

2026-10-16；plan_revision=1；review=R3 pass（绑定 561a61fc6）。

## 交付链（rebase 后 ff-only）

原链（复审绑定）：`9c4780c54`(T-01 冻结)→`b8b439304`(core crate)→`6ee5123eb`(adapter 委托)→`21ff04030`(consumer 门)→`0d8eafb05`(基线刷新1)→`0cd3e3e9f`(SD 素材)→`b54cc4a2a`(R1 修复)→`561a61fc6`(R2 基线刷新2，R3 受审冻结)。

主检出 master 并发推进（787/788/790 簿记，零路径重叠）→ worktree 内 rebase，`git range-diff 561a61fc6~7..561a61fc6 85993b4b2..3f2ef93bb` **7/7 全等（`=`，0 处改动）**，映射：
`b8b439304→e9f2e0f2b`、`6ee5123eb→726188a7c`、`21ff04030→b77415023`、`0d8eafb05→339ae8ed5`、`0cd3e3e9f→92677c3b3`、`b54cc4a2a→a65687a90`、`561a61fc6→3f2ef93bb`。

沉淀提交（本提交，docs/projection-only descendant）：spec-index GROUPS 登记、overview/goals 回写、双模块 plans.md、INDEX 再生、本收据、ledger v30→v31 守卫式离线投影。实现/依赖/测试配置/锁与 3f2ef93bb 零变化（diff 证明 docs-only）。

## 并发合流（第二轮 rebase，over 789 落地）

789 于本轮合入 master（auto-lang 词法源码变更 + auto-lex crate + ledger v31 + 基线/spec-index/根清单变更）→ 第二次 rebase 解冲突：根 Cargo.toml exclude 并集（lex-consumer + gen-core-consumer）；边界基线两处取 789 锚（最终干净树重锚 = 43 边含双方新边 auto-lang→auto-lex / auto-gen→auto-gen-core）；ledger v31(master P789 五条) ∪ P791 五条 → **v32**（守卫式：备份/外科手术/回读校验）；spec-index GROUPS 并集。`git range-diff` 第二轮：代码提交（T-02/03/04/SD）全 `=`；三个 `!`（core-crate/consumer-gate/consolidation）仅含上述 union 解冲突与上下文漂移（789 注释行插入致 Cargo.toml hunk 上下文位移），无语义改写。附带补齐首轮遗漏件：docs/specs/plans.md 双索引 791 行 + INDEX 再生（34 projects）。

## merge-owned 冻结门禁（@最终 tip，重跑因 789 改 auto-lang 源码作废首轮证据）

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t`（日常档 nextest -p auto-lang，@46983e0be） | **三次完整全绿 5318/5318**（67.9s/110.1s/101.6s 量级；789 落地后选择面 5382→5318，词法测试迁 auto-lex）。六次运行中两次满池负载瞬态：1×TMO（名未捕获）+1×`state_file::tests::lock_serializes_critical_sections` FAIL 0.342s——**归因预存负载 flake**：solo 复跑 3/3 绿（0.11-0.14s），该文件归属 PLAN-044/738 谱系，本计划 diff 零 auto-lang 触面（与批量回执 plan707/plan484 满池红/单跑绿同族，不豁免门禁、仅记录观察） |
| `cargo test -p auto-gen -p auto-gen-core`（定向全测） | **50 绿**：adapter lib 11 + V02 金样 14 + V03 失败语义 8 + core 12 + doc 5（bin/core-doc 0 tests） |
| `cargo test --manifest-path test/fixtures/modularity/gen-core-consumer/Cargo.toml` | **4 绿**（外部无解释器消费方） |
| `bash test/fixtures/modularity/gen-core-consumer/verify-deps.sh` | **PASS**（normal/build 闭包=core→auto-val+叶依赖，零 auto-lang/auto-man/auto-vm/UI） |
| `cargo fmt -p auto-gen-core -p auto-gen -- --check` | exit 0 |
| `python scripts/modularity-audit.py --mode strict` | **pass**（violations=0，grandfathered=2 不变：auto-man/auto-gen→auto-lang，R5 退出） |

未触 VM/transpiler/book/UI-codegen/aavm 路径 → tv/tt/tb/tu/taa 按 AGENTS.md 触面映射不适用；tf 为批量回归档非 per-plan 门禁。

## canonical 沉淀

- SD-01 新增 `docs/specs/auto-gen-core/project.md`（单核心、无解释器边界/owned renderer/guard 与报告合同、冻结语义五条、诚实债登记）。
- SD-02 改写 `docs/specs/auto-gen/project.md`（纠正"简单文本替换"旧描述；Auto adapter+兼容门面定位；legacy 模板执行留 adapter、guard feed 单一定义在 core；未迁清单）。
- module 回写：`docs/specs/auto-gen/plans.md`、`docs/specs/auto-gen-core/plans.md`（新建，各一行）。
- 全局件：`docs/specs/overview.md` 工具链表（auto-gen 行改述 + auto-gen-core 新行）；`docs/specs/goals.md` GOAL-003/GOAL-016 追加 791 引用。
- 索引：`scripts/spec-index.py` GROUPS 登记 auto-gen-core + `INDEX.md` 再生（33 projects）。
- ledger：`.autoos/specs.json` **v31→v32**（master 789 五条 ∪ P791-D1/D2/T/R3/M；守卫式离线投影：无 store writer 可达、字节级备份、diff 证明仅 version 行+5 追加、schema/read-back 校验、goals/architecture 节逐字不变）。

## 未迁/债（诚实边界）

- auto-man 仍经 legacy AutoGen/Mold 消费 auto-gen；assets/templates 自有解释器——Man→语言栈依赖未消（module-boundaries grandfather，exit R5）。
- 预存缺陷原样保留并冻结为合同（见 auto-gen-core/project.md"已知债"）：AutoFile/AutoCode 数据源与 lib_files 定义对模板不可见（仅 DataSource::Atom 可渲染）、fstr_note 在共享解释器渲染路径无效、use 解析未接线。
- autogen_tests.md Test4 期望对齐（解释器 bool 渲染 true；档外红自 f955fa533 迁移起）。

## 部署物与批量回归

- 部署物检查：auto-lang 仓无运行时生产工件（backend 二进制/web bundle/daemon 属 auto-musk 域）；本计划纯库/测试面，无可重建工件。
- 批量回归到期判定：收据 last_covered_plan_id=786（2026-10-16）；本轮窗口内 789 已落地（789%5=4≠0）+791（%5≠0），收据未超 48h → **不到期**。下一次到期由后续 %5 计划（795）或 48h+合并触发。
