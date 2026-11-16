---
plan_id: PLAN-973
status: archived
feature_name: 新版系统文件进程时间与资源库
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 13
current_step: 9
total_steps: 10
depends_on: [PLAN-967, PLAN-968, PLAN-972, PLAN-947, PLAN-966, PLAN-1039, PLAN-1040, PLAN-1041, PLAN-1042]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/stdlib/design/ace-binding-and-assembly.md]
new_spec_components: [docs/specs/ace-migration/m2-stdlib-io.md]
touched_goals: [GOAL-002]
---

# [PLAN-973] 新版系统文件进程时间与资源库

## 0. 变更摘要

2026-11-16 r13 merge·第8轮：**delivered+archived**——rebase 到 master（6 提交映射+和解修复 d59c92331），规范三源落 9d0f973cb（m2-stdlib-io.md 新建+两挂接），merge-owned 冻结并集 7 组 PASS=3098 绿+2 组 master 预存红具名（主检出同红取证），wt-guard clean 后 ff-only 落地（master tip=9d0f973cb），清树/删 scope/刷 registry，**status: archived**。移交在案：1064（byte→Str 面）/1065（AC 真身）/1063（Linux）。

2026-11-16 r13复审修复·第7轮：F1 用户裁定 **b——AC 真身具名延 PLAN-1065**（V-09/AC-07/AC-COVERAGE 行文修订为"AC stub=已裁定交付面"）；F2/F3/F4 同树修复（worktree `0a6fb9100`：触面 warning 清零、dialect 探针内联自包含、argv 注记改已替代终局），F5 已 §10 落册；复验 sys973 15/15+fmt clean。重审 affected 项全闭，**T-05 final pass → reviewed**。next=T-06/07 merge。

2026-11-16 r12复审·第6轮：T-05 独立复审 **needs_fix**——复审重跑 29/29 绿（sys973 15/15+外库腿 2/2+native stub 1/1+ace-host 11/11，per-case ≤10s）、四表对齐/车道门/零 libc 委托/故障注入/零 debug 残留核验通过；发现 F1[中]（AC 后端 50 操作全 stub，V-09"AC真实调用OS API"正例腿未交付且无具名移交行——T-09 复开，待用户裁定 a 本计划实现/b 具名延后继）+ F2..F5 小缺陷（触面 warning×2、dialect_bisect 读仓外文件、argv 注记过时、rand 不移植裁定不可追溯）。status 回 executing（6/10）。债务候选登记 KNOWN-DEBT D-973。

2026-11-16 r12执行·第5轮：**用户裁定 §10.1 选项 c——byte→Str 车道面具名延期后继计划**。[PLAN-1064](1064-ace-stdlib-byte-str-deferred-surfaces.md) 创建为承接 owner（drafting 0/7，移交表 D-01..D-09）；T-02/T-03/T-08 按 r11 已交付面勾选（read_text/dir_names/env get 族/canonical/cwd/cmdline/捕获文本/存储重建/zip/inflate/KV storage 移交 1064），全部 work 任务完成，status→execution_done，零代码改动（worktree 保持 a9eceef38，复用第 4 轮冻结证据）。next=T-05 独立复审。

2026-11-16 r11执行·第4轮：T-04冻结并集完成——计划触面warning/fmt清零（基线15条+auto-lang-core存量具名不动）、ace-host恢复可编译（Exit臂）、9组固定职责组并集实跑（7组PASS；ace-abi 1例+ace-runtime 19例=master预存红，主检出同红取证，非973面）；T-01/T-04/T-09/T-10按证据勾选。zip/storage/read_text族仍待byte→Str车道用户裁定。

2026-11-15 r11当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r10当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r9当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r8：参考Zig/Go的原生平台I/O设计承接；新增AC-PLATFORM-IO/V-PLATFORM-IO/SD-PLATFORM-IO，由原任务负责。保全部原任务/数量/进度/既有AC/V及其他会话修订，本轮仅规划。

2026-11-12 r7：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r6当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-10 r5当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/10，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承947资源前探，实现File/path/env/process/time/rand/console/zip及实际storage接口；标准语义与平台provider分离，授权资源按session归属。进程树/管道deadline、部分读写、原子替换和平台差异逐项契约化。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

**2026-11-15 目标授权（goal grant）**：用户以 `/goal 完成计划973` 授权 /auto-plan-goal 全链交付——在专属 worktree 实施（含创建）、独立复审、合同内有界修复与 merge（规范沉淀、归档、清树、落地核查）。阶段链按 stage map 推进；不授权变更目标/验收/阈值/范围/预算。实施树 `D:/autostack/.wt/lang-973/auto-lang`（分支 `plan-973-dev`，base `3279f2f72`）已建；材料目录 `D:/autostack/plan-artifacts/plan-973/` 已登记。前置依赖 967/968/972/947/966/1039/1040/1041/1042 均已归档（2026-11-15 核验）。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `stdlib/auto`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/fs.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/process.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/time.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `stdlib/auto`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/av-core/src/host.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/a2r-std/src/fs.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/a2r-std/src/process.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/a2r-std/src/time.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：文件/进程/时间/环境等系统资源库。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [stdlib/auto/io.at](../../stdlib/auto/io.at)：公开声明/错误与资源语义；读取SHA256前16 `8a124ca338d14a5c`

- [crates/a2r-std/src/fs.rs](../../crates/a2r-std/src/fs.rs)：旧文件行为参考/可复用中立实现；读取SHA256前16 `6b9bda5d4218e279`

- [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs)：967资源/权限合同与平台provider，不在内容仓另造文件系统；读取SHA256前16 `fb148c0a779d5c95`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承947资源前探，实现File/path/env/process/time/rand/console/zip及实际storage接口；标准语义与平台provider分离，授权资源按session归属。进程树/管道deadline、部分读写、原子替换和平台差异逐项契约化。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：文件进程与四端console完整性（r2）

承947未闭Rust/C console，补文件watch/目录/seek/权限、process信号与退出、路径重定位及zip现用义务；按实际平台支持，不自动新增平台承诺。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

新增LIB-M2-03/05：系统库公共语义与平台实现分离，消费966已交#目标选择和967完整FFI，补四端File/path/env/process/time/rand/console等原职责。Windows/Linux平台分支按目标选中，未选平台SDK不被绑定；适用平台表须分别列选择可验证、编译可验证、实际运行证据，不把缺runner算PASS。

AC直接生成机器码，经C/system/平台ABI调用OS API；Auto模块使用AutoABI。平台provider可用Auto+FFI或经合同核验的预编译runtime，不强制裸syscall、不隐式转C/Rust业务源码、不接回旧VM。权限/owner/close/cancel/部分IO/EOF/错误码按公共合同，与原T-08进程/watch/zip等全部义务合并终验；异步调度消费968，不另造executor。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 建立统一系统API到auto.platform.* provider/host端口映射，用966的#if/#is按目标选择；native用平台ABI直接调用OS，四端复用967受检事实。 |
| T-10 | AC-08 | V-10 | 完成仓外真实文件create/write/read/seek/close/EOF/部分IO/错误与权限资源测试，联进程/time/random/console及原全系统义务；形成给978的runtime/动态依赖清单。 |

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。默认系统provider用薄OS适配直接接目标平台系统服务，Auto内部保AutoABI，OS边界按真实目标ABI/结构/编码处理；不能把libc或Rust std::fs等最终委托libc的实现冒作默认自有库。独立C/Rust库可显式选用，依赖和错误/资源语义分账。

