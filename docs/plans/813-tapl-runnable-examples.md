---
plan_id: PLAN-813
status: executing
feature_name: tapl-runnable-examples
author: [Codex]
plan_revision: 1
created_at: 2026-10-19
updated_at: 2026-10-19

supersedes_spec_components: [docs/specs/website/design/learning-reading.md, docs/specs/website/project.md, docs/specs/auto-playground/project.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/website/design/book-example-execution.md]
touched_goals: [GOAL-014, GOAL-016]

affects: [website, auto-playground, stdlib, auto-lang/vm]
current_step: 6
total_steps: 9
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
| AC-07 | review | 冻结 diff、AC-01..06、SD-01..05 独立审计，扫描遗漏/未批准延后/workaround；零新增未处理 warnings/debug/格式问题；门禁 pending 明示。 |
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
| T-06 | work | T-02/04/05 | `scripts/check-tapl-examples.py`、`website/tests/tapl-run.spec.ts`；最终清单和可重放证据、规范增量 | ⏳ 复开 PLAN813-R5：执行合同规范必须准确覆盖 local-only / expected-error / stdout_contains；目录审计声称拒绝孤立锚点前，补充孤立 `example-id` 负控并覆盖 54 页。 |
| T-07 | work | T-06 | 两仓提交本计划改动；固定 code/input/dependency hash、结果/命令、债务和规范增量；主计划 execution_done | ⏳ 复开：规范与 checker 修订提交后重冻实现树，更新完整 hash/证据，再交最终复审。 |
| T-08 | review | T-07 | /auto-plan:review 从冻结源/全清单/AC/SD 重建证据及负控 | AC-07，pass 后 reviewed；merge-owned 全档门禁 pending，不能声称交付完成 |
| T-09 | merge | T-08 | /auto-plan:merge：按 diff 分级门禁→双仓合入→SD/ledger/index→docs/plans/archive/→到期裁决→guard 清理 | AC-08；每树移除前 wt-guard clean，保留他人 WIP，提供实际收据 |

T-01..T-05 保持完成。T-08 的 R2 复审发现 PLAN813-R5 并退回工作阶段，当前复开 T-06/T-07；修复后需以新冻结提交重新做最终复审。`cargo t`、`cargo tv`、`cargo tb` 仍是复审通过后的 merge-owned 门禁，未在执行阶段提前运行。

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

## 10. 待澄清事项

必需决策：已由用户确认 r1“两仓全书修复 + 网站 Run/输入/测试/依赖支持”范围。T-01 的实现选择不改变范围或兼容承诺。

T-01 内解决 stdin/EOF 最小接口、真实公共库缺口和执行元数据字段，不另加问卷；仅在必须改变目标/验收/兼容承诺或扩大架构时提交具体裁定。
