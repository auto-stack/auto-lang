---
plan_id: PLAN-783
status: execution_done
feature_name: AC/ACC 统一编译产物契约与缓存可行性探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-15
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-cache/project.md
  - docs/specs/auto-hir/stage-contract.md
new_spec_components:
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-ac/artifact-probe.md
touched_goals: []
affects: [auto-ac, auto-acc, auto-hir, auto-cache]
---

# [PLAN-783] AC/ACC 统一编译产物契约与缓存可行性探针

## 0. 变更摘要

承接 [Design 39](../design/39-ac-incremental-syntax-and-language-platform.md)、[Design 40](../design/40-compiler-artifacts-and-cache-lifecycle.md)、[Design 41](../design/41-interface-metadata-and-foreign-abi.md)，统一编译各层产物、身份、引用与生命周期合同，用隔离实验验证真实源码→声明摘要/HIR→对象文件的持久化、重启复用、按需读取和自动回收。

782 尚在修复复审。本计划必验路径使用 master 已合入的 core 能力，辅以 scalar 兼容控制，不依赖其未合入 text-record 实现；可以独立推进。文本资源组合后续接入仍须等待 782 正式收口。

## 1. 目标

1. A00–A12 各层明确 Schema、生产者/消费者、key、引用类别、存储及驻留/回收策略，并区分已实现、探针实现与后续实现。
2. 同一逻辑数据比较两种直接读字段的冻结二进制布局，证明小查询不需加载整库/全树。
3. 接入真实 parser、HIR verifier、native lowering，证明跨工程、跨进程缓存复用和正确失效。
4. 在独占临时 store 验证强数据引用、软保留、lease、自动水位回收、故障恢复和实际空间计量。
5. 给出耗时、内存、磁盘基线及后续阶段建议。

非目标：完整 parser-generator、改写 lexer/parser、全语言增量 CST、全类型序列化、生产缓存服务/迁移上线、pack compaction、热重载/插件装载、C importer、Rust shim 编译系统、ACC 自举迁移。亚秒周转是产品目标，本计划测量距离，不宣称达标。

只新增本仓 `experimental/ac-artifact-probe/` 与配套脚本/报告/计划资料。不修改 `crates/**` 或 `experimental/ac-core/src/**` 的生产行为；auto-down 兄弟仅供解析，不修改。若公开 API 不足或必须扩界，先记录反例并修订契约，不能复制编译器或绕过 verifier。touched_goals 为空，不关闭语言完整性或自举目标。

## 2. 架构方案

独立 probe workspace 消费 ac-core 公开接口，可用隔离 adapter 复用 `auto-cache::storage::BlobStore`；ac-core 不反向依赖产品 crates。新实验不得成为第三套生产缓存入口。

三条线共享身份/manifest 合同：真实编译纵线；大规模语法节点/摘要布局对照；小预算多工程/多进程生命周期验证。造数证明布局和预算，不能冒充真实通用 CST。当前真实纵线从已有 AST 起步，不要求同时建设全量 CST。

未来语法产品消费 GrammarId/Revision、SourceSnapshot、语法节点/摘要合同；FFI 消费接口、布局、调用约定、ownership/callback 与工具链输入合同。本计划冻结接缝和最小描述样例，不交付这两条产品线。

## 3. 技术栈

Rust 独立 workspace；已有 AC parser/descriptor/verifier/native/link；仓内 blake3；结构描述优先 Atom + Schema。二进制为实验版本化布局，不宣称已有 Batom 新格式。元数据层优先评估现有 rusqlite，块层评估 BlobStore。T-01 固定依赖版本、独立 Cargo.lock 与接入方式。

原生平台沿用 Windows x86-64/MSVC、Cranelift、rust-lld。复用 `link::run_deadlined_split`、staged link/publish，不另建无期限执行器。子进程默认截止 60 秒；工具准备若另需时限要写入收据，超时与运行结果分开报告。

## 4. 需求分析与背景调查

入口：[Spec overview](../specs/overview.md)、[auto-ac](../specs/auto-ac/project.md)、[auto-hir](../specs/auto-hir/stage-contract.md)、[auto-cache](../specs/auto-cache/project.md)、[auto-bindgen](../specs/auto-bindgen/project.md)。Spec 是当前知识，Design 是目标，不因起草计划提前回写为已完成。

