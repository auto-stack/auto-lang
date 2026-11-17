---
plan_id: PLAN-1030
status: execution_done
feature_name: ACE 864历史源码会话失败重试一致性
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 3
current_step: 3
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-av/collections-variants.md, docs/specs/auto-av/resident-online-reload.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1030] ACE 864历史源码会话失败重试一致性

## 0. 变更摘要

2026-11-17 实施（work，/auto-plan-goal 授权整链）：T-01..T-03 完成，执行中 3/6。外层 `CollectionsSourceSession`
的每函数指纹表改为**内层接受后换入**的事务（warmup/edit 候选副本 → `apply_revision` 成功才换入），失败保旧
fps/revision/`Arc`，同一失败源重试不再被判为 carry。红/绿证据、门禁与耗时见 §9；T-04/T-05/T-06 待
review/merge。

2026-11-17路线状态联络：本包仍drafting r3、0/6，尚无登记的实施树；本次只更新状态与排程，不推进Task/AC。当前位置为**第2：B历史外层事务修复（保P1）**；总表见[1024当前路线状态](archive/1024-ace-m2-code-quality-and-refactoring-audit.md#quality-route-20261117)。9个同路线包已交付归档，不再照2026-11-13旧队列重复开工；实施合同/Case/独立oracle保持。

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](archive/1024-ace-m2-code-quality-and-refactoring-audit.md)的F06，优先级**P1（历史测试入口，生产紧急度低于同级生产风险）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：本包归B，不作为985/A整包前置；原6任务/0完成、Case和独立关闭条件保持，当前仅规划。

## 1. 目标

外层fps与内层接受版本在warmup/edit成功时一起前进；失败保旧，同一请求重试不得错误carry。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。只承接仓内历史864外层消费者；不把删除函数当原支持义务，不重写AV全部session。

## 2. 架构方案

candidate fps在inner成功后接受，保真实生产计数；Debug指纹是否替换单独决定，不借修复增加拓扑删除或升为产品主session。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1030/auto-lang，建议分支codex/plan-1030-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1030/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/av/src/collections_864_session.rs](../../experimental/ace/crates/av/src/collections_864_session.rs) | 45f0e1f339ff5532a75768a9c6adb82c81435d524d895c934365e167ed003def | 静态取材，非运行证据 |
| [experimental/ace/crates/av/src/collections_session.rs](../../experimental/ace/crates/av/src/collections_session.rs) | 05b28dc216512735a3768b493909fea90ff5c42b2f48f0ce98f20166e115b28f | 静态取材，非运行证据 |
| [experimental/ace/crates/av/tests/cases/av_collections_session.rs](../../experimental/ace/crates/av/tests/cases/av_collections_session.rs) | bd116666026a7a4cfb6bbb2a8fe33e4c4ec3bf407b3bb5a0a0845020c31fa2b2 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-av/collections-variants.md](../../docs/specs/auto-av/collections-variants.md) | 5801fc0d16d7ee4099fdbf6e33fd995e65e74793f6f7d33a1cdb6c552df1cd1b | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-av/resident-online-reload.md](../../docs/specs/auto-av/resident-online-reload.md) | 53f21686212ee12b8d0722254db2387c0b14ae35aa77c9cce6c597e7ae0e6fd1 | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### 开工重核（2026-11-17 work，实施期来源/漂移）

- 实施树=`D:/autostack/.wt/lang-1030/auto-lang`，分支`plan-1030-dev`，基面`5b5940d4f`（创建时 master；
  其后 master 仅新增 docs/plan 提交`7430b0c78`，无代码漂移）。材料唯一`D:/autostack/plan-artifacts/plan-1030/`
  （owner=PLAN-1030，用途=红/绿日志与门禁产物，merge 收尾删除）。
- 三份钉源**无漂移**（work 前哈希=§4 表）：`collections_864_session.rs` 45f0e1f3、`collections_session.rs`
  05b28dc2、`av_collections_session.rs` bd116666。
- Spec 漂移 1 处：`docs/specs/auto-av/collections-variants.md` 钉 5801fc0d，实际 1a835a09——PLAN-1035
  （提交 d2847bf4c，2026-11-16）落地 SD-01..07 分类派生增量。已重读：支持域/指令面/`av.trap.list-index`
  SEM-02 规则不变，本 Plan 规则与之相容。`resident-online-reload.md` 仍 53f21686（无漂移）。
