---
plan_id: PLAN-741
status: reviewed
feature_name: AC 首个闭环：独立 Atom HIR、语义校验与 Windows 原生 AOT
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-05
plan_revision: 2
current_step: 17
total_steps: 17
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-ac/project.md
touched_goals: []
affects: [auto-hir, auto-ac]
---

# [PLAN-741] AC 首个闭环：独立 Atom HIR、语义校验与 Windows 原生 AOT

## 0. 变更摘要

在 v0.6-dev 建立独立 Rust 原型，从 Schema-bound Atom HIR 文本经过类型化绑定与 verifier，
生成 Windows x86-64 COFF 对象并链接、运行 PE 可执行文件。
覆盖现有加法、计数循环、调用求值顺序样例，并增加原生入口/拒绝样例。
交付“HIR → 原生执行”，不宣称“Auto 源码 → 原生执行”或 AAC 自举完成。

本计划不修改 v0.5 的 parser/VM/A2X/旧 Atom，不拆仓，不依赖其他仓未 push 的代码。
这是计算核心 profile 的首条纵向实现；完整计算子集、AST adapter、B/C 子集和生产 ABI
由后续计划完成。静态库、DLL、热重载仍是 v0.6 主线，不因 exe 里程碑而延后。

## 1. 目标

1. 建立独立于 auto_val::Value 的有限 Atom 数据层与 core HIR 契约。
2. 分别检查语法绑定、结构、HIR 语义和后端能力，并提供可定位诊断。
3. 保持类型身份、作用域、块顺序、调用求值顺序、可变性与明确数值行为。
4. 在实际 Windows x64 工具链上生成、链接并运行真实机器码。
5. 保持公共 HIR 与 target lowering 的边界，提供独立复审所需证据。

非目标：完整 Atom v2/Batom/Schema DSL 引擎、自动 struct codec、Lisp/CST 无损往返；
旧前端接入、堆对象/所有权、i64/浮点/泛型/结构体/enum 值、完整 A 子集；
AVM/A2X 迁移、生产 ABI 冻结、DLL/hot reload 实现、AAC、Linux 发布验收、性能领先承诺。
上述是本计划起草时的边界，不得在执行中删掉已列入的验收项。

## 2. 架构方案

    Atom text → SyntaxNode/Atom → core descriptor/binder → Unchecked HIR
                                                            ↓
                                                     semantic verifier
                                                            ↓
                                                       Checked HIR
                                                            ↓
                                                  native capability check
                                                            ↓
                                                   backend lowering
                                                            ↓
                                                       COFF .obj
                                                            ↓
                                              startup + platform linker
                                                            ↓
                                                   Windows PE .exe

新增 experimental/ac-core/，使用独立 Cargo workspace（拟新增路径）。
package 暂定 auto-ac-prototype，binary 暂定 ac-probe。
Cargo.toml 明确 [workspace]，依赖显式声明，独立 Cargo.lock/target；
不改根 Cargo.toml/Cargo.lock，不依赖 auto-lang/auto-atom/auto-val/auto-man。
根 Cargo 配置的继承与 patch 行为由 T-01 实测，避免拉入旧 UI 依赖。

数据层暂用 Rust enum/Vec/typed ID 表，不冻结 cell/Batom 布局或最终 crate 拆分。
后端只能消费 verifier 生成的 CheckedModule，构造私有；文件中的 checked 标记无授权作用。
使用版本化 core descriptor 做有限字段/槽/判别分支校验；
这不冒充已实现通用 Schema DSL，也不把一次性 JSON 映射当作新 Atom。

## 3. 技术栈

- Rust + 独立 Cargo workspace；实测 rustc -Vv，记录精确依赖版本与 lockfile。
- Cranelift codegen/frontend/module/object 为优先候选，T-01 查官方资料并验证选型。
- Windows 11 x86-64，target profile 为 x86_64-pc-windows-msvc。
- 平台 linker 评估 lld-link/link.exe/rust-lld COFF 模式，记录路径、版本与 SDK/import libs。
- 最小启动包装器、ExitProcess 和确定的 trap 路径。
- Rust 集成测试与 PowerShell 原生验收脚本，所有运行子进程有截止时间。

若 Cranelift 必须能力不成立，提交有证据的选型结论并修订合同；
不能静默换成 a2r/C 转译、JIT、解释执行或忽略 native 测试满足验收。
本计划不要求第一步手写寄存器分配、机器码编码或完整 linker。

## 4. 需求分析与背景调查

### 4.1 授权、编号与分支例外

- 用户明确要求使用 D:/autostack/auto-musk/.agents/skills/auto-plan-new/SKILL.md，
  在 v0.6-dev 建立首个计划，编号从 741 开始；主机器最新计划据用户记忆为 740。
- 本地 active/archive 最大编号实查为 726，两检出的 .next-id 原为 727。
  727–740 为主机器已有/未同步工作保留，不创建空计划补号，也不声称已看到远端740。
- 本轮独占分配锁、复查编号后，将本分支计数器调为741并运行 scripts/new-plan.sh，
  创建唯一741文件，.next-id=742；主检出 master 文件不变。
- 用户明确指定在 v0.6-dev 起草，优先于 skill/AGENTS 的默认 master 簿记位置。
  本轮授权仅创建和保存计划/设计；不执行实现、不创建实现 worktree、不 push。
- 确认执行后，从 v0.6-dev 已提交计划/设计基线创建
  D:/autostack/.wt/lang-741/auto-lang、plan-741-dev；计划进度回写 v0.6-dev。
  离线阶段 merge 目标为 v0.6-dev，不直接合入 master；本计划不发布版本。
- 没有用户指定预算、自动续跑限制或跨仓修改授权。

### 4.2 当前 Specs、代码与设计证据

| 来源 | 观察与结论 |
|---|---|
| docs/specs/overview.md | 提供 monorepo 索引，但日期快照需与代码核对 |
| docs/specs/auto-atom/project.md、crates/auto-atom/Cargo.toml | 旧 Atom 依赖 auto-val，本计划不扩写该实现 |
| docs/specs/auto-lang/frontend/overview.md | 当前源码→AST 与 ToNode/ToAtom 不等于 Checked HIR |
| docs/specs/auto-lang/types/overview.md | 类型/推导/所有权状态；本计划只定义精确 i32/bool |
| docs/specs/auto-lang/trans/overview.md | 当前 AST→源码转译，不是新 native pipeline |
| parser.rs::parse_ident_or_generic_type、trans/rust.rs::rust_type_name | int/i32 同归 Type::Int，A2R 发 i64；后续 AST adapter 核定映射 |
| docs/design/strategy/auto-hir-design.md、auto-hir-atom-text.md | HIR 身份/顺序/四份文本附件及拒绝矩阵 |
| docs/design/strategy/atom-batom-v2-design.md、atom-schema-dsl-design.md | 独立数据模型、槽绑定、有限判别分支；不要求先实现完整格式 |
| docs/design/strategy/auto-native-abi-rfc.md | Windows 首个平台、独立 ABI、模块目标；探针不冻结生产 ABI |
| docs/design/strategy/auto-ac-subset-and-target.md | A0/A1 与 A/B/C 分期；741完成A0，A1接完整基线 |

