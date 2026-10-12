# PLAN-768 T-08 工作侧证据包

- 日期：2026-10-12；stage: work（T-08，work-owned 全部完成）。
- 绑定：worktree `D:/autostack/.wt/lang-768/auto-lang`，branch `plan-768-dev`，
  base `540dc7d30839eb7df8e68b4d19472358e170fe5b`，代码提交链
  `2291cd78e → 8feeb9678 → cbe902715 → 1b13aa323 → 7df514aa2 → 85710109d → 8a9f305cb`。
- 依赖版本：auto-down pinned 分离 worktree `fba6563ed2148ce85e68208863159b4ccccac710`
  （组内只读兄弟，无 junction/symlink）；vendor 快照指纹 = feasibility §1。

## 1. V-01–10 控制矩阵（全部可执行、全部通过）

| Case | 实现（测试/命令） | 结果 |
|---|---|---|
| V-01 严格合同/发现/身份 | auto-test-case：`contract_decode.rs` 14 例（未知字段定位、封闭矩阵、弱 oracle 拒绝、product_from 引用+环、skip/xfail、计数限额）+ `discovery_suite.rs`（逐文件错误保留、重复 case id 定位、零发现硬错）；CLI `lint`（21 case 清单） | 全过；CLI lint exit 0 |
| V-02 bytes/fence/映射 | `document_fences.rs` 9 例（CRLF 混 LF 逐字节、四反引号嵌套、空 fence、尾空格、缩进/中缀不误判、树↔raw 一一对应、anchor 恰一、lone CR、mermaid 映射）+ `vendor_provenance.rs`（指纹冻结防漂移） | 全过 |
| V-03 双后端独立判定 | `avm_positive_negative_and_independence`（错 oracle 只废对应 check）+ pilot 3 个双 check case（hello/str-upper/eq-false，24 检查 24 PASS） | 全过 |
| V-04 产物接口/BLOCKED | `a2r_emit_matches_legacy_golden_bytes`（逐字节金样）+ `a2r_project_emission_multi_artifact_manifest`（5 产物清单 ≡ 同布局直接调用）+ `a2r_product_chain_blocked_on_failed_upstream`（上游 FAIL→下游 BLOCKED、actual 仍持久化、exit 2） | 全过 |
| V-05 强诊断/基础设施隔离 | `avm_wrong_stage_expectation_fails`（错阶段 FAIL）+ `infra_errors_do_not_satisfy_language_negatives`（parse 错不满足 run 预期）+ classify 单测（MultipleErrors 递归、VMError 变体名=code）+ `a2r_missing_manifest_is_contract_error_not_silent_output` | 全过 |
| V-06 只读/隔离/proposal | 树哈希证明（§3）+ `proposals_are_written_to_proposal_dir_only`（oracle 前后字节不变、候选=actual、PROPOSALS.md）+ scratch 唯一 run 目录（run-<millis>-<pid>） | 全过 |
| V-07 对账/顺序无关 | `pilot_legacy_map_reconciles`（map↔报告集合精确相等、adapter 断言、21 case、多产物/强化/复活类必含）；发现顺序=include 声明序（suite 加载确定性） | 全过 |
| V-08 required/selector/stale | `skip_and_coverage_gap_semantics`（gap→exit 2）+ report 单测（退出码 2>1>0、NOT_RUN 破绿、identity 漂移=stale）+ CLI selector（未选项记 NOT_RUN 带 detail，报告 partial） | 全过 |
| V-09 来源固定/无 GUI/资源 | `vendor_provenance`（指纹）+ `--no-default-features --features test-cases` CLI 全链 24/24 exit 0 + `cargo tree`（运行时依赖仅 vendor 解析器）+ admission 单测（默认/硬顶/clamp、虚拟路径规则） | 全过 |
| V-10 项目/多模块/.as/entry | `a2r_project_emission_multi_artifact_manifest` + `avm_as_source_lowers_and_records_hash`（.as lowered hash 证据）+ pilot `.as` case + module entry case；目录派生元数据=Cargo.toml app 名（feasibility §4 发现，oracle 锁定） | 全过 |

## 2. 成本（构建/执行分开）

| 项 | 实测 |
|---|---|
| 协议 crate 测试（`cargo test -p auto-test-case`） | 57 例 <1s |
| 适配器+pilot 测试（`--lib --features test-cases test_cases`） | 18 例 ~2s |
| pilot CLI 暖跑（24 检查，含 cargo 启动） | 3.2 s（exit 0） |
| pilot CLI 冷跑（含增量重链） | 224 s（链接占绝对主导） |
| no-default CLI 暖跑 | 同 3 s 量级，24/24 exit 0 |

## 3. 只读证明（哈希）

对试点引用的全部 legacy 目录树（vm/a2r/cases 共 12 个子树、全部文件逐一
sha256 后再聚合哈希）：

- 运行前：`4ce1f974dfc04282dbb62a94da4102f802911fd6a0c97409c1133c24a3bceba5`
- 完整 CLI run 后：`4ce1f974dfc04282dbb62a94da4102f802911fd6a0c97409c1133c24a3bceba5`

