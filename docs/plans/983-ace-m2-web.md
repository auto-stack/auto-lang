---
plan_id: PLAN-983
status: drafting
feature_name: Web服务嵌入与实际后端迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 6
current_step: 0
total_steps: 19
depends_on: [PLAN-967, PLAN-968, PLAN-969, PLAN-974, PLAN-975, PLAN-978, PLAN-980, PLAN-982]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-web.md]
touched_goals: [GOAL-002]
---

# [PLAN-983] Web服务嵌入与实际后端迁移

## 0. 变更摘要

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r5当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-07 r3：细化为4批，§5–8为当前执行合同；仍仅规划、不施工。


迁Web owner线程/handler调用、动态与生成Rust服务、jade-garden及下游实际server入口，去除AutoVM私有类型耦合；保文件返回、上传、认证/权限、错误、shutdown。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-web/crates/auto-web/src/host.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-web/crates/auto-web/src/http_server/server_dispatch.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-web/crates/auto-web/src/http_transport.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：Web宿主端口/动态与生成服务/真实消费者。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-web/crates/auto-web/src/host.rs](../../../auto-web/crates/auto-web/src/host.rs)：public WebHttpServiceHost端口；读取SHA256前16 `a3be862079c790c6`

- [auto-web/crates/auto-web/src/http_server/server_dispatch.rs](../../../auto-web/crates/auto-web/src/http_server/server_dispatch.rs)：替Rc<AutoVM>与旧handler调用；读取SHA256前16 `44e4eeedac749dc4`

- [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs)：真实安装消费/端口关闭oracle；读取SHA256前16 `ff449536a73274af`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A=PORT/PROOF，PORT只等967/968/969/974，PROOF补975/978；不等982/EXEC、980整包或四仓CONSUMERS。B=AVSERVICE/GENERATED/CONSUMERS，保完整原功能及四仓收据。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PROOF-I, B-PROOF-V, B-PROOF-R, B-PORT-I, B-PORT-V, B-PORT-R |
| M2-B | B-AVSERVICE-I, B-AVSERVICE-V, B-AVSERVICE-R, B-GENERATED-I, B-GENERATED-V, B-GENERATED-R, B-CONSUMERS-I, B-CONSUMERS-V, B-CONSUMERS-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan局部批次可独立实施/验证/复核，不新增Plan状态，也不等于可单批归档/删树。本Plan唯一worktree保留至全部批次及最终review/merge完成。分支冻结SHA只供受控准备；下游最终验收须核已合入来源，批次review不冒正式D。

frontmatter是整包最终依赖并集，各批按下表最小接口前置启动。LNNN=auto-lang/PLAN-NNN；UI/AG/OS标仓身份。依赖收据必须包含repo/commit、Plan revision、接口/schema/能力域和实际consumer/oracle。只有计划名字不等接口就绪。

唯一协调组D:/autostack/.wt/lang-983/，各实际改动仓保一个兄弟tree；实施/review顺序复用，公共文件单写。外仓开工重核实际唯一树，不建junction/symlink。OS/App采用OS064规定的OS worktree/独立App clone，不套语言树规则。

| 批次 | 交付结果 | 最小前置 | owner |
|---|---|---|---|
| 983/PORT | Web中立执行与资源端口 | L967/L968/L969/L974 | auto-web |
| 983/AVSERVICE | AV真实HTTP服务 | 983/PORT; L975 | auto-web |
| 983/GENERATED | 生成Rust服务 | 983/PORT; L978/L975 | auto-web + auto-lang |
| 983/CONSUMERS | Web Man UI Down真实消费闭环 | 983/AVSERVICE; 983/GENERATED; 982/EXEC; 980/WEB | auto-web + auto-man + auto-ui + auto-down |
| 983/PROOF | RG-A02真实AV与生成服务 | L983/PORT, L974, L975, L978 | 原owner；RG-A具名真实验证 |