起草时 branch HEAD=6bd64072ecd173ac8d7964f8c597f7580da05911，
旧 HIR 文本样例快照=a404dfebc；本轮设计补充与计划一并保存。
以下 SHA-256 供复审定位，执行前重算并记录变化：

| 文件 | SHA-256 |
|---|---|
| docs/specs/auto-atom/project.md | 822a2f11184868495b94c75cb9f1797ef3e4e42031cbf8cf8ad20349270dddec |
| docs/specs/auto-lang/frontend/overview.md | fccb811a79b24b95678efa57b87bcc9ded10e5afd2b9abc44236b99aebb16dfe |
| docs/specs/auto-lang/types/overview.md | b027459f0339bcca81c3874b8eee251026e86706ed84db8f1fb01ee9145b2fcf |
| docs/design/strategy/auto-native-abi-rfc.md | 5fb6592390a95abec3a28eda4a50bd478922b14b0a052d8d2e39fad8825776b8 |

现有 Specs 尚无新 HIR/AC 实现规范；本计划提出新增两份，不将旧 Specs 改写成已迁移。
GOAL-017 描述已完成的 AAVM/AA2R 自举，不把本计划算作其重复完成。
touched_goals 暂为空，关联 v0.6 roadmap AC/HIR 主线；新的 native 目标后续独立登记。

### 4.3 工具链观察与缺口

确认 cargo/rustc 在 PATH、VS2022 BuildTools 安装目录与 Rust toolchain 的 rust-lld 存在。
普通 PATH 未发现 link/lld-link，常用 LLVM 安装路径不存在；
未确认 SDK/import libs、环境初始化及版本兼容，T-01 负责探测。
目录存在不等于 native 闭环可执行，缺工具链不能用 mock 成功代替。
相关主机器计划739/740不可见，本次隔离路径和旧入口，恢复后仍须做差异核查。

## 5. 详细设计

### 5.1 有界 Atom 输入与 TypeUse

- 只接收公布的 core-i32-draft descriptor 身份/revision，未知 tag/case/字段/capability 明确拒绝。
- 支持样例用到的 node 双槽、命名 args/body、Obj、Array、字符串、bool、enum，
  i32/u32 数字后缀；保留 source byte span、语句顺序与字段出现次数。
- primary 按 Schema 绑定文本 ID；secondary 绑定 enum 或 TypeUse；
  头/args/body 同字段重复报错，不猜 id/name/text，不按前缀猜引用类别。
- TypeUse 接受基本 enum i32/bool 和字符串类型引用。local 双槽 i32/bool 绑定内建，
  其他标识符绑定类型表；函数签名仍保留结构化定义/引用。
- 内建 enum 与指向同一内建类型的引用语义归一；不得任意折叠名义 newtype。
- typed IDs 与 owner 表区分类别/作用域；Obj 字段顺序不决定执行，Block 内容有序。
- canonical writer 用全命名形式，重新读取语义等价；不要求保留注释/排版。
- 四份旧引用形式 fixture 与新增 enum 简写都验收，不强迫全部样例改写。

### 5.2 core HIR 与 verifier

支持 function/intrinsic 签名，param/binding locals，local place；
constant/read_local/binary/call；let/return/if/loop/assign/break/continue。
操作符先覆盖 add_i32/mul_i32/lt_i32，加/乘必须声明 overflow:trap；
此范围不宣称全 A 子集或全部算术运算符。

验证引用类别/owner、签名/参数绑定、结果类型、初始化/可变性、loop target，
块包含无环/入口角色唯一、返回路径及表达式求值位置。
if 出口初始化取可达分支交集；循环按可能零次迭代保守处理；
仅在循环内初始化不保证出口已初始化，终结语句后的可执行尾语句拒绝。
表登记顺序不是执行顺序；循环条件/步进每轮重新求值。
eval_args 决定调用副作用顺序，bindings 决定结果到形参映射。
递归用 DefRef，不复制 body；有副作用 ExprId 不可被多个求值位置共享。
未知/错误类型不进入 Checked HIR，深递归资源策略不作语言承诺。

### 5.3 native lowering、入口与 trace

Checked HIR 不存寄存器/SSA；native pass 将局部/place/结构化块降到后端表示。
使用目标 Windows 调用约定，明确 bool 内部和传参表示，不称生产 Auto ABI 已冻结。
溢出 trap 使用独立错误退出路径，不依赖 Rust debug overflow。
本测试 profile 的 overflow trap 固定退出码70，正常测试入口退出0；这不是全语言异常 ABI。

CLI 构建入口为零参数、返回 i32 的已声明 DefRef，签名不匹配/不存在则拒绝。
add/count 由新增 HIR 零参数测试入口调用；生成包装器通用，不按 fixture 名硬编码结果。
最小启动调用所选 entry 再调用 ExitProcess，Auto 函数不直接作为 OS 加载入口。
保留对象和 linker 收据，失败构建不覆盖既有成功制品。

test.trace 由最小原生宿主支持库实现 mark_a/mark_b，
实际原生调用记录 b,a，返回值12；不生成 Rust 版被测函数、不解释 HIR。
普通后端无该能力时拒绝03-call-order；测试模式显式提供能力才允许链接。
这不是生产字符串/I/O/插件 ABI，支持库的语言可用 Rust/C，但仅负责启动/trace服务。

### 5.4 新建路径与 CLI

以下均拟新增，等价模块调整需记录：

    experimental/ac-core/
      Cargo.toml / Cargo.lock / README.md
      src/{lib,main,atom_text,descriptor,hir,verify,native,link}.rs
      schema/core-i32.atom
      tests/{text_binding,hir_verify,native_execution}.rs
      fixtures/{valid,invalid}/
      test-support/
    scripts/verify-ac-741.ps1
    docs/reports/741-ac-hir-native/{toolchain,verification}.md

CLI：ac-probe check <file>；
ac-probe build <file> --entry <DefId> --output <exe>。
check 成功0，拒绝非零并标注阶段与 span/ID；build 成功留下 .obj/.exe/link收据。
产物放 experimental/ac-core/target/，脚本对子进程设置显式超时。
文件读取、诊断、简单驱动可作为宿主 Rust 工具能力，不反推 Auto 已支持文件库。

### 5.5 Phase 2 设计（plan_revision 2）：P741-QA-01..07 修复

2026-10-04 外部复审（docs/reports/741-quality-review-20261004/REVIEW.md，needs_fix）
揭示四项 P1 缺陷与三项次要问题。方向：修复实现以兑现 r1 已沉淀的 SD-01/02 承诺，
不缩减 Spec。以下逐项设计与复审 finding 对应：

- **QA-01（bool 表示）**：`native.rs` 全部 `icmp` 结果（LtI32、checked_add 溢出链、
  checked_mul 溢出判定）统一 `uextend i8→i32` 后再进入变量/分支/调用/返回，
  内部 bool 值恒为 i32 0/1（SD-02 原文"bool 内部/传参均为 i32 0/1"）。
  `lower_object` 对 body 参数下标访问补越界防御，超界返回 backend 诊断而非 panic。
- **QA-02（bindings 映射）**：`verify.rs` 的 call 实参类型检查从"按 eval_args 下标对位"
  改为按 bindings 映射：先独立校验映射本身（param 全覆盖唯一、arg 越界、arg 未被任何
  binding 消费、arg 被多个 binding 重复消费均拒绝），再按 `params[b.param]` 与
  `type(eval_args[b.arg])` 逐位核对；eval_args 仍按序求值（副作用顺序契约不变）。
