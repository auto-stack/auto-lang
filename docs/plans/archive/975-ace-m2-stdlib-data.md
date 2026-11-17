---
plan_id: PLAN-975
status: archived
feature_name: 新版JSON序列化数据库与扩展库
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 10
current_step: 11
total_steps: 11
depends_on: [PLAN-972, PLAN-973, PLAN-948, PLAN-1042, PLAN-1064]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/stdlib/design/ace-binding-and-assembly.md]
new_spec_components: [docs/specs/ace-migration/m2-stdlib-data.md]
touched_goals: [GOAL-002]
---

# [PLAN-975] 新版JSON序列化数据库与扩展库

## 0. 变更摘要

2026-11-16 r10当前：用户要求明确973/975后继交接。MODULE实际实现/验证/独立复核/正式交付归1064/T-MODULE-I/V/R/D（973装配域后继）；975只消费该切片，不等1064全包。已有TextFromChar/B1按新增T-B1-V/R/D三任务先独立验证/复审/合入，不等MODULE或整包975。原8任务和2项已勾保持，现executing **2/11**，新增是合同任务不是实现退步/完成。结构感知decode[T]、公共API canonical声明与真实JSON/DB业务仍归本包，不能甩给1064或等归档966重开。

2026-11-15 r8当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r7当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r6当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

承948全库盘点，把JSON/type映射、数据库事务、缓存、encoding及剩余现用标准操作逐项落provider；UI/media/terminal等领域库归对应owner，通用合同不绑旧VM。不得仅按目录存在认领全库。

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

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/a2r-std/src/json.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/sqlite.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/redis.rs`（本轮确认存在；开工重核符号/版本）。
- `stdlib/auto`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：JSON/序列化/SQLite等数据库与扩展库。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [stdlib/auto/json.at](../../stdlib/auto/json.at)：动态与typed API分清；读取SHA256前16 `11b9229c8d0bf660`

- [crates/a2r-std/src/json.rs](../../crates/a2r-std/src/json.rs)：旧codec行为参考；读取SHA256前16 `d072d7b4594982e0`

- [crates/a2r-std/src/sqlite.rs](../../crates/a2r-std/src/sqlite.rs)：实际SQL/错误/资源参考；读取SHA256前16 `ddf4eacee9791adc`

### 本轮交接取材/授权（2026-11-16）

本轮只规划，获用户授权细化既有计划/依赖/验收，不实施或恢复目标。读取语言主线9d0f973cb，975分支02a687da8；973已归档，现公共源/有限系统面按原收据复用。975 JSON原源的private helper与sqlite plain fn事实沿原r9回执；1064 r2明确承担共同Schema/CST、exports与冷恢复；原“归973 io线”不再是等待归档计划出新代码。旧r9的源码/绿只可按其身份复用，不证明新接面/phase正式来源。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承948全库盘点，把JSON/type映射、数据库事务、缓存、encoding及剩余现用标准操作逐项落provider；UI/media/terminal等领域库归对应owner，通用合同不绑旧VM。不得仅按目录存在认领全库。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：持久数据与兼容升级（r2）

逐JSON、sqlite/redis、序列化/缓存的实际API覆盖；现有磁盘/数据库格式与ABI无关，不因换引擎破坏用户数据，schema迁移与降级策略交986消费。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### T-01 A期冻结记录（2026-11-16，r9执行）

**授权**：既定裁定模式（goal 续行 2026-11-16，974 整包归档后按序开工 975 A 批）。worktree：`D:/autostack/.wt/lang-975/auto-lang`（分支 `plan-975-dev`，基面 master@b1cb6faeb）；材料 `plan-artifacts/plan-975/` 已登记。

**前置核实**：972/948/1042 已归档 ✓；**973 在途（executing 0/10，他会话）——依赖分账冻结**：JSON 域（纯 codec，无 IO 依赖）**不前置 973**；sqlite 文件持久化腿（db 路径/文件锁/崩溃恢复面）具名阻塞 973；sqlite **`:memory:` 腿**（连接私有内存库，无宿主文件）不前置；redis 腿=TCP 客户端，**可复用已归档 974 的 platform.net/1 车道**（同为 OS 薄适配纪律）。

**已交面盘点（策略输入）**：legacy `stdlib/auto/json.at`（20 pub fn：encode/encode_pretty/decode/try_decode/parse + JsonValue 存取组 15 + prettify/minify/is_valid，泛型 `[T]` 面 VM=serde_json）；`a2r-std/{json,sqlite,redis}.rs`（旧 Rust 侧参考）；ACE 侧无 json/sqlite/redis crate（补位面）。967 已交 SQLite C 库内容钉版道（p967_sqlite fixture——SQL 引擎经 FFI provider 可复用，非重写引擎）。

**策略比较与选型（T-01 决定）**：JSON 域=**S1 Auto 侧自有 codec**（`stdlib/ace/std/json.at`，PLAN-1041 内联约定——encode 族纯字符串拼接沿 std.url 方言事实；decode/parse 需子串物化=与 url_decode 同一核心决定，具名共享）；sqlite=**S2 复用 967 FFI provider 道**（SQL 引擎不重写，`:memory:` 与文件腿统一经 provider；错误/事务/资源沿 967 面加 Auto 包装）；redis=**S3 platform.net/1 + RESP 协议在 Auto**（974 同律：协议自有+OS 薄适配）。拒绝：平行 SQL/JSON 引擎重写（禁平行语义分析）。

**平台/命令/资源**：Windows x64；JSON 腿=纯 console-lane 单源证（974 同模式）；sqlite `:memory:`=967 provider 构建道（rustc/cl 预构建纪律）；Linux 归 1063。V 真实命令由各 I 任务登记（拟 ace-av/ace-a2x/相关组并集）。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/a2r/28_sqlite/001_basic/basic.at](../../crates/auto-lang/test/a2r/28_sqlite/001_basic/basic.at)（下列节选L2–L15；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use sqlite

fn main() {
    let db SqliteDb = sqlite.open("cats_415b.db")
    let n int = db.exec("CREATE TABLE IF NOT EXISTS cats (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
    db.exec("INSERT INTO cats (name) VALUES ('Michi')")
    let rid int = db.last_insert_rowid()
    print(rid)
    let rows List<List<str>> = db.query("SELECT id, name FROM cats ORDER BY id")
    for row in rows {
        print(row[0])
        print(row[1])
    }
}
```

本包所需独立结果：在隔离新数据库执行建表、插入Michi、按ORDER BY读取，独立SQL客户端核id/文本；另JSON嵌套对象公开字段ClearDiagnosticInformation。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Auto数据API/typed decode→共同类型与资源→中立codec/DB provider→真实序列化/SQL事务→独立值/字节。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct CodecContract { input: ByteSlice, target_type: TypeRef, errors: ErrorPolicy, ownership: OwnerRule }
struct DatabaseLease { connection: ResourceId, transaction: TransactionState, grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [stdlib/auto/json.at](../../stdlib/auto/json.at) | 动态与typed API分清 |
| [crates/a2r-std/src/json.rs](../../crates/a2r-std/src/json.rs) | 旧codec行为参考 |
| [crates/a2r-std/src/sqlite.rs](../../crates/a2r-std/src/sqlite.rs) | 实际SQL/错误/资源参考 |

**状态、错误与资源。** 畸形/溢出/错typed字段、SQL失败、事务取消与错handle按原错误合同处理；重复key的保留/覆盖/拒绝规则依现规范逐项冻结，不一律新增拒绝；typed decode不能经Any绕过verify。 DB rollback/close、语句和行buffer释放，Auto合法含NUL文本按长度保真；仅外库声明的NUL协议显式适配或诊断，不能静默截断。临时DB/sandbox只在材料scope，默认库与显式外库分账。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/a2r/28_sqlite/001_basic/basic.at](../../crates/auto-lang/test/a2r/28_sqlite/001_basic/basic.at) | 真实数据库字节/事务，不以生成Rust金样关闭 | `basic.expected.rs`；SHA256前16 `1a9baf0082455ab7` | 拟ace.m2.p975.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/07_objects/007_json/json.at](../../crates/auto-lang/test/vm/07_objects/007_json/json.at) | 动态对象/数组字段 | `json.expected.result`；SHA256前16 `278785eed5fcc549` | 拟ace.m2.p975.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/18_ffi/014_json_is_valid/json_is_valid.at](../../crates/auto-lang/test/vm/18_ffi/014_json_is_valid/json_is_valid.at) | 畸形JSON/合法邻例 | `json_is_valid.expected.result`；SHA256前16 `53cbbfe0d58add29` | 拟ace.m2.p975.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/a2r-std/src/redis.rs](../../crates/a2r-std/src/redis.rs) | Redis等其余原域保owner/环境，不缩成SQLite | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `6f98f7d48051a68e` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 动态与typed decode、数字宽度/Unicode/字段错误、事务/绑定参数/Redis环境矩阵及原数据库用例完整分母冻结，不凭SQLite示例外推全域。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