- 消费者重扫（仓内）：`CollectionsSourceSession` 仅 `av_collections_session.rs` 三例 + `lib.rs` 导出 + 文档引用，
  与 §5 A/B 裁定一致；无 auto-ui/auto-man/auto-web/auto-os 生产调用。
- 主线 WIP：主检出另有他 Plan 的未提交改动（含 `experimental/ace/crates/a2x-worker/tests/cases/sched_m2_968.rs`
  一处代码路径 + 60 项文档/计划），非本 Plan 所有，本会话一律未触碰、未纳入任何提交。

### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1失败源和重试结果未固定，容易用早parse错误测试晚失败；私有fps与inner接受态的观察方式也未指定。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/av/tests/cases/av_collections_session.rs | bd116666026a7a4cfb6bbb2a8fe33e4c4ec3bf407b3bb5a0a0845020c31fa2b2 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F06。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：外层fps与内层接受版本在warmup/edit成功时一起前进；失败保旧，同一请求重试不得错误carry。 candidate fps在inner成功后接受，保真实生产计数；Debug指纹是否替换单独决定，不借修复增加拓扑删除或升为产品主session。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** 复用av_collections_session.rs的BASE及CollectionsSourceSession::new(None,"d_main")。原BASE只有一个Item，helper=5，main返回code1+helper5=6。B只将xs.get(0).code改为xs.get(1).code：必须先通过编译/准入，再在真实执行到ListIndex失败。若当前前门先拒该具体写法，T-01选择同profile等价动态索引构造，证明是run阶段而非parse错误。

**状态、故障与责任。** 在collections_864_session.rs内部cfg(test)核私有fps和inner.snapshot()/unit()，生产API不暴露测试状态。持久化故障落produce_unit写入点，先用现store配额/错误返回；若没有可控入口，T-01只准设计一个测试私有的一次性failpoint，不删该路径也不靠磁盘随机拒绝。原fresh/body/hit/helper/interface三测试保留；不新增拓扑删除支持。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### T-01 实施期决定与反例（2026-11-17，已运行）

**A/B/C 可区分且 B 固定在执行阶段。** 复用 §6 输入：`warmup(A)` → rev1/value6；`edit(B)` 返回
`Err("revision did not execute: Trap { kind: ListIndex, span: Some(SpanInfo { func: 1, start: 146, end: 160 }) }")`
——前端准入、verify/encode/admit 全部通过，失败发生在 `apply_revision` 的 link+execute 段（SEM-02 唯一索引
陷阱码）。同一轮探针复现旧故障：`edit(B)` 重试返回 `Ok(revision:2, value:6, changed0/carried2, production 全0)`
（被误判为纯 hit，把 A 的代码对象当 B 交付）；`warmup(helper6+get(1))` 失败后再 `edit(helper6)` 返回 6 而非 7。
该探针输出即红证据，永久化后见 §6/§9。

**状态观测方式（生产 API 不扩）。** 私有 `fps`、`inner.snapshot().revision()`、`inner.unit("d_helper"/"d_main")`
的 `Arc` 身份只在 `collections_864_session.rs` 内 `#[cfg(test)] mod audit1030` 读取；`collections_session.rs`
零改动（内层 build-aside/原子提交本身正确，问题只在外层）。

**持久化故障入口（用现 store 错误返回，不造 failpoint）。** store root 上放一个同名普通文件 →
`ManagedStore::open` 的 `create_dir_all` 失败；已存在 store 则改 `AC783-STORE.marker` 的 `version` →
`open` 回 `NotManaged`。`produce_unit` 的 `store` 段因此返回 `open store: ...`，清故障（删文件/回写原
marker 字节）即可恢复——真实错误路径，非磁盘随机拒绝。

**修约（等价局部机制，不扩范围）。** T-01 实测：**任何**改动都会改变每个函数的 MIR（加一个未用函数也会重提交
`d_main`），而 store 的 A22 查询键不含 body 身份，因此“已有 store + 重提交已持久化单元”一律被
`key conflict ... (uncertainty reported, not overwritten)` 拒绝——与注入故障无关（清故障后同一 `BODY_EDIT`
仍报同一冲突，证据 `plan-artifacts/plan-1030/persist-keyconflict.log`）。故 AVTX-02 的“清故障后同一输入成功”
腿改在**冷准入**上取（冷 `warmup` 首persist 无既有对象 → 不冲突），另以“已接受 A + 迟到持久化故障”腿覆盖
拒绝/保旧/重试再拒。store 键缺 body 身份属**另一缺陷**，本 Plan 不扩面（公共身份/ABI 变化须具名设计），
已列入 §10 交 review 处置。