- **QA-03（owner 双向）**：`verify.rs` check_module 增加 function→body 反向遍历：
  每个 Function def 的 body 字段必须指向 owner 恰为该 def 的 body，否则
  `verify.owner-mismatch`（共享 body、同签名共享、冒领 body 全在此拒绝）；
  既有 body→owner 方向检查保留。
- **QA-04（发布事务）**：`link.rs`/`main.rs` 改为"全部制品先暂存、再发布、失败回滚"：
  link 只产出唯一暂存名 exe（`.tmp-ac741-<pid>`）；新 `publish_artifacts` 先写暂存收据，
  备份既有 exe/obj/收据为 `.bak-ac741-<pid>`，再依次改名发布 exe/obj/收据；任一步失败
  则删除已就位新制品、还原备份、清理暂存并返回错误；成功后删除备份。obj 暂存名同步加 pid。
- **QA-05（README）**：命令改为实际 bin 名 `auto-ac-prototype` 的正确 cargo 调用
  （与 verify-ac-741.ps1 一致，去掉多余的 ac-probe argv）；build 示例改用带 d_entry 的
  `fixtures/native/add-2-3.atom`；计划链接更新到 `docs/plans/archive/`；
  descriptor 路径笔误 `schema/schema/core-i32.atom`→`schema/core-i32.atom`；
  验收方式=从 README 逐字复制命令运行。
- **QA-06（截止时间）**：`run_with_deadline` 重构为并发读管道（stdout/stderr 各起
  reader 线程，避免先等退出再读导致的管道阻塞）+ deadline 参数化；`find_rust_lld`
  （rustc --print sysroot）与 `find_sdk_um_dir`（reg query）统一接入该执行器。
  单测用可控 helper（cmd/ping）证明超时终止与大输出不阻塞，不终止真实系统工具。
- **QA-07（warning）**：`tests/common/mod.rs` 删除未使用的 descriptor 导入；
  `verify-ac-741.ps1` 增加 `cargo check --all-targets` 覆盖测试目标的健康检查。

### 规范增量（Phase 2 追加行）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-hir/project.md | 无实现规范→有限文本/TypeUse/校验/顺序契约及限制 | 沉淀实证能力，不宣称完整HIR | AC-01,AC-02,AC-03,AC-04 |
| SD-02 | add | docs/specs/auto-ac/project.md | 无native实现规范→Checked HIR AOT/Windows工具链/入口/trap/测试能力 | 区分原型、生产ABI与源码编译 | AC-05,AC-06,AC-07,AC-08 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 语义契约两点明确化：call 实参类型按 bindings 映射逐位核对（arg 恰被一处 binding 消费）；函数↔body 双向归属唯一（共享/冒领 body 拒绝） | QA-02/03 复验 SD-01 原承诺，实现修正而非缩规约 | AC-10,AC-11 |
| SD-04 | modify | docs/specs/auto-ac/project.md | bin 名澄清为 auto-ac-prototype（ac-probe 为历史文档名）；制品发布措辞=暂存+备份+发布+失败回滚；硬截止时间覆盖全部子进程（工具发现/链接/运行） | QA-01/04/05/06 复验 SD-02 原承诺 | AC-09,AC-12,AC-13,AC-14 |

Spec 文件在 review/merge 根据实现沉淀，本 new 阶段不编辑 canonical Specs。
overview/.autoos/specs.json/index 在 merge 按实际模块布局登记。

## 6. 测试设计

| 测试族 | 输入/操作 | 预期 |
|---|---|---|
| text_binding | 四份已有文件、基本enum、字段位置互换 | 真实读入，author/explicit/简写/往返语义一致 |
| text_binding拒绝 | 重复字段、错误槽/enum/范围、未知字段/profile | 结构诊断，不进入lowering |
| hir_verify | 三组语义样例、入口包装、ID重映射/表重排 | 有效且顺序/身份不变 |
| hir_verify拒绝 | HIR文本§9矩阵、跨body LocalId、分支/循环未初始化、块环、求值共享 | 分别命中错误类别，无Checked句柄 |
| native_execution | add(2,3)/(-2,3)、count(3/0/-2) | 原生结果5/1/3/0/0；测试入口exe退出0 |
| native_execution边界 | i32最大值+1、乘法溢出 | 退出码70，不能回绕成正常结果 |
| native_execution顺序 | 03-call-order，显式测试capability | 原生日志b,a与结果12；无能力build拒绝 |
| CLI/process | 入口不存在/签名错误、linker缺失、运行超时 | 可定位非零错误，不静默skip、不覆盖成功制品 |
| hir_verify Phase2 | bindings swap 合法映射接受；swap 类型不匹配/arg 重复消费/arg 未消费拒绝；共享 body（异签名/同签名）与冒领 body 拒绝 | verify.type-mismatch / verify.binding-invalid / verify.owner-mismatch 按码命中 |
| native_execution Phase2 | bool 条件正例（local+if）、bool ABI 正例（形参/实参/返回）、bindings swap 正例 | 原生退出码 = fixture 设计值（3/5/9） |
| cli Phase2 | 发布阶段失败（收据路径为目录占位）：重建同目标 | exit 1 + link.receipt；exe/obj 哈希不变；旧 exe 原生退出码不变；无 .bak/.tmp 残留 |
| link 单测（内嵌） | 可控挂起 helper 超 1s 截止；大输出（>64KB 管道）子进程 | link.deadline 终止；输出完整不阻塞 |

执行命令（路径/测试族由本计划新建）：

    cargo check --manifest-path experimental/ac-core/Cargo.toml --locked
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_binding
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test hir_verify
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test native_execution -- --test-threads=1
    cargo fmt --manifest-path experimental/ac-core/Cargo.toml -- --check
    powershell -NoProfile -File scripts/verify-ac-741.ps1

CLI例：cargo run --manifest-path experimental/ac-core/Cargo.toml --locked --bin ac-probe -- check docs/design/strategy/hir-examples/01-add.atom。
native脚本发现工具链、构建/运行、处理超时并保存收据；SDK缺失为 blocker，
不能 ignore native测试再宣称通过；其他平台的reader/verifier不能替代Windows原生门禁。

最终按 AGENTS 编译器改动档跑一次 cargo check -p auto-lang、cargo t、cargo tv；
开发迭代仅跑原型定向测试。旧测试红需同基线对照、证据与复审裁定。
不触 aavm/UI/trans/book 时不跑 taa/tu/tt/tb；本worktree不跑 tf/ta/t3。
本轮起草为doc-only，不运行上述实现测试或docs_gen。

## 7. 验收标准

- [ ] **AC-01**：依赖树无旧产品依赖；根workspace/lock及旧生产代码diff为零。
- [ ] **AC-02**：四份已有文本与基本enum经过真实reader/binder；同义/往返一致，重复字段等反例拒绝。
- [ ] **AC-03**：引用/类型/初始化/可变性/控制流/求值位置反例全部拒绝，后端无法消费unchecked输入。
- [ ] **AC-04**：core descriptor、TypeUse、顺序与能力边界明确；未知项拒绝，无源码/VM回退。
- [ ] **AC-05**：由HIR lowering生成真实Windows COFF与PE，add/count满足§6，工具链/链接/运行收据完整。
- [ ] **AC-06**：加/乘溢出trap；trace原生结果b,a和12；无能力build拒绝。
- [ ] **AC-07**：CLI/脚本可复现；入口/工具链/超时错误非零；失败构建不覆盖成功制品。
- [ ] **AC-08**：定向与最终门禁、独立复审和SD-01/02证据完整；未将A0宣称源码编译/生产ABI/AAC。

