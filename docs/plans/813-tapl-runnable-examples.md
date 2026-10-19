---
plan_id: PLAN-813
status: executing
feature_name: tapl-runnable-examples
author: [Codex]
plan_revision: 1
created_at: 2026-10-19
updated_at: 2026-10-19

supersedes_spec_components: [docs/specs/website/design/learning-reading.md, docs/specs/auto-playground/project.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/website/design/book-example-execution.md]
touched_goals: [GOAL-014, GOAL-016]

affects: [website, auto-playground, stdlib, auto-lang/vm]
current_step: 1
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
| AC-07 | review | 冻结 diff、AC-01..06、SD-01..04 独立审计，扫描遗漏/未批准延后/workaround；零新增未处理 warnings/debug/格式问题；门禁 pending 明示。 |
| AC-08 | merge | 分级门禁后双仓合入、规范/账本/索引回写、归档；fresh guard clean 后清理所有实施树/分支，tf 到期裁决及实际收据完整。 |

## 8. 执行步骤

预期唯一实施组 `D:/autostack/.wt/lang-813/{auto-lang,book,auto-down}`；lang 分支 `plan-813-dev`，book 分支 `plan-813-book`，auto-down 只读 detached。先确认书仓 AGENTS、基线和两仓 WIP，使用 `AUTO_BOOK_ROOT`/`AUTO_BOOK_LISTINGS` 指向兄弟仓，**禁止 junction/symlink**。全部代码/构建/运行在实施树，计划簿记留主检出。创建树前按仓库要求提交本计划和取号簿记，不夹带他人修改。

| ID | owner_stage | 依赖 | 操作/真实路径或符号 | 结果及验证 |
|---|---|---|---|---|
| T-01 | work | 用户确认 r1 | 建实施组，核对 prepare-content.js/auto-fence-md.ts 与书稿；公共 CLI/API/test_code 跑 §5.1 探针，设计执行元数据和审计入口 | ✅ 完成：决策与基线探针已回写 §5.1 并记录于 `t01-findings.md`；T-02..06 依赖此合同 |
| T-02 | work | T-01 | Playground `src/routes/run.rs,vm_runner.rs,project.rs`、auto-lang `src/lib.rs,vm/host_capabilities.rs,vm/ffi/stdlib.rs` 最小相关路径 | 输入/测试/fixture 接线，定向 Rust/API 检查及 cargo check；V-03/04/05/07；AC-03/04 |
| T-03 | work | T-01/02 | website CodeView.vue/AutoFence.vue、auto-fence-md.ts、prepare-content.js、scripts/build-playground-notes.mjs；必要的包 SnippetRunner.vue/usePlayground.ts/types.ts | 共用描述/客户端、完整源/输入界面与配置/错误/服务模式；定向 tapl-run + 原阅读测试；AC-02/03/04 |
| T-04 | work | T-01/02/03 | book/tapl ch00..13 中英、对应 listings；上下文、猜数、模块、错误教学；必要的标准库/绑定局部修正 | 每条描述/期望更新、逐块定向验收；V-02/03/06/07；AC-01/02/03/05 |
| T-05 | work | T-01/02/03 | book/tapl ch14..22 中英、附录 A..D、listings/pac/fixture/受影响 expected；文件/测试/标准库/异步/聊天及通用缺口 | 正反例和完整工程；中英/多目标展示真实，不能静默降标准；V-04/05/06/07；AC-04/05 |
| T-06 | work | T-04/05 | 新增 scripts/check-tapl-examples.py、website/tests/tapl-run.spec.ts；最终清单和可重放证据、SD 草案 | 全有效源复验、网站生产构建/阅读定向 Playwright、两仓 diff check；AC-01..06 通过，不跑整档 Cargo |
| T-07 | work | T-06 | 两仓提交本计划改动；固定 code/input/dependency hash、结果/命令、债务和规范增量；主计划 execution_done | 无 pending work，实施树冻结，交 review；current_step=7/9 |
| T-08 | review | T-07 | /auto-plan:review 从冻结源/全清单/AC/SD 重建证据及负控 | AC-07，pass 后 reviewed；merge-owned 全档门禁 pending，不能声称交付完成 |
| T-09 | merge | T-08 | /auto-plan:merge：按 diff 分级门禁→双仓合入→SD/ledger/index→docs/plans/archive/→到期裁决→guard 清理 | AC-08；每树移除前 wt-guard clean，保留他人 WIP，提供实际收据 |

T-01 已完成；T-02..07 尚未执行。静态普查与 T-01 基线探针只界定修复方式，不冒算全源运行验收。

## 9. 复审记录

2026-10-19：用户确认 PLAN-813 r1 并要求实施；状态转为 executing。T-01 已在专用工作树完成：将静态清单对上生成页面（实际 399 Run，另两处 TOML 应静态展示）、执行 ch03/ch05/ch10 公共 CLI 探针、确认 stdin 空行/EOF 行为、确认 `/api/test` 缺失及 test VM 无 deadline，并回写元数据/输入/测试/API 决定。书稿静态基线为 54 页/1829 围栏/401 原始位置；逐项真实执行验收仍 pending。

## 10. 待澄清事项

必需决策：已由用户确认 r1“两仓全书修复 + 网站 Run/输入/测试/依赖支持”范围。T-01 的实现选择不改变范围或兼容承诺。

T-01 内解决 stdin/EOF 最小接口、真实公共库缺口和执行元数据字段，不另加问卷；仅在必须改变目标/验收/兼容承诺或扩大架构时提交具体裁定。