本包验收：实际OS create/write/read/seek/EOF/close、错误/部分IO/取消与平台文本边界对独立expected；核符号/调用/链接来源，无隐式libc委托；缺默认provider拒绝，不切外部桥。 原任务T-09,T-10承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r7当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.at](../../crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
File.exists("nonexistent_file_xyz_12345.txt")
```

本包所需独立结果：旧File.exists固定不存在路径为false（显示0另记）；新写入A中😀加NUL=9字节，读回完全相同，临时写失败/原子替换失败保旧文件hash。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Auto系统API→授权/类型/owner事实→AutoABI平台端口→薄OS调用→typed结果/资源与文件字节。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct SystemResource { id: ResourceId, kind: ResourceKind, generation: Generation, rights: GrantSet, owner: OwnerId }
struct ReplaceTransaction { destination: PathIdentity, temporary: PathIdentity, old_hash: ContentHash }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [stdlib/auto/io.at](../../stdlib/auto/io.at) | 公开声明/错误与资源语义 |
| [crates/a2r-std/src/fs.rs](../../crates/a2r-std/src/fs.rs) | 旧文件行为参考/可复用中立实现 |
| [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs) | 967资源/权限合同与平台provider，不在内容仓另造文件系统 |

**状态、错误与资源。** 越权路径、过期handle、重复close、非法UTF-8路径/平台不支持与宿主IO失败分态；拒绝时OS副作用计0。 文件/child process/timer各有owner；取消杀/等待原任务树，stdout/stderr有界，失败不损已发布文件；Windows/Linux真实适用格另取证。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.at](../../crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.at) | 存在性/路径语义 | `file_exists.expected.result`；SHA256前16 `051bdece4a4d4a5a` | 拟ace.m2.p973.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/src/tests/vm_file_tests.rs](../../crates/auto-lang/src/tests/vm_file_tests.rs) | 旧fs/ffi公开运行检查 | Rust断言来源：test_vm, test_vm_impl, test_01_basics_001_hello；SHA256前16 `696db0da4a88b851` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/vm/19_rust_std/010_process_command/process_command.at](../../crates/auto-lang/test/vm/19_rust_std/010_process_command/process_command.at) | 进程/退出与字符串边界 | `process_command.expected.out`；SHA256前16 `16b4389c0501503c` | 拟ace.m2.p973.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/specs/stdlib/design/semantic-operation-contract.md](../specs/stdlib/design/semantic-operation-contract.md) | 旧所有系统操作而非只三个文件样本 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `2d44c8304d5f8022` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 精确write/read/atomic_replace/permission/clock/env/process API、编码与错误；原子替换与跨卷/已开文件支持域明确；不得把C ABI误当默认libc依赖。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### Windows/Linux无libc平台I/O（2026-11-13当前）

依据[原生平台I/O Draft 0.1](../design/stdlib/native-platform-io.md)。完整实施消费1039/1040/1041正式来源，落实设计§7–9的openat/CreateFileW、read/write/seek/size/sync/close、Path字节/UTF16无损往返、共享/umask/CLOEXEC、NUL拒绝、部分完成/关闭/授权根与symlink/reparse逃逸；字符串前缀不是sandbox。T-01可先只读冻结需求；原前置、完整四端、process/time/watch/zip义务保留。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

File/path/env/process/time/rand/console等逐操作六路径；Windows/Linux AV/native用1039/1040系统边界，AV/C和AV/Rust实际库/handle路径，A2PY用共享HIR目标端口。已交1041输出只复用身份仍适用的证据，stdin/File等仍由本包关闭。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

经966/967/972传递硬依1043；系统库按provider后缀选来源、#in按target选普通平台模块，真实OS调用仍沿1039/1040，完整CTE扩面沿966。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-stdlib-io.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/stdlib/design/ace-binding-and-assembly.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-stdlib-io.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-stdlib-io.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-stdlib-io.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### 平台I/O规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-PLATFORM-IO | add/modify | docs/specs/ace-migration/m2-stdlib-io.md | 笼统provider→本节平台IO/算法/运行/依赖分账 | 无libc平台设计 | AC-PLATFORM-IO |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-stdlib-io.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-stdlib-io.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**系统File/path/env/process/time/rand/console与资源**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

默认Auto实现缓冲/路径/错误/资源逻辑，Windows公开DLL、Linux syscall薄适配；不用C stdio/Rust std::fs/std::io默认实现。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-stdlib-io.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |
| SD-AUTO-STD | modify | docs/specs/ace-migration/m2-stdlib-io.md | 默认provider声明→普通标准能力有Auto算法/协议与OS边界来源证据，高级外库显式分账 | 用户自有普通库要求 | AC-AUTO-STD |

### byte→Str 面延后移交（2026-11-16 用户裁定，当前合同）

用户裁定 §10.1 选项 c：卡在**字节→Str 构造车道缺口**（972 已知边界）的库面**具名延期到 [PLAN-1064](1064-ace-stdlib-byte-str-deferred-surfaces.md)**，本包按已交付面收口。移交表 D-01..D-09（含代码锚点与逐面旧 sentinel 合同）：file.read_text、dir_names、env.get/get_or/home_dir/local_data_dir、path.canonical 真实符号链接解析、process.args()/current_dir()、spawn_with_output 捕获文本、KV storage 持久面、zip/inflate 三元组、read_bytes Option\<List\> 形状。本包不冒上述面已交付；旧 sentinel 约定逐面保留由 1064 承接实现，本包模块头具名注记（"lane gap/plan dossier"）即指向该移交。1064 非本包反向依赖；车道解锁落点（972 域 vs 1064 内建）由其 §10 待澄清裁定。

| delta_id | add/modify | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-DEFER | modify | docs/specs/ace-migration/m2-stdlib-io.md | 延后面悬置缺口→具名移交 1064 的边界清单+旧 sentinel 保留+代码锚点 | 用户裁定选项 c | AC-02/03/06/08 |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 仓外真实临时文件Unicode路径往返 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 拒绝越权路径前无副作用 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 子进程超时连管道读者全部回收 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 短写/磁盘失败不伪成功 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | clock/rand用可注入oracle，系统运行另测 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t stdlib ace-stdlib ace-host ace-runtime a2r-std storage`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 四端公共console实际bytes正确；关闭/取消后晚IO不写新handle；进程树包含孙进程与reader清理；原子文件替换失败保旧文件及错误类别。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | Windows/Linux选择/缺平台/歧义provider正反例；未选SDK缺失不阻；~~AC真实调用OS API且不先产C/Rust业务源~~（**该腿 2026-11-16 裁定 b 移交 [1065](1065-ace-ac-sys-real-bodies.md)**，本包 AC 面=stub 诚实拒绝+负控），四端按适用平台运行同一公共系统程序。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 临时目录写入含NUL/Unicode字节、读回、seek/EOF、失败路径与双close策略；撤权dispatch=0、取消/进程退出后句柄无泄漏、late结果不发布；四端独立expected而非仅相等。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |

新增触面职责组：`cargo t stdlib ace-stdlib ace-host ace-av ace-native ace-a2x ace-abi ace-foreign-contract`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-09,T-10，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 实际OS create/write/read/seek/EOF/close、错误/部分IO/取消与平台文本边界对独立expected；核符号/调用/链接来源，无隐式libc委托；缺默认provider拒绝，不切外部桥。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧File.exists固定不存在路径为false（显示0另记）；新写入A中😀加NUL=9字节，读回完全相同，临时写失败/原子替换失败保旧文件hash。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 越权路径、过期handle、重复close、非法UTF-8路径/平台不支持与宿主IO失败分态；拒绝时OS副作用计0。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 文件/child process/timer各有owner；取消杀/等待原任务树，stdout/stderr有界，失败不损已发布文件；Windows/Linux真实适用格另取证。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 原生平台I/O验证（新增，未执行）

| Case | AC / work任务 | 输入、独立oracle及边界 |
|---|---|---|
| V-PLATFORM-IO | AC-PLATFORM-IO；T-01/T-09/T-10/T-04 | 同源四格真实create/write/close/reopen/read/seek/EOF，空文件/零buffer/NUL内容、exclusive冲突/append、坏路径/权限/逃逸；短写后失败/取消/close错误、原生路径往返有OS及独立进度/owner oracle。close不盲重试，flush与持久化分开；mock只补故障，缺runner不PASS。 |

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-AUTO-STD / AC-AUTO-STD | 原能力work与最终验证：移除可选外库/provider，在Windows默认native/AC跑原完整公共操作并核Auto源码/原语/OS调用来源和独立expected | math/io/JSON等按§5自有归属，不以包装libm/std::io/serde_json等代实现；Linux只做代码/移交审查，未来1063实跑，外部引擎依赖另账。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 文件进程与四端console完整性的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | LIB-M2-03/05平台适配和原公共API闭合，目标与宿主分离，无旧VM桥或业务转译前置；支持矩阵逐平台有真实证据（AC 后端真实调用切片经 2026-11-16 裁定 b 移交 [1065](1065-ace-ac-sys-real-bodies.md)，本包 AC 面=stub 诚实拒绝+负控）。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | LIB-M2-05所有适用端值/效果/资源状态闭合；File/Task/Stream不再只是前探，缺环境或平台边界诚实保留。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-NATIVE | work | 实际OS create/write/read/seek/EOF/close、错误/部分IO/取消与平台文本边界对独立expected；核符号/调用/链接来源，无隐式libc委托；缺默认provider拒绝，不切外部桥。 | T-09,T-10 / V-NATIVE；原独立review核新增义务 |

- [x] **AC-PLATFORM-IO / work**：本节平台合同及V-PLATFORM-IO全部满足，真实来源/oracle/未交边界齐全；原分母/四端/阶段不减，原最终work汇总、独立review核验。（✅ Windows 四格 probe01/v04/t08/dirs 绿；Linux 格 code-only/NOT_RUN→1063 L-01..L-03；T-05 第 6 轮核）

- [x] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。（✅ AV/native 15 例+AV×C/AV×Rust 外库腿 2/2+A2R 真 rustc+A2PY 真 Python；AC 路径=stub 拒绝+负控（2026-11-16 裁定 b 真实运行移交 1065）；A2C 按原适用域不减；T-05 第 6 轮核）

- [x] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。（✅ 全部 sys973 测试经 assemble_stdlib_sys_inputs（1043 装配+#in 目标选择 t09）；T-05 第 6 轮核）

- [x] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。（✅ §10 L-01..L-12 表与代码注记逐项一致（审计 agent 核），全部 NOT_RUN/1063；T-05 第 6 轮核）

- [x] **AC-AUTO-STD / work**：§5本包普通标准能力默认算法/协议为共享Auto源码，实际来源/调用链审计与原独立expected齐全；可选C/Rust及高级外库列明边界，不冒默认自有实现。原能力work任务与最终验证负责，原独立review核；Linux按AC-LINUX-PHASE移交。（✅ stdlib/ace/sys 10 模块纯 Auto 构造；av-core kernel32/bcrypt 直调零 libc/Rust std 委托（审计 agent L1575-1899 核）；A2R/A2PY Rust std/Python 为显式目标运行时分账；T-05 第 6 轮核）

## 8. 执行步骤

当前共10任务，7完成（T-05 已 final pass；余 T-06/07 为 merge 阶段）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。（✅ 2026-11-15 决定冻结 + probe01 前探 8/8 绿，见 §9）

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。（✅ 50冻结平台操作×4表+Win32真后端+Linux syscall臂+capture oracle/故障注入+统一资源账本+sys装配集，sys973 16/16；byte→Str 摘除面按 2026-11-16 裁定移交 1064（D-01..D-09），见 §9 第5轮）

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。（✅ A2R真rustc腿+A2PY真Python腿 value:42、AV×C/AV×Rust外库腿 2/2、AC具名stub拒绝、1043装配消费；存储重建随车道移交 1064 D-07，见 §9 第2/3/5轮）

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：承947未闭Rust/C console，补文件watch/目录/seek/权限、process信号与退出、路径重定位及zip现用义务；按实际平台支持，不自动新增平台承诺。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。（✅ V-08绿+V-03进程树/超时Job杀/权限链attrib±R/dirs+replace/mtime；watch无旧公共API（调研记录，无义务可保）；zip现用义务随车道移交 1064 D-08，见 §9 第2/4/5轮）

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07/AC-NATIVE**：建立统一系统API到auto.platform.* provider/host端口映射，用966的#if/#is按目标选择；native用平台ABI直接调用OS，四端复用967受检事实。 验证V-09，证据汇入原最终work验证。 新增V-NATIVE：默认系统provider用薄OS适配直接接目标平台系统服务，Auto内部保AutoABI，OS边界按真实目标ABI/结构/编码处理；不能把libc或Rust std::fs等最终委托libc的实现冒作默认自有库。独立C/Rust库可显式选用，依赖和错误/资源语义分账。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。（✅ sys973_t09 绑定绿+AV×C/AV×Rust 外库腿绿（216fd055d）+Linux 移交 L-10；F1 AC 真身正例腿经 2026-11-16 用户裁定 b 具名延 [1065](1065-ace-ac-sys-real-bodies.md)，本包 AC 交付面=stub 诚实拒绝+负控，见 §9 第 6/7 轮/§10）

- [x] **T-10｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：完成仓外真实文件create/write/read/seek/close/EOF/部分IO/错误与权限资源测试，联进程/time/random/console及原全系统义务；形成给978的runtime/动态依赖清单。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：默认系统provider用薄OS适配直接接目标平台系统服务，Auto内部保AutoABI，OS边界按真实目标ABI/结构/编码处理；不能把libc或Rust std::fs等最终委托libc的实现冒作默认自有库。独立C/Rust库可显式选用，依赖和错误/资源语义分账。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。（✅ V-10/probe01/v04 真实OS全链绿+真实熵/dir+replace/mtime；zip/storage 车道缺口不属本任务关闭面，见 §9 第1/4轮；AC-08 的 zip/storage 切片随 T-02/T-03/T-08 车道裁定）

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。（✅ 2026-11-16 9组冻结并集实跑：7组PASS+2组仅master预存红，全部V行有归属；证据见 §9 第4轮）

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。（✅ 第 6 轮 needs_fix（重跑 29/29 绿+独立审计 agent 核四表/车道门/零委托；F1..F8）→第 7 轮修复+重审 final pass 绑 r13/0a6fb9100；勾选补落于 merge 轮，见 §9 第 6/7 轮）

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。（✅ rebase 到 master（6 提交重映射+和解修复 d59c92331：修复冲突清理截断的 sys caps 准入、补全并集边界吞失的 974 net 臂/条目/NetListener 字段、net 载荷 cleanup no-op 臂保 master teardown 语义）；规范三源 9d0f973cb（m2-stdlib-io.md 新建 67 行 sha256 45f9db6a2270253f、replacement-scope +13、binding-and-assembly +12）；merge-owned 冻结并集 9 组 18m15s：**7 组 PASS=3098 绿**（stdlib 171/ace-stdlib 2/ace-source 610/ace-av 1272/ace-native 281/ace-a2x 247/a2r-std 115）+2 组 FAIL=master 预存红（ace-abi 1 例/ace-runtime 19 例，主检出同例同红 7.78s/1.24s 取证，owner=批量回归 triage）；wt-guard clean 后 ff-only 落地 master tip=9d0f973cb；补提交他-session 遗留 untracked 1063 骨架 fe75d239b，见 §9 第 8 轮）

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。（✅ 见 §9 第 8 轮 cleaned 检查点）

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-973/auto-lang`，分支 `plan-973-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-973/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-16 stage: merge | plan_id: PLAN-973:r13 | completion_kind: delivered | outcome: **pass**（landed+canonical_verified+archived+cleaned）
- **prepared**：reviewed 基线 0a6fb9100（r13 final pass）→ rebase 到 master fe75d239b；提交映射（旧→新，经两次 rebase）：e1dc1d389→3ccb1b77a、1f234a9e8→7d9dd671f、132193f7e→2f86a9288、216fd055d→f29935840、a9eceef38→6c0facac9、0a6fb9100→01be31d4e、+和解修复 d59c92331、+规范沉淀 9d0f973cb=**delivery commit**。range-diff 非全等（冲突和解增量并入首尾提交：974 net 臂/条目补全、NetListener released 字段、RandFill/NetTcpListen 条目修复、net 载荷 cleanup no-op 臂、sys caps 准入被截断分号修复——后者由 post-rebase 复跑 sys973 捕获并实证修复），等价性以 rebased 基线上的实跑证据替代证明（见下）。
- **rebase 后 sanity**：sys973 AV 15/15+native stub 1/1 绿（首跑 a2py 13.9s/a2r 14.6s 冷缓存超 10s 线→热复跑 2.95s/3.79s 归因闭环）；ace-av-core/ace-native/ace-abi 编译零错。
- **merge-owned 冻结并集**（worktree @9d0f973cb，`cargo t stdlib ace-stdlib ace-source ace-av ace-native ace-a2x ace-abi ace-runtime a2r-std`，wall 1095.0s）：7 组 PASS（3098 例）+2 组 FAIL=master 预存红（ace-abi `source_foreign_formal_scalar_review_transition…` 1 例；ace-runtime text 车道 19 例 fault::v05×5/lifecycle::v04×6/smoke×2/textops::v07×6/trap::v06×2，rust-lnd `__CxxFrameHandler3/_fltused`）——两组均在主检出 master fe75d239b 同例同红隔离复现（7.78s/1.24s），非 973 面，owner=批量回归 triage（KNOWN-DEBT D-973）。
- **landed**：主检出仅 conformance.rs CRLF 工件碰撞（numstat 0/0 零内容，还原后放行）；`git merge --ff-only plan-973-dev` 成功，master tip=9d0f973cb=分支 tip，无 merge commit；主检出代码路径零 WIP（余脏项均为他-session docs/design/plans 簿记）。
- **canonical_verified**：docs/specs/ace-migration/m2-stdlib-io.md（新建 67 行，sha256 前16 `45f9db6a2270253f`）、replacement-scope-and-evidence.md（+13，PLAN-973 A 切片挂接节）、stdlib/design/ace-binding-and-assembly.md（+12，sys 装配集增量节）均在交付提交；遗留 untracked 的 1063 计划骨架补提交 fe75d239b（规范/计划引用依赖）。
- **archived**：git mv → docs/plans/archive/973-ace-m2-stdlib-io.md，status: archived，T-05/06/07 勾选，本收据落档。
- **cleaned**：wt-guard clean → 移除 worktree D:/autostack/.wt/lang-973/auto-lang+分支 plan-973-dev+空组目录；plan-artifacts/plan-973/ 整目录删除（材料已折入本 Plan；无具名持有）；retention registry 刷新；批量回归 due check：**未到期**（973 非 5 倍数；收据 2026-11-14T23:23Z+36h<48h）。
- 测试纪律：本 stage 执行=冻结并集 9 组 1095.0s（准备含增量编译）+sys973 sanity 三套+主检出隔离重放 2 例；无新增用例；a2x 腿冷/热计时分列如上。
- 移交在案：1064（byte→Str 面 D-01..D-09）/1065（AC 真身）/1063（Linux L-01..L-12）；KNOWN-DEBT D-973（master 预存红两组/基线 warning/967 读出参帧/F6..F8）。

stage: goal | plan_id: PLAN-973 | plan_revision: r13 | round: 8 | from: merge | to: done | status: archived | outcome: pass（delivered：landed 9d0f973cb+规范三源+归档+清树；known reds 具名非 973 面） | worktree: （已移除） | commit: 9d0f973cb | next: done

2026-11-16 stage: work+review 修复轮（第 7 轮，F1 裁定 b + F2..F5 修复 + 重审）| plan_id: PLAN-973 | plan_revision: r13 | worktree @ **0a6fb9100**（base 3279f2f72）| outcome: **final pass → reviewed**
- **F1 裁定落册（用户选项 b）**：[PLAN-1065](1065-ace-ac-sys-real-bodies.md) 取号创建为 AC 真身承接 owner（drafting 0/6，承接表 A-01..A-04：Win64 50 真身/V-09 AC 正例腿/AC-COVERAGE 升真实运行/Linux 代码臂协同 1063）；V-09/AC-07/AC-COVERAGE 行文修订（§6/§7/§10），T-09 按裁定复勾（973 AC 交付面=stub 诚实拒绝+负控 sys973_native_stub_returns_unsupported）。
- **F2/F3/F4 同树修复（commit 0a6fb9100，2 文件 +2/-1）**：F2 触面 warning 清零——`stdlib_sys_973.rs` 删未用 import `assemble_stdlib_aggregates_inputs`、`asm_lane_product` 参数 `tag`→`_tag`；F3 `sys973_dialect_bisect` 探针源内联（原读仓外 `plan-artifacts/plan-973/probe_main.at`，现自包含 const，"temporary" 注记改 permanent）；F4 `process.at:47-52` argv staging 注记改"已由 spawn5/cps 终局替代"。F5 rand 裁定已 §10 落册（34343fe51）。
- **复验（修复后）**：sys973 AV **15/15 PASS**（max 8.73s/例 ≤10s）；重编译 warning 输出零 `stdlib_sys_973` 命中（余为其他基线文件）；rustfmt --check 该文件 exit 0。本轮无新增用例（dialect_bisect 改内联不增语义面）。
- **重审（affected 项，同会话限制声明）**：F1=裁定+1065 承接表核对（替换集 SYS973_KINDS 与 50 操作清单一致）✓；F2=warning 复编译实证 ✓；F3=15/15 含 dialect_bisect 绿+grep 确认无仓外读取 ✓；F4=注记文本核对 ✓；F5=§10 记录核对 ✓。未受影响证据（四表/车道门/零委托/外库腿/ace-host/第 4 轮并集）复用第 6 轮（代码仅 +2/-1 注释与测试 hygiene，a9eceef38 证据链有效基线推进至 0a6fb9100）。
- **pending_closeout_gates（merge-owned）**：冻结并集复跑或复用判定（基线 0a6fb9100 vs 第 4 轮 a9eceef38——改动仅测试 hygiene+注释，`ace-av` 组证据需基线推进重跑，merge 时核）；master 预存红两组归批量回归。
- 基线：base 3279f2f72 → e1dc1d389→1f234a9e8→132193f7e→216fd055d→a9eceef38→**0a6fb9100**。

stage: goal | plan_id: PLAN-973 | plan_revision: r13 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass（修复轮+重审 affected 全闭；F1→1065/F2..F5 闭；最终 pass 绑 r13/0a6fb9100） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: 0a6fb9100 | next: merge（T-06 规范沉淀+合回，T-07 归档清树）

2026-11-16 stage: review | review_scope: final | reviewer: 独立只读审计 agent（本 goal 会话派发，108 次只读工具调用，file:line 证据）+ 复审主持（同会话限制：r1–4 为前会话执行，本轮结论从工件重建并独立重跑） | plan_id: PLAN-973 | plan_revision: r12 | outcome: **needs_fix** | worktree: D:/autostack/.wt/lang-973/auto-lang | branch: plan-973-dev | reviewed_commit: a9eceef38 | base_commit: 3279f2f72
- **复审重跑（29/29 绿，per-case 计时全 ≤10s）**：sys973 AV 15/15（max 6.13s a2r 腿）；abi 外库腿 2/2（av_c 7.74s/av_rust 8.00s）；native stub 1/1（0.59s）；ace-host 11/11（max 0.92s）。准备编译增量热 2.35s/11.02s。第 4 轮 9 组冻结并集同基线 a9eceef38 复用有效。本轮无新增用例。
- **核验通过**（审计 agent + 主持抽查）：四表对齐 50/50/50/51（source platform_io.rs L196-540 恰 50 op + av-core 枚举/规格表 50 + native Win64 COFF 50 符号 + a2r rt.rs 50 fn + a2py ports.rs 51 含 mem.read_u8 辅助，符号 diff 空）；车道门 7 处注记生效非死代码（ExecOutcome::Exit 生产点 av.rs:6596 由 v03 exit{code:97} 实证）；默认 provider 零 libc/Rust std 委托（kernel32/bcrypt extern "system" 直声明，std:: 仅 BTreeSet 账本与 Linux OnceLock）；V-04 故障注入真实触发（inject_file_error:1123→-1 且磁盘零副作用）；装配集/绑定双点身份/消费入口齐全；新增代码零 TODO/FIXME/unimplemented/println 残留；规范增量目标存在（两个 modify 对象在库；m2-stdlib-io.md 待 merge 新建，符合流程）。
- **AC 结果映射**：AC-01/02/03/06/08（交付面）/AC-PHASE/AC-PLATFORM-IO（Windows 格）/AC-LOAD/AC-LINUX-PHASE/AC-AUTO-STD = pass；**AC-07/AC-COVERAGE = partial**（AC 后端腿）；AC-NATIVE = pass（默认 provider 面，AC 路径例外见 F1）；AC-04 = needs_fix（本复审）；AC-05 = merge 待。
- **发现（F-稳定 ID）**：
  - **F1[中]** AC 后端 sys973 全 50 操作为 named-unsupported stub（-10），V-09"AC真实调用OS API"正例腿未交付且无具名移交行——`native/src/platform_io.rs:139-142`（"a REAL body lands with its AC increment"无 owner）、`:1476-1482` Win64 全 stub、`platform_io_linux.rs:1066-1070` ELF 同（L-12 仅 Linux 侧）。T-01 冻结原文"AV/native+AC+走平台操作真跑"；T-09 勾选证据（绑定绿+外库腿）不覆盖 V-09 AC 腿。影响 AC-07/AC-COVERAGE/T-09。**修复路由=用户裁定**：a) 本计划内实现 50 真身（cranelift 内联或 shim 符号两模式均有存量先例）或 b) 具名延后继计划并修订 V-09/AC-07 行文（验收变更须用户批准）。
  - **F2[低]** 触面 warning 未清零：`stdlib_sys_973.rs:17` 未用 import `assemble_stdlib_aggregates_inputs`、`:46` 未用变量 `tag`——与第 4 轮"计划触面 warning 清零"声明矛盾（AC-04 健康项）。修复=同树清理+复编译验证。
  - **F3[低]** `sys973_dialect_bisect` 非封闭：读仓外未跟踪 `D:/autostack/plan-artifacts/plan-973/probe_main.at`（`stdlib_sys_973.rs:210-218`）——merge 删材料目录后必红。修复=内联源入测试体。
  - **F4[低]** `process.at:47-51` argv List\<Str\> staging 注记指向"车道增量"，实际已被 spawn5/cps 终局替代（§9 第 1 轮）且 D-01..D-09/L 表无此行——修复=修订注记为已替代终局。
  - **F5[低]** `rand.at:8-10` 旧 `random.rand_int`/`Random` 不移植"recorded ruling"不可追溯（973/1063/1064 均未逐名记录）——修复=计划 §10 具名落册。
  - **F6..F8[信息→债务登记]** Unicode **路径名**切片未测（内容已测）；file.at:162-171 read_text 注记块重复两遍；V-02 sys 能力无授权拒绝专测（机制由 1039 泛测+ledger 共享）。