### 983/PORT：Web中立执行与资源端口
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/host.rs`；`D:/autostack/auto-web/crates/auto-web/src/http_transport.rs`；`D:/autostack/auto-web/crates/auto-web/src/http_server_handle.rs`。
I实现：把public host的Rc<AutoVM>/旧native签名替换为中立受检端口，保owner线程、取消、File/Stream所有权和shutdown。
V验收：独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/AVSERVICE：AV真实HTTP服务
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/http_server/server_dispatch.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_runtime.rs`。
I实现：动态路由、参数、认证/权限/中间件、上传/下载/stream与DB接ACE，保transport现有实现。
V验收：真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/GENERATED：生成Rust服务
改面（均核存在）：`D:/autostack/auto-web/crates/auto-web/src/api_gen.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_generation.rs`；`crates/auto-generation-core/src`。
I实现：新HIR A2R生成服务/共享载体，保route/store/schema/diagnostics与失败保旧；本批可与AVSERVICE在端口稳定后独立。
V验收：未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 983/CONSUMERS：Web Man UI Down真实消费闭环
改面（均核存在）：`scripts/web-four-repository.py`；`D:/autostack/auto-man/crates/auto/tests/plan890_web_consumers.rs`；`D:/autostack/auto-down/jade-garden`。
I实现：jade-garden与既有Web生产调用逐行接新SDK；每owner代码独立交付，消费相同source/tool版本，保旧四仓义务。
V验收：原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。


A期生成入口实核：现有`http_generation.rs::cold_generated_rust_executes_and_missing_web_preserves_workspace`直接调用公开`auto_web::generate_api`后真实cargo编译/TCP验证，不依赖Man全目标CLI。本批沿该公开入口补新A2R接线与所需case，完整GENERATED与CONSUMERS仍B；不得为等待982/RUST或EXEC整包制造A→B边。

### A期PROOF批次（新增、未实施）

最小前置：L983/PORT, L974, L975, L978。实际改面：`D:/autostack/auto-web/crates/auto-web/tests/http_runtime.rs`；`D:/autostack/auto-web/crates/auto-web/tests/http_generation.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：在原TCP fixture给一份原工程接ACE AV服务和新A2R生成服务，读写隔离DB/JSON与真实响应；只接所需最小生产路径，完整AVSERVICE/GENERATED API面和四仓CONSUMERS仍B。
V：两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at](../../../auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at)（下列节选L4–L39；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
pub type CounterResult = { ok: bool, err: str, value: int }

#[api(method = "GET", path = "/value")]
pub fn value() int {
    let marker int = store.version_marker()
    if store.data_root() == "" {
        return 0 - marker
    }
    return marker
}

#[api(method = "POST", path = "/counter/add")]
pub fn counter_add(delta int) CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    let next int = store.add_counter(delta)
    return CounterResult { ok: true, err: "", value: next }
}

#[api(method = "POST", path = "/counter/double")]
pub fn counter_double() CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    let next int = store.double_counter()
    return CounterResult { ok: true, err: "", value: next }
}

#[api(method = "GET", path = "/counter/get")]
pub fn counter_get() CounterResult {
    if store.data_root() == "" {
        return CounterResult { ok: false, err: "data-root-missing", value: 0 }
    }
    return CounterResult { ok: true, err: "", value: store.read_counter() }
}
```

本包所需独立结果：原fixture/API路由要经真实TCP返回独立预期；http_runtime的nonce正文web-package-runtime-890、names含照片.png及ints[-1,-4,-5]全部保；生成服务冷编运行同源。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Auto API路由/参数/鉴权源→公共ACE执行/typed host→既有transport→真实TCP/文件/DB/流→独立客户端。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct WebInvocation { route: RouteId, request: RequestContext, session: SessionId, cancellation: CancelToken }
struct WebHostPort { signatures: SignatureCatalog, owner_thread: ThreadOwner, grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-web/crates/auto-web/src/host.rs](../../../auto-web/crates/auto-web/src/host.rs) | public WebHttpServiceHost端口 |
| [auto-web/crates/auto-web/src/http_server/server_dispatch.rs](../../../auto-web/crates/auto-web/src/http_server/server_dispatch.rs) | 替Rc<AutoVM>与旧handler调用 |
| [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs) | 真实安装消费/端口关闭oracle |