Phase 2 追加（plan_revision 2，2026-10-04）：

- [ ] **AC-09**（QA-01）：bool 合法程序原生闭环——bool local 赋值/读取/if 条件、bool 形参传递、bool 返回值程序 check=0、build=0、原生退出码为 fixture 设计值（3/5）；复审复现脚本 bool-local（原 panic 101）与 bool-return（原 build 1）变为构建成功且退出码正确。
- [ ] **AC-10**（QA-02）：call 实参类型按 bindings 映射核对——binding-valid（swap 映射、类型匹配）check=0 且原生运行 exit 9；binding-invalid（swap 映射、类型不匹配）check 阶段拒绝 verify.type-mismatch（不再漏到后端）；arg 重复消费/未消费在 check 拒绝。
- [ ] **AC-11**（QA-03）：function↔body 双向归属——shared-body（零参 alias 共享双参 body）、同签名共享、冒领 body 均 check 阶段拒绝 verify.owner-mismatch；native 不再出现 index-out-of-bounds panic。
- [ ] **AC-12**（QA-04）：发布阶段失败不覆盖——以成功制品（原生 exit 5）为基线，收据路径目录占位后重建：CLI exit 1 + link.receipt 诊断，exe/obj 哈希不变，旧 exe 原生运行退出码仍为 5，无 .bak/.tmp 残留；成功构建后同样无残留。
- [ ] **AC-13**（QA-05）：README 中命令逐字复制可运行（check exit 0、build 产出可运行 exe）；计划链接指向 docs/plans/archive/；descriptor 路径无 schema/schema 笔误。
- [ ] **AC-14**（QA-06）：工具发现（rustc/reg）、链接、运行全部子进程经带硬截止时间的执行器；执行器并发读管道；单测证明：可控挂起 helper 在截止处被终止报 link.deadline，>64KB 输出子进程正常完成且输出完整。
- [ ] **AC-15**（QA-07）：tests/common/mod.rs 无未使用导入；`cargo check --all-targets` 零 warning 并纳入 verify-ac-741.ps1 健康检查。

## 8. 执行步骤

所有步骤尚未执行；完成记录commit、命令、结果与AC。

### 执行进度（worktree plan-741-dev，基线 951b6c70f）

- [x] **T-01** commit `0f2b232d7`：rustc 1.98.1/msvc host、SDK 10.0.26100.0（D盘，注册表 KitsRoot10）、MSVC 14.44 实测；Cranelift 0.126.2 选型（cargo tree 无旧产品依赖）；最小 COFF/link/call 探针 rust-lld 与 MSVC link.exe 双通过（exit 5）。收据 `docs/reports/741-ac-hir-native/toolchain.md`。AC-01, AC-05。
- [x] **T-02** commit `632e48a03`：atom_text.rs（span 分词/解析）+ descriptor.rs（schema/core-i32.atom 版本化 descriptor、双槽/命名字段互换、重复字段、分支 require/forbid、类别失配/悬空引用诊断、canonical writer）+ hir.rs 类型化 ID。text_binding 11/11：四份设计文档绑定、作者/显式/canonical 往返语义相等、enum 简写、15 反例按码拒绝。AC-02, AC-04。
- [x] **T-03** commit `b3bed7d72`：verify.rs（CheckedModule 私有构造；类型/初始化交集/循环零次保守/可变性/loop 作用域/块唯一入口角色/可达性/终结后尾语句/返回路径/求值位置唯一/表达式无环）。hir_verify 8/8 覆盖 §9 矩阵+组合反例。AC-03, AC-04。
- [x] **T-04** commit `cf5aca9f8`：native.rs（capability gate、find_entry、结构化 lowering、可移植有符号溢出检测、trap=ExitProcess(70)、通用 ac_start）。native_execution 8/8：add(2,3)=5、add(-2,3)=1、count(3/0/-2)=3/0/0 真实 PE 退出码；溢出 trap 70×2；COFF 符号断言。AC-05, AC-06。
- [x] **T-05** commit `523fabfe1`：link.rs（rust-lld+SDK 发现、子进程截止时间、原子制品替换、链接收据）+ main.rs（ac-probe check/build，退出码 0/1/2）。cli 9/9：错误路径全非零、SENTINEL 不覆盖、add.exe 运行 exit 5。AC-05, AC-07。
- [x] **T-06** commit `53775205c`：test-support/ no_std 静态库（hir.test.mark_a/mark_b 经 kernel32 写 stderr）+ `--support-lib`。trace_execution：无能力拒建；有能力+支持库原生跑通，stderr b→a、exit 12。AC-06。
- [x] **T-07** （本提交）README + scripts/verify-ac-741.ps1（§6 命令 + CLI 实机构建/运行 + 主仓门禁，全部子进程带截止时间）+ verification.md。原型门禁 11/11 PASS；主仓门禁结果见 verification.md §4。AC-01..AC-08 证据表。
- [ ] **T-08** /auto-plan:review 独立核对（执行侧不预先勾选）

组内依赖：auto-lang 的 autodown-core 可选路径依赖需组内兄弟检出，
已按依赖 worktree 规则建 `D:/autostack/.wt/lang-741/auto-down`
（detached @ fba6563e，只读，无 junction/symlink；fold 前随主 worktree 一并
wt-guard 后移除）。


| task | 依赖 | 文件/符号与动作 | 验证命令/预期 | AC |
|---|---|---|---|---|
| T-01 | 执行确认 | 从v0.6-dev建专用worktree；探测Rust/SDK/linker；新增Cargo.toml/lock；toolchain.md记录backend选择 | rustc -Vv、cargo tree --manifest-path experimental/ac-core/Cargo.toml、最小COFF/link/call探针；无旧依赖 | AC-01,05 |
| T-02 | T-01 | atom_text.rs/descriptor.rs、schema/core-i32.atom；有限文本/槽/重复字段/TypeUse/span | text_binding测试；四文件/同义/反例通过 | AC-02,04 |
| T-03 | T-02 | hir.rs/verify.rs；typed IDs、Unchecked/Checked、语义/数据流/求值检查 | hir_verify；拒绝矩阵稳定诊断 | AC-03,04 |
| T-04 | T-03 | native.rs；函数/locals/place/算术/块/调用lowering、对象与trap | native_execution加法/循环/溢出对象探针正确 | AC-05,06 |
| T-05 | T-04 | link.rs/main.rs；入口/启动、链接/CLI、原子制品替换 | check/build与add/count exe；失败非零、不覆盖 | AC-05,07 |
| T-06 | T-05 | test-support与调用顺序runner；显式capability | native_execution顺序族；b,a/12与无能力拒绝 | AC-06 |
| T-07 | T-06 | README、verify-ac-741.ps1、verification.md；定向与最终门禁 | §6命令通过/失败有证据；无未处理warnings/debug | AC-01,AC-02,AC-03,AC-04,AC-05,AC-06,AC-07,AC-08 |
| T-08 | T-07 | /auto-plan:review独立核对AC/SD、遗漏/规避/债务，绑定revision | 每项有代码/执行证据，pass后才merge沉淀 | AC-08 |