- **债务候选已登记**：KNOWN-DEBT D-973（master 预存红两组、基线存量 warning ACE15+core132、967 读出参缓冲帧缺口、F6..F8）。
- next: work（F2/F3/F4/F5 同树修复；F1 待用户裁定后按 a/b 路径收口）。

stage: goal | plan_id: PLAN-973 | plan_revision: r12 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix（29/29 重跑绿+四表/车道门/零委托核验过；F1 AC 真身腿未交付未移交=验收内缺口，F2..F5 小缺陷） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: a9eceef38 | next: work（先 F1 用户裁定）

2026-11-16 stage: work | plan_id: PLAN-973 | plan_revision: r12 | **第 5 轮收口（车道裁定落册，work 全任务完成 → execution_done）** | worktree @ a9eceef38（零代码改动，clean；复用第 4 轮冻结证据） | outcome: **pass**
- **用户裁定（2026-11-16，原 §10.1 选项 c）**：byte→Str 车道面具名延期后继计划。[PLAN-1064](1064-ace-stdlib-byte-str-deferred-surfaces.md) 以 `scripts/new-plan.sh` 取号创建（master 主检出，drafting 0/7）为承接 owner：移交表 D-01..D-09 逐面记录代码锚点（file.at:162-168/env.at:18-21/process.at:40-44,139-141/storage.at:1-8 等，基线 a9eceef38 快照）与旧 sentinel 合同；车道解锁落点（972 域修订 vs 1064 内建）与 972 归档同车道 (b) 解码项是否同批=其 §10 待澄清。973 不反向依赖 1064。
- **任务关闭**：T-02/T-03/T-08 按 r11 已交付面勾选。byte→Str 摘除面（read_text/dir_names/env get 族/canonical/cwd/cmdline/捕获文本/存储重建）与未实现面（zip/inflate、KV storage）全部落 1064 移交表，无悬空项；AC-02/AC-03/AC-06/AC-08 按已交付范围关闭（延后切片 owner=1064，非 NA）。AC-01/AC-PHASE/AC-07/AC-NATIVE/AC-PLATFORM-IO/AC-COVERAGE/AC-LOAD/AC-LINUX-PHASE/AC-AUTO-STD 沿第 1–4 轮证据关闭（Windows required 门+Linux 移交 1063）。
- **零代码/零测试轮**：裁定为纯合同变更，代码基线不变（worktree a9eceef38，git status clean 复核）；复用第 4 轮冻结并集（9 组：7 PASS+2 组 master 预存红具名）与 sys973 16/16、abi 外库腿 2/2、ace-host 11/11。本轮无新增用例、无测试执行（test-discipline：纯文档轮）。
- **work 交接就绪**：全部 work 任务与 criteria 完成、阻塞问题清零（原 §10.1 已裁定）；唯一树保持给 T-05 复审（停写）；master 预存红两组归批量回归 triage（非 973 面，§9 第 4 轮取证）。
- 基线：base 3279f2f72 → code a9eceef38（e1dc1d389→1f234a9e8→132193f7e→216fd055d→a9eceef38）；材料 scope D:/autostack/plan-artifacts/plan-973/。