调查基面 master `1b9cb8db692eab7e04e8f71155f159e375b52edf`；来源指纹见 [planning-inputs.tsv](attachments/783/planning-inputs.tsv)。work 开始重新核对基面/API与782状态，变更按影响重新调查。

| 实际接缝 | 现状/约束 |
|---|---|
| `experimental/ac-core/src/source/parser.rs::parse_program_with` | 有限 profile AST；没有通用 CST/增量 parser |
| `source/adapter.rs::compile_parts_with`、SourceParts | 真源码→Bundle；现有 input_hash 收据不当作全局安全内容键 |
| `descriptor::{bind,write_canonical}`、`verify::*` | 编码基线与可信准入；磁盘 trusted 标志不能构造 CheckedModule |
| `native::lower_object`、`link::link_object_staged` | 真 Checked→COFF→PE 与发布设施，不得以模拟 blob 替代 |
| `crates/auto-cache/src/storage.rs::BlobStore` | 已有块读写；固定临时文件名需要外层同键写协调 |
| `crates/auto-cache/src/{lib,gc}.rs` | 需区分元数据删除和物理释放；本轮盘点差距，不修生产 GC |
| Design41 / auto-bindgen | Auto→C 不是任意 C importer；头文件闭包/宏/搜索路径/sysroot/shim 配置均影响 FFI key |

用户授权：本轮“规划下一个计划”，既有方向包括增量/热重载/插件、未来语法产品、低内存全局 cache/自动清理与通用 ABI。当前授权草拟和调查，不实施代码、不创建树、不联系其他聊天。没有用户指定预算/模型角色；下文数字是探针提出的工作负载，非产品默认上限。

774 与已合入各 profile 为背景；必验纵线依赖 core，scalar 作兼容控制。782 非硬前置。旧版代码供语义、诊断、性能基线参考；将来 parser 解释 grammar 或生成代码尚不裁定，保留共同版本化接缝。

## 5. 详细设计

### 5.1 层级、身份与引用

登记 A00 输入、A01 grammar/table、A02 token/checkpoint、A03 CST、A04 AST/声明摘要、A05 macro/comptime、A06 semantic/interface、A07 HIR、A08 pass/resource、A09 backend/object、A10 link/publish、A11 diagnostics/tooling、A12 runtime generation。每层有 schema/版本、payload/索引、真实 key 输入、消费者、ref、可再生性、内存预算归属、根与失效规则。

全部层级提供边界合同；实际 codec 仅覆盖 source snapshot 引用、真实 AST 声明摘要、core HIR 的受限可执行子集、object manifest、recipe/ref/lease。其余层只给 schema 接缝和合法/非法描述样例，标记未来实现。能映射 Atom/Batom + Schema；热路径用 typed view，不强制动态 Obj 查找。

区分 QueryKey、ArtifactId、ChunkId、逻辑 Module/DeclId、SourceSnapshotId、LocalRef/ExternalRef。hash 包含类型域与规范编码/算法版本；内容身份与来源位置分离。required-data ref 保活闭包；失效指纹/provenance 不默认强 pin 历史上游。对象独立可链接时，不要求 HIR/CST 常驻。

reader 受控拒绝未知 schema/profile、错误 owner/类别、越界/非法循环、损坏块；不存原生指针/struct 转储。原生 payload 重建 Bundle 后重新 verify，不持久化 CheckedModule。缺源/工具链时明确 miss/unavailable，不能假定所有数据无条件可再生。

### 5.2 两种可读布局

A 为固定记录/分段表+索引；B 为变长记录+offset目录。同逻辑数据比空间、访问、修改后新版本生成/共享成本，Atom 文本为语义基线。都用冻结 view，不允许完整反序列化再声称零拷贝；mmap 后遍历全库也不算按需读取。

单独布局 store：至少128 MiB真实写入、64 KiB逻辑页、8 MiB受管热点预算，小查询所需payload≤512 KiB。显式热点分配≤8 MiB，I/O暂存另限两页，索引超预算须分页。生成器流式写入；记录读/解码放大、实际进程 private bytes基线/增量及映射共享页，不能只有自定义计数。结果与全量语义基线一致。

