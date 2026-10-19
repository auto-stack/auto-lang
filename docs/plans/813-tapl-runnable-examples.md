---
plan_id: PLAN-813
status: reviewed
feature_name: tapl-runnable-examples
author: [Codex]
plan_revision: 2
created_at: 2026-10-19
updated_at: 2026-10-19

supersedes_spec_components: [docs/specs/website/design/learning-reading.md, docs/specs/website/project.md, docs/specs/auto-playground/project.md, docs/specs/stdlib/project.md, docs/specs/a2r-std/project.md]
new_spec_components: [docs/specs/website/design/book-example-execution.md]
touched_goals: [GOAL-014, GOAL-016]

affects: [website, auto-playground, stdlib, auto-lang/vm, a2r-std]
current_step: 9
total_steps: 10
---

# [PLAN-813] TAPL 全书示例与网站 Run 修复

## 0. 变更摘要

对 TAPL 中英全部章节、所有代码块及实际 Listing 源建立完整执行清单；修复网页 Run 的后端地址、上下文、输入、多文件/测试入口、缺失 API 和错误书稿。优先复用现有标准库并明确 use；确有通用缺口才补真实实现。

起草证据：[全书分析](attachments/813/analysis.md)、[逐入口定位](attachments/813/static-inventory.md)、[完整围栏与源哈希](attachments/813/static-inventory.json)。这些是静态普查，运行状态全部 not_run，不能作为执行通过证据。

## 1. 目标

- G-01：ch00–22、附录 A–D 中英 54 页、1829 个源围栏无漏项；当前 401 个网站 Run 位置均有处置和验收记录。修改后重新从最终内容生成数量。
- G-02：普通示例得到教学所述的真实结果；片段能查看并运行完整上下文，输入/文件/模块具备真实运行条件。
- G-03：测试真执行断言，故意错误核对具体诊断；配置/命令不当程序运行；长驻服务有完整工程与可验证本地启动入口。
- G-04：修改写回 book 作者源和 auto-lang 实现，EN/ZH、书页/Notes 一致；增加定向回归防止坏例重新进入 Run。

非目标：重写编译器/互操作体系；实现所有第三方生态库；翻修另外七本书；未经授权部署。若后端专属例子无法保留教学目标，须提出具体变更裁定，不能静默删去或改为展示来降低门槛。

## 2. 架构方案

保留 VitePress + Playground + AutoVM。给示例增加显式执行描述，记录展示源、完整执行源/文件/输入、预期和模式，供 CodeView、AutoFence 和 Notes 共同消费。

模式为 standalone、with-setup、project、test、expected-error、local-only、display-only。普通教学示例优先前五类；后两类必须逐项有语义理由，适用于真实环境/长驻服务和配置/命令/纯语法。禁止整体关闭 Run、加 ignore、用固定输出清零错误。

跨块依赖显式最小化，不能自动拼整章；完整源可见且实际运行保留编辑器改动。标准库 use 不等于不可执行；多文件复用现有 files/entry 物化路径。测试优先接现有 `auto_lang::test_code`。

网页输入为每次 Run 提交的一段可编辑文本，暂不扩成逐键交互 WebSocket 终端。输入必须请求/VM 隔离，区分空行和 EOF，不读服务端系统 stdin，不挂死。沿用现有 deadline（默认 10s、clamp 1..60），测试路径也须受截止保护。EOF 表达及最小注入点由 T-01 探针确定并回写后才做依赖实现。

这是局部网站/运行宿主补齐，优先复用既有 host capability、stdlib、项目和测试装配。若探针证明需要改变核心并发/资源协议，按 L2 先补设计并拆计划，不在本计划暗中扩大架构。

## 3. 技术栈

Markdown/.at/真实多目标金样；VitePress/Vue/TypeScript/Playwright；Rust axum Playground、CompileSession/AutoVM；既有标准库/native。起草普查脚本只依赖 Python 标准库。

## 4. 需求分析与背景调查

### 授权及工作区

用户 2026-10-19 明确要求“实施该计划”，已确认 PLAN-813 r1 的既定范围并授权执行。按本计划实施，不部署、不代发消息，不覆盖两仓既有 WIP。

涉及 `D:/autostack/auto-lang` 与作者源 `D:/autostack/book`。无预算/续跑次数约束；不部署、不代发消息，不覆盖两仓既有 WIP。

### 取材与基线