stage: goal | plan_id: PLAN-973 | plan_revision: r12 | round: 5 | from: work | to: review | status: execution_done | outcome: pass（用户裁定 c 落册+1064 承接建立+T-02/03/08 关闭；work 全任务完成） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: a9eceef38 | next: review（T-05 独立复审）

2026-11-16 stage: work | plan_id: PLAN-973 | plan_revision: r11 | **第 4 轮收口（commit a9eceef38，T-04 冻结并集，非完成）** | worktree @ a9eceef38（base 3279f2f72） | outcome: 继续 executing（T-01/T-04/T-09/T-10 勾选；T-02/03/08 待车道裁定）
- **警告清理**：本计划引入仅 2 处——mir_verify.rs:857 重复 `CAP_PROBE` 臂（e1dc1d389 diff 错位）已去重；ace-host `map_outcome` 缺 `ExecOutcome::Exit` 臂（自 e1dc1d389 起该 crate 非穷尽匹配不可编译）已补（映射 `HostFailure{code:"av.exit"}`，退出码在消息）。计划触面 rustfmt 清零（rustfmt 沿 mod 树多格式化同 crate 5 个未直接触碰文件：mir.rs/ownership.rs/f0_abi_axes.rs/f0_t13.rs/sched_m2_968.rs，纯格式）。**基线存量具名不动**（git 取证在 3279f2f72 同样存在）：ACE 侧 15 条（mn-linux-av-runner×9、closures_fn_session×2、elf_admission×1、owned_session×1、native cache/emit×2）+ auto-lang-core ~130 条——均不在本计划触面，待 T-05 记 KNOWN-DEBT 候选。
- **受影响回归（清理提交验证）**：av+native sys973 **16/16**（24.3s 含增量编译；15 例 av+1 例 native stub）；abi 外库腿 **2/2**（5.8s，av_c/av_rust dispatch）；ace-host **11/11**（51.4s，e0/t02/t08 facade——首次随 Exit 臂修复恢复可编译，单例 ≤0.64s）。
- **T-04 冻结并集**（`cargo t stdlib ace-stdlib ace-source ace-av ace-native ace-a2x ace-abi ace-runtime a2r-std`，worktree @ a9eceef38，14m46.8s）：**7/9 组 PASS**（stdlib/ace-stdlib/ace-source/ace-av/ace-native/ace-a2x/a2r-std）。两组红均为 **master 预存**（主检出同例同错复跑取证，隔离重放非争用）：
  - ace-abi 293/294：`source_foreign::source_foreign_formal_scalar_review_transition_restores_without_hidden_interface_production`（restore→`foreign.authority.runtime-or-emitter`；worktree 6.3s 红 + master 7.2s 红；最后全绿=批量回归 2026-11-14T23:23Z b1dbab313；此后 abi 路径零提交，依赖面有 968/972/974 合入——归属待 triage，非 973 面）。
  - ace-runtime 65/84：text 车道 19 例（fault::v05×5/lifecycle::v04×6/smoke×2/textops::v07×5(实为6含重复行)/trap::v06×2）全为 `rust-lld: undefined symbol: __CxxFrameHandler3/_fltused`（master `textops::v07_slice_ranges` 0.4s 同红；`_fltused` 指向浮点目标物链接缺 MSVC CRT 符号，疑似 972 f64 五端面后置影响或工具链变动——归属待 triage，非 973 面）。
  - 973 自身全部用例绿：sys973 15 例（probe01/V-03×2/V-04/V-05/V-08/V-09/V-10/dirs+replace/env 真实进程/path/time/熵/a2r/a2py/dialect）+ native stub + AV×C/AV×Rust 外库腿 + ace-host 11 例。§6 各 V 行归属齐全（V-LINUX-PHASE=§10 表 NOT_RUN/1063 按 r11 门；V-AUTO-STD=sys 层自有 .at 源构造）。