**指纹口径。** 保留现 `format!("{u:?}")` 的 FNV 内容指纹（§2 所述“Debug 指纹是否替换”为独立决定）；本次只改
换入时机，不动身份轴，V-04 计数语义不变。

### A/B裁定：F06历史864外层后移B

当前源码静态扫描显示CollectionsSourceSession仅在av/tests/cases/av_collections_session.rs三个原测试消费，av/src/lib.rs导出该历史模块；auto-ui/auto-man/auto-web/auto-os现有生产路径未找到调用。它不是980/981当前通用AV/session执行前门。本轮将AVTX-01/02修复与原T-01..06放M2-B历史兼容修复，不因标题含AV就阻A，也不把它冒已修/已退役。原三个普通回归/公开兼容事实保留，不按本次阶段划分停用。

A由985 T-A03核此调用/recipe闭包与真正A session（MIN-A、970）失败恢复证据，避免用864绿替当前Session。如果A消费者/导入/recipe今后真实进入此外层，需在A出口前提升其实际缺口，不能沿本次静态边界豁免。B按原T-01建运行期ListIndex/produce_unit失败控制、T-02/03修旧fps与inner一致性、T-04独立审、T-05/06正式交/收尾，AVTX-01/02旧值与同批修复重试全部闭合；最晚985/B候选核旧消费者时有正式修复或另经正式义务替代审查的退役来源，不能只删测试。


### 路线排程与当前进入条件（2026-11-17）

本包保持P1但属于B历史测试consumer修复；默认在1037后安排，不阻1037/985-A，亦不硬依赖1037整包。T-01先重核当前864外层是否仍在仓内/真实A消费链及当前CollectionsSourceSession原入口；原A→失败B→同批重试B/合法邻例不减。若最新A实际消费该缺口，按985已定规则提升受影响义务，不沿B标签放行。1038只读盘点可并行，不与此包并发写同一实现。

本轮只做计划簿记；原T-01..06、AC-01..06、Case及现阶段分类不变，plan_revision仍r3。正式实施从当时master建立本包唯一树并核源漂移/可用consumer，不凭本状态快照跳过T-01。Windows不运行Linux/WSL腿且不记PASS，Linux义务保留给专用环境；常规正确性与显式ace-perf分开。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-av/collections-variants.md | 现规则/支持域保持；补充以下修复后的明确合同：外层源码指纹与内层接受态必须同一成功边界切换；执行失败后的相同坏请求不能carry旧成功。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-av/resident-online-reload.md | 现规则/支持域保持；补充以下修复后的明确合同：明确历史source wrapper支持域与回归入口，不把修复写成M2全会话/拓扑删除支持。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-av。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| AVTX-01 | AC-02 / T-02 | 已接受A→合法可编译B但执行trap→重试B/修复B | CollectionsSourceSession::warmup/edit→CollectionsUpdateSession::apply_revision | 失败后fps/root/inner版本/旧结果仍A；重试不被误判hit | 已实施/已运行（绿；修前红，见§9） |
| AVTX-02 | AC-02 / T-02 | 持久化失败/原warmup失败，正常body-edit/hit/helper/interface | 同历史入口及CollectionsUpdateSession | 原值/身份保全，清故障后可成功；原三成功测试仍保留 | 已实施/已运行（绿；修前红，见§9） |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

