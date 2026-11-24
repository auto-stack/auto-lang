---
plan_id: PLAN-1125
status: archived
feature_name: "AV/C 语言验收提供者工件可重建"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-24
plan_revision: 1
current_step: 5
total_steps: 5
depends_on: [PLAN-967, PLAN-978, PLAN-1093, PLAN-1096, PLAN-1114]
supersedes_spec_components: [docs/specs/auto-av/language-parity.md, docs/specs/ace-migration/m2-host.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-abi, auto-test-case, auto-av, ace-m2-build]
---

# [PLAN-1125] AV/C 语言验收提供者工件可重建

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](1096-av-parity-full-language-certification.md)。

## 1. 目标

让干净 Windows 检出使用可追溯的 C provider 源和准备流程，重放 1093 的显式 C 标准库正例及 C 头部拼写反例，关闭 1114 复验仍报告的两条工件缺失。

## 2. 架构方案

入库的 C provider fixture 源和配方 → 现有显式 build/prepare → AV worker 装配已钉 provider → 普通源码调用与受检诊断。构建使用测试既有准备段，运行阶段消费已准备工件。

## 3. 技术栈

`crates/auto-lang/test/cases/language/ffi/imports-callbacks.case.ad`、ACE provider 准备及 worker、现有 Windows C 工具链和闭包元数据。

拟实施树为 `D:/autostack/.wt/lang-1125/auto-lang`，分支 `plan-1125-dev`。

## 4. 需求分析与背景调查

[1093](1093-av-parity-foreign-language-surface.md)保留真实 C 调用及原反例。[1114 冻结收据](1114-ace-av-rs-stdlib-providers.md)点名 `av.parity.1093.explicit-c-library` 与 `av.parity.1093.c-header-spelling-rejected`，失败为 `foreign.prepared`，C 闭包源未入库。活动 [1108](../1108-batch-regression-gate-repair-after-1098.md)负责更广的固定门红项和 `source_foreign` authority 对账，T-01 对照共享准备故障，复用已经交付的修复。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