- 已读 `docs/specs/overview.md`、`README.md`、`goals.md`、website/project 与 design/learning-reading、auto-playground/project、stdlib/project、auto-lang/vm/overview、frontend/design/module-resolution。
- auto-lang 起草基线 `99d1bda7ea5e8371555a1ff5dc915000dbd5ebf1`；book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`。书稿/实际源/包/金样 hash 在 static-inventory.json::input_hashes，消费者/规范 hash 在 source-baseline.json。work 入口需重确认。
- book 当前 9 项 WIP 在另一本 Rust 教程 `rust/listings/.../*.expected.rs`，不覆盖。auto-lang 亦有其他 Plan 文档 WIP，不宣称主工作区 clean。
- 作者源 `book/tapl` → `prepare-content.js` → website/books 与 zh/books；后者是生成物，不能只改它们。
- 直接证据详见 analysis.md F-01..11：305 个 Listing 默认 localhost；TOML 错给 Run；片段不组装；两入口导入策略不同；缺 stdin；死循环；现有 book 测试默认覆盖 Rust 书而非 TAPL 全围栏；写死的文件/解析行为；测试入口不匹配；库名/返回类型不一致；EN/ZH 章节缺项。正文和实际源的 17 处字节差异含注释差异，须语义核对。
- 复用候选：io/read_line、random/rand_int、json、encoding/base64/hex、time/now_ms/sleep_ms、datetime/DateTime、hash/sha256、url/Url、File/fs/path。存在声明不能代替公共入口运行证明。

## 5. 详细设计

### 5.1 先做有界调查

T-01 在批准后的专用树完成，产出决定记录并回写本计划，然后 T-02..06 才能依赖：

1. 将静态清单与预处理后真实页面逐项对应；执行记录含稳定 ID、源 hash、模式、setup/files/entry/stdin、预期、处置。中英仅在源/配置/输入相同时复用 VM 证据，页面映射仍逐条检查。
2. API 逐项核对声明→解析/绑定→真实调用→返回/错误。TOML、解析失败、日期时间、错误传播、FFI 未知项都需明确决定；优先复用已有功能。
3. 验证最小 stdin 注入点、空行/EOF、中文/CRLF/末行无换行、连续与同时请求。禁止全局 stdin 替换。确认现有请求体预算；任何新增限额明确单位/错误，不隐式截断。
4. 走通三个端到端探针：ch03 前文函数、ch05 读输入退出、ch10 多文件。探明 test_code 的 deadline 缺口并分配 T-02 修复。
5. 落定执行元数据作者位置/字段/稳定 ID 迁移。若需要改变目标/验收/兼容范围或新增核心架构，先让用户裁定；常规实现选择继承已确认范围。

#### T-01 决定记录（2026-10-19）

- 预处理实际页面为 EN/ZH 各 27 页；生成后可见 303 个 CodeView 与 96 个 AutoFence，共 399 个 Run 入口。与静态 401 的差额是两种语言页各一处 `ch01 Listing 1-2`：作者源是 `auto.toml`，但 `prepare-content.js` 只接受 Auto 源，当前生成 `Listing source not found` 注释。修复为明确 `display-only`，不得计作可执行 Run；其余入口必须由执行描述显式授权。
- 共用执行合同放在作者仓 `tapl/examples.json`。条目使用不可变 `id`（locale + 页/示例锚点）、`pair_id`（译本对应）、作者页定位、显示块 ID、`mode`（`run`/`test`/`display-only`）、完整项目文件与入口、请求输入、预期类型/值或诊断、超时。Listing 通过 `example-id` 属性绑定；Auto 围栏通过 `example-id` 信息属性绑定。页面转换、AutoFence、CodeView、Notes 和全源门禁均读取同一合同；缺少/重复/孤立绑定均令门禁失败，不再靠 `use/import` 正则猜可运行性。
- `CodeView` 目前无条件显示 Auto Run 且默认 `localhost:3030`；RunRequest 有 source/files/project_dir/entry/timeout_secs，无输入字段；`/api/test` 不存在。ch03 的完整 Listing 1-1 经 CLI 成功，页面独立调用因 `greet` 未定义而失败；ch05 示例调用未定义的 `input()`，完整 Listing 5-4 还在运行前因 `none` 未定义失败；ch10 Listing 10-5 只有 main/pac，缺 `utils`。已知多文件 Playground demo 经公共 CLI 成功，证明现有模块解析可复用，但 TAPL 必须提供完整文件树。
- `io.read_line()` 公共 CLI 探针接受文本行并正确移除 CRLF；空行和立即 EOF 都返回 `""`。禁止替换全局 stdin。为 Playground 扩展请求级输入队列并随每次 VM 执行注入；浏览器总是显式提供输入（无输入即有限 EOF），CLI 旧 API 继续读取 OS stdin。新增 `io.read_line_opt() -> Option<str>` 以区分空行 `Some("")` 与 EOF `None`，处理 LF/CRLF/末行无换行。
- `test_code` 会真实收集并执行 `#[test]`，但构造 VM 时未设置 deadline；Playground 没有测试 endpoint。T-02 增加带截止时间的测试入口/API，并确保超时结果可见、下一次请求不受影响。Run 继续默认 10 秒、后端限幅 1–60 秒；请求输入/文件超出既有预算时返回具体错误，不截断。
- Playground 项目执行已经为 files 建立独立临时目录并清理，使用文件入口路径参与模块解析；T-02 复用此路径并修正临时目录唯一性/安全检查（按探针需要），不在书稿目录改 cwd 或环境变量。线上 Run 使用同源 `/api/run`（由站点反向代理），不得默认请求读者本机 localhost。
- 基线探针与页面映射细节见本计划附件 `t01-findings.md`。T-01 已完成；代码执行仍为 T-02 起逐条验证，静态分析不计运行通过。

### 5.2 修复原则

- 不存在函数先改正确现有命名/use；通用缺口可补最小 stdlib 实现、声明、装配与错误用例，默认不把教程所有裸名注册为全局内建。
- 片段补显式 setup/完整工程，读者能看完整源和文件；复制/下载提供完整版本，不能执行另一个隐藏样例绕过编辑内容。
- 修作者 `.md/.cn.md`、Listing `.at`、fixture/pac 和受影响的真实生成金样，禁止手写假转译或把错误输出盲目设为 expected。
- 错误示例保留教学目的，核对明确阶段/诊断。测试真执行；编译期例验证编译期行为，不以 main 打印常量代替。
- 文件/环境使用每次运行独立 fixture/目录并清理所有自建资源；不能全局修改 cwd/环境污染后续请求。优先复用 files 物化，不改源码目录。
- 长驻聊天服务提供完整工程、启动条件与本地命令；页内用有限消息交换演示，并明确二者关系。本地工程验收一次 connect/send/reply/disconnect 并关闭，不能以杀死挂起进程冒充成功。
- 书页与 Notes 共用执行描述，不能一处修复另一处退回坏片段。无法适配的消费者明确报错。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/learning-reading.md | use/import 正则锁定与无条件 Listing Run → 显式模式，标准库/完整项目可运行，正文可读，输入/错误明确 | 统一两入口 | AC-02, AC-03, AC-06 |
| SD-02 | add | docs/specs/website/design/book-example-execution.md | 无全书执行合同 → 作者/完整源/元数据/入口/oracle 对应、Notes 共用和译本映射 | 防漂移假绿 | AC-01, AC-04, AC-06 |
| SD-03 | modify | docs/specs/auto-playground/project.md | 现有 source/files/deadline → 请求输入/EOF/隔离/测试支持；字段经 T-01 落定 | 网页真运行 | AC-03, AC-04, AC-05 |
| SD-04 | modify | docs/specs/stdlib/project.md | 现有库边界 → 实际补齐/修复的 API、导入及验证边界；零新增时明确消费更正 | 不凭书稿虚构库 | AC-05, AC-06 |
| SD-05 | modify | docs/specs/website/project.md | 网站项目缺少 TAPL 执行合同入口 → 链接到规范执行合同 | 让网站规范导航覆盖本次新增合同 | AC-01, AC-02, AC-06 |
| SD-06 | modify | docs/specs/a2r-std/project.md | a2r-std 未明确浮点 helper 的 f64 API 和 parity harness 边界 → 记录四项函数与名称/元数比对、行为单测范围 | 说明 TAPL Rust transpiler 路径证据 | AC-05, AC-07 |

### Phase 2：关闭 Windows P765 退出钩门禁回归（T-10）

R9 门禁显示 P765 trampoline 已进入，但其 `RtlCaptureStackBackTrace` FFI 声明漏掉 Windows API 的可选第 4 个 `BackTraceHash` 参数，属于 x64 ABI 未定义行为。修正返回类型为 `u16`、补齐 `*mut u32` 并在两个调用点传 `NULL`；其四参数签名与可选指针见 [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtlcapturestackbacktrace)。不改变书稿范围或退出钩行为。V-09 用既有子进程测试确认退出码 9、审计行和栈帧。

## 6. 测试设计

T-01..06 只做定向检查/反例和必要 `cargo check -p auto-lang`，不触发整档。拟新增 `scripts/check-tapl-examples.py`（T-01/T-06 落地，尚不存在），消费真实元数据调用公共 CLI/API，比较输出/具体诊断；不能只扫描字符串或报计数。

拟新增 `website/tests/tapl-run.spec.ts`（T-03/T-06）。语料 runner 串行验收全部实际有效源，浏览器全页核对策略/源映射并对各模式真实运行；不能只验证精选示例。其他书保留现有阅读 smoke。

| Case ID | AC / invariant | Input / entrypoint / owner | Observable expected result | Coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02 入口真实 | work：EN/ZH 全页预处理、CodeView/AutoFence | 全入口映射齐，TOML 无 Run；故意漏配一条审计非零 | TAPL 全源、其他书 smoke |
| V-02 | AC-02 编辑生效 | work：greet/add 与循环片段改参数/文本再 Run | 完整源可见，输出随编辑变化，依赖真实补齐 | 声明/调用/控制流/多文件 |
| V-03 | AC-03 输入隔离 | work：猜数有效/非法/不足输入、空行/CRLF/中文、连续及同时请求 | 真正读取对应行，确定 EOF、不读 OS stdin、不串线/死循环 | source/files 两路 |
| V-04 | AC-03/05 真文件 | work：ch14 fixture、缺文件/非法数据 | 核对读取和写出实际字节，失败走错误分支；次轮无残留 | 目录/文件/环境清理 |
| V-05 | AC-04 测试真跑 | work：ch18 测试与故意错误断言，ch20 相关项 | executed>0；正确通过、故意错误失败；超时终止可继续下一例 | test_code 和运行入口 |
| V-06 | AC-05 API 真装配 | work：ch05/09/21 公共调用 | 随机端点、解析/编码正反例、时间单位/类型吻合，非固定假值 | 保留 API；不泛称所有后端支持 |
| V-07 | AC-04/05 错误与长驻 | work：预期错误/死循环/有限 actor/聊天工程 | 诊断吻合，失控按时终止；消息交换完成且关闭，无遗留端口/进程 | 不依赖任意第三方服务在线 |
| V-08 | AC-06 回归可失败 | work/review：全有效源、Notes/书页、变异缺依赖/错误期望 | 普通条目语义通过，负控使总门禁失败；静态项逐条有理由 | hash 绑定源/配置/依赖/输入 |
| V-09 | AC-07/08 Windows FFI ABI | work/review：P765 子进程测试分别以 `test-book` 与 `ui-iced,test-cases` feature 运行 | 子进程退出码为 9，审计包含 `exitproc_trace` 与 `frames=[+]`；缺失可选参数的 ABI 回归由用例检出 | 当前 Windows x64 IAT 追踪路径，不覆盖其他平台 FFI |

纯页面/书稿/脚本不运行 cargo t/docs_gen。若改 Rust，最终 review pass 后，merge 在冻结提交统一执行裸 `cargo t`；VM/编译器触面加 tv、书测试触面加 tb、转译触面加 tt，按实际 diff 决定；本范围不触 ui_gen/aavm，不跑 tu/taa。无生成器/Schema/语法参考改动不跑 docs_gen。tf 仅 merge 判到期后主检出单实例按 regress。

用户未指定模型或另一个 chat；本会话实施。最终显式 `/auto-plan:review`，从冻结 diff 和 AC 重建证据并诚实说明上下文独立性；不自动创建子代理/chat。review 不修代码；需要修复回 work。pass 明记 merge-owned 门禁 pending。

## 7. 验收标准

| ID | owner_stage | 可观察结果及验证 |
|---|---|---|
| AC-01 | work | 全 54 页/所有围栏/实际入口无漏项，更新后重新计数，每 entry 有定位/hash/模式/预期；未实跑或跳过不算通过。V-01/08。 |
| AC-02 | work | 两组件共用正确后端/执行描述，不默认连线上读者 localhost；上下文/标准库/多文件可真实执行，编辑生效，配置不误显示 Run，原文和双语正确。V-01/02。 |
| AC-03 | work | 输入可编辑且真读取，EOF 有限退出/明确错误；连续并发不串线、CLI stdin 兼容，截止生效；文件/环境不污染下一例。V-03/04/07。 |
| AC-04 | work | 测试真执行断言且失败可见；错误诊断/编译期行为符合教学；actor/聊天有有限验证与完整工程边界。V-05/07。 |
| AC-05 | work | 未定义 API/类型/依赖/错误片段全部闭环，已有库优先复用，新增库有真实声明/装配/正反例；无常量冒充文件/解析/超时。V-04/06/07。 |
| AC-06 | work | 全有效源定向复验通过，每 Run 有输出/诊断/时限证据；书稿/实际源/Notes 一致，双语缺项补齐；负控能使门禁失败，其他书无新增回归。V-08。 |
| AC-07 | review | 冻结 diff、AC-01..06、SD-01..06 独立审计，扫描遗漏/未批准延后/workaround；零新增未处理 warnings/debug/格式问题；门禁 pending 明示。 |
| AC-08 | merge | 分级门禁后双仓合入、规范/账本/索引回写、归档；fresh guard clean 后清理所有实施树/分支，tf 到期裁决及实际收据完整。 |

## 8. 执行步骤

预期唯一实施组 `D:/autostack/.wt/lang-813/{auto-lang,book,auto-down}`；lang 分支 `plan-813-dev`，book 分支 `plan-813-book`，auto-down 只读 detached。先确认书仓 AGENTS、基线和两仓 WIP，使用 `AUTO_BOOK_ROOT`/`AUTO_BOOK_LISTINGS` 指向兄弟仓，**禁止 junction/symlink**。全部代码/构建/运行在实施树，计划簿记留主检出。创建树前按仓库要求提交本计划和取号簿记，不夹带他人修改。

| ID | owner_stage | 依赖 | 操作/真实路径或符号 | 结果及验证 |
|---|---|---|---|---|
| T-01 | work | 用户确认 r1 | 建实施组，核对 prepare-content.js/auto-fence-md.ts 与书稿；公共 CLI/API/test_code 跑 §5.1 探针，设计执行元数据和审计入口 | ✅ 完成：决策与基线探针已回写 §5.1 并记录于 `t01-findings.md`；T-02..06 依赖此合同 |
| T-02 | work | T-01 | Playground `/api/test` handler、共享 deadline、输入隔离 | ✅ 完成（闭合 R2）：真实 handler 单测覆盖正确/错误断言、超时响应、下一请求恢复、并发请求输入隔离；定向测试 3/3。浏览器测试仍单独验证前端请求形状。 |
| T-03 | work | T-01/02 | website CodeView.vue/AutoFence.vue、prepare-content.js 与执行描述消费者 | ✅ 完成：共享 catalog、同源 API、完整可编辑源/输入/多文件/测试/展示和本地运行模式；最终生产构建 96.38s 通过，`tests/tapl-run.spec.ts` 6/6。 |
| T-04 | work | T-01/02/03 | book/tapl ch00..13 中英、对应 listings；上下文、猜数、模块、错误教学；必要的标准库/绑定局部修正 | ✅ 完成：修复 ch00–13 作者源、Listing、预期与页面映射；ch08 按用户确认保留可运行改写并说明差异；ch09 文件内容去除重复末尾换行。 |
| T-05 | work | T-01/02/03 | book/tapl ch14..22 中英、附录 A..D、listings/pac/fixture/受影响 expected；文件/测试/标准库/异步/聊天及通用缺口 | ✅ 完成（闭合 R1/R3）：Base64/Hex 复用现有依赖并可运行；新增 loopback TCP server/client 本地工程及明确范围说明，完成连接、消息往返、断开和优雅关停验收。 |
| T-06 | work | T-02/04/05 | `scripts/check-tapl-examples.py`、`website/tests/tapl-run.spec.ts`；最终清单和可重放证据、规范增量 | ✅ 完成（闭合 R5）：新规范准确列出五种模式和 stdout_contains/expected-error；checker 对 54 页执行 catalog↔page 双向扫描；有效样例通过，错误预期、旧锚点、孤立锚点、错误诊断负控均失败。复核后全量 396/396 通过。 |
| T-07 | work | T-06 | 两仓提交本计划改动；固定 code/input/dependency hash、结果/命令、债务和规范增量；主计划 execution_done | ✅ 完成：auto-lang 现冻结提交 `5c6daa1439d95dbf2811f4f242895490b6d40d2a`，书稿继续冻结在 `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；两个实施工作树 clean，文档复审修订已提交。 |
| T-08 | review | T-07/T-10 | /auto-plan:review 重审冻结修复 diff、V-09、既有 AC/SD 证据及 debt 归因 | ✅ R10 独立复审 pass，绑定 auto-lang `78e1c31e8be30b2c758d6034f211f48c6731dc87`、book `50153be9f4e0ac035ca3d0c2bd0da2d876d84875`；V-09 pass，无 finding；三项 merge-owned gates pending。 |
| T-09 | merge | T-08 | /auto-plan:merge：在 review pass 后于最终冻结组合串行跑 `cargo t`、`cargo tv`、`cargo tb`，全绿后再双仓合入与收尾 | pending：R9 的 P765 失败已由 T-10 修复；须在本轮复审通过后的冻结提交重跑三项门禁。此前 PLAN-819 两个 Rust listing 修复目标已通过；未全部绿前不合入/归档/清理。 |
| T-10 | work | R9 gate failure | 修正 `crates/auto-lang/src/vm/ffi/stdlib.rs` 中 `RtlCaptureStackBackTrace` ABI；按 V-09 在 `test-book` 和日常 feature 下重放 P765，并运行 `cargo check -p auto-lang` | ✅ 完成：缺失可选 hash 参数被补齐并显式传 NULL；两个 feature 配置下各 1/1 通过，`cargo check` 通过。提交 `78e1c31e8be30b2c758d6034f211f48c6731dc87`；日志见 `attachments/813/p765-ffi-fix-r13.log.txt`、`p765-daily-feature-r14.log.txt` 与 `cargo-check-p765-r14.log.txt`。 |

T-01..T-08、T-10 已完成，R10 独立最终复审通过，当前状态为 reviewed（current_step=9/10）。T-10 修复 P765 的 Windows ABI 未定义行为，V-09 定向回归与类型检查通过。`cargo t`、`cargo tv`、`cargo tb` 仍是 merge-owned pending，须在本冻结提交串行执行；在全部通过前不合入、归档或清理。

## 9. 复审记录

2026-10-19：用户确认 PLAN-813 r1 并要求实施；状态转为 executing。T-01 已在专用工作树完成：将静态清单对上生成页面（实际 399 Run，另两处 TOML 应静态展示）、执行 ch03/ch05/ch10 公共 CLI 探针、确认 stdin 空行/EOF 行为、确认 `/api/test` 缺失及 test VM 无 deadline，并回写元数据/输入/测试/API 决定。

2026-10-19：T-02..07 完成，work handoff 为 `pass`，下一责任角色为 review。实现身份与证据如下：

| 仓库 | worktree / branch | base commit | 冻结 code commit |
|---|---|---|---|
| auto-lang | `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev` | `30c1b9a7edc3112cac99089d14169d9f9bdeaeb6` | `cacb9fcc335b4addc5a0db67e7c64282b30d445e` |
| book | `D:/autostack/.wt/lang-813/book` / `plan-813-book` | `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a` | `de49ae222da45fd9b33eeb443fdd776658a1750c` |

- auto-down 依赖 worktree 为 `D:/autostack/.wt/lang-813/auto-down`，detached HEAD `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。该共享依赖树有 12 个 tracked 文件的既有 WIP（约 +2915/-757 行）；本计划未修改、提交或清理它。实施阶段 cargo 默认依赖树未启用 `autodown-core`（`cargo tree -p auto-lang -e features` 未输出该依赖）。
- 输入/作者源哈希：`Cargo.lock` `1E1E432EEFC4C75981A5B1E788830A6791E911949FD3153B3E22570EFC1BFF9E`；`website/package-lock.json` `DC7121A2C03D91D0A20CD8C3CB8DC31B68CA235E8FC2364F0837082434F6FF82`；`book/tapl/examples.json` `89B64109AA5BB692F6BFC5279A90760E36D6C86043571B25AE8A27CE2C26FF89`。
- 核心证据：`cargo check -p auto-lang` 通过；`cargo test -p auto-lang --test plan813_hosted_input` 3/3；`cargo build -p auto-playground --locked` 通过；TAPL 401 项最终全量 API 审计 401/401、0 失败，含错误期望与失效锚点负控；生产构建通过；`CI=1 AUTO_WEBSITE_TEST_PORT=4185 npm run test:e2e -- tests/tapl-run.spec.ts` 5/5。目标 Rust 文件定向 rustfmt、两仓 `git diff --check`（按 CRLF 处理）通过。报告见 `attachments/813/tapl-full-execution.json.txt`。
- 边界如实记录：环境无 `gcc`/`clang`/`cl`，C 等价代码人工核对、未编译；全仓 `cargo fmt ... --check` 被既有 `crates/auto-lang/examples/ui_container.rs:195` 语法错误及其他既有格式差异阻断，所有本次修改 Rust 文件的定向 rustfmt 检查通过；`cargo check` 显示 406 条既有 warning，未将其误报为零 warning。没有在 work 阶段运行整档 Cargo。
- 规范增量候选仍为 SD-01..04（见 §5）；合并前执行的 merge-owned 门禁 `cargo t`、VM 触面 `cargo tv`、book 触面 `cargo tb` 均 pending。AC-08 的双仓落地、规范/ledger/index、归档、fresh guard 与清理属于 T-09，尚未宣称完成。工作树冻结后无 work-owned 待办；依赖树中的他人 WIP 需保留并在复审/合并收据继续注明。

stage: work | plan_id: PLAN-813 | plan_revision: 1 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-813/auto-lang` + `D:/autostack/.wt/lang-813/book` | branch: `plan-813-dev` + `plan-813-book` | base_commit: `30c1b9a7edc3112cac99089d14169d9f9bdeaeb6` + `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a` | code_commit: `cacb9fcc335b4addc5a0db67e7c64282b30d445e` + `de49ae222da45fd9b33eeb443fdd776658a1750c` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` (12 tracked WIP files preserved) | task_ids: T-01..T-07 | evidence: §9 and `attachments/813/tapl-full-execution.json.txt` | blockers: none for work-owned criteria; C compiler unavailable and full-repository rustfmt blocked by pre-existing issues, both disclosed above | next_responsible_role: review

### 2026-10-19 最终复审 R1（needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: Codex 当前会话，与实施共用同一对话上下文；未创建独立任务或代理。结论从冻结提交、作者源、规范差异和测试断言重新构建，不把 work handoff 或测试总数当作验收结论。
- plan_id/revision: PLAN-813 / 1；code_commit: `cacb9fcc335b4addc5a0db67e7c64282b30d445e`；book_commit: `de49ae222da45fd9b33eeb443fdd776658a1750c`；base_commit: auto-lang `30c1b9a7edc3112cac99089d14169d9f9bdeaeb6`，book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；worktrees/branches: `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev`，`D:/autostack/.wt/lang-813/book` / `plan-813-book`。
- dependency_revisions: `auto-down` `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；其 12 个既有修改文件未纳入复审改动。
- spec_inputs（SHA-256，来自 code_commit 工作树）：`website/design/learning-reading.md` `6D72CF5DAE547E6DE28F03DB7E522F1DCD4312B7BECE8FCBAE9A45C87C9BF5EA`；`website/design/book-example-execution.md` `E190527D16F87830F0584844D60FD33B3BBD211C007CEE61D30B48BB184507F1`；`website/project.md` `275EB05793589D1F20875C5BBF4CF8134DD24F068FD2029BCCE7866B73138039`；`auto-playground/project.md` `87039E4443AB4A0F5C1178979EB0E5D7A6ACA6F899FE98CFF7362F1B58181ADD`；`stdlib/project.md` `02CD89AA2C415100ACA8A49F708EBCC7C6B0159973B47A046A0CF7530CB94CAE`。
- acceptance_results: AC-01 pass（独立计数 54 页 / 1829 个围栏 / 401 个 catalog 项，模式为 377 run、14 test、10 display-only）；AC-02 partial（Playwright 5/5 与生产构建证据，但前端 API 用 mock）；AC-03 partial（VM 级空行/EOF/并发输入覆盖，没有 HTTP handler 并发输入隔离验收）；AC-04 fail（缺失测试断言失败/超时恢复证据，且聊天章没有完整本地服务器工程）；AC-05 fail（Base64/Hex 现有运行时 API 被误判不存在；聊天工程不完整）；AC-06 partial（报告记录 401/401 和负控通过，但 Base64/Hex 的 display-only 分类掩盖了可运行条目）；AC-07 fail（Spec 影响元数据漏掉已修改的 `docs/specs/website/project.md`）；AC-08 pending。
- evidence: 计数由 `book/tapl` Markdown 与 `examples.json` 独立读取重算；编码 API 可在 `crates/auto-lang/src/vm/native_catalog.rs` 查到 Base64 IDs 2710/2711、Hex IDs 2720/2721，并由 `test/cookbook/encoding/004_base64/base64.at` 展示 `dep` / `use.rs` 形式；`crates/auto-lang/tests/plan813_hosted_input.rs` 仅含输入/HashSet 测试，`website/tests/tapl-run.spec.ts` 的 `/api/test` 用 mock 返回成功；`book/tapl/ch22-chat-server.md` 声称完整服务器，`listings/ch22/listing-22-06/pac.at` 只有空 `app`，对应 Auto 源是有限内存状态演示；code diff 含 `docs/specs/website/project.md`，但 §5 的 SD 表原仅列 SD-01..04。原全量报告 `attachments/813/tapl-full-execution.json.txt`（SHA-256 `87E76198D73D3C0A557B67546EB3238FF9DCB1612C239174062C7B7B6C9911D3`）绑定旧源，需在修复后重跑。

| finding_id | severity / type | affected AC / tasks | finding and required correction |
|---|---|---|---|
| PLAN813-R1 | P1 / 原验收缺口 | AC-05/06；T-05/T-06 | EN/ZH 的 Base64 与 Hex 示例在 `examples.json` 中标为 display-only，作者源只输出“不支持”占位；运行时已有可调用依赖。复用现有依赖 API 补齐真实编码/解码示例、预期和双语说明，再全源验证。 |
| PLAN813-R2 | P1 / 原验收缺口 | AC-03/04/06；T-02/T-06 | `/api/test` 只有 mock 成功 UI 用例；没有实际 handler/API 的正确测试、故意失败断言、超时终止后下一请求成功及并发输入隔离证据。补真实定向用例并检查响应计数、诊断、deadline 与恢复。 |
| PLAN813-R3 | P1 / 原验收缺口 | AC-04/05/06；T-05/T-06 | ch22 目标要求完整本地工程及 connect/send/reply/disconnect/clean shutdown。现有 22-5/22-6 是有限内存演示，22-6 的 `pac.at` 是空项目；书中运行时注记没有替代本地工程验收。补可启动工程和完整周期验收；若现有语言/运行时边界使其超出已批准设计，先按流程提交契约修订，不得把静态或内存演示记作完成。 |
| PLAN813-R4 | P2 / 收口缺陷 | AC-07；T-07 | 代码提交修改 `docs/specs/website/project.md` 并增加导航链接，但 frontmatter 与规范增量表遗漏该文件。已补 `supersedes_spec_components` 与 SD-05，后续须随修订后 diff 再核对。 |

merge_owned_gates: `cargo t`、`cargo tv`、`cargo tb` 均 pending；本次没有运行。outcome: needs_fix；Plan 状态回到 `executing`，T-02/T-05/T-06/T-07 复开，current_step=3；next=work，完成 R1..R4 后提交新冻结提交并重新最终复审。

### 2026-10-19 复审修复收口（work handoff）

- stage: work；review_scope: final 的 R1 已退回 work，本记录只声明已完成修复，不代替 T-08。复审使用的旧实现基线为 auto-lang `cacb9fcc335b4addc5a0db67e7c64282b30d445e`、book `de49ae222da45fd9b33eeb443fdd776658a1750c`；修复后的冻结提交为 auto-lang `b8901c4e4de237c53cd89cfa23b844066234fb70`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`。实施树分别是 `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev` 与 `D:/autostack/.wt/lang-813/book` / `plan-813-book`，冻结时均 clean。最初批准基线仍见上方 R1 记录；auto-down 依赖修订仍为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，其 12 个既有 WIP 文件未纳入或清理。
- 修复闭合：R1 为 Base64/Hex 书稿改用现有依赖 API 并恢复双语真实运行；R2 为 `/api/test` 实际 handler 增加成功/失败断言、超时返回、下一次请求恢复和并发输入隔离测试；R3 为 ch22 新增本地 TCP server/client 与说明、端到端检查 connect/send/reply/disconnect、正常进程退出和监听端口关闭；R4 的 SD-05/frontmatter 已在 R1 修复，当前重新核对过。
- 最终静态清单：54 页、1831 个代码围栏、310 个 Listing 视图、96 个独立 AutoFence，共 406 个 catalog 入口：382 run、14 test、6 display-only、4 local-only。完整执行报告 `attachments/813/tapl-full-execution.json.txt` SHA-256 `FFDBF8863F57C0FB24F967B308CDCB8FB12B4D2B8D6F19D6019FD43DF94D9758`；`tapl/examples.json` SHA-256 `ED4E42E87DBEFF12B4E46291A63CAE92333FAFDA5270182C39420ED4CE1817D3`。静态清单 CSV/MD SHA-256 分别为 `1AFDEA629C16D765D0195CFF0D0A907D0D8E8B0B1B2B56B5DFFE44EFA38B4FA3` / `9334D39D16B808807D53D7E455030EDED638E7F7E261DE714757E3A276DEC760`。
- 可重放结果：`check-tapl-examples.py --audit-only` 0 错；同工具全量执行 396/396（382 run + 14 test）通过、0 失败；负控在一项有效样例通过后分别将 expected stdout 和 page line 锚点实际变异，均返回退出码 1；最终网站生产构建 96.38s；`CI=1 AUTO_WEBSITE_TEST_PORT=4185 npm run test:e2e -- tests/tapl-run.spec.ts` 6/6；loopback chat verifier 输出 `PASS: connect, send, reply, disconnect, and graceful shutdown`。R2 handler 三测试 3/3；stdlib assembly 定向测试 1/1；`cargo check -p auto-lang` 与 `cargo build -p auto` 在本计划此前验证通过。`git diff --check` 使用 `core.whitespace=cr-at-eol` 在两树通过。
- AC-01..06 现有工作证据均为 pass；AC-07 交给 T-08 从冻结差异重新审计。C 编译器仍不可用，C 对应代码作了逐段核对但未编译，这是明确环境限制，不冒充运行证据。网站构建仍有既有自定义语法语言回退到纯文本的提示。
- merge_owned_gates: `cargo t`、VM 触面 `cargo tv`、书测试触面 `cargo tb` 均 pending；没有在 work/internal review 阶段运行全档。`tf` 由 T-09 主检出按批量回归到期规则裁决。

stage: work | plan_id: PLAN-813 | plan_revision: 1 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-813/auto-lang` + `D:/autostack/.wt/lang-813/book` | branch: `plan-813-dev` + `plan-813-book` | reviewed_commit: `b8901c4e4de237c53cd89cfa23b844066234fb70` + `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` | diff_base: `cacb9fcc335b4addc5a0db67e7c64282b30d445e` + `de49ae222da45fd9b33eeb443fdd776658a1750c` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | task_ids: T-02,T-05,T-06,T-07 | evidence: this record and `attachments/813/tapl-full-execution.json.txt` | blockers: no work-owned criteria; C compiler unavailable, disclosed above | next_responsible_role: review

### 2026-10-19 最终复审 R2（needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: Codex 当前会话，与实施共用上下文；按规范要求未伪称独立任务/代理，重新从当前冻结提交、实际 Spec 与验证器行为核对。Plan revision 1；worktree/branch 为 `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev` 与 `D:/autostack/.wt/lang-813/book` / `plan-813-book`；被审提交 auto-lang `b8901c4e4de237c53cd89cfa23b844066234fb70`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`，相对 work handoff 基线 `cacb9fcc335b4addc5a0db67e7c64282b30d445e` / `de49ae222da45fd9b33eeb443fdd776658a1750c`，两个树当时 clean。
- acceptance_results：AC-01..06 依工作记录通过（406 个目录入口与 1831 围栏、396/396 执行、handler 3/3、页面 6/6、chat 本地真实循环）；AC-07 needs_fix；AC-08 pending。Cargo `t/tv/tb` 仍为 merge-owned pending。
- finding PLAN813-R5 | P1 / Spec 与验证器合同不一致 | AC-01/AC-07；T-06/T-07：新 `book-example-execution.md` 只列 run/test/display-only，遗漏实际使用的 `local-only` 和支持的 `expected-error`；run 预期漏 `stdout_contains`；还把一次性 401 项旧基线写进规范。该 Spec 声称目录审计拒绝孤立锚点，但 checker 只验证 catalog→页面方向，未扫描页面中未入册的 `example-id`。`stdlib/project.md` 未记录本次实际修正的 `TcpStream.read_line` 在 peer EOF 时返回空字符串的行为。
- required correction：修正规范字段/模式/预期及历史计数表述；将 EOF 行为沉淀到 stdlib Spec；补齐 page→catalog 完整性审计并用真实孤立锚点变异验证失败；对修订后的 54 页目录和代表性执行路径重新回归。此前代码与书稿行为证据只覆盖当前提交，不自动覆盖新 validator/Spec 修订。
- evidence：直接读取代码冻结 diff、`docs/specs/website/design/book-example-execution.md`、`docs/specs/stdlib/project.md` 与 checker；独立枚举 `book/tapl/*.md` 中 406 个 example-id（当时 406 个唯一目录项全部对应，未变异时无孤立项），确认这证明 valid mapping，但并未证明 validator 对孤立项会拒绝。Spec 输入 hashes沿用本计划上一条 work handoff，修订后重取。
- outcome: needs_fix；状态回到 `executing`，T-06/T-07 复开，current_step=6；next=work。该次问题是在前两轮 review/findings 后从完整规范契约与验证器反方向覆盖审计中发现；第三次 final review 前要先完成此处提出的 coverage/design adjustment，不以修订文案替代负控证据。

### 2026-10-19 R5 修复收口（work handoff）

- stage: work；本次只修 Spec/验证器并重跑受影响门禁，没有改变实现架构或用户批准范围。auto-lang 新冻结提交 `110a5cb6f6fbfa59beece490fd815e9fe55f672c`，其直接前版 `b8901c4e4de237c53cd89cfa23b844066234fb70`；book 仍为 clean 的 `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`。worktree 分别 `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev` 与 `D:/autostack/.wt/lang-813/book` / `plan-813-book`；两树冻结时 clean。auto-down 仍 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未变更。
- PLAN813-R5 已闭合：`book-example-execution.md` 列全 run/test/expected-error/local-only/display-only 模式与 stdout/stdout_regex/stdout_contains/tests/diagnostic 预期；删除一次性 401 条统计。`stdlib/project.md` 记录 TcpStream.read_line 对端 EOF 返回空字符串且与 TcpStream.read 合同分开。checker 扫描全部 TAPL author Markdown 的 example-id，与 406 条目录双向核对并拒绝孤立/重复锚点。
- 验证器变异：有效单条 run 1/1 通过；将 expected stdout 改错、行锚点改旧、增加无目录记录的 AutoFence、expected-error 诊断改错，四种变异均退出 1；expected-error 的真实编译错误诊断通过。54 页全量静态审计 406 条通过、0 错；全量 run/test 重放 396/396 通过、0 失败。一次早期复跑中 13-5 出现无 stdout/stderr 的 CLI exit 1；独立用相同物化路径重跑 10/10 成功，随后全量再次 396/396 成功，未复现。
- 重新构建生产站点 170.01s 通过；最终 `CI=1 AUTO_WEBSITE_TEST_PORT=4185 npm run test:e2e -- tests/tapl-run.spec.ts` 6/6。此前已针对 Rust 改动运行 `cargo test -p auto-playground routes::test::tests -- --nocapture` 3/3、`cargo test -p auto-lang --lib core_supported_symbols_resolve_and_bind -- --nocapture` 1/1、`cargo check -p auto-lang` 与 `cargo build -p auto`；R5 未改 Rust 或依赖，按原证据复用。loopback chat 端到端输出仍绑定不变的 book commit `2de2fe3`。C compiler unavailable，C 目标代码未编译；网站自定义语法仍提示回退为纯文本。
- 最终全源报告 `attachments/813/tapl-full-execution.json.txt` SHA-256 `E732684902B6B9C30312A227A566E7691E0A0686547E662DA4B83ED04D0C1D53`；catalog `tapl/examples.json` SHA-256 `ED4E42E87DBEFF12B4E46291A63CAE92333FAFDA5270182C39420ED4CE1817D3`。review Spec inputs SHA-256：learning-reading `6D72CF5DAE547E6DE28F03DB7E522F1DCD4312B7BECE8FCBAE9A45C87C9BF5EA`；book-example-execution `1B753BE438D16216A49F9B198B5D92978343D7807343BD3F212592FF3B88486A`；website/project `3B3F342A81003868949209559155A7B3FDD3AA262D79044059A0AC3D8254EE13`；auto-playground/project `87039E4443AB4A0F5C1178979EB0E5D7A6ACA6F899FE98CFF7362F1B58181ADD`；stdlib/project `5505C717EC4156308D2FB98F91F1F061C9179859FDEFCE3DADB98CC6EEEADB52`。依赖 hashes保持上条 work handoff 记录值。
- merge_owned_gates: `cargo t` / `cargo tv` / `cargo tb` pending；`tf` 留 T-09 到期裁决。AC-01..06 有新/复用的有效证据；AC-07 重新交 T-08，AC-08 pending。

stage: work | plan_id: PLAN-813 | plan_revision: 1 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-813/auto-lang` + `D:/autostack/.wt/lang-813/book` | branch: `plan-813-dev` + `plan-813-book` | reviewed_commit: `110a5cb6f6fbfa59beece490fd815e9fe55f672c` + `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` | diff_base: `b8901c4e4de237c53cd89cfa23b844066234fb70` + `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | task_ids: T-06,T-07 | evidence: this record and `attachments/813/tapl-full-execution.json.txt` | blockers: no work-owned acceptance blocker; disclosed toolchain limits above | next_responsible_role: review

### 2026-10-19 最终复审 R3（pass）

- stage: review；review_scope: final；reviewer_identity/limitations: Codex 当前会话，与实施共用上下文；未声称有独立会话/代理。按工件重新核对最终冻结树，且对未变实现明确复用同哈希下已重跑证据。Plan revision 1；auto-lang commit `110a5cb6f6fbfa59beece490fd815e9fe55f672c`，book commit `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；diff base 分别 `30c1b9a7edc3112cac99089d14169d9f9bdeaeb6` 与 `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；两个目标树 clean。依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 保持不变，其共享 12 个 WIP 文件未进入 diff。
- acceptance_results：AC-01 pass（54 页/1831 围栏；310 Listing + 96 AutoFence = 406 个唯一 page/catalog 绑定；模式 382 run、14 test、6 display-only、4 local-only；396 可执行项 396/396）；AC-02 pass（同源 API、编辑后的完整源和输入/文件传递，TAPL 浏览器 6/6；ch08 按用户裁定保留可运行改写并解释差异）；AC-03 pass（真实 handler 输入并发隔离/timeout recovery 3/3，VM hosted-input 定向 3/3，源目录每次物化清理）；AC-04 pass（test 入口真执行并能报告失败，预期诊断控制和 local TCP 完整周期均通过）；AC-05 pass（Base64/Hex 复用现有依赖，全 catalog 实跑；TcpStream EOF 返回合同有定向验证）；AC-06 pass（双向目录静态审计 406/406，运行/测试 396/396，负控退出码正确）；AC-07 pass（Plan/SD/frontmatter/hash、遗漏/绕过扫描、差异与健康检查完成）；AC-08 pending（归 T-09）。
- validator counters：有效示例 1/1 pass；错误 stdout 预期、陈旧锚点、孤立 AutoFence、错误 expected-error diagnostic 四项实际变异各以退出码 1 拒绝。静态审计 post-commit 重跑 406 entries、0 failure；完整 CLI 回放报告绑定冻结书稿与依赖，396 executed、396 pass、0 fail。第 13 章 13-5 在一次完整回放有过无诊断 exit 1，但 exact materialized path 10/10 重跑及随后完整 396/396 均通过，未复现；记录此瞬态，不隐去。
- 规范增量：SD-01..05 target 均存在、before/after 与实现吻合；SD-02 包含五种模式、所有输出预期和双向锚点审计；SD-04 记录 TcpStream.read_line EOF 空串行为；SD-05 链接网站规范。`touched_goals` GOAL-014/016 已在 `docs/specs/goals.md` 核对有效。review Spec inputs SHA-256：learning-reading `6D72CF5DAE547E6DE28F03DB7E522F1DCD4312B7BECE8FCBAE9A45C87C9BF5EA`；book-example-execution `1B753BE438D16216A49F9B198B5D92978343D7807343BD3F212592FF3B88486A`；website/project `3B3F342A81003868949209559155A7B3FDD3AA262D79044059A0AC3D8254EE13`；auto-playground/project `87039E4443AB4A0F5C1178979EB0E5D7A6ACA6F899FE98CFF7362F1B58181ADD`；stdlib/project `5505C717EC4156308D2FB98F91F1F061C9179859FDEFCE3DADB98CC6EEEADB52`。输入依赖 hash：Cargo.lock `1E1E432EEFC4C75981A5B1E788830A6791E911949FD3153B3E22570EFC1BFF9E`；website/package-lock `DC7121A2C03D91D0A20CD8C3CB8DC31B68CA235E8FC2364F0837082434F6FF82`；examples.json `ED4E42E87DBEFF12B4E46291A63CAE92333FAFDA5270182C39420ED4CE1817D3`。
- health/debt scan：两树冻结 diff `git diff --check`（Windows CRLF-aware）无 whitespace error；相关 Rust 文件已定向 rustfmt，reviewed diff 无 debug prints/无新增 skip 或 ignore。保留环境限制：无可用 C 编译器，C 对应实现已核对但未编译；全站现有自定义语法高亮回退到纯文本提示。未发现 in-scope 延后/绕过需要登记为新 debt。
- pending_closeout_gates: `cargo t`、VM 触面 `cargo tv`、书测试 `cargo tb`；必须在此最终 pass 后、T-09 合入前执行。`cargo tf` 仅按主检出最近批量收据和 48h/计划号规则裁决。outcome: pass；Plan 状态 `reviewed`，current_step=9；next=T-09。

stage: review | review_scope: final | reviewer_identity/limitations: Codex 当前会话（与实施共用上下文，按最终工件重建） | plan_id: PLAN-813 | plan_revision: 1 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-813/auto-lang` + `D:/autostack/.wt/lang-813/book` | branch: `plan-813-dev` + `plan-813-book` | reviewed_commit: `110a5cb6f6fbfa59beece490fd815e9fe55f672c` + `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` | base_commit: `30c1b9a7edc3112cac99089d14169d9f9bdeaeb6` + `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`; Cargo.lock/package-lock hashes above | spec_inputs: §9 R5 repair handoff | acceptance_results: AC-01..07 pass, AC-08 pending | pending_closeout_gates: `cargo t`, `cargo tv`, `cargo tb` | findings: none | evidence: `attachments/813/tapl-full-execution.json.txt` SHA-256 `E732684902B6B9C30312A227A566E7691E0A0686547E662DA4B83ED04D0C1D53`; static audit/negative controls/site build and Playwright listed above | next: T-09 merge

### 2026-10-19 独立最终复审 R4（needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 独立子任务 `/root/plan813_final_review`，相对实现会话使用新上下文；可读取 auto-lang 冻结提交 `380df7a1adad662be2575b4248fef713753eaf45` 的源码和 Plan 记录，但其环境无法直接读取 book worktree，故书稿全量执行/浏览器结果引用已记录收据而非本轮重跑。book commit `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；Plan revision 1；依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- finding PLAN813-R6 | P2 / Spec 证据精度与计划索引缺项 | AC-05/AC-07；T-07/T-08：math Spec 未列四个浮点 API 的返回类型；原描述没有说明签名 parity harness 只比较名称和元数、不比较类型。复审者未找到覆盖这四项的测试，建议校正到可见证据。复核代码时确认 `a2r_std_signature_parity` 的 `pairs()` 包含 `math.rs.at` → `a2r-std::math`，会检查名称和元数；`a2r-std` 另有 f64 行为单测。计划索引缺 `stdlib`、`website`、`auto-playground` 条目，VM 索引亦随 `affects` 范围补录。
- R6 修复已提交：code worktree `637b2c0c88b22781339abd44768d873acf5de343` 修正 `stdlib/project.md` 为四项 `double` 返回，并准确限定 parity/行为单测覆盖；新增 `a2r-std/project.md` 数学 helper 说明和四个受影响模块的 PLAN-813 索引。执行 `python scripts/spec-index.py` 后生成索引仍为 47 projects；仅有仓库已有的 3 个 ungrouped-project 提示。`git diff --check` 通过。
- outcome: needs_fix（此轮记录所述修复后等待新最终复审）；Plan 状态退回 `executing`，T-08 reopened，current_step=8；merge-owned `cargo t` / `cargo tv` / `cargo tb` 仍 pending。

### 2026-10-19 独立最终复审 R5（pass）

- stage: review；review_scope: final；reviewer_identity/limitations: 独立子任务 `/root/plan813_final_review`，新上下文复审。最终绑定 auto-lang `5c6daa1439d95dbf2811f4f242895490b6d40d2a`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`，Plan revision 1；base 分别为 auto-lang `13367d127584461449ee2f151c0c5987f0740044` 与 book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。书稿工作树本轮仍不可读取，书籍验收依赖冻结提交及已有报告，不声称本轮复跑。
- acceptance_results：AC-01..06 复用冻结的全书报告和 R5 修复记录（54 页、406 项 catalog、396/396 实际执行通过，浏览器 `tapl-run.spec.ts` 6/6，handler 3/3，loopback chat connect/send/reply/disconnect/graceful shutdown PASS）；AC-07 pass（独立复核差异、SD-01..06、触面元数据、负控/遗漏/未批准延后/Workaround 与健康边界）；AC-08 pending，归 T-09。
- 规范/索引：`supersedes_spec_components` 与 SD-01..06 对齐；`affects` 包含 website、auto-playground、stdlib、auto-lang/vm、a2r-std；`a2r_std_signature_parity` 覆盖 `math.rs.at` ↔ `a2r-std::math` 名称/元数（不比较类型），返回类型为 double，行为单测范围准确；四个既有模块 plans.md 均链接当前计划并处于 reviewed（merge 门禁待完成）。a2r-std 无既有 plans.md。
- review input SHA-256：`learning-reading.md` `6D72CF5DAE547E6DE28F03DB7E522F1DCD4312B7BECE8FCBAE9A45C87C9BF5EA`；`book-example-execution.md` `1B753BE438D16216A49F9B198B5D92978343D7807343BD3F212592FF3B88486A`；website `project.md` `3B3F342A81003868949209559155A7B3FDD3AA262D79044059A0AC3D8254EE13`；auto-playground `project.md` `87039E4443AB4A0F5C1178979EB0E5D7A6ACA6F899FE98CFF7362F1B58181ADD`；stdlib `project.md` `4FEB124789EE25E23B57C97A5769C1401895DE8233268CDDE3716204668B517B`；a2r-std `project.md` `7AC4B9586FA97937C6071917ABEAF00766887B04B6F987F0C066130E8B5D6F05`；`Cargo.lock` `4BC409294E0466BAF940AE4B9303DAB92DD1FE8ADE3EFC5777896F8A1E2B4E0F`；website package lock `DC7121A2C03D91D0A20CD8C3CB8DC31B68CA235E8FC2364F0837082434F6FF82`；catalog `ED4E42E87DBEFF12B4E46291A63CAE92333FAFDA5270182C39420ED4CE1817D3`。
- 限制/健康：C 编译器不可用，C 分支人工核对未编译；全仓 rustfmt 被既有问题阻断，本次涉及 Rust 文件定向格式检查已通过；既有 compiler warnings 已如实记录。没有新发现需登记的债务。此前 `cargo t`/`cargo tb` 曾揭示新增 math 名称缺少 a2r-std 镜像，已由 R6 修复；最终三门禁须在此最终 pass 之后于冻结提交执行。
- outcome: pass；Plan 状态 `reviewed`，current_step=9；pending merge-owned gates `cargo t`、`cargo tv`、`cargo tb`；next=T-09。

### 2026-10-19 T-09 门禁故障归因与修复（work handoff）

- R5 冻结的 auto-lang `5c6daa1439d95dbf2811f4f242895490b6d40d2a` 上首次 `cargo t` 未通过：4851 passed、3 failed、1 timed out、1549 skipped；失败涉及两个并行负载下的进程收敛用例、PLAN-796 语料和 IPC P95。定向单测分别重跑后，两个进程收敛用例与 IPC P95 均通过。
- 语料失败可稳定复现：`infer-bindings.case.ad` 与 `cache-roundtrip.case.ad` 引用的两个 `oracle/*.bindings.json` 被根 `.gitignore` 的 `*.json` 规则忽略，因而未入库；该缺陷在 PLAN-813 代码变更之外。新增并跟踪准确的 `x/y/flag` 与 `x` 绑定 oracle，并为两文件加窄范围 ignore 例外。`probe_adapter_corpus_executes_green` 在新夹具上定向通过 1/1。修复提交 `6643ceaeb`。
- 首轮 `cargo t` 同时生成了标准库矩阵的确定性更新：`read_line_opt` 使 `auto.io` 公共符号数由 10 变为 11；保留更新后的 JSON/Markdown 报告（提交 `01662e818`）。IPC 测试产生的机器相关周转报告已恢复到原提交，不纳入差异。
- 最新冻结提交：auto-lang `4f813786781ab09d66ae7f10a5cf48e653463a7f`（含上述夹具、矩阵和索引收口），book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；基线 auto-lang `13367d127584461449ee2f151c0c5987f0740044`、book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-08 已重新打开后由 R6 通过；R6 handoff 记录此前 cargo t 失败及定向复验，不将失败门禁标为通过。最终 `cargo t`、`cargo tv`、`cargo tb` 仍 pending。

### 2026-10-19 独立最终复审 R6（pass）

- stage: review；review_scope: final；reviewer: 独立子任务 `/root/plan813_final_review`。最终绑定 auto-lang `4f813786781ab09d66ae7f10a5cf48e653463a7f`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；基线分别为 `13367d127584461449ee2f151c0c5987f0740044` 与 `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。书仓在复审环境不可直接读取，书稿验收沿用冻结提交和先前全量执行收据，未声称本轮重跑。
- 新增差异复核：两条 `.gitignore` 例外仅涵盖缺失的 PLAN-796 绑定 oracle；`infer-bindings` 的 `x:i32, y:i32, flag:bool` 与 `cache-roundtrip` 的 `x:i32` 顺序和类型匹配语料，runner 按序比较。`read_line_opt` 矩阵行列值为 11 且各后端状态如实；四个模块索引状态与本计划 reviewed 一致。`4f813` 相对通过内容审查的 `9920e7f` 仅改四个索引状态标签，无实现/规范变化。
- AC-01..06 沿用 R5 绑定的 54 页、406 项目录、396/396 实跑、浏览器 6/6、handler 3/3 和本地 TCP 周期证据；AC-07 pass；AC-08 仍 pending，归 T-09。该复审未运行 cargo 门禁。
- pending_closeout_gates: `cargo t`、`cargo tv`、`cargo tb`；执行顺序按代码工作树最终冻结 revision 串行运行。outcome: pass；Plan 状态 `reviewed`，current_step=9；next=T-09。

### 2026-10-19 T-09 门禁故障归因与修复（R7 work handoff）

- R6 绑定提交 `4f813786781ab09d66ae7f10a5cf48e653463a7f` 上的 `cargo t` 重跑两次均未通过。第一次由既有 `plan498_bar_group_emphasis` 失败终止（1863 pass / 1 fail / 1549 skipped / 19 canceled）；精确单测重跑 1/1 通过，未复现。第二次在 `plan707_cancel_active_job_aborts_and_frees_permit` 失败（2035 pass / 1 fail / 1549 skipped），精确单测也复现。
- 定责确认 PLAN-707 实现和测试文件均与 master 一致，失败由测试竞态造成：工作线程发出 entered 信号后，测试线程立即取消并开放 gate；运行时可能尚未处理 abort，job 先继续执行 marker。测试现改为保持 gate 关闭，在 2 秒界限内等待 active 许可回到基线，再断言 marker 未置位及迟到完成被拒绝。定向命令 `cargo test -q -p auto-lang --features ui-iced,test-cases --lib tests::plan707_cancel_tests::plan707_cancel_active_job_aborts_and_frees_permit -- --nocapture` 通过 1/1。
- 修复提交 `4a708c841`；模块索引随 T-08 重开更新并提交 `2451497f6`。当前 review target auto-lang `2451497f6c269c1cd378715645fa4426679704ea`，book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`，auto-down 依赖 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-08 reopened；此前 R6 证据不能替代对新 test diff 的复审；全部 merge-owned gates 待在 R7 pass 后的最终冻结提交上重跑。
- stage: work | plan_id: PLAN-813 | plan_revision: 1 | outcome: pass | worktree_path: `D:/autostack/.wt/lang-813/auto-lang` + `D:/autostack/.wt/lang-813/book` | branch: `plan-813-dev` + `plan-813-book` | base_commit: `13367d127584461449ee2f151c0c5987f0740044` + `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a` | code_commit: `2451497f6c269c1cd378715645fa4426679704ea` + `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` | dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | task_ids: T-09 test stabilization; T-08 reopened | evidence: scoped rerun 1/1; two failed full-gate attempts documented above | blockers: none for work repair; `cargo t/tv/tb` pending | next_responsible_role: review

### 2026-10-19 独立最终复审 R7（pass）

- stage: review；review_scope: final；reviewer: 独立子任务 `/root/plan813_final_review`。内容审查绑定 auto-lang `2451497f6c269c1cd378715645fa4426679704ea`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`；依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。reviewer 不能直接读取两个 worktree；主会话补核 `git status --short` / `git status --porcelain` 均为空，分别位于上述提交与 `plan-813-dev` / `plan-813-book` 分支。
- 新增测试差异通过：保持 gate 关闭后，取消必须使 wrapper 任务退出并让 active 许可在 2 秒内回归基线；迟到完成仍被拒。reviewer 确认没有弱化取消断言。R6 已审查的全书验收和 SD-01..06 证据继续适用；T-08 pass，AC-08 pending。
- reviewer 执行 `git -c core.whitespace=cr-at-eol diff --check 13367d1..2451497` 通过；索引状态和主计划当前阶段一致。reviewer 没有运行 `cargo t/tv/tb`，它们仍为 merge-owned pending gates。
- pending_closeout_gates: `cargo t`、`cargo tv`、`cargo tb`；必须在本复审对应的最终冻结 revision 上串行执行，再合并。outcome: pass；Plan 状态 `reviewed`，current_step=9；next=T-09。

### 2026-10-19 独立复审 R8（状态簿记复核，pass）

- stage: review；review_scope: status-only；reviewer: 独立子任务 `/root/plan813_final_review`。绑定 auto-lang `990ed70b9bf0da82b09596605e71486095060f53`、book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253`。独立确认 `990ed70` 只把四个 PLAN-813 模块索引从 executing 改为 `reviewed（merge 门禁待完成）`；主计划状态为 reviewed/current_step=9，T-08 R7 pass，T-09 三项门禁仍 pending。R7 内容复审继续适用。

### 2026-10-19 T-09 merge-owned 门禁结果

- 冻结 auto-lang `990ed70b9bf0da82b09596605e71486095060f53` 上串行执行：`cargo t` 通过 5321/5321（1549 skipped，1 slow）；`cargo tv` 通过 163/163（6693 skipped）。
- `cargo tb` 首次运行因 `http_upload_service::tests::plan730_raw_receive_commit_conflict_roundtrip` 返回 409 而失败；精确单测复跑 1/1 通过，未复现。第二次运行中 `tests::book_listing_tests::book_ch02_05` 与 `book_ch03_08` 失败，输入在 `book/rust/listings/ch02/listing-02-05/main.at` 与 `book/rust/listings/ch03/listing-03-08/main.at`，解析器要求 `->` 却收到 `=>`。
- 归属核对：book 分支 `d7a71a7..2de2fe3` 不改任何 `rust/` 路径；auto-lang `13367d1..990ed70` 不改 book listing harness、parser、lexer 或 transpiler。Plan 813 改动位于 `book/tapl`，因此该红是预存且超出“另外七本书”非目标。以 `AUTO_BOOK_LISTINGS` 指向主书仓的定向对照中，两个用例都依现有 dirty-book 守卫因 9 个既有 `.expected.rs` 修改而跳过；未将这个对照写成 `cargo tb` 通过。`cargo nextest list -p auto-lang --lib --features test-book` 确认标准 `cargo tb` 选择 `generate_book_expected`，该测试会写回 `.expected.rs`；因此未让整档生成器指向主书仓，也未触碰其 WIP。
- 对上述两项 Rust listing 红项，用户明确要求保留 PLAN-813 范围，并将问题和修复作为 PLAN-819 新 phase 记录；PLAN-819:r5 已提交修复并归档。它们在本次 PLAN-813 最终组合的 `cargo tb` 中均通过；见下方 R9 收据。原候选 P813-D1 已解决，当前新门禁阻断是 P812-D2 的 P765 Windows IAT 测试。

### 2026-10-19 R9 rebase 复审与 T-09 门禁重跑

- stage: review / merge；独立复审 reviewer 为 `/root/plan813_final_review`。R9 绑定 auto-lang `5a52efa2b960425060d2d4b697904df4504669a2`、book `50153be9f4e0ac035ca3d0c2bd0da2d876d84875`，两个 worktree 均 clean。rebase range-diff 为 auto-lang 14/14、book 2/2 全部等价（`=`）；两仓相对上游的 PLAN-813 路径均无重叠。Book 基线 `d05218bff49a52314ce436f78d9df2a69bc3357e` 含 PLAN-819 修复，其 `book/rust` 触面与本计划 `book/tapl`、`scripts/sync_tapl_listings.py` 无重叠。CRLF-aware diff-check 通过。review outcome: pass；此前 R7 验收复审继续适用。
- 冻结提交门禁按序执行：`cargo t` 失败，5062 passed / 1 failed / 1549 skipped，P765 IAT 子进程退出码为 `-1073740940`（Windows `0xC0000374`）；相同日常 feature 配置隔离重跑 0/1 pass。`cargo tv` 通过 163/163（6693 skipped）。`cargo tb` 失败，5046 passed / 1 failed / 1547 skipped；失败仍为同一 P765 测试。`AUTO_BOOK_LISTINGS` 指向 PLAN-813 Book worktree；其中 PLAN-819 修复目标 `book_ch02_05`、`book_ch03_08` 分别通过。`test-book` feature 下隔离重跑亦 0/1 pass，退出码 `-1073741819`（Windows `0xC0000005`）。原始日志及 SHA-256：`merge-cargo-t-r9.log.txt` `6AC8B454805156D4E91520BB037D96983E92D761FC2483AC678A4A4D714CA317`；`cargo-t-isolated-p765-r9.log.txt` `49C7EFDFC39C86ED4F20F5768070780AC2AF20D788169264F77B6C9650F3390A`；`cargo-tv-r9.log.txt` `EFAB529A4954F74AB16814E846F49DC962FCBE7541804D705D49567AF5E3D731`；`cargo-tb-r9.log.txt` `CE87E2056F3AE6B83316675679DD1C74DC919972193A4C83B68C835FC2B4DFD0`；`cargo-tb-isolated-p765-r9.log.txt` `B63417E68A154164187DD007211A7DEC842C4B3925020C287E9A1EE066AFE39B`。
- 归因边界：P765 测试和 `install_exit_process_trace` IAT 钩逻辑在 PLAN-813 diff 中未修改，已存在的观察登记为 P812-D2；本轮两项整档和两种 feature 配置下的隔离重跑均复现，故更新该债务记录。PLAN-813 虽在同一 `stdlib.rs` 增加 hosted 文件/输入能力，现有证据不能证明二进制变化与该退出码无关，也未证明根因；不将其标成 PLAN-813 引入或已修复。按门禁规则 T-09 仍 pending，禁止双仓合入、归档和清理，待 P812-D2 的根因被解决并在最终冻结组合上重跑门禁。
- outcome: review pass / merge blocked；Plan remains `reviewed`, current_step=9；`cargo tv` pass，`cargo t` / `cargo tb` fail，T-09 incomplete；next=closeout after P812-D2 resolution. No code changes were made after R9 review.

### 2026-10-19 Phase 2 P765 FFI ABI 修复（T-10 work handoff）

- R9 两项门禁及隔离重跑曾在 P765 子进程失败。临时探针确认 `install_exit_process_trace` 成功补丁 2 个 IAT 槽并进入 `traced_exit_process`，但未完成栈审计。Windows `RtlCaptureStackBackTrace` 的正式签名包含 4 个参数，第 4 个 `BackTraceHash` 是可选指针；原 FFI 只声明/传入前三个参数。补齐返回类型 `u16` 和 `*mut u32` 参数，并在 ExitProcess 与 TerminateProcess 两条路径均传 `NULL`。
- 验收：`cargo nextest run -p auto-lang --lib --features test-book p765_iat_trace_writes_audit_and_preserves_exit_code` 1/1 pass（6860 skipped）；相同过滤器在 `--features ui-iced,test-cases` 下 1/1 pass（6855 skipped）；`cargo check -p auto-lang` exit 0（405 条既有 warning）；`rustfmt --edition 2021 --check` 与 `git diff --check` 通过。临时标记与结果见 `attachments/813/p765-install-markers-r11.log.txt`、`p765-hook-entry-r12.log.txt`、`p765-ffi-fix-r13.log.txt`、`p765-daily-feature-r14.log.txt`、`cargo-check-p765-r14.log.txt`；关键日志 SHA-256：test-book `9250E6BC41E691EBB80D21C606EC169F1A91578212C8308C5E00739AB189F0E1`；daily-feature `E00B63242D29020B5C0ED60AEC8265D90990F9C72BE25B6FE6D454078D64789D`；cargo-check `CFFAFF51458A71B40F7D296DA330EDCA796FF140EC78A8DC24A8BEDEA728F005`。
- 提交：auto-lang `78e1c31e8be30b2c758d6034f211f48c6731dc87`，base `5a52efa2b960425060d2d4b697904df4504669a2`；worktree `D:/autostack/.wt/lang-813/auto-lang` / `plan-813-dev` clean；book 保持 `50153be9f4e0ac035ca3d0c2bd0da2d876d84875`，auto-down 依赖 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。当前 Plan revision 2、status `executing`、current_step=8/10；T-08 final review 重开，三项全档门禁留待 review pass 后。
- debt closure：P812-D2 的未归因 P765 观察现已确认是漏传 Windows API 可选参数导致的 ABI 未定义行为，并由本阶段修复；不修改已归档 PLAN-819。

### 2026-10-19 独立最终复审 R10（pass）

- stage: review；review_scope: final；reviewer_identity: 独立子任务 `/root/plan813_final_review`。plan_id/revision: PLAN-813/2；reviewed_commit: auto-lang `78e1c31e8be30b2c758d6034f211f48c6731dc87`，book `50153be9f4e0ac035ca3d0c2bd0da2d876d84875`；base: auto-lang `5a52efa2b960425060d2d4b697904df4504669a2`，book `d05218bff49a52314ce436f78d9df2a69bc3357e`；dependency: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（保留 12 个既有 WIP 文件）。
- 接受结果：V-09 pass；reviewer 在 `test-book` 配置下重跑 P765 1/1（6860 skipped），在 `ui-iced,test-cases` 下 1/1（6855 skipped）；两树 clean，`rustfmt` 与 CRLF-aware `diff --check` 通过。确认 FFI 四参数及 `u16` 返回匹配 Windows 文档；审计检查 ExitProcess 路径实际断言 exit 9、trace 与 frames，并静态核对 TerminateProcess 调用点。P812-D2 归因与闭环正确，无新增 finding。
- 既有 AC-01..06 和 SD-01..06 的 R9 证据仍适用（仅改一处 Windows FFI ABI，不改书稿、前端、API 或 Spec）；T-10 无规范增量。AC-07 pass。pending_closeout_gates: `cargo t`、`cargo tv`、`cargo tb`；本轮复审未运行全档。outcome: pass；Plan status=`reviewed`，current_step=9/10；next=T-09 merge gates。

## 10. 待澄清事项

必需决策：已由用户确认 r1“两仓全书修复 + 网站 Run/输入/测试/依赖支持”范围。T-01 的实现选择不改变范围或兼容承诺。

T-01 内解决 stdin/EOF 最小接口、真实公共库缺口和执行元数据字段，不另加问卷；仅在必须改变目标/验收/兼容承诺或扩大架构时提交具体裁定。