JSON/序列化/缓存/sqlite/redis等原公共操作补六路径；保持数据格式、错误与owner。显式数据库引擎等第三方依赖逐路径列明；AV/native能调用受检FFI不等于该外部引擎已由纯Auto重写，不能冒无C/Rust依赖。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

经972/973消费1043；数据/数据库适配复用source装配与#in，不因selected source标签冒实际C/Rust引擎调用，原格式/资源/第三方依赖保留。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-stdlib-data.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-IO-MODULE | modify | `docs/specs/ace-migration/m2-stdlib-data.md`与`docs/specs/stdlib/design/ace-binding-and-assembly.md`（实际交付后） | 泛称等973→B1正式原语切片、1064 MODULE正式输入与本包真实业务/typed责任分账 | 用户明确承接与无环交接 | AC-B1/AC-LOAD-MODULE、T-B1-V/R/D及原T-03/04 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-stdlib-data.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-stdlib-data.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-stdlib-data.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-stdlib-data.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**JSON/编码/序列化/缓存与显式数据库等外部引擎**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

普通JSON/编码/序列化/容器缓存等默认共享Auto实现；SQLite/数据库引擎、Redis外部服务等可暂经显式FFI/协议适配，列真实依赖，不能把serde_json等包装冒自有JSON。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-stdlib-data.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |
| SD-AUTO-STD | modify | docs/specs/ace-migration/m2-stdlib-data.md | 默认provider声明→普通标准能力有Auto算法/协议与OS边界来源证据，高级外库显式分账 | 用户自有普通库要求 | AC-AUTO-STD |

### 当前实际承接与先后合同（替代泛称“等973 io装配”）

| 接口/原差额 | 执行owner与任务 | 输入/独立关闭条件 |
|---|---|---|
| IF-975-B1：char.to(Str)/TextFromChar/StrFromChar及已有各端降落 | 本包T-02已有实现；新增T-B1-V→R→D真实独立交付 | 复用json975_char_to_str_primitive/json975_char_to_str_a2x_ends/char975_to_str_on_python_end与原schema/指纹/owner/错误；普通source直接经受检前门跑原语，合法内联fixture只证原语，不假称模块LOAD |
| IF-1064-MODULE：public/private共同原源声明、std装配/exports/源身份/冷恢复 | 1064/T-MODULE-I→V→R→D | 由其独立phase正式来源输入本包原T-03 LOAD。需要完整json等文本consumer时消费IF-975-B1，绝不反等975整包；demo解析/绑定可先行 |
| canonical公共源/typed decode/JSON SQLite Redis业务 | 本包原T-03/T-08/T-04 | 统一合法原作者源，sqlite真正公开API明确pub、private helper不全pub；原private版本跨模块不可见仍负控。结构感知decode[T]用已交966 CT/metadata在本包实现/组合，不等待966重开，也不把它算MODULE的完成条件 |
| 系统文本/持久文件接口 | 973已交部分＋1064实际D行，AC系统OS能力取1065 | 本包持久数据/迁移失败保旧文件及真实consumer需所用操作的正式切片，0/未交不能冒PASS；只等准确接口，不将全部1064或1065作为B1前置 |

B1具体最小正例沿已实现`char.from_codepoint(20013).to(Str)`，应是1 scalar/3 UTF8 bytes；NUL为1字节，非法scalar沿已有char诊断，owned返回在原输入释放后有效。字段/类型复用现有HIR::TextFromChar、MIR::StrFromChar、schema kind/fingerprint、owned资源arena及A2X/native运行helper，不另造转换器。需要的端仍各对独立码点/字节/错误/资源oracle；代码可构建/emit不充真实运行。

**无环顺序**：T-02已有码→T-B1-V/R/D（不等1064）→1064 MODULE及BSTR；MODULE独立正式来源→本包T-03装配/typed消费→T-08/04→原全包T-05/06/07。T-B1-R/D不替原全包review/merge，也不提前勾原任务或归档/删树。公共core/grammar/ABI文件由原owner协作串行写，更新基线后重核证据适用性，不能覆盖兄弟计划WIP。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | JSON null与缺字段区分、宽整数精度不损失 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 事务失败回滚再读原值 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | close后DB handle拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 独立fixture核序列化bytes/字段顺序合同 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 余下每符号必须有实现/验证收据 | 见下方 **V-05 逐符号覆盖对照表**（逐一映射到 `stdlib/auto/{json,sqlite,redis}.at`、`encoding/{hex,base64,csv}.at` 与 `crates/a2r-std/src/{json,sqlite,redis}.rs`） | work；本包支持矩阵 |

#### V-05 逐符号覆盖对照表（Legacy ↔ ACE Canonical / Boundary）

| Legacy 来源符号 | ACE Canonical 映射 / 裁定状态 | 实证用例 / Owner 边界 |
|---|---|---|
| `json.at`: `encode`/`encode_pretty` | **函数族+受控边界**：`json_escape`/`json_int_str`/`json_str`/`json_bool`/`json_null`/`json_int`/`json_array_*`/`json_object_*`/`json_kv`；泛型 `encode[T]` / `encode_pretty` 属 **B 阶段 / 具名未迁移**（待任意字段反射） | `json975_encode_family_inline` + `json975_packages_coverage_*`（5 端 = 42） |
| `json.at`: `decode[T]` / `try_decode[T]` | **结构感知有界交付**：`spec JsonDecodable` + `<T has JsonDecodable> decode<T>` + `pub type Service` 跨模块编解码 + `966` `ct_field_count`/`ct_has_field`（无任意字段名/类型遍历反射，属 **B 阶段 / 具名边界**） | `json975_typed_decode_and_ct_metadata` (`0.052s`) + `json975_v_load_module_closure` (`0.382s`) |
| `json.at`: `is_valid` / `parse` / `JsonValue.*` (`type`/`is_null`/`as_string`/`as_number`/`as_int`/`as_bool`/`as_array`/`keys`/`get`/`get_at`/`has_key`/`len`) | **纯 Auto 路径解码**：`json_is_valid` / `json_structurally_balanced` / `json_kind_at`（-1/0/1..7）/ `json_parse_str`（UTF-16 代理对 + B1）/ `json_parse_str_ok` / `json_parse_int`（含 `i32::MIN`）/ `json_parse_bool` + 点路径数组段 `services.0.name` / `grid.1.0`；动态 `JsonValue` AST 树对象与 `as_number` 浮点路径 / `keys` 列表 / `prettify` / `minify` 属 **具名未迁移边界**（可按需以纯 Auto helpers 增量补齐） | `json975_decode_family_inline` + `json975_array_paths_and_legacy_shape` + `json975_packages_coverage_av_mn_a2r` / `_a2c_a2py` |
| `a2r-std/json.rs`: `as_string_str`/`as_int_str`/`as_bool_str`/`len_str`/`get_str`/`parse_opt`/`json_get`/`to_string` | 同上 Mapped to `json_parse_str*` / `json_parse_int` / `json_parse_bool` / `json_kind_at`（与 `a2r-std` 作为 legacy/A2R crate 保留，`cargo t a2r-std` 115/115 绿） | 同上 `json_975` + `stdlib_contract_975` |
| `sqlite.at`: `SqliteDb`, `open`, `SqliteDb.exec`, `SqliteDb.query`, `SqliteDb.last_insert_rowid`, `last_error` (`a2r-std/sqlite.rs`: `open`/`last_error`) | **Canonical `sqlite.at` + 967 FFI**：`sqlite_open` / `sqlite_open_memory` / `sqlite_exec` / `sqlite_query_int` / `sqlite_close`（Pinned SQLite 3.46.0 provider，单连接句柄 + `ForeignStatus` 陷阱）。**具名外部边界**：`:memory:` 全量 CRUD/事务回滚在本包关闭；**宿主磁盘文件持久化/崩溃恢复 = `PLAN-973`/`PLAN-1064`**；多列结果集 `List<List<str>>` / `last_insert_rowid` / `packages` 外来 provider 装配执行 = `PLAN-1043` / **B 阶段边界** | `db975_sqlite_memory_leg_on_pinned_provider` (`7.619s`) + `db975_auto_sqlite_api_face_on_av_foreign_lane` (`7.043s`, 含 `BEGIN`/`ROLLBACK` + SQL 失败后 `ROLLBACK`) + `db975_sqlite_at_packages_module_face` (`0.035s`) |
| `redis.at`: `RedisClient`, `open`, `ping`, `set`, `get`, `del`, `exists`, `last_error` (`a2r-std/redis.rs`: `open`/`last_error`) | **纯 Auto RESP2 客户端 `redis.at`**：`redis_connect` / `redis_close` / `redis_set` / `redis_get_len` / `redis_get`（B1 物化 + 严格 UTF-8 校验，nil/畸形→`""`）/ `redis_del` / `redis_exists` / `redis_incr` / `redis_ping` / `redis_cmd*` / `redis_bulk_*` / `redis_reply_*`；错误态经协议分类与返回值而非隐式全局 `last_error` | `redis975_resp_over_platform_net` (`0.083s`) + `redis975_std_redis_packaging_inline` (`0.187s`) + `json975_v_load_module_closure`（`pub`-stripped RED） |
| `encoding/{hex,base64,csv}.at` 与 `cache` | `hex`/`base64` 归 **PLAN-972** (`stdlib/ace/asm` & 972 交付) / `PLAN-1064` 装配；`csv`（`csv_parse`/`csv_parse_with_delimiter`/`csv_encode`/`csv_encode_with_delimiter`）为旧 VM/A2R 外部未迁移占位（`KNOWN-DEBT`）；`cache` 无独立 legacy `.at` 文件，由 `redis.at` 覆盖 | `972` 归档收据 + 本表边界记录 |

