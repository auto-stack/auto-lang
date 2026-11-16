---
plan_id: PLAN-975
status: executing
feature_name: 新版JSON序列化数据库与扩展库
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 9
current_step: 2
total_steps: 8
depends_on: [PLAN-972, PLAN-973, PLAN-948, PLAN-1042]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-stdlib-data.md]
touched_goals: [GOAL-002]
---

# [PLAN-975] 新版JSON序列化数据库与扩展库

## 0. 变更摘要

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
| V-05 | AC-02/03 | 余下每符号必须有实现/验证收据 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

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

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

- [ ] **AC-AUTO-STD / work**：§5本包普通标准能力默认算法/协议为共享Auto源码，实际来源/调用链审计与原独立expected齐全；可选C/Rust及高级外库列明边界，不冒默认自有实现。原能力work任务与最终验证负责，原独立review核；Linux按AC-LINUX-PHASE移交。

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] 冻结记录见§5"T-01 A期冻结记录"：前置核实+973分账（JSON/:memory:/redis不前置，文件腿具名阻塞）；三域选型冻结（S1自有JSON codec/S2复用967 FFI sqlite/S3 RESP over platform.net）；worktree已建。

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [✅ 2026-11-16, worktree commit bc19182d9] **JSON encode族已交**（T-02 首增量）：`stdlib/ace/std/json.at`（json_escape/json_int含负数/json_bool/json_null/json_str/kv与array/object组合helper + json_structurally_balanced字符串感知括号扫描=廉价有效面**明非parse**）＋`json975_encode_family_inline`（verbatim内联14独立手算oracle含3负控，0.09s）。decode/parse/泛型typed面=具名pending（子串物化决定共享）。**sqlite :memory:腿已交（bb8d56e07）**：`db975_sqlite_memory_leg_on_pinned_provider`——967内容钉版SQLite 3.46.0 provider以`:memory:`路径开启（无宿主文件=零973依赖），手算oracle SUM(4,5)=9＋独立cl oracle exe同amalgamation私有`:memory:`库同语句复核；负控exec-before-open/-1、double-close/-1；oracle语义在案（changes=末语句、close ack out=1）；5.6s。**Auto侧sqlite.* API面=具名后续**（console道foreign dispatch缝）。**redis腿（S3 RESP）待续**。

- [ ] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06**：逐JSON、sqlite/redis、序列化/缓存的实际API覆盖；现有磁盘/数据库格式与ABI无关，不因换引擎破坏用户数据，schema迁移与降级策略交986消费。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-975/auto-lang`，分支 `plan-975-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-975/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

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

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