T-01 冻结布局与测量细节；不能事后扩大预算使测试变绿。上述规模是实验负载，不是语言长度/嵌套上限。资源拒绝与 malformed/schema 错误分开，均有边界控制。

### 5.3 真实编译重启闭环

采用 `experimental/ac-core/fixtures/source/valid/` 的 `p01-precedence.at`（exit14）、`p04-while-retest.at`（exit3）、`p12-unexecuted-trap-not-lifted-x1.at`（exit0）、`p13-add-overflow-trap.at`（exit70）；T-01 重放现场 oracle，再选择一个现存 scalar 兼容控制。不能通过更换输入掩盖失败。

真实 AST 摘要和 compile_parts 的 HIR进入产物体系。T-01 先证明一例二进制恢复 Bundle→verify→native执行，以及对象持久化后新进程只链接执行。恢复若只重读文本不能替代二进制可行性证明。有限 codec 支持面须列明。

按阶段纳入 source、grammar/parser/profile/schema、接口依赖、函数体/HIR、lowering/backend、target/ABI/feature、链接输入/支持库/工具链/参数。只改body而签名相同也必须失效对象。仅改工程路径可复用内容，诊断来源绑定当前工程。不支持的target受控拒绝，不制造假产物。

重启HIR命中/对象缺失时重建Bundle并重新verify/lower；对象命中可跳过前端/verify/lower但仍真实link/run。用真实API计数与禁调失败注入共同证明跳过，禁止命中后重新全编译却只改统计。语义对照exit/stdout/stderr，PE非确定字段不当作语义失败；内容ID仍绑定真实持久字节。

### 5.4 生命周期与健康

隔离store实现不可变块、manifest、事务/发布记录、同key跨进程协调；冲突结果不得最后写赢后宣称正常。lease获取和GC判活/删除互锁；崩溃会话可恢复，活会话不得只凭TTL/PID被误删。本轮实际文件/映射reader不等于真实DLL/callback代际安全。

GC与布局使用不同store：高64 MiB、低48 MiB、额外临时空间上限8 MiB。计入块、DB/WAL/journal、临时、日志、orphan、待删文件。至少512次工程/版本churn，无超额硬pin时自动回低水位；硬pin/活lease超额时有界退让/停写并报资源状态，不删live数据。

故障点至少：staging完未登记、登记/发布切换、tombstone后删除前。显式事件屏障触发杀进程，重启验证幂等，无半提交hit、不全清健康store。报告逻辑失效、真实文件/allocated bytes释放、待reader释放、失败重试、峰值与耗时；无法测allocated bytes则明确局限，不拿metadata大小冒称物理释放。没有pack，不宣称解决pack碎片。

只操作有所有权marker的独占受管目录，检查canonical路径并拒绝越界/reparse跳转；用户源码、交付exe不在删除集。删一个项目不能破坏另一项目共享块。provenance关联但非required的旧HIR可回收，缓存对象仍能链接运行。

### 5.5 决策输出

分别测fresh、同进程热点、重启磁盘热、跨工程、body修改、GC并发查询；每场景≥30测量样本，预热另列。记录机器/提交/配置、P50/P95/P99、I/O、内存和磁盘；Rust工具构建准备与编译请求分列，重启不冒称OS全冷。

结论比较A/B、现有AutoCache可复用面/生产差距，提出生产store/健康治理、通用lexer/parser/CST、enum/is/List/模块、接口/C ABI、运行代际的独立后续阶段，不在本计划顺手实现。

### 规范增量

本表为merge拟沉淀内容，本轮不编辑canonical Specs。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/artifact-contract.md | 无统一表→A00–12身份/ref/失效，区分实际codec范围 | 共用接缝 | AC-01 |
| SD-02 | add | docs/specs/auto-ac/artifact-probe.md | 无持久纵线→实际支持面/布局/预算/命令/限制 | 可复验 | AC-02–05 |
| SD-03 | modify | docs/specs/auto-ac/project.md | 新增独立probe导航，不扩大语言面 | 实验边界 | AC-06 |
| SD-04 | modify | docs/specs/auto-cache/project.md | 现有入口→补现状差距与实验/迁移边界 | 防多套生产GC | AC-04、06 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | 补持久payload不携带信任、恢复重验、provenance非强pin | 合法性 | AC-01、03 |