```auto
type Item { name Str; code int }
fn helper() int { return 5 }
fn main() int {
    var xs = List<Item>.new()
    xs.push(Item("ab", 1))
    return xs.get(0).code + helper()
}
```
A=上源，独立结果6；B仅改get(1)，两次连续edit(B)都必须返回受控执行失败。修复C改回get(0)，结果6。旧故障是第一次失败后fps错误接受B，第二次B可能carry A而成功返回6；所以“重试成功”不是B的关闭预期。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| AVTX-01 | warmup A→edit B失败→同B重试→C→C hit | 两个B都失败，接受fps/inner snapshot revision及Arc身份保A；C成功6且不夹带B |
| AVTX-02 | 已有A后warmup B失败；持久化失败后同批重试；原三成功例 | 失败保旧，去故障后正常B'才成功；attempt计数可增，成功接受计数不虚增 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| AVTX-01 | audit1030_avtx_01 | [experimental/ace/crates/av/tests/cases/av_collections_session.rs](../../experimental/ace/crates/av/tests/cases/av_collections_session.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| AVTX-02 | audit1030_avtx_02 | [experimental/ace/crates/av/src/collections_864_session.rs](../../experimental/ace/crates/av/src/collections_864_session.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

### 实施结果（2026-11-17 work，实际运行）

永久测试（全部在既有入口，无新 test 文件、无新 Cargo target）：

| 锚点 | 文件 | 断言面 |
|---|---|---|
| `audit1030_avtx_01` | `av/tests/cases/av_collections_session.rs`（`conformance` 目标） | 公开 API：B 首拒/重试再拒均含 `revision did not execute`+`ListIndex`；C=A 为 `changed0/carried2` 的纯 carry 且值 6；随后真实 body 编辑 `1/1`、rev3、`production.verified==1` |
| `audit1030_avtx_02` | `av/src/collections_864_session.rs` `#[cfg(test)] mod audit1030` | 私有 `fps` 与内层 `snapshot().revision()`/`unit()`：被拒后 fps==A、rev 不变、`Arc` 指针同一；重试再拒；接受后 fps 前进且新 hit 成立；冷失败 `fps.is_empty()` |
| `audit1030_avtx_02_warmup` | 同上 | 已接受 A + 整体 warmup 被拒 → fps/`Arc` 保 A；随后 `edit(helper6)` 必须 `changed1/carried1`、值 7（修前 6） |
| `audit1030_avtx_02_persist` | 同上 | 冷准入持久化故障→空态/重试再拒/清故障同源成功（`store_writes=2`）；已接受 A + 迟到持久化故障→fps/rev/`Arc` 保旧、attempt `admitted=1/store_writes=0`、重试再拒 |

原三成功例（`session_fresh_then_body_edit_rebuilds_one` / `session_helper_edit_invalidates_consumer` /
`session_interface_edit_invalidates_consumers`）未改一字，门禁中全部仍绿。

| 本次实际运行 scope | 结果与计数 | 准备/构建 | 发现 | 执行 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| `cargo t ace-av`（worktree，nextest，`--features ace-av/test-case-batch`，表达式 `all() and not performance::`，另按组配置排除 `tests::aavm2_t3::test_aavm2_t3_tower_milestone`） | PASS，executed=1348（ok=1348，failed=0），`not_run=0`，`native_unselected=0`，`errors=0` | 975.7 s（13 个工具 release/预构建：av 242.8、case-parent 213.4、abi 131.1、ac 68.3 等；`nextest list` 36.6） | 含在准备内（同一次 list） | 60.7 s | 未单列（nextest 自带） | **774.6 s**（`results.json.wall_seconds`；与准备重叠计时按驱动口径） |
| 定向：`cargo test -p ace-av --lib audit1030` / `--test conformance av_collections_session::` | 3 + 4 passed，0 failed | 增量 5.8 s（sccache 命中） | — | 0.16 s / 0.06 s | — | 约 12 s |
| 红相（修前同测试） | 3 FAILED（fps 断言）+ 1 FAILED（重试被误判 hit） | 增量 | — | 0.09 s / 0.03 s | — | 同上 |

| 新增用例 | 测的行为与 oracle | Profile | 实测耗时 | 结果 / ≤10s |
|---|---|---|---|---|
| `audit1030_avtx_01` | B 执行陷阱两次拒绝、C=A 纯 carry 值 6、后续编辑 1/1 值 6 rev3 | debug/test | 0.0775 s | pass |
| `audit1030_avtx_02` | fps/rev/`Arc` 保旧 + 重试再拒 + 接受后 hit | debug/test | 0.0473 s | pass |
| `audit1030_avtx_02_warmup` | 被拒 warmup 不污染 fps，后续 helper 编辑值 7 | debug/test | 0.0342 s | pass |
| `audit1030_avtx_02_persist` | 冷/迟到持久化故障的拒绝与恢复（含 store 写入计数） | debug/test | 0.2579 s | pass |

