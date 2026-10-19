---
plan_id: PLAN-819
status: archived
completion_kind: delivered
feature_name: rust-book-listing-syntax-fixes
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 5

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [book/rust/listings]
current_step: 8
total_steps: 8
---

# [PLAN-819] Rust 书 Listing 示例语法与金样修复

## 0. 变更摘要

修复 Rust 教程中 Auto Listing 把模式分支写成 Rust `=>` 的问题，改为 Auto 使用的 `->`。同步第 2 章英文与中文正文中的 Listing 2-5 代码片段，并用现有书稿对拍验证 Listing 2-5 与 Listing 3-8。新增 Phase 2 收口第 6 章：修正 listing 05/06/08/09 中的 Auto 分支箭头，并同步英/中文正文代码片段；逐项对拍门禁暴露的枚举 golden 漂移及同族的 listing 11，按实际转译结果更新必要 `main.expected.rs`。新增 Phase 3 收口最终 `cargo tb` 暴露的 ch08 三份旧 golden 与 ch09 listing 02 的 `=>` 语法错误，并同步中英文正文 Auto 示例。此计划不改解析器、转译器或对拍框架。

## 1. 目标

- G-01：`listing-02-05/main.at` 与 `listing-03-08/main.at` 使用有效 Auto 语法，能够通过现有 Rust 转译器。
- G-02：第 2 章英文/中文正文的 Listing 2-5 与作者 `.at` 源一致；两处产物与现有金样一致，或仅在实际生成结果证明有差异时更新对应金样。
- G-03：新增 Phase 2 修复 listing 05/06/08/09 的 `=>` Auto 语法错误，并同步中英文正文里的同一 Auto 代码片段；相应第 6 章定向测试通过。
- G-04：Phase 2 覆盖 `book_ch06_01/02/03/04/05/06/08/09/10/11`：逐项对拍、按真实输出更新必要金样（已观察的枚举差异为新增 `Hash` derive），全部定向测试通过。
- G-05：在干净、隔离的书仓工作树中，原两个定向 Listing 测试、Phase 2 十个定向 Listing 测试和最终 `cargo tb` 门禁通过；门禁不触碰主书仓已有的用户修改。
- G-06：Phase 3 修复最终 `cargo tb` 失败的 `book_ch08_01/02/04` 三份过期 golden，并将 `book_ch09_02` Auto 模式分支从 `=>` 改为 `->`，同步第 9 章英文/中文正文中的同源 Auto 片段；按修复后的实际输出更新 listing 09-02 的旧 golden。
- G-07：`book_ch08_01/02/04` 与 `book_ch09_02` 四个定向测试在隔离书树均通过；ch09 EN/ZH Auto block 与 `.at` 一致，Rust `match =>` 未改；四份必要 golden 与当前转译产物逐字节一致。
- G-08：revision 5 最终复审通过后，在冻结组合执行 `cargo tb` 全档并通过；生成器副作用已清理。主书仓 9 项用户 WIP 与计划实产哈希完全一致，合入前保存为可恢复 stash；fast-forward 后九项文件字节不变。完整结果为 5312 passed、1547 skipped、1 slow；因默认并发出现两次未修改测试的间歇性失败，最终通过运行将 `NEXTEST_TEST_THREADS` 限为 4。

非目标：改 Auto 语法规则或 Rust 转译实现；翻修其他章节/其他书（Phase 3 列明的 ch08/ch09 失败闭包除外）；修改 `book_listing_tests` 守卫或全量金样生成器；更新本计划 §5.1 未列出的金样；部署或修改外部服务。

## 2. 架构方案

Phase 1 以书仓作者源为唯一修改源。修正 `.at` 源及正文中实际重复的 Auto 代码，利用现有 `tests::book_listing_tests` 进行转译产物与金样对拍；仅在确认源改动导致的差异后更新两个目标金样。Phase 2 先将 listing 05/06/08/09 的四组模式分支 `=>` 改为 `->`，同步英/中文书稿里的 Auto 片段，再用真实转译结果逐项核对 ch06 原门禁九个目标及直接过滤器发现的 listing 11。只更新对拍证明确需更新的 `main.expected.rs`：01/02/03/04/10 的枚举输出新增 `Hash` derive；05/06/08/09 还包含旧标准库导入、类型宽度、借用/分支形状等转译产物变化；11 输出移除了不必要的 `.unwrap_or(0)`。Phase 3 依据 Phase 2 冻结组合的最终 `cargo tb` 四个失败，更新 ch08 listing 01/02/04 的 golden 至实际输出，并修正 ch09 listing 02 Auto 分支箭头及中英文正文同源片段。金样只取自隔离计划书树的实际转译产物；主书仓已有用户 WIP 先比较哈希并完整保留。

实现只改书仓。Auto-lang 测试代码作为只读依赖，用专用 sibling worktree 运行，`AUTO_BOOK_LISTINGS` 指向该计划的书仓树。整档 `cargo tb` 会执行 `generate_book_expected` 并写出金样，所以只能在干净计划工作树运行；不可把它指向主书仓或带用户 WIP 的目录。Phase 2 的源、正文与 golden 目标都限于 §5.1 表列路径；既有书仓 WIP 内容先核对后保留。

## 3. 技术栈

Auto `.at`、书稿 Markdown、现有 Rust 转译函数 `transpile_rust`、`test-book` 测试档与书仓 Listing 金样。

## 4. 需求分析与背景调查

### 授权及工作区

用户于 2026-10-19 选择保持 PLAN-813 的既定范围并暂缓其合并，同时要求为 Rust 书中发现的两处运行/转译失败另立修复计划。用户随后明确指示“计划819: 实施”，授权按本合同实施。

用户于 2026-10-19 在 PLAN-819 的第一次 `cargo tb` 发现 ch06 门禁失败后明确指示：“不要独立计划，直接更新计划819,记录问题和解决方案，作为新的phase解决”。这项明确授权将问题纳入 PLAN-819 新 Phase 2。后续定向复现确认 05/06/08/09 作者源有同类 `=>` 语法错误，且直接过滤器额外发现 11 号同族金样漂移；Phase 2 因此覆盖整个已观察到的 ch06 问题闭包，仍排除其他章节和编译器实现。