## 6. 测试设计

以下新入口由T-02–06创建，尚未执行：`cargo test --manifest-path experimental/ac-artifact-probe/Cargo.toml <filter>`；新 `scripts/verify-ac-artifact-probe.ps1 -Scope acceptance` 聚合必验矩阵，缺例/skip/子步骤失败须整体失败；`-Scope closeout`供merge冻结提交跑完整probe套件。

| case | AC / invariant | 输入/状态 | filter / work owner | 区分性oracle | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01、03 身份 | 相同内容不同路径；同签名不同body | identity_ / T-02、04 | 内容共享、当前span正确；body对象miss | core，非完整模块系统 |
| V-02 | AC-01、03 信任 | 真HIR篡改type/ref/owner，未知版本/坏块 | admission_ / T-02、04 | mutation生效、真实reader/verifier拒绝；合法兄弟成功 | 有限codec |
| V-03 | AC-02 分页 | 128MiB、小查询、跨页记录 | layout_ / T-03 | A/B与Atom值一致，预算达标，OS/I/O证据 | 造数，非通用CST |
| V-04 | AC-03 真纵线 | fresh/restart HIR hit/obj hit | native_resume_ / T-04 | 四core语义一致，禁调控制，真verify/link/run | Windows现有profile |
| V-05 | AC-03 隔离 | profile/schema/ABI/toolchain/body变化 | invalidation_ / T-04 | 每阶段期望hit/miss明确；unsupported拒绝 | 不实现新target |
| V-06 | AC-04 引用 | 共享块、删项目、对象留/上游冷 | reachability_ / T-05 | required保留、非required HIR删后对象仍运行 | 非runtime pin |
| V-07 | AC-04 并发 | 两producer同key/冲突结果、reader-GC交错 | concurrency_ / T-05 | 实际生产/提交计数，冲突拒绝，无悬空读 | 文件/映射多进程 |
| V-08 | AC-04 恢复 | 三发布点杀进程并重启 | recovery_ / T-05 | 无半提交hit，幂等恢复/orphan回收 | 非pack |
| V-09 | AC-04 健康 | 512churn、水位、pin超额、删除受阻 | gc_budget_ / T-05 | 收敛或有界退让，失效/物理/待删分列 | 专用小store |
| V-10 | AC-04 边界 | 缺源/工具链、越界ref、外部哨兵 | store_boundary_ / T-05 | unavailable/miss清晰，外部源码/exe hash不变 | 不清理任意target |
| V-11 | AC-05 性能 | 六场景，每场景≥30样本 | perf_ / T-06 | 可复验时延/IO/内存/占用，不减验证 | 参考机，无亚秒承诺 |
| V-12 | AC-06 范围 | 依赖图/新旧入口 | acceptance脚本 / T-06、07 | ac-core无反向产品依赖，生产读取/GC零切换 | 依赖/文档审计 |

work运行定向验收和必要的独立cargo check。没有crates源码/语法参考改动，按AGENTS Category A，不运行根cargo t/tv/tt/tb/tu/taa/tf或docs_gen。完整probe套件merge-owned，不得把V-01–12正确性验证后置。

实施与自检由后续work负责；独立最终review尚未指定人，本轮不创建代理/聊天。唯一预定树 `D:/autostack/.wt/lang-783/auto-lang`、分支 `plan-783-dev`，实施/自检/复审顺序使用同树，计划簿记留主检出。review绑定commit，实施暂停写入，自检不替代独立复审；pass显式记录merge-owned门禁pending。

## 7. 验收标准