机器占用检查（test-discipline 要求）：统一门禁前两次有界等待（45 s+60 s）期间观察到他 Plan 在用同一机器编译/测试
——`lang-1067` 的 `cargo t ace-native` 与 `--profile runtime-release` 构建、`lang-1066` 的 `cargo t vm corpus`、
主检出 `cargo t ace-facade auto-playground`/`cargo test -p auto-lsp`、`ui-088` 产品验证；等不到空闲后以
`CARGO_BUILD_JOBS=6` 限流开跑，本 Plan 各目标目录独立、无锁冲突，计时仅受竞争影响（不改变判定）。


## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。（2026-11-17：三钉源无漂移、Spec 1处漂移已重读；B 固定在 `revision did not execute / ListIndex`；观测面=私有 cfg(test)；持久化故障=现 store 错误返回。见 §5「T-01 实施期决定与反例」）
- [x] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。（4 个永久锚点全部落地并红→绿；AVTX-02 持久化恢复腿按 §5 修约移至冷准入，附 `persist-keyconflict.log` 反证；原三例未改）
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。（消费者重扫=仅三例；`cargo t ace-av` 1348/1348；SD-01/02 + 归属文档 C8 已在实施树备好待 merge 沉淀；G1002 分流不变）
- [ ] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 使A/B/C在实际compile_units→apply_revision路径可区分，固定B到执行的证据、snapshot与fps断言及持久化故障注入点。 AC-01 |【2026-11-17】钉源/消费者重核、B=执行期 `ListIndex`、观测面与 store 故障入口已定（§5） |
| T-02 / work | [x] T-01决定与可区分输入 | 外层candidate fps在inner成功后接受，覆盖edit与warmup；补AVTX-01/02私有及原集成测试，不改内层正确的build-aside。 AC-02 |【2026-11-17】warmup/edit 改候选副本+成功后换入；新增 1 集成 + 3 私有永久测试；`collections_session.rs` 零改动；提交 `3ce8628b8` |
| T-03 / work | [x] T-02 | 运行失败→失败重试→修复→hit序列、原三成功例与持久化恢复；核旧snapshot可执行、Arc/身份及成功/attempt计数分账。 AC-02/03 |【2026-11-17】红相 4 FAILED（修前）→绿；`cargo t ace-av` PASS 1348/1348；新例 0.03–0.26 s；见 §6/§9 |
| T-04 / review | [ ] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1失败源和重试结果未固定，容易用早parse错误测试晚失败；私有fps与inner接受态的观察方式也未指定。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1030 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | B承接及A边界核验明确；6原任务/勾选/0完成保留，不取消原case/角色，不提前签修复/研究完成 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。


2026-11-17 stage:planning-status-sync | scope:1024→1027..1038修复优化路线 | source_HEAD:e85096628c468376c0c3632d832420a2e7f6d439 | 方法：逐Plan元数据/末review-merge清理收据、九个正式交付提交祖先及worktree登记只读核对 | 结论：9包交付归档、1030/1037/1038仍drafting0/6；当前建议1037→1030→1038，1024 T-05文档收尾独立；1038只读研究可提前，无新增整包硬依赖。1034头部3/6与终态收据差额按收据解释，不重开归档。原Task/AC/Case/接受范围及进度保持；未建树、修改实现/测试/配置/canonical Spec、运行ACE/构建/性能或提交/合入/清理。本轮仅状态/导航簿记，不签新的实现/独立终审PASS。


2026-11-17本轮状态同步静态核对通过：九个交付提交祖先、17份来源Plan（本轮四份计划变更前冻结；其余13份保持字节不变）、12包状态9已交/3未开工、原35项发现分配、三草案18任务/18AC/原Case、章节/围栏/新增链接与quality-route锚点一致。只修改五份计划/路线文档；未改归档/实现、未运行ACE或构建/性能；此检查不是新产品验收。

`stage: work | plan_id: PLAN-1030 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1030/auto-lang | branch: plan-1030-dev | base_commit: 5b5940d4f | code_commit: 3ce8628b8（实现+测试）/ 025298412（Spec 准备） | dependency_revisions: 无跨仓依赖（仅语言仓；ace workspace 内部 path 依赖不变） | task_ids: T-01,T-02,T-03 | evidence: 见下 1–7 | blockers: 无 | next: review`