把受控 C 输入、签名、构建参数及实际输出身份绑定到已有测试准备流程。独立验证正例确实进入 C 并调用库，反例实际到达约定的源码拒绝阶段。按同一闭包重放缺工件、撤权和错误身份。生成文件落在执行计划区，验收后依现行规则清理。旧完整报表里的其他基面红保留原治理队列；同根因改动只交一处。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/auto-av/language-parity.md`](../../specs/auto-av/language-parity.md) | 补齐 1093 Windows 两项的准备入口、独立 oracle 和范围 | AC-01/02 |
| SD-02 | modify | [`docs/specs/ace-migration/m2-host.md`](../../specs/ace-migration/m2-host.md) | 补记 C 测试 provider 重建来源、身份和显式准备边界 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 干净 Windows 检出，无旧 provider 根或缓存，显式准备并复放正例 | `explicit-c-library` 调用真实 C `strlen`，结果为 5，记录源及工件身份。 |
| V-02 | AC-01 · T-02 | 同一准备闭包复放 `c-header-spelling-rejected` | 到达 Case 当前约定的阶段/诊断。缺工件另触发 `foreign.prepared`，不能冒充该反例成功。 |
| V-03 | AC-02 · T-03 | 换敏感 provider 输入或制造错误身份/撤权，恢复原输入 | 有意义的已授权变更可观察，错身份/撤权具名拒绝且无调用。恢复后原正反例通过。 |

冻结验证命令为 `cargo t auto-test-case ace-abi ace-av ace-m2-build case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](../1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [x] AC-01 · owner_stage=work。V-01/02 在干净检出可复建，旧两条必要用例都得到自己的 oracle。（证据见 T-02）
- [x] AC-02 · owner_stage=work。V-03 证明身份、来源与授权敏感。固定组保留其他已知失败的实际计数和 owner。（证据见 T-03）
- [x] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [x] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [x] T-01 核定闭包失效原因与 1108 交界
  - owner_stage=work。以现有 recipe 核准备输入、resolver 必读字段和 ABI 红项归属，定位缺失最小源与编译入口。
  - 证据（2026-11-24，worktree `D:/autostack/.wt/lang-1125/auto-lang`，分支 `plan-1125-dev`，基面 master `9b16876f1`，auto-down 兄弟 `456c86a`）：根因不是源缺失——`parity1093_c.{h,c}` 与 `parity1093_c_impl.c` 自 1094 起已入库于 `experimental/ace/crates/foreign-contract/c/parity1093_c/`（`git ls-files` 可见）。断点在 `experimental/ace/crates/abi/src/bin/prepare-av-providers.rs` 的 `prepare_c`：仅在 `ACE_AV_C_PROVIDER_DIR` 已设时才编译，否则打印 skipped 并返回 Ok；而 `scripts/test-governance/toolchains.json` 的 `av-providers` 条目没有 `warm_env`，预热从不设置该变量 → 闭包与描述符不落盘 → `load_prepared_c` 读 `parity1093_c.descriptor.json` 失败 → `foreign.prepared`。1108 的 T-01 尚未执行（未勾选、无交付提交），无共享 producer 修复可复用，本计划自行关闭。附带发现：`open_c_provider` 把 `.c-build-<name>/` 写进 provider 源目录（仓库中被误跟踪的 `.c-build-parity1093_c/` 含 696B `.obj` 即由此而来），故准备器不能直接指向入库源目录，须先暂存副本。worker 侧 `load_prepared_c` 原先不校验描述符 `source_b64` 与 `provider_digest` 的一致性、也不比对 manifest.provider，仅凭描述符自述的 digest 构造 HostAuthority。
- [x] T-02 纳入 C fixture 并接正式准备入口
  - owner_stage=work。依赖 T-01。纳入可追溯测试源和配方，使用现有 prepare/replay 在干净环境运行 V-01/02。
  - 提交 `9ac82a7bd`（实现）、`4b3ff7766`（规范增量草稿）：① `toolchains.json` 的 `av-providers` 增加 `warm_env.ACE_AV_C_PROVIDER_DIR = crates/foreign-contract/c/parity1093_c`（预热 cwd 为 `experimental/ace`）；② `prepare_c` 把三个源文件暂存到 `<root>/out/c-provider-src/` 后再 `open_c_provider`，描述符新增 `source_sha256`（按文件名）；③ worker `load_prepared_c` 在 LoadLibraryW 之前新增两处校验（头源 blake3 ≠ `provider_digest` → `foreign.identity`；manifest.provider ≠ 闭包 provider → `foreign.identity`）；④ 新用例文件 `experimental/ace/crates/av/tests/cases/c_provider_1125.rs`（注册于 `tests/conformance.rs`）。未改动 C 源本身（1094 已入库，无需新增 fixture），未改动两条原 Case 的期望。
  - 准备实测：`prepare-av-providers` 预热全程 1.396–1.545 s（含 f0/py/C 三个闭包），`parity1093_c: dll … provider_digest=e7ed83f446c659363773ad56508c31b78d59a733dd7faf343d6922ad3211b2cc`；暂存后被跟踪树 `git status` 零新增（`.c-build-*` 只落在 `out/c-provider-src/`）。
  - **V-01 / V-02（公开入口）**：worktree 为新建的干净检出（根与 ACE target 均为冷构建），`cargo t case-av-language-parity --prepare-only`（163 s；启动时机器空闲）后 `--run-prepared` 两次：571 PASS / 2 FAIL / 1 ERROR / 214 NOT_RUN / 161 SKIP，执行 574，84.2 s。`av.parity.1093.explicit-c-library` **PASS**（837/878 ms，outcome success、value），`av.parity.1093.c-header-spelling-rejected` **PASS**（831/865 ms，outcome diag，走 Case 约定的 `source.use-rs.provider`），1093 五条全绿。
  - **V-02 后半（缺工件）**：同一已备父二进制在 `ACE_AV_PROVIDER_ROOT=<空目录>` 下复放：`explicit-c-library` FAIL（`expected "success", actual "diag (diag foreign.prepared)"`），`c-header-spelling-rejected` FAIL（`expected "source.use-rs.provider", actual "foreign.prepared"`，stage `admit` ≠ `Resolve`）——反例未被冒充为成功。该次 530 PASS / 44 FAIL（所有 foreign 用例随之为红，符合预期）。
- [x] T-03 验证准备和授权反控
  - owner_stage=work。依赖 T-02。完成 V-03、来源身份和恢复记录，运行受影响的固定组。
  - 代码提交 `9ac82a7bd`（其后仅规范文档提交 `4b3ff7766`）。
  - **V-03 实测（`D:/autostack/plan-artifacts/plan-1125/`，日志为 `new-tests.log`/`oracle-mutation.log`/`prepare-mutated*.log`）**：
    - 错身份具名拒绝且无调用（在**DLL 原样可调用**的闭包副本上篡改描述符）：`artifact_sha256` 改 1 位 → `foreign.artifact`；`source_b64` 追加一行注释 → `foreign.identity`；manifest.provider 改名 → `foreign.identity`；空目录 → `foreign.prepared`。四者均 `value: null`。（**复审 F-01 更正**：初稿在此以"`runtime_calls: 0`、`host_dispatches: 0`"作无调用证据——该断言恒真，真实调用成功时两计数器同为 0，已从用例移除；"无调用"的现行证据是成对对照：同一 DLL 字节的未篡改副本先返回 5，篡改后才被拒。）
    - 撤权：去掉 `foreign.scalar` 或 `foreign.resource` → `source.foreign-capability`（stage bind）；同一闭包权限齐全时返回 5（成对对照）。
    - 已授权变更可观察：① 改签名的 provider 副本被 cl `/WX` 具名拒绝（`warning C4100: 'unused': 未引用的参数` 被视为错误），旧闭包 DLL sha 与描述符保持不变；② 把实现单元改为返回字节长度后重新准备——`parity1093_c_impl.c` 的 SHA-256 `ad612ef8…→feca24cb…`、DLL sha `1f76439b…→cade4c69…`，`"hello\0world"` 的结果 5→11，来源绑定用例变红（2 FAIL / 6 PASS）。
    - 恢复原输入并重新准备 → 8 条新用例全绿。
    - Oracle 区分力：仅禁用两处新校验（保留描述符字段）→ 恰好 `…foreign_manifest_provider_is_rejected` 与 `…foreign_header_source_is_rejected` 两条变红，其余 6 条通过；已恢复。（整文件回退到旧 worker 的那次运行不可用作证据：旧结构体缺 `source_sha256` 且 `deny_unknown_fields`，全部报 `foreign.prepared`，属 fail-closed。）
    - **边界（如实记录）**：同一份源两次准备出的 DLL SHA-256 不同（实测 `1f76439b…`/`188f018d…`/`bf368263…`/`de87bb59…`），`provider_digest` 恒为 `e7ed83f4…`、`source_sha256` 恒定。因此"可重建"指来源身份稳定且运行时逐次重验，不是位级可复现二进制。
  - **新用例耗时**（nextest，debug/test profile，单条 ≤0.11 s；`ace-av` 整组并行时 0.069–0.304 s）：`plan1125_c_provider_calls_the_real_c_strlen` 0.108 s（`"hello"`→5、`""`→0、`"hello\0world"`→5、`"héllo😀"`→10，证明进入真实 C strlen 而非字节长度）；`…_closure_is_bound_to_the_in_repo_sources` 0.020 s；`…_wrong_artifact_identity_is_rejected` 0.068 s；`…_foreign_header_source_is_rejected` 0.071 s；`…_foreign_manifest_provider_is_rejected` 0.079 s；`…_missing_closure_is_a_preparation_failure` 0.079 s；`…_revoked_capability_rejects_before_the_call` 0.076 s；`…_header_spelling_reaches_the_source_rejection` 0.072 s。整批 8 条 0.121 s。
  - **受影响固定组**（Windows 环境；Linux/WSL 腿按 2026-11-14 裁定不在本机执行）：
    - `case-av-language-parity`：准备 163 s（case-parent 构建 94.0 s、av 30.5 s、av-providers 35.5 s + 预热 1.6 s，冷构建）；执行两次，574 执行 / 84.2 s 与 83.7 s，均 571 PASS / 2 FAIL / 1 ERROR / 214 NOT_RUN / 161 SKIP。1093 五条全 PASS。其余红项**均非本计划触面**：(a) `av.parity.1082.empty-array-rejects`、`av.parity.1084.boxed-nil-arithmetic-script-catch`——与 [1124](1124-av-stdlib-json-and-csv.md) 归档收据逐名一致的基线红；(b) `py.float-args.1129.torch-lr-computed` ERROR `av.resource.execution-wall: console preparation exceeded wall budget`——用例首条 `print` 前的 torch 导入+训练贴着 `MAX_AV_WALL=2000 ms`。**master A/B（`ad0268ba7`，无本修复）同一已备清单三次复放：PASS / ERROR(4323 ms) / PASS**，本树两次均 ERROR（3064/3573 ms），直连 worker 执行段 2003 ms；属 1129 用例的边界时序，owner = PLAN-1129，非本改动引入。master 基线同组结果 570 PASS / 4 FAIL（1093 两条 C 用例为红），与本树 571 PASS / 2 FAIL 的差正是本计划关闭的两条。
    - `ace-av`：准备 749 s（冷构建全部 ace-av 测试二进制）；执行 1703 / 74.0 s，**1690 通过 / 13 失败 / 23 跳过**；8 条新用例全部 `ok`。13 红逐名核对均有既往记录：D-1055-BASE-01（[KNOWN-DEBT](../KNOWN-DEBT-AND-RISKS.md)）与 D-1088-LAYOUT-01（`layout_rejects_non_record_and_empty`），与 [1121](1121-av-stdlib-map-set.md)/[1124](1124-av-stdlib-json-and-csv.md) 收据的 13 红清单一致，无新增红。
    - 未跑（归属 T-05 merge 冻结并集）：`auto-test-case`、`ace-abi`、`ace-m2-build`。
  - 机器占用登记（`occupancy.log`）：`case-av-language-parity` 两次执行启动时均空闲；`ace-av` 准备启动时空闲，执行前有界等待 60 s 后仍有 5 个其他会话进程（plan-1068 `prepare-native`、ui-118 release 示例构建）即启动，执行期数字含该背景负载。定向调试期间这些会话持续在场，未声称空闲。
  - 健康检查：本计划改动的 4 个源文件无编译警告、无调试输出（`ace-av`/`ace-abi` 构建中其余 warning 为既有）。
- [x] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
  - 2026-11-24 首审 needs_fix（F-01）→ work 修复 `09d084b25` → 重审 pass；记录见 §10。
- [x] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1125 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

2026-11-24 · stage: work | PLAN-1125 r1 | outcome: **pass（work 阶段 T-01..T-03 完成）** | worktree_path: `D:/autostack/.wt/lang-1125/auto-lang` | branch: `plan-1125-dev` | base_commit: `9b16876f1`（master，开工时；期间 master 仅前移 5 个纯文档提交至 `ad0268ba7`） | code_commit: `9ac82a7bd`，规范文档提交 `4b3ff7766` | dependency_revisions: auto-down 兄弟 `456c86a`（detached，只读，未改动） | task_ids: T-01, T-02, T-03 | evidence: 见 §8 各任务证据；材料区 `D:/autostack/plan-artifacts/plan-1125/`（已登记 `.retention/policy.json` scopes，in-flight） | blockers: 无 | next_responsible_role: review（T-04 / AC-03）。

- **work 自有 AC**：AC-01 pass（V-01/V-02 公开入口复放 + 缺工件对照）；AC-02 pass（V-03 四类错身份 + 撤权 + 已授权变更 + 恢复 + oracle 变异）。
- **未闭合的 in-contract 发现**：无。
- **遗留给 review/merge**：AC-03（独立复审、重放关键正反例）、AC-04 与 T-05 的冻结并集 `auto-test-case ace-abi ace-av ace-m2-build case-av-language-parity`（本阶段实跑 `case-av-language-parity`、`ace-av` 两组；`auto-test-case`/`ace-abi`/`ace-m2-build` 未跑，归 merge）。规范增量 SD-01/SD-02 已在 worktree 提交为草稿，待 review 认定。
- **需 review 留意的既有问题（非本计划引入，未在本计划内修）**：① `py.float-args.1129.torch-lr-computed` 贴 `MAX_AV_WALL=2000 ms` 间歇 ERROR（master 可复现，owner PLAN-1129）；② 仓库中被误跟踪的 `experimental/ace/crates/foreign-contract/c/parity1093_c/.c-build-parity1093_c/`（含 696B `.obj`）是 1094 遗留构建副产物，本计划不以其为输入、未删除，是否去跟踪建议作为债务候选登记；③ `case-av-language-parity` 的 `av.parity.1082.empty-array-rejects`、`av.parity.1084.boxed-nil-arithmetic-script-catch` 基线红仍在 1124 的治理队列。
- **"第一个阶段"的范围解释**：用户目标为"完成计划 1125 的第一个阶段"，按计划状态机（`drafting → executing → …`）与 AC 的 owner_stage 划分，取 **work 阶段 = T-01..T-03 / AC-01, AC-02**；未执行 review（T-04）与 merge（T-05）。若用户本意仅指 T-01，则 T-02/T-03 属超额完成，均在 work 授权范围内且未触碰 master 代码。

2026-11-24 · stage: review | review_scope: final | reviewer_identity: **实施会话内自审**（非独立上下文；结论由提交、原始 results.json 与复审方自选探针重建，不采信 work 摘要）| PLAN-1125 r1 | outcome: **needs_fix** | worktree_path: `D:/autostack/.wt/lang-1125/auto-lang` | branch: `plan-1125-dev` | reviewed_commit: `4b3ff77667e1dc54b8903030272ccc6acb6a98a9`（干净，dirty=0）| base_commit: `9b16876f1` | dependency_revisions: auto-down `456c86a` | spec_inputs: worktree `language-parity.md` sha256 前缀 `3d137e936aa88027`、`m2-host.md` `c88db421c8a46c12`（master 自基线仅 PLAN-1068 文档提交，与本分支零重叠；`git merge-tree` 无冲突）| acceptance_results: AC-01 pass；AC-02 **partial**（见 F-01）；AC-03 pending | findings: F-01 | next: work 修复后重审。

- **复审方探针**（`review/probe.py`、`probe2.py`、`probe3.py`，直接驱动已备 `av-worker.exe case-serve`，不经测试脚手架）：`"hello"`→5、`""`→0、`"hello\0world"`→5、`"héllo😀"`→10 均 success；`use.c <string.h>` 与 `use.c string_h` → `source.use-rs.provider`；撤 `foreign.scalar` → `source.foreign-capability`；描述符 sha 翻位 → `foreign.artifact`；`kind=rust` 指向 C 闭包 → `foreign.prepared: provider descriptor identity mismatch`；`kind=c` 指向 `f0_text` → 同码拒绝；未注册 provider → `foreign.prepared`（描述符缺失）。接线与实施声明一致。
- **F-01（合同内，V-03 / AC-02 / T-03，oracle 缺陷）**：`c_provider_1125.rs` 的 `rejected()` 以 `runtime_calls == 0 && host_dispatches == 0` 作为"无调用"的断言，但复审探针显示**真实调用 C `strlen` 成功的 step 这两个计数器同样为 0**（`runtime_calls` 统计的是 AV 调用帧提交，不含 foreign stub 派发）——该断言恒真，对"错身份/撤权 ⇒ 无调用"不构成证据；§8 T-03 记录与 SD-01 草稿文本中"`runtime_calls`/`host_dispatches` 为 0"的表述因此不成立。违反的既有规则：验收 oracle 必须能区分合理的错误实现（verification-contract）。实际的"无调用"证据只有结构性的（`load_provider` 在编译/执行前以 `?` 返回；变异运行显示去掉校验后篡改闭包会被放行）。修复要求：在每条篡改/撤权用例内建立**同 DLL 字节的未篡改对照返回 5** 作为可调用性见证，删除恒真计数器断言与误导性注释；同步更正 §8 记录与 SD-01 文本。最小复放：`cargo nextest run --offline -p ace-av --test conformance -E 'test(c_provider_1125)'` + 直连 worker 观察成功 step 的 counters。影响路径：T-03、AC-02、SD-01 文本；T-01/T-02/AC-01 不受影响。
- **观察 O-01（合同外，不阻断）**：`ac.source.language.av/v1` 在 foreign 链与普通链上都不是必需能力（仅给 `['console.stdout']` 甚至空 caps 的普通程序 `6*7` 仍 success），与 `language_source_worker.rs` 第 79–81 行注释"declared caps enter the source admission axis"的读法不符。属 worker 既有通用行为，非本计划触面，作为债务候选交 merge 登记。

2026-11-24 · stage: work（F-01 修复）| PLAN-1125 r1 | outcome: **pass** | worktree_path: `D:/autostack/.wt/lang-1125/auto-lang` | branch: `plan-1125-dev` | base_commit: `9b16876f1` | code_commit: `09d084b25d0cda592189f0c6c33e52ffcc43e2f8`（修复提交；树干净）| dependency_revisions: auto-down `456c86a` | task_ids: T-03 | evidence: `review/fix1-tests.log`、`review/fix1-mutation.log`、`review/fix1-final.log`（材料区 `D:/autostack/plan-artifacts/plan-1125/`）| blockers: 无 | next_responsible_role: review。

- **F-01 → 不变量 → 修复 → 结果**：F-01（"无调用"oracle 为恒真计数器断言）→ 不变量"身份或授权不成立时 provider 不被调用，且测试能区分缺少该门的实现" → 影响路径：`c_provider_1125.rs` 的 4 条篡改用例、撤权、缺闭包、头拼写用例；SD-01 文本；§8 T-03 记录 → 修复：`tampered()` 先对副本跑 `control()`（未篡改即返回 5）再改描述符，其余拒绝用例各自先对真闭包跑 `control()`；删除计数器断言并写明原因 → 结果：`cargo nextest run --offline -p ace-av --test conformance -E 'test(c_provider_1125)'` 8/8（0.031–0.261 s）。变异复核（仅禁用两处新校验）：`…foreign_header_source_is_rejected` 失败于 `outcome: success, value: 5`——**被篡改的闭包真的被调用**，新 oracle 抓住；`…foreign_manifest_provider_is_rejected` 失败于下游 `source.use-rs.provider`（名称绑定兜底，非 `foreign.identity`）。校验已恢复（`grep -c 'if false'` = 0）。本轮测试范围仅该过滤器 + 变异运行；worker/准备器/工具链源码自 `9ac82a7bd` 未变，T-01/T-02/AC-01 与两组固定组证据按"输入未变"复用，未重跑。

2026-11-24 · stage: review | review_scope: final | reviewer_identity: **实施会话内自审**（非独立上下文；结论由提交、原始 results.json、复审方自选探针与变异运行重建，不采信 work 摘要）| PLAN-1125 r1 | outcome: **pass** | worktree_path: `D:/autostack/.wt/lang-1125/auto-lang` | branch: `plan-1125-dev` | reviewed_commit: `09d084b25d0cda592189f0c6c33e52ffcc43e2f8`（干净）| base_commit: `9b16876f1` | dependency_revisions: auto-down `456c86a` | spec_inputs（冻结副本哈希）: worktree `docs/specs/auto-av/language-parity.md` sha256 `5bd84a28aa8ed00a18f962988144436a38603267e8695ade608154ce097a739d`；`docs/specs/ace-migration/m2-host.md` sha256 `c88db421c8a46c12c58ab5d9e040b8bad2687158c4660e1c7afaefa9b7664dc9` | acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass | pending_closeout_gates: 冻结并集 `auto-test-case ace-abi ace-av ace-m2-build case-av-language-parity`（merge 所有）| findings: 无未闭合项 | next: merge。

- **AC 证据映射**：AC-01 → T-02 → `toolchains.json`/`prepare-av-providers.rs` → `run-case-av-run*/results.json`（1093 两条 PASS）+ `missing-run/report.json`（缺工件两条 FAIL `foreign.prepared`）；AC-02 → T-03 → `foreign_source.rs` 两处校验 + `c_provider_1125.rs` → 首审探针 `review/probe*.py` 结果 + `fix1-mutation.log`；AC-03 → T-04 → 本记录。
- **遗漏/延后/Workaround 扫描**：§5 要求"把受控 C 输入、签名、构建参数及实际输出身份绑定"。已绑定：输入（源 SHA-256、头摘要）、输出（DLL SHA-256）。**签名**未单独进描述符（由 manifest 的 contract_digest 在 `admit` 内覆盖）。**构建参数**（cl 版本、include 闭包、`-O1`）未进入描述符——这是未做项，非 workaround，记入下方边界 (a)，交 merge 登记为债务候选。§5"按同一闭包重放缺工件、撤权和错误身份"已完成。无未经批准的范围缩减，无遗留 `.autoos/specs.json` 写入。
- **规范增量复核**：SD-01（modify `auto-av/language-parity.md`，新增 PLAN-1125 增量节）与 SD-02（modify `ace-migration/m2-host.md`，新增 C provider 节）描述现行行为与持久决定，对应 AC-01/02；计数器表述已随 F-01 更正；节内 `[PLAN-1125]` 链接已指向归档路径（归档后生效）。`new_spec_components` 为空（仅修改既有文档）；`supersedes_spec_components` 两路径准确；`touched_goals: [GOAL-002]` 沿用 frontmatter。
- **测试纪律**：新增 8 条用例均 ≤0.261 s；只消费已备闭包，无可达编译/缓存回退路径（缺闭包 = 具名 panic）；构建为 debug/test profile。复审会话实跑表：

| 测试模块/范围 | 结果 | 构建 | 执行 |
|---|---|---|---|
| `ace-av::conformance` 过滤 `c_provider_1125`（修复后） | 8/8 | 增量重建含在内 10 s | 0.293 s |
| 同上，变异运行（禁用 2 处校验） | 6 过 / 2 红（预期） | 含在内 11 s | 0.137 s |
| 同上，提交后复核 | 8/8 | 8 s | 0.125 s |
| 复审方直连 worker 探针 `probe.py`/`probe2.py`/`probe3.py` | 14 次调用，全部符合预期 | 无 | <5 s |

新用例行为/耗时（修复后）：`…calls_the_real_c_strlen` 0.103 s（四组输入进入真实 C strlen）；`…closure_is_bound_to_the_in_repo_sources` 0.031 s；`…header_spelling_reaches_the_source_rejection` 0.084 s；`…revoked_capability_rejects_before_the_call` 0.092 s；`…missing_closure_is_a_preparation_failure` 0.096 s；`…foreign_manifest_provider_is_rejected` 0.256 s；`…foreign_header_source_is_rejected` 0.260 s；`…wrong_artifact_identity_is_rejected` 0.261 s。未重跑：`ace-av` 整组与 `case-av-language-parity`（证据复用，归 merge 冻结并集重跑）。启动时机器 busy=15（其他会话），本轮仅定向过滤器用例。
- **边界（不阻断，交 merge 登记）**：(a) 描述符不记录构建参数与工具链身份，DLL SHA-256 逐次链接不同——"可重建"= 来源身份稳定 + 运行时逐次重验；(b) O-01：`ac.source.language.av/v1` 在普通链与 foreign 链上均非必需能力（worker 既有通用行为）；(c) `py.float-args.1129.torch-lr-computed` 贴 `MAX_AV_WALL=2000 ms` 间歇 ERROR（master 三次复放 PASS/ERROR/PASS，owner PLAN-1129）；(d) 误跟踪的 `.c-build-parity1093_c/`（含 `.obj`）。

2026-11-24 · stage: merge | PLAN-1125:r1 | outcome: **pass** | completion_kind: delivered | delivery_commit: `8ea33745fbce7c6b5de0abe3ec511ca2ef58f016`（master 尖端，`--ff-only`）| archive_path: `docs/plans/archive/1125-av-c-provider-reproducible-fixtures.md` | worktree_path: `D:/autostack/.wt/lang-1125/auto-lang` | branch: `plan-1125-dev`（已删除）| base_commit: master `38d2b86ce` | dependency_revisions: auto-down `456c86a`。

- **prepared**：复审提交 `09d084b25`（基面 `9b16876f1`）。master 在本计划执行期间持续前移（PLAN-1068 文档；`6c90a3569` 等 PLAN-118 代码），开发分支共变基三次，逐次 `git range-diff` 三个补丁均为 `=`：`9ac82a7bd→fe16cf947→0278bf25c→4393ce4fb`、`4b3ff7766→827370c46→c2ddc23bd→524ba101d`、`09d084b25→aa37339bc→876dfc4b1→8ea33745f`。最后一次变基（`c7824bb02→38d2b86ce`）前后非计划文档树逐字节相同（`git diff --quiet -- . ':!docs/plans'`）。
- **landed**：主检出 `git merge --ff-only plan-1125-dev` 成功，master 尖端 = `8ea33745f`，线性历史、无合并提交。
- **canonical_verified**：main 上 `docs/specs/auto-av/language-parity.md` sha256 `5bd84a28aa8ed00a18f962988144436a38603267e8695ade608154ce097a739d`、`docs/specs/ace-migration/m2-host.md` sha256 `c88db421c8a46c12c58ab5d9e040b8bad2687158c4660e1c7afaefa9b7664dc9`，与复审冻结值一致。未写 `.autoos/specs.json`，未发布索引。
- **冻结并集（T-05 / AC-04）**：`cargo t auto-test-case ace-abi ace-av ace-m2-build case-av-language-parity`，worktree 内，提交 `876dfc4b1`（= 交付提交的补丁等价前体，基于 master `c7824bb02`；其后 master 仅文档提交）。准备 589 s（启动时机器空闲），执行 317.2 s：

| 组 | 结果 | 归属 |
|---|---|---|
| `auto-test-case` | PASS 157/157 | — |
| `ace-m2-build` | PASS 50/50 | — |
| `case-av-language-parity` | 572 PASS / 2 FAIL / 214 NOT_RUN / 161 SKIP（执行 574） | 1093 五条全 PASS（`explicit-c-library` 1485 ms、`c-header-spelling-rejected` 4596 ms）；2 红 = `av.parity.1082.empty-array-rejects`、`av.parity.1084.boxed-nil-arithmetic-script-catch`（D-1129-12 基线） |
| `ace-abi` | 341 ok / 28 failed（执行 369） | 与 **master 尖端同环境基线逐名相同**（worktree 分离到 `15e3f08c9` 跑 `cargo t ace-abi`：369 执行 / 341 ok / 28 failed，双向差集为空）。25 条见 1109 归档的基线清单；另 3 条 `source_resources` 的 `native_lower_*`/`native_foreign_call_*` 同为基线红 |
| `ace-av` | 1654 ok / 51 failed（执行 1705） | 与 **master 尖端 `38d2b86ce` 同环境基线逐名相同**（1697 执行 / 1646 ok / 51 failed，双向差集为空）；本计划净增 8 ok（新用例 8/8，最大 0.529 s）。51 = 13 条 D-1055-BASE-01/D-1088-LAYOUT-01 继承红 + **38 条 master 新破损**（见下） |

- **master 新破损（非本计划引入，登记 D-1125-01）**：`ace-av` 第二轮比第一轮多 38 红（`stdlib_asm_972_*`、`sys973_*`、`bstr_1064_*`、`v02_/v03_/v10_*` 等）。失败信息一致：`"std::list" … source.unsupported `fn` is not part of this source profile @1526..1528`。来源：master 提交 `6c90a3569`（PLAN-118）在 `stdlib/ace/asm/list.at` 新增 `pub fn join` 与非 pub 的 `fn join_into`，asm 标准库 profile 不接受私有 `fn`，所有 graft `std::list` 的用例编译失败。空闲机器上原样重跑仍红（确定性）。第一轮并集（旧 master，`aa37339bc`）该组为 1690 ok / 13 failed。
- **未跑 / 排除**：Linux/WSL 腿按 2026-11-14 裁定不在本机执行。本合并会话未重跑 `auto-test-case` 以外的其它组；无新用例新增（8 条即前述，耗时见 §9 复审记录）。
- **机器占用**：并集准备与基线准备均在有界等待后启动（`occupancy.log`）；冻结第二轮执行与基线执行启动时 busy=0。定向探针期间其他会话（plan-1068、ui-118、PLAN-118 合并）持续在场。
- **archived / cleaned / 批量回归到期**：见下一条收据。
2026-11-24 · stage: merge（收尾收据）| PLAN-1125:r1 | checkpoints: **archived ✓ · cleaned ✓**。

- **archived**：`git mv` 至 `docs/plans/archive/1125-av-c-provider-reproducible-fixtures.md`，`status: archived`，T-01..T-05 / AC-01..AC-04 全部勾选，`current_step: 5/5`，`completion_kind: delivered`。归档及簿记提交 `5d259d3b5`（含 KNOWN-DEBT 新增 D-1125-01..05、策略文档 `av-autovm-language-parity.md` 指向归档路径的链接修正）。归档后计划内 13 个相对链接全部可解析（脚本核对，broken = 0）；`language-parity.md` 中 `[PLAN-1125]` 指向归档路径。
- **cleaned**：移除前复核 worktree 干净（`git status --porcelain` 0 行）、`plan-1125-dev` 已是 master 祖先；`pwsh -NoProfile -File D:/autostack/wt-guard.ps1` 对 `D:/autostack/.wt/lang-1125/auto-lang` 与 `…/auto-down` 均报 clean；随后 `git worktree remove` 两仓、`git branch -d plan-1125-dev`（was `8ea33745f`）、删除空组目录。auto-down 兄弟未改动（`456c86a`）。
- **材料区删除**：`D:/autostack/plan-artifacts/plan-1125/` 整目录在证据折入后删除——10,734 个文件、1,031,826,055 字节、0 个 reparse point；无具名未解决问题持有。构建中间产物随 worktree 删除，未回收。收尾路径对账：本计划命令/日志使用的输出目录仅该材料区、worktree 内两处 `target`（随 worktree 删除）、主检出既有 `target`（A/B 基线构建的忽略输出，非本计划所有、未删除）及 runner 自建自清的系统临时 `tmcase-*` / `ace-av-c1125-*`（运行时清理标记为真）；未发现其他盘符或散落目录。
- **批量回归到期检查**：`docs/plans/.last-batch-regression.json` 的 `last_covered_plan_id = 1126`（2026-11-23T00:03:39Z）。1125 ≤ 1126 且收据距今约 32.5 h（< 48 h）→ **未到期**，未触发 `cargo t all`。提醒：收据满 48 h 且其后有合并时到期；届时 master 已知的 `ace-av` 38 条 `std::list` 红（D-1125-01）会被计为新红或需对照。
- **部署观察**：本计划只改测试准备器、测试配置、worker 的 C provider 校验与测试/规范文档；无发行包、常驻应用或 web bundle 依赖本改动，未触发任何 release 构建或重启。

## 10. 待澄清事项
本计划可直接核干净检出失败。1108 如已修共享 producer，以其当前交付复用，保留本计划两条原 Case 的独立关闭条件。
