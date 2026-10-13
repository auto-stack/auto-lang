---
plan_id: PLAN-741
status: drafting
feature_name: AC 首个闭环：独立 Atom HIR、语义校验与 Windows 原生 AOT
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-04
plan_revision: 1
current_step: 0
total_steps: 8
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

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-hir/project.md | 无实现规范→有限文本/TypeUse/校验/顺序契约及限制 | 沉淀实证能力，不宣称完整HIR | AC-01,AC-02,AC-03,AC-04 |
| SD-02 | add | docs/specs/auto-ac/project.md | 无native实现规范→Checked HIR AOT/Windows工具链/入口/trap/测试能力 | 区分原型、生产ABI与源码编译 | AC-05,AC-06,AC-07,AC-08 |

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

## 8. 执行步骤

所有步骤尚未执行；完成记录commit、命令、结果与AC。

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

## 9. 复审记录

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