- **具名未闭**：①zip/inflate+storage+read_text 族（byte→Str 车道缺口，**待用户裁定**，阻塞 T-02/T-03/T-08 勾选与 AC-08 storage/Task/Stream 切片）；②两组 master 预存红（上列，建议走批量回归 triage/修复 worktree，不阻本包 review 的自有面）；③T-05 复审、T-06/07 merge 未开始。
- 测试纪律报告：本轮**无新增用例**；会话计时=警告清扫 cargo check ×2（缓存热）+ scoped 三套件（24.3s/5.8s/51.4s）+ 并集 14m46.8s + 隔离重放 3 例（6.3s/7.2s/0.4s）+ 用例列举 2 次（nextest list，秒级）；并集前占用检查首次因 bash/PowerShell 引用错误无效（已订正复查：仅并集自身 5 个 cargo 进程）。

stage: goal | plan_id: PLAN-973 | plan_revision: r11 | round: 4 | from: work | to: work | status: executing | outcome: pass（里程碑级：警告/fmt 清理+T-04 冻结并集 7/9 组 PASS+2 组 master 预存红具名；**整包未完成**：zip/storage 车道待裁定） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: a9eceef38 | next: 用户裁定 byte→Str 车道（或直接排 T-05 独立复审）

2026-11-16 stage: work | plan_id: PLAN-973 | plan_revision: r11 | **第 3 轮收口（commit 132193f7e，里程碑 3：A2PY 腿+Linux 移交表，非完成）** | worktree @ 132193f7e | outcome: 继续 executing |
- **A2PY 腿 GREEN（真 OS Python，value:42）**：51 个 sys 端口契约（io.stdin/mem.read_u8/file×26/env×4/clock×4/entropy×1/process×12）入 ports 目录表+lower platform_port_id；python 运行时系统资源实现（二进制模式文件、UTF-16 暂存、句柄表、clock-hi 寄存器、secrets 熵、subprocess 捕获/超时杀树）；修复 provider 平台运行时并入重复（限定 Collections 家族——1041 vcoverage 复绿）。
- **§10 Linux 移交表 12 行入册**（L-01..L-12，deferred-required/NOT_RUN，owner=1063；实现/验证状态分列；L-07 进程子序具名未写+设计注记；canonical 符号链接解析具名缺口）。
- 15/15 stdlib_sys_973；1041/1042/1043+972 套件 31/31；ace-a2x-python/rust/core PASS。