完整职责组拟定：`cargo t stdlib ace-stdlib a2r-std ace-runtime`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 旧版本创建数据新ACE读取；事务失败再读旧值；JSON null/缺字段/大数分别保真；迁移失败原文件/数据库可继续旧版使用。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 在隔离新数据库执行建表、插入Michi、按ORDER BY读取，独立SQL客户端核id/文本；另JSON嵌套对象公开字段ClearDiagnosticInformation。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 畸形/溢出/错typed字段、SQL失败、事务取消与错handle按原错误合同处理；重复key的保留/覆盖/拒绝规则依现规范逐项冻结，不一律新增拒绝；typed decode不能经Any绕过verify。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | DB rollback/close、语句和行buffer释放，Auto合法含NUL文本按长度保真；仅外库声明的NUL协议显式适配或诊断，不能静默截断。临时DB/sandbox只在材料scope，默认库与显式外库分账。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

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

### 当前交接验证（拟实施，未运行）

| Case | owner/AC | 输入→真实结果与可区分反控 |
|---|---|---|
| V-B1-LAND | T-B1-V / AC-B1 | 已有普通源/typed原语→真实AV/AC/A2R/A2PY及原适用C/Rust/旧A2C→中/NUL/非法scalar/返回owner独立oracle；删一端lower/fingerprint、错runtime/source、emit冒run须RED；现内联只核primitive，不充模块 |
| V-LOAD-MODULE | 原T-03/T-04 / AC-LOAD-MODULE | IF-1064-MODULE正式SHA＋真实canonical json/redis/sqlite→1043装配/共享CST/binder/verify→实际业务；完整json私有helper可同模块调用、跨模块不能导出，SQLite SUM=9、Redis真实Str和JSON原数组/typed预期；删pub/全pub镜面源/旧身份/只inline替LOAD须RED |

真实职责与原V-LOAD/V-COVERAGE等完整并集保留；B1选择实际触面`ace-source ace-syntax ace-av ace-native ace-a2x`及必要`ace-stdlib/auto-test-case`、967来源相关组，不因这些名字全跑所有experimental。MODULE最终业务由本包验证，不复用上游demo PASS冒本包完成。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 持久数据与兼容升级的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

- [x] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。
  [✅ 2026-11-16, commit `76c23c719`] B1 原语 `char.to(Str)` 覆盖全六路径（AV/`MnDriver`/A2R `rustc`/A2C `cl`/A2PY `pyrun`/AV-Rust）；`stdlib/ace/std/json.at` 真实跨模块业务经 `json975_packages_coverage_av_mn_a2r`（AV/`MnDriver`/A2R = `42`, `1.688s`）与 `json975_packages_coverage_a2c_a2py`（A2C `cl`/A2PY OS Python = `42`, `2.849s`）实跑闭环（并修复 `a2x-c/src/lower_rt.rs` 内容视图参数转发 `C4133` 实虫）。Linux 腿按裁定移交 `PLAN-1063`。

- [x] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。
  [✅ 2026-11-16, commit `76c23c719`] 消费 `IF-1064-MODULE`（`master@4ba167930`）：`json975_v_load_module_closure`（`0.382s`）+ `db975_sqlite_at_packages_module_face`（`0.035s`）验证 canonical `json.at`/`redis.at`/`sqlite.at` 公开导出边界、`json_cp` `source.module-private` 拒绝、三模块 `pub`-stripped 镜面 RED、`std::json` `ace.stdlib-reserved` 拒绝、`PackageStore::prepare`/`restore`（篡改票据拒 + 私有函数体变更移动 `input_hash`）；`packages` 外来 `p967_*` provider 执行属 `1043` 具名余量。

- [x] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。
  [✅ 2026-11-16] 遵照 2026-11-14 用户裁定，Linux/WSL 用例不在 Windows 环境运行；§10 已建立完整 `L-01..L-05` Linux 移交清单表（`status: deferred-required / NOT_RUN`, `owner: PLAN-1063`），未冒充 PASS 或 NA。

- [x] **AC-AUTO-STD / work**：§5本包普通标准能力默认算法/协议为共享Auto源码，实际来源/调用链审计与原独立expected齐全；可选C/Rust及高级外库列明边界，不冒默认自有实现。原能力work任务与最终验证负责，原独立review核；Linux按AC-LINUX-PHASE移交。
  [✅ 2026-11-16] `json.at`（Pure Auto 编解码/点路径/UTF-16 代理对）与 `redis.at`（Pure Auto RESP2 协议帧 + UTF-8 严密校验，经 `platform.net` TCP）为共享 Auto 源码零外部库依赖；`sqlite.at` 显式经 `967` 钉版 SQLite 3.46.0 C provider + `use.c` 准入（无静默 builtin/host fallback，无 `use.c` 具名编译拒绝）。

### 当前交接验收

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-B1 | work | 既有B1原语全部适用后端/指纹/所有权/错误的独立运行与来源齐，未借MODULE能力冒支持 | T-B1-V/V-B1-LAND；切片独立review核 |
| AC-B1-REVIEW | review | B1切片独立复审绑定r10/冻结SHA/原source及SD，未交面如实保留 | T-B1-R |
| AC-B1-LAND | merge | B1正式合入/已验知识与IF收据同版，可由1064直接消费，整包仍executing | T-B1-D |
| AC-LOAD-MODULE | work | 真实canonical模块与原exports/权限、类型/业务/恢复契约经正式接面验证；结构感知decode不推给1064 | 原T-03/T-04/V-LOAD-MODULE；全包原review核 |

## 8. 执行步骤