实施时使用唯一 sibling group：`D:/autostack/.wt/lang-819/{auto-lang,book,auto-down}`；分支分别为 `plan-819-dev` 与 `plan-819-book`，`auto-down` 只读 detached。Phase 1 已在书仓 master 提交（`abe1e9e`、`fb744f5`），r4 的 Phase 2 worktree 从当前 master 重建；修改、构建和测试均在组内，主仓已有 WIP 不得覆盖或纳入提交。不得在 worktree 内创建 junction/symlink。

### 现状与复现证据

- PLAN-813 §9 的 T-09 收据记录：冻结 code `990ed70b9bf0da82b09596605e71486095060f53` + book `2de2fe3c66ac54e22d98c6b077a92d1dce10e253` 上，`cargo t` 与 `cargo tv` 通过；`cargo tb` 的 `tests::book_listing_tests::book_ch02_05`、`book_ch03_08` 分别在以下 Auto 源中遇到 `=>`，解析器要求 `->`。
- 目标文件：`D:/autostack/book/rust/listings/ch02/listing-02-05/main.at`、`D:/autostack/book/rust/listings/ch03/listing-03-08/main.at`。第 2 章同一 Auto 片段也出现在 `rust/ch02-guessing-game.md` 与 `rust/ch02-guessing-game.cn.md`；第 3 章正文没有重复这段 Listing 3-8 源码。
- 对拍入口：`crates/auto-lang/src/tests/book_listing_tests.rs` 中 `book_ch02_05` 与 `book_ch03_08` 调用 `test_book_listing`。主书仓现存 9 个 `.expected.rs` 用户修改会触发 dirty-book skip；`generate_book_expected` 会重写所选书树中的多个金样，因此必须通过专用干净 worktree 验证。
- PLAN-813 记载的书仓基线为 `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；实际 commit、工作区状态和内容 hash 须在实施 worktree 建立时再次确认。起草时读取的 SHA-256：
  - `rust/listings/ch02/listing-02-05/main.at`: `BB1268CEDD8A4DB679E9038F296792D94A847599E897A6645EC3DEB612513BD0`
  - 对应 `main.expected.rs`: `65A946B491EEEB9D3B5C3DE1DCE53E240228CC630525DB70B5DBA8EBB0815BE6`
  - `rust/listings/ch03/listing-03-08/main.at`: `5090190BEF6E74ED612CF31FDCD20FB57C9F79FDF0A2B95456E0D136A2AD693D`
  - 对应 `main.expected.rs`: `4650A0292B4C530113DB92761D0DCB9A981543020D6E01F9364193D8ADA30EC7`
  - harness `crates/auto-lang/src/tests/book_listing_tests.rs`: `285794840881B9225D630F235FF7A386B5B206582F686904E1434A23C7521E45`
  - 参考 Spec `docs/specs/auto-lang/trans/overview.md`: `124735299C79F0857986309C8091ADADF38F5797E720E43CA01A71628AEB6209`
- Spec 无需增量：这是作者示例把其他语言的标点带入 Auto 源，不改变 Auto 语法、转译接口或测试合同。
- Revision 4 归因校正：r2 `cargo tb` 在 ch06 fail-fast 后报告 9 个失败名（01/02/03/04/05/06/08/09/10），但不能据此断定九项都是 golden drift。Phase 2 定向复现发现 listing 05/06/08/09 作者源均在 `=>` 处报 `UnexpectedToken: expected Arrow`；订正源后，这四项也与现有 expected golden 不一致（旧标准库导入、类型宽度、借用/分支形状等差异）。01/02/03/04/10 的枚举 derive 新增 `Hash`；同一 ch06 过滤还发现 listing 11 输出移除 `.unwrap_or(0)`。新 Phase 2 将这些 ch06 问题一起闭环；精确输出差异及主仓 WIP hash 在 T-05 收据补全。

## 5. 详细设计

### 5.1 修改边界

| 目标 | 修改 | 验收方式 |
|---|---|---|
| Listing 2-5 | `book/rust/listings/ch02/listing-02-05/main.at` 中两处分支箭头由 `=>` 改为 `->`；同步 ch02 英文/中文 Markdown 中对应 Auto 代码块 | `book_ch02_05` 通过；两种语言正文与作者源逐行对应 |
| Listing 3-8 | `book/rust/listings/ch03/listing-03-08/main.at` 中三处分支箭头由 `=>` 改为 `->` | `book_ch03_08` 通过 |
| Phase 2 第 6 章语法 | `book/rust/listings/ch06/listing-06-{05,06,08,09}/main.at` 中 Auto 模式分支箭头由 `=>` 改为 `->`；同步 `rust/ch06-enums.md` 与 `.cn.md` 的四个同源 Auto 片段 | 对应四个 `book_ch06_*` 定向测试通过；EN/CN 片段与作者源一致 |
| Phase 2 第 6 章 golden | `book/rust/listings/ch06/listing-06-{01,02,03,04,05,06,08,09,10,11}/main.expected.rs` 共 10 个候选目标；只按定向测试产生的完整输出核实并更新实际不匹配者 | 十个对应 `book_ch06_*` 测试通过；逐项确认 diff 仅为生成输出差异 |
| Phase 3 第 8 章 golden | `book/rust/listings/ch08/listing-08-{01,02,04}/main.expected.rs` | 三个对应 `book_ch08_*` 定向测试通过；diff 与实际转译输出一致 |
| Phase 3 第 9 章语法与正文 | `book/rust/listings/ch09/listing-09-02/main.at` 中两处分支箭头改为 `->`；同步 `rust/ch09-error-handling.md` 与 `.cn.md` 对应 Auto block | `book_ch09_02` 通过；EN/ZH Auto 片段逐行与 `.at` 一致 |
| Phase 3 第 9 章 golden | `book/rust/listings/ch09/listing-09-02/main.expected.rs` | `book_ch09_02` 通过；golden 与当前实际输出一致（Option<i64> 且无旧版冗余导入） |
| Rust 金样 | 默认不变；仅当转译器实产与金样存在经解释的差异时，更新对应 Listing 的 `main.expected.rs` | 定向对拍及 `cargo tb`；diff 仅限目标文件 |

### 5.2 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| — | none | — | 无语言或转译器规范变化 | Phase 1 修正书稿示例语法，Phase 2 对齐书内已有示例与当前转译产物 | AC-01, AC-02, AC-05 |

## 6. 测试设计

不变式：书稿中的 Auto 模式分支使用 Auto 规定的 `->`，并能由既有转译器产生与金样相符的 Rust；金样生成只可能写入隔离工作树。

实现期在干净的 auto-lang 计划工作树中设置 `AUTO_BOOK_LISTINGS=D:/autostack/.wt/lang-819/book/rust/listings`，分别运行：

- `cargo test -p auto-lang --lib --features test-book -- tests::book_listing_tests::book_ch02_05`，预期 1 passed、0 failed、0 skipped。
- `cargo test -p auto-lang --lib --features test-book -- tests::book_listing_tests::book_ch03_08`，预期 1 passed、0 failed、0 skipped。
- Phase 2：先修正并同步四个 Auto 语法片段，再定向运行 `book_ch06_01/02/03/04/05/06/08/09/10/11` 十个过滤器，预期每项 1 passed / 0 failed / 0 skipped；比较每份 `main.expected.rs` 与测试实际输出，只更新对拍确认不一致的目标。保存 Phase 2 定向证据与主书仓 WIP 的前后哈希。
- Phase 3：先记录最终 `cargo tb` 的四项失败；运行 `book_ch08_01/02/04` 与 `book_ch09_02` 的精确过滤器，修正 ch09 源/正文，并按 `.wrong.rs` 实际产物更新 ch08 三份及 ch09 一份金样，再复跑四项，预期每项 1 passed / 0 failed / 0 skipped。保存主书仓三份重叠 ch08 WIP 哈希及定向证据。

若采用 Cargo nextest 运行器，使用等价的测试名过滤；必须确认测试实际运行而非 dirty-book 守卫 skip。上述命令为开发期定向证据。独立复审通过后，在冻结 code/book revision 上执行 merge-owned `cargo tb`，预期整档成功。核对该档的生成副作用，仅保留本计划允许的 Phase 1 六个文件及 Phase 2 的四份作者源、两个正文文件和经逐项对拍确认必要的金样；不得用 `cargo t`、`cargo tv` 替代书测试门禁，也不改 Rust 实现，因此本计划不要求这两档。

实现者：当前会话；内部核查：同一工作树完成定向对照；最终 review：独立上下文的计划复审流程；merge：按仓库 Plan 归并流程。复审须审计源/正文/金样的闭包、diff 边界及门禁收据。

## 7. 验收标准

| ID | owner_stage | 可观察结果及验证 |
|---|---|---|
| AC-01 | work | 两个目标 `.at` 源以及 ch02 英文/中文对应 Auto 代码块均使用 `->`，保留原示例含义；parser/transpiler/harness 无实现改动。核对目标文件 diff。 |
| AC-02 | work | `book_ch02_05` 与 `book_ch03_08` 在干净书仓工作树中各通过 1/1；转译实际输出与现存或更新后的目标金样字节一致，测试没有因 dirty-book 守卫跳过。 |
| AC-03 | review | 独立复审确认本计划 Phase 1、Phase 2 与 Phase 3 的变更闭包、无遗漏/擅自扩展/未解释 workaround、书稿 EN/ZH 同步与各章输出归因；确认 `cargo tb` 已作为 merge-owned 门禁，并由 AC-06/09 的冻结门禁收据闭环。 |
| AC-04 | work | ch06 listing 05/06/08/09 的四份 Auto 源和英/中文正文中的同源片段均改用 `->`，含义不变；四个定向测试不再因解析错误失败。 |
| AC-05 | work | 十个 Phase 2 Listing 定向测试均通过；只更新实际输出与当前 golden 不一致的 §5.1 目标，其他目标 golden 保持不变。 |
| AC-06 | merge | revision 4 门禁失败已记为历史；revision 5 冻结组合 `cargo tb` 全档已通过。合入本计划允许的书稿源与必要目标金样后，将 P813-D1 标记为由 PLAN-819 承接；PLAN-813 自身的 `cargo tb` 仍须在其最终 code/book 组合上重跑后才能宣称该门禁通过。 |
| AC-07 | work | ch08 三份及 ch09 listing 02 golden、ch09 源/EN/ZH 正文修正只覆盖 G-06 闭包；四份 golden 与实际生成输出逐字节一致；主书仓原 9 项用户 WIP 的哈希与计划实产一致，未引入额外内容，合入前存入可恢复 stash，合入后字节哈希不变。 |
| AC-08 | work | `book_ch08_01/02/04` 与 `book_ch09_02` 在隔离树各 1 passed，无 skip；EN/ZH ch09 Auto block 与 `.at` 一致，Rust `match =>` 保持不变。 |
| AC-09 | merge | revision 5 复审通过后，冻结组合上的 `cargo tb` 全档通过（5312 passed / 1547 skipped / 1 slow，exit 0）；门禁生成副作用已清理且计划书树 clean。书仓合入及归档仍待完成。 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/真实路径或符号 | 结果及验证 |
|---|---|---|---|---|
| T-01 | work | 用户确认本计划（✅ 用户 2026-10-19 指示「计划819: 实施」，合同获授权） | 建 `D:/autostack/.wt/lang-819` sibling group；复核书仓实际基线、主仓 WIP 和 4 个目标源/金样 hash；设置 `AUTO_BOOK_LISTINGS` 指向组内 book 树 | ✅ 干净隔离工作树已建（auto-lang@plan-819-dev=7d4baf004 / book@plan-819-book 基线 d7a71a7f / auto-down detached@c39ac7f）；4 个目标文件 SHA-256 与 §4 记录逐一相符；主书仓 9 个既有 `.expected.rs` 用户修改保持原状未触碰；主 auto-lang 检出 crates/test 零 WIP |
| T-02 | work | T-01 | 修正两份 `main.at`，同步 `rust/ch02-guessing-game.md` 与 `.cn.md` Listing 2-5；用现有两个定向 Listing 测试核对实际输出和金样 | ✅ AC-01/02：book@`abe1e9e` 修正 2 处 + 3 处 `=>`→`->` 并同步 EN/CN 正文（4 文件 9 行）；首跑定向对拍红=两份金样为 d7a71a7 再生成时因源解析失败遗留的陈旧产物（a2r_std 导入/i32/尾分号三处漂移，ch02+ch03 内仅这两个金样仍含 a2r_std，兄弟金样均已为新格式）→ 按计划 §5.1 仅更新这两个金样（book@`fb744f5`，2 文件）；复跑 `tests::book_listing_tests::book_ch02_05` + `book_ch03_08`（`AUTO_BOOK_LISTINGS` 指向组内干净书树）= 2 passed / 0 failed / 0 ignored，无 SKIP 输出；auto-lang 侧零代码改动 |
| T-05 | work | T-02 | 以当前书仓 master (`fb744f5`, 已含 Phase 1) 建立 `D:/autostack/.wt/lang-819/{auto-lang,book}` 隔离 worktree；记录主书仓相关 WIP 哈希；将 ch06 listing 05/06/08/09 的 Auto 分支 `=>` 改为 `->` 并同步 EN/CN 片段；逐项比对十个 `book_ch06_*` 的实际输出，只更新确实不匹配的目标 golden；复跑十个定向测试并确认 diff 闭包 | ✅ Phase 2 分两笔提交：`93bc04a`（4 个 `.at` + EN/CN 同步）和 `8e32e25`（10 个实际不匹配 golden）；ch06 十项各 1/1 通过，另有 Phase 1 的 `book_ch02_05` / `book_ch03_08` 各 1/1 通过，均无 SKIP；改动闭包正好 16 个 Phase 2 文件；主书仓 6 个重叠 golden WIP 均与实际输出同 SHA-256，另外 3 个 ch08 WIP 未触碰。revision 4 复审 pass；其最终 `cargo tb` 新发现的 ch08/ch09 红项已由 Phase 3 接手 |
| T-03 | review | T-05 | `/auto-plan:review` 对 revision 4 的最终冻结 diff 执行复审，审计 AC-01/02/03/04/05、Phase 1+2 文件闭包、用户授权边界及无 Spec 增量理由；复审通过后记录 T-04 门禁待办 | ✅ revision 4 final pass；复审记录见 §9；`cargo tb` 留给 T-04 |
| T-04 | merge | T-03 | 在最终复审通过后冻结 revision 上执行 merge-owned `cargo tb`；通过后核验生成副作用、只合入计划范围文件、更新 P813-D1 承接记录；按 wt-guard 规程处理本计划工作树 | ⛔ revision 4 门禁失败（4 个 ch08/ch09 红项）；最终门禁转为 T-08，历史收据见 §9 |
| T-06 | work | T-04 | 将 revision 4 最终 `cargo tb` 的失败登记为 PLAN-819 Phase 3；在隔离树修复 ch08 listing 01/02/04 金样、ch09 listing 02 Auto `=>` 语法及其旧 Option<i32> golden，并同步英文/中文 Auto block；逐项实测更新四份必要金样/正文 | ✅ Book 提交 `41f5dda`（ch09 源/EN/ZH）+ `d05218b`（四份实产 golden）；四项精确测试各 1 passed / 0 failed / 0 ignored，日志已归档至 `docs/plans/attachments/819/phase3-book_ch08_01-final.log.txt` 等；EN/ZH Auto block 与 `.at` 一致，Rust `match =>` 保留；主仓 ch08 WIP 三份哈希与实产完全相同；Book worktree clean、`.wrong.rs` 清零。T-07 revision 5 复审待办 |
| T-07 | review | T-06 | 对 revision 5 冻结 diff、Phase 3 范围闭包、全部相关定向证据、主仓 WIP 哈希、规范/风险账本进行最终复审 | ✅ 独立只读复审 pass，报告见 §9；AC-06/09 merge-owned `cargo tb` 保持 pending |
| T-08 | merge | T-07 | 在 revision 5 最终复审通过后，对冻结 code/book 组合执行 `cargo tb`；核对生成副作用与主书仓原 9 项 WIP 哈希并保存可恢复副本；通过后执行合入与 P813-D1 承接 | 门禁 ✅：冻结 code `4c8a28b92dab3ef9ed270cf1a25c5414c002343a` + book `d05218bff49a52314ce436f78d9df2a69bc3357e` + auto-down `c39ac7f`；标准 target（未设置 `CARGO_TARGET_DIR`）、`AUTO_BOOK_LISTINGS=D:/autostack/.wt/lang-819/book/rust/listings`、`RUSTC_WRAPPER=''`、`NEXTEST_TEST_THREADS=4` 下 `cargo tb` exit 0，5312 passed / 1547 skipped / 1 slow，234.627s；日志 `docs/plans/attachments/819/cargo-tb-r5-4threads.log.txt`。Book master 已 fast-forward 到 `d05218b`；auto-lang ledger 已落 master `216c9629d`（投影提交重基到并行更新的 master，range-diff 等价）。P813-D1 已记录由本计划修复承接；归档与 worktree 清理待完成。 |

## 9. 复审记录

### 2026-10-19 revision 2 最终复审（historical pass）

- stage: review；plan_id: PLAN-819；plan_revision: 2；review_scope: final；outcome: pass；reviewer: 当前 Codex 会话（复审直接核对提交对象、工作树文件、计划验收项与定向测试证据；会话继承了实施历史，故不声称来自独立会话）；worktree_path: `D:/autostack/.wt/lang-819/{auto-lang,book}`；branch: `plan-819-dev` / `plan-819-book`；reviewed_commit: code `7d4baf004dda7dcb5707af3c3210a0ec52464635`（代码侧无本计划改动）+ book `fb744f52757b6155e8d7707c43323e3ea8ebd88f`；base_commit: book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；dependency_revisions: auto-down detached@`c39ac7f`（只读未用）。
- acceptance: AC-01 pass（两个 `.at` 与 ch02 英文/中文正文均使用 `->`；改动闭包没有 parser/transpiler/harness）；AC-02 pass（`book_ch02_05`、`book_ch03_08` 各 1 passed / 0 failed / 0 ignored，无 dirty-book skip）；AC-03 pass（检查到的差异恰为 6 个计划目标文件，金样差异由定向对拍确认并逐项核对）；AC-04 pending（冻结提交上的 `cargo tb`、P813-D1 承接记录及书仓合并）。
- evidence: book base 到冻结提交的 diff 仅有 §8 所列 6 个文件；`core.whitespace=cr-at-eol` 下 `git diff --check` 通过；冻结树中六个文件的磁盘内容与 HEAD blob 一致，cached diff 为空；定向测试结果见 T-02。Spec 无增量，spec-impact 元数据为空适当。
- limitations: 当前会话无法从 Git 完整读取 Book worktree status/untracked inventory；已核对的六个目标文件与提交树一致。测试冷编译输出 561 条来自未改动 Rust 源文件的既有 warning，本计划未改 Rust 源码，未发现由本变更引入的 warning。未运行 `cargo tb`，按流程留给 merge 阶段。
- findings: 无需修复项；next_responsible_role: merge（先跑 T-04 门禁，门禁通过后才可合并）。

### 2026-10-19 merge gate attempt（blocked）

- stage: merge；plan_id: PLAN-819；plan_revision: 2；outcome: blocked；gate: `cargo tb`；code_commit: `7d4baf004dda7dcb5707af3c3210a0ec52464635`；book_commit: `fb744f52757b6155e8d7707c43323e3ea8ebd88f`；dependency_revisions: auto-down detached@`c39ac7f`；inputs: `AUTO_BOOK_LISTINGS=D:/autostack/.wt/lang-819/book/rust/listings`，独立 target dir `D:/autostack/.wt/lang-819/auto-lang/target/plan819-merge`。
- result: exit 1；nextest 开始 5312 项，执行 1131 项（1122 passed / 9 failed），1547 项由 profile 跳过；失败快停，剩余项未执行。失败名为 `book_ch06_01/02/03/04/05/06/08/09/10`。当时将它们概括为全部缺少 `Hash` derive，已由 r4 的隔离定向复现校正：其中 05/06/08/09 是作者源 `=>` 不符合 Auto `->` 语法；01/02/03/04/10 属 golden mismatch。直接 ch06 过滤还发现 11 号 golden mismatch。主书仓原有 9 个未提交金样 WIP 中，01/02/03/04/10/11 与当前实际输出的 SHA-256 相同；Phase 2 记录这些哈希后仅在计划树据实际产物修复，不覆盖主检出。所有归因与证据见 revision 4 收据。
- post-gate state: PLAN-819 Book worktree `D:/autostack/.wt/lang-819/book` 仍 clean，HEAD 仍为 `fb744f5`；书仓默认检出的 9 个既有修改保持原样。没有执行 merge、归档或 worktree 清理；AC-04 pending，待相关基线红项有独立授权/计划并在最终组合上通过门禁后再继续。

### 2026-10-19 revision 3 范围变更记录

- 用户明确要求把 merge gate 暴露的 ch06 九个 `Hash` derive golden mismatch 作为 PLAN-819 新 Phase 2 解决，不另立计划。计划升至 `plan_revision: 3`、状态 `executing`；保留 r2 的 review 与失败门禁收据作为历史，不再把它们视为当前验收结论。
- 新授权边界：仅可更新 §5.1 所列九个 `main.expected.rs` 并做相应定向验证；不修改 ch06 作者源、transpiler/harness、其余章节或九项范围以外的主书仓 WIP。修复后需重新复审，再在新冻结提交上跑 merge-owned `cargo tb`。
- 当前工作树：T-01/T-02 保持既有通过证据；T-05 为本次新增执行阶段，T-03/T-04 重置为待办。`cargo tb` 的 r2 失败不能当作 r3 的最终门禁结果。

### 2026-10-19 revision 4 归因与恢复记录

- 修订原因：r3 把门禁列出的九项都当作 `Hash` golden 差异；新建干净 worktree 后运行 `cargo test -p auto-lang --lib --features test-book -- tests::book_listing_tests::book_ch06_ --nocapture`，结果为 1 passed（`book_ch06_07`）/10 failed。listing 05/06/08/09 的作者源先因 `=>` 报 `UnexpectedToken { expected: Arrow, found: "=>" }`；修正并提交源+EN/CN 片段后，在干净书树重新运行，十一项测试真实执行，仍为 1 passed/10 failed（无 SKIP），失败项均是 golden mismatch。01/02/03/04/10 的枚举 derive 新增 `Hash`；05/06/08/09 还出现旧标准库导入删除、类型宽度与借用/分支形状变化；11 将 `x.unwrap_or(0)` 改为 `x`。主书仓六份相关 WIP golden 与相应实际输出 SHA-256 相同；源扫描只在 05/06/08/09 发现 Auto `=>`。
- 授权范围：用户已要求将门禁问题并入 PLAN-819 新 phase；本修订将上述同章同类问题一并闭环，不碰其他章节、转译器或 harness。英/中文正文对应的 Auto 片段分别在 `rust/ch06-enums.md` 与 `.cn.md`；同期 Rust `match` 示例保留 `=>`。
- 当前提交状态：书仓 master `fb744f5` 已包含 Phase 1（`abe1e9e`、`fb744f5`）；此前 worktree/分支已不在注册表。Auto-lang master 当前 HEAD 为 `4c8a28b92`，新建 `plan-819-dev` 与 `plan-819-book` worktree 从当前 master 重新开始，auto-down detached `c39ac7f`。书仓主检出的 9 个未提交 golden WIP 保持原样。
- 工作状态：T-05 已完成。Book base=`fb744f52757b6155e8d7707c43323e3ea8ebd88f`（含 Phase 1），branch=`plan-819-book`，HEAD=`8e32e25e404955e5a70c3243faf39656ab4209b1`（栈含 `93bc04a`）；Auto-lang test tree=`plan-819-dev` @`4c8a28b92dab3ef9ed270cf1a25c5414c002343a`（本计划零代码改动），auto-down detached=`c39ac7f`。提交源修复后，ch06 定向测试先真实执行并确认十个 golden mismatch（1 passed/10 failed/0 skipped）；按每份 `.wrong.rs` 的实际输出更新十个 golden 后，十个指定测试各 1 passed/0 failed/0 ignored；原两项 Phase 1 测试也各 1 passed/0 failed/0 ignored。书仓主检出 01/02/03/04/10/11 六个 WIP golden 的哈希分别与计划树输出相同；ch08 的三项 WIP 保持原样。Phase 2 diff 恰 16 个列明文件，`core.whitespace=cr-at-eol` 下 `git diff --check` 通过，临时 `.wrong.rs` 已清除。T-03 revision 4 复审与 T-04 merge-owned `cargo tb` 尚 pending；通过前不合并。

### 2026-10-19 revision 4 最终复审（pass）

- stage: review；review_scope: final；reviewer: 当前 Codex 会话（复审从 Plan、冻结 Git diff、逐项生成产物与测试日志重建结论；上下文继承实施历史，故不声称来自独立会话）；plan_id: PLAN-819；plan_revision: 4；outcome: pass。
- baseline: worktree=`D:/autostack/.wt/lang-819/book`；branch=`plan-819-book`；reviewed_commit/book=`8e32e25e404955e5a70c3243faf39656ab4209b1`；base_commit/book=`fb744f52757b6155e8d7707c43323e3ea8ebd88f`（Phase 1 已在基线）；code_worktree=`D:/autostack/.wt/lang-819/auto-lang`；branch=`plan-819-dev`；reviewed_commit/code=`4c8a28b92dab3ef9ed270cf1a25c5414c002343a`（本计划无 Rust 代码变更）；dependency_revisions: auto-down detached@`c39ac7f`。两树 status clean；书仓 diff 恰 §5.1 列出的 16 个 Phase 2 文件。
- spec_inputs: `docs/specs/auto-lang/trans/overview.md` SHA-256=`124735299C79F0857986309C8091ADADF38F5797E720E43CA01A71628AEB6209`（main 与 review worktree 一致）；规范增量 none，因为改动仅修正书稿作者示例并使书稿金样跟随现有转译器行为。
- acceptance: AC-01 pass（Phase 1 两个 `.at`/正文闭包无 parser/transpiler/harness 改动）；AC-02 pass（`book_ch02_05`、`book_ch03_08` 各 1 passed，无 skip）；AC-03 pass（授权范围、Phase 1/2 16 文件闭包、历史归因校正、Spec 空增量与待门禁均记录）；AC-04 pass（listing 05/06/08/09 的源与 EN/CN Auto block 逐项一致，Rust `match =>` 样例未改）；AC-05 pass（10 个 ch06 定向测试各 1 passed，0 failed/ignored/skipped，逐项输出与 golden 相同）；AC-06 pending（merge-owned `cargo tb` 全档及 P813-D1 merge 承接记录）。
- evidence: 12 个 exact `cargo test -p auto-lang --lib --features test-book -- tests::book_listing_tests::<name> --exact --nocapture` 过滤器日志已保存在 `docs/plans/attachments/819/final-book_*.log.txt`，每项显示 1 passed / 0 failed / 0 ignored，dirty-book SKIP 计数为 0；四个 EN/CN Auto code block 与对应 `.at` 比对一致；九个与主书仓重叠的 user WIP golden 哈希与交付文件逐字节相同；`git -c core.whitespace=cr-at-eol diff --check fb744f5..HEAD` 通过；临时 `.wrong.rs` 已删除。未修改 Rust 源或测试基建；格式检查覆盖提交后的实际差异。
- findings: 无未解决 finding。r2 对九个失败同归因 Hash 的历史记录已由 r4 明确纠正；本轮未发现未授权延期或 workaround。review 内部限制为同一会话复核，按 skill 记录此限制并独立重读证据；没有声称另一会话审查。pending_closeout_gates: 在冻结 code/book revision 上执行 merge-owned `cargo tb`；通过后再执行 Book fast-forward/收据和 P813-D1 承接。next: merge（T-04）。

### 2026-10-19 revision 4 最终门禁结果与 revision 5 Phase 3 授权

- stage: merge gate；plan_id: PLAN-819；reviewed revision: 4；frozen code=`4c8a28b92dab3ef9ed270cf1a25c5414c002343a`；book=`8e32e25e404955e5a70c3243faf39656ab4209b1`；dependency auto-down detached=`c39ac7f`；evidence attachments=`docs/plans/attachments/819/`。
- result: `cargo tb` exit 100；nextest 5312 项中 1150 passed、4 failed、1547 skipped，fail-fast 后其余未执行。失败为 `book_ch08_01/02/04`（当前转译输出分别含 `a2r_std::value_len`、`get(...).cloned().unwrap_or(0)` / `value_len`，旧 golden 仍为旧 Vec/index 输出）及 `book_ch09_02`（`main.at` 的 `is` 分支使用 `=>`，解析器要求 Auto 箭头 `->`）。ch09 EN/ZH 正文含同一错误 Auto 片段。Phase 3 初次真实定向复现中，ch08 三项仍为 golden mismatch；订正 ch09 箭头后其实际输出又证明原 golden 同时滞后于当前转译器（Option<i32> 改为 Option<i64>，冗余标准库导入消失），故将 ch09 golden 一并列入闭包。主书仓 ch08 三份 golden WIP 的 SHA-256 已与实产一致；Phase 3 从隔离树实产更新四份 golden，并保留主书仓原文件和状态。
- side effects: `cargo tb` 在计划书树生成了 ch08 三份临时 expected 写回；已从 `plan-819-book` 隔离 worktree 恢复到提交状态并确认 clean，主书仓未触碰。失败日志：`docs/plans/attachments/819/cargo-tb-final.log.txt`。
- scope decision: 用户明确要求不另立计划，直接在 PLAN-819 记录问题与解决方案作为新 phase；因此升至 revision 5、status executing，新增 Phase 3（ch08 三份 golden + ch09_02 源及 EN/ZH 对应 Auto block）；仍不改 parser/transpiler/harness 或其他 Listing。T-04 记录 r4 红门禁，T-06 执行 Phase 3，T-07 复审，T-08 新冻结 revision 的最终门禁。通过前不合并、不归档。

### 2026-10-19 revision 5 Phase 3 work receipt

- stage: work；plan_id: PLAN-819；plan_revision: 5；scope: `book_ch08_01/02/04` golden + `book_ch09_02` Auto 源/EN/ZH 正文/golden；Book base=`8e32e25e404955e5a70c3243faf39656ab4209b1`；HEAD=`d05218b`（栈含 `41f5dda`）；Auto-lang code tree=`4c8a28b92dab3ef9ed270cf1a25c5414c002343a`；auto-down detached=`c39ac7f`。
- diagnosis: 首次尝试在未提交正文的书树运行 `book_ch09_02` 被 dirty-book 守卫 skip（不计验证）；提交 ch09 源/正文后四个精确测试均真实执行并首先红于 golden mismatch。实际产物：ch08 01 新增 `a2r_std::value_len` 导入/调用；08-02 索引转译为 `get((n) as usize).cloned().unwrap_or(0)`；08-04 新增 `a2r_std::value_len`；09-02 类型从 `Option<i32>` 更新为 `Option<i64>` 并移除旧标准库导入。只将这些实产写入四个目标 golden。
- acceptance: 修正后 `book_ch08_01/02/04`、`book_ch09_02` 分别 1 passed / 0 failed / 0 ignored；日志在 `docs/plans/attachments/819/phase3-book_ch08_01-final.log.txt`、对应 02/04/09_02 文件；日志无 dirty-book SKIP。ch09 EN/ZH Listing 9-2 Auto block 去掉首行注释后逐字匹配 `.at`；相邻 Rust 输出块的 `=>` 仍在。四份 golden 与各测试产物一致，目标源/正文与 golden 共 7 个改动文件仅在计划允许闭包中。
- WIP and hygiene: 主书仓 ch08-01=`5636EAD1E5F1D7D769D87F990969A9EE40FE015A060DDBF93B67186863861E0C`、08-02=`314801C0C715844E2C8F521D797E734887318A8141E66A789AF784E6EC5D6D8F`、08-04=`AEB064509B2FFCC0AB365E3874013CA3123C099AFE15A9AD32208B4F57ADC319`；均逐字节匹配隔离计划实产且主文件未触碰。计划书树 clean，无残留 `main.wrong.rs`；`git -c core.whitespace=cr-at-eol diff --check` 通过。没有 Rust 实现或 Spec 改动。
- readiness: T-06 complete；T-07 revision 5 final review 与 T-08 冻结版 `cargo tb` pending；因完整门禁此前失败，当前不合并、不归档。

### 2026-10-19 revision 5 独立最终复审（pass）

- stage: review；review_scope: final；reviewer: 独立只读复审者 `/root/plan819_review_r5`；plan_id: PLAN-819；plan_revision: 5；outcome: pass。
- baseline: Book base=`fb744f52757b6155e8d7707c43323e3ea8ebd88f`，reviewed HEAD=`d05218bff49a52314ce436f78d9df2a69bc3357e`（含 `41f5dda`）；Book worktree `D:/autostack/.wt/lang-819/book` clean。相对 Phase 2 基线共 23 个文件（Phase 2 16 + Phase 3 7）；相对初始书仓 `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a` 总计 29 个（Phase 1 6 + Phase 2 16 + Phase 3 7），与计划闭包一致；无 parser/transpiler/harness/Spec 修改。
- acceptance: AC-01/02 pass（Phase 1 两项真实定向对拍各 1 pass）；AC-03 pass（范围、无 Spec delta 理由与 merge-owned pending 记录吻合）；AC-04/05 pass（ch06 四个 Auto 源和中英文片段同步、十个目标真实对拍各 1 pass）；AC-07/08 pass（四个 Phase 3 目标真实对拍各 1 pass，0 ignored，日志无 dirty-book SKIP，四份 golden 被实际比较）；AC-06/09 merge-owned pending（revision 5 冻结组合 `cargo tb` 尚未运行，禁止据此宣称交付/合并）。所有定向测试日志路径见 T-02、T-05、T-06。
- WIP/hygiene: 主书仓仍有原先 9 个 modified golden；当前九项 SHA-256 与隔离计划树对应 golden 相同，其中三份 ch08 digest 已列于 Phase 3 收据，六份 ch06 哈希一致性由独立复审逐项核实。文件时间早于本轮 gate/phase3 定向运行；主书仓未被门禁指向。Book diff check 无输出，计划树无 `.wrong.rs`；PLAN819 group 无 reparse point。Auto-lang worktree 的提交与 clean 状态已通过 Git admin entry 核验，但 Windows `git worktree list` 将其 WSL `/mnt/d/...` `.git` pointer 标为 prunable，后续清理须用兼容该登记路径的 Git 上下文。
- findings: 无实施或验收 finding；仅 merge-owned `cargo tb` pending。next_responsible_role: merge（T-08 冻结门禁先行；通过后再合入和归档）。

### 2026-10-19 revision 5 merge-owned `cargo tb`（pass）

- stage: merge gate；plan_id: PLAN-819；plan_revision: 5；reviewed revision: 5；outcome: pass；code_commit=`4c8a28b92dab3ef9ed270cf1a25c5414c002343a`；book_commit=`d05218bff49a52314ce436f78d9df2a69bc3357e`；dependency_revisions: auto-down detached@`c39ac7f`；输入：`AUTO_BOOK_LISTINGS=D:/autostack/.wt/lang-819/book/rust/listings`，`CARGO_TARGET_DIR` 未设置（Cargo 标准 target），`RUSTC_WRAPPER=''`，`NEXTEST_TEST_THREADS=4`。
- 初次错误配置尝试：把 `CARGO_TARGET_DIR` 指向隔离缓存目录时，5 个 FFI fixture 子进程因预期的 `target/release` 可执行文件不存在而失败；这是测试运行器产物路径配置错误，该次不作为门禁结果。随后两个使用标准 target、默认 nextest 并发的完整运行分别被未修改测试的间歇性问题中断：第一次 5311 passed 后，`plan707_wait_generator_park_drive_count_static` 超过 120s；单项隔离重跑 1.046s 通过。第二次在第 1908 项出现 `plan492_m4_tests::m4_pkg_compile_chain::c1_prop_compare_in_init` 断言失败（此前 1907 passed）；单项隔离重跑 0.334s 通过。未修改这些测试或其代码。
- 最终门禁：在同一冻结代码、标准 target 和完整测试选择下仅将 nextest 并发限制为 4，`cargo tb` exit 0，5312 passed、1547 skipped、1 slow，耗时 234.627s；日志 `docs/plans/attachments/819/cargo-tb-r5-4threads.log.txt`。这次完整绿档为 AC-06/09 提供门禁证据；不同配置的失败尝试均保留记录，不据以宣称默认并发全绿。
- 收尾核验（合入前）：Book 计划树仍为 reviewed HEAD 且 clean、无 `.wrong.rs`；主书仓原有 9 个 modified golden 哈希与交付文件一致，门禁未指向主书仓，文件内容未变。最终门禁所需日志均已复制到 `docs/plans/attachments/819/` 并逐文件 SHA-256 核对，工作树删除后证据仍可读。next_responsible_role: merge。

### 2026-10-19 revision 5 merge consolidation（进行中）

- prepared：reviewed 基线为 PLAN-819 revision 5、Book `d05218bff49a52314ce436f78d9df2a69bc3357e`、Auto-lang code `4c8a28b92dab3ef9ed270cf1a25c5414c002343a`、auto-down `c39ac7f`；独立最终复审 pass；冻结 `cargo tb` pass（最终日志 SHA-256=`07107603224097f332c325d7602d46ff54d333f207ee70dd3a946e77c2f156e3`）。
- landed：Book master 从 `fb744f52757b6155e8d7707c43323e3ea8ebd88f` `--ff-only` 到 `d05218bff49a52314ce436f78d9df2a69bc3357e`；交付差异为 23 个书稿文件。主书仓原 9 项 WIP 在合入前逐项哈希与计划输出一致，提交前保存于 Git stash `dd877c9e3fb4cc06aacfec1d0f2c78d5077f5a61`，合入后工作区文件哈希未变。Auto-lang ledger 投影提交原为 `37b5f305fe5bf95fd6c34ba53c5420dccc60ea8f`；并行 master 簿记 `aafdf9bd521a5c9ac6431822179dce27636a9df0` 到达后，重基得到 `216c9629d58cff34bd4e85c6137807ee1b7c503f`，`git range-diff` 显示该单提交补丁完全等价（`=`）；随后 auto-lang master `--ff-only` 到该提交。
- ledger_refreshed：workspace=`D:/autostack/auto-lang`，tracked `.autoos/specs.json`，通道=`guarded offline projection`（无可用 store writer；`GET http://127.0.0.1:8080/api/specs` 连接拒绝）。准确备份=`%TEMP%/auto-lang-plan819-specs-before.json`；version `53→54`；SHA-256 `964087A9E4FE98CEB8E38F74C3DBFDBD192BDF84E64210FCA2869592DAC816D4` → `0e790ecd79706f9b7cf89cae3b56120b1d126826f224ec2d7d14b39f38e60a0a`；新增 `reviews:P819-R1`、`reports:P819-M`，两项均引用归档计划路径并带 plan/reviewed-commit/gate-log hash。六个 section 与其余全部条目、顶层字段逐结构保持相同；既有重复 ID `P014-1` 原样保留，无新的 ID 冲突。无 module contract 变化，故未增补 `designs` 项或 canonical Spec 文件。
- evidence retention：target 中 29 个 Phase 1/2/3 测试及历史门禁日志与 5 个 merge gate / 隔离复跑日志共 34 份（8,754,225 bytes）复制到 `docs/plans/attachments/819/`，每个副本与源文件 SHA-256 相同；目录索引见 `docs/plans/attachments/819/README.md`。archive 与 worktree cleanup checkpoints 仍 pending。

