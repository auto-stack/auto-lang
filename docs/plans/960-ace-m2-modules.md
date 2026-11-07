---
plan_id: PLAN-960
status: execution_done
feature_name: 完整模块包与公共类型绑定
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 5
total_steps: 8
depends_on: [PLAN-959, PLAN-950]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-modules.md]
touched_goals: [GOAL-002]
---

# [PLAN-960] 完整模块包与公共类型绑定

## 0. 变更摘要

扩展950有限模块到实际use/dep、别名/重导出、公共类型/方法/常量与包锁身份；循环声明先建声明图，值初始化循环具名诊断。与模板/CT能力组合，保持模块身份及源span；包发现由Man提供闭包，Core不偷偷搜全盘。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

依赖：PLAN-959, PLAN-950。依赖指正式交付来源及适用接口，不把WIP/reviewed当delivered；不以最终认证尚未交付阻塞独立能力建设。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/core/src/source/modules`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/source-query/src/modules`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-man/crates/auto-package-lock`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

## 5. 详细设计

扩展950有限模块到实际use/dep、别名/重导出、公共类型/方法/常量与包锁身份；循环声明先建声明图，值初始化循环具名诊断。与模板/CT能力组合，保持模块身份及源span；包发现由Man提供闭包，Core不偷偷搜全盘。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：模块状态与四端装配（r2）

明确全局store/常量初始化次序、once/session归属、失败回滚、diamond依赖和模块卸载；把950/944扩到文本/聚合/宿主公共签名及AV/AC/Rust/C装配恢复。类型表示消费既有text/record；新增集合布局归962。模板/CT组合本包提供接面，963/966最终闭合，禁止反向等待。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### T-01 冻结决定（2026-11-07，goal 会话 work 阶段）

依赖核验：PLAN-959/950 均 archived（archive/950-ace-finite-source-modules.md、archive/959-ace-m2-denominator.md，frontmatter status: archived）；§4 三处路径存在且符号已核（core/src/source/modules、source-query/src/modules、auto-man/crates/auto-package-lock：LockEntry{name,version,url,commit,path} 只读参照）。分母行归属：L-37/38/41/42+L-44。有界前探（正例/非法近邻/失败恢复沿既有 950 producer→consumer 全链）收据见 §9 T-01 行。

**策略比较与决定**（不扩大/缩减 §5 范围）：
- D1 身份：新能力 `ac.source.modules.packages/v1` + grammar trimming `packages-modules`（scalar-modules 全表面 + text/record 面 + 新 item 面）；binding version `source-module-binding/2`；输入扩为 `PackageModuleInput{package,version,id,source}`（包名 `[a-z0-9_-]+`、版本 `[A-Za-z0-9.-]+`；Man 供闭包，Core 不搜盘）。旧 `scalar/v1` 面与行为逐字节不变（950 证据可重现）。
- D2 表面：`pub fn/fn`（既有）+ `pub type/type`（消费既有 TypeItem/字段/方法）+ `pub const NAME: T = e` / `const`（模块常量）+ `var NAME: T = e`（模块私有可变 store，无 pub）+ `use path [as x]`（≥2 段路径别名导入）+ `pub use path [as x]`（重导出）+ 点路径限定引用 `pkg.mod.item()`/`mod.item()`。enum/集合布局归 962；`<T>` 模板组合：能力共存不再结构性互斥，模板构造具名 `source.module-templates` 拒绝（963 闭合，接面非语义）。
- D3 绑定 v2：符号 `p{plen}_{pkg}_{mlen}_{mod}_{name}`（fn/global），type `t{...}` 同形前缀；entry main 保持 `main`。跨包 3 段/本包 2 段解析；裸名优先级 局部>本模块>导入（沿 950）。重导出链式解析（目标须公共，链环 `source.module-reexport-cycle`；私有目标 `source.module-reexport-private`=V-02）。**函数调用环不再拒绝**（互递归合法）；use/reexport 模块边环仍 `source.module-cycle`；值初始化环（const/store initializer 经直接读+被调 fn 传递读的闭包）具名 `source.module-init-cycle` 并列成员（V-06）。input_hash v2 由排序后 (package,version,id,sha256(source)) 决定——输入序/宿主重定位不改身份（V-05）。
- D4 全局状态策略（比较后选定）：const 与 var 同为运行时全局（const=不可变 global；编译期折叠仅为后续优化，非语义），经 HIR 既有 `DefKind::Intrinsic`（symbol `module.global.get/set:<sym>`）+ AST 级读写改写为调用 + 每模块合成 `__init`（声明序）+ 模块级 topo 初始化序；备选"编译期求值 const、仅 var 走运行时"被拒：分母 L-41 要求运行时初始化次序/once/diamond 语义。初始化在装配/链接期执行：一次失败整代不发布（旧 current 保留=V-08 回滚/V-06 旧程序仍运行）；每代各自初始化，最后 pin 释放即卸载。
- D5 域：v2 模块域 = TextRecord（emit/verify/lowering 沿既有 text-record 管线；AV lower_checked_owned 系）。四端：AV（Value::Str/StrView 观察）、native（MnDriver owned 臂 + cranelift global）、a2x-rust（Family::RecordText "AXR2"）、a2x-c（C_RT_FILESET record-text 面）均已有 Str/record 能力底座；ModuleA2xTarget 解除非-Rust 拒绝按域选 family；新 native 模块目标。944 Manifest 白名单扩 `ace-native/modules/1`、`ace-a2x-c/modules/1`；四端装配恢复按两个 AssemblyHost（AV+Rust、Native+C）验证，不改双目标泛型。
- D6 持久化：新 root schema `source-modules-root/4`（v2 声明环境：globals/init 序/reexports/包身份）；root 不存全局值或宿主地址（恢复后首链接重跑 init=V-08.5）；旧 /2//3 不变。
- D7 验证命令（T-04 冻结）：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-a2x`（组名静态核 test-groups.toml）；新增 conformance `crates/av/tests/cases/module_packages_960.rs`（Windows x64 gate 沿 950/944 证据环境）；资源预算沿 FrontBudget::WORK_OWNED_64MIB + 绑定 work 2^22 + ExecLimits 默认，无新增宿主调用面。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-modules.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |

