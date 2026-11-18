---
plan_id: PLAN-983
status: executing
feature_name: Web服务嵌入与实际后端迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-18
plan_revision: 11
current_step: 14
total_steps: 20
depends_on: [PLAN-967, PLAN-968, PLAN-969, PLAN-974, PLAN-975, PLAN-978, PLAN-980, PLAN-982]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-web.md]
touched_goals: [GOAL-002]
---

# [PLAN-983] Web服务嵌入与实际后端迁移

## 0. 变更摘要

2026-11-18 r11（范围增补：C0-A 落地 + U0 备例）：①**C0-A 已交**——消费者仓新增可执行 AV 业务源 `auto-down/jade-garden/back/auto/av/workspace_service.at`（`211b6d5`；`api.at` 的 `get_workspace()` 契约桩旁标注指向），auto-web 增 `AvServiceEngine::new_with_grants`（显式 `platform.file/mem/env`）并加真实 TCP 直连验收用例（`56b3ac5`）：`--test http_runtime` 8/8（新例 0.215s），字节级 200 JSON、null 形态、404 派发前拒绝、重复稳定、shutdown 端口重绑。②**纠正 C0 合同错误**：`wiki_dir` 按 Windows `PathBuf::join` 为 `<W>\wiki`（JSON `\\`），原 `<W>/wiki` 已修。③**U0 备例**（980/WEB UI 腿）：002-counter 浏览器 `Counter: 0→1`，oracle 先例=`p980_web_lane_browser_mount`（未运行）。982 CLI 接线后同一 AV 源再由 `auto service` 真 CLI 复验；C0-B 前端页与 U0 运行仍待。

2026-11-18 r10（范围增补）：①关闭 **B-GENERATED-R**——委派独立复核（`gen-reviewer`，与实现会话不同 agent）从 2026-11-17 `fd115ce` 旧双批收据出发，核对 `fd115ce → 3f8de15` 增量：生成道文件/依赖 crate 空 diff、生成服务 `main` 直接 `axum::serve` 不构造 `ServiceEngine`，故不触生成服务验收结论；AC-B-GENERATED 四项逐条 pass，复用同 SHA `http_generation` 3/3 并做两次定向复验（lane 单测 4/4 0.122s、参数例 9.702s、生成+坏源例 6.793s）。②**冻结 983/CONSUMERS 首例 C0**：jade-garden `GET /api/workspace`（无文本参数、两字段 JSON），固定字节级预期/失败面/资源清理与接口清单（983/AVSERVICE 已交付；982/EXEC `--server av` 需改走 auto-web `serve_av`；980/WEB 与 983/GENERATED 本首例不用）。本轮只做复核收据与首例定义，未写消费者代码、未运行新栈请求；B-CONSUMERS 仍未勾。

2026-11-18 r9（范围增补）：依用户指令「先接 `auto_web` 生产栈，由 983 主导；完整 JSON 等第一个真实消费者明确需要哪些字段后再做」，新增具名切片 **B-AVSERVICE-TEXT**：把 982 已验的受检文本入口（`fn main(query str, body str) int`）接到 auto_web 既有 TCP 生产栈——**同一份已编译 handler**收查询串 + UTF‑8 体，请求不重编译、不经文件；等字节长度 `中文`/`日文` 按内容分流，收据 `av_dispatches>0 / legacy_dispatches=0 / compile_count=1`，现有服务回归保持。**未做**：文本参数与平台 I/O 的同一次调用（未触发；触发时交 982 共享入口补齐）、完整 JSON/请求头（等首个真实消费者字段）。原19任务/AC/进度保留，总数 20、12 完成；本次只实施+验证这一切片并落规范增量，review 仍归 B-AVSERVICE-R/T-05。

2026-11-16 r7（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包仍完成现有Web服务/前后端实际消费者及RG证明；消费976/980所需TS/JS闭包，不将Godot或独立无消费的a2ts产品作为整包门，也不删Vue/Web现有业务。原19任务/进度保持。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

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

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

## 5. 详细设计

### CONFIG-ACE-B交接（2026-11-17）

[982/B-CONFIG](982-ace-m2-man-gen.md)现统一生产配置文档Schema/AV/owned SDK与Man接口，见[设计](../design/ace-config-document-evaluation.md)。本包原19任务/已交PORT和PROOF A面保持。AVSERVICE/GENERATED按原端口先行；CONSUMERS用到配置的工程须收IF-982-CONFIG-ACE＋982/EXEC＋980/WEB对应正式接口，不把同SDK的legacy配置owner冒新引擎。

CONSUMERS I/V/R负责每个真实配置入口/dep参数/env/锁、坏配置零写/修复下一读和实际服务结果；配置算法由982生产，本包只接真实Web/Man/UI/Down consumer，不重复parser/求值器。982 CONFIG不等本包或980/WEB完成才交能力；本包CONSUMERS也不因只缺某产品接口阻已就绪服务批。全部required最终齐才原T-05/06/07独立收口。

### M2-TARGET-SCOPE-20261116：本包当前边界

本包仍完成现有Web服务/前后端实际消费者及RG证明；消费976/980所需TS/JS闭包，不将Godot或独立无消费的a2ts产品作为整包门，也不删Vue/Web现有业务。原19任务/进度保持。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

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

#### 983/CONSUMERS 首例冻结（2026-11-18，C0：jade-garden `GET /api/workspace`）

**选择与理由。** 从 jade-garden（auto-down）现有生产调用取一条真实请求作首例：`GET /api/workspace`。它是应用启动/桌面 daemon 的既有调用（[front/src/lib/api.ts](../../../auto-down/jade-garden/front/src/lib/api.ts) `getWorkspace()`；e2e [01-workspace.spec.ts](../../../auto-down/jade-garden/front/e2e/01-workspace.spec.ts) 与 [global-setup.ts](../../../auto-down/jade-garden/front/e2e/global-setup.ts) 已对真实后端断言），**无查询串/请求体文本**（不触发 982 的「文本参数＋平台 I/O」组合缺口），响应只有两个可空字符串字段（root/wiki_dir）——正是「首个用例只读少量字段就先交那部分」的最小 JSON 面。