起点为已提交revision1，不复制WIP；一计划一worktree，无junction/symlink。
后续清理须wt-guard与merge收据；进度/合入目标v0.6-dev。
主机器恢复后的master/v0.5整合另行处理，不顺手继续旧收尾。
T-01/04使用临时native探针验证能力，但最终验收必须来自实际Checked HIR流水线。

### Phase 2 执行进度（plan_revision 2，2026-10-04 起）

依据：外部复审 needs_fix（docs/reports/741-quality-review-20261004/REVIEW.md，
reviewed_commit c1ac219e7）。**2026-10-04 用户裁定：不另开新计划，激活本文件以
plan_revision 2 追加 Phase 2**（优先于复审报告"另开独立合同"的建议）。
Phase 1 归档记录与 T-01..08 勾选保持原样不改。worktree 重建：
`D:/autostack/.wt/lang-741/auto-lang`，分支 `plan-741-dev`（旧分支已随 r1 fold 删除，
从 v0.6-dev 含本 Phase 2 合同提交重建）。改动面=experimental/ac-core + scripts/verify-ac-741.ps1
+ README/spec（SD-03/04 在 merge 时沉淀），crates/** 零改动。

| task | 依赖 | 文件/符号与动作 | 验证命令/预期 | AC |
|---|---|---|---|---|
Phase 2 执行进度（commit 哈希=plan-741-dev；基线 3a7967262）：

- [x] **T-09** commit `28c0bbfcd`：native.rs icmp 全点位 uextend 归一 i32 + lower_object 参数下标越界诊断化；fixtures/native/{bool-if,bool-abi}.atom；native_execution 11/11（bool-if exit 3、bool-abi exit 5）。AC-09。
- [x] **T-10** commit `28c0bbfcd`：verify.rs call 检查改 bindings 映射制（映射独立校验+按映射类型核对，arg 恰消费一次）；fixtures native/binding-swap.atom、invalid/{binding-swap-type-mismatch,binding-dup-arg}.atom；hir_verify 9/9；native swap exit 9。AC-10。
- [x] **T-11** commit `28c0bbfcd`：verify.rs check_module 增加 function→body 反向 owner 校验；fixtures invalid/{shared-body-owner,shared-body-same-sig,stolen-body}.atom 全部 verify.owner-mismatch 拒绝。AC-11。
- [x] **T-12** commit `db75c494f`：link.rs link_object_staged/publish_artifacts 暂存-备份-发布-回滚事务（companion 全名后缀防同名 stem 备份碰撞——开发中由回滚测试实抓并修复）+ main.rs 接线；cli 10/10 含发布失败回滚测试（exe/obj 字节不变、旧 exe exit 5、无 .bak/.tmp 残留）。AC-12。
- [x] **T-13** commit `6ad396379`：README 命令改真实 bin 调用（auto-ac-prototype，与验证脚本一致）、build 示例改 add-2-3 fixture、归档链接、schema/schema 笔误；三条命令逐字复制 PowerShell 运行全过（check 0、add.exe exit 5、trace.exe exit 12 + stderr b,a）。AC-13。
- [x] **T-14** commit `db75c494f`：run_with_deadline 并发读管道+deadline 参数化；rustc/reg/lld/run 全部接入；lib 单测 2/2（1s 截止杀挂起 helper、>128KB 输出子进程完整跑通——旧实现该用例必撞 60s 死锁）。AC-14。
- [x] **T-15** commit `6ad396379`：tests/common 未用 descriptor 导入清除；verify-ac-741.ps1 增加 all-targets 零 warning 步（`^warning:` 断言）与 --lib 步。AC-15。
- [x] **T-16** （本提交）门禁复跑：text_binding 11/11、hir_verify 9/9、native_execution 11/11、cli 10/10、trace_execution 1/1、lib 2/2（计 44 项，r1 基线 37 项）；cargo fmt --check 过；cargo check --all-targets 零 warning；复审 reproduce.ps1 复跑六反例全部翻转（bool-local/bool-return build 由 101/1→0、binding-valid check 1→0、binding-invalid check 0→1、shared-body build 101→check 1、发布失败 exeChanged true→false 且旧 exe exit 5）；`git diff 3a7967262 -- crates/ Cargo.toml Cargo.lock test/` 为空。AC-09..AC-15 执行侧证据齐备。
- [x] **T-17** /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 2 独立复审）；merge 沉淀 SD-03/04 并销账 KNOWN-DEBT P741-QA-01..07 行待 /auto-plan:merge。

## 9. 复审记录

### r2 合入后的独立复核（2026-10-05，用户再次要求检查）

- stage: review | plan_id: PLAN-741 | plan_revision: 2 | outcome: **needs_fix**
- reviewed_commit: 965b368a20db7c97fab7d1b0d51863b3ccca0f11
- base_commit: 3a7967262；交付52c67d3df祖先关系已验证。
- dependency_revisions/spec_inputs/frozen delta hashes/acceptance_results：
  [r2复核报告](../../reports/741-quality-review-20261005/REVIEW.md)。
- 独立复跑：44/44原型测试、all-targets零warning、fmt、旧reproduce脚本、
  README三命令实机运行（add5、trace12/ba）。
- 原四项P1均确认已修复；新增P741-R2-QA-01块图自环stack overflow/断开环被接受，
  QA-02 reader join超过60s截止、QA-03收据暂存失败exe残留；QA-04记录归档状态不一致。
- acceptance_results: AC-09/10/11/13/15 pass；
  AC-03/14 fail，AC-04/07/08/12 partial；完整旧AC映射见报告。
- findings/evidence：报告相邻日志、fixtures与helper/复现脚本；未修改实施源码/Spec/ledger。
- next: 补修新拒绝/截止/清理边界并独立复审；本轮不激活或实施计划，
  不把旧pass或原型测试绿覆盖新增反例，不改历史状态/勾选。


### Phase 2 独立复审（2026-10-04，/auto-plan:review，独立代理）

- stage: review | plan_id: PLAN-741 | plan_revision: 2 | outcome: **pass**
- reviewed_commit: `1d195fe6e`（worktree clean；实现提交 28c0bbfcd / db75c494f /
  6ad396379；复审后仅追注 comment-only 提交 `fb38632f2` 修 P741P2-R1，无语义变化）
- base_commit: `3a7967262`（Phase 2 合同激活提交）
- 复审方式: 全新上下文独立代理执行（未参与实现、未读执行侧推理），全部结论来自
  其自行运行的命令与源码审查；AC-01 面与 canonical Specs 零 diff 经其独立核实
  （`git diff 3a7967262 -- Cargo.toml Cargo.lock crates/ test/ docs/specs/` 为空）
- dependency_revisions: cranelift 栈 0.126.2（Cargo.lock 已提交）、rustc 1.98.1、
  rust-lld/SDK 本机状态（与 r1 冻结收据一致）
- spec_inputs: SD-03 → docs/specs/auto-hir/project.md（modify，merge 时沉淀）；
  SD-04 → docs/specs/auto-ac/project.md（modify，merge 时沉淀）；复审确认增量
  措辞与实现逐条吻合、无缩水（复验 r1 SD-01/02 原承诺）
- acceptance_results（全 **pass**，均为复审代理自行取证）:
  - AC-09 pass：icmp_bool 唯一出口 uextend 归一（native.rs，LtI32+checked_add×4+
    checked_mul 共 6 调用点，无裸 icmp）；lower_object 越界优雅诊断；native
    bool 正例 exit 3/5；reproduce bool-local/bool-return 由 101/1 → check0/build0
  - AC-10 pass：类型按 params[b.param] vs eval_args[b.arg] 核对 + 映射双射独立校验
    （param 重复/越界、arg 重复消费/未消费/越界各有诊断）；求值顺序保留；
    binding-valid check0+原生 exit 9，binding-invalid check1 type-mismatch
  - AC-11 pass：function→body 反向校验与既有 body→owner 合成唯一归属；
    shared-body-owner / same-sig / stolen-body 全按 verify.owner-mismatch 拒绝；
    reproduce shared-body 由 build 101 panic → check1
  - AC-12 pass：暂存→备份（仅文件型 final）→发布→失败还原备份+清除未还原 placed；
    companion 全名后缀防同 stem 备份碰撞；目录占位触发失败回滚；
    reproduce atomic：exit1+link.receipt、exeChanged=false、objChanged=false、
    旧 exe 原生 exit 5；cli 回滚测试字节级断言+残留扫描
  - AC-13 pass：README 三条命令逐字复制运行（check exit0；add build 产三件套、
    exit5；trace build 产三件套、exit12+stderr b,a）；归档链接与 descriptor
    路径正确；bin 名与实际一致
  - AC-14 pass：rustc/reg/lld/run 全走 run_with_deadline（并发 reader 线程）；
    lib 单测证明 1s 截止杀 30s 挂起 helper、~126KB 输出子进程完整跑通
    （旧实现该场景 60s 死锁）
  - AC-15 pass：tests/common 无未用导入；all-targets 零 warning（复审自跑）；
    verify 脚本含 all-targets 断言步与 --lib 步
- findings:
  - P741P2-R1（info，已处置）：大输出单测注释字节数 128KB→~126KB 修正
    （comment-only `fb38632f2`）
  - P741P2-R2（info，merge 时自愈）：README:8 归档链接在激活态暂态悬空；
    **merge 必须确认 git mv 归档步骤执行**，链接即生效
  - P741P2-R3（info，非缺陷）：reproduce.ps1 的 OS 错误文本 GBK 乱码为该脚本
    .NET 流解码显示件，CLI 本体输出正常
- 反向审查: 新测试均为真断言（dup-arg 刻意同类型参数仅映射校验可拒、same-sig
  共享仅 owner 检查可拒、回滚测试字节级对比+旧 exe 实跑+残留扫描）；
  无缩减 AC、无 workaround、无遗漏子项
- evidence: 44/44 测试（text_binding 11 + hir_verify 9 + native_execution 11 +
  cli 10 + trace_execution 1 + lib 2）、fmt --check clean、all-targets 零 warning、
  一键 verify-ac-741.ps1 -SkipMainGates 13/13 PASS；六反例翻转可由
  docs/reports/741-quality-review-20261004/reproduce.ps1 复现
- next: merge（/auto-plan:merge 沉淀 SD-03/04、销账 KNOWN-DEBT P741-QA-01..07 行、
  合回 v0.6-dev、wt-guard 后清理 plan-741-dev worktree 与组目录）

### Phase 2 工作交接（2026-10-04，/auto-plan:work 执行完毕）

- stage: work
- plan_id: PLAN-741
- plan_revision: 2
- outcome: pass（Phase 2 执行完成；AC-09..AC-15 勾选移交独立复审裁定）
- code_commit: plan-741-dev @ T-16 提交（T-09..T-11 `28c0bbfcd` /
  T-12+T-14 `db75c494f` / T-13+T-15 `6ad396379` / T-16=本提交；基线 3a7967262，
  worktree D:/autostack/.wt/lang-741/auto-lang 分支 plan-741-dev 重建）
- task_ids: T-09..T-16 完成（证据见 §8 Phase 2 执行进度）；T-17=复审本身未执行
- evidence:
  - 原型全族 44 项全绿（text_binding 11 + hir_verify 9 + native_execution 11 +
    cli 10 + trace_execution 1 + lib 2，r1 基线 37 项；新增全为复审反例锁定）
  - 复审 reproduce.ps1 复跑：六反例全部翻转为修复后预期（bool-local build
    101→0、bool-return build 1→0、binding-valid check 1→0、binding-invalid
    check 0→1、shared-body build 101→check 1 owner-mismatch、发布失败
    exeChanged true→false 且旧 exe 原生 exit 5）
  - README 逐字复制运行：check exit 0、add.exe exit 5、trace.exe exit 12+stderr b,a
  - fmt --check 干净；cargo check --all-targets 零 warning（含测试目标）
  - `git diff 3a7967262 -- Cargo.toml Cargo.lock crates/ test/` 为空（AC-01 面不变）
  - 备注：复审报告 readme 探针（`ac-probe check ...` argv 形式）保留 exit 2 为
    历史缺陷记录；README 本体已改用正确调用并实证可运行
- blockers: 无
- next: review（/auto-plan:review 独立核对 AC-09..AC-15 与 SD-03/04；worktree
  保留待复审；merge 沉淀 SD-03/04 并销账 KNOWN-DEBT P741-QA-01..07）

### 起草交接（2026-10-04，非实现独立复审）

- stage: new
- plan: PLAN-741
- plan_revision: 1
- outcome: pass（合同草案可供确认/work接手，不是实现验收通过）
- next: work（用户确认此计划后从T-01开始；本轮不自动执行）
- changed_tasks: T-01..T-08（新建）
- changed_acceptance: AC-01..AC-08（新建）
- spec_delta: SD-01, SD-02
- checked: 范围/路径/Spec来源、编号唯一、AC/SD任务覆盖；未知工具链由有界T-01负责。
- not_checked: 实现parser/verifier/native与Rust门禁；暂无实现，不以文档自检代替运行证据。

### 工作交接（2026-10-04，/auto-plan:work 执行完毕）

- stage: work
- plan_id: PLAN-741
- plan_revision: 1
- outcome: pass（执行完成；日常档继承红与负载 flake 的裁定移交独立复审）
- code_commit: plan-741-dev @ `4dc4da636`（T-01 `0f2b232d7` / T-02 `632e48a03` /
  T-03 `b3bed7d72` / T-04 `cf5aca9f8` / T-05 `523fabfe1` / T-06 `53775205c` /
  T-07 `4dc4da636`；基线 951b6c70f）
- task_ids: T-01..T-07 完成（证据见 §8 执行进度与
  docs/reports/741-ac-hir-native/{toolchain,verification}.md）；T-08=复审本身未执行
- evidence:
  - text_binding 11/11、hir_verify 8/8、native_execution 8/8、cli 9/9、
    trace_execution 1/1（§6 全部命令经 scripts/verify-ac-741.ps1 一键复现，
    原型门禁 11/11 PASS，收据 experimental/ac-core/target/ac-verify-receipts/）
  - 真实原生闭环：add(2,3)=5、count 循环、i32 溢出 trap=70、调用顺序原生
    b→a 且 pair=12（exit 12）、无 capability 拒建、失败构建不覆盖（SENTINEL）
  - 主仓门禁：cargo check -p auto-lang PASS；cargo tv 162/162 PASS；
    cargo t 4945/4971，26 红经同基线对照=14 继承红 + 12 负载 flake
    （对照证据 verification.md §4.1；crates/** 零 diff，根 workspace/lock 零 diff）
- blockers: 无阻塞执行的问题；待复审裁定项——(a) 26 个日常档红的继承认定与
  挂账归类；(b) 组内 auto-down detached 依赖 worktree 的 fold 时机
- next: review（/auto-plan:review 独立核对 AC-01..AC-08 与 SD-01/02 后 merge；
  worktree D:/autostack/.wt/lang-741/auto-lang 保留待复审，fold/清理归 merge）

### 独立复审（2026-10-04，/auto-plan:review）

- stage: review
- plan_id: PLAN-741
- plan_revision: 1（语义合同自起草未变更；进度标记不改变 revision）
- outcome: **pass**
- reviewed_commit: `4dc4da6365868494023dca73d7ded583c90fb033`（worktree clean，
  无未提交实现）
- base_commit: `951b6c70ff596f79e464ba977139669f2d196821`（v0.6-dev 计划起草提交）
- dependency_revisions: auto-down `fba6563ed2148ce85e68208863159b4ccccac710`
  （组内 detached 依赖 worktree，值=auto-down master HEAD）；cranelift 栈
  0.126.2（Cargo.lock 已提交）；rustc 1.98.1 / rust-lld / SDK 10.0.26100.0
  （toolchain.md 冻结收据）
- spec_inputs: SD-01 → `docs/specs/auto-hir/project.md`（add，merge 时沉淀）；
  SD-02 → `docs/specs/auto-ac/project.md`（add，merge 时沉淀）。复审冻结快照=
  计划 §5 规范增量表 + §5.1/5.2/5.3 行为契约（无 canonical 先行版本，无冲突
  对象）；supersedes=[]、new=[auto-hir, auto-ac]、touched_goals=[]（空影响
  说明见 §4.2——关联 v0.6 roadmap 主线，native 目标后续独立登记）。
- acceptance_results（全部 **pass**，均为复审独立复跑取证，非采信执行摘要）:
  - AC-01 pass：`git diff 951b6c70f..HEAD -- Cargo.toml Cargo.lock crates/ test/`
    为空；改动面仅 docs/ experimental/ scripts/；`cargo tree --locked` 无
    auto-val/auto-lang/auto-atom/auto-man 依赖（唯一 grep 命中为根包自身路径）。
  - AC-02 pass：text_binding 11/11 复跑——四份设计文档真实绑定；
    author/explicit/canonical 三形语义相等（手写 PartialEq 排除 span/出现位）；
    enum 简写绑定为 Builtin；15 反例按预期码拒绝且 span 指向冒犯 token。
  - AC-03 pass：hir_verify 8/8 复跑——§9 矩阵 11 例 + 组合反例 10 例按码命中
    （跨 body local、共享表达式/块、表达式环、初始化交集、循环零次保守、
    返回路径等）；CheckedModule 仅 verify() 可构造（测试断言 API 形状），
    拒绝路径无产物。
  - AC-04 pass：descriptor 身份三重匹配（schema/revision/profile mismatch
    夹具拒）；未知 tag/case/字段/分支违约全拒（unknown-* 夹具绿）；单模块
    类型化引用作用域；本仓无源码前端/VM 回退路径（零 crates 依赖）。
  - AC-05 pass：native_execution 8/8 复跑——add(2,3)=5、add(-2,3)=1、
    count(3/0/-2)=3/0/0 均为 Cranelift→COFF→rust-lld→PE 真实退出码；
    cli 套件断言 .obj/.exe/.ac-link.txt 收据与 COFF 符号
    （ac_start/test_entry/add/ExitProcess）。
  - AC-06 pass：i32::MAX+1 与 MAX×2 trap=70（独立 ExitProcess(70) 路径，
    非回绕）；03-call-order 原生执行 stderr 顺序 b→a、退出 12；无 capability
    拒建（cli + trace 双重断言）。
  - AC-07 pass：cli 9/9 复跑——entry.not-found/entry.signature/
    capability.missing/link.failed/link.lld-not-found 全非零且可定位；
    SENTINEL 夹具证明失败构建不覆盖；§6 命令经 verify-ac-741.ps1 一键复现
    （原型门禁 11/11，子进程全带截止时间）。
  - AC-08 pass：主仓门禁 cargo check -p auto-lang PASS、cargo tv 162/162、
    cargo t 4945/4971（26 红裁定见 findings）；SD-01/02 delta 就绪且与实现
    一致；边界宣称受计划 §1/README 约束（未宣称源码编译/生产 ABI/AAC）。
- findings:
  - **P741-R1**（info，已裁定归档）：cargo t 26 红 = 基线继承 + 负载 flake，
    非本计划回归。证据：crates/** 零 diff；同基线 scoped 对照 14 例同红
    （全部落入 KNOWN-DEBT THR-D3 既有清偿族：musk p053/p054、plan707、
    plan502、plan498、plan484、ffi_dual/dep_parity、projector、plan606）+
    12 例 scoped 串行绿（THR-D4 同模式负载 flake）+ 4 例 filter 未命中
    （schema_drift/docs_gen/ash_leak 环境族）。已在
    KNOWN-DEBT-AND-RISKS.md 归档（P741-R1 行），无新增挂账项。
  - **P741-R2**（low，运维债，非验收缺口）：组内 auto-down detached 依赖
    worktree（fba6563e）为 worktree 内构建 auto-lang 所需；merge 时随主
    worktree wt-guard 后一并移除（无 junction/symlink，只读检出）。
  - 遗漏/延期/workaround 扫描：无未授权延期、无 workaround 补丁、无缺失
    子项；ac-core `cargo check` 0 warning、fmt --check 干净、无 debug 残留
    （fixture 生成脚本已清理，probe 产物目录 gitignored）。
- evidence（复审复跑，2026-10-04，均为入库可解析工件或可重现命令）:
  `cargo test --locked` 37/37（text_binding 11 + hir_verify 8 +
  native_execution 8 + cli 9 + trace_execution 1，含真实 PE 链接/运行）；
  `cargo fmt -- --check` 过；零 diff 核对；`cargo check -p auto-lang` PASS；
  `cargo tv` 162/162。work 期收据 docs/reports/741-ac-hir-native/
  {toolchain.md, verification.md}（§4.1 基线对照）随本提交入库。
- next: merge（/auto-plan:merge 沉淀 SD-01/02 与 ledger、合回 v0.6-dev、
  wt-guard 后清理 plan-741-dev 与 P741-R2 依赖 worktree）


### 合并收据（2026-10-05，/auto-plan:merge）PLAN-741:r2

- stage: merge | plan_id: PLAN-741 | plan_revision: 2 | outcome: pass
- prepared: reviewed 基线 8a7a1b793（rebase 后；worktree clean）；canonical
  delta=SD-03/SD-04 → docs/specs/{auto-hir,auto-ac}/project.md（modify，复验
  r1 SD-01/02 承诺）；无依赖 worktree（ac-core 自包含，无需 auto-down 兄弟检出）
- landed: rebase 到 v0.6-dev（range-diff 6 条全 `=`，旧→新映射
  28c0bbfcd→c2dfe84ed / db75c494f→db7fccbd5 / 6ad396379→33e18aa61 /
  1d195fe6e→ec24c0003 / fb38632f2→443386850 / 36c0fd4c7→8a7a1b793）；
  delivery 链 8a7a1b793（reviewed）→ 52c67d3df（SD-03/04 沉淀 + ledger 投影
  P741-4，纯文档/投影 descendants）；主检出 `git merge --ff-only` 无 merge
  commit，tip=52c67d3df；主检出冒烟 ac-core cli 10/10 + native_execution
  11/11 全绿（真实 PE）
- ledger_refreshed: .autoos/specs.json（LF 净增量 +23/−4）designs P741-1/P741-2
  增补 r2 契约事实、reviews 新增 P741-4（r2 复审+修复收据 → 本归档路径）；
  docs/specs/INDEX.md 经 spec-index.py 重建=零 diff（无新模块面）
- archived: docs/plans/archive/741-ac-hir-native-core.md（git mv），status:
  archived，completion_kind: delivered；README:8 归档链接随之生效
  （P741P2-R2 自愈）；KNOWN-DEBT P741-QA-01..07 行已销账（a04b979d7）
- cleaned: **完成（2026-10-05）**——文档引用的 D:/autostack/wt-guard.sh 本机
  不存在（与 r1 同一偏差），按其文档语义以 PowerShell ReparsePoint 递归扫描
  替代（D:/autostack/.wt/lang-741 全树 clean）后：worktree
  D:/autostack/.wt/lang-741/auto-lang 移除、分支 plan-741-dev 删除
  （was 52c67d3df=已落地）、组目录 lang-741 rmdir 成功
- 交付摘要: Phase 2 修复外部复审全部 7 项（P741-QA-01..07），原型测试族
  37→44 项全绿、六反例翻转、一键门禁 13/13；主仓 crates/ 与根 workspace
  全程零改动

### 已交付实现的质量复审（2026-10-04，用户显式要求）

- stage: review | plan_id: PLAN-741 | plan_revision: 1 | outcome: **needs_fix**
- reviewed_commit: c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75
- base_commit: 951b6c70ff596f79e464ba977139669f2d196821；交付304519113已核实在祖先链。
- dependency_revisions/spec_inputs/增量快照指纹与完整AC对账：
  [质量复审报告](../../reports/741-quality-review-20261004/REVIEW.md)。
- 本会话未实施741；自行读代码、在detached复审检出重跑37项原型测试/fmt，
  原套件全过，额外反例重现bool宽度、bindings类型映射、body双向owner、
  发布阶段失败覆盖四项核心缺陷（P741-QA-01..04）。
- acceptance_results: AC-01/02/06=pass；AC-03/07=fail；AC-04/05/08=partial。
  另记录README不可复现/发现子进程缺截止/测试warning（QA-05..07）。
- findings/evidence: 报告及相邻fixtures/results/复现脚本；KNOWN-DEBT追加对应未解决项。
- next: 为具体修复起草独立合同，修复后重跑并复审；本轮未改实现。
- 已归档Plan保持archived，不重开/回滚旧记录、不改revision/历史任务勾选；
  此为新基线的交付后质量结论，不沿用旧pass覆盖新增反例。

### 合并收据（2026-10-04，/auto-plan:merge）PLAN-741:r1

- stage: merge | plan_id: PLAN-741 | plan_revision: 1 | outcome: pass
- prepared: reviewed 基线 4dc4da636（clean）；canonical delta=SD-01/SD-02 →
  docs/specs/{auto-hir,auto-ac}/project.md；依赖 auto-down fba6563e（detached）
- landed: rebase 到 v0.6-dev（range-diff 全 `=`，旧→新映射
  0f2b232d7→bc9d2a05f / 632e48a03→987e614d4 / b3bed7d72→f5fafd829 /
  cf5aca9f8→22b5e1376 / 523fabfe1→65a32fc08 / 53775205c→8921edf75 /
  4dc4da636→cbce56d22）；交付链 1e4a2533a（spec 沉淀）→ cf369e0d5（lockfile
  补交：根 .gitignore 全局 `Cargo.lock` 曾静默排除两新 workspace 的 lock，
  force-add 修复 --locked 可复现性）→ 304519113（ledger 投影）；全部
  --ff-only 无 merge commit；主检出冒烟 ac-core 全族绿（含真实 PE 运行）
- ledger_refreshed: .autoos/specs.json（git 跟踪，经 worktree 提交）
  designs P741-1/P741-2 → docs/specs/{auto-hir,auto-ac}/project.md，
  reviews P741-3 → 本归档路径；canonical 面 docs/specs/INDEX.md +2 行
  （spec-index.py 生成）+ overview.md 外围与实验 +2 行
- archived: docs/plans/archive/741-ac-hir-native-core.md, status: archived,
  completion_kind: delivered
- cleaned: **完成（2026-10-04）**——两 worktree tracked 零脏、分支 plan-741-dev
  （已并 @ 304519113）删除；reparse-point 扫描双 worktree 均 clean 后移除
  （D:/autostack/.wt/lang-741/{auto-lang,auto-down}）；组目录 rmdir 成功。
  偏差记录：文档引用的 D:/autostack/wt-guard.sh 在本机不存在，按其文档语义
  （reparse point 扫描，非空即拒）以 PowerShell 等价扫描替代，结果归档于本行。
  注：auto-down worktree 注册在 auto-down 仓（创建即如此），以
  `git -C auto-down worktree remove` 移除。

## 10. 待澄清事项

1. 工具链/后端：T-01实测锁版本、SDK/linker；必须能力缺失记录blocker与修订候选。
   常规路径发现无需反复确认，但不能扩大为LLVM全栈重写或安装无关软件。
2. 编号一致性：741按用户指定分配，主机器740尚不可见；恢复后核对，
   若另一侧也使用741，保留证据并协调改号，不覆盖文件；执行前发现碰撞立即报告。
3. 源语言类型/AAC：旧int/i32、完整所有权、AAC主体清单由后续A1/B/C承接；
   当前精确i32不能替代Auto全语言兼容结论。
4. 生产模块ABI：由关联Native ABI设计线细化；这里的最小启动/trace接口不冻结
   生产符号、布局、ABI版本或模块生命周期。

无必须先由用户裁定的技术问题阻止起草完成。
实施授权限于后续用户确认，不由outcome:pass推定。

### Phase 2 授权与边界（2026-10-04）

- 用户裁定：P741-QA 修复**不另开新计划**，激活本文件追加 Phase 2（plan_revision 2）；
  此裁定优先于质量复审报告"另开独立合同/不重开归档"的建议。
- 授权范围：experimental/ac-core 修复 + scripts/verify-ac-741.ps1 + README；
  canonical Spec 增量（SD-03/04 modify）在复审 pass 后 merge 沉淀。
  crates/** 与根 workspace 不触碰；不跑 tf/ta/t3 批量档。
- 验收基线：复审报告的 5 个缺陷 fixture + extra-results（bool-local build 101、
  bool-return build 1、binding-valid check 1、binding-invalid check 0、
  shared-body build 101、发布失败 exe 哈希变化）全部翻转为 AC-09..12 所述预期。
