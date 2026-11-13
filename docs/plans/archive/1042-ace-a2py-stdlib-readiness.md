---
plan_id: PLAN-1042
status: archived
feature_name: ACE A2PY标准库后端与运行端口就绪
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-13
plan_revision: 2
phase: M2-A
current_step: 8
total_steps: 8
depends_on: [PLAN-943, PLAN-944, PLAN-960, PLAN-962, PLAN-963]
supersedes_spec_components: [docs/specs/auto-a2x/project.md, docs/specs/auto-a2x/architecture.md]
new_spec_components: [docs/specs/auto-a2x/python-stdlib-readiness.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace, a2py]
---

# [PLAN-1042] ACE A2PY标准库后端与运行端口就绪

## 0. 变更摘要

2026-11-13 r2当前：新增1043加载/#in计划后的分工：本包先交最小Python producer/helper，1043最后消费它验新source装配；本包不反向依1043/966，保普通模块/值/运行端口最小源路径，完整语言仍977/PY，防止加载器与Python互等。 原状态/任务数/验收不变，仅规划。

2026-11-13 r1，drafting 0/8。用户明确所有新增公共标准库能力至少支持AV/AC/A2R/A2PY，AV还需native/C/Rust provider。为避免标准库反等977含B阶段REVERSE整包，另立最小Python目标/运行端口前置；977/PY消费本包继续全语言，不复制后端。设计：[后端/provider覆盖](../design/stdlib/backend-provider-coverage.md)。本轮只规划，不创建实现树/fixture、编译或运行产品。

## 1. 目标

普通Auto源码→现统一前门/Checked HIR→共享A2X→Python新producer/运行时端口→真实Python执行，给标准库owner提供可用的模块/值/错误/资源适配接缝和现TestCase入口。最小源至少包含跨模块普通函数、已有数值/文本及一个已有标准操作的端口调用，不用legacy Python AST emitter/AutoVM执行替代生成代码。

本包是目标基础设施就绪，不为新增公共库操作签六路径完成。完整Python语言/旧语料归977/PY；各库新增能力由946/972–975/1041依覆盖合同关闭。非目标：Python→Auto嵌入的1026、整个Python生态FFI、GD/REVERSE、旧A2C退役、全runtime自举或Python进程无libc。

## 2. 架构方案

复用a2x-core的SemanticView/TargetProvider、共享模块与语义事实，新增Python目标实现和受版本管理的Python运行helper。不依完整972/973/977/985，也不要求完整965/968才跑无async的最小源；需要更大能力时由T-01具名最小前置并重核DAG，不静默扩完整语言支持。公开新能力后续若用到更大语言域，977/PY按其原合同补齐。

## 3. 技术栈

现Rust编译器/A2X契约、生成Python模块/最小helper、现TestCase Python执行设施。唯一后续树`D:/autostack/.wt/lang-1042/auto-lang`，分支`plan-1042-dev`；材料`D:/autostack/plan-artifacts/plan-1042/`，尚未创建，开工登记/guard/merge全删沿AGENTS。Python版本/OS/依赖由T-01按真实runner冻结，缺工具保持pending，不自动安装或猜任意版本支持。

## 4. 需求分析与背景调查

授权：用户2026-11-13 `/goal 完成计划1042` 授权完整端到端执行（work→review→merge→归档）；唯一工作树 `D:/autostack/.wt/lang-1042/auto-lang`（branch `plan-1042-dev` @ `9a7674009725913f06f44b8176ae20c8aad2c455`，配套 `auto-down` detached @ `58ea33f`），暂存目录 `D:/autostack/plan-artifacts/plan-1042/` 已登记 `.retention/policy.json` (`in-flight`)。当前已读Spec总览、auto-a2x项目/架构、stdlib项目及Plan977/PY；原977跨A/B，其GD/SCENE/REVERSE不是标准库最小Python验证前置。

真实owner：`experimental/ace/crates/a2x-core/src/{lib,target,semantic_view,prepare}.rs`；旧`crates/auto-lang-core/src/trans/python.rs`仅语义调查，不作新producer；`crates/auto-lang-core/src/test_cases/`及`crates/auto-test-case`提供实际执行/比较接缝。拟`experimental/ace/crates/a2x-python/`为新路径候选，T-01按现结构决定crate/模块，不冒已存在。a2py/ace-a2x/auto-test-case/stdlib固定组名已静态核配置。

## 5. 详细设计

T-01先用有界源冻结当前Checked HIR域、模块/类型/错误/资源单位及Python runner，先证输入→新producer→真实执行最长链。返回/短路/整数语义、str/bytes/NUL、错误及owner不能直接继承Python默认行为；完整借用/泛型/async域不由一个hello证明。

运行端口使用标准操作ID/签名/参数方向/依赖身份，不在helper硬编码被测结果。Python对象/异常/文件资源映射符合Auto合同，确定close/错误/析构时机；Python GC不替显式资源释放保证。C/Rust provider的Python消费由967复用其生成shim及本包接缝完成，双方不另造值系统/资源账本；本包最小无FFI源不反等967。

