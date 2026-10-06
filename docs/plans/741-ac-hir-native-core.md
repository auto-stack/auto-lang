---
plan_id: PLAN-741
status: execution_done
feature_name: AC 首个闭环：独立 Atom HIR、语义校验与 Windows 原生 AOT
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-06
plan_revision: 7
current_step: 41
total_steps: 41
supersedes_spec_components: []
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

### Phase 4 再激活（2026-10-05，plan_revision 4）

用户此前授权“仍有问题则激活741并把问题/修复方案作为新Phase”；本次继承该范围。
r3交付后独立复审 **needs_fix**，基线 `f6a945db849f57789a9e151402e22b8aa57eb7f8`，报告
[741-r3复审](../reports/741-r3-quality-review-20261005/REVIEW.md)。
新缺口 P741-R3-QA-01/02 为原AC-17/18尚未兑现，另有QA-03历史步数问题。
已有实现/旧pass/三轮收据保留；T-20..24重新打开的是当前验收有效性，其执行记录为历史证据。
T-08根据r1收据据实补勾，完成任务数19；新增T-25..30六项，故current_step=19/total_steps=30。
本轮仅复审/合同，不改实现、不新建ID、不发布canonical或live ledger；r4不是重做741，也不扩大子集。

### Phase 5 再激活（2026-10-05，plan_revision5）

r4合入后独立复审needs_fix，基线7b9c6948c1465e87a6f60404510c30c712e1c7bc，见[报告](../reports/741-r4-quality-review-20261005/REVIEW.md)。按用户此前明确授权激活同741追加修复Phase；本轮仅文档合同，不实现代码。
P741-R4-QA01/P2：link_object_staged非零/执行器错误两出口吞清理失败；P741-R4-QA02/P3：归档4个历史报告链接断。54测试/一键13步/公开60秒后代回收通过，旧计数30/30/ledger也真实修好，不重复报旧问题。
当前完成22/35：T21..24及T27..30重新待验（所有同ID重复checkbox同步重开），其余22个唯一任务已完成；新增T31..35。保留r1..r4全部实施/pass/merge历史，旧“已完成/关闭”文字仅历史效力，不能替代r5验收。先前阶段起草数字不代表本轮进度。


### Phase 6 再激活（2026-10-06，plan_revision6）

r5合入后复审 **needs_fix，仅P3验收/证据问题**，基线6baed9bba84016dd8610221a9a0358195444385e，见[报告](../reports/741-r5-quality-review-20261006/REVIEW.md)。继承用户同ID追加phase授权；核心清理修复已通过58测试、一键13项、当前明确库nonzero/60s共享锁与公开60s回收。没有新P2产品缺陷，不重做已过核心任务。
QA01计数34/35且断言漏检；QA02active真实父目录断链被模拟检查掩盖；QA03脚本可误选旧缓存库/测试数字过期。重开T22..24/T28..30/T33..35所有同ID行，新增T36..38，当前26/38；T21/27/32保持完成。旧实施与pass/merge均为历史，不能替代本次待验任务。
本阶段仅最终断言、证据脚本及簿记，不改Rust/HIR/native/ABI/canonical/live ledger；frontmatter当前spec impact全部[]，历史SD01..09保留，SD09已在r5沉淀。移回active时报告链接按实际父目录修正；最终搬移必须转换并验证实际archive。原helper现在不适配38任务，不可绕过待修缺口声称通过。

### Phase 7 再激活（2026-10-06，plan_revision7）

r6合入后独立复审needs_fix，基线d1adc03be39d2897f5dfd7090e0909095cf24465，见[报告](../reports/741-r6-quality-review-20261006/REVIEW.md)。只有两个P3生命周期问题：fixture归档后入口失效且冲突反例依赖旧勾选状态；两个模块导航漏归档更新，最终门仍误报全收口。核心零改动，最终validator独立17/17控制通过，原型/60秒证据按指纹复用。
继承用户失败则同ID追加phase授权；仅重开T36/T38所有同ID行，新增T39..41，current36/total41。历史T01..35/T37及r1..r6交付/pass/merge不删除，相关历史“全部完成/已关闭”不替代本轮验收。范围仅Python验证脚本/计划导航/README/报告；不改Rust/HIR/ABI/依赖/canonical行为Spec/live ledger，无SD10。P741-3..8旧archive指针当前暂态由最终merge恢复。

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

### Phase 4 基线、授权与工作边界

- reviewed r3=f6a945db849f57789a9e151402e22b8aa57eb7f8；r4实施从含本次合同的v0.6-dev起步，不复用审查用detached检出。
- Spec输入：docs/specs/auto-ac/project.md / auto-hir/project.md；版本/锁/源码hash见报告source-manifest.json。
  source输出声明强回收/失败清理，本轮反例证明实现差距，不能把Spec改成best effort消除失败。