### 2026-10-19 work 收口（execution_done）

- stage: work；plan_id: PLAN-819；plan_revision: 1；outcome: pass；worktree_path: `D:/autostack/.wt/lang-819/{auto-lang,book,auto-down}`；branch: `plan-819-dev`（auto-lang，零提交=基线 7d4baf004）/ `plan-819-book`；base_commit: book `d7a71a7fb1fa42ddd26d6cec859715ed98161f7a`；code_commit: book `fb744f5`（栈：abe1e9e 源/正文修正 + fb744f5 两目标金样再生）；dependency_revisions: auto-down detached@c39ac7f（只读未用）；task_ids: T-01, T-02；evidence: 见 §8 行内收据；blockers: 无；next_responsible_role: review（T-03）。
- readiness：work 侧 AC-01/02 全绿（2 passed/0 failed/0 ignored，非守卫 skip）；在约 findings 与覆盖缺口=空；金样变更经归因（陈旧产物，仅 2 目标文件）；review/merge 侧门禁（`cargo tb` 冻结档、P813-D1 承接标注）保持 pending，归 T-03/T-04。
- 授权记录：用户 2026-10-19「计划819: 实施」指示视为对 drafting 合同的执行授权。

### 2026-10-19 Plan 草案交接

- stage: new；plan_id: PLAN-819；plan_revision: 1；outcome: pass（合同范围明确，可供用户确认）；next: work（用户确认后）；changed task/acceptance IDs: T-01..04 / AC-01..04。
- 依据：用户选择维持 PLAN-813 原范围并另立修复计划；已绑定 PLAN-813 §9 的 `book_ch02_05` / `book_ch03_08` 门禁证据和当前源码、harness、Spec 指纹。

## 10. 待澄清事项

无待澄清项。Phase 3 实施、revision 5 独立最终复审、冻结组合 `cargo tb`、Book fast-forward 与账本刷新已通过；计划已交付归档，worktree 清理仍待完成并记录最终收据。

## Spec-sync 回写记录

- 规范增量为 none：本计划修正书稿 Auto 示例和实产 golden，不改变 Auto 语法、transpiler 接口、测试合同或运行时行为；无对应的 auto-lang module contract 需回写，也不新增人工 `plans.md` 索引行。
- `python scripts/spec-index.py` 已运行，生成 47 个 projects；`docs/specs/INDEX.md` 内容无变化。输出的 3 个既有 ungrouped project 警告（auto-man-core、auto-ui-dialect、repository-boundaries）与本计划无关。
- 兼容 ledger 走 guarded offline projection（本地 store GET 连接拒绝）：`.autoos/specs.json` version 53→54，新增 `P819-R1`（reviews）与 `P819-M`（reports），读回和零扰动结构校验通过。提交 `216c9629d58cff34bd4e85c6137807ee1b7c503f`；projection 前置备份与哈希、writer channel、稳定条目 ID 见 §9。