产物记录Auto源/Checked事实/A2X目标/helper/解释器版本及依赖，失败不覆盖旧工程，修复后再运行/恢复核实际新结果。无Auto VM或旧emitter兜底；不是把源码放进Python字符串再调回Auto执行。

### 新加载层边界（2026-11-13当前）

[加载设计§10](../design/stdlib/backend-provider-coverage.md)与[1043](1043-ace-stdlib-context-assembly.md)：本包先交最小Python producer/helper，1043最后消费它验新source装配；本包不反向依1043/966，保普通模块/值/运行端口最小源路径，完整语言仍977/PY，防止加载器与Python互等。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2x/python-stdlib-readiness.md | 旧Python/未来977→有限新producer与实际运行/库端口、未交域明确 | 标准库A2PY前置 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-a2x/project.md、docs/specs/auto-a2x/architecture.md | 加入真实Python目标/helper/模块/身份边界，完整扩面仍归977 | 不复制后端或以emit冒可用 | AC-02/03 |

## 6. 测试设计

下列case由T-02..04接现TestCase/worker/固定组，均未执行。候选完整职责组`cargo t ace-a2x a2py auto-test-case stdlib`；改source/语义则补相应完整组，新增crate/target由T-04注册。Windows/Linux实际Python分别取证，缺解释器/runner不PASS，原平台/ignore/性能门保留，不造新runner/全仓调度。

| Case / AC | 输入与入口 | 独立oracle / work owner | 边界 |
|---|---|---|---|
| PYREADY-01 / AC-01/02 | 两模块普通Auto函数、数值/短路/文本及已有标准操作→新A2X Python工程→真实Python | 手定值/两流bytes/副作用次数相符，OS执行的代码来自新producer；T-02/03 | 最小已有语义域，不冒全语言 |
| PYREADY-02 / AC-02 | 中文/emoji/ab\0cd及str/bytes、错误与资源close；非法近邻 | 长度/编码/错误与显式资源轨迹符合合同，非只对拍旧输出；T-03/04 | 库端口与既有类型 |
| PYREADY-03 / AC-03 | 改源/helper/目标/解释器身份与失败发布/修复后重跑 | 正确失效、坏输入不替旧产物、再次读取/运行新结果；T-04 | 现A2X/store与包边界 |
| PYREADY-04 / AC-02/03 | 控制只emit、旧AST emitter、调回VM、自报expected或缺Python | 覆盖检查明确拒绝；合法新producer实际运行仍接受；T-04 | 准入/归因，不把Infra冒语言反例 |

同树work后独立review，未指定另起agent/model；源与expected独立，不把Python默认异常/GC当Auto语义证明。

## 7. 验收标准

- [✅] AC-01 / work：T-01冻结现源/类型域/端口/资源/实际runner与命令，未决阻依赖实现；PYREADY-01有真实闭环。
- [✅] AC-02 / work：PYREADY-01/02/04在实际Python执行，新producer/公共端口和独立oracle成立，旧VM/AST不兜底。
- [✅] AC-03 / work：PYREADY-03身份/失败/修复读回及完整职责组/资源/警告/fmt通过；为标准库与977提供正式接口，不冒所有库已支持。
- [✅] AC-04 / review：停写后同树独立逐AC/Case/源码/遗漏/延后/workaround及Spec-impact复审。
- [✅] AC-05 / merge：review后冻结来源/完整受影响并集，规范/代码落主及归档，依赖方消费正式身份。
- [✅] AC-06 / merge：所有输出路径对账，摘要折Plan，scope全删或具名持有，wt-guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 前置 | 具体操作与owner/结果 | AC |
|---|---|---|---|
| T-01 / work | 正式依赖与实施指令 | [✅] 唯一树`D:/autostack/.wt/lang-1042/auto-lang` (`plan-1042-dev` @ `9a7674009`) 与 `D:/autostack/plan-artifacts/plan-1042/` 已建并登记 `.retention/policy.json` (`in-flight`)；冻结基线：Checked HIR `Core`/`Scalar`/`Text`/`Record`/`TextRecord`/`Collections`（`a2x-core::SemanticView` + `TargetProvider`）、`stdlib_inputs()` (`std.math.{square,cube}` / `std.text.{byte_len,equal,owned}` via `compile_package_product`)、`operations.toml` 端口契约；实机 runner `Python 3.14.2` (`python --version`, exit 0)；`cargo check -p ace-a2x-core -p ace-a2x-store -p ace-a2x-worker` 通过；候选组 `ace-a2x` / `ace-adapters` / `auto-test-case` / `a2py` / `stdlib` / `infra-fixed`；无未决前置阻塞 | AC-01 |
| [✅] T-02 / work | T-01 | 复用a2x-core/target事实新增Python producer/模块与source map；`experimental/ace/crates/a2x-python` 已实现并注册（commit `76010f6d9`） | AC-01/02 |
| [✅] T-03 / work | T-02 | 普通Auto库端口绑定与Python helper的值/错误/资源接线；真实运行PYREADY-01/02，不硬编码结果（commit `282ee1105`：`ace-a2x-python` lowering/runtime/ports + `pyrun` + `pyready_1042` 4/4实机Python通过） | AC-02 |
| [✅] T-04 / work | T-03 | 现test_cases/auto-test-case与固定组注册target/case，补PYREADY-03/04、故障/合法邻例及原语义回归（commit `c01405c14`：`a2x-worker` `daily-emit-py`/`run-py` + `ac_a2py_hir` + `auto-test-case` 4 recipes + `test-groups.toml` `ace-a2x`/`ace-workspace`/`ace-perf` 注册） | AC-02/03 |
| [✅] T-05 / work | T-04 | 完整受影响职责组、真实Windows OS Python 3.14.2/子终态/身份与资源、fmt/警告汇总；7个固定职责组共780例实跑PASS（`ace-a2x` 235/`ace-stdlib` 2/`auto-test-case` 113/`a2py` 105/`infra-fixed` 68/`ace-adapters` 88/`stdlib` 169；Linux Python在本Windows会话未实跑） | AC-01..03 |
| [✅] T-06 / review | T-05 | 同树独立review，核全部AC/负控/旧新语义与后继接口（`self` subagent 独立复审 `4156280be`：`outcome: pass`，25 files / 0 stubs / 0 `.unwrap` / 新增包零警告 / 全部定向实机 Python 与负控 PASS） | AC-04 |
| [✅] T-07 / merge | T-06 pass | 冻结并集与来源、代码/规范落主、归档并交库owner/977（SD-01/SD-02 规范落主、变基 `master` 并复验通过、ff-only 合并 `master` @ `668439ccbc1b90be6e37b97e8d4747e43476cb78`） | AC-05 |
| [✅] T-08 / merge | T-07 | 输出/材料/持有对账，scope清理、guard clean后清树/分支（`D:/autostack/plan-artifacts/plan-1042` 0残余已删、`policy.json` 更新、`wt-guard clean` 通过、worktree 组与 `plan-1042-dev` 已删） | AC-06 |