| ID | owner_stage | 可观察结果 / 验证 |
|---|---|---|
| AC-01 | work | A00–12合同和有限schema/codec范围齐全，身份/ref拒绝正确；V-01/02 |
| AC-02 | work | 两布局同值、小查询按需读取与预算达标，有实际内存/IO证据；V-03 |
| AC-03 | work | 真实fresh/resume/obj跨进程跨工程一致，隔离正确且不绕过verifier；V-01/02/04/05 |
| AC-04 | work | 强弱边/lease/并发/三点恢复/回收/空间/目录边界有区分控制；V-06–10 |
| AC-05 | work | 六场景基线、取舍、瓶颈与下一阶段建议可复验；V-11 |
| AC-06 | work | 改动/依赖守住边界，SD草案齐备，支持面与证据相符；V-12 |
| AC-07 | review | 独立核全部AC/SD/补充例，零未处理阻断，绑定commit、列明merge pending |
| AC-08 | merge | 冻结提交完整probe通过，Spec/索引沉淀、归档合入、guard清理有实际收据 |

## 8. 执行步骤

T-01只选择既定目标/预算内的实现；不可行留下最小反例回规划，不把“研究完成”替代实现。初始全部未完成。

| 状态 / ID | owner_stage | 依赖 | 操作与路径（未有者为新建） | 验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 执行授权 | 核树/基面；新增probe workspace，最小纵向实验；冻结codec范围、布局、依赖、发布协议；新docs/reports/plan783/feasibility.md | 二进制HIR恢复verify/native与对象重启link/run，重放oracle；AC-01–04 `[✅ 已完成]` 基面重核=ec934b000(指纹全匹配)；独立 workspace+lock 离线解析；四 oracle 纵线/对象缺源重启/二进制 HIR 恢复全过(native_resume_ 3/3)；冻结决策 8 项见 feasibility.md §1 |
| [x] T-02 | work | T-01 | 新src/{schema,identity}.rs、fixtures 及contract报告；A00–12登记与实际子集codec | identity_、admission_；AC-01 `[✅ 已完成]` A00–12 登记+合法/非法样例(lib 测试)；域分离哈希/三分身份/引用四类拒绝；HIR codec(core+scalar)解码全边界校验；identity_ 4/4、admission_ 6/6；contract.md |
| [x] T-03 | work | T-02 | 新src/layout/，流式造数、两种view、分页/预算计量 | layout_；AC-02 `[✅ 已完成]` 128MiB×2 实测写入；两布局值与纯函数基线+Atom 文本一致；小查询 I/O≤128KiB(上限512KiB)；热缓存≤8MiB+淘汰>0+OS private bytes<32MiB；后缀版本共享>90% 页；layout_ 6/6(修复:payload 填充偏移计数) |
| [x] T-04 | work | T-02 | 新src/driver.rs、纵线集成测试，复用公开编译/link接口，阶段cache/来源重定位 | native_resume_、invalidation_、admission_；AC-03 `[✅ 已完成]` 四 oracle×(fresh/跨进程对象命中(源已删注入)/HIR 恢复重验)全过；失效矩阵 body-only/profile/target(受控拒绝零产物)/entry/toolchain 全钉；native_resume_ 3/3、invalidation_ 5/5 |
| [x] T-05 | work | T-02、04 | 新src/store/，隔离adapter/refs/lease/发布/回收/故障子进程 | reachability_、concurrency_、recovery_、gc_budget_、store_boundary_；AC-04 `[✅ 已完成]` BlobStore 适配+chunk 锁;三故障点 kill+幂等恢复；lease OS 真值判活(TTL 过期仍活/进程死即收)；同键冲突报告非 LWW；reader-GC 门锁无半读；共享块删工程不破坏他工程；对象 provenance 回收后仍链接；水位/硬 pin 退让/staging 上限；reparse 拒跳+哨兵不变；reachability_ 4/4、concurrency_ 3/3、recovery_ 4/4、gc_budget_ 4/4(+512 churn closeout)、store_boundary_ 5/5 |
| [x] T-06 | work | T-03–05 | 新verify脚本及docs/reports/plan783/结果、性能报告，矩阵聚合拒绝缺例 | acceptance、perf_，V-01–12当前证据；AC-01–05 `[✅ 已完成]` verify-ac-artifact-probe.ps1(acceptance/closeout)；30样本×6场景 perf.json+perf-report.md(P95<400ms,语义 30/30×6,GC 并发尾延迟诚实记录)；perf_ 2/2 |
| [x] T-07 | work | T-06 | 最终取舍/迁移差距，新attachments/783/spec-delta.md，更新计划证据及债务候选 | 依赖/范围/SD追踪；AC-06；execution_done=7/9 `[✅ 已完成]` spec-delta.md(SD-01..05 逐条落地文本)；dep_audit_ 5/5(ac-core 零产品依赖/探针独立/无生产入口切换)；feasibility.md §3 实测发现+§4 边界 |
| [ ] T-08 | review | T-07 | 同树独立复审全部AC/SD及遗漏/延期/旁路，绑定commit | AC-07，pass→reviewed=8/9，merge-owned pending |
| [ ] T-09 | merge | T-08 pass | 冻结commit完整probe closeout；按实际diff判门禁、沉淀SD/索引、归档合入；按技能判批量回归到期；wt-guard后清理 | AC-08，实际收尾完成才9/9 |