2026-11-16 stage: work | plan_id: PLAN-973 | plan_revision: r11 | **第 2 轮收口（commit 1f234a9e8，里程碑 2，非完成）** | worktree @ 1f234a9e8 | outcome: 继续 executing |
- **A2R 腿 GREEN（真 rustc+运行，value:42）**：平台操作经 collections 家族 a2x 降级（共享操作表驱动 a2x_<callee>）；运行时 sys 资源存根按 has_platform_ops 并入文件集；修 4 个真实缺陷——ListLen 出口清扫先于内联渲染（len=0）、A2xList\<E\> 变体载荷泛型（RustProgram.list_elems+prelude 表）、借用返回尾须 .clone()、AV 值风格 List 返回改原地累积（单尾移纪律保持）。
- **V-03 进程实测全绿**：spawn5 退出码（cmd /c exit 7）、捕获计数（echo CRLF=7）、1500ms 超时 Job 树杀（-11）、proc_exit=ExecOutcome::Exit；引号策略改按需（cmd.exe 对全引号敏感）。
- **T-08 权限链**：spawn5 attrib ±R → is_read_only/write 失败/delete 失败 oracle；mtime 带。**T-09**：同源 TargetOs::Linux 装配绑定（code-only/NOT_RUN/1063）。**T-10**：真实熵（charset/两次差异，码点列表）。
- 14/14 stdlib_sys_973；972 套件 18/18；ace-a2x-rust/core PASS。
- 仍开放：AV×C/AV×Rust 腿（967 帧边界——读出参缓冲帧为具名缺口）、AC 具名 stub 拒绝测试、zip/storage（byte→Str 车道缺口阻塞）、T-04 冻结并集、T-05 复审、T-06/07 merge。

stage: goal | plan_id: PLAN-973 | plan_revision: r11 | round: 3 | from: work | to: work | status: executing | outcome: pass（里程碑级：A2R 腿绿+A2PY 腿绿+V-03/08/09/10 全绿+Linux 移交表入册；**整包未完成**） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: 132193f7e | next: work（AV×C/Rust 腿→AC stub 测试→T-04→T-05→merge；zip/storage 与 read_text 面待车道缺口用户裁定）