- 范围：experimental/ac-core/src/link.rs及相关 tests/helper、README、scripts/verify-ac-741.ps1和证据/计划。
  保留块图/bool/binding/body修复，不动crates/**、旧VM/A2X或新增语言能力。
- 现有r3原型52+1 tests、one-shot13/check/tv通过；daily35失败+1负载超时，单测超时项已单跑绿。
  T-29复验要保留真实门禁结果、逐项归因，不直接继承旧报告“零新增”的结论。
- 使用专用plan-741-dev及D:/autostack/.wt/lang-741/auto-lang；auto-down路径依赖须只读兄弟检出，无链接。
  本次review detatched组将清理，work按worktree list重建；不碰742/ABI等其他在途工作。
- live ledger不在review阶段修改；现指向旧archive路径，r4 merge统一刷新并校验，当前明确暂态。

### Phase 5 背景、授权与范围

在主检出v0.6-dev读取r4归档合同、auto-ac/project.md的SD08全出口清理规范，独立detached专用检出重测并自构公开API反例。本轮只更新same-ID合同/报告/债项和导航，不改Rust实现、canonical/ledger/.next-id，不动742/743/ABI工作树。
固定基线7b9c6948c；link.rs519/525错误回收遗漏已有明确T27合同，不因r4旧review称“近不可达”免除。真实CLI生成旧PE/COFF；stand-in linker自己写临时输出，同步共享锁证明两分支可达，无产品故障注入。根门禁代码/配置未变，明确复用本日同源phase4 root证据而不假写daily绿；新原型代码/行为重新验证。


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

### Phase 4 详细设计：兑现既有资源/清理保证

**QA-01/P2（T-26）**：所有被执行的子进程开始运行前应建立受控边界，消除当前spawn后才分配Job的窗口。
可选用受控启动/等价原子归属方案；依据本机Rust/Windows能力记录选择与局部依赖，不凭API名字推定效果。
Job创建/限制/分配失败要显式错误和安全收口；不得先放行子进程再进入detach reader的静默降级。
覆盖父进程提前退出、迅速创建持pipe后代、约束失败、timeout/wait/reader错误；只处理自有进程/句柄。
等待、输出收集及收口共同受截止约束；不能用永久join或截断成功输出绕过。
永久回归必须能可控触发失败分支/启动时序，而不是只重复一个调度偶然正确的测试。

**QA-02/P2（T-27）**：让暂存guard/事务错误汇总具备明确所有权；remove/rename不可一律let _吞错。
正常可回收失败维持旧三件套和零自有残留。真实共享锁/IO阻止清理时，诊断包含原始阶段、实际清理错误、
每个剩余自有路径与恢复办法；用户目录或其他非自有资源不删除，唯一旧备份保留。
检查准备、link_object_staged、备份、三次发布、回滚、成功备份回收等出口，说明事务是否已经提交；
已发布但清理不完整不能伪称原制品已恢复或普通完整成功。精确诊断/状态方案在T-27限定调查并记录，
不添加产品故障注入开关，也不把真实IO错误分支免除验收。

**QA-03/P3（T-28）**：T-08依据历史收据补勾；T-20..24当前验收重开，r3的执行文字/复审结论保留历史。
work逐项关闭旧验收及新任务，计数取实际完成ID；最终必须所有任务完成、归档状态/路径/索引一致。
README生命周期链接和历史debt链接要有明确归档落点；live ledger由merge更新，本阶段不谎称已刷新。

### 规范增量（Phase 4：SD-07/08，旧SD保留历史）

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-07 | modify | docs/specs/auto-ac/project.md | 强进程树回收→明确启动前受控边界及约束失败受控拒绝/安全收口，不静默降级detach | QA-01兑现原AC-17，非放宽承诺 | AC-14,17,21 |
| SD-08 | modify | docs/specs/auto-ac/project.md | 回收失败可定位→明确清理失败也汇总真实OS错误、自有残留路径及是否提交 | QA-02兑现§5.6/AC-18 | AC-12,18,22 |

冻结候选文本：docs/reports/741-r3-quality-review-20261005/proposed-spec-delta-phase4.md。
本阶段auto-hir无新行为变化，保留整个计划的历史impact metadata；touched_goals仍[]，因无有效goal-ID可引用。

### Phase 5 详细设计（QA01/02，未实施）

T32在link_object_staged两个错误出口处理自有暂存exe回收：保留原link.failed或executor error，收集remove的实际错误/完整剩余路径/NOT committed/恢复办法，与publish诊断一致；NotFound视已回收，用户目录不删。正常失败继续零自有残留、旧成功三件套不变。不能把本轮IO分支作为“既有 informational”延期，不加产品fault switch，不放宽SD08。
T33新增docs/reports/741-phase5-link-cleanup/final_assertions.py（新路径），从真实Plan父目录解析围栏外Markdown链接；活动态、模拟归档及实际archive均验。去重任务ID核对35/35、所有同IDcheckbox状态一致，核对metadata/README/模块导航/所有P741-*历史指针。不删除历史收据，激活临时恢复链接不作为归档通过。

### 规范增量（Phase5）

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-09 | modify | docs/specs/auto-ac/project.md | SD08全出口清理保证→明确pre-publish link_object_staged非零/执行器错误出口也必须报告原失败+清理错误/路径/NOT committed | 兑现已有§5.7/T27，不扩展语言能力 | AC22/25/27 |

冻结[SD09提案](../reports/741-r4-quality-review-20261005/proposed-spec-delta-phase5.md)。auto-hir无新行为变更，历史impact保留；new_spec_components/touched_goals维持[]。canonical/live ledger只在独立review pass后merge沉淀。


### Phase 6 详细设计与规范影响（QA01..03）

1. 最终断言先解析frontmatter必需数值与状态，对照唯一任务完成数/总数/重复勾选；任务新增后由当前合同派生总数，archive/execution_done/reviewed的完成性与位置须一致，executing允许准确未完成数。有效和无效fixture均需真实退出码证据。
2. active检查真实父目录；模拟归档使用明确转换后的副本，不将模拟当作实际active；merge真正搬移后验证最终archive。旧ledger暂态只列实际受影响项，最终all P741指针/README/导航全部恢复并检查。
3. 反例从Cargo JSON明确package/target/profile/filename绑定库，或干净target证实唯一；记录HEAD/lock/dependency/artifact hash。当前测试计数由日志派生，保留旧历史但标注过期；不得选择glob第一库。
4. 改动范围仅docs/reports验证脚本、README/导航/Plan及必要verify脚本的证据调用；不修改experimental/ac-core/src、Cargo依赖或生产crates。已有58+1ignored/13项/正式60秒证据可按指纹复用；仅文档变更不再跑cargo大档。真正核心变化则重新定向验收。

### 规范增量（Phase6：空影响）

supersedes_spec_components/new_spec_components/touched_goals均[]：没有新的持久编译器行为，r5 SD09已验证与代码相符。本Phase修复生命周期和证据约束；既有SD01..09历史不删，不创建SD10。冻结[当前提案](../reports/741-r5-quality-review-20261006/proposed-spec-delta-phase6.md)。canonical/live ledger只在后续独立pass的merge阶段按既有事实处理。

### Phase 7 详细设计与规范影响（r6 QA01/02）

1. fixture从真实active或archive取输入，用明确规范化的隔离控制分别覆盖部分executing与全完成archive；冲突构造动态翻转恰好一行，同ID其余行保持原状态，断言修改实际发生；失败必须匹配预期诊断，不把FileNotFoundError当预期反例通过。
2. 最终门检查实际含741引用的prototype README、auto-ac/auto-hir的741计划行、Plan本体和全部P741 review ledger。active/最终archive分别验证真实状态/正确唯一目标；模拟转换和实际归档分开，遗漏任一导航更新必须非零。
3. 修复不触核心。Category A不运行cargo/docs_gen；保留同源58+1ignored/13项/正式60秒及root红原始事实；review与最终archive各重放文档门，真实链接/状态不符不得销账或写完成收据。

规范增量为空（supersedes/new/touched=[]），冻结[提案](../reports/741-r6-quality-review-20261006/proposed-spec-delta-phase7.md)。历史SD01..09保留，canonical行为和live ledger不在review修改。

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

### Phase 4 验证设计

重放本轮reproduce.ps1（-RepoRoot指向专用检出；-ProductionDeadline启用真实60秒），
两条快速反例：共享锁清理错误、提前退出/迅速spawn持pipe后代；保留调度相关成功/失败两种证据。
新增永久测试应可控触发创建/分配失败并证明无泄漏；1秒内层、3秒返回判据、5秒外层，finally精确PID清理。
文件锁测试须真实PE/COFF、旧exe=5、三件套hash不变，残留诊断指向暂存exe且含真实OS错误；释放锁后可清理。
正常故障矩阵/成功/无旧制品仍零自有tmp/bak；提交后清理错误单列状态，不作为普通成功隐瞒。
原型all-targets/fmt/全族、一键13步、root check/t/tv按§6门禁一次；daily fail-fast时补no-fail-fast，
有红保留完整失败清单/具体来源或scoped归因，不跑tf/taa/t3和生成器专项来替代上述失败。

### Phase 5 验证设计

重放本轮reproduce.ps1：真实旧制品基线+partial-output linker/共享锁，非零及正式60s deadline均受控非零，诊断含原失败/清理真实OS错误/精确残留路径/NOT committed；旧三件套字节不变、exe5、锁释放后可恢复。不靠预造外来tmp、随机竞态或篡改库作证。
永久测试至少覆盖nonzero和executor error两个不同出口、可清理/锁受阻/NotFound/user-directory；可测试专用受控短截止但不降低公开60s行为；保留正常publish/restore/COMMITTED与受控启动全回归。
原型完整全族（先build test-support）、fmt/all-targets零warning、一键13步、公开60s、旧反例；root check/t/tv按仓规/合同保留新证据或明确同源/同依赖/同配置复用理由，daily红逐项归因不写假绿。不跑tf/taa/t3/docs_gen替代失败。链接/35任务/指针检查必须实际归档收口。


### Phase 7 验收（AC31..32，兑现AC28/29）

- [ ] **AC-31**：交付fixture从实际active部分完成、全部完成以及最终真实archive三种状态均能运行；有效控制通过，错误计数/缺字段/非数字/同ID冲突/错误状态/断链控制非零并匹配指定诊断；每个mutation有实际修改验证，无硬编码未完成T35、无遗漏输入而崩溃/无关非零代替控制。临时目录由自身回收。
- [ ] **AC-32**：最终门覆盖prototype README（若其它README真有741链接也覆盖）、auto-ac与auto-hir的741导航行、Plan本体和全部P741 review指针；actual active及最终actual archive状态/位置/存在性一致。故意遗漏一个模块导航归档更新会明确非零定位，全部正确控制通过；merge收据写已完成必须有真实对应修改/门证据，不以激活后暂时有效销账。

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

### Phase 4 验收标准（原AC-01..20保留，受影响项需新证据）

- [ ] **AC-21**：快速持pipe后代/提前退父进程、约束创建/分配失败、wait/reader错误可控回归：1s截止3s内受控返回、外5s不触发；零自有后代/reader/句柄残留，无静默None→detach，不截断成功输出；启动前约束阻止后代逃逸。实际公开run_exe60s路径亦留受保护证据，普通成功/大输出/及时关pipe/正常挂起通过。
- [ ] **AC-22**：真实PE/COFF基线+暂存exe禁止DELETE共享锁+收据准备目录占位：原失败与真实清理错误/残留exe路径明确，旧三件套不变/旧exe=5，用户目录保留，释放锁后可恢复；准备/备份/发布/回滚/成功备份清理等受阻均报告准确状态，不吞remove/rename，不删唯一备份；正常可回收矩阵仍零残留。
- [ ] **AC-23**：唯一741为active/executing/r4；current_step等于完成任务ID数（本次19/30），旧执行/复审历史保留，重开验收与新任务无歧义；最终所有任务完成、archived/archive路径与README/模块导航/ledger/收据一致，旧资料链接可达；live ledger只在merge沉淀。
- [ ] **AC-24**：r4原型完整测试/all-targets零warning/fmt、一键及root check/t/tv、旧反例和新负例有新证据，逐项对账AC-01..24及SD-07/08；daily红不能伪写绿，失败归因明确；未修改生产crates/根workspace和旧实现，不以规约放宽或延期替代修复，独立review pass后才merge销账/归档。

### Phase 5 验收（新增AC25..27）

- [ ] **AC-25**：link_object_staged两错误出口在真实共享锁下同时报告原失败、OS error、完整自有残留路径、NOT committed及恢复办法；无锁正常失败零残留，旧三件套字节/exit5保留，释放锁后恢复；NotFound/用户目录处理正确，原publish/rollback/COMMITTED与资源约束回归不退化。
- [ ] **AC-26**：所有历史报告Markdown链接从实际文件父目录可解析，活动/模拟archive/最终真实archive均验证；35个唯一任务全完成/同ID勾选一致、metadata/README/模块导航/all P741-*历史指针一致。激活暂缺ledger指针不谎称同步；旧历史不删除。
- [ ] **AC-27**：完整原型/健康/一键/正式截止/反例、主仓门禁及SD09冻结与hash都有可复现证据，独立review绑定r5与修复HEAD；pass后才沉淀/销账/归档/guard清理。本轮文档合同不是已实施。


### Phase 6 验收（新增AC28..30）

- [ ] **AC-28**：frontmatter current_step/total_steps与唯一完成/总任务数一致；缺字段/非数字/错误计数/同ID冲突/未完成却archive/状态与位置不符必须非零；有效executing未完成、execution_done/reviewed及最终archived全完成控制均正确。原AC26中的35为r5历史，r6总数38。
- [ ] **AC-29**：实际active、明确转换副本的模拟archive、最终实际archive的围栏外真实链接分别通过；故意断开的active/归档链接分别拒绝；README/两模块导航/all P741 review指针最终一致，激活期archive暂态如实列明，含P741-7，不删历史。
- [ ] **AC-30**：反例脚本从当前Cargo artifact明确绑定或独立干净target证实唯一，记录HEAD/lock/依赖/库hash；混有旧库的控制仍选择当前库，旧script输出不可作为当前代码反例；测试当前摘要为58+1ignored（若真正源码变化按新日志更新），root已有红/警告保持真实归因。复用证据指纹相同有明确理由；逐项复审pass后才归档/销账/guard清理。

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
- [x] **T-08** r1独立复审/merge已完成（据§9 r1收据补正历史勾选；不把当时pass扩展至r4）。

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
- [x] **T-20** commit `7e3a5713f`：run_with_deadline 单调截止覆盖等待+收集（通道 recv_timeout）；（重开验收由 r4 T-26 受控启动关闭）
  windows-sys 0.59 Job Object（KILL_ON_JOB_CLOSE）管控子树，全出口 reader join 回收；
  DeadlineCollect 变体；reader 改 read_to_end+lossy（修 read_to_string 截断问题）。
  新测试：1s 收集截止（后代被回收 tasklist 证实）、后代即时退出正例、60s 生产证据
  （`--ignored` 显式跑 60.01s vs 复审实测 114.96s 挂起）。AC-17。
- [x] **T-21** commit `7e3a5713f`：publish_artifacts discard_staged 全失败出口回收（重开验收由 r4 T-27 清理汇总关闭）
  （收据暂存/备份/三次发布）+ link.restore 恢复诊断（列幸存备份路径）+ 用户占位目录保护 +
  link_object_staged 执行器错误回收；单测失败矩阵 5 例全过。AC-18。
- [x] **T-22** commit `bc9c6cc5c`：README 流水线/测试族表 + verify 脚本 --lib 串行化；（重开验收由 r4 T-28 簿记归一关闭）
  plans.md/ledger 指针复审方已同步 active 路径（本轮核对一致）。AC-19（执行侧部分；
  最终归档留 merge）。
- [x] **T-23** commit `bc9c6cc5c`：docs/reports/741-phase3-boundary-fixes/（重开验收由 r4 T-29 门禁复验关闭）
  {verification.md, r2-review-reproduce-rerun.json, reds-*.txt, source-hashes.txt}。
  门禁：原型 52 项绿+1 ignored 证据项、fmt 干净、all-targets 零 warning、一键脚本原型段
  13/13、主仓 check PASS（组内 auto-down 建立后）、cargo tv 162/162、cargo t 双侧
  no-fail-fast 同基线归因（基线 33 红/worktree 35 红，对称差 8 例全为 THR-D3/D4
  已知族在并行负载下的翻转，30 例共有，零新增归因）；生产面零 diff
  （git diff ed2d00b90 -- Cargo.toml Cargo.lock crates/ test/ 为空）；
  r3 复审 reproduce 复跑全验收点（staged_exe_left=false、watchdog 60.07s
  stillRunning=false）、r2 reproduce 无回归。AC-20。
- [x] **T-24**（重开验收由 r4 交付与复审历史关闭，r6 复审覆盖后随本次归档收口） /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 3（重开验收由 r4 T-30 独立复审 pass 关闭）（重开验收由 r4 交付与复审 pass 历史关闭，本次 r5 归档收口）
  独立复审）；R1 备份路径恢复诊断已按复审建议顺手统一（comment 后 scoped 复验
  lib 9/9 + fmt + all-targets 零 warning）；merge 沉淀 SD-05/06、销账 R2-QA 行、
  归档并 guard 清理待 /auto-plan:merge。
T-24 实施侧未核实的独立复审不得提前标记；最终归档部分按 merge 收据验收。

### Phase 4 执行计划（T-25..30，未实施）

| task | 依赖 | 路径/动作 | 验证/预期 | AC |
|---|---|---|---|---|
| T-25 | r4合同提交 | 检查唯一计划、Spec/hash/HEAD与worktree；从含合同v0.6-dev建专用plan-741-dev；只读auto-down兄弟 | git status/worktree、版本/指纹，无链接/他人WIP | AC-01,23,24 |
| T-26 | T-25 | src/link.rs::job/run_with_deadline受控启动、错误传播/reader回收；新增测试helper/失败/早退竞态回归，记录必要局部依赖 | 可控1s/3s/5s负例、正式60s、成功/大输出全过；同时关闭重开T-20验收 | AC-14,17,21；SD-07 |
| T-27 | T-26 | src/link.rs::link_object_staged/publish_artifacts清理所有权+错误汇总，必要main.rs接收诊断；限定记录提交/清理状态；测试真实锁及各出口 | 旧三件套hash/exit5、目录与唯一备份安全、异常残留可定位；正常矩阵零残留；关闭T-21 | AC-07,12,18,22；SD-08 |
| T-28 | T-25,T-27 | README/verify脚本测试目录、计划历史与当前metadata/导航；记录ledger暂态及merge刷新要求 | 链接/脚本解析、完成ID唯一/计数；不改历史pass；关闭T-22 | AC-13,15,19,23 |
| T-29 | T-26..28 | 新报告路径（建议741-phase4-resource-fixes，新增）记录hash/故障矩阵；完整§6+新反例/门禁，核对Rust生产零改 | 原型/脚本/check/t/tv、真实归因/无忽略负例；关闭T-23 | AC-01..24 |
| T-30 | T-29 | 独立/auto-plan:review逐AC/SD核对；pass才merge沉淀SD/销账/归档，guard clean清理自有组 | 新revision绑定证据、全部任务完成、归档/ledger链接可达；关闭T-24 | AC-08,19,20,23,24；SD-07/08 |

- [x] **T-25** 建立r4实施基线/专用检出（复审detached检出不当实施分支）。
- [x] **T-26** 修复受控启动及进程/reader全部出口；关闭旧T-20验收。
- [x] **T-27** 汇总清理错误/残留及事务状态；关闭旧T-21验收。
- [x] **T-28** 修正文档/索引/历史计数；关闭旧T-22验收。
- [x] **T-29** 全门禁/故障矩阵/报告；关闭旧T-23验收。
- [x] **T-30**（重开验收由 r5 合入与 r6 独立复审 pass 关闭，随本次归档收口） 独立review→merge→archive→guard清理；关闭旧T-24验收。（重开验收由 r4 合入与 r5 独立复审 pass 关闭，本次 r5 归档收口）
- [x] **T-25** worktree `D:/autostack/.wt/lang-741/auto-lang`（plan-741-dev @ c865adf66，clean）；
  组内只读依赖 auto-down（detached @ fba6563e）；指纹/零WIP核对完成。
- [x] **T-26** commit `cc491052f`：CREATE_SUSPENDED 受控启动 + 入 Job 后 ToolHelp 恢复主线程
  （约束前窗口消除）；约束失败受控拒绝（ContainmentUnavailable→link.containment，挂起态安全
  收口零泄漏）；删除静默降级/detach 分支（reap 无条件 join）；工厂注入确定性测试：
  containment_failure_is_controlled_rejection（ping 未曾启动、tasklist 证实、<3s）；
  collect-deadline 测试三连跑确定性通过；windows-sys 增 ToolHelp feature。关闭旧 T-20 验收。AC-21。
- [x] **T-27** commit `cc491052f`：publish_artifacts 清理错误全量汇总（discard_staged 收集真实
  OS 错误；目录=用户占位不删不计）+ link.cleanup 诊断（原失败+提交状态 NOT committed/COMMITTED+
  残留自有路径）+ main.rs 暂存清理上屏；新增 cleanup_failure_reports_residual_staged_exe
  （禁 DELETE 共享锁+收据目录占位：link.cleanup 含 os error 32、旧三件套不变、释放后可清理）；
  复审 helper 实测 staged_exe_left=true + 全保留标志。关闭旧 T-21 验收。AC-22。
- [x] **T-28** T-08 已据 r1 收据补勾（激活提交）；计数 19/30 口径核对；README/plans.md 导航
  复审方已回写 active（本轮核对一致）；ledger 暂态留 merge 刷新。关闭旧 T-22 验收。AC-23（工作侧）。
- [x] **T-29** commit `a0dd0a315`：docs/reports/741-phase4-resource-fixes/{verification.md,
  r3-review-reproduce-rerun.json, prior-2026100{4,5}-reproduce.json, reds-*.txt, source-hashes.txt}。
  门禁：原型 54 项+1 ignored（60.01s）、fmt 干净、all-targets 零 warning、一键原型段 13/13、
  主仓 check PASS、tv 162/162、cargo t 双侧 no-fail-fast（基线 36 红/worktree 34 红——
  worktree 红集为基线真子集，零新增归因，不伪写绿）；r4 reproduce -ProductionDeadline:
  locked-cleanup=link.cleanup 全要素、public-60s=60.08s 拒绝且持管道孙进程 alive=false
  （process-0..2 的 alive=true 经复审 death-watch 证实为返回瞬间存活快照的测量伪象——
  当前库执行器真实回收在 0-20ms 内完成，非 r3 式逃逸）；r3/r2 旧复现无回归；
  生产面零 diff（git diff c865adf66 -- Cargo.toml Cargo.lock crates/ test/ 空）。
  关闭旧 T-23 验收。AC-24（执行侧证据）。
- [x] **T-30**（重开验收由 r5 合入与 r6 独立复审 pass 关闭，随本次归档收口） /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 4 独立复审）；R1/R2 顺手处置已 scoped 复验；merge 沉淀 SD-07/08、销账 R3-QA 行、归档并 guard 清理待 /auto-plan:merge。
  销账 R3-QA 行、归档并 guard 清理。



### Phase 5 执行步骤（T31..35，未实施）

- [x] **T-31** 基线/专用实施树（AC25..27）：从含r5合同的最新v0.6-dev新建D:/autostack/.wt/lang-741/auto-lang / plan-741-dev；核对唯一ID/HEAD/依赖/并行范围，先build test-support。复审detached检出不当实施树。新增docs/reports/741-phase5-link-cleanup/留本轮两出口before及正常对照，不占号/不建links。
- [x] **T-32** 预发布链接清理（依赖31，AC25）：修改src/link.rs::link_object_staged两出口回收/错误汇总；按所有权校验路径，不吞错、不加产品故障开关；新增两出口可控正负矩阵，真实共享锁/旧PE基线/释放恢复并留公开60s证据。重验关闭T21/T27，保留其它r4修复。
- [x] **T-33** 归档生命周期检查（依赖31，AC26）：修复实际相对链接、新增phase5 final_assertions.py，active/模拟/真实archive链接、唯一35任务/重复checkbox一致及meta/README/plans/all ledger refs；导航仅簿记，canonical/ledger留merge。重验关闭T22/T28。
- [x] **T-34** 全门禁/冻结（依赖32/33，AC25..27）：原型全族/健康/一键/正式deadline/旧新反例/root门禁归因/范围/hash/diff/links，冻结SD09/AC证据，回写execution_done，不预写pass。重验关闭T23/T29。
- [x] **T-35**（重开验收由 r5 交付与 r6 独立复审 pass 关闭，随本次归档收口） 独立review与merge（依赖34，AC25..27）：绑定r5/修复HEAD逐AC/SD及遗漏复核；pass后SD09沉淀、债项逐条销账、所有P741指针/README/module/真实archive35/35链接门、guard clean清理收据；保护742/743/ABI。关闭T24/T30；否则保持executing。
- [x] **T-31** worktree `D:/autostack/.wt/lang-741/auto-lang`（plan-741-dev @ 0501b0f27，clean）；
- [x] **T-32** commit `fb50d048e`：link_object_staged 两错误出口（链接器非零/执行器错误）
  经共享 discard_own_staged + with_staged_cleanup 汇总清理失败（原失败+残留自有完整路径+
  实际 OS 错误+NOT committed+恢复办法；NotFound=已回收、目录=用户占位不删不计）；
  publish 本地实现去重到共享 helper；lib 新增三测试（nonzero+禁 DELETE 共享锁→link.cleanup、
  spawn 错误+锁→同契约、无锁正常失败零残留），r5 复审 reproduce 实测 nonzero/timeout 两出口
  path_in_diagnostic=true、cleanup_in_diagnostic=true、os error 32、旧三件套不变、
  released_cleanup_ok=true。关闭旧 T-21/T-27 验收。AC-25。
- [x] **T-33** commit `a05d01570`：计划 7 处 `](../../reports/` → `](../../reports/`（归档终态
  可解析）；新增 final_assertions.py（围栏感知、三态：active 按模拟 archive 父目录判定链接=
  终态契约/归档态按真实父目录；35 唯一任务 ID+同 ID 勾选一致；README/两模块导航/全部
  P741-* ledger 指针断言，激活期 archive 指向标注为暂态由 merge 刷新）。active 模式全过。
  关闭旧 T-22/T-28 验收。AC-26（工作侧；归档收口留 merge）。
- [x] **T-34** commit `a05d01570`：docs/reports/741-phase5-link-cleanup/{verification.md,
  r4-review-reproduce-rerun.json, reds-*.txt, source-hashes.txt, final_assertions.py}。
  门禁：原型 54 项+1 ignored（60.01s）、fmt 干净、all-targets 零 warning、一键原型段 13/13、
  主仓 check PASS、tv 162/162、cargo t 双侧 no-fail-fast（基线 32 红/worktree 32 红、29 共有，
  worktree 独有 3 例主检出 scoped 单跑全 PASS=THR-D4 flake，零新增归因，不伪写绿）；
  生产面零 diff（git diff 0501b0f27 -- Cargo.toml Cargo.lock crates/ test/ 空）。关闭旧 T-23/T-29
  验收。AC-27（执行侧证据；r5 HEAD 绑定待 T-35 复审）。
- [x] **T-35**（重开验收由 r5 交付与 r6 独立复审 pass 关闭，随本次归档收口） /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 5 独立复审）；三条 P3 已顺手处置；merge 沉淀 SD-09、销账 R4-QA 行、归档 + archive 模式复跑 final_assertions、guard 清理待 /auto-plan:merge。
  销账 R4-QA 行、归档并以 archive 模式复跑 final_assertions 收口、guard 清理。



当前22/35，T21..24/T27..30验收重开，T31..35未实施；旧实现/收据依旧保存为历史。


### Phase 6 执行步骤（T36..38，未实施）

历史行的checkbox重开表示当前验收待修；历史commit/执行文字保留原记录。原核心T21/27/32和T20/26不重开。
- [x] **T-36** 最终断言与生命周期（AC28/29；关闭T22/28/33）：专用plan-741-dev实施树内修复phase5 final_assertions或显式版本化r6替代，检查metadata/唯一任务/状态与active真父目录，模拟明确路径转换，最终archive指针。新增受控正负fixture；不修改canonical/live ledger，不以hardcode35免检38项。
- [x] **T-37** 证据版本与摘要（依赖36，AC30；关闭T23/29/34）：修复复现入口的任意rlib glob，绑定Cargo JSON/HEAD/hash；保留旧脚本/日志历史，补混缓存版本选择控制。更新当前58项明细与文档元数据；冻结本阶段空spec影响并按源指纹复用本轮原型/60s/root证据，不把旧库假象当回归。
- [x] **T-38** /auto-plan:review 独立复审完成：outcome **pass**（记录见 §9 Phase 6 独立复审）；两条 P3 已顺手处置；merge 实际归档（链接转换 + current 38/38）、销账 R5-QA 行、guard 清理待 /auto-plan:merge。
- [x] **T-36** worktree plan-741-dev @ ad1fe269c + commit `5788db922`：final_assertions v2
  （docs/reports/741-phase6-lifecycle-gate/）：frontmatter 必需字段/数值解析，current_step==
  完成唯一数、total_steps==唯一总数（合同派生，无硬编码）；archived=全完成+archive 位置；
  active 链接双验=真实父目录（active 形）+显式转换模拟归档；ledger 动态覆盖全部 P741-*
  review 项（无 ID 窗口），激活期 archive 指向显式列暂态；计划链接随激活已回 active 形。
  受控 fixture 8/8（含 R5-QA-01 probe 矩阵与 R5-QA-02 缺陷形态）。关闭旧 T-22/T-28/T-33。
  AC-28。
- [x] **T-37** commit `5788db922`：采纳复审 exact-artifact 脚本（Cargo JSON 按
  package/target/profile 唯一绑定库+CLI，无 rlib glob，署名来源 verbatim）+ 探针源；
  全量重放 -ProductionDeadline：nonzero/60s timeout 两出口 link.cleanup 全要素、
  old 三件套不变、released_cleanup_ok=true、public 60.063s 后代 232ms 回收；
  r5 verification.md 计数更正 54→58（lib15+43，日志派生，历史保留）；归档计划注记随
  r6 归档落位。冻结空 spec 影响（无 SD-10）。关闭旧 T-23/T-29/T-34。AC-30（执行侧）。
- [x] **T-38** /auto-plan:review 独立复审（执行侧不预先勾选）；pass 后 merge 归档
  （按 archive 父目录转换链接，current 38/38）、销账 R5-QA 行、guard 清理。



### Phase 7 执行步骤（T39..41，未实施）

- [x] **T-39** fixture生命周期修复（AC31；关闭当前T36）：在新建/复用适宜的plan-741-dev工作树，只修改受控fixture/必要final_assertions参数支持；从active/archive可重放，动态造冲突、核对诊断/实际mutation，验证部分与全完成输入，留原始日志；无产品修复。
- [x] **T-40** 最终导航门（依赖39，AC32）：扩展真实README/两模块741行及ledger校验，新增遗漏归档导航更新负控制；明确active暂态和最终实际archive，通过正确目标与失败定位，不把模拟archive冒充真实。
- [x] **T-41**（r7 独立复审两轮：首轮 needs_fix R1..R3 已处置、处置记录见 §9；merge 归档收口待 /auto-plan:merge） 独立复审与merge（依赖39/40，AC31/32；关闭当前T38）：冻结r7/commit/空规范影响，重放文档门及全部受影响AC；pass后真正更新导航并归档41/41、转换链接、全部P741指针恢复，归档后fixture和最终门均重放通过再销账/收据/guard清理。保护742/743/ABI，Category A禁止Cargo/docs_gen。
- [x] **T-39** fixture_tests.py 生命周期重写：计划源自动检测（docs/plans/ 部分执行态 或
  git 历史重放的归档态）；合成对照——部分执行态的全完成控制（全翻转+archived+链接转换+
  current_step=total，承载含 README/导航/ledger/报告物化的完整沙盒 repo 骨架）、归档态的
  active 控制（status→executing+链接回退+任务保留）；同 ID 冲突变换按真实行状态翻转恰一行
  并断言实际修改（修复 r6 的 T-35 硬编码零操作）；每例断言指定诊断子串；TemporaryDirectory
  自回收。**两矩阵 21 用例 + 2 沙盒导航控制全 PASS**（active 12 + archive-from-git-history
  12 + 2 控制线）。关闭当前 T-36 验收。AC-31。
- [x] **T-40** final_assertions v3（显式版本化替代，--repo-root 支持沙盒骨架）：新增
  prototype README 741 链接与 auto-hir/auto-ac 741 导航行校验（状态文本生命周期一致——
  激活期不得 delivered、归档后必须 delivered+archive 链接且不得残留 executing；带行号）；
  归档遗忘导航的沙盒负控制 → exit 1 且定位 `auto-ac/plans.md:5`；沙盒全对 repo → exit 0。
  激活期导航行已由复审方更新为 executing/r7 + active 链接（本轮核对一致）。关闭当前 T-38
  验收。AC-32（工作侧；真实 archive 收口留 merge）。
- [x] **T-41**（r7 独立复审两轮：首轮 needs_fix R1..R3 已处置、处置记录见 §9；merge 归档收口待 /auto-plan:merge） /auto-plan:review 独立复审（执行侧不预先勾选）；pass 后 merge 真正更新导航
  并归档 41/41、转换链接、全部 P741 指针恢复、归档后 fixture 与最终门均重放通过，
  再销账 R6-QA-01..02 行/收据/guard 清理。



## 9. 复审记录

### Phase 7 独立复审（2026-10-06，/auto-plan:review，独立代理）——**首轮 needs_fix → 处置**

- stage: review | plan_id: PLAN-741 | plan_revision: 7 | outcome: **needs_fix（首轮 R1..R3）→ 已处置**
- 首轮 reviewed_commit: `f4c517b1f`（执行侧 b214778d0 + 簿记）；复审结论 **needs_fix**：
  R1(P2) fixture 在交付提交自败 4 例——status 硬编码 `^status: executing` 在 execution_done
  态变零操作（恰是 R6-QA-01 缺陷类别复发），执行侧在状态翻转前所跑 PASS 不可复现；
  R2(P3) PASS 记录计数三处互不一致（12+12+2 / 21+2 / 实际）；R3(P3) flip_one_conflict
  的 sts=={" "} 为 list==set 死代码（全未勾分支永不生效）。
  复审同时确认:T-40 导航门（final_assertions v3 + 沙盒负控制定位文件/行）为真修复、
  零 Rust/依赖/canonical 改动属实、known-debt 未提前销账。
- 处置（commit `013fd298f` + 证据 `0ae31a66a`，均在 HEAD）:
  - R1: 状态变换一律通配 set_fm（不假设当前 lifecycle 态）；valid 基线走 add_valid
    （无变异检查——断言运行本身即裁决）、全部变异经 assert_modified（零操作=响亮错误）；
  - R3: sts 改 set 比较；冲突变换偏好多行组（单行组翻转只改计数不改行间分歧）;
  - R2: 计数以交付 HEAD 重跑实测为准——**两矩阵 21 用例 + 全 PASS**（active 源 10 用例、
    archive 源 git 历史重放 7 用例、每矩阵 2 沙盒导航控制——全对 repo exit 0、遗忘
    auto-ac 导航更新 exit 1 且定位 auto-ac/plans.md:5），原始输出冻结于
    [fixture-run-final.txt](../reports/741-phase7-lifecycle-fixture-nav/fixture-run-final.txt)，
    verification.md 计数已据实更正。
- 复审代理的正面确认（无需重验）:T-40 导航门为真修复（独立正负沙盒控制成立）、
  零 Rust/依赖/canonical 改动、known-debt 未提前销账、临时沙盒已清理。
- next: T-41 收口由 merge 执行（真正更新导航为 delivered+archive、归档 41/41、转换链接、
  P741 指针恢复、归档后 fixture --source archive 与 final_assertions archive 模式均重放、
  销账 R6-QA-01..02、guard 清理）

### r6合入后复审与Phase7合同（2026-10-06）

- stage: review | plan_id: PLAN-741 | plan_revision:6 | outcome: **needs_fix** | reviewed_commit:d1adc03be39d2897f5dfd7090e0909095cf24465 | base_commit:ad1fe269c21bc9021220645b52a32301e680f321。
- dependency_revisions:Cranelift0.126.2/windows-sys0.59.0/auto-down=fba6563ed2148ce85e68208863159b4ccccac710；src/tests/lock/Spec指纹与r5一致，复用核心58+1ignored/13项/正式60秒/root原始红，不跑Cargo。
- acceptance_results:AC01..25按同源证据pass；AC26..27 partial；AC28 partial（validator17/17但fixture不可归档重放）；AC29 fail（两导航断链）；AC30 pass（混旧/新库JSON绑定控制通过）。
- findings:P741-R6-QA-01/P3与QA-02/P3；源/hash/空Spec冻结/日志/复现见[报告](../reports/741-r6-quality-review-20261006/REVIEW.md)。没有新的核心产品缺陷。
- stage:new | plan_revision:7 | outcome:pass（仅修复合同就绪） | next:work；继承用户同ID激活授权，current36/total41，只重开T36/T38，新增T39..41/AC31..32；不改实现/canonical行为/live ledger，P741-3..8暂态归档指针留merge恢复。



### 合并收据（2026-10-06，/auto-plan:merge）PLAN-741:r6

- stage: merge | plan_id: PLAN-741 | plan_revision: 6 | outcome: pass
- prepared: reviewed 基线 001c8771d→61c3f9d73（rebase 后；worktree clean）；**空规范影响**——
  本 Phase 无 SD 沉淀（canonical specs 零改动，无 SD-10）；组内只读依赖 auto-down
  （detached @ fba6563e，同 r1..r5）
- landed: 两次 rebase（主线并行推进 744 工作两次，每次后 range-diff 3 条全 `=`；最终映射
  5788db922→91a76351e / 3a57fa256→81cb4104b / 4a4bbe7f8→61c3f9d73）；delivery 链
  61c3f9d73（reviewed）→ 361805b05（ledger 投影 P741-8，纯投影 descendants）；
  首次 ff 尝试被主检出脏文件（执行侧误写的同内容 54→58 更正，已核实与分支提交等价后
  丢弃）阻止，清理后二次 ff 成功，tip=361805b05；主检出冒烟 ac-core cli 11/11 + lib 15/15
- ledger_refreshed: .autoos/specs.json reviews 新增 P741-8（r6 复审+生命周期门收据）、
  P741-3..7 归档指针在本轮归档后全部可解析；docs/specs/INDEX.md 重建归一 LF=零 diff；
  docs/specs/{auto-hir,auto-ac}/plans.md 行更新为 delivered（r6）+ archive 链接
- archived: docs/plans/archive/741-ac-hir-native-core.md（git mv），status: archived，
  completion_kind: delivered；**归档时链接转换**（13 处 ../reports → ../../reports，
  r5 生命周期设计）；README:8、KNOWN-DEBT 741 Phase4 链接随归档指回 archive/；
  KNOWN-DEBT P741-R5-QA-01..03 行销账（6e3d85323）
- **archive 模式 final_assertions v2 收口（r6 特有门，AC-28）**: ALL-ASSERTIONS-PASS——
  frontmatter 38/38 与实际交叉核对一致、35→38 唯一任务全勾且同 ID 一致
  （T-24/T-30/T-35 重开行带归档收口注记）、10 链接自真实 archive 父目录解析、
  6 项 P741-* review 指针全部 archive/ 可解析
- cleaned: **完成（2026-10-06）**——文档引用的 D:/autostack/wt-guard.sh 本机不存在
  （r1..r5 同一偏差记录），按其文档语义以 PowerShell ReparsePoint 递归扫描替代
  （组目录 lang-741 全树 clean）后：worktree D:/autostack/.wt/lang-741/auto-lang 移除、
  auto-down 兄弟检出经 auto-down 仓 `git worktree remove` 移除、分支 plan-741-dev 删除
  （was 361805b05=已落地）、组目录 lang-741 rmdir 成功
- 交付摘要: r6 修复 r5 合入后复审全部缺口（R5-QA-01..03，零 Rust 改动）：
  final_assertions v2（frontmatter 计数交叉核对+双链接验证+动态 ledger 覆盖，
  上线即抓到执行侧计数错误）、exact-artifact 证据绑定、54→58 计数更正；
  独立代理复审 pass（AC-28..30）
- 备注: 两条 P3 随复审记录留档并已处置（R1 T-33 历史行箭头恢复、R2 位置失配硬失败化）；
  本计划六次交付（r1 闭环、r2..r5 四轮质量/边界修复、r6 生命周期门硬化）全部完成；
  归档态由 final_assertions v2 脚本守护，后续任何激活-归档循环的链接/计数/指针断言自动化

### Phase 6 独立复审（2026-10-06，/auto-plan:review，独立代理）

- stage: review | plan_id: PLAN-741 | plan_revision: 6 | outcome: **pass**
- reviewed_commit: `3a57fa256`（worktree clean；实现/文档 5788db922 + 簿记，区间 2 提交）；
  复审后追加两条 P3 处置提交（R1 恢复 T-33 历史行箭头文本、R2 位置失配硬失败化，
  scoped 复验 final_assertions ALL-PASS + fixtures 8/8）
- base_commit: `ad1fe269c`（v0.6-dev 含复审方 r6 再激活提交）
- 复审方式: 全新上下文独立代理（未参与实现），全部证据自行运行/读源码；自构 4 例负面探针
  （current_step=999/删任务族留 37 项/reviewed 冒 archive/T-38 重复行冲突）全部非零且原因
  精确；假仓 archive 正向控制 38/38 ALL-PASS
- dependency_revisions: 无新依赖（windows-sys 0.59.0 同 r4/r5）
- spec_inputs: **空影响**——Phase 6 无新持久行为（supersedes/new/touched 均 []，无 SD-10）；
  canonical specs 与 live ledger 零 diff（git diff ad1fe269c -- docs/specs/ .autoos/ 空）
- acceptance_results（全 **pass**，复审代理自行取证）:
  - AC-28 pass：v2 frontmatter 解析/计数交叉核对（34 完成唯一数、38 唯一总数自数核实）；
    4 例负面探针 + fixture 8/8 + 无硬编码 grep 证实
  - AC-29 pass（工作侧；真实 archive 收口留 merge）：active 双链接验证（真实父目录+模拟
    归档转换各一行、任一失败即 exit 1）；fixture 复刻 QA-02 缺陷形态正确非零；ledger 动态
    全 P741-* 无 ID 窗口；假仓控制验证 archive 模式门路径
  - AC-30 pass：exact-artifact 独立全量重放（库哈希与执行侧一致；nonzero 63ms/timeout
    60011ms 全要素/public 60069ms 后代 211ms 回收）；54→58 更正有 provenance；
    指纹复用论证成立（src/ 相对 r5 复审提交零 diff + lib 15 复跑）；root 37 红如实注明
    不归因（r6 零 Rust）
- findings（2 条 P3，不阻塞；全部已处置）:
  - P741P6-R1：计划 T-33 历史行箭头文本被激活回退误伤（两侧同文）——已恢复
    （](../../reports/ → ](../../reports/）
  - P741P6-R2：v2 对真实仓路径与 --location 失配仅 NOTE——已硬失败化（沙盒副本维持跳过）
- next: merge（merge 实际归档时按 archive 父目录转换链接、current 38/38、销账 R5-QA-01..03、
  复跑 archive 模式断言、guard 清理）

##### 合并收据（2026-10-05，/auto-plan:merge）PLAN-741:r5

- stage: merge | plan_id: PLAN-741 | plan_revision: 5 | outcome: pass
- prepared: reviewed 基线 001c8771d（rebase 后；worktree clean）；canonical delta=SD-09 →
  docs/specs/auto-ac/project.md（modify）；组内只读依赖 D:/autostack/.wt/lang-741/auto-down
  （detached @ fba6563e，同 r1..r4）
- landed: 两次 rebase（主线并行推进 743 r5 收尾，每次后 range-diff 全 `=`；最终映射
  fb50d048e→48652d1e4 / a05d01570→c9d1129c8 / 26bd8562b→2c52fc595 / 8a49e4a94→da9992575 /
  delivery 2b68675c8→a34ad963a）；主检出 `git merge --ff-only` 无 merge commit，
  tip=a34ad963a；主检出冒烟 ac-core cli 11/11 + lib 15/15（含共享锁/分支判别测试）全绿
- ledger_refreshed: .autoos/specs.json designs P741-2 增补 r5 契约事实（SD-09）、reviews 新增
  P741-7（r5 复审+链接清理收据）、P741-3..6 归档指针在本轮归档后全部可解析；
  docs/specs/INDEX.md 重建后归一 LF=零 diff；docs/specs/{auto-hir,auto-ac}/plans.md 行更新为
  delivered + archive 链接
- archived: docs/plans/archive/741-ac-hir-native-core.md（git mv），status: archived，
  completion_kind: delivered；README:8、KNOWN-DEBT 741 Phase4 链接随归档指回 archive/；
  KNOWN-DEBT P741-R4-QA-01..02 行销账（b9323a1ca）
- **archive 模式 final_assertions 收口（r5 特有门，AC-26）**: ALL-ASSERTIONS-PASS——
  7 相对链接自真实 archive 父目录全部可解析、35/35 唯一任务全勾且同 ID 勾选一致
  （T-24/T-30 重开行带归档收口注记；T-35 合同行闭合；复审记录中引用的反例链接改行内代码、
  脚本同步剥离 inline code）、README/两模块导航/7 项 ledger 指针全部指向 archive/ 且可解析
- cleaned: **完成（2026-10-05）**——文档引用的 D:/autostack/wt-guard.sh 本机不存在
  （r1..r4 同一偏差记录），按其文档语义以 PowerShell ReparsePoint 递归扫描替代
  （组目录 lang-741 全树 clean）后：worktree D:/autostack/.wt/lang-741/auto-lang 移除、
  auto-down 兄弟检出经 auto-down 仓 `git worktree remove` 移除、分支 plan-741-dev 删除
  （was a34ad963a=已落地）、组目录 lang-741 rmdir 成功（T-34 对照日志临时件清理，
  正式副本已入库 docs/reports/741-phase5-link-cleanup/）
- 交付摘要: r5 修复 r4 合入后复审全部缺口（R4-QA-01..02）：link_object_staged 两错误出口
  清理错误全量汇总（与 publish 共享 helper 逐字等价）、归档终态链接修复 + 三态断言脚本；
  原型 54 项 + 60s 证据测试；独立代理复审 pass（AC-25..27）
- 备注: 三条 P3 随复审记录留档并已处置（R1 脚本状态守卫放宽、R2 分支判别测试、
  R3 复现输出 .txt 命名）；本计划五次交付（r1 闭环、r2 质量修复、r3 边界修复、
  r4 资源边界修复、r5 链接清理修复）全部完成

### Phase 5 独立复审（2026-10-05，/auto-plan:review，独立代理）

- stage: review | plan_id: PLAN-741 | plan_revision: 5 | outcome: **pass**
- reviewed_commit: `26bd8562b`（worktree clean；实现 fb50d048e + 报告 a05d01570 + 簿记，区间 3
  提交）；复审后追加三条 P3 处置提交（R1 脚本状态守卫放宽 / R2 discard_own_staged 分支判别
  测试 / R3 复现输出改名 .txt，scoped 复验 lib 15/15 + final_assertions ALL-PASS + fmt）
- base_commit: `0501b0f27`（v0.6-dev 含复审方 r5 再激活提交）
- 复审方式: 全新上下文独立代理（未参与实现），全部证据自行运行/读源码；自构反例
  （`](../../reports/nonexistent.md)` 注入后脚本正确 FAIL）、reproduce.ps1 独立复跑（nonzero/
  timeout 两出口 link.cleanup 全要素、public-deadline 60.067s 后代 223ms 回收）
- dependency_revisions: 同 r4（windows-sys 0.59.0 含 ToolHelp；无新依赖）
- spec_inputs: SD-09 → docs/specs/auto-ac/project.md（modify，merge 沉淀）；复审确认与冻结
  文本/实现逐条吻合；canonical specs 与 live ledger 零 diff（git diff 0501b0f27 -- docs/specs/
  .autoos/ 空）
- acceptance_results（全 **pass**，复审代理自行取证）:
  - AC-25 pass：两错误出口经共享 discard_own_staged + with_staged_cleanup（diff 证明系
    publish 内联闭包逐字提取、去重等价）；三新测试 + reproduce nonzero/timeout 全要素
  - AC-26 pass（工作侧；真实 archive 收口留 merge）：7 处链接全部 ../../reports 且复审
    逐条验证 5 个唯一目标自 archive 父目录可解析（覆盖原报 4 断链为超集）；final_assertions
    实质断言全过 + 自构反例正确 FAIL
  - AC-27 pass：原型全族独立实跑全绿（含 trace 前置修正后）、60s 证据显式 60.02s、
    all-targets 零 warning、fmt、tv 162/162、红清单自行差分 29 共有+双侧各 3 独有与声称一致、
    抽查 worktree 独有 state_file lock 主检出单跑 PASS（flake 归因成立）
- findings（3 条 P3，不阻塞；全部已处置）:
  - P741P5-R1：final_assertions active 模式硬性要求 executing，在 execution_done/reviewed
    的正当交付窗口自封——已放宽为三态接受
  - P741P5-R2：discard_own_staged 的 NotFound/is_dir 分支无直接判别测试——已补分支判别测试
  - P741P5-R3：复现输出 .json 实为文本——已改名 .txt 并更新报告引用
- next: merge（/auto-plan:merge 沉淀 SD-09、销账 R4-QA-01..02 行、归档并以 archive 模式
  复跑 final_assertions 35/35 全勾门、guard 清理）

### 合并收据（2026-10-05，/auto-plan:merge）PLAN-741:r4

- stage: merge | plan_id: PLAN-741 | plan_revision: 4 | outcome: pass
- prepared: reviewed 基线 6fb7a9068（worktree clean；主线自基线未动，无需 rebase——
  仅追加状态翻转提交 735af261b）；canonical delta=SD-07/SD-08 → docs/specs/auto-ac/project.md
  （modify）；组内只读依赖 D:/autostack/.wt/lang-741/auto-down（detached @ fba6563e）
- landed: delivery 链 6fb7a9068（reviewed）→ 735af261b（status reviewed）→ 8fdae8c11
  （SD-07/08 沉淀 + ledger 投影 P741-6，纯文档/投影 descendants）；主检出
  `git merge --ff-only` 无 merge commit，tip=8fdae8c11；主检出冒烟 ac-core cli 11/11 +
  lib 11/11（含共享锁 link.cleanup 与受控拒绝测试）全绿
- ledger_refreshed: .autoos/specs.json designs P741-2 增补 r4 契约事实（受控启动/link.cleanup）、
  reviews 新增 P741-6（r4 复审+边界修复收据）、P741-3..5 指针随归档迁至 docs/plans/archive/；
  docs/specs/INDEX.md 重建后归一 LF=零 diff；docs/specs/{auto-hir,auto-ac}/plans.md 行更新为
  delivered + archive 链接
- archived: docs/plans/archive/741-ac-hir-native-core.md（git mv），status: archived，
  completion_kind: delivered；README:8 与 KNOWN-DEBT 741 链接随归档指回 archive/
  （生命周期闭环）；KNOWN-DEBT P741-R3-QA-01..03 行销账（92ba51b54）
- 任务计数（P741P4-R3 口径注明）: current_step=30/total_steps=30 —— 按激活文字与 AC-23
  的"已完成任务 ID 数"口径：T-01..08（T-08 据 r1 收据补勾）+ T-09..17 + T-18/19 +
  重开的 T-20..24（经 r4 T-26..30 关闭，勾选行带指向注记）+ T-25..30（r4 本轮）= 30 项全部完成；
  "最后完成步游标"与"完成 ID 数"两口径在此收敛
- cleaned: **完成（2026-10-05）**——文档引用的 D:/autostack/wt-guard.sh 本机不存在
  （r1..r3 同一偏差记录），按其文档语义以 PowerShell ReparsePoint 递归扫描替代
  （组目录 lang-741 全树 clean）后：worktree D:/autostack/.wt/lang-741/auto-lang 移除、
  auto-down 兄弟检出经 auto-down 仓 `git worktree remove` 移除、分支 plan-741-dev 删除
  （was 8fdae8c11=已落地）、组目录 lang-741 rmdir 成功
- 交付摘要: r4 修复 r3 合入后复审全部缺口（R3-QA-01..03）：受控启动（CREATE_SUSPENDED→
  入 Job→ToolHelp 恢复，约束前窗口消除）、约束失败受控拒绝（link.containment，删除静默
  降级/detach）、清理错误全量汇总（link.cleanup：原失败+提交状态+残留路径+OS 错误）；
  原型 54 项 + 60s 证据测试；独立代理复审 pass（AC-21..24）；windows-sys 增 ToolHelp feature
- 备注: 三条 P3 随复审记录留档（R1 报告表述已更正、R2 link.restore 并报清理失败已修复、
  R3 计数口径已注明）；本计划四次交付（r1 闭环、r2 质量修复、r3 边界修复、r4 资源边界
  修复）全部完成

### Phase 4 独立复审（2026-10-05，/auto-plan:review，独立代理）

- stage: review | plan_id: PLAN-741 | plan_revision: 4 | outcome: **pass**
- reviewed_commit: `fa4eb10b8`（worktree clean；实现 cc491052f + 报告 a0dd0a315 + 簿记，区间 3 提交）；
  复审后追加 P741P4-R2 顺手修复（link.restore 并报 cleanup_failures）与 R1 表述更正
  （d43bfcd33/37b78f19d，scoped 复验 lib 11/11 + fmt + all-targets 零 warning）
- base_commit: `c865adf66`（v0.6-dev 含复审方 r4 再激活提交）
- 复审方式: 全新上下文独立代理（未参与实现），全部证据自行运行/读源码；自构 death-watch
  反例（复刻外层 UILIMIT_HANDLES Job + run_exe，5/5 零泄漏、孙进程返回后 0-20ms 内死亡）
- dependency_revisions: windows-sys 0.59.0 增 Win32_System_Diagnostics_ToolHelp（受控恢复的
  线程枚举）；其余同 r3
- spec_inputs: SD-07 → docs/specs/auto-ac/project.md（modify，merge 沉淀）；SD-08 → 同（modify，
  merge 沉淀）；复审确认与冻结文本/实现一致、无缩水；canonical specs 零 diff
- acceptance_results（全 **pass**，复审代理自行取证）:
  - AC-21 pass：受控启动（挂起→入 Job→ToolHelp 恢复）+ 约束失败受控拒绝 + 收集截止确定性；
    death-watch 反例 5/5 零泄漏；public-60s 60.05s
  - AC-22 pass：locked-cleanup 全要素（原失败+残留路径+os error 32+NOT committed+全保留标志）；
    库内字节级断言同结论；COMMITTED 段单列状态
  - AC-23 pass（工作侧）：active/executing/r4、指针一致、T-08 补勾与历史收据一致、
    live ledger 留 merge
  - AC-24 pass：全族+fmt+all-targets 零 warning+tv 162/162+一键原型段 13/13；reds 双侧
    worktree 34 红 ⊂ 基线 36 红（comm 实证零独有），抽查基线独有 2 红主检出单跑 PASS
    （THR-D4 flake 归因成立）；生产面零 diff；KNOWN-DEBT R3 三行未预销账
- findings（3 条 P3，不阻塞；R1/R2 已处置）:
  - P741P4-R1：执行侧报告称 process-0..2 为"冻结 r3 副本"系错误——reproduce.ps1 每次从当前
    link.rs 现生成 helper，alive=true 是返回瞬间存活快照的测量伪象（death-watch 实证 0-20ms
    真死）——报告与计划表述已更正（d43bfcd33/37b78f19d）
  - P741P4-R2：link.restore 复合故障未并报 cleanup_failures——已修复（并报残留路径）
  - P741P4-R3：current_step=29 为"最后完成步游标"口径，与 AC-23"完成 ID 数"口径在 merge 全勾
    后收敛 30/30——merge 收据注明口径
  - informational：link_object_staged 两处 let _ = remove_file 为 r3 既有、近不可达，留档
- next: merge（/auto-plan:merge 沉淀 SD-07/08、销账 R3-QA-01..03 行、归档并 guard 清理）

# Phase 3 再激活 / 合同修订交接（2026-10-05）

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

### r3合入后独立复审与Phase4修订（2026-10-05）

- stage: review | plan_id: PLAN-741 | plan_revision: 3 | outcome: **needs_fix**
- reviewed_commit: f6a945db849f57789a9e151402e22b8aa57eb7f8 | base_commit: ed2d00b9076e68fdc41ca59dfa096c753cee5f9e
- dependency_revisions: rustc1.98.1、Cranelift0.126.2、windows-sys0.59.0、auto-down fba6563e。
- spec_inputs/frozen delta/source fingerprints/evidence：docs/reports/741-r3-quality-review-20261005/。
- acceptance_results: AC-01..06/09..13/15/16 pass（限定见报告）；AC-17/18 fail；AC-07/08/14/19/20 partial。
- findings: P741-R3-QA-01/P2进程回收启动/降级缺口；QA-02/P2清理错误吞掉；QA-03/P3进度计数。
- evidence: 原型52+1绿、one-shot13/13、root check/tv162绿；root daily全跑4935绿/35失败/1超时（超时scoped绿），未把daily总门禁标绿；正式60s与锁反例见报告。
- next: 同计划追加Phase4，由work修复；旧r3 pass限定为历史，不覆盖本轮反例。未改实现/Spec/live ledger。

- stage: new | plan_id: PLAN-741 | plan_revision: 4 | outcome: pass（合同就绪，非实现通过）
- authorization: 继承用户“仍有问题则激活741/新Phase修复方案”；没有新增平台/语法或仓库范围。
- changed_contract: T-20..24验收重开，T-08历史据实补勾，T-25..30、AC-21..24、SD-07/08新增；executing/r4/19-of-30。
- next: /auto-plan:work，在专用plan-741-dev实施，不复用旧pass，不预销账。最终merge承担live ledger及归档指针刷新。
- review_cleanup: 本次detached auto-lang及只读auto-down已在精确路径/等价ReparsePoint guard clean、双方git clean后移除；没有创建实施分支，下一work从本合同重建专用检出。

### r4合入后独立复审 needs_fix（2026-10-05）

stage: review | plan_id: PLAN-741 | plan_revision:4 | outcome:needs_fix | reviewed_commit:7b9c6948c1465e87a6f60404510c30c712e1c7bc | base_commit:c865adf66a75099df07113b053eb9d19e92986a3
依赖windows-sys0.59/Cranelift0.126.2，源/Spec哈希、24项验收/SD07–08复核、新反例见[独立报告](../reports/741-r4-quality-review-20261005/REVIEW.md)。54原型/一键13步/公开60s后代回收绿；旧30/30与ledger真实绿。新QA01/P2两链接出口清理诊断不足；QA02/P3归档4个历史报告链接断。
继承用户授权same-ID追加r5：stage:new | plan_revision:5 | outcome:pass（仅修复合同就绪） | next:work。current22/total35；本轮不改Rust/canonical/ledger；SD09提案冻结，P741-3..6旧archive指针激活后暂缺由merge恢复，不假称已刷新。


### r5合入后独立复审与Phase6合同（2026-10-06）

- stage: review | plan_id: PLAN-741 | plan_revision:5 | outcome: **needs_fix** | reviewed_commit:6baed9bba84016dd8610221a9a0358195444385e | base_commit:39a4f9133 | dependency_revisions:auto-down=fba6563ed2148ce85e68208863159b4ccccac710
- spec_inputs与冻结SD09/hash、全部AC1..27映射、原型58+1ignored/一键13/当前库nonzero与正式60s/root门禁原始结果见[报告](../reports/741-r5-quality-review-20261006/REVIEW.md)。AC25/pass，AC26/fail，AC27/partial；旧核心P2已修复，本轮QA01..03均P3生命周期/证据，不归因新的Rust缺陷。
- stage: new | plan_revision:6 | outcome: pass（仅小型修复合同就绪，非实施通过） | next: work。用户此前明确同ID追加phase授权；current26/total38。空spec影响冻结，canonical/live ledger本轮未改；激活暂缺P741-3..7 archive指针由T38最终归档恢复，不假称已同步。

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