**真实请求与预期（字节级）。** 后端固定端口 `18181`（e2e `BACKEND_PORT`），fixture 工作区 `front/e2e/.runtime/workspace`（正斜杠形态 `WORKSPACE_ROOT`）。请求行 `GET /api/workspace HTTP/1.1`、`Host: 127.0.0.1:18181`、`Connection: close`、空体。
- 配置了工作区（`JADE_GARDEN_DEFAULT_WORKSPACE=<W>`）⇒ `200`、`content-type: application/json`、体逐字节 = `{"root":"<W>","wiki_dir":"<W>\\wiki"}`——**wiki_dir 用 Windows `MAIN_SEPARATOR`**：`PathBuf::from("<W>").join("wiki")` 实测为 `<W>\wiki`（`<W>` 以 `/` 或 `\` 结尾时才直接拼 `<W>wiki>`），JSON 中写作 `\\`；字段序 root→wiki_dir（serde_json struct 序），路径取 `to_string_lossy` 语义。
- 未配置（env 缺省且无配置文档）⇒ `200` + `{"root":null,"wiki_dir":null}`（与 01-workspace.spec.ts 的拦截 oracle 同形）。
- 工作区目录不存在 ⇒ 仍返回该路径与 `<W>/wiki`（原 `get_workspace_impl` 不校验存在性；校验只在 POST /open）。

**失败面。** 未声明路由（如 `GET /api/nope`）⇒ `404` 空体（AV 服务臂与 legacy 预处理同形；不回落 `d_main`、不回退 VM）；畸形请求 ⇒ 传输层 400/断开且不派发；AV 执行诊断 ⇒ `500` 具名 `ace.service/*`，不得静默 200。

**资源与清理。** 只读配置/env 与工作区路径：不写工作区、不建/改配置文件；每请求文件句柄关闭（AV 平台台账「每调用强制关句柄」）；连续 2+ 请求体字节相同且 `av_dispatches` 累加、`legacy_dispatches=0`；外部停止 ⇒ 排水、端口可重绑、进程退出 0。

**接口清单（实际用到 / 明确不用）。**

| 接口 | 状态 | 本首例用途 |
|---|---|---|
| 983/AVSERVICE：`auto_web::serve_av` + `AvServiceEngine`（文件通道；`#[api]` 路由、404/501、shutdown；2026-11-18 增 `new_with_grants` 显式 grants） | **已交付**（auto-web `3f8de15`；直连切片 `56b3ac5`，m2-web.md B 节） | 承载 `/api/workspace` 的 AV 程序与真实 TCP/配置/生命周期 |
| 982/EXEC：`auto service --server av`（`resolve_back_api`、真实 TCP、ready 行、receipt） | **部分交付**（982 rounds 149–166；man master `1d26b50`）：现为 man 自带最小 TCP 宿主（文本入口/int 返回，无 auto-web transport/配置/grants） | 需改走 auto-web `serve_av` 并暴露 grant/scratch/入口（见下方 982 请求） |
| 982/CONFIG：服务配置 `--http-config-inline`（listen/profile/drain） | **已交付**（982/EXEC 面）；配置文档引擎本请求**不用**（jade-garden 后端无 pac.at；本轮配置入口=env+服务配置） | listen/profile/drain/退出；坏配置具名失败、旧工作区零写 |
| 980/WEB：Vue/TS 表达式后端 | **本首例不用**：jade-garden 前端由 a2ts 生成 + 手写 fetch，不经 auto-ui Vue 表达式面；其未闭项不阻塞 C0，四仓整体验收的 UI 腿须另立 case | — |
| 983/GENERATED：A2R 生成服务 | **本首例不用**：生成道无平台 I/O/配置读取；生成服务腿由既有 `http_generation` 用例与后续 case 覆盖 | — |

**C0-A 实现与直连验收（2026-11-18 已交）。** 可执行业务源=auto-down `jade-garden/back/auto/av/workspace_service.at`（提交 `211b6d5`；`api.at` 的 `get_workspace()` 契约桩旁已标注指向）：读 `JADE_GARDEN_DEFAULT_WORKSPACE`（`env_get_w16` UTF-16LE）→ UTF-8/JSON 转义（`"`、`\`、控制字符 < 0x20）→ 写响应文件；`wiki_dir` 按 Windows join 语义拼 `\wiki`；只读、每请求关句柄。宿主直连=`AvServiceEngine::new_with_grants`（`platform.file/mem/env`；默认 `new` 授权面不变，提交 `56b3ac5`）。真实 TCP 用例 `http_runtime.rs::c0_workspace_p983::jade_workspace_route_serves_exact_json_from_the_consumer_av_source`（0.215s）：200 字节级 JSON（含 `\\wiki`）、重复请求稳定、未配置 ⇒ `{"root":null,"wiki_dir":null}`、`/api/nope` ⇒ 404 空体（派发前）、shutdown 端口重绑。用例**直接读消费者仓源**（`AUTO_DOWN_ROOT` 或相对 `../../../auto-down`，缺失具名失败），不复制；`--test http_runtime` 8/8 绿（7.92s）。**982 CLI 接线到位后**同一源再由 `auto service` 真 CLI 复验（未做）。

**分期。** C0-A 后端线已交（上段）。C0-B：前端页（`getWorkspace()` 经浏览器/桌面形态消费同一响应；jade-garden 前端产物构建面另核）待验。C1（下一例，将触发 982 组合缺口）：`POST /api/workspace/open`——文本根路径 + 目录创建/索引重建（平台 I/O）+ JSON 入/出，需 982 在同一次调用里组合文本参数与平台 I/O。

**U0（UI 腿，980/WEB 消费；2026-11-18 备例）。** 真实交互例=`auto-ui/examples/ui/002-counter`（[app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at)，初始 `Counter: 0`）：浏览器加载生成的 Vue 页面，点 `+` ⇒ `Counter: 1`，`-`/`Reset` ⇒ `Counter: 0`；DOM 文本逐字断言、无控制台错误。oracle 先例=auto-ui-runtime 浏览器挂载档 `p980_web_lane_browser_mount`（真 002 源 `.Inc` lane 体，41→42；`examples/ui/ace-web-lane/tests/test_lane_browser.mjs`，排窗 `AUTO_UI_ACE_WEB_E2E=1`）。接口：980/WEB——`ui_gen/vue` SFC/表达式与 lane handler 体（切片 #1–#6p 已交；**B-WEB-I/V/R 未勾**，余=真语料全量 spliced 切换 watch/setup/expose/ext_imports/style）；982/EXEC——产品形态需 `auto run`（Vue）真 CLI，候选腿当前对 run 家族具名拒绝、ACE 服务化未交（与 C0 同一 982 交接）；配置=`pac.at`（002 有）+ 应用窗口配置，实际字段随 CLI 腿冻结；983/AVSERVICE/GENERATED 不参与。失败/清理：坏源保旧产物、构建诊断具名；浏览器/服务子进程回收、端口释放、生成 tree 不手改。**现状**：只备例，未运行浏览器档；four-repo 的 UI 腿等 980/WEB 批勾选与 982 `auto run` 接口。

**给 982 的优先请求（EXEC 接口）。** ①`auto service --server av` 的宿主换成 auto-web `serve_av`（同 transport/config/shutdown/ready/receipt；`AUTO_AV_*` 保留为引擎选项）；②API 源路径可显式指定（默认发现 `src/back/api.at` 不变；jade-garden 用 `back/auto/api.at`），无 API 仍具名拒绝、不回退 vm；③暴露 AV 引擎的 grant/scratch（`platform.file`/`platform.mem`，jade-garden 首例还需 `platform.env`）与程序入口；④`--http-config-inline`/配置语义同 `serve_vm`，坏配置零写、失败具名。C/EXPORT 等按自身顺序并行。

**不声称。** C0-A 只证「同一 AV 业务源 + 直连宿主」的字节级语义；982 CLI 腿、C0-B 前端页、U0 运行与四仓整体验收（980/WEB、982/EXEC、实际用到的配置接口）均未完成，按 §6 V-B-CONSUMERS 待所用接口齐备后收口。

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
| SD-CFG-HANDOFF | modify（拟，原owner） | `docs/specs/ace-migration/m2-web.md` | 原候选接收→实际配置新AV来源/宿主与guest边界、错误恢复/原平台，真实交付后才写已支持 | 982生产一次，本包只核实际消费者/安装退役阶段；A收据保留 | AC-CFG-HANDOFF、原AC与最终review |
| SD-01 | add | `docs/specs/ace-migration/m2-web.md` | 分批沉淀实际支持/来源/拒绝及未闭行 | 不把计划当现状 | AC-01..06及所有AC-B-* |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 959行关联本批新producer/consumer与收据 | 替换追踪 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-web.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-web.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-web.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

## 6. 测试设计

### V-CFG-HANDOFF（本轮新增接收控制，未执行）

沿[配置设计九行Case](../design/ace-config-document-evaluation.md#6-验证合同与旧用例映射)，在本包所用真实consumer/原测试入口验合法动态值及坏源/错SDK-context/零旧求值/失败保旧/修复下一次read；把route C legacy config冒ACE、只核same_sdk、缺source或仅用静态投影则拒绝。原V全部保留，不新造runner。实际未用配置的域不增加虚假依赖；原平台/ignore/输入规模保持。

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

- [ ] **AC-CFG-HANDOFF / work**：§5本包实际配置consumer/阶段的正式来源和V-CFG-HANDOFF闭合，区分宿主AV与目标业务、原A与完整B；不以未交接口/上游状态或peer-equal代真实consumer与独立oracle。原工作任务负责，原final独立复审核验。

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

当前共20任务（14 完成）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。
  [✅ 2026-11-17 T-01 冻结] 前置核对：A最小前置 L967/968/969/974/978 全部 ARCHIVED（archive/ 目录实测）；L975 亦 ARCHIVED。owner/AGENTS：auto-web `AGENTS.md` 已读（唯一 Plan worktree、无 symlink、纯合同无 Core/UI 依赖、wt-guard 清树）。基面：auto-lang master `e85096628`、auto-web master `0adebc0`（两仓 clean，主检出仅 docs WIP）。A能力冻结（B-PORT=公共 host 面中立受检端口；B-PROOF=RG-A02 两腿）：①AV 腿 runner=既有 `ace_av::console_host::compile`+`run_platform`（net974 同款，非第二 runner；`run_ace` E0 无 platform.net 道已核 ace_entry.rs/桥三车道）；②A2R 腿=沿公开 `auto_web::generate_api` 补显式新引擎道（ACE 前门→SemanticView→a2x-rust lower；无静默回退，域外具名拒；sqlite/fs/env/store 留旧道=B-GENERATED，具名不冒）；③PROOF fixture：`auto-web/tests/fixtures/ace-p983/`（AV 服务程序 + A2R 纯 handler 工程），永久用例接原 target `http_runtime.rs`/`http_generation.rs`；④依赖：auto-web 增 dev-dep `ace-av`/`ace-av-core`（根 workspace path 模式同 auto-lang-ace-host）；auto-lang 侧改面=auto-generation-core 新道 seam（组内兄弟树）。V 命令冻结=V-B-PORT（web: nextest -p auto-web --lib；--test http_runtime）与 V-A-PROOF（web: nextest -p auto-web --test http_runtime --test http_generation），串行条件沿原 HTTP 资源组；新增用例逐条计时（>10s 的编译绑定用例如实记录并按 2026-11-14 裁定在收据中显名）。

- [x] **B-PORT-I｜M2-A｜owner_stage: work｜AC-B-PORT/AC-02/03/06**：把public host的Rc<AutoVM>/旧native签名替换为中立受检端口，保owner线程、取消、File/Stream所有权和shutdown。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @93712e5（auto-web plan-983-dev）] 新增 `web_host_port`（WebHostedService/WebListenRequest/WebListenKind/WebRouteDecl/WebPortError：控制面冲突/空地址/owner 单次消费命名拒绝 + 静态 admit）；`host.rs` 公共 `listen_borrowed`/`serve_owned` 重定型为端口类型（core legacy trait 实现留内部桥经同一准入面）；`http_server_handle::serve_server_handle` 公共边中立化（legacy 臂 `serve_server_handle_legacy` crate 内私有）；`http_transport.rs` 冻结面文档化（签名本已引擎中立）。行为保持：owner 线程/取消/File-Stream/shutdown 机制未动。

- [x] **B-PORT-V｜M2-A｜owner_stage: work｜AC-B-PORT**：运行V-B-PORT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @93712e5] V-B-PORT 冻结命令实测（.wt/lang-983/auto-web，基面 auto-lang e85096628/auto-web 0adebc0）：`cargo nextest run -p auto-web --lib --no-fail-fast` = 184 例 183 绿/1 红/1 skip，17.76s；红 = `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，**基线 0adebc0 原样红（stash 复证），非本批引入，具名既有红**。`--test http_runtime` = 3/3 绿，5.94s（真实 TCP 消费正常调用+shutdown join+port 释放+上传 413 拒绝面）。新增端口单测 4 例（控制面冲突/空地址/单次消费/中立路由视图）各 ≤0.021s。`cargo tree -p auto-web-contract` = 仅 serde/serde_json，无 Core/UI 依赖。取消/晚完成不跨请求写入：plan730 cancel/timeout/lease 清理用例绿（lib 门内）。

- [x] **B-PORT-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。
  [✅ 2026-11-17 phase review pass @bdc48e8] 见 §9 phase 复核收据（review_scope=phase；F-R1 格式发现→work 修复 bdc48e8→复验）。

- [x] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：在原TCP fixture给一份原工程接ACE AV服务和新A2R生成服务，读写隔离DB/JSON与真实响应；只接所需最小生产路径，完整AVSERVICE/GENERATED API面和四仓CONSUMERS仍B。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。
  [✅ 2026-11-17 @35b9e47（auto-web plan-983-dev）] AV 腿：永久 fixture `tests/fixtures/ace-p983/av-service.at`（member-delivery/b API 面：/value、/counter/add|double|get + 自有 /__auto/shutdown；counter.txt 持久化经 `file_open_w16`（UTF-16LE staged，Win32 面 file_open_bytes 为 Linux-only 已核）+ file_read/write_some；单 owner 串行 accept 与 legacy 同步 server-handle 同构）；http_runtime.rs 新增 2 用例（`ace_av::console_host::compile`+`run_platform`，net974 同款 runner，ace-av/ace-av-core dev-dep）。A2R 腿：新增 `ace_a2r_lane`（ACE 前门 `compile_text_record_source`+`ac.runtime.text.v1`（cert_rt_902 实证配对）→SemanticView→a2x-rust lower_program→record-text 文件集；显式 `AUTO_WEB_A2R_ENGINE=ace`，子集外具名拒绝不静默回退）；api_gen.rs 早子集门（零参 int/str/bool、无 db.at/伴生）零写入拒 + ACE 道写 a2x 双模块 + api.rs 全 handler 委派 generated_ace + main.rs mod 声明；fixture 工程 `tests/fixtures/ace-p983/a2r-service/` + http_generation.rs 新增 1 用例。生产依赖新增 ace-core/ace-a2x-core/ace-a2x-rust（发射侧；解释器 ace-av 保持 dev-only）。

- [x] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录两腿各对独立HTTP bytes/状态/DB结果，大整数不损；并发请求不串、断连/取消/关端口资源释放；生成物不手改，坏源保旧工程。 核子终态与身份，未执行不PASS。
  [✅ 2026-11-17 @35b9e47] V-A-PROOF 冻结命令实测：`cargo nextest run -p auto-web --test http_runtime --test http_generation` = **7/7 绿**（30.59s）。新增用例逐例计时（≤10s 门）：ace_lane 9.14s、ace_av_service 0.86s、ace_av_cancel 0.63s；lane 单测 3 例绿。AV 腿 oracle 实测：独立 HTTP bytes/状态（200/404/202 精确、CounterResult JSON 逐字节）、隔离 DB 独立读回（counter.txt="2147483647" 测试直读）、大整数 i32 全宽 2147483647 精确、断连（半请求+关）不触业务态、cancel（CancelFlag）→ Cancelled+端口重绑、shutdown 202+join+端口重绑。A2R 腿 oracle 实测：未手修生成物断言（generated_ace.rs/a2x_runtime.rs 在场+api.rs 委派）、坏源（带参端点）具名 zero-param 拒+旧 workspace manifest 字节不变、--offline 冷编译真实 cargo 子进程终态、真 TCP 独立 expected（890/"p983-ace-a2r-service"/2147483647）、202 shutdown join+端口释放。lib 回归 187 例 186 绿（唯一红=具名既有红 test_handler_body_r2_matrix_compile_oracle，基线同红）。既有 legacy 用例（cold_generated 14.24s、installed_host 等）保持绿——默认道行为不变。

- [x] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。
  [✅ 2026-11-17 phase review pass @bdc48e8] 见 §9 phase 复核收据；985/A 交接待 merge 阶段 fold 合入后生效（本勾选仅关 A 切片复核义务）。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [x] **B-AVSERVICE-I｜M2-B｜owner_stage: work｜AC-B-AVSERVICE/AC-02/03/06**：动态路由、参数、认证/权限/中间件、上传/下载/stream与DB接ACE，保transport现有实现。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @76c8001（auto-web plan-983-dev）] 新增 `av_service` 模块：`AvServiceEngine`（console 前门编译驻留 + Win32PlatformIo 与 `Rc<AutoVM>` 同律驻 owner 线程 + `#[api]` 路由抽取（剥属性行后交前门）+ 文件通道：每请求 staged 原始 HTTP 请求文件→`run_platform`（grants platform.file/mem；与 AV 平台账"每调用强制关句柄"纪律天然兼容）→响应文件读回）。http_server 增 `ServiceEngine` 枚举（Legacy 段驱动全量保留 / Av 同步完成永不 park——parked 机制自然旁路）与 `serve_av` 公共入口；server_dispatch 增引擎级派发 + AV 臂（CORS/404 与 legacy 逐字同形；**具名 501 边界**：已注册中间件链/上传路由——语言侧凭据消费通道缺位；限流=legacy 专有 VM launch settings 面）。**冻结设计依据**：AV 公开面无「具名函数+参数+平台IO」联合入口（入口参数仅标量无 Str、每调用 mint 新 ledger 强制关文件、session/snapshot !Send）——侦察报告已核 av-core av.rs/av.rs resident/session。永久 fixture `tests/fixtures/ace-p983/av-runtime-service.at`（member-delivery/b API 面 + counter.txt 持久化 + #[api] 声明）+ http_runtime 新用例。认证/中间件/上传/stream/SSE/`~T` 面：具名未执行（R 条款），非 NA。

- [x] **B-AVSERVICE-V｜M2-B｜owner_stage: work｜AC-B-AVSERVICE**：运行V-B-AVSERVICE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @76c8001] V-B-AVSERVICE 实测：`nextest -p auto-web --test http_runtime` = **6/6 绿**（7.22s；含 legacy installed-host 双例证 transport/legacy 臂零变化 + A 腿两例仍绿）。新用例 `av_service_engine_serves_axum_transport_over_real_tcp`（0.407s ≤10s 门）oracle 全实测：真 TCP 经既有 axum transport + AV 引擎执行；写 DB 再读（add7→get7→add3→10→double20→+100=120）；并发到达经 owner 串行不串上下文；断连（半请求+关）不触业务态；404 与 legacy 预处理同形（空体+请求 id 头）；隔离 DB 文件测试直读（counter.txt="120"）；外部 stop→排水→端口重绑+service 完结。lib 186/187（1 红=具名既有红）。lang `cargo t http`：lang 仓零改动，按 V 行"仅语言相关改动"条款免跑（记录）。

- [x] **B-AVSERVICE-TEXT｜M2-B（2026-11-18 用户裁定切片）｜owner_stage: work｜AC-B-AVSERVICE/AC-02/03/06**：把 982 的受检文本入口接到 **auto_web 生产栈**——同一份已编译 handler 直接收查询串 + UTF‑8 体（`fn main(query str, body str) int`），真实 TCP 路由；请求不重编译、不经文件；断言 `av_dispatches>0`、`legacy_dispatches=0`、产物指纹不变并跑现有服务回归。文本参数 + 平台 I/O 的组合调用与完整 JSON 不在本切片（见证据/§10）。
  [✅ 2026-11-18 @3f8de15（auto-web plan-983-dev；lang 组树 ff `850a49bf8` 零代码改动，规范提交 `b016ff9d3`）] 交付：①新 `av_text_service` 模块——`AvTextServiceEngine::new` 启动时**编译一次**（Text profile + `ac.runtime.text.v1` → `lower_checked_owned` → `verify_mir` → ABC encode/decode → `verify_abc`；`compile_count` 恒 1、ABC 指纹固定），请求级只调 `ace_av_core::av::execute_entry_text(&[query, body])`（982 受检入口；文本在本次执行 arena 内借用，非法 UTF‑8 在执行前具名拒）；查询串取原始 target 的 `?` 之后并按 `%XX`/`+` 解码（传输层给的是百分号编码 ASCII），体为原始 UTF‑8 字节；`int` 返回 ⇒ 200 + 十进制文本，Cancelled/Resource ⇒ 504，Trap/Error/Infra/Exit ⇒ 500 具名；`AvTextReceipt` 暴露 compile/av/legacy 计数与产物指纹。②接线：`http_server::serve_av_text` + `ServiceEngine::AvText`（transport/owner 线程原样、`legacy()`=None、无 legacy 路径），`server_dispatch` 文本臂——中间件/上传 501 与未命中 404 与文件通道 AV 臂逐字同形，miss **不派发**。③永久 fixture `tests/fixtures/ace-p983/av-text-service.at` + `http_runtime.rs` 新用例 `av_text_entry_reuses_one_compiled_handler_and_branches_by_content`（真实 TCP）。**实测**：等字节长度 `中文`/`日文`（各 6 UTF‑8 字节）按内容分流——`GET /text?<中文>`→3、`?<日文>`→4、`POST /text` 体 中文→1/日文→2、查询+体组合体优先→2；`/text?%FF`→500 `not valid UTF-8`（执行前拒）；`/missing`→404；收据 `av_dispatches=6 legacy_dispatches=0 compile_count=1 artifact_fingerprint=9db3cce95d63f38e entry=d_main`（6 次=5 成功+1 入口内受检拒绝；请求间指纹不变=同一编译产物）。**回归**：`--test http_runtime` 7/7 绿（7.48s；新例 0.60s ≤10s 门）、`--lib` 188 例 187 绿/1 红（21.39s，红=基线既有 `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，非本切片引入）、`--test http_generation` 3/3 绿（176.43s，默认道不变）。**未做（具名）**：①文本参数 + 平台 I/O 同一次调用——983 首消费者本轮只用文本，未触发；触发时按用户指令交 982 共享入口验证补齐（982 收据仅覆盖文本参数）；②完整 JSON/嵌套/浮点/字符串/请求头——按首个真实消费者字段推进；③中间件/上传/stream/SSE/`~T` 仍具名 501。**规范**：`m2-web.md` 增「B-AVSERVICE 文本入口」节（lang 树 `b016ff9d3`；已随 merge rebase 落主 `043542b2e`/`1af135c31`，见 §9 merge 收据）。

- [x] **B-AVSERVICE-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。**复核面含 2026-11-18 文本入口切片 B-AVSERVICE-TEXT**（`av_text_service`/`serve_av_text`/fixture/真实 TCP 用例与收据断言）。
  [✅ 2026-11-18 review pass @3f8de15] 见 §9 复核收据（review_scope=phase，含文本入口切片）。冻结基线复跑：http_runtime 7/7（新例 0.172s）、lib 187/188（1 红=基线既有 `api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，文件不在 diff）、http_generation 3/3；`git diff --check` clean、新文件 0 warning/0 调试残留。F-R3（规范状态行陈旧任务计数）→ work 修复 `b952cc9a4` 后复验；无 in-contract 缺陷。落主：auto-web master `3f8de15`、auto-lang master `1af135c31`（见 §9 merge 收据）。B-GENERATED-R 与 T-05..07 仍独立执行。

- [x] **B-GENERATED-I｜M2-B｜owner_stage: work｜AC-B-GENERATED/AC-02/03/06**：新HIR A2R生成服务/共享载体，保route/store/schema/diagnostics与失败保旧；本批可与AVSERVICE在端口稳定后独立。 在§5改面接永久fixture，先冻结前置接口/命令。
  [✅ 2026-11-17 @6808511] lane 子集扩面：`int`/`str` 视图/`bool` **参数**（值传；a2x `l{idx}_` 局部名还原源参数名）与同族返回；owned/mut str 参数、record/list/浮点具名拒（后续扩面；`str+literal concat 产 owned Str 与 str 返回不匹配`=verify 具名拒已实证，恒等体为子集内形态）。api_gen：端点门放宽至参数类型 {int,str,bool}；`generate_ace_api_rs` 发射参数提取器（GET/DELETE=Query struct、POST/PUT=Json body，与 legacy 委派形态同源）+ 实参编组（int `as i32`/str `as_str`/bool）。保旧面：route（路由注册沿用端点驱动）、store/db（ACE 道仍具名拒 db.at/伴生——默认道 store 支持不变，member_delivery 由 V 证）、schema（types.rs/generation.json 字段不变）、diagnostics/失败保旧（早门零写入+receipt park+write_if_changed 原样；坏源负例改 scale(float) 具名拒+旧 workspace manifest 字节不变）。fixture 扩 echo/whisper/pick 参数端点。

- [x] **B-GENERATED-V｜M2-B｜owner_stage: work｜AC-B-GENERATED**：运行V-B-GENERATED及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。
  [✅ 2026-11-17 @6808511] V-B-GENERATED 实测：`nextest -p auto-web --test http_generation` = **3/3 绿**（73.5s）。新例 `ace_lane_param_endpoints_execute_over_real_tcp`（6.45s ≤10s 门）：生成→参数 glue 断言（fn_echo(body["delta"]..)）→--offline 冷编译（真实 cargo 子进程终态）→真 TCP：echo(41)→82、whisper("p983")→"p983"（str 参数编组）、pick(true)→true（bool）→202 shutdown join+端口释放；原例（6.21s）保持含坏源负例（scale(float) 具名拒+保旧）。既有 `cold_generated`（legacy 道，60.7s）与 lane 单测 4/4 绿=默认道不变。lang 组（组树零改动非回归证据）：`cargo t auto-generation-core` PASS + `cargo t web-delivery-controls` PASS（冷建 3m16s，组树 @d43e7970+master 前移）。

- [x] **B-GENERATED-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。
  [✅ 2026-11-18 phase review pass @3f8de15（委派独立复核 `gen-reviewer`）] 从 2026-11-17 `fd115ce` 旧双批收据出发：`fd115ce→3f8de15` 八文件增量不含 `ace_a2r_lane.rs`/`api_gen.rs`/`http_generation.rs`/`auto-generation-core`，生成服务 `main` 直接 `axum::serve`（`api_gen.rs:5310`）不构造 `ServiceEngine` ⇒ 不触生成验收结论；AC-B-GENERATED 四项逐条 pass（冷编真 TCP/独立 expected/坏源保旧/子进程终态无自报）。证据：复用同 SHA `http_generation` 3/3（118.56s）＋定向复验 lane 单测 4/4（0.122s）、参数例 9.702s、生成+坏源例 6.793s；F-G1（低，非 AC 缺陷）首跑 19.46/22.53s>10s 系冷建+并发盘压，稳态 ≤10s，owner=B/CONSUMERS 预构建（r7 已名）。见 §9 复核收据。

- [ ] **B-CONSUMERS-I｜M2-B｜owner_stage: work｜AC-B-CONSUMERS/AC-02/03/06**：jade-garden与既有Web生产调用逐行接新SDK；每owner代码独立交付，消费相同source/tool版本，保旧四仓义务。 在§5改面接永久fixture，先冻结前置接口/命令。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。
  [C0-A 已交 2026-11-18 @211b6d5（auto-down）/ @56b3ac5（auto-web）] 见 §5「C0 首例冻结」：可执行 AV 业务源 `jade-garden/back/auto/av/workspace_service.at`（env→UTF-8/JSON→响应；Windows join `\wiki`；只读）+ `api.at` 指针；auto-web `AvServiceEngine::new_with_grants` 显式 grants；真实 TCP 直连用例（源读消费者仓、不复制）——`--test http_runtime` 8/8（7.92s；新例 0.215s），字节级 200 JSON / null 形态 / 404 派发前 / 重复稳定 / shutdown 端口重绑。**仍未勾**：982 CLI 腿（待接线）、C0-B 前端页、U0（980/WEB UI 腿）运行与四仓整体验收。

- [ ] **B-CONSUMERS-V｜M2-B｜owner_stage: work｜AC-B-CONSUMERS**：运行V-B-CONSUMERS及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

- [ ] **B-CONSUMERS-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-PORT-I/B-PORT-V/B-AVSERVICE-I/B-AVSERVICE-V/B-GENERATED-I/B-GENERATED-V/B-PROOF-I/B-PROOF-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。


## 9. 复审记录

2026-11-18 stage: work | PLAN-983 r11 | outcome: pass（**C0-A 切片：AV 业务源 + 直连验收**；B-CONSUMERS 仍未勾）| 用户指令：现在就做 C0-A 的 AV 业务源和直连验收；`api.at` 的 `get_workspace()` 契约桩需可执行实现；982 接线后再跑真 CLI + C0-B；另备一例消费 980/WEB。| worktrees: auto-down `D:/autostack/.wt/lang-983/auto-down`（`211b6d5`）、auto-web `D:/autostack/.wt/lang-983/auto-web`（`56b3ac5`）；auto-man 组树已建（`60321f9`，未改）待 CLI 腿 | 交付：①`jade-garden/back/auto/av/workspace_service.at`——读 `JADE_GARDEN_DEFAULT_WORKSPACE`（`env_get_w16`）→ UTF-8/JSON 转义 → 响应；`wiki_dir` 按 Windows join 拼 `\wiki`；只读、每请求关句柄；`api.at` 契约桩旁标注指向。②auto-web `AvServiceEngine::new_with_grants`（显式 `platform.file/mem/env`；默认 `new` 不变）。③真实 TCP 直连用例（auto-web `http_runtime.rs::c0_workspace_p983`；源读消费者仓，`AUTO_DOWN_ROOT` 或相对 `../../../auto-down`，缺失具名失败）。| evidence: `--test http_runtime` **8/8 绿 7.92s**（新例 0.215s）；字节级 200 JSON（含 `\\wiki`）、重复稳定、未配置 ⇒ `{"root":null,"wiki_dir":null}`、`/api/nope` ⇒ 404 空体（派发前）、shutdown 端口重绑；`cargo fmt --check` clean；回执行 `✓ C0-A workspace receipt: exact JSON ...; null form; 404 pre-dispatch`。| 合同纠正：C0 的 `wiki_dir` 预期由 `<W>/wiki` 更正为 `<W>\wiki`（Rust `PathBuf::join` Windows 实测；JSON `\\`）——原文本错误已修（r11 同次）。| U0 备例：002-counter 浏览器 `Counter: 0→1`（含 `-`/`Reset` 回 0），oracle 先例 `p980_web_lane_browser_mount`（41→42，`test_lane_browser.mjs`）；**未运行**，前置=980/WEB 批勾选 + 982 `auto run` 接口。| 未做：982 CLI 腿、C0-B 前端页、U0 运行、四仓整体验收分母不变。| next: 982 `auto service --server av` 走 auto-web `serve_av` → 同源 CLI 复验；C0-B；U0 待前置

2026-11-18 stage: review | review_scope: phase（B-GENERATED-R；整包仍 executing，CONSUMERS/T-05..07 未动）| reviewer_identity: 委派独立复核（gen-reviewer；与实现会话不同 agent、同机；旧收据延续+同 SHA 复用已声明）| plan_id: PLAN-983 | plan_revision: r9 | outcome: pass | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: 3f8de15（auto-web，=master tip；复核前后 clean）| base_commit: fd115ce | dependency_revisions: lang 组树 1af135c31（auto-generation-core/auto-lang-core/ace-{core,a2x-core,a2x-rust,av,av-core,lowering} 对 850a49bf8 空 diff；套件跑于 850a49bf8，代码等同）| spec_inputs: docs/specs/ace-migration/m2-web.md（B-GENERATED 节、文本入口节）| delta_impact: fd115ce→3f8de15 八文件 +547/−1，无 ace_a2r_lane.rs/api_gen.rs/http_generation.rs/auto-generation-core/lane 依赖 crate；生成服务 main 走 axum::serve（api_gen.rs:5310），不构造 ServiceEngine，故不影响本批验收 | acceptance_results: AC-B-GENERATED pass——①未手改生成物冷编真 TCP（双模块在场+api.rs 委派；cargo build --offline 子进程 success；spawn exe 真 TCP 200/202/端口重绑）②独立 expected 890/"p983-ace-a2r-service"/2147483647/82/"p983"/true ③坏源 scale(float) 具名拒含端点名+member manifest 字节不变 ④真实 cargo 子进程终态与产物退出码断言，无自报布尔 | findings: F-G1（低，非 AC 缺陷）新 ACE 例首跑 19.46/22.53s>10s（冷 cargo+新 ace-lowering 首建+并发盘压），稳态复验 6.793/9.702s≤10s；owner=B/CONSUMERS 预构建（r7 已记）；无 in-contract 缺陷；探针误用 `-E test(/.../)` 选 0 例已剔除（非产品）| evidence: lane 单测 4/4 0.122s；参数例 PASS 9.702s；生成+坏源例 PASS 6.793s；复用 3f8de15 套件 3/3 118.562s（19.464/22.529/76.562）、lib 188 例 187 绿 63.662s（唯一红=基线既有且文件不在 diff）| pending_closeout_gates（merge-owned）: B-GENERATED 已随 fd115ce phase fold 落主、文本切片 3f8de15 已落主；T-06/T-07 待 CONSUMERS | next: work/merge 按 owner（本批无修复项；F-G1 归 B/CONSUMERS 预构建）

2026-11-18 stage: plan（首例冻结，非实施）| PLAN-983 r10 | outcome: pass（合同冻结；B-CONSUMERS 仍未勾）| 用户指令：先收 B-GENERATED-R，同时为 B-CONSUMERS 选第一个真实业务用例；982 优先交首例实际需要的 EXEC 接口。本轮交付：①B-GENERATED-R 复核 pass（见上一条）；②§5「983/CONSUMERS 首例冻结」——C0 = jade-garden `GET /api/workspace`：无文本参数、两字段 JSON（root/wiki_dir），固定字节级预期（含 `null` 失败形态与 Windows JSON 转义）、404/500 失败面、只读+句柄/端口清理；接口清单=983/AVSERVICE 已交付、982/EXEC `--server av` 部分交付（需改走 auto-web `serve_av`、API 路径可指定、grant/scratch 暴露、配置/lifecycle 同 `serve_vm`）、982/CONFIG 服务配置已交付/配置文档引擎本请求不用、980/WEB 与 983/GENERATED 本首例明确不用。③分期：C0-A 后端线 → C0-B 前端页 → C1 `POST /api/workspace/open`（将触发文本+平台 I/O 组合，交 982）。**未做**：未写 jade-garden 适配/AV 程序、未运行新栈请求、未改四仓整体验收分母。| evidence: 事实取自 jade-garden `back/server/src/workspace.rs`/`state.rs`、`front/src/lib/api.ts`、`front/e2e/01-workspace.spec.ts`/`global-setup.ts`/`runtime.ts`（端口 18181、WORKSPACE_ROOT 正斜杠）、982 EXEC receipts（rounds 149–166）与 man `http_service.rs` 实测读取；接口状态以 auto-web `3f8de15`/man `1d26b50`/lang `1af135c31` 为据。| next: work（C0-A 实现随 982 EXEC 接口到位分段接；本 Plan 不动 982 代码）

2026-11-18 stage: merge | PLAN-983 r9 | outcome: pass（**phase fold：AV 文本入口切片合入**；整包不归档，status 保持 executing）| delivery_commit: auto-web master `3f8de15`（=reviewed commit；ff-only，tip 相等已核；主检出 README WIP 属他 session 保留）| canonical: auto-lang master `1af135c31`（reviewed 规范 `b952cc9a4` rebase 至 `bf006d2e7` 后 ff；两提交=`043542b2e` 文本入口节 + `1af135c31` 状态/边界行修复；rebase 内容无漂移已核 tip 含该节）| checkpoints: prepared=review pass（下一行）；landed=两仓 ff-only tip 相等；canonical_verified=`m2-web.md` 文本入口节 + status/boundary 已在 master；**archived/cleaned=不适用**（CONSUMERS/T-05..07 在途；worktree/分支/lang-983 组保留）| **口径澄清**：用户称 T-06；按计划 T-06 为整包终态合入（前置含 CONSUMERS），本次只执行本切片的 phase fold 落主，**T-06/T-07 复选框保持未勾** | 消费登记：文本入口生产面=`http_server::serve_av_text`+`ServiceEngine::AvText`+`av_text_service::AvTextServiceEngine`（编译一次；`fn main(query str, body str) int`；收据 compile/av/legacy 计数+ABC 指纹）；source/接口/边界以 m2-web.md 文本入口节与 auto-web `3f8de15` 为准；982 文本参数契约（m2-man-gen.md）+ 本仓宿主接线=组合消费面 | merge 会话测试：复用 review 会话同 SHA 门禁（`3f8de15`：http_runtime 7/7 9.31s、lib 187/188 63.66s、http_generation 3/3 118.56s；准备=0 热构建）；本 merge 会话无新增用例 | next: B-CONSUMERS（前置 983/AVSERVICE+GENERATED 已落、982/EXEC、980/WEB）

2026-11-18 stage: review | review_scope: phase（B-AVSERVICE 批复核，含 2026-11-18 文本入口切片 B-AVSERVICE-TEXT）| reviewer_identity/limitations: 实现会话内复核（同一上下文；结论自冻结提交的 diff/断言/门禁复跑重建，不采信执行者摘要；独立复审窗口不可用已声明）| plan_id: PLAN-983 | plan_revision: r9 | outcome: pass（phase；整包仍 executing）| worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: auto-web `3f8de15` | base_commit: `fd115ce` | dependency_revisions: auto-lang 组树 `850a49bf8`（自 `19281566f` FF；零代码改动；982 文本入口/`execute_entry_text` 已含）；规范 reviewed `b952cc9a4` → 落主 `1af135c31` | spec_inputs: docs/specs/ace-migration/m2-web.md（本切片节；ace-migration/m2-man-gen.md 982 文本入口契约）| acceptance_results: AC-B-AVSERVICE pass（真实 TCP 7/7；文件通道臂维持 DB 写读/并发 owner 串行/断连/404 同形/端口释放；文本臂：同一编译产物收查询串+体、等字节长度 中文/日文 按内容分流、`av_dispatches=6 legacy_dispatches=0 compile_count=1`）；AC-02 pass（既有用例未改且 lib 187/188、generation 3/3）；AC-03 pass（文本臂无 `Rc<AutoVM>`、无 legacy 回退；未命中 404 在派发前拒）；AC-06 pass（AVSERVICE 批 T-08 面：中间件/上传 501 具名；认证/权限/stream/SSE/`~T` 按 R 条款记录）；AC-04 pass（本批）；AC-05/T-06..07 merge-owned 未闭 | findings: F-R3（已修）规范 status/boundary 行携带陈旧任务计数（13/19）→ work `b952cc9a4` 改为交付切片表述并去掉易腐计数，复验通过；无 in-contract 缺陷。观察（非阻塞/debt 候选）：①两 AV 臂 501/404 预处理为重复代码（未来边界改动需双写）②`failure_reply` detail 转义 `"`/换行但未转义 `\`（与文件通道臂同形；Infra 含 Windows 路径时错误体 JSON 可能不合法，当前消息域未见）③`legacy_dispatches=0` 为结构常量（无 legacy 句柄，非可增长计数器）——配套证据=无 AutoVM 构造、404 派发前拒、响应体由 AV 内容分支产出④路由 `fn_name` 未与入口交叉校验（单入口臂设计）| evidence: 复核会话实测（冻结 `3f8de15`；他 session 构建并发已观察并记录，均在其他 worktree）：`--test http_runtime` 7/7 9.31s（新例 0.172s）；`--lib` 188 例 187 绿/1 红/1 skip 63.66s（红=`api_gen::tests::test_handler_body_r2_matrix_compile_oracle`，该文件不在 diff，r7 已在基线 `0adebc0` stash 复证）；`--test http_generation` 3/3 118.56s；`git diff --check` clean；新文件 0 TODO/dbg/println/unwrap、0 warning；fmt clean | pending_closeout_gates（merge-owned）: 本切片 phase fold（已落：auto-web `3f8de15` / auto-lang `1af135c31`）；T-06 final 与 T-07 清理待 CONSUMERS | next: merge（phase fold，已完成）→ B-CONSUMERS

2026-11-18 stage: work | PLAN-983 r9 | outcome: pass（**B-AVSERVICE-TEXT 切片 work handoff，非整包 execution_done**）| worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | base_commit: auto-web fd115ce | code_commit: auto-web `3f8de15`；lang 组树 ff `850a49bf8`（零代码改动）+ 规范 `b016ff9d3` | task_ids: B-AVSERVICE-TEXT | evidence: 见 §8 任务证据行与 m2-web.md 文本入口节；命令/实测：`--test http_runtime` 7/7（7.48s）、新例 0.60s、`--lib` 188（187 绿/1 基线既有红/1 skip，21.39s）、`--test http_generation` 3/3（176.43s）；收据 `av_dispatches=6 legacy_dispatches=0 compile_count=1 fingerprint=9db3cce95d63f38e` | 用户指令执行口径：983 主导接生产栈（已交）；982 共享入口缺口本轮未触发（文本+平台 I/O 组合调用未出现，未派 982）；完整 JSON 等真实消费者字段（未做）| blockers: 无 | next_responsible_role: review（B-AVSERVICE-R 复核本切片）；982 交接按触发条件。

2026-11-17 维护（B fold 后）：按用户磁盘偏好再次清理组内构建中间产物——auto-web/target（17GB）+ auto-lang/target（3.1GB）删除（他 session cargo nextest auto-ui 与本组无交集已核）；组余纯检出；两树 git 不变（auto-web @fd115ce=master、lang 树待 CONSUMERS 开工 resync master 22ff37dbb）。**CONSUMERS 开工注意：双树冷重建。**

2026-11-17 stage: merge | PLAN-983 r7 | outcome: **pass（phase fold：M2-B 双批 AVSERVICE+GENERATED 合入；整包不归档，CONSUMERS/T-05..07 在途）** | delivery_commit: auto-web master `fd115ce`（=reviewed commit；rebase 无漂移；ff-only 无 merge commit，tip 已核相等；主检出 README WIP 属他 session 保留）| canonical: m2-web.md 增"B 批已验接口"节+Status/边界更新；replacement-scope-and-evidence.md 增 B 批增量挂接——随本 docs 提交落 auto-lang master | checkpoints: prepared=双批 phase review pass fd115ce；landed=auto-web master tip==fd115ce（wt-guard clean）；canonical_verified=本提交；**archived/cleaned=不适用**（CONSUMERS 在途；worktree/分支/组保留；plan-artifacts scope 保留）| 供他计划消费登记：消费面=AVSERVICE（`http_server::serve_av`+`AvServiceEngine`+`ServiceEngine` 枚举+av-runtime-service.at fixture 形态）与 GENERATED（`AUTO_WEB_A2R_ENGINE=ace` 参数子集+Query/Json 提取器 glue）；source/接口/边界以 m2-web.md B 节+本仓 SHA 为准 | merge 会话测试：复用 worktree 同 SHA 门禁证据（fd115ce：http_runtime 6/6、http_generation 3/3、lang 两组 PASS）；主检出身份相同未重跑；本 merge 会话无新增用例 | next: CONSUMERS 批（前置：983/AVSERVICE+983/GENERATED 已落、982/EXEC、980/WEB 归档面沿 §5 阶段表核）

2026-11-17 stage: review | review_scope: phase（M2-B 双批：B-AVSERVICE-R/B-GENERATED-R；整包仍 executing，CONSUMERS/T-05..07 未动）| reviewer_identity: 实现会话内复核（局限声明；结论自工件重建：双批 diff、门禁复跑、断言核读）| plan_id: PLAN-983 | plan_revision: r7（r8 为他 session 规划交接，与本实施收据并存）| outcome: **pass（phase，双批）** | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: fd115ce（=6808511 B-GENERATED + 76c8001 B-AVSERVICE + 复核修复；基 bdc48e8=A 落地点）| base_commit: bdc48e8 | dependency_revisions: auto-lang 组树零改动（lang 组门禁作非回归证据跑于组树）；前置 983/PORT（A 切片 master）+ L975/L978 ARCHIVED | acceptance_results: AC-B-AVSERVICE pass（V 6/6 + 复核复跑 av_service 0.915s；oracle 断言逐条核读：DB 写读/并发经 owner 串行不串/断连不触态/404 同形/独立 DB 直读/shutdown 端口释放；**具名边界**：中间件链/上传路由=501（语言侧凭据通道缺位）、限流=legacy 专有、上传失败清理面随 501 无 AV 面；认证/权限/stream/SSE/~T 未执行=R 条款记录非 NA）；AC-B-GENERATED pass（V 3/3 + 复跑；参数编组 int/str/bool 独立 expected、坏源 scale(float) 具名拒+保旧、真实 cargo 子进程终态断言；store 保旧=cold_generated legacy 例绿；str concat owned-Str 边界具名）| findings: F-R2（已修）av_service 未用绑定 warning→fd115ce→0 warning+用例绿；观察：①两新生成例计时波动 6.21–11.8s/6.45–9.66s（冷 cargo 启动+并发盘压；稳态≈6s；>10s 离群已记录，B/CONSUMERS 考虑预构建阶段）②ace-av/ace-av-core 转 auto-web 正式依赖=构建图扩大（AVSERVICE 引擎为生产面）③AV 臂 ASCII scratch/数据根约束沿 A 腿 | evidence: 复核会话实测——av_service 0.915s、ace_lane 双例复跑（22.4s 含冷重建波动；首测 6.21/6.45s 见 V 行）；批内文件 0 warning、0 调试打印；diff 复核=seam 仅加引擎枚举臂，legacy 路径未动 | 测试计时表（复核会话）：3 例 22.4s；准备=0（复用热构建）| pending_closeout_gates（merge-owned）：①双批 fold 合入 auto-web master ②m2-web.md 增 B 段 + SD-02 挂接 ③树保留待 CONSUMERS | next: merge（phase fold 双批）

### 2026-11-17 CONFIG-ACE-B规划交接（r8；非实施验收）

stage: new/revision | outcome: pass（规划合同预检） | 本轮用户仅授权补承接/设计/接口/验收及排程；未work/建树/运行测试/探针/构建/合入/归档/派agent。GAP-982-CONFIG-ACE由982原B-CONFIG两仓端到端承接，能力本身保持未实现；本包按§5–8现任务生产或接收，不新增Plan/任务，不改已验A收据。下一步须未来实施指令，982先B-CONFIG-I全链前探/接口与原输入冻结，再V→独立R→正式phase交接；其他owner按所用正式来源接。

取材和静态核验：7计划123原任务ID/顺序/勾选与状态/进度保留，语义revision各增1；仅982补已交967/979直接基础依赖。9个配置判别Case/4个能力AC、typed接口/值语义/预算/生命周期/原用例映射与Spec候选已列；24节点30边阶段图无环，新增本地链接可达，18份源码/Spec/归档/计数器身份不变，目标文档git diff --check通过。旧无owner快照/route C/执行回执作历史保留，不把规划PASS当真实能力或独立产品review。M2A-08 r29/95c7fbad8冻结不变。

2026-11-17 维护：用户指令清理构建中间产物（等 985/A、B 组 parked）。`.wt/lang-983/auto-web/target`（20GB，末次写入 09:06=本切片最后测试；他 session 09:48 `cargo t ace-*` 与本树无交集已核）删除；plan-artifacts/plan-983 内 2 个调试日志删除（scope 目录保留在途）。组余 526MB 纯检出。两树 git 状态不变（auto-web @bdc48e8=master、auto-lang @d43e7970f）。**B 开工注意：冷重建 target（约一次全量 debug 编译）。**

2026-11-17 stage: goal | plan_id: PLAN-983 | plan_revision: r7 | round: 6 | from: merge | to: done（**目标范围=983/A（M2-A 切片）**） | status: executing（B 组在途，7/19） | outcome: pass | worktree: D:/autostack/.wt/lang-983/auto-web（保留）+ auto-lang（resync d43e7970f） | commit: auto-web master `bdc48e8` / auto-lang master `d43e7970f` | next: done（/A 目标）；B 组另启（M2A-08 收据=上行 merge 收据）

2026-11-17 stage: merge | PLAN-983 r7 | outcome: **pass（phase fold：M2-A 切片合入；整包不归档，status 保持 executing 供 B 组）** | delivery_commit: auto-web master `bdc48e8`（=reviewed commit；rebase 无漂移=master 基 0adebc0 未动；ff-only 落地，无 merge commit，已核 tip 相等） | canonical: auto-lang `docs/specs/ace-migration/m2-web.md`（新建，A 切片接口+边界）+ `replacement-scope-and-evidence.md`（PLAN-983 增量挂接，沿 980/976 同款 SD-02 格式）——随本 docs 提交落 auto-lang master | checkpoints: prepared=review pass bdc48e8+冻结 delta；landed=auto-web master tip==bdc48e8（README WIP 属他 session 保留）；canonical_verified=两文件落库本提交；**archived=不适用**（phase fold；B 组 12 任务在途，A交接规约"同树保留/resync 至 B 完成"）；**cleaned=不适用**（worktree/分支/lang-983 组保留；plan-artifacts scope 保留在途；.retention 登记表带 BOM 损坏=PLAN-1036 已登记既有债，fail-closed 只记录，本 fold 无删除/持有变更故无需写） | 985/A 消费登记：A 切片正式收据=本行+m2-web.md；可消费面=中立端口类型、AV 服务腿 fixture/runner 形态、A2R 道显式选择与子集界；source/SDK/接口/实际结果以 m2-web.md"依赖身份"节与上表 SHA 为准 | merge 会话测试：复用 worktree 同 SHA 门禁证据（bdc48e8：两腿 3/3 10.55s、lib 186/187 20.05s、单测 7/7；主检出身份相同未重跑，准备=0）；本 merge 会话无新增用例 | next: B 组开工（M2A-08 收据=本行；B 前置沿 §5 阶段表）

2026-11-17 stage: review | review_scope: phase（M2-A 切片：B-PORT-R/B-PROOF-R；整包仍 executing，B 组 12 任务未动）| reviewer_identity: 实现会话内复核（局限已声明；结论自工件重建：两仓 diff、门禁复跑、断言逐条核读，非采信执行者摘要）| plan_id: PLAN-983 | plan_revision: r7 | outcome: **pass（phase）** | worktree_path: D:/autostack/.wt/lang-983/auto-web | branch: plan-983-dev | reviewed_commit: bdc48e8（=35b9e47+格式修复；基 93712e5→0adebc0） | base_commit: 0adebc0 | dependency_revisions: auto-lang 组树 e85096628 零改动（纯依赖钉基，已核 clean）；前置 967/968/969/974/975/978 ARCHIVED | spec_inputs: docs/specs/ace-migration/{m2-host,m2-facade,m2-stdlib-network,m2-stdlib-data,m2-build}.md（取材已核）| acceptance_results: AC-01 pass（T-01 冻结）；AC-B-PORT pass（公共面中立性=inspection 无 AutoVM 泄漏；行为保持=http_runtime 3/3+shutdown/upload 负例绿+内部机制 diff 未动；contract 纯度=cargo tree 仅 serde/serde_json）；AC-A-PROOF pass（V-A-PROOF 7/7+复核重跑 3/3；oracle 断言逐条核读：独立 bytes/DB 独立读回/大整数/断连/cancel/端口释放/坏源保旧/未手修）；AC-PHASE（A 部分）pass（A 阶段 required 5 任务全闭）| findings: F-R1（已修）本批文件 rustfmt 漂移 26 处→work 同树修复提交 bdc48e8→7 单测+3 集成+lib 186/187 复验绿；具名观察（非阻塞，B 域）：①core trait `listen_blocking` 签名仍为 core legacy trait 所定（桥已文档化，B/AVSERVICE 替换）②A2R 道零参 int/str/bool 子集（B/GENERATED 扩面）③AV fixture ASCII 数据根约束（B/AVSERVICE/973 域）④ace_lane 用例在具名 target 内显式 cargo --offline 子进程（计划 r7 明文合同"新增ACE具名case接原target并核实际子进程"；≤10s；与 2026-11-14 纪律的 build-lane 张力已记录，B/GENERATED 考虑预构建阶段）| evidence: 复核会话实测——单测 7/7（0.154s）；两腿 3/3（重跑 10.55s，含 ace_lane 5.89s）；lib 186/187（20.05s，1 红=test_handler_body_r2_matrix_compile_oracle **基线 0adebc0 原样红**，stash 复证，具名既有非本批）；fmt：本批文件 clean（余 http_server.rs:119 基线既有漂移，不在触面）；新文件零 warning、零调试打印 | 测试计时表（复核会话）：单测 7 例 ≤0.032s；集成 3 例 5.892/0.620/0.874s（首测 9.14/0.63/0.86）；lib 并集 20.05s；准备=复用 work 会话已热 target（未重编译全量）| pending_closeout_gates（merge-owned，phase fold）: ①auto-web master 合入 A 切片（bdc48e8）②规范沉淀 docs/specs/ace-migration/m2-web.md A 切片段（SD-01/SD-DETAIL/SD-PHASE-A/SD-TARGET-SCOPE 关联）+ replacement-scope-and-evidence.md 959 行关联（SD-02）③worktree/分支保留至 B（同树 resync 规约）④985/A 消费登记 | next: merge（phase fold）

2026-11-17 stage: work | PLAN-983 r7 | outcome: pass（**M2-A 切片 work 完成 handoff，非整包 execution_done**）| worktree: D:/autostack/.wt/lang-983/auto-web @plan-983-dev | code_commit: 93712e5（B-PORT）+ 35b9e47（B-PROOF） | base: auto-web 0adebc0 / auto-lang e85096628（lang 树零改动，纯依赖钉基） | dependency_revisions: 前置 967/968/969/974/978/975 全 ARCHIVED | task_ids: T-01、B-PORT-I、B-PORT-V、B-PROOF-I、B-PROOF-V | evidence: 见 §8 各任务证据行；V-B-PORT（lib 183/184 绿+1 具名既有红基线同红、http_runtime 3/3、contract 纯 serde/serde_json）、V-A-PROOF（7/7 绿 30.59s，新增 3 例 ≤10s 门：9.14/0.63/0.86s）、lane 单测 3/3 | session 测试表：lib 并集 23.78s（准备=冷编译已热，执行 23.78s，复用=既有证据未重跑）、V-A-PROOF 30.59s、http_runtime 门 5.94s（B-PORT 时点）；新增用例 4 例计时见 B-PROOF-V 行 | blockers: 无 | 本切片 open 项：①既有红 test_handler_body_r2_matrix_compile_oracle（基线 0adebc0 同红，非本切片引入，owner=待查既有）②A2R 道子集界（零参 int/str/bool、无 store/db/伴生）= B/GENERATED 扩面，具名不冒 ③AV 腿 ASCII 数据根约束（测试保证；非 ASCII 路径 staged 未支持，B/AVSERVICE 或 973 域）| next_responsible_role: review（A 切片独立复核 B-PORT-R/B-PROOF-R）

2026-11-17 stage: work | PLAN-983 r7 | goal 授权（/auto-plan-goal 完成计划983/A）开工 M2-A 切片：T-01 冻结完成（见 §8 T-01 证据行）；工作树组 `D:/autostack/.wt/lang-983/{auto-lang,auto-web}`，branch `plan-983-dev`（两仓），材料 scope `D:/autostack/plan-artifacts/plan-983/`。本记录非 review。

2026-11-16 stage: new/revision | PLAN-983 r7 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-12 stage:new / PLAN-983 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原19任务/全部AC与V/依赖/阶段/预算和drafting 0/19；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部19个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/19；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-07 r3：细化4批，0/16。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

## 10. 待澄清事项

**2026-11-18 982/EXEC 优先项（首例 C0 依赖）**：按用户裁定「尽快解锁 983 ⇒ 982 优先交首例实际需要的 EXEC 接口」——`auto service --server av` 需改走 auto-web `serve_av`（transport/config/shutdown/ready/receipt 同 `serve_vm`），API 源路径可显式指定（默认 `src/back/api.at` 不变，jade-garden 用 `back/auto/api.at`），暴露 AV 引擎 grant/scratch（`platform.file`/`platform.mem`）与程序入口；坏配置零写、失败具名、不回退 vm。C/EXPORT 按自身顺序并行，不作本首例前置。详见 §5「首例冻结」。

**2026-11-18 用户指令（983 先接 auto_web 生产栈）**：本轮已交文本入口切片 B-AVSERVICE-TEXT。两项未触发/未做，按用户口径保留为下一步条件：①「文本参数 + 平台 I/O 同一次调用」——983 首个消费者本轮只用文本；当真实消费者需要组合调用时，把**具体需求**交 982 验证并补齐（982 收据仅覆盖文本参数，组合调用需单独验证）。②完整 JSON（嵌套/浮点/请求头）——等 983 选定真实消费者、明确所需字段后再做，先交实际用到的少量字段。983/CONSUMERS 四仓整体验收仍按计划核对 980/WEB、982/EXEC 与实际用到的配置接口。

**2026-11-17 用户指令（B 期开工授权）**：用户明确指示先行实施 B-AVSERVICE 与 B-GENERATED 两批并合入主分支供其他计划使用——本指令解除此前"等 985/A 完成再继续 B 组"的等待（CONSUMERS 与 T-05..07 仍按原门保留）。B 前置实测：983/PORT 已随 A 切片落 auto-web master `bdc48e8`；L975/L978 ARCHIVED ✓。

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