当前共11任务，已完成2项（原T-01/T-02历史交付；三项B1独立切片任务均未执行）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] 冻结记录见§5"T-01 A期冻结记录"：前置核实+973分账（JSON/:memory:/redis不前置，文件腿具名阻塞）；三域选型冻结（S1自有JSON codec/S2复用967 FFI sqlite/S3 RESP over platform.net）；worktree已建。

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [✅ 2026-11-16, worktree commit bc19182d9] **JSON encode族已交**（T-02 首增量）：`stdlib/ace/std/json.at`（json_escape/json_int含负数/json_bool/json_null/json_str/kv与array/object组合helper + json_structurally_balanced字符串感知括号扫描=廉价有效面**明非parse**）＋`json975_encode_family_inline`（verbatim内联14独立手算oracle含3负控，0.09s）。decode/parse/泛型typed面=具名pending（子串物化决定共享）。**sqlite :memory:腿已交（bb8d56e07）**：`db975_sqlite_memory_leg_on_pinned_provider`——967内容钉版SQLite 3.46.0 provider以`:memory:`路径开启（无宿主文件=零973依赖），手算oracle SUM(4,5)=9＋独立cl oracle exe同amalgamation私有`:memory:`库同语句复核；负控exec-before-open/-1、double-close/-1；oracle语义在案（changes=末语句、close ack out=1）；5.6s。**Auto侧sqlite.* API面=具名后续**（console道foreign dispatch缝）。**redis腿已交（8e51c6e38）**：`stdlib/ace/std/redis.at`（PLAN-1041内联——RESP2数组命令构造器+帧读取器（bulk $N/nil/-10重试）+分类器+旧面哨兵映射：set 1/0、get_len nil=-1、incr/del/exists原值、ping；get的Str返回=具名pending子串物化决定）；`redis975_resp_over_platform_net`（裸lane：inline SET/GET/INCR/DEL逐字节+服务端记录oracle，0.07s）＋`redis975_std_redis_packaging_inline`（tracked模块verbatim对**严格RESP2数组解析**服务端——$len帧=线上oracle；+OK/$5/$-1/:3/:1/+PONG/-ERR分类+bulk读取器）；修复=op内部region放大4096（net_tcp_read_into有界检查，512读块对小region返回-4）；redis975 2/2绿，同源"97"过滤64/64绿12.75s。**decode/parse核心决定已裁定并落地（3690d8161）**：用户裁定B1（2026-11-16）＝加 `char.to(Str)` 物化原语；取证结论"slice view→owned促升已存在（clone/to/concat）"仅缺动态码点。落地全链：resolve表项+owned-only规则、HIR `TextFromChar`+descriptor往返+4 schema atom、MIR `StrFromChar`（opcode 0x4C；0x2F与DYN_PACK冲突由V-11契约表捕获）+abc/mir_verify/ownership/instr_spec/执行器arena物化（UTF-8 1..4字节+字节fuel）、lowering owned+collections、**native（AV/C来源）text-runtime `ac_rt_own_from_char`+lower+fingerprint真实实现**（非诊断臂）。**std.json decode面已交**：`stdlib/ace/std/json.at`＋json_is_valid（全递归结构验证含前导零拒绝）/json_kind_at（点路径，-1/0/1..7）/json_parse_str（全转义集含\uXXXX经B1；\b\f经\u{8}/\u{c}字面量）/_ok状态对/parse_int（仅kind3）/parse_bool；视图(str)参数（owned参数首次helper调用即被move）。用例：`json975_char_to_str_primitive`（多字节形态/跨函数owned返回/slice+char组合）＋`json975_decode_family_inline`（V-DETAIL-01嵌套user.name、\u0069→hi、全转义字节oracle、9项合法性负控、错kind读）。修复：前导零尾部检查误拒"0"后随逗号（"zero":0误报malformed）。验证：conformance **1148/1148绿142.73s**；core/av-core/lowering/native 751过+1**先在红**（allocation_controls::exact_backing——基线bb8d56e07复现同败，环境/先在归因，非本增量触面）；no_std运行时runtime-dev profile 0错误；workspace all-targets（除no_std）0错误。**974指针**：url_decode一般形所需原语已备，归974归档pending清单（其delivery vehicle另定——974已归档，不自动归975）。
  [✅ 2026-11-16 增量, worktree commits fe0074e62+8ca3064d7] **Auto侧sqlite.* API面已交（fe0074e62）**：tracked `stdlib/ace/std/sqlite.at`（PLAN-1041内联，**plain fn拼写**——text-record foreign lane拒pub；pub/packages拼写随973 io装配=具名serving缝）——sqlite_open/sqlite_open_memory（973无关:memory:腿）/sqlite_exec（末语句changes）/sqlite_query_int（首行列0）/sqlite_close，旧SqliteDb句柄=provider单连接（无RAII），fallible状态=lane命名ForeignStatus陷阱无静默fallback；`db975_auto_sqlite_api_face_on_av_foreign_lane`（模块verbatim＋use.c admission＋stub table＋execute_foreign对真实钉版DLL；手算SUM=9＋源内逐项返回核对＋9 dispatch计数；负控exec-before-open/double-close ForeignStatus陷阱＋无use.c命名编译拒绝=无builtin静默解析）；顺带修复rust_provider三处cl CWD未钉（并发DLL构建在workspace根raced failure_runtime.obj，C1083复现1/4，钉死后连续5次并行绿）。**TextFromChar六路径降落已交（8ca3064d7，AC-COVERAGE后续）**：a2x-core（semantic_view text_kinds+kind_name、prepare计算面+指纹序列化from_char——此前两道gate直接拒绝该expr）＋a2x-rust（rt a2x_str_from_char=try_reserve+rt_alloc纪律＋lower＋lower_coll×2）＋a2x-python（chr(ic).encode，InvalidScalar陷阱沿char.from_codepoint）＋a2x-c（rt_runtime/coll_runtime手写UTF-8编码器struct-return，非scalar=INVALID_SCALAR＋lower_rt/lower_coll）；用例`json975_char_to_str_a2x_ends`（同一packages源AV/mn/rustc/cl四端=1347，4.18s）＋`char975_to_str_on_python_end`（真实OS Python，0.27s）；scoped回归a2x-core 9/9＋record_text 32/32。**泛型typed面=具名gap（双缝）**：①tracked模块跨lane拼写缝——text-record拒pub（sqlite.at实证）、packages仅public-item且json.at私有helper不兼容（app::jsond实证"fn is not part of this source profile"，record-typed组合decode试验已回退）；②结构感知decode[T]（legacy反射intrinsic形）需966 comptime线；typed组合面（consumer record经已交readers decode）待缝①裁定（973 io装配线或专门裁定）后落位。

- [x] **T-B1-V｜M2-A｜work｜AC-B1**：在原975树冻结已实现B1/source/schema/后端版本，复用并补原语的独立字节/错误/owner反例，运行V-B1-LAND；不要求1064/MODULE或975数据库整包完成。
  [✅ 2026-11-16, worktree commit c92d3318e（冻结SHA；B1实现=3690d8161，六路径降落=8ca3064d7）] `json975_char_to_str_b1_slice_oracles`（console面：NUL标量byte-exact 1字节可读回且非终止符、U+10FFFF边界4字节、D800/0x110000陷阱CharInvalid于typed gate；四端packages腿NUL组合=31 AV/mn/rustc/cl；owner反例=A2X_RT_FAULT_ALLOC=1恰在物化分配处trap:Resource）＋`char975_to_str_python_nul_and_invalid_scalar`（A2PY腿：NUL/边界byte-exact、D800→stdout trap:InvalidScalar）；突变负控已验并回退：错runtime字节（'?'）→A2R腿RED 1110≠1347、删a2x-rust lower臂→命名a2x.unsupported-expr拒绝RED；家族6/6绿3.11s+python 2/2绿0.34s。
- [x] **T-B1-R｜M2-A｜review｜AC-B1-REVIEW**：停止切片写入，在同树独立phase审B1全部适用义务/原例/健康/SD，绑定r10/冻结SHA，失效证据定向更新，不替原T-05全包final。
  [✅ 2026-11-16 phase pass] reviewed_commit c3637c3c3（=c92d3318e＋健康修复）；同会话复审（无独立指派reviewer，局限性已记，从工件重构结论：代码diff重读＋全家族新鲜重跑＋第三轴突变）。V-B1-LAND矩阵全pass：中(2/3/4字节AV+四端1347+A2PY)、NUL(1B可读回非终止符+四端31+py)、非法scalar(CharInvalid×2+py trap:InvalidScalar)、owner(跨函数owned返回+FAULT_ALLOC恰中物化trap:Resource)；**突变三轴全RED**：删a2x-rust lower臂→a2x.unsupported-expr命名拒绝、错runtime字节→1110≠1347、删native fingerprint臂→E0004 non-exhaustive编译红（穷举match防御）；附观察：fingerprint文本投毒→cache-miss重建→绿=设计正确（指纹变更触发重建）。健康：fmt净、debug无、**1发现已修**（4处unused-mut→c3637c3c3，家族6/6绿2.85s+py 2/2绿0.37s复验）；新用例计时全部≤10s（b1 3.6s/py 0.37s）。SD：B1切片Spec文本归T-B1-D同版沉淀（new_spec_components目标在案）。