2026-11-16 stage: work | plan_id: PLAN-973 | plan_revision: r11 | **第 1 轮工作收口（里程碑 1，非完成）** | worktree: D:/autostack/.wt/lang-973/auto-lang @ e1dc1d389（base 3279f2f72） | outcome: **继续 executing**（T-01 前探+T-02 核心落地并全绿；整包未完，下列余项具名） |
- 已验证：8/8 stdlib_sys_973（probe01/V-04/V-05/env真实进程/dirs+replace/真实时钟/path/方言）；ace-av conformance 1139/1141（2 例 ::performance:: 并行负载归因、隔离复跑绿：closures_recovery 27.2s、closures_fn 33.5s PASS；closures_native_924 200s 档同类别记录）；ace-core/source-query/av-core/lowering/native 993/995（同上 1 例 performance 争用）。
- 已交付（commit e1dc1d389）：50 冻结平台操作×4 表；Win32 真实后端（CreateFileW/ReadFile/WriteFile/SetFilePointerEx/GetFileSizeEx/GetFileInformationByHandle/FlushFileBuffers/CloseHandle/GetFileAttributesW/DeleteFileW/CreateDirectoryW/RemoveDirectoryW/MoveFileExW/GetFullPathNameW/GetFinalPathNameByHandleW/FindFirstFileW/FindNextFileW/FindClose/env三件/GetSystemTimeAsFileTime/QPC/QPF/SleepEx/BCryptGenRandom/CreateProcessW+Job+管道+PeekNamedPipe/WaitForSingleObject/GetExitCodeProcess/TerminateJobObject/GetCurrentDirectoryW/SetCurrentDirectoryW/GetCommandLineW）；Linux 原始 syscall 臂（代码完整 NOT_RUN/1063）；Capture oracle+故障注入；统一资源账本（文件/目录/进程载荷、lo+hi、读写预算、cleanup 关闭）；sys 冻结装配集（与 972 集分离：stdlib_sys_files + is_frozen_std_sys_module + assemble_stdlib_sys_inputs + binding 双点身份）；车道门 7 处（resolve/verify×2/lowering×2/abc×2/host/mir_verify/UnitView::host_import_at/ExecOutcome::Exit×6 消费点）。
- **车道缺口（具名，待用户裁定或后续计划）**：①字节→Str 构造（影响 read_text/dir_names/env.get/canonical/cwd/cmdline/存储重建/捕获文本——均已从模块面摘除并在模块头注明）；②argv List<Str> 暂存（元素视图仅 len 读）→已以 spawn5/cps API 替代；③zip/inflate、V-03 进程树实测、AC/A2R/A2PY 六路径腿、AV×C/Rust、Linux 移交表、规范增量——全部未做。
- 下一步（next: work）：T-02 余项/T-03 消费者接线/T-08/09/10 → T-04 冻结并集。
- 材料目录 D:/autostack/plan-artifacts/plan-973/（探针/脚本/回归记录）；本轮无新 Linux 测试执行（裁定遵守）。

stage: goal | plan_id: PLAN-973 | plan_revision: r11 | round: 1 | from: work | to: work | status: executing | outcome: pass（里程碑级：T-01 前探绿+T-02 核心管线绿+回归绿；**整包未完成**） | worktree: D:/autostack/.wt/lang-973/auto-lang | commit: e1dc1d389 | next: work（T-02 余项→T-03→T-08/09/10→T-04；车道缺口 2 项待具名裁定）

2026-11-16 stage: work / T-01..T-02 首增量·核心管线全通（goal 链第 1 轮续）：
- **T-01 前探 GREEN**（probe01）：A中😀\0=9B Unicode+NUL 临时文件 write→exists→size→read_bytes 字节同一性→失败恢复→append→原子替换→清理，全链 .at→1043装配→aggregates绑定→HIR→MIR→平台操作→Win32 CreateFileW/ReadFile→真实磁盘→读回，8/8 sys973 测试 PASS（probe01/V-04注入/V-05 oracle/env真实进程/dir+replace/真实时钟/path算法/方言探针，cargo nextest -p ace-av --test conformance stdlib_sys_973）。
- **车道门扩展（全部带 PLAN-973 注记）**：resolve.rs Aggregates 面准入平台保留名；verify.rs Aggregates 域平台内在+973能力集（stdin/file/path/env/clock/entropy/process/io.write/mem）+Closures 域平台准入；lowering hir_adapter 两条内在路径（closures 家族 auto.platform.*→host_import→PlatformCall）；abc.rs 编解码 PlatformCall 家族加 Closures；host.rs 平台导入模块准入；mir_verify PlatformCall 非 host-console 面准入；ModuleView::host_import_at（unit 视图按属主单元解析 host 索引）；av.rs proc_exit→ExecOutcome::Exit（5 处消费点补臂）。
- **方言适配裁定（工程决定，已验证）**：packages-aggregates 面无 while/计数 for→全部分块递归（768×40×40≈1.2MiB≤运行预算，帧深≤848<1024）；if 条件以标识符结尾必须括号（命名构造器歧义）；`is` 全 return 臂需 int/unit/Str 结果→变体面 validate+decode 分离/省略；元素视图仅方法读→argv 暂存改全引号 cps 链；Str 参数尾移一次纪律（972 同源）→ 所有走查器重构；**字节→Str 构造缺口（972 已知边界）→ read_text/dir_names/env_get/canonical/cwd/cmdline/存储重建/捕获文本 = 具名车道缺口**（下条 needs_replan）。
- **append 语义修正**：OPEN_APPEND 一律 FILE_APPEND_DATA（OS 追加语义，设计§7），探针 -11 复现后修复转绿。

2026-11-15 stage: work / T-01 决定冻结（goal 授权全链，worktree lang-973 @ 3279f2f72）：
- **分层**：共享 Auto 算法（stdlib/ace/sys/ 新装配集：file/path/env/process/time/rand/zip/storage 八模块，1043 平铺候选 .at.at/.c.at/.rs.at/.py.at）→ 受检平台操作（扩展 1039 冻结模式：source 表 + av-core 表 + AC COFF/ELF shim + a2x 运行时端口五处对齐）→ OS 入口（Win kernel32/bcrypt 公开 DLL；Linux 直接 syscall）。全平台操作保 i32→i32 冻结准入形状；64 位值（size/offset/now_ms/mtime）以 lo+hi 两 i32 组合，配 `file_hi`/`clock_hi` 高半查询操作。路径遵循 1041 冻结模式：Windows 平台操作收 UTF-16LE region(units)，Linux 收原始字节 view；UTF-8↔UTF-16 转换在共享 Auto 源内完成。
- **平台操作集（~47）**：file(11) open_w16/open_bytes/read_some/write_some/seek/tell/hi/size/mtime_ms/flush/close；path(10) attrs/delete/replace/canonical × w16/bytes + dir_create ×2；dir(5) open_w16/next_w16/close + open_bytes/fill_bytes(getdents64→region，Auto 解析)；env(4) get/set/remove_w16 + get_bytes(/proc/self/environ)；io(1) io_stdin；clock(4) unix_ms/mono_ms/hi/sleep_ms；entropy(1) rand_fill(BCryptGenRandom/getrandom)；proc(11) spawn_w16(CreateProcessW+Job kill-on-close+管道)/pipe_read(PeekNamedPipe+WaitForSingleObject 有界等待，非忙轮询)/wait/release/exit(AV=ExecOutcome::Exit，AC=ExitProcess/exit_group)/cmdline/cwd/chdir。
- **公开 API**＝旧 sentinel 约定逐面保留（read_text ""/write 0/-1/exists bool/size -1/secure_hex ""等）；zip 读三元组与 KV storage 为纯 Auto 组合（zip inflate 纯 Auto 实现）；console 复用 1041 io.at（具名边界），973 新增 stdin 读。具名延期：Linux env_set/remove（无 syscall 可用，需 1040 入口捕获 envp 或宿主 environ 符号——owner=1063 移交行）；文件 watch 无旧公共 API（调研发现，无义务可保，记录不造）。mtime/walk/metadata 以现有操作 Auto 组合，如未及交付则具名缺口。
- **六路径**：AV/native+AC+走平台操作真跑；A2R 经 rt.rs 新增 a2x_* 存根（Rust 目标运行时依赖另列）真 rustc 腿；A2PY 经 ports+runtime 真 Python 腿；AV×C/AV×Rust 经 967 帧（(view,≤2×i32[+out])→i32）覆盖写路径族实际 dispatch，读路径出参缓冲帧为 967 域缺口（与 972 同源具名项，unblock=967 域 out-buffer 帧交付）；不以配置名/同 native 执行冒充。
- **测试车道**：av/tests/cases/stdlib_sys_973.rs（AV+capture oracle+负例）+ native 腿（COFF 进程内）+ a2r/a2py 显式构建腿（禁日常测试内起 cargo）；迭代 `cargo nextest -p ace-av --test conformance <filter>`；冻结并集 `cargo t stdlib ace-stdlib ace-source ace-av ace-native ace-a2x ace-abi ace-runtime a2r-std`（触面定稿前可增）。新用例 ≤10s/条；Linux 腿全部 NOT_RUN/owner=1063。
- **前探**：最小正例（Unicode 临时文件 write→read 往返经 packages 装配 lane AV 真跑）+ 非法近邻（不存在路径 open→sentinel，零副作用）+ 失败恢复（错误后重建再读写）串最长 producer→consumer 链，随 T-02 首增量落地。
- **依赖核验**：9 依赖全 archived（2026-11-15）；967 已合入但帧仅 (view,≤2×i32[+out])→i32（读出参缺口如上）；1043 装配/#in、966 #is、972 asm 集、1041 console、1039/1040 底座符号/预算（65_536/call、1_048_576/run、handle 1024）沿冻结值。
材料目录 D:/autostack/plan-artifacts/plan-973/ 已登记。本轮未实施代码；实施自本记录起算。

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r11：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 stage:new / r8：设计承接；新增平台I/O AC/V/SD由T-01,T-09,T-10及原最终work负责。原任务/进度保持，无实现或独立产品review；新三包drafting。

