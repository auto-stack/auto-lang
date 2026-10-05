---
plan_id: PLAN-741
status: archived
feature_name: AC 首个闭环：独立 Atom HIR、语义校验与 Windows 原生 AOT
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-05
plan_revision: 3
current_step: 23
total_steps: 24
supersedes_spec_components:
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-ac/project.md
new_spec_components: []
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


### Phase 3 再激活（2026-10-05，plan_revision 3）

用户明确要求重新激活 PLAN-741，将合入后复审的新发现及修复方案追加为新 Phase。
本合同从 archive/ 移回 docs/plans/，status=executing；T-01..T-17、r1/r2 的交付、
复审与 merge 收据保留为历史。当前待实施 T-18..T-24；本轮只更新合同与状态，不实施修复。
修复 P741-R2-QA-01..04，不扩展语言子集、不更换后端，不宣称旧 pass 覆盖 r3。

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


### 4.4 Phase 3 授权、证据与当前基线

- 用户原话（2026-10-05）：“请重新激活计划741，把刚才发现的问题及修复方案记录进去（可以作为新的phase）”。
  此显式授权优先于归档终态/不重开建议；复用同一 ID，不消耗 .next-id，不另建计划。
  当前授权是再激活与修订合同；本轮不调用 work 实施。已有修复范围保持在本仓原型及验证资料。
- 激活前基线：v0.6-dev @ b85e2bb76b748f6a3f104096542837b44bdefbfc；r3 实施起点为包含本合同的激活提交，
  T-18 记录完整 hash。复审实测基线仍为 965b368a20db7c97fab7d1b0d51863b3ccca0f11，二者差异仅复审资料。
- 证据：[r2 合入后复审](../reports/741-quality-review-20261005/REVIEW.md)，包含源文件/Spec SHA-256、
  原始日志、两个块图输入、review-helper.rs 与可运行复现脚本。
  原四项 P1 已修，44 项原型测试通过；新自环导致 0xC00000FD，断开循环 check=0，
  退出后持管道反例在 114.963s 仍未返回，收据暂存失败遗留 staged exe。
- canonical 输入：docs/specs/overview.md、auto-hir/project.md（SHA-256
  ae1a898fe12d5cd9f1d883fb68564049355556b488bc995b894f0976babe9660）、auto-ac/project.md（SHA-256
  9679401e686e5de6ed11bc678ea0866d630f26fdfc7890249332a5a44e997f03）。
  全局 overview 为历史总览；r3 具体接口以这两个模块 Spec 与当前原型代码为准。
- 代码锚点：src/verify.rs::check_blocks/walk_block，src/link.rs::run_with_deadline/publish_artifacts，
  src/main.rs 的发布失败清理；路径均相对于 experimental/ac-core。
  src/verify.rs SHA-256=2091454643f54446ca04ff7f497a682a74cef27d98d519ea38470e7769ed5260；
  src/link.rs SHA-256=6eb8c5ab0d3f3bfa41f9e252073db7fd7b9080ab3fae35c1633258ec848a7566。