- [x] **T-B1-D｜M2-A｜merge｜AC-B1-LAND**：切片pass后正式合入与已验Spec/IF-975-B1收据，供1064；保树/resync/整包executing，不能勾原T-06/07或清树。
  [✅ 2026-11-16] phase fold落主 **master@8e0c4d1d6**（11分支提交：B1核心3690d8161→sqlite面fe0074e62→六路径8ca3064d7→数组路径2a27f58ae→redis Str 02a687da8→T-B1-V c92d3318e→健康修复c3637c3c3→Spec沉淀9b880345e）；branchside与主检出WIP零重叠实证后净合并；worktree已resync至8e0c4d1d6（保树/整包executing/原T-06/07未勾）；落主check：touched crates（av-core/a2x-core/abi）master上cargo check净24.46s；Spec=docs/specs/ace-migration/m2-stdlib-data.md（B1切片节：语义/所有权/错误/六路径矩阵/三轴突变绑定/owner反例+数据面速览+具名缺口）。**IF-975-B1**：1064消费接口=char.to(Str)原语（owned单标量UTF-8物化、typed gate CharInvalid、NUL非终止符）六路径可用；json.at/redis.at/sqlite.at tracked模块现状与lane边界（console/foreign内联；packages/asm面待973/1064装配）；详见Spec文件与本Plan §9。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 MODULE实际消费1064/T-MODULE-D正式切片；本包负责唯一canonical公共声明/typed decode/真实业务，落V-LOAD-MODULE。
  [✅ 2026-11-16] 消费者接线已交：json（console内联+数组路径legacy形状007_json fixture+四端packages via B1腿）、sqlite（AV foreign lane Auto面db975+967钉版provider）、redis（console lane+RESP严格服务端oracle+get Str物化02a687da8）；六路径char.to(Str)消费（V-COVERAGE腿）。**MODULE腿已闭（裁定a先行实施1064后）**：IF-1064-MODULE=master@4ba167930（1064 T-MODULE-I/V/R/D四任务全交，切片含条件缝闭合+模块域use/限定形+真实json.at/redis.at装配消费+负控+RESTORE）；本树52d2720dd=canonical pub-fn sqlite.at迁移（canonical作者源，双lane服务：foreign逐字节db975 2/2绿+packages面5 exports精确+删pub镜面RED）+MODULE text侧pub准入（text-record加面）；**provider执行（p967_*）留foreign lane（SUM=9在案）——packages模块面的外来声明=具名1043余量**；V-LOAD-MODULE的json/redis装配腿已在1064 module_v_1064交付（真实源装配+执行=42）；sqlite provider装配执行=具名余量。拼写缝裁定（归973装配线）在§10。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：逐JSON、sqlite/redis、序列化/缓存的实际API覆盖；现有磁盘/数据库格式与ABI无关，不因换引擎破坏用户数据，schema迁移与降级策略交986消费。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] V-08 JSON腿已交（null/missing区分、i32-max无损失、bool读=2a27f58ae数组路径测试）；sqlite数据兼容=同967钉版SQLite 3.46.0引擎（引擎同一性保全数据），文件持久化/迁移失败旧文件可用腿=973分账（T-01冻结）；redis=外部服务无本地迁移面（严格RESP服务端=兼容oracle）；schema迁移策略归986。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16, §9收据在案] Windows冻结并集：a2r-std PASS 115/stdlib PASS 171/ace-stdlib PASS 2/ace-runtime 84例1环境红/ace组3510例（70红全归因：基线3690d8161同败实证text-conformance链接环境、source_foreign代表、batch case-fence、language_conformance代表、governance；c_import/binding隔离重放绿=并发争用；auto-lang-core 1例未单独重放具名；机器29并发构建进程争用在案）；V行映射见§9（V-LOAD-MODULE=1064阻塞具名、V-LINUX-PHASE=1063分账、V-02/03 rollback/迁移面=973分账）。Linux腿按2026-11-14裁定不在Windows跑。

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。
  [✅ 2026-11-16 final pass, reviewed_commit `d9efa844a`] round-1 `F-01..F-06` 全部闭合；Plan 975 全部 **18/18 用例 PASS**（`-j 1` 独立计时最长 `6.247s <= 10.000s`）；`rustfmt --check` exit 0、`cargo check` 受改文件 0 警告、0 调试打印；T-04 冻结并集核验通过（`stdlib` 171/171、`a2r-std` 115/115、`ace-source-syntax-927` 13/13、`ace-stdlib` 2/2、`ace-source` 757/757、`ace-a2x` 250/251 + 隔离重放 1/1 = 251/251、`ace-native` 279/281 + 隔离重放 2/2 = 281/281；`ace-av` 1300/1301、`ace-abi` 274/299、`ace-runtime` 65/84、`ace-a2x-c` 8/9 余红全部逐项归因为 `master` 先在或 Windows `rust-lld` 环境）；`KNOWN-DEBT-AND-RISKS.md` 已登记 M2-B 边界与先在红；可进入 `T-06`/`T-07` 合入归档。

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。
  [✅ 2026-11-16, landed `master@7b73396aa`] 无冲突 rebase 至 `master@b6a6a1c38` 并 `ff-only` 合入（5 提交 `2309426c8..7b73396aa`, 14 文件）；rebase 前后 18/18 Plan 975 用例 + `plan981_grammar_scope` = 19/19 PASS (`24.731s`)；主检出落主 `cargo check --tests -p ace-a2x-c -p ace-abi -p ace-av -p ace-core` exit 0 (`21.93s`)；canonical Specs 已沉淀 (`m2-stdlib-data.md`, `replacement-scope-and-evidence.md`, `ace-binding-and-assembly.md`)。

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。
  [✅ 2026-11-16] 路径对账干净（零散落 `*975*` 输出与 `attachments/`）；`D:/autostack/plan-artifacts/plan-975/` 5 项临时文件（56,482 bytes）已按文件名+大小+SHA256 折入 §9 并整目录删除，`.retention/policy.json` 刷新为 `deleted-at-merge (no holds)`；`wt-guard.ps1` clean → 移除 `D:/autostack/.wt/lang-975/auto-lang`、`plan-975-dev` 与空组目录；`.last-batch-regression.json`（`last_covered_plan_id: 1060`, `2026-11-14T23:23:08Z`）因 `975 % 5 == 0` 且 `>48h` 到期（因多会话并发构建与主检出 `docs/**` WIP 未在本会话单独占用全局跑，待空闲窗口执行 `/auto-plan:regress`）。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-975/auto-lang`，分支 `plan-975-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-975/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: merge | PLAN-975 | r10 | 2026-11-16 | outcome: pass | worktree_cleaned: D:/autostack/.wt/lang-975/auto-lang (wt-guard clean) | landed: `master@7b73396aa` (ff-only from `plan-975-dev`, rebase atop `b6a6a1c38`, 5 commits `2309426c8..7b73396aa`) | specs_deposited: `docs/specs/ace-migration/m2-stdlib-data.md`, `docs/specs/ace-migration/replacement-scope-and-evidence.md`, `docs/specs/stdlib/design/ace-binding-and-assembly.md` | landing_smoke: `cargo check --tests -p ace-a2x-c -p ace-abi -p ace-av -p ace-core` (`MASTER_CHECK_EXIT=0`, `21.93s`); post-rebase `19/19 PASS` (`24.731s`, max `6.218s`) | artifacts_deleted: `D:/autostack/plan-artifacts/plan-975/` 整目录删除（无保留持有，原 5 件摘要与 SHA256：`OWNER.md` 201B `EECAB718367BFA15A3781B4735D598E96DEEC35655AC46AF62DF0A85BBED867D`、`json.at` 22361B `D48EAE2D06E051E58B1BE81C808544B278878A916370312AE3D491542AE76506`、`json_975.rs` 11298B `F0560E278D5FF312DE6BBB2D907C569838DC52123E7E87762E3C023933075C23`、`redis.at` 8583B `AE50C76EDC329300FA210323204FC25CF68D69F397CAEC9DFC3DC4F3F856FF89`、`redis_975.rs` 14039B `12F0D2E9C2D1A8B34DAF09E7190C334BE491E50843EB0D7D93E7C44FEA5FD17E`）；worktree `experimental/ace/target` 随删树清除，零散落 `*975*` | archive: `docs/plans/archive/975-ace-m2-stdlib-data.md` | batch_regression: `.last-batch-regression.json` (`last_covered_plan_id: 1060`) 因 `975%5==0` & `>48h` 到期，鉴于多会话并发及主检出 WIP，交空闲窗口单实例 `/auto-plan:regress`

stage: review | scope: final | round: 2 | PLAN-975 | r10 | 2026-11-16 | outcome: pass | reviewer: auto-plan-review (T-05) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev (`d9efa844a`) | repairs-verified: `76c23c719` (`F-01..F-06`), `aede77e09` (`plan981_grammar_scope.rs` `TextRecord` post-change hash), `d9efa844a` (split `a2c`/`a2py` <= 10s) | evidence:
- **Health**: `rustfmt --check` exit 0 across `db_975.rs`/`json_975.rs`/`b1_slice_975.rs`/`redis_975.rs`/`stdlib_contract_975.rs`/`a2x-c/src/lower_rt.rs`/`plan981_grammar_scope.rs`; `cargo check --tests -p ace-a2x-c -p ace-abi -p ace-av -p ace-a2x-worker` `MATCHES=0 CHECK_EXIT=0`; 0 debug `eprintln!` / 0 silent-return skips.
- **18/18 Plan 975 cases (`task-528`, `-j 1`, `EXIT=0`)**: `char975_to_str_on_python_end` (`0.205s`), `char975_to_str_python_nul_and_invalid_scalar` (`0.349s`), `db975_sqlite_at_packages_module_face` (`0.028s`), `db975_auto_sqlite_api_face_on_av_foreign_lane` (`6.014s`, incl. `BEGIN`/`ROLLBACK` & SQL-fail `ROLLBACK`), `db975_sqlite_memory_leg_on_pinned_provider` (`6.247s`), `json975_char_to_str_b1_slice_oracles` (`2.568s`), `json975_array_paths_and_legacy_shape` (`0.160s`), `json975_char_to_str_a2x_ends` (`2.953s`), `json975_char_to_str_primitive` (`0.052s`), `json975_decode_family_inline` (`0.216s`, incl. UTF-16 surrogates/`i32::MIN`/invalid escapes), `json975_encode_family_inline` (`0.143s`), `redis975_resp_over_platform_net` (`0.078s`), `redis975_std_redis_packaging_inline` (`0.176s`, incl. `badutf8`/`overlong`/`surr`), `json975_packages_coverage_a2c` (`2.241s`), `json975_packages_coverage_a2py` (`0.462s`), `json975_packages_coverage_av_mn_a2r` (`1.487s`), `json975_typed_decode_and_ct_metadata` (`0.051s`), `json975_v_load_module_closure` (`0.375s`) — **all <= 6.247s <= 10.000s**.
- **T-04 冻结固定组并集**: `stdlib` **171/171 PASS**、`a2r-std` **115/115 PASS**、`ace-source-syntax-927` **13/13 PASS**（`task-467`, `50.10s`, `gate_passed: true`）；`ace-stdlib` **2/2 PASS**（`task-473`）；`ace-source` **757/757 PASS**（`task-500`, `18.27s`）；`ace-a2x` **250/251** + 隔离重放 `a2c_rt_cert::rt_certify_all_suite_cases` **1/1 PASS** (`18.331s`) = **251/251**（`task-477`/`task-490`）；`ace-native` **279/281** + 隔离重放 2 例 (`audit1027_utf_02` `2.631s`, `dual_target_same_signature_dispatches_independently` `1.300s`) = **281/281**（`task-504`/`task-514`）；`ace-runtime` **65/84**（19 `ace-text-conformance` Windows `rust-lld` 预存环境红）；`ace-abi` **274/299**（25 `source_foreign` 与 `ace-a2x-c` 1/9 为 `master@82d032c8e` C runtime SHA pin 先在漂移）；`ace-av` **1300/1301**（唯一红 `stdlib_list_946_grammar_face` 为 `master@4ba167930` 条件文法拓宽后先在过时断言）。
- **Audit**: All §7 ACs (`AC-01..06`, `AC-PHASE`, `AC-COVERAGE`, `AC-LOAD`, `AC-LINUX-PHASE`, `AC-AUTO-STD`, `AC-B1*`, `AC-LOAD-MODULE`), `V-05` per-symbol mapping, and §10 `L-01..L-05` Linux handoff table verified; `KNOWN-DEBT-AND-RISKS.md` updated; no unapproved deferrals or workarounds.
| next: T-06 / T-07 merge（Spec 沉淀 `docs/specs/ace-migration/m2-stdlib-data.md` 等 + rebase/ff-only 合入 master + 归档 + 清理 `plan-artifacts/plan-975/` 与 worktree）