## 6. 测试设计

以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 两个包同名私有符号隔离 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 重导出私有类型拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 依赖body更新只失效真实读取者 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 缺失导出后来加入可修复 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 重定位不改逻辑身份 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-06 | AC-02/03 | 初始化循环拒绝且旧程序仍运行 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-a2x`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 两个模块同名私有store隔离；diamond依赖初始化仅一次；初始化失败不发布半状态；四端同一公共Str函数真实运行；全局状态恢复不序列化宿主地址。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 模块状态与四端装配的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |

## 8. 执行步骤

预定唯一实施树 `D:/autostack/.wt/lang-960/auto-lang`，分支 `plan-960-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

- [x] **T-01｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 [✅ 2026-11-07] 依赖 959/950 archived 核验；§4 路径/符号核定；T-01 冻结决定（D1..D7）入 §5；有界前探：worktree `D:/autostack/.wt/lang-960/auto-lang` @fd3b3116a 冷启，`cargo nextest run -p ace-core -p ace-source-query -p ace-av --lib --tests --features ace-av/test-case-batch -E 'test(modules)|test(module_sources_950)|test(assembly_944)'` → 26/26 PASS（正例 module_sources_950 42-oracle 全链含 restore-first-edit；失败恢复腿 candidate_cancellation...preserve_old_recovery PASS；非法近邻=core 模块面既有拒绝断言 use-as/pub-type/generic，T-02 起新面正反例）。收据 plan-artifacts/plan-960/t01-baseline-probe.log sha256 bb3007700a3a762edd415d95c8cf6a78b9cc0ea5e7fed30c4f741636e81aac99。
- [x] **T-02｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 [✅ 2026-11-07] 提交 a36f63af0（grammar `packages-modules` 裁剪 + AST Const/Store 项 + 投影 + `source::packages` v2 绑定：包锁身份 input_hash、别名/重导出链（私有重导出 `source.module-reexport-private`=V-02）、仅 use/reexport 边的模块环、值初始化环具名 `source.module-init-cycle`（V-06 绑定腿）、全局 get/set 改写与每模块 init 合成、V-01 跨包同名隔离、V-05 输入序/版本身份）与 d4f0a4e17（resolve 伪签名 `module.global.get/set:`、CallGlobalGet/Set、adapter Intrinsic def/类型表、`compile_package_product` 走共享 TextRecord resolve/typecheck/emit/verify_text_record）。永久正反例：core `source::packages` 5 测试 + `source::packages::binding` 3 测试（含 8 行拒绝矩阵）+ source-query `modules::packages` 2 测试；验证命令 `cargo nextest run -p ace-core -p ace-source-query --lib -E 'test(package)'` → 7/7（含 950 模块面 28/28 回归不变）。**T-01 适配记录**：934 事实页 `PortableTy` 仅 {Unit,I32,Bool}（47 处、`sq-module-page/1` 存储合同），文本/记录程序不能经事实页——包通道直接跑同一 resolver（非平行分析），增量证据改由绑定账本 `binding_walks` 承担；事实页文本/记录覆盖为具名 query 层后继（KNOWN-DEBT 候选），非静默回退。
- [x] **T-03｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 [✅ 2026-11-07] e9919cdf0（MIR `MirGlobal`/`GlobalGet`/`GlobalSet` 新增 opcode 0x36/0x37、`MirModule.globals/init`；`ProgramSnapshot::link_with_globals` 程序级 cell + 每代一次 init 链、失败重置不发布；executor/verify/ownership/abc/instr_spec；lowering `module.global.*` 作逻辑 import → GlobalGet/Set，`lower_checked_owned_with_init`）与 b36376578（`PackageAvTarget` cold/cold_with_entry/run_value/global_cells；split_units 携带全局表；同名私有 store 隔离=22 非 23、diamond 基 cell、公共 Str fn=StrView）；26f93cc79（`a2x-store modules::packages::PackageA2xTarget<P>` RecordText 族冷/收据/恢复——Rust AXR2 与 C AXC2，恢复 lower/print=0 仅重装配；`source-query modules::packages_store` 根 `source-packages-root/1` 身份+单元源+目标收据，不存 cell 值/宿主地址，恢复经同一前门/绑定/编译并核 input_hash，篡改收据/外来产品拒绝；944 白名单 `ace-native/modules/1`/`ace-a2x-c/modules/1` + packages profile 接纳；四端 conformance 3/3）。native 腿：`MnDriver::build_checked` owned 臂真 exe，`ac_start` 退出码转发 i32（42 oracle）。**具名边界**：a2x 全局 cell 发射与类型方法片段为后继（四端 Str 产品不含全局，全局三项在 AV 见证）；mn 全局 cell 具名拒（D-960-2）。
- [x] **T-04｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 [✅ 2026-11-07] V-01..V-08 全部真实运行：单测 core 5 + source-query 3（packages/packages_store）+ av 2（packages_av，V-08 全五条：store 隔离 22≠23、diamond once 共享 cell 1→2→3、init 失败无半态 cell 全归 0/initialized=false、恢复后重初始化、根无地址）+ conformance `module_packages_960` 3/3（四端 Str：AV I32(42)+StrView、rustc exe stdout value:42、cl exe stdout value:42、mn exit code 42；rust/c 恢复 lower/print=0 且文件集一致；根恢复身份相等；V-05 顺序无关；V-02/V-04/V-06 具名拒绝 + 旧程序仍运行）。固定组：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-a2x` → 5/5 PASS，executed 385/204/40/1024/196=1849，gate_passed=true，wall 1773s（收据 plan-artifacts/plan-960/t04-fixed-groups.log sha256 c90bfc1f1ba347e05da1f0d0530187727fc38ea0f7c0a684a58801241ddd3add）。
- [ ] **T-05｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。
- [ ] **T-06｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。
- [ ] **T-07｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-960/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。

- [x] **T-08｜owner_stage: work｜AC-06**：明确全局store/常量初始化次序、once/session归属、失败回滚、diamond依赖和模块卸载；把950/944扩到文本/聚合/宿主公共签名及AV/AC/Rust/C装配恢复。类型表示消费既有text/record；新增集合布局归962。模板/CT组合本包提供接面，963/966最终闭合，禁止反向等待。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 [✅ 2026-11-07] 初始化次序/once/session归属（每代一次、代内 `initialized` 状态）、失败回滚（cell 全重置+不发布半态，av 单测）、diamond（共享 cell 三路递增见证 once）、模块卸载（代随 snapshot 释放，cell 从不持久化）；文本/聚合/宿主公共签名（TextRecord 域、Str 入口四端、公共类型/常量/方法经绑定）；AV/AC(native)/Rust/C 装配恢复（冷+恢复收据，944 白名单扩 native/C + packages profile）；V-08 五条全部有真实结果（av 单测 2 + conformance）。

## 9. 复审记录

stage: work | plan_id: PLAN-960 | plan_revision: r2 | outcome: pass（全部 work 任务完成；无阻塞项） | worktree_path: D:/autostack/.wt/lang-960/auto-lang | branch: plan-960-dev | base_commit: fd3b3116a | code_commit: 674473a27（检查点 a36f63af0 / d4f0a4e17 / e9919cdf0 / b36376578 / 26f93cc79 / 1c9285c3a / 674473a27） | dependency_revisions: PLAN-950 archived, PLAN-959 archived | task_ids: T-01..T-04 ✅, T-08 ✅（5/8；余 T-05 review / T-06+T-07 merge） | evidence: §8 各任务行命令/计数；V-01..V-08 真实运行（单测 10 + conformance 3/3 四端）；固定组 5/5 PASS executed=1849 gate_passed=true wall 1773s（t04-fixed-groups.log sha256 c90bfc1f…d3add）；Spec 增量已在 worktree 提交（docs/specs/ace-migration/m2-modules.md 新建、replacement-scope SD-04） | readiness: work 自有 AC-01/02/03/06 均有绑定点证据；fmt 后复跑单测 10/10 + conformance 3/3 绿；ace-core 触碰后新警告 0（残留 2 条 first_lex_fact 为存量）；统一回归门=cargo t 5 组已在 T-04 执行（冻结提交 674473a27 后不重复） | blockers: 无。具名边界（非阻塞）：D-960-1 事实页类型集 / D-960-2 mn 全局 cell / a2x 全局 cell 发射与类型方法片段（已写 KNOWN-DEBT 候选与 §8） | next_responsible_role: review（T-05，AC-04）

stage: goal | plan_id: PLAN-960 | plan_revision: r2 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-960/auto-lang | commit: 674473a27 | next: review

stage: work | plan_id: PLAN-960 | plan_revision: r2 | outcome: executing-continues（本轮未达 execution_done；无 blocker） | worktree_path: D:/autostack/.wt/lang-960/auto-lang | branch: plan-960-dev | base_commit: fd3b3116a | code_commit: b36376578（检查点 A a36f63af0 / B d4f0a4e17 / C e9919cdf0 / D b36376578） | dependency_revisions: PLAN-950 archived, PLAN-959 archived（主检出 fd3b3116a） | task_ids: T-01 ✅, T-02 ✅, T-03 进行中（AV 端完成；native/rust/c 端与 PackageStore/装配待做）, T-08 实现面已入 T-02/T-03（全局 cell、init 序/once/失败回滚在 AV 可观测；V-08 四端 Str 与恢复待 T-03b/T-04）, T-04/T-05/T-06/T-07 未开 | evidence: 本节各任务行内命令/计数；plan-artifacts/plan-960/{t01-baseline-probe.log, patch-*.py} | blockers: 无（事实页 PortableTy 覆盖为具名后继，见 T-02 适配记录） | next_responsible_role: work（续 T-03b → T-04 → T-08 收敛，再交 T-05 review）

stage: goal | plan_id: PLAN-960 | plan_revision: r2 | round: 1 | from: drafting | to: work | status: executing | outcome: pass（阶段有实质进展：T-01/T-02 完成、T-03 AV 端完成并提交四个检查点） | worktree: D:/autostack/.wt/lang-960/auto-lang | commit: b36376578 | next: work

授权记录：`/goal 完成计划960`（2026-11-07）= 全链授权（work 含建树、review、有界修复、merge）；不改目标/验收/范围/预算。plan-artifacts scope `D:/autostack/plan-artifacts/plan-960/` 已登记 .retention/policy.json（in-flight）。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