2026-11-17 证据（命令、退出码、关键数字）：

1. **反例界定（探针，修前）**：`warmup(A)`=rev1/value6；`edit(B)`→`Err("revision did not execute: Trap { kind:
   ListIndex, span: Some(SpanInfo { func: 1, start: 146, end: 160 }) }")`；**重试 `edit(B)`→`Ok(rev2,value6,
   changed0/carried2, production 全0)`**；`warmup(helper6+get(1))` 失败后 `edit(helper6)`→**6（应为 7）**。
   即旧故障=外层 fps 提前接受，重试按 hit 交付旧代码对象。
2. **红相（修复前跑永久测试）**：`cargo test -p ace-av --lib audit1030` → 3 FAILED（fps 断言 left 含
   `17525033987230416544`/`2707126400848338093`，right 为 A 的 `2861593873133751202`/`13090792784392237092`）；
   `cargo test -p ace-av --test conformance --features ace-av/test-case-batch audit1030_avtx_01` → FAILED，逃逸值
   `(revision: 2, value: Some(6), rebuilt_units: 0, production 全 0)`。日志：
   `plan-artifacts/plan-1030/red-lib-audit1030.log`、`red-integration-audit1030.log`。
3. **绿相**：`--lib audit1030` → 3 passed；`--test conformance av_collections_session::` → 4 passed（含原三例，
   未改一字）。新例耗时 0.0342 / 0.0473 / 0.2579 / 0.0775 s（均 ≤10 s）。
4. **固定组门**：`cargo t ace-av`（worktree，`--features ace-av/test-case-batch`）→ exit 0、`PASS: ace-av`、
   executed=1348 / ok=1348 / failed=0 / `not_run=0` / `native_unselected=0` / errors=0；wall 774.6 s（工具预构建
   975.7 s：av 242.8、case-parent 213.4、abi 131.1、ac 68.3 等；执行 60.7 s；发现 36.6 s）；results
   `source_state=0dd4504e85f204cb…`。四个新锚点均在 executed 列表且 ok。
5. **发现并修约**：store A22 查询键不含 body 身份 → 任何“重提交已持久化单元”的修订都被
   `key conflict ... (uncertainty reported, not overwritten)` 拒绝（清故障后同一 `BODY_EDIT` 仍冲突，
   `plan-artifacts/plan-1030/persist-keyconflict.log`）；加一个未用函数同样会重提交 `d_main`。故 AVTX-02
   持久化恢复腿下沉到冷准入（§5/§6），该 store 键缺陷本 Plan 不修，列 §10。
6. **健康**：只改 2 个源文件 + 3 份 Spec 文档；`rustfmt --check` 两文件通过；`cargo fmt -p ace-av` 顺带改动的
   7 个无关文件已备份 `plan-artifacts/plan-1030/fmt-unrelated-backup/` 后还原、未入提交；编译 warning 与基面同类
   （既有 `decode_abc`/`parse_header`/`triple`/`err` 命名等），本 Plan 未新增。
7. **主线保护**：主检出他人 60 项 WIP（含 `experimental/ace/crates/a2x-worker/tests/cases/sched_m2_968.rs`）
   未触碰；本会话未在主检出改代码/文档，只改本 Plan 文件。

保留给后续 owner：T-04 独立复审、T-05 合入+规范沉淀、T-06 输出对账/清树；`cargo t all` 周期批量不属本 Plan。

`stage: goal | plan_id: PLAN-1030 | plan_revision: r3 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1030/auto-lang | commit: 3ce8628b8 | next: review`

## 10. 待澄清事项

只承接仓内历史864外层消费者；不把删除函数当原支持义务，不重写AV全部session。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。

2026-11-17 work 发现（交 review 处置，不属本 Plan 范围）：`CollectionsSourceSession` 带 store 时，任何重提交
已持久化单元的修订都被 store 的 A22 查询键以 `key conflict ... (uncertainty reported, not overwritten)` 拒绝——
该键不含 body 身份（`EnvelopeIdentity.body_fp` 只在信封里），故该会话的持久化实质只能用于冷准入。修它需要改
公共身份/查询键，属具名设计与合同修约；本 Plan 只留证据（`persist-keyconflict.log`）并改用等价测试输入，未改动。