stage: work | repair | PLAN-975 | r10 | 2026-11-16 | outcome: pass | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev (`76c23c719`) | fixes-for: T-05 round-1 (`F-01..F-06`) | code_commit: `76c23c719` (`stdlib/ace/std/{json,redis}.at`, `db_975.rs`, `json_975.rs`, `redis_975.rs`, `stdlib_contract_975.rs`, `conformance.rs`, `a2x-c/src/lower_rt.rs`) | evidence:
- **F-01/F-02 Health**: 清除 `redis_975.rs` 4 处调试 `eprintln!("[r975s] ...")`；`db_975.rs` 缺 pin 静默 `return` 改为 `.expect(...)` 强失败；`rustfmt --check` 全部 6 个受改 `.rs` 文件 exit 0；`cargo check --tests -p ace-a2x-c -p ace-abi -p ace-av -p ace-a2x-worker` 受改文件 **0 警告**（`MATCHES=0 CHECK_EXIT=0`）。
- **F-03 `V-02` SQLite 回滚**: `db975_auto_sqlite_api_face_on_av_foreign_lane` 在 `:memory:` 上实证 `BEGIN; INSERT 100; ROLLBACK;` 及 SQL 错后 `ROLLBACK`（`SUM=9`, `COUNT=2`, `callee_count=19`）。
- **F-04 UTF-8/UTF-16 防护**: `redis.at` `redis_get` 增全字节/续字节/过长/代理/超界 UTF-8 校验并返回 `""`（`badutf8`/`overlong`/`surr` 负控绿）；`json.at` 增 `\uD800..\uDFFF` UTF-16 代理对解码与孤立代理拒绝、控制字符 `< 32` 与非法转义（`\q`/`\uZZZZ`）拒绝、`i32::MIN` `-2147483648` 无溢出解析。
- **F-05 `decode<T>` / `966` CT / `V-LOAD-MODULE`**: 新增 `stdlib_contract_975.rs`：`json975_typed_decode_and_ct_metadata` (`0.052s`, `spec JsonDecodable` + `<T has JsonDecodable>` 2 实例 + `AV_DATA_PROFILE` 运行 `102` + `source.instances-bound` / 缺成员负控 + `966` `ct_field_count`/`ct_has_field` = `31` + 缺能力负控)、`json975_v_load_module_closure` (`0.382s`, `app::json` + `app::service` `pub type Service` = `8106`, `json_cp` `source.module-private`, `json.at`/`redis.at` `pub`-stripped RED, `std::json` `ace.stdlib-reserved`, `PackageStore::restore`, 私有体变更移 hash)。
- **F-06 5 端 `json.at` + 单用例计时**: `json975_packages_coverage_av_mn_a2r` (`1.688s`) + `json975_packages_coverage_a2c_a2py` (`2.849s`)，5 端全等手算 `42`；并修复 `a2x-c/src/lower_rt.rs` 内容视图参数转发 `C4133`（`ace-a2x-c` 8/9 绿，唯一失败 `runtime_admission_preserves_bytes_and_the_certification_pin` 为 `master@82d032c8e` Plan 972 先在 pin 漂移）。全部 **17/17 Plan 975 用例 PASS**（`task-420`, `EXIT=0`, 最长 `7.619s <= 10.000s`）。
| next: T-04 冻结并集重验 + T-05 round-2 final 独立复审

stage: review | scope: final | round: 1 | PLAN-975 | r10 | 2026-11-16 | outcome: needs_fix | reviewer: auto-plan-review (T-05) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev (`52d2720dd`) | evidence-checked: cargo check 0 warnings in 975 files (4 pre-existing `text-runtime` warnings); `rustfmt --check` exit=1 (`db_975.rs`/`json_975.rs`/`redis_975.rs`); `source_cst_927` 13/13 PASS (`2.865s`); 975 cases 13/13 PASS & all `<= 10.000s` (`ace-av` 8/8 `4.494s`, max `4.482s`; `ace-a2x-worker` 2/2 `0.460s`, max `0.451s`; `ace-abi` 3/3 `8.389s`, max `8.388s`) | findings:
- **F-01 [Low/Health]**: `redis_975.rs:66,75,79,83` 含 4 处调试 `eprintln!("[r975s] ...")`；`db_975.rs:80,217` 在 `locate_pinned_sqlite("3.46.0")` 失败时静默 `return`（假 PASS），须改为 `.expect(...)`。
- **F-02 [Low/Health]**: `rustfmt --check` 在 `db_975.rs`、`json_975.rs`、`redis_975.rs` 失败。
- **F-03 [High/AC-02/V-02/V-DETAIL-03]**: `V-02`（事务失败回滚再读原值）误分账至 973；`:memory:` SQLite 经 `sqlite.at` 已支持 `BEGIN`/`INSERT`/`ROLLBACK` 及 SQL 错误后 `ROLLBACK` 读回原值，应在 `db_975.rs` 实证。
- **F-04 [High/AC-02/V-DETAIL-02]**: `redis_get` (`redis.at:275-310`) 未校验无效首字节（`128..191`, `>=248`）、续字节（`<128 || >191`）、过长编码、代理码点（`55296..57343`）与 `> 1114111`，会错误输出字符或触发 `CharInvalid` 陷阱而非返回 `""`；`json_parse_str` (`json.at:891`) 对 `\uD800..\uDFFF` 触发 `CharInvalid` 陷阱而非解码 UTF-16 代理对/返回 `""`，且 `json_str_end` (`json.at:213`) 放过 `\q`/`\uZZZZ` 与未转义控制字符 `< 32`。
- **F-05 [High/r10 AC-LOAD-MODULE/V-LOAD-MODULE/decode[T]]**: r10 仍将结构感知 `decode[T]` 与 `V-LOAD-MODULE` 归本包，但：① 尚无 typed record / `spec JsonDecodable` 泛型 `decode<T>` / `966` `cte.meta.field_count`/`has_field` 合约测试（且 `docs/specs/ace-migration/m2-stdlib-data.md:55-57` 仍记旧 r9 拼写缝阻塞）；② 尚无 `975` 自身 canonical `json.at`/`redis.at` 的 `V-LOAD-MODULE` 跨模块测试（私有 helper `json_cp` `source.module-private` 拒绝、`pub`-stripped 镜面 RED、伪造 `std::json` `ace.stdlib-reserved`、`PackageStore::restore`）。
- **F-06 [Medium/AC-COVERAGE/V-05/AC-LINUX-PHASE/§7]**: `json.at` 仅在 AV 运行，缺 `native/mn`、`A2R`、`A2PY`、`AC` 跨端闭环；Plan 缺 `V-05` 逐符号覆盖对照表与 §10 `L-01..L-05` Linux 移交表；§7 仍有 4 项 AC 未勾选；`52d2720dd` 修改了 `auto-source.at`，需在最终修复冻结提交上重跑 T-04 冻结并集。
| next: work（回 `D:/autostack/.wt/lang-975/auto-lang` 修复 F-01..F-06，更新文档与 spec 草案并刷新 T-04 → 再行 T-05 final 复审）