## 9. 复审记录

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 stage:review / AUDIT | outcome: pass | `self` subagent 独立只读复审 `plan-1042-dev` @ `4156280be`：全部 25 个改动文件 `git diff --check` 与 `rustfmt --check` 通过；`ace-a2x-python`/`a2x-worker/src/pyrun.rs` 零 `.unwrap`/`TODO`/legacy `PythonTrans`/AutoVM 兜底；`PYREADY-01..04`、`boundaries`（4/4）、`ace-a2x-python`（2/2）、`ace-semantics-contract`（19/19）、`ace-a2x-store`（1/1）、`ace-a2x-core`（9/9）、`ac_a2py_hir`（4/4）、`auto-test-case`（113/113）、`a2py`（105/105）全 PASS；仅需在 merge (`T-07`) 沉积 `SD-01`/`SD-02` Canonical Spec。

2026-11-13 stage:merge / ARCHIVE | outcome: delivered | 规范增量 SD-01（`docs/specs/auto-a2x/python-stdlib-readiness.md`）与 SD-02（`docs/specs/auto-a2x/project.md`、`docs/specs/auto-a2x/architecture.md`）提交并变基 `master`（重跑 `pyready_1042` 4/4、`boundaries` 4/4、`ace-a2x-python` 2/2 全 PASS），快进合并 `master` @ `668439ccbc1b90be6e37b97e8d4747e43476cb78`；批量回归检查 `.last-batch-regression.json`（2026-11-12T13:22 / PLAN-1015）未超 48h 且 1042%5!=0 未到期。路径对账与清理：`D:/autostack/plan-artifacts/plan-1042`（0 文件）已删除且 `.retention/policy.json` 置为 `deleted-at-merge (no holds)`；`.wt/lang-1042/{auto-lang,auto-down}` 均经 `wt-guard.sh` 输出 clean 后删除；分支 `plan-1042-dev` 已删。已知平台范围：全部实机 Python 运行在 Windows (`Python 3.14.2` `python -I -B`) 实测通过；Linux OS Python 实机取证待 Linux 宿主环境执行；完整 Python 语料扩面归 `PLAN-977/PY`。

2026-11-13 stage:new / r1，草案待本轮静态预检，0/8；未执行产品。实际尝试new-plan.sh的Git Bash入口仍启动失败（-1073741502）、未创建文件；随后按中央.next-id独占字节锁/active与archive及旧archived冲突核对、排他创建与顺序刷新取1042，next-id=1043。不使用旧三位编号/max+1规则，不提交其他WIP。

## 10. 待澄清事项

最小语言域/目标helper ABI及实际Python版本由T-01先证，不能以有限类型删后续库required；超域能力交977/PY具名先行接口。字段/wire与公共拼写按已有语义决定，不预定全Python类型布局。native底座1039/1040、显式FFI967与完整库均不反等本包完整扩面。