根crates/编译器源变更若获修订授权，必须重判门禁，不能沿用probe-only豁免。门禁失败导致代码变化时回work/review，再在新冻结提交执行受影响门禁。

## 9. 复审记录

2026-10-15:`stage: work` 完成,`PLAN-783 r1`,`outcome: pass(工作阶段,execution_done 7/9)`,`worktree D:/autostack/.wt/lang-783/auto-lang`,`branch plan-783-dev`,`base ec934b000`,`code 62d93dae8`,`deps: ac-core(master 内置,指纹复核一致)/auto-cache(同)`。**验收收据**:acceptance 脚本整跑 PASS(13 族:lib 14 + 集成 62 = 76 例全绿,0 skip;`/tmp/ac783-acceptance.log` 同轮);512 churn 通过(210s,断言含 iterations=512/GC≥60 次/final≤48MiB/peak<128MiB);30样本×6场景 perf 报告提交于 docs/reports/plan783/(P95<400ms,语义 30/30×6)。**修复过程钉面的不变式**:A10 回执内容=链接输入纯函数(provenance 走图边);摘要内容不含 span;hit 消费方注册当前版本根(否则只恢复的消费者失去保活);回执 chunk id 用框架字节哈希(与存储 CAS 一致);GC 级联删行+死行可替换;LockFileEx 33/32 都映射为忙;GC low=0 禁用水位语义(测试全回收)。**开放项**:无合同内未闭合 finding;merge-owned 门禁 pending(closeout 完整 probe=512 churn+30 样本 perf 再生+按实际 diff 判根门禁;Category A 预期零根 cargo 档)。债务候选 D-783-A..E 已登记 KNOWN-DEBT。`next_responsible_role: review(T-08)`。

2026-10-15:`stage: work` 进入。用户下达执行指令;基面重核:master 已前进至 `ec934b000`(PLAN-780 合入,plan 簿记+project-model 代码),规划输入全部源码指纹复核不变(parser/adapter/descriptor/verify/native/link/storage 等 SHA-256 与 planning-inputs.tsv 一致),782 仍 executing 未合入,不影响本探针接缝。worktree `D:/autostack/.wt/lang-783/auto-lang`、分支 `plan-783-dev` 已建,基线 `ec934b000`。

2026-10-15：`stage: new`，`PLAN-783 r1`，`outcome: pass`（草案契约预检，非实现/独立验收），`next: work`。当前只授权规划，交付后等待执行指令。新增T-01–09、AC-01–08，所有实施/验收未执行。细节调查归T-01，依赖任务等待真实纵向可行性收据。

见 [planning-preflight.md](attachments/783/planning-preflight.md)。782非硬前置；其修复状态也不作为本probe可行性的证据。

## 10. 待澄清事项

无阻断起草的用户决策。T-01待冻结有限HIR字段、页目录、DB/BlobStore adapter与跨进程协调、机器内存/物理占用测量。只允许在既定合同内选择；修改验收、预算、信任或实现范围须回规划修订。

未来parser解释/生成策略、Batom正式格式、daemon/独立模式、pack压缩、平台ABI及runtime pin均由后续计划决策。本计划不预报生产技术定案。