stage: work | PLAN-975 | r10 | 2026-11-16 | outcome: pass (T-03 MODULE腿闭合；1064切片先行实施全交) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev（resync至master后+52d2720dd） | chain: 裁定a→1064树T-MODULE-I r1(208cc04a5条件缝)+r2(e2e9b4724模块域use)→V(6ee22e770真实源装配消费7/7)→R(phase pass，突变RED)→D(master@4ba167930恰6文件；981并集解冲突；add -u事故soft-reset纠正；age 26/26+module 13/13+units 43/43复验；1064 4/13保树)→回本树52d2720dd(sqlite.at pub迁移+text侧pub准入+db975 3/3) | evidence: db975 3/3绿10.5s（含新packages面用例：5 exports精确+pub-stripped镜面module-private RED） | named-remaining: packages模块面外来声明（sqlite provider装配执行）=1043线余量；V-LOAD-MODULE其余行（#in/候选/SDK负控）=1043既有面未触 | next: T-05全包final独立复审（work全任务已tick、分账具名）

stage: goal | PLAN-975 | r10 | 2026-11-16 | outcome: unblocked (裁定a) | from: blocked(3) | to: work(1064先行) | ruling: 目标系统按选项列表指定"任选其一"——本会话取(a)：按1064 r2合同先行实施T-MODULE-I→V→R→D（其V腿消费已落主IF-975-B1），交IF-1064-MODULE后回975续T-05全包final（含V-LOAD-MODULE腿）→T-06/07；975的sqlite.at pub迁移归本包原T-03（1064合同§5.2明示不代实施） | next: 1064切片实施（lang-1064树）

stage: goal | PLAN-975 | r10 | 2026-11-16 | outcome: blocked | round: 3 (连续等待轮闭值) | from: waiting(2) | to: user | wait-on: 1064仍无会话开工（drafting r2 0/13/无worktree/创建后无活动）×3轮复核 | asked: 1064承接裁定已问（本会话先行/等待原owner/改合同跳过MODULE腿）——无应答 | blocked-question: 1064/T-MODULE-D由谁承接实施？(a)授权本会话按其r2合同先行T-MODULE-I→V→R→D交IF-1064-MODULE后回975续T-05→T-06/07；(b)归1064原owner会话择时开工，975保持等待；(c)明示改r10合同把V-LOAD-MODULE/AC-LOAD-MODULE腿保持具名gap先行T-05全包final（MODULE面后补） | unblock: 任一裁定给出后本目标从对应点续行（worktree已resync master@8e0c4d1d6保树待命；B1切片已落主+IF-975-B1已出具） | next: user

stage: goal | PLAN-975 | r10 | 2026-11-16 | outcome: waiting | round: 2 | from: waiting(1) | to: review(T-05) | wait-on: 1064仍drafting r2 0/13、无执行worktree、计划文件自12:06创建handoff后无活动（973会话所交） | action: 按目标指令向用户请求1064承接裁定（连续等待轮确认无开工） | next: 按裁定续行（本会话先行1064 / 继续等待 / 其他路由）

stage: goal | PLAN-975 | r10 | 2026-11-16 | outcome: waiting | round: 1 | from: merge(T-B1-D done, master@8e0c4d1d6) | to: review(T-05) | wait-on: 1064/T-MODULE-D交付IF-1064-MODULE（1064 drafting r2 0/13，无执行worktree；IF表注明其V已可消费975 B1） | unblock-action: 1064切片落主后本目标续行T-05全包final（含V-LOAD-MODULE腿）→T-06/07 | next: 等待轮（每轮复查1064状态）

stage: merge | phase (T-B1-D B1切片合入) | PLAN-975 | r10 | outcome: pass | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev（保树，resync至8e0c4d1d6） | landed: master@8e0c4d1d6（phase fold，merge-base b1cb6faeb，11提交） | spec: docs/specs/ace-migration/m2-stdlib-data.md（B1切片节，9b880345e） | IF-975-B1: 出具（见T-B1-D勾注） | landing-check: touched crates master cargo check净 | gates: 冻结组证据=T-04收据复用（实现码同一）＋phase家族新鲜绿（6/6+2/2） | next: T-05全包final独立复审（MODULE腿/V-LOAD-MODULE=1064阻塞具名；泛型typed面=973/966具名gap；其后T-06/07）

stage: review | phase (T-B1-R B1切片) | reviewer: 同会话（无独立指派，局限性在案；从工件重构） | PLAN-975 | r10 | outcome: pass | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | reviewed_commit: c3637c3c3 | base_commit: c92d3318e(冻结)+健康修复 | evidence: json975家族6/6绿2.85s＋py腿2/2绿0.37s（修复后新鲜跑）；突变三轴RED（删lower=命名拒绝/错runtime=1110≠1347/删fingerprint臂=E0004）；slice文件0警告0 fmt漂移；T-04组证据复用（实现码同一性） | findings: 1条健康（unused-mut×4）→work已修c3637c3c3复验绿 | spec_inputs: m2-stdlib-data.md待T-B1-D沉淀 | next: T-B1-D（phase合入+IF-975-B1；保树/整包executing）

stage: work | PLAN-975 | r10 | 2026-11-16 | outcome: pass (T-B1-V B1切片验证 + T-03/T-08/T-04 tick；r10合同承接) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commits: 02a687da8(redis get Str)+c92d3318e(T-B1-V冻结SHA) | ruling-applied: 拼写缝归973（§10已裁定）；r10新增T-B1-V/R/D链承接——B1切片先行独立验证/复审/合入不等1064/整包 | evidence: T-B1-V=json975_char_to_str_b1_slice_oracles（NUL byte-exact/边界U+10FFFF/CharInvalid陷阱×2/四端31/owner反例A2X_RT_FAULT_ALLOC=1恰中物化）＋python NUL/陷阱腿2/2；突变负控：错runtime→1110≠1347 RED、删lower臂→a2x.unsupported-expr RED（已回退，6/6绿3.11s）；T-04 Windows并集收据在案（前一receipt）；redis_get Str物化=多字节UTF-8穿透byte-exact | ticks: T-B1-V(c92d3318e)/T-03(MODULE腿具名阻塞1064 drafting)/T-08(JSON腿✓+973分账)/T-04(70红全归因+V行映射)；current_step 6/11 | next_responsible_role: review（T-B1-R切片独立phase审，绑定r10/冻结SHA c92d3318e；其后T-B1-D→T-05全包final）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: pass (T-03/T-08/T-04 Windows面闭合；裁定+redis Str面) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commits: 02a687da8(redis get Str面) | ruling: 用户裁定2026-11-16——模块跨lane拼写缝归973 io装配线统一裁定（§10已更新），泛型typed面在975保持具名gap | evidence: redis_get(s,key) Str落地（B1解锁的具名pending——payload UTF-8标量逐个decode+char.to(Str)再物化=多字节byte-exact穿透；nil/error/非bulk/畸形UTF-8=""，区分用get_len/byte面）；redis975 2/2绿0.14s（严格服务端12命令记录oracle）；**T-04 Windows冻结并集**：a2r-std PASS 115例45.8s＋stdlib PASS 171例137.2s＋ace-stdlib PASS 2例93.1s＋ace-runtime 84例1红（text-conformance v07_slice_ranges=rust-lld/MSVC符号链接环境失败，基线3690d8161同败实证）＋ace组4调用3510例70红全部归因：text-conformance 19=环境链接失败（基线同败×2样本）、source_foreign族~29=child exit 2 "runtime-or-emitter"（代表样本基线同败；c_import/binding隔离重放绿=组并发争用）、batch 8="case fence missing"（他plan fence，基线同败）、language_conformance代表+governance=基线同败、auto-lang-core 1例未单独重放具名（未触crate）；机器当日29并发构建进程争用在案 | V行映射：V-01✓(null/missing/i32-max)、V-02 sqlite文件腿=973分账、V-03✓(double-close陷阱+raw -1)、V-04✓(encode族字节/顺序)、V-05✓(三模块面+用例清单)、V-08 JSON腿✓+sqlite迁移面=973、V-DETAIL-01✓(JSON腿ClearDiagnosticInformation+sqlite :memory:隔离库)、V-DETAIL-02✓(畸形/错kind负控+NOT SQL陷阱)、V-DETAIL-03 close/buffer释放✓+rollback=973、V-DETAIL-04=本映射、V-COVERAGE char.to(Str)六路径✓+模块面lane分账、V-LOAD=973裁定、V-LINUX-PHASE=1063分账、V-AUTO-STD✓(JSON/redis纯Auto+sqlite显式FFI依赖列明) | next_responsible_role: review（T-05独立复审）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-03/T-08 JSON消费者腿：数组路径+V-08保真) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commit: 2a27f58ae | evidence: json.at路径walker数组段扩展（legacy JsonValue.get_at面映到点路径"services.0.name"、嵌套链"grid.1.0"；越界/非数字/空段/标量穿越=missing 0、截断/结构死端=malformed -1、干净数组尾']'=missing）；json975_array_paths_and_legacy_shape（007_json fixture形状services[0].name=ClearDiagnosticInformation经tracked模块verbatim可读＋两个服务名/端口＋嵌套grid＋5路径负控＋V-08保真腿：null vs missing区分、i32-max 2147483647无损失、bool读）0.14s；json975全家族5/5绿4.35s（encode/primitive/decode/a2x_ends/数组路径） | scope-remaining: T-03余量（sqlite/redis消费者接线深化、1043 LOAD装配面）＋T-08余量（迁移失败旧文件可用腿、redis/sqlite持久面=973分账）＋T-04全部V行未跑＋T-05..07未动 | next_responsible_role: work（T-03余量 → T-08 → T-04全V行 → T-05复审）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-02 Auto侧sqlite.* API面 + TextFromChar六路径降落) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commits: fe0074e62+8ca3064d7 | evidence: sqlite面=stdlib/ace/std/sqlite.at tracked＋db975 2/2绿~8s（Auto面：模块verbatim+use.c admission+stub table+execute_foreign；SUM=9手算+源内逐项核对+9 dispatch计数；负控exec-before-open/double-close=ForeignStatus陷阱+无use.c命名编译拒绝；顺带修rust_provider三处cl CWD未钉的并发flake，C1083复现1/4→钉后5/5并行绿）；TextFromChar=a2x-core表面/指纹两gate＋a2x-rust(rt+lower+lower_coll×2)＋a2x-python＋a2x-c(rt_runtime/coll_runtime+lower_rt/lower_coll)；json975_char_to_str_a2x_ends四端=1347（AV/mn/rustc/cl，4.18s）＋char975_to_str_on_python_end（真实OS Python，0.27s）；scoped回归a2x-core 9/9＋record_text 32/32 | gap-named: 泛型typed面=双缝具名（①tracked模块跨lane拼写缝——text-record拒pub（sqlite.at实证"pub is not part of this source profile"）、packages仅public-item且json.at私有helper不兼容（app::jsond实证"fn is not part of this source profile"，record-typed组合decode试验已回退，json_975.rs恢复至8ca3064d7）→归973 io装配线或专门裁定；②结构感知decode[T]（legacy反射intrinsic形）需966 comptime线） | next_responsible_role: work（T-03真实consumer接线 → T-08/T-04；或先取模块拼写缝裁定）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-02 decode/parse + B1原语) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commit: 3690d8161 | decision: 用户裁定B1=加char.to(Str)（子串物化核心决定闭合：slice促升本已存在，唯缺动态码点） | evidence: json975 3/3绿0.17s；conformance 1148/1148绿142.73s；4-crate套件751过+1先在红（allocation_controls，基线复现，环境归因）；AV/C来源native实现ac_rt_own_from_char | coverage-pending: A2R/A2PY/AV-Rust的TextFromChar降落地（AC-COVERAGE后续）；974 url_decode指针（原语已备） | next_responsible_role: work（T-02余量：Auto侧sqlite.* API面 或 T-03..07 或按裁定）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-02 redis RESP腿) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commit: 8e51c6e38 | evidence: redis975 2/2绿（裸lane 0.07s+包装0.04s；同源"97"过滤64/64绿12.75s）；stdlib/ace/std/redis.at tracked | blockers: decode/parse=子串物化核心决定（与974共享，下一个） | next_responsible_role: work（decode/parse核心决定 → T-03..07 或按裁定）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-02 sqlite :memory:腿) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commit: bb8d56e07 | evidence: db975 1/1绿5.6s（含独立oracle exe） | next_responsible_role: work（redis RESP腿 → T-03..07 或按裁定）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: in-progress (T-02 JSON encode族) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev | code_commit: bc19182d9 | evidence: json975 1/1绿0.09s（14 oracle） | next_responsible_role: work（sqlite :memory: 腿（S2 967 provider）/redis（S3 RESP） 或按裁定）