产物/报告仅写入 `--out` 指定 scratch（`run-<millis>-<pid>` 唯一目录）；
proposal 仅写提议目录（测试锁定）。

## 4. 门禁结果（本树实测，2026-10-12）

| 档 | 命令 | 结果 |
|---|---|---|
| 协议 | `cargo test -p auto-test-case` | 57/57 ok |
| 适配器 | `cargo test -p auto-lang --lib --features test-cases test_cases` | 18/18 ok（含 1 ignored 工具测试） |
| 日常 | `nextest … --features ui-iced,test-cases --no-fail-fast` | 仅 4 在案红：schema_drift_fence、queue_coverage_drift_fence、kitchen_sink_page_in_sync（docs 围栏族）+ ash_leak_probe（环境红）；**test_cases 0 红** |
| 语料 | `cargo tv`（含 test-cases） | cookbook cb_* 10 红——**stash 基线对照证实预存**（程序内断言语义红，与 native 修复无关；本计划试点入口全过） |
| 转译 | `nextest … --features test-trans,test-cases --no-fail-fast` | 5327/5357，30 红全为预存（plan707 sse 帧时序 flake=4f123a50e 在案、plan606 画廊、ffi_dual_017、temp_plan009、KNOWN-DEBT 396 signature drift=Plan 744 fs.dir_names 遗留等）；**test_cases 0 红** |
| 无 GUI | `--no-default-features --features test-cases`（CLI 全链） | 24/24 exit 0 |
| 依赖 | `cargo tree -p auto-test-case` | 运行时仅 autodown-test-parser（vendor） |

注：examples/ 基线即非 fmt-clean（`a2r_step02/03/04、p023_probe、mpv_spike、
ui_accordion`），非本计划引入；本计划新文件全部 rustfmt 过。

## 5. 工作侧附带修复（预存缺陷，均有实证）

1. `--no-default-features` 编译失败（frame_bench 4×E0433，Plan 019 起遗留）
   → 2291cd78e：no-ui 按通道语义"未捕获=0"。
2. 新鲜二进制首跑 VM 必炸（`Static ID must be < 10000`：共享 next_id 被
   register_with_id 顶过 10000，register_shim_by_name 对动态段 id 走静态断言；
   探针实证 registry=239 / next_id=10015；libtest 二进制靠注册顺序侥幸）
   → 8a9f305cb：动态段 id 改走 dynamic 表。

## 6. 拟议 Spec delta（供 review 冻结；merge 才写 canonical）

- **SD-01**（add `docs/specs/auto-test-case/project.md`、`profile-v1.md`）：
  v1 支持表/schema/bytes/ref/suite/defaults/DAG/只读/report——以
  feasibility §8/§9 冻结表为准；`fixture_root` 单指针引用根；弱 oracle
  （裸 stage）拒绝；XFAIL/XPASS unsupported。
- **SD-02**（add `docs/specs/auto-test-case/architecture.md`、`plans.md`）：
  pinned vendor 来源（fba6563e+指纹守卫）、raw locator 规则、adapter 依赖
  方向（auto-lang→auto-test-case 单向）、大栈发射/manifest 预检/BLOCKED/
  proposal-only；执行边界=pilot（无通用 deadline/隔离）。
- **SD-03**（modify `docs/specs/auto-lang/trans/design/test-convention.md`）：
  双轨只读 pilot——emit 快照与（B 期）compile 分开；`module_name` 惰性、
  project 模式≠per-file 金样之和、Cargo.toml 目录派生名=输入身份。
- **SD-04**（modify `docs/specs/auto-lang/vm/overview.md`）：
  pilot/阶段分类（变体法）/weak 强化对照/旧 gate 边界（01_basics 等
  wrapper 为 #[ignore]，活跃门禁=发现式 corpus runner）。

## 7. 内部合同预检（AC-01–08 自检）

| AC | 自检结论 |
|---|---|
| AC-01 | 过：V-01 全绿；坏合同逐项定位；无静默忽略（未知字段/unsupported 显式 ERROR） |
| AC-02 | 过：V-02/09；指纹守卫防漂移；no serializer 载荷 |
| AC-03 | 过：3 双 check case 独立判定；actual/expected 分离（BLOCKED 链保 actual） |
| AC-04 | 过：强诊断=stage+code/miette；panic/缺 manifest=基础设施 ERROR 不满足语言负例 |
| AC-05 | 过：树哈希前后一致；scratch 唯一；proposal 只写提议目录 |
| AC-06 | 过：21 case/12 列 map 逐项对账=报告集合相等；多产物/强化/复活类必含；强化具名（map+README） |
| AC-07 | 过：required gap→exit 2；selector 未选=NOT_RUN；identity 漂移=stale（单测） |
| AC-08 | 过：轻核心（cargo tree）；no-default CLI 全链；t/tv/tt 旧 filter 触达 pilot；成本如 §2 |

剩余（review/merge 拥有，不计入 work）：AC-09 独立复审、AC-10 沉淀归档清理。