2026-11-12 stage:new / PLAN-973 r7：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原10任务/全部AC与V/依赖/阶段/预算和drafting 0/10；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r6：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部10个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/10；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..10、AC-07/AC-08及V-09..10，按Design56正式接M2 FFI/新版标准库责任，0/10。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

### 已裁定事项（2026-11-16 第 5 轮落册）

1. **字节→Str 构造车道缺口**（原 §10.1 待裁定项）：**用户裁定选项 c——具名延期后继计划**。承接 owner=[PLAN-1064](1064-ace-stdlib-byte-str-deferred-surfaces.md)（drafting 0/7，移交表 D-01..D-09：read_text/dir_names/env get 族/canonical/cwd/cmdline/捕获文本/存储重建/zip/inflate/KV storage/read_bytes Option\<List\> 形状）。本包 T-02/T-03/T-08 按已交付面关闭；AC-08 的 storage/Task/Stream byte→Str 切片移交 1064，不冒已交付。车道解锁落点与 972 (b) 解码项是否同批=1064 §10 待澄清，非本包反向依赖。
2. （非阻塞本包，供 triage）两组 master 预存红见 §9 第 4 轮：ace-abi source_foreign authority 1 例、ace-runtime text 车道 19 例（rust-lld `__CxxFrameHandler3`/`_fltused`）；主检出同红，建议批量回归 triage 或独立 fix worktree。

### 待用户裁定（2026-11-16 第 6 轮复审 F1，阻塞 T-09/V-09 AC 腿收口）

**AC 后端 sys973 真身**：AC（直接编译后端）的 50 个 sys973 平台操作当前全部为 named-unsupported stub（-10，诚实拒绝），V-09"AC真实调用OS API"正例腿未交付且无移交 owner（代码注记"a REAL body lands with its AC increment"无具名计划；Linux 侧仅 L-12 覆盖）。选项：a) 本计划内实现 50 真身（cranelift 内联体与 shim 符号链接两模式均有 1039/1041 存量先例，属验收内工作，体量较大）；b) 具名延后继计划并修订 V-09/AC-07/AC-COVERAGE 行文为"AC stub=已裁定交付面"（验收变更，须本裁定批准）。

**→ 已裁定（2026-11-16 第 7 轮落册）：选项 b**。承接 owner=[PLAN-1065](1065-ace-ac-sys-real-bodies.md)（drafting 0/6，承接表 A-01..A-04：Win64 50 真身/V-09 AC 正例腿/AC-COVERAGE 升真实运行/Linux 代码臂协同 1063 L-12）。本包 V-09/AC-07/AC-COVERAGE 的 AC 真实调用切片自本条起移交 1065，973 的 AC 交付面=stub 诚实拒绝（-10、无静默 fallback）+具名负控测试（sys973_native_stub_returns_unsupported）；T-09 按此复勾。

### 具名裁定补录（2026-11-16 第 6 轮复审 F5）

旧 `random.rand_int`/`Random`（[stdlib/auto/random.at](../../stdlib/auto/random.at)，Plan 250 非凭据 PRNG 面）**不移植**到 ACE sys 库——ACE 熵面以 `entropy.rand_fill`/`secure_hex`（CSPRNG）承接，非凭据 PRNG 如需再立显式计划。本条为该工程决定的正式落册（原 rand.at:8-10 注记"recorded ruling"自此可追溯）。

### Linux 移交表（2026-11-16，owner=PLAN-1063；本表=当前 NOT_RUN 清单）

按 2026-11-14 裁定：Windows 开发环境不执行 Linux 测试（不启动 WSL/VM、不隐式准备 runner）；以下行全部 `deferred-required / NOT_RUN`，owner=1063，实现与验证状态分离——**代码已写**（编译于 Windows 侧目标面），**未运行**；变化后的实现不自动继承旧绿。

| # | 项 | backend/provider | 实现位置（已写） | 1063 验证要求 |
|---|---|---|---|---|
| L-01 | file open/read/write/seek/tell/hi/size/mtime/flush/close（bytes 族） | AV/Linux（LinuxPlatformIo raw syscall） | av-core platform_io.rs（LSEEK/FSTAT/FSYNC 等直接 syscall 臂） | 同 V-PLATFORM-IO 四格之 Linux×AV：真实 create/write/close/reopen/read/seek/EOF、NUL/空文件/exclusive/append、坏路径/权限/逃逸负例 |
| L-02 | path attrs/delete/dir create+delete/replace/canonical（bytes 族） | AV/Linux | 同上（O_PATH+fstat、unlinkat/mkdirat/renameat、openat 链） | attrs 位/删除/替换原子性/符号链接解析（canonical v1 为语法归一——**具名缺口**：真实符号链接解析移交 1063） |
| L-03 | 目录枚举 getdents64 | AV/Linux | dir_open_bytes/dir_fill_bytes（getdents64 syscall） | dirent64 记录解析（sysplat_lnx.at 内 Auto 解析）与 NUL 分隔名流 |
| L-04 | env get（/proc/self/environ 快照） | AV/Linux | env_get_bytes + Auto 端 env_scan | KEY=VALUE 解析、缺键 ""; env set/remove=具名延期（无 syscall，需入口 envp 或宿主 environ——见 T-01 冻结记录） |
| L-05 | clock trio + sleep | AV/Linux | clock_gettime(0/1)/nanosleep 直接 syscall | 时间带、mono 单调、sleep 真实阻塞 |
| L-06 | entropy getrandom | AV/Linux | getrandom(318) syscall | secure_hex 字符集/两次差异 |
| L-07 | process spawn/pipe/wait/kill 树 | AV/Linux | **未写**（clone+vfork+execve 子序无法在 Windows 宿主上编译验证，冻结设计注记在案） | 1063 交付子序 + V-03 进程树/超时/管道读回收全负例 |
| L-08 | proc_exit | AV/Linux | exit_group(231) 臂 | 受控退出码 |
| L-09 | cmdline/cwd/chdir | AV/Linux | /proc/self/cmdline、getcwd(79)、chdir(80) | 参数拼接与 cwd 往返 |
| L-10 | #in Linux 装配面 | 全端 | sysplat_lnx.at（已随 sys 集冻结） | 绑定已完成（T-09 sys973_t09 真实绑定+链接通过，NOT_RUN 断言在案）；1063 补真实运行 |
| L-11 | A2R/A2PY 目标运行时 Linux 路径 | A2R/A2PY | rt.rs sys 块 cfg(not(windows)) 臂（/dev/urandom 等） | 真实 Linux 解释器/工具链上跑同一腿 |
| L-12 | AC native ELF shim | AC | native/platform_io_linux.rs stub 发射（-10） | 真实 ELF 构建 + Linux 执行 |

旧 1040/1041 归档实际证据保留为历史，不自动为上述行背书。T-09 测试（sys973_t09_target_selection_linux_binds）是绑定面证据；无任何 Linux 执行在本轮发生。


T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