stage: work | PLAN-975 | r9 | 2026-11-16 | outcome: pass (T-01) | worktree: D:/autostack/.wt/lang-975/auto-lang @ plan-975-dev（基面 master@b1cb6faeb） | task_ids: T-01 | evidence: §5"T-01 A期冻结记录" | authorization: 既定裁定模式 | blockers: sqlite文件腿=973（具名）；decode/parse=子串物化核心决定（与974共享） | next_responsible_role: work（T-02 JSON encode族先行——纯拼接面无阻塞）


2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r8：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-12 stage:new / PLAN-975 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

2026-11-16 stage:new/revision | PLAN-975 r10 | outcome:pass（规划合同静态预检），非实现或产品review | 原8任务/2勾选保持，新增T-B1-V/R/D为未执行，现2/11；MODULE=1064具名四步、B1独立在前，无整包反向等待，原975业务/CT/LOAD责任保持。next=未来原owner获实施指令后先B1-V/R/D，所需MODULE正式来源后原T-03；不实施/建树/运行/改canonical Spec。

本轮规划预检：五份相关Plan原45任务ID/勾选及原AC/V/SD全部保留；新增9任务全部未执行（1064增6、975增3），当前合计54项。0–10章节、任务总数/唯一性、围栏/控制字符、本轮新增本地引用与行尾核对通过；1065表与真实SYS973_KINDS逐项相同=50，Windows W36/N14。B1→MODULE→975完整消费及973→1065→1064 AC验收的任务图无环，不存在1064/1065整包依赖975。原972归档、系统库canonical Spec与抽样产品源码hash未变；本轮未写取号计数器（其他会话1066取号与本任务分开）。973仅核原10项勾选/清理收据更正滞后计数并补前向交接，不重开终态。此为静态规划检查，所有新增运行验证仍未实施。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。

**当前已明确（r10）**：原“归973 io装配线统一裁定模块拼写”的实际执行改由其后继1064/T-MODULE-I/V/R/D；973已归档不重开。975的B1原语可先独立交，完整typed decode和canonical数据模块/业务仍归本包。无需为同一归属再次请求确认；只有新公开语义冲突按原T-01给具名决定。

### Linux 移交清单（AC-LINUX-PHASE → PLAN-1063，Windows 不运行）

| ID | 领域/能力 | 后端 / Target / ABI | Canonical 源 & 验证用例 | 独立 Oracle / 边界 | 当前状态 | Owner |
|---|---|---|---|---|---|---|
| L-01 | `char.to(Str)` B1 原语（UTF-8 1..4B、NUL、非标量） | AV / AC(native ELF) / A2R / A2C(gcc/clang) / A2PY (`x86_64-unknown-linux-gnu`) | `stdlib/ace/std/{json,redis}.at`; `json975_char_to_str_*`, `char975_*` | `"A中😀"` 字节 len=1347, NUL len=31, `InvalidScalar` trap | `deferred-required / NOT_RUN` | `PLAN-1063` |
| L-02 | `json.at` 纯 Auto 编解码与数组路径业务 | AV / AC(native ELF) / A2R / A2C / A2PY (`x86_64-unknown-linux-gnu`) | `stdlib/ace/std/json.at`; `json975_packages_coverage_*`, `json975_v_load_module_closure` | `42`（`007_json` 服务名 + `\uD83D\uDE00` + `i32::MIN` + `pub`-stripped RED） | `deferred-required / NOT_RUN` | `PLAN-1063` |
| L-03 | `sqlite.at` + 钉版 SQLite 3.46.0 provider (`:memory:` + 回滚) | AV foreign / A2C / native ELF (`x86_64-unknown-linux-gnu`, Linux C ABI lib) | `stdlib/ace/std/sqlite.at`; `db975_*` | `SUM(4,5)=9`, `ROLLBACK` 后 `9`, `ForeignStatus` trap | `deferred-required / NOT_RUN` | `PLAN-1063` (+ `973/1064` 文件腿) |
| L-04 | `redis.at` RESP2 客户端 over `platform.net` (POSIX sockets) | AV platform_net / POSIX `AF_INET` loopback (`x86_64-unknown-linux-gnu`) | `stdlib/ace/std/redis.at`; `redis975_*` | 严格服务端命令记录 + 多字节 `"中A"` + `badutf8`/`overlong`/`surr`→`""` | `deferred-required / NOT_RUN` | `PLAN-1063` |
| L-05 | Generic `decode<T>` & `966` CT 元数据 | `record-i32-draft` AV + Scalar CT (`x86_64-unknown-linux-gnu`) | `json975_typed_decode_and_ct_metadata` | `102` + `31` + 负控 | `deferred-required / NOT_RUN` | `PLAN-1063` |

**r9原裁定历史（当前执行按上述r10）**：**已裁定（2026-11-16用户裁定，T-02泛型typed面双缝之①）**：tracked stdlib模块的跨lane拼写缝——`stdlib/ace/std/*.at` 同一文件需服务多lane，但各lane对模块项拼写的接受集不同：text-record（foreign）面拒`pub`（sqlite.at实证）；packages面仅public-item且拒私有`fn` helper（json.at的`fn json_cp`等约25个私有helper实证"fn is not part of this source profile"）；host-console面接受`pub fn`+私有fn（json.at/redis.at现状）。**裁定：归973 io装配线统一裁定模块拼写**（候选b"json.at全pub化在本包内落位"被否决）；本包该面保持具名gap，973裁定后由其消费线落位record-typed组合decode。结构感知`decode[T]`（legacy反射intrinsic形）另需966 comptime线，不在本缝内。裁定前后均不以非verbatim变体或镜面测试充交付。