- 本轮仅计划/状态/引用簿记；实施允许 experimental/ac-core、scripts/verify-ac-741.ps1 与新增验证资料，
  仅原型自己的 Cargo.toml/lock 可因必要的进程管理依赖调整。根 workspace/lock、crates/**、
  v0.5 parser/VM/A2X 与其它仓不动；canonical 行为 Spec 在独立复审 pass 后 merge 沉淀 SD-05/06。
- r1/r2 pass 仅为对应 revision 与 commit 的历史证据；受影响 AC-03/07/08/12/14 需在 r3 重新验证，
  其余旧 AC 亦须回归确认。旧任务勾选不回滚，新任务/新 AC 不预先勾选。

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


### 5.6 Phase 3 设计（plan_revision 3）：P741-R2-QA-01..04

**QA-01 / P1：先验证完整块包含图，再做数据流。**
由 If 的 then/else 与 Loop 的 body 引用构造块包含边，检查 ID 边界、入口零入边、
其余块唯一入边、从入口到所有块的可达性，以及全部块的无环性。
使用三色 DFS（优先显式栈）或等价算法；不能以“入度=1”替代可达性，也不能只遍历入口漏掉断开 SCC。
发现结构错误即返回 verify.block-structure 等定位诊断，不再进入 walk_block；
后续递归走查加循环防护，不能在已记录结构错误后仍递归到栈溢出。
这里禁止的是块包含环；合法运行时 loop 重入、break/continue 的 LoopId 目标、函数递归调用
不是块包含边，必须继续被接受。此修改不冻结全语言深递归资源策略。

将报告两份 Atom 输入物化为新 fixtures/invalid 块图反例，补入口自环、多块环、
断开循环 SCC、共享块及合法嵌套 if/loop；真实 bind→verify 与 CLI check/build 均覆盖。
CLI 负例用外层截止保护：不得正常接受，不得 panic/异常崩溃，不产出 native 制品。

**QA-02 / P2：一个单调 deadline 覆盖运行与输出收集。**
run_with_deadline 中父进程退出不等于 stdout/stderr 到 EOF；
退出后的 reader join、错误分支的 wait/回收同样不得无限等。保持并发排空 stdout/stderr，
在 deadline 前同时等待退出状态与两个输出完成；任何阶段超时均报 link.deadline，不能返回截断的成功输出。
选择可截止的输出协调与受控进程树机制；Windows 优先 Job Object（关闭即终止自有子树）
或经反例证明等价的方案，必要的原型局部依赖由实施记录。
提前退出/超时/IO 错误必须关闭句柄并回收本次自有进程、reader；不能只 detach 永久阻塞线程来满足“返回快”。
清理仅针对本次拥有的进程/句柄，禁止按进程名全局杀 rustc/lld 等工具。

新增可控 helper：父进程 spawn 继承输出管道的后代后立即退出，后代保持管道至明显超过测试 deadline。
用参数化 1s deadline（外层 5s watchdog）证明退出后的收集阶段也按时拒绝；
测试必须 finally/RAII 清理自身 helper/后代，不能真实等待生产 60s 或留下后台进程。
保留挂起父进程与 >64KB 双管道输出正例，并增加父进程/后代及时结束的成功正例。

**QA-03 / P2：统一拥有暂存制品并覆盖每个失败出口。**
由明确的 stage guard/事务所有权管理 obj/exe/receipt 暂存及备份；main.rs 与 publish_artifacts
约定由谁清理，避免仅删除 tmp_obj。覆盖收据暂存准备、备份、exe/obj/receipt 发布失败及成功结束。
任何正常可回收的失败出口均清除本次拥有的暂存文件、恢复旧制品；exe/obj/receipt 字节不变，旧 exe 仍 exit 5。
已存在的占位目录或其它非本次创建资源不能被递归删除。
若真实 IO 错误阻止回滚/清理，必须返回可定位的恢复诊断，保留唯一可恢复备份并列出路径，
不能吞掉 rename/remove 错误后宣称回滚完整，也不能为“无残留”删除唯一旧制品。
这不是放宽受控失败的无残留验收；失败注入的正常可恢复场景仍要求零自有 .tmp/.bak 残留。

测试层以真实 PE/COFF 基线控制阶段故障，不给产品 CLI 新增故障注入参数：
收据暂存路径目录占位、备份目标阻塞、分别在三次发布时失败，以及无既有制品与成功发布。
区分用户占位目录与本次生成文件；验证哈希、收据内容、原生 exit 与残留清单。
直接事务单测和真实 CLI 的既有收据最终路径失败用例都必须保留。

**QA-04 / P3：状态、路径与收据一致。**
本轮已获用户明确再激活授权；移动唯一 741 文件到 active 路径并设 executing/r3，
同步当前 README/模块计划索引/ledger 文件引用，不改历史 pass 的时间或含义。
前两轮 “archived” 收据描述当时交付，本次追加再激活收据解释实际 frontmatter 曾 reviewed 的偏差。
r3 结束必须独立 review→merge→archive，最终 frontmatter=archived、位置=archive/、链接/收据一致；
不得把归档目录本身当作状态变更。AC-13 的历史 archive 路径要求在 r3 按 active→archive 生命周期更新，
README 命令可执行等其余要求保持不变。

### 规范增量（SD-01..04 为历史交付；Phase 3 追加 SD-05/06）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-hir/project.md | 无实现规范→有限文本/TypeUse/校验/顺序契约及限制 | 沉淀实证能力，不宣称完整HIR | AC-01,AC-02,AC-03,AC-04 |
| SD-02 | add | docs/specs/auto-ac/project.md | 无native实现规范→Checked HIR AOT/Windows工具链/入口/trap/测试能力 | 区分原型、生产ABI与源码编译 | AC-05,AC-06,AC-07,AC-08 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 语义契约两点明确化：call 实参类型按 bindings 映射逐位核对（arg 恰被一处 binding 消费）；函数↔body 双向归属唯一（共享/冒领 body 拒绝） | QA-02/03 复验 SD-01 原承诺，实现修正而非缩规约 | AC-10,AC-11 |
| SD-04 | modify | docs/specs/auto-ac/project.md | bin 名澄清为 auto-ac-prototype（ac-probe 为历史文档名）；制品发布措辞=暂存+备份+发布+失败回滚；硬截止时间覆盖全部子进程（工具发现/链接/运行） | QA-01/04/05/06 复验 SD-02 原承诺 | AC-09,AC-12,AC-13,AC-14 |
| SD-05 | modify | docs/specs/auto-hir/project.md | 入边/入口角色契约→显式区分块包含边与运行时循环，全部块可达且包含图无环，结构失败阻止数据流/Checked 构造 | QA-01 兑现既有拒绝承诺，避免小环崩溃及断开 SCC 漏检 | AC-03,AC-04,AC-16 |
| SD-06 | modify | docs/specs/auto-ac/project.md | 全部子进程硬截止→进程退出及输出收集共用 deadline、自有资源受控回收；失败回滚→准备/备份/发布的暂存所有权与清理、恢复失败显式诊断 | QA-02/03 补全执行器与事务全路径，不以缩规约规避失败 | AC-07,AC-12,AC-14,AC-17,AC-18 |

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

CLI例：cargo run --manifest-path experimental/ac-core/Cargo.toml --locked --bin auto-ac-prototype -- check docs/design/strategy/hir-examples/01-add.atom。
native脚本发现工具链、构建/运行、处理超时并保存收据；SDK缺失为 blocker，
不能 ignore native测试再宣称通过；其他平台的reader/verifier不能替代Windows原生门禁。

最终按 AGENTS 编译器改动档跑一次 cargo check -p auto-lang、cargo t、cargo tv；
开发迭代仅跑原型定向测试。旧测试红需同基线对照、证据与复审裁定。
不触 aavm/UI/trans/book 时不跑 taa/tu/tt/tb；本worktree不跑 tf/ta/t3。
本轮起草为doc-only，不运行上述实现测试或docs_gen。


### Phase 3 验证补充（AC-16..20）

| 测试族/证据 | 操作 | 预期 |
|---|---|---|
| hir_verify + CLI | 自环、多块环、断开 SCC、共享块、合法嵌套/循环/表重排 | 非法 check/build 正常 exit 1 + verify.block-structure；合法原型行为不变；build 不触 linker/无产物 |
| link 内嵌单测 / 新增 tests/process_deadline.rs（按实施选用） | 1s deadline：父先退、后代持 stdout/stderr；双管道大输出/及时关闭 | 持管道约 1s 报 link.deadline（3s 容差上界），外层 5s 保护；无自有进程/reader 遗留；完整输出正例成功 |
| link 事务单测 + cli | 准备、备份、各发布出口受控失败；原生旧 exe=5；占位目录 | 哈希/收据不变，旧 exe=5；零自有 .tmp/.bak，用户目录保留；成功产三件套 |
| 当前引用/计划状态 | active 唯一文件、revision/step、README/模块索引/ledger；最终归档 | 当前 executing/r3/active，交付后 archived/archive，一致；历史证据原样保留 |

在 plan worktree 执行：

    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test hir_verify
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --lib -- --test-threads=1
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test cli -- --test-threads=1
    cargo test --manifest-path experimental/ac-core/Cargo.toml --locked -- --test-threads=1
    cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets
    cargo fmt --manifest-path experimental/ac-core/Cargo.toml -- --check
    powershell -NoProfile -File scripts/verify-ac-741.ps1

脚本完整门禁包含主仓 check/t/tv，不能只跑 -SkipMainGates 声称通过全门禁。
若另建 process_deadline.rs，接入上述 cargo test 全量原型步骤和验证脚本，不允许靠 ignore 通过。
复审报告 reproduce.ps1 是观察工具，不以脚本自身 exit 0 判定修复：两份非法输入须正常 exit 1，
watchdog 必须收到 deadline 拒绝且仍运行=false，receipt-prepare 的 staged_exe_left=false。
快速 -SkipWatchdog 不能证明 AC-17；参数化 1s 永久回归与真实 run_exe 截止各留证据。
44 项原型测试是 r2 基线，新增数量按实际记录；全仓历史红需同基线对照，零新增未解释红。
本轮合同/簿记为 doc-only，不运行 cargo 测试或 docs_gen；实施/复审按 §6/AGENTS 分级门禁。

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


Phase 3 追加（plan_revision 3，2026-10-05；原 AC-01..15 不移除）：

- [ ] **AC-16**（R2-QA-01）：全部块可达、包含图无环、入口角色唯一。报告两份输入及新增自环/多块环/断开 SCC 经真实绑定后在 verify 拒绝；CLI check/build 正常 exit 1、有错误阶段/code/span，外层 5s 内返回，不栈溢出/panic，不进入后端或创建制品；合法嵌套 if/loop、break/continue、循环计数及递归调用边界不被误判。
- [ ] **AC-17**（R2-QA-02）：参数化 1s 截止覆盖父进程先退出、后代继承管道的收集阶段，3s 内返回 link.deadline、外层 5s 不触发；本次自有 helper/后代和 reader 已回收；及时关闭管道、大输出及普通挂起回归通过。实际 run_exe 固定 60s 路径亦有受外层保护的截止证据，不再复现 115s 卡住，不通过截断成功输出/泄漏线程规避。
- [ ] **AC-18**（R2-QA-03）：收据准备、备份和三次发布的受控失败均保留旧 exe/obj/receipt 字节，旧 exe exit 5，零本次自有 .tmp/.bak 残留；用户占位目录不删；无旧制品时失败不留半成品，成功完整产三件套且无残留。恢复本身受阻时返回恢复诊断与备份路径、保留可恢复旧制品，禁止吞错或错误销账。
- [ ] **AC-19**（R2-QA-04）：当前唯一 741 位于 docs/plans/，executing/r3，current_step/total_steps 与任务进度一致；README/模块计划索引/ledger 引用实际文件，r1/r2 pass 标明历史效力，r3 需新复审。最终 merge/归档实际写 archived，移回 archive/ 并同步指针，收据与 frontmatter 一致；不得重复分配 ID 或改变历史结论。
- [ ] **AC-20**：原 44 项与新增测试全绿、fmt、all-targets 零 warning、一键门禁及主仓 check/t/tv 有 revision 3 的新证据；生产面相对 r3 起点无改动，旧报告与实现保留。独立复审逐项覆盖 AC-01..20/SD-05..06，发现闭环后才销账和 merge；未知结果不复用历史 pass 代替。

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


### Phase 3 执行计划（plan_revision 3，T-18..T-24 尚未执行）

优先级：先 T-19（P1），再 T-20/21（P2）；簿记 T-22 和全门禁在独立复审前收口。
本合同提交后由 /auto-plan:work 创建或确认 D:/autostack/.wt/lang-741/auto-lang、plan-741-dev，
从含 r3 合同的 v0.6-dev 起步，不复用旧 r2 tip。本轮未创建实施 worktree、未执行代码修复。

| task | 依赖 | 文件/符号与动作 | 验证命令/预期 | AC |
|---|---|---|---|---|
| T-18 | r3 合同已提交 | 核对活动 741 唯一性、HEAD/原型/报告指纹及其它在途计划；建立专用 worktree/分支，记录完整基线。不存在则新建，不覆盖其它 checkout，无链接 | git worktree list、git status、Get-FileHash；旧实现已合入、工作树 clean、工具链可用 | AC-01,19,20 |
| T-19 | T-18 | src/verify.rs::check_blocks/walk_block：全图无环/入口可达结构门及走查防护；新增 fixtures/invalid 块图反例与 tests/hir_verify.rs、tests/cli.rs 回归 | hir_verify + cli，外层 5s；两报告负例及新组合正常拒绝，合法循环/嵌套通过 | AC-03,04,16；SD-05 |
| T-20 | T-19 | src/link.rs::run_with_deadline：截止覆盖退出后收集/错误回收；受控进程树/reader 生命周期。新增测试 helper/可控 deadline 用例（tests/process_deadline.rs 为候选新路径）；必要局部依赖记录理由 | --lib/新增专项；1s/3s/5s 三层判据、无自有进程/reader 遗留，大输出完整；生产 60s 路径留证 | AC-07,14,17；SD-06 |
| T-21 | T-20 | src/link.rs::publish_artifacts、src/main.rs：stage 所有权和统一清理/恢复诊断；事务单测与 tests/cli.rs 准备/备份/发布受控失败矩阵 | --lib + cli；旧三件套哈希/旧 exe=5、零自有残留、目录保留；成功和无旧制品回归 | AC-07,12,18；SD-06 |
| T-22 | T-18,T-21 | 核对 active metadata 与当前指针；README 命令/文档链接、一键验证脚本纳入新增测试；记录再激活/归档状态偏差解决方式，最终归档留给 merge | markdown 路径/唯一 ID/frontmatter 检查、PS 语法解析；本阶段 active=executing/r3，历史保留 | AC-13,15,19 |
| T-23 | T-19..T-22 | 新增 docs/reports/741-phase3-boundary-fixes/（验证/指纹/故障矩阵资料，新路径），跑 §6 全门禁，对账所有 AC 与旧/新反例；进度只回写主检出活动计划 | 原型全量+all-targets+fmt、一键脚本与主仓 check/t/tv；根 Cargo.toml/Cargo.lock/crates/test 相对 r3 基线零 diff；所有红逐项归因 | AC-01..20 |
| T-24 | T-23 | /auto-plan:review 独立核实 r3/HEAD、AC/SD 和源码，扫描遗漏/规避/新债；pass 后交 /auto-plan:merge 沉淀 SD-05/06、销账 QA-01..04、正确归档并 guard 清理 | revision-bound 独立证据，pass 才 merge；归档前后状态/指针/收据一致，guard clean 才移除 | AC-08,19,20；SD-05/06 |

原步骤 T-01..T-17 为已交付历史；上文“所有步骤尚未执行”仅是 r1 起草时的记录。
r3 current_step=17、total_steps=24；T-18..24 完成后逐步回写，不预勾成功。

Phase 3 执行进度（commit 哈希=plan-741-dev，基线 ed2d00b90，2026-10-05）：

- [x] **T-18** worktree `D:/autostack/.wt/lang-741/auto-lang`（分支 plan-741-dev 重建 @ ed2d00b90，
  activity 741 唯一性/git status clean/指纹已核）；跨仓依赖按组内规则建只读兄弟检出
  `D:/autostack/.wt/lang-741/auto-down`（detached @ fba6563e，同 r1，无链接）。
- [x] **T-19** commit `585224bf1`：verify.rs check_block_graph（显式栈 DFS 可达性+入边不变量+结构门
  阻止走查）+ walk_block 防御 visited；复审两输入物化为 invalid fixtures；hir_verify 拒绝矩阵 +2、
  cli 块图反例测试（exit 1、定位 span、0.14s<5s、无制品）。AC-16。
- [x] **T-20** commit `7e3a5713f`：run_with_deadline 单调截止覆盖等待+收集（通道 recv_timeout）；
  windows-sys 0.59 Job Object（KILL_ON_JOB_CLOSE）管控子树，全出口 reader join 回收；
  DeadlineCollect 变体；reader 改 read_to_end+lossy（修 read_to_string 截断问题）。
  新测试：1s 收集截止（后代被回收 tasklist 证实）、后代即时退出正例、60s 生产证据
  （`--ignored` 显式跑 60.01s vs 复审实测 114.96s 挂起）。AC-17。
- [x] **T-21** commit `7e3a5713f`：publish_artifacts discard_staged 全失败出口回收
  （收据暂存/备份/三次发布）+ link.restore 恢复诊断（列幸存备份路径）+ 用户占位目录保护 +
  link_object_staged 执行器错误回收；单测失败矩阵 5 例全过。AC-18。
- [x] **T-22** commit `bc9c6cc5c`：README 流水线/测试族表 + verify 脚本 --lib 串行化；
  plans.md/ledger 指针复审方已同步 active 路径（本轮核对一致）。AC-19（执行侧部分；
  最终归档留 merge）。
- [x] **T-23** commit `bc9c6cc5c`：docs/reports/741-phase3-boundary-fixes/
  {verification.md, r2-review-reproduce-rerun.json, reds-*.txt, source-hashes.txt}。
  门禁：原型 52 项绿+1 ignored 证据项、fmt 干净、all-targets 零 warning、一键脚本原型段
  13/13、主仓 check PASS（组内 auto-down 建立后）、cargo tv 162/162、cargo t 双侧
  no-fail-fast 同基线归因（基线 33 红/worktree 35 红，对称差 8 例全为 THR-D3/D4
  已知族在并行负载下的翻转，30 例共有，零新增归因）；生产面零 diff
  （git diff ed2d00b90 -- Cargo.toml Cargo.lock crates/ test/ 为空）；
  r3 复审 reproduce 复跑全验收点（staged_exe_left=false、watchdog 60.07s
  stillRunning=false）、r2 reproduce 无回归。AC-20。
- [x] **T-24** /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 3
  独立复审）；R1 备份路径恢复诊断已按复审建议顺手统一（comment 后 scoped 复验
  lib 9/9 + fmt + all-targets 零 warning）；merge 沉淀 SD-05/06、销账 R2-QA 行、
  归档并 guard 清理待 /auto-plan:merge。
T-24 实施侧未核实的独立复审不得提前标记；最终归档部分按 merge 收据验收。

## 9. 复审记录

### Phase 3 再激活 / 合同修订交接（2026-10-05）

- stage: new | plan_id: PLAN-741 | plan_revision: 3 | outcome: **pass（合同就绪，非实现/复审 pass）**
- authorization: 用户明确再激活同一 741、追加问题与修复方案；本轮只修订合同与簿记，next=work。
- reactivation: archive/741-ac-hir-native-core.md → docs/plans/741-ac-hir-native-core.md；
  status=executing，current_step=17，total_steps=24；.next-id 保持 744。
- input: b85e2bb76（r2 复审证据提交）；r2 reviewed_commit=965b368a2；
  findings=P741-R2-QA-01..04。旧 pass 保留为 r1/r2 历史，受影响验收在 r3 重新验证。
- changed_contract: §4.4/5.6/6/7/8，T-18..24、AC-16..20、SD-05/06；
  supersedes_spec_components=auto-hir/project.md、auto-ac/project.md（modify），不修改 canonical 行为内容。
- metadata_note: r2 归档收据称 archived 但实际 frontmatter=reviewed 的偏差不删历史掩盖；
  本次以用户授权的 active/executing 合同恢复一致，r3 merge 必须真实写 archived。
- maintenance: 随文件移动修正两个报告相对链接；历史 CLI 示例的 bin 名纠正为实际 auto-ac-prototype。
- checks: 唯一计划 ID、任务→AC→Spec 覆盖、现有/新建路径、引用与计数；doc-only 无 cargo 测试。
- next: /auto-plan:work 执行 T-18 起；本轮不创建实施分支/worktree、不关闭尚未修复的债务。


### r2 合入后的独立复核（2026-10-05，用户再次要求检查）

- stage: review | plan_id: PLAN-741 | plan_revision: 2 | outcome: **needs_fix**
- reviewed_commit: 965b368a20db7c97fab7d1b0d51863b3ccca0f11
- base_commit: 3a7967262；交付52c67d3df祖先关系已验证。
- dependency_revisions/spec_inputs/frozen delta hashes/acceptance_results：
  [r2复核报告](../reports/741-quality-review-20261005/REVIEW.md)。
- 独立复跑：44/44原型测试、all-targets零warning、fmt、旧reproduce脚本、
  README三命令实机运行（add5、trace12/ba）。
- 原四项P1均确认已修复；新增P741-R2-QA-01块图自环stack overflow/断开环被接受，
  QA-02 reader join超过60s截止、QA-03收据暂存失败exe残留；QA-04记录归档状态不一致。
- acceptance_results: AC-09/10/11/13/15 pass；
  AC-03/14 fail，AC-04/07/08/12 partial；完整旧AC映射见报告。
- findings/evidence：报告相邻日志、fixtures与helper/复现脚本；未修改实施源码/Spec/ledger。
- next: 补修新拒绝/截止/清理边界并独立复审；本轮不激活或实施计划，
  不把旧pass或原型测试绿覆盖新增反例，不改历史状态/勾选。


### Phase 3 独立复审（2026-10-05，/auto-plan:review，独立代理）

- stage: review | plan_id: PLAN-741 | plan_revision: 3 | outcome: **pass**
- reviewed_commit: `f60fa9a93`（worktree clean；实现 585224bf1/7e3a5713f/bc9c6cc5c + 簿记 f60fa9a93，区间恰 4 提交）；复审后仅追加 R1 顺手统一提交（备份失败路径 link.restore 对称化，scoped 复验 lib 9/9 + fmt + all-targets 零 warning）
- base_commit: `ed2d00b90`（v0.6-dev 含本轮复审方 r3 再激活 fa3abe476）
- 复审方式: 全新上下文独立代理（未参与实现），全部证据自行运行/读源码；含自构反例（3 块断开环、else 回边入口、合法嵌套 loop+if+break 不误拒）与 reproduce.ps1 实跑（watchdog observed 60.05s、stillRunningAfterDeadline=false；receipt-prepare staged_exe_left=false）
- dependency_revisions: windows-sys 0.59.0（本轮新增，JobObjects/Foundation 面，合同允许的局部依赖）；其余同 r2（cranelift 0.126.2、rustc 1.98.1）
- spec_inputs: SD-05 → docs/specs/auto-hir/project.md（modify，merge 沉淀）；SD-06 → docs/specs/auto-ac/project.md（modify，merge 沉淀）；复审确认与实现逐条吻合无缩水；canonical specs 零 diff（git diff ed2d00b90 -- docs/specs/ 空）
- acceptance_results（全 **pass**，复审代理自行取证）:
  - AC-16 pass：块包含图结构门 + 自构反例 + 复审 fixtures 经 hir_verify/cli 拒绝
  - AC-17 pass：单 deadline 覆盖收集 + Job Object 全出口回收 + watchdog 60.05s
  - AC-18 pass：discard_staged 全失败出口 + link.restore + 占位目录保护 + 矩阵 5/5
  - AC-19 pass（带簿记注记）：active 唯一、executing/r3、指针一致、归档留 merge
  - AC-20 pass：43+9 全绿、fmt、all-targets 零 warning、一键原型段 13/13、tv 162/162、cargo t 双侧清单对称差 8 例全 THR-D3/D4 族 + 抽查 3 例主检出单跑 PASS（并行负载 flake 证实）、生产面零 diff
- findings（3 条 P3，不阻塞）:
  - P741P3-R1：备份失败路径恢复报告与发布路径不对称——已按建议顺手统一为 link.restore（提交随本记录，scoped 复验通过）
  - P741P3-R2（informational）：降级路径（job 不可用）失败出口不 join reader，代码注释已明示、deadline 仍约束——留档
  - P741P3-R3（簿记）：①KNOWN-DEBT 相对链接 archive/ 在激活期暂态失效，merge 归档后自愈（同 r2 README 链接模式）；②frontmatter current_step 漏步进——本轮已修（17→23）
- next: merge（/auto-plan:merge 沉淀 SD-05/06、销账 R2-QA-01..04 行、归档至 docs/plans/archive/ 并 guard 清理 worktree 与组内 auto-down 兄弟检出）

### 合并收据（2026-10-05，/auto-plan:merge）PLAN-741:r3

- stage: merge | plan_id: PLAN-741 | plan_revision: 3 | outcome: pass
- prepared: reviewed 基线 383f90090（rebase 后；worktree clean）；canonical delta=SD-05/SD-06 → docs/specs/{auto-hir,auto-ac}/project.md（modify）；组内只读依赖 D:/autostack/.wt/lang-741/auto-down（detached @ fba6563e，同 r1）
- landed: rebase 到 v0.6-dev（range-diff 6 条全 `=`，旧→新映射 585224bf1→dd89a0156 / 7e3a5713f→4857ddb83 / bc9c6cc5c→510979929 / f60fa9a93→9a2e356fd / c6ceb613f→fe2334f55 / c0a367496→383f90090）；delivery 链 383f90090（reviewed）→ a0b3750c4（SD-05/06 沉淀 + ledger 投影 P741-5，纯文档/投影 descendants）；主检出 `git merge --ff-only` 无 merge commit，tip=a0b3750c4；主检出冒烟 ac-core cli 11/11 + lib 9/9 全绿
- ledger_refreshed: .autoos/specs.json（LF 净增量 +23/−4）designs P741-1/P741-2 增补 r3 契约事实、reviews 新增 P741-5（r3 复审+边界修复收据）、P741-3/P741-4 指针随归档迁至 docs/plans/archive/；docs/specs/INDEX.md 经 spec-index.py 重建后归一 LF=零 diff；docs/specs/{auto-hir,auto-ac}/plans.md 行更新为 delivered + archive 链接
- archived: docs/plans/archive/741-ac-hir-native-core.md（git mv），status: archived，completion_kind: delivered；README:8 计划链接随归档指回 archive/（r3 生命周期闭环，P741P2-R2 同模式）；KNOWN-DEBT P741-R2-QA-01..04 行销账（97c7e8d0c）
- cleaned: **完成（2026-10-05）**——文档引用的 D:/autostack/wt-guard.sh 本机不存在（r1/r2 同一偏差记录），按其文档语义以 PowerShell ReparsePoint 递归扫描替代（组目录 lang-741 全树 clean）后：worktree D:/autostack/.wt/lang-741/auto-lang 移除、auto-down 兄弟检出经 auto-down 仓 `git worktree remove` 移除、分支 plan-741-dev 删除（was a0b3750c4=已落地）、组目录 lang-741 rmdir 成功（T-23 对照日志临时件清理，正式副本已入库 docs/reports/741-phase3-boundary-fixes/）
- 交付摘要: r3 修复 r2 合入后复审全部缺口（R2-QA-01..04）：块包含图结构门、执行器截止覆盖输出收集（Job Object 管控）、发布事务全出口回收+恢复诊断、状态/指针簿记；原型测试 44→52 项 + 60s 生产截止证据测试；独立代理复审 pass（AC-16..20）；windows-sys 0.59 为合同允许的局部依赖（JobObjects 面）
- 备注: 三条 P3 注记随复审记录留档（R2 降级模式 informational、R3-① KNOWN-DEBT 相对链接已随归档自愈、R3-② current_step 步进已修）；本计划三次交付（r1 闭环、r2 质量修复、r3 边界修复）全部完成

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
  [质量复审报告](../reports/741-quality-review-20261004/REVIEW.md)。
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

### Phase 3 边界与交接

无阻止合同交接的用户决策。T-20 负责验证 Windows 受控进程树/可截止 reader 的实现方案，
T-21 负责明确 stage 清理所有权；两者在既有目标内选用等价实现并记录证据，不能默默延后。
若本机 SDK/链接工具缺失，执行者记录 blocker 并保留证据，不能 ignore 原生门禁。
wt-guard.sh 本机缺失已有 r1/r2 收据；清理时先检查并运行规定 guard，若仍缺失，
按已有记录的等价 ReparsePoint 扫描验证精确组路径，非 clean 不删除，不创建任何 junction/symlink。
原历史“是否执行 741”疑问现仅为起草历史；本轮任务是计划修订，实施由 work 阶段接手。