**状态、错误与资源。** 错handle/路由参数/权限/畸形上传和断连精确拒；缺DB/环境保持缺证，不能用内存fakeHTTP或父exit0代子终态。 request/session/DB/file/stream各有owner，取消/晚完成不串请求；shutdown释放后台任务和端口，Core/UI闭包不反引入auto-web-contract。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at](../../../auto-web/crates/auto-web/tests/fixtures/member-delivery/b/src/back/api.at) | 原Auto服务源/路由/状态 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `25dce22ebbb82249` | 拟ace.m2.p983.source01：serve-av-tcp/serve-a2r-tcp；负控/资源见V-DETAIL-02/03；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [auto-web/crates/auto-web/tests/http_runtime.rs](../../../auto-web/crates/auto-web/tests/http_runtime.rs) | 精确JSON/文件/权限/关闭 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `ff449536a73274af` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [auto-web/crates/auto-web/tests/http_generation.rs](../../../auto-web/crates/auto-web/tests/http_generation.rs) | 生成服务独立冷编/TCP | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `899b751e11a1db15` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |
| [crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at](../../crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at) | 客户端/传输公共语义，不能替服务端测试 | `http_client_sync.expected.rs`；SHA256前16 `3ab20dc82b46f669` | 拟ace.m2.p983.source04：serve-av-tcp/serve-a2r-tcp；负控/资源见V-DETAIL-02/03；原任务 B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** PORT先交通用接口，AVSERVICE/GENERATED可按稳定端口并行，CONSUMERS保真实四仓产品；不反等Man全CLI再交公共Web能力。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-PORT-I`、`B-PORT-V`、`B-AVSERVICE-I`、`B-AVSERVICE-V`、`B-GENERATED-I`、`B-GENERATED-V`、`B-PROOF-I`、`B-PROOF-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量
| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-web.md` | 分批沉淀实际支持/来源/拒绝及未闭行 | 不把计划当现状 | AC-01..06及所有AC-B-* |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 959行关联本批新producer/consumer与收据 | 替换追踪 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-web.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-web.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部命令为未来执行，本轮未运行。cwd前缀lang/ui/man/web/ag-examples分别为本Plan对应唯一tree。永久新fixture由I接到所列target；旧用例绿不能证明新ACE支持。参数尚待冻结的产品命令由I第一步读取现有runner/help与工程配置固定，之前不进入V，不声称未来runner当前已可用。

材料/外置target/生成安装副本只放D:/autostack/plan-artifacts/plan-983/下按仓/批次区分，普通target留树。AG仅受影响文件/case、窗口/GPU单实例；原ignore/平台/资源预算保留。发现/执行/排除、准备/运行/清理/总时与子终态分列，零选择/emit-only/未执行不能PASS。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-PORT | AC-B-PORT | B-PORT-V；web: cargo nextest run -p auto-web --lib；cargo nextest run -p auto-web --test http_runtime | 独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。 |
| V-B-AVSERVICE | AC-B-AVSERVICE | B-AVSERVICE-V；web: cargo nextest run -p auto-web --test http_runtime；lang: cargo t http（仅语言相关改动）；完整TCP断言沿现有runtime target | 真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。 |
| V-B-GENERATED | AC-B-GENERATED | B-GENERATED-V；web: cargo nextest run -p auto-web --test http_generation；lang: cargo t auto-generation-core web-delivery-controls | 未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。 |
| V-B-CONSUMERS | AC-B-CONSUMERS | B-CONSUMERS-V；lang: python scripts/web-four-repository.py --out <scope>/web-four；man: cargo test -p auto --test plan890_web_consumers；沿原四仓门核实际子项 | 原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。 |

最终T-05核全批收据适用性、遗漏、组合并集和原资源门；证据身份不变可复用，不反复全仓。每批独立复核不替整包final review，未闭in-scope阻execution_done。


### A期PROOF验证（本轮未运行）

| Case | AC | 实际/拟补入口 | 独立oracle |
|---|---|---|---|
| V-A-PROOF | AC-A-PROOF | web: cargo nextest run -p auto-web --test http_runtime --test http_generation；新增ACE具名case接原target并核实际子进程；与HTTP资源组原串行条件一致 | 两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 原fixture/API路由要经真实TCP返回独立预期；http_runtime的nonce正文web-package-runtime-890、names含照片.png及ints[-1,-4,-5]全部保；生成服务冷编运行同源。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 错handle/路由参数/权限/畸形上传和断连精确拒；缺DB/环境保持缺证，不能用内存fakeHTTP或父exit0代子终态。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | request/session/DB/file/stream各有owner，取消/晚完成不串请求；shutdown释放后台任务和端口，Core/UI闭包不反引入auto-web-contract。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/平台/输入oracle完整 | T-01及各I |
| AC-02 | work | 原整包能力经真实源码/工程运行，范围不减 | 全部I/V |
| AC-03 | work | 全consumer接线与权限/资源/恢复、无legacy fallback，跨仓身份准确 | 全部V及959行 |
| AC-04 | review | 批次及全包独立复核，原AC反例/health/Spec均核 | 各R及T-05 |
| AC-05 | merge | 正式来源/Spec/归档/清理完成 | T-06/07 |
| AC-06 | work | 原T-08补充义务分配至批次且全闭 | 所有AC-B-* |
| AC-B-PORT | work | Web中立执行与资源端口：独立真实TCP consumer正常调用和拒绝错误handle；取消/晚完成不跨请求写入；auto-web-contract不引Core/UI依赖。 | B-PORT-V/R |
| AC-B-AVSERVICE | work | AV真实HTTP服务：真实请求写DB再读；并发上下文不串；断连/上传失败清理；shutdown释放端口与后台任务，原HTTP状态/bytes保持。 | B-AVSERVICE-V/R |
| AC-B-GENERATED | work | 生成Rust服务：未修改生成物冷编译运行真实TCP；同请求独立expected；错误源码不替换旧工程；禁止自报PASS代内部子进程终态。 | B-GENERATED-V/R |
| AC-B-CONSUMERS | work | Web Man UI Down真实消费闭环：原fixture、真实业务请求/资源/退出均匹配；错装载身份/旧绝对路径拒绝；测试环境外部服务缺失具名阻塞。 | B-CONSUMERS-V/R |
| AC-A-PROOF | work | 两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共19任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [ ] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。

- [ ] **B-PORT-I｜M2-A｜owner_stage: work｜AC-B-PORT/AC-02/03/06**：把public host的Rc<AutoVM>/旧native签名替换为中立受检端口，保owner线程、取消、File/Stream所有权和shutdown。 在§5改面接永久fixture，先冻结前置接口/命令。

- [ ] **B-PORT-V｜M2-A｜owner_stage: work｜AC-B-PORT**：运行V-B-PORT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-PORT-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：在原TCP fixture给一份原工程接ACE AV服务和新A2R生成服务，读写隔离DB/JSON与真实响应；只接所需最小生产路径，完整AVSERVICE/GENERATED API面和四仓CONSUMERS仍B。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。

- [ ] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 核子终态与身份，未执行不PASS。

- [ ] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-AVSERVICE-I｜M2-B｜owner_stage: work｜AC-B-AVSERVICE/AC-02/03/06**：动态路由、参数、认证/权限/中间件、上传/下载/stream与DB接ACE，保transport现有实现。 在§5改面接永久fixture，先冻结前置接口/命令。

- [ ] **B-AVSERVICE-V｜M2-B｜owner_stage: work｜AC-B-AVSERVICE**：运行V-B-AVSERVICE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-AVSERVICE-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-GENERATED-I｜M2-B｜owner_stage: work｜AC-B-GENERATED/AC-02/03/06**：新HIR A2R生成服务/共享载体，保route/store/schema/diagnostics与失败保旧；本批可与AVSERVICE在端口稳定后独立。 在§5改面接永久fixture，先冻结前置接口/命令。

- [ ] **B-GENERATED-V｜M2-B｜owner_stage: work｜AC-B-GENERATED**：运行V-B-GENERATED及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-GENERATED-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-CONSUMERS-I｜M2-B｜owner_stage: work｜AC-B-CONSUMERS/AC-02/03/06**：jade-garden与既有Web生产调用逐行接新SDK；每owner代码独立交付，消费相同source/tool版本，保旧四仓义务。 在§5改面接永久fixture，先冻结前置接口/命令。

- [ ] **B-CONSUMERS-V｜M2-B｜owner_stage: work｜AC-B-CONSUMERS**：运行V-B-CONSUMERS及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-CONSUMERS-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-983 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原19任务/全部AC与V/依赖/阶段/预算和drafting 0/19；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部19个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/19；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化4批，0/16。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
