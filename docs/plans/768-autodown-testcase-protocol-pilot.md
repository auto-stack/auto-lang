---
plan_id: PLAN-768
status: execution_done
feature_name: AutoDown TestCase 协议与只读试点
plan_revision: 1
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12

supersedes_spec_components:
  - docs/specs/auto-lang/trans/design/test-convention.md
  - docs/specs/auto-lang/vm/overview.md
new_spec_components:
  - docs/specs/auto-test-case/project.md
  - docs/specs/auto-test-case/profile-v1.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/plans.md
touched_goals: [GOAL-016]
affects: [auto-test-case, auto-lang/vm, auto-lang/trans]
current_step: 8
total_steps: 10
---

# [PLAN-768] AutoDown TestCase 协议与只读试点

## 0. 变更摘要

实施 [Design 35](../design/35-autodown-test-cases.md) 的 A 阶段：真实 AutoDown 块树
定位静态 Atom 合同，原始 byte range/sidecar 提供载荷；建立严格发现/校验、AVM run
与 a2r emit 独立检查、只读预期、逐 check 报告。选 20–30 个现有逻辑用例作引用式
试点，登记原入口、比较器、门禁与弱 oracle，保留旧执行路径。

当前仅交付执行合同草案；尚未修改 runner 或执行全部测试。Plan 767 的 AC 标量
能力、工作树和验收独立推进；本计划无其完成依赖。

## 1. 目标

1. 一份 source 工程可登记多个独立后端检查，身份、输入、期望和实际产物可追踪。
2. 坏合同、零发现、缺 expected、required 缺口不能算通过；独立检查继续收集失败。
3. 原始 bytes 保真，普通运行只读预期，失败产物与候选基准写隔离目录。
4. 从旧入口/旧断言到新 case/check 逐项对账，保留比较精度并显式登记诊断强化。
5. 协议核心独立于 AutoLang/图形运行时，不让被测 a2r 在运行时再生裁判。

范围：仅 auto-lang 仓；auto-down 为只读来源。新增协议核心、适配层与开发入口。
非目标：Rust typecheck/build-run、AC/ACC/native、其他后端/AST/HIR、cache、任意
shell recipe、通用进程隔离、网络/FFI/自举重型执行、完整迁移、编辑矩阵/点击运行、
`$case/$expect` 组件、正式 `auto test cases` CLI 接线。它们属于 Design 35 B/C。
不退役旧 helper，不重写全库金样，不改分档，也不以 4,489 个资产声称执行覆盖。

## 2. 架构方案

```text
.case.ad/.suite.ad 原始 bytes
 → pinned headless AutoDown block tree + 严格 raw fence locator
 → AtomParser → schema/ref/suite defaults/check DAG → CaseIR
 → AVM run / a2r emit adapters → StageObservation（完成状态与 actual）
 → Compare（独立预期）→ 统一 Report 的 JSON/console
```

- 新 `crates/auto-test-case/`：profile、范围映射、IR、选择、比较、报告；只依赖
  auto-atom/auto-val 与必要轻依赖，不依赖 auto-lang/auto-man/ui-iced。
- 新 `crates/auto-lang/src/test_cases/`：生产 API adapters、工程 materializer、
  legacy importer、执行编排。依赖方向只有 auto-lang → auto-test-case。
- 新 `crates/auto-lang/src/bin/auto-case.rs`：开发入口，新增空 feature `test-cases`
  门控；无默认 feature 可无窗口运行。正式 CLI 留 B，同核心接口供后续消费。
- block tree 负责标题层级/anchor/合同语义；raw locator 只提供字节区间与 fence
  闭合证据。无法一一映射报 ERROR，不另造 Markdown case 语义，不取 serializer 载荷。
- 拟在本仓 `vendor/autodown-test-parser/` 固定必要 headless Rust 模块快照，保留
  许可和来源清单，不手改 a2r 生成物、不在 build/run 再生、不依赖浮动主检出。
  T-01 核实闭包/许可/来源；不可行则记录证据并修订合同后再推进，不能替换成 Adoc。

## 3. 技术栈

Rust workspace；`auto-atom::AtomParser`；固定 headless `autodown-core` 的
`markdown_parser::parse_blocks`/block_model/ial；现有 AVM path-aware 执行 API、
a2r source_dir/project 发射 API；临时目录设施、JSON、Rust harness/nextest。

## 4. 需求分析与背景调查

### 4.1 授权与工作树边界

用户交付 Design 35/盘点结论，建议独立推进第一计划；当前授权按创建可评审草案
处理。**尚无本 revision 的实现确认**，没有预算/模型/自动续跑约束或跨仓修改授权。
依仓库 L1 规则，完成合同并呈现后等待确认，不把建议当作已批准实施。

实际主检出 `D:/autostack/auto-lang`，分支 **v0.6-dev**（旧规约称 master），
HEAD `897eb34911a4f496a28ba7cdfd8a82815fa043d1`。保持此主检出作计划簿记源，
不切分支或整理其他会话 WIP。active/archive 最大 ID 767，`.next-id` 768；
使用排他分配锁调用 `scripts/new-plan.sh` 取 768，next 为 769。

实施拟用唯一工作树 `D:/autostack/.wt/lang-768/auto-lang`、分支 `plan-768-dev`，
尚未创建。实现/独立复审在同一树顺序进行；禁止 junction/symlink。767 为 executing
r2，在 lang-767；本计划不改 experimental/ac-core，不引用其未合入能力为 implemented。

### 4.2 Spec 与代码依据

| 源 | 当前事实 / 处理 |
|---|---|
| `docs/specs/overview.md`、`README.md` | VM/trans/门禁保留；README 旧 tf per-plan 表述采用最新 AGENTS 的 t + 触面档裁定 |
| `docs/specs/auto-lang/trans/design/test-convention.md` | 金样按 bytes；旧 `.wrong.*` 写原目录，新入口隔离产物并在 Spec 写清双轨 |
| `docs/specs/auto-lang/vm/overview.md`、`auto-atom/project.md` | VM 文件语料/静态 Atom 已有，无 TestCase profile 的 current-state Spec，需 SD-01/02 |
| `src/test_runner.rs:discover_vm_tests/discover_a2r_tests` | 固定目录/stem、缺 expected 跳过；新发现不能复用静默过滤 |
| `src/tests/vm_file_tests.rs:test_vm` | `.as` 显式 lower；result/stdout 精确比较；expected.error 仅存在后判断 Err，须 weak 标记 |
| `src/tests/a2r_tests.rs:test_a2r_with_base` | source_dir/module_name 影响产物，Sink::done 取 bytes；部分 compile smoke 编译 expected，不可当 actual 编译证据 |
| `src/lib.rs:run_with_capture_and_path` | 能带源路径；外观只返通用错误，准确阶段需 T-01 调查/适配，不从 message 猜测 |
| `src/trans/rust.rs:transpile_rust_with_source_dir/transpile_rust_project` | 单/多文件发射入口已有；保留 entry/module/source_dir，不能隐式改源码 |
| 外仓 engine/rust | Cargo 无依赖；parser 规范化换行、BlockNode 无可靠 raw span；复用树语义并补原字节通道 |

盘点是静态资产分析，重复 bytes 不证明相同 entry/module/deps。外仓 Git revision
本次未取得，仅有文件 hash；T-01 固定快照前补来源追踪，不称已锁外仓版本。

### 4.3 内容指纹（SHA-256，2026-10-12）

| 源 | SHA-256 |
|---|---|
| Design 35 | `EB7BD735854FD270AA39DAB5C40432EA291DEFD66D7235857D3B9744EEADD34C` |
| 盘点 README | `F99BA95633028F4F723D4912E6F458067D1858D796795EFAB5864D5534509EFB` |
| trans test-convention | `B3D4A165796AC98DF29110E1EB8D45D04817F011051BFCA220BBAAEE64FEC7B7` |
| 外仓 engine/rust/src/markdown_parser.rs | `4C41853517D09F4740ED32F3A6645F3FEDD8BD7AD69AC0FD3ECE16F21427FFC9` |
| 外仓 engine/rust/src/block_model.rs | `8E99920FF0F078706EA7D8E8B2F351E3FE90C5220D57C791D6CA0E1000B04C36` |

## 5. 详细设计

### 5.1 首期能力表与静态合同

采用 Design 35 `autolang.case/1`/`autolang.suite/1` 版本域并发布支持表：
id/tags/links、source/files、entry、checks/options/expect/tier/platform/required policy/
provenance；suite include/exclude、显式 defaults。至少支持试点 expression/script/module；
program/main 在 T-01 核实生产 API 后登记，不能隐式补调用。`.at/.as` 分开 syntax，
`.as` lower 的实际提交 bytes 另记 hash。仅运行 `avm/run`、`a2r/emit`。

其余字段/动作（variants、relations、fixture driver、recipe、native 等）明确
unsupported ERROR，不解析后忽略。通用与 adapter 严格 schema；Atom 重复键须在
丢失重复信息前检测；合同不触发 Auto 求值。product_from 校验引用/DAG，本期无
编译消费者，不让不可执行的下游 PASS；将来 B 扩展能力表。

source 为 block/file/text 恰一，syntax/虚拟相对 path 必填；source/files 的虚拟路径
唯一。case ID 在选择集合唯一、check ID case 内唯一、anchor 文件内唯一。
载荷 ref 的标题小节到下一同级/上级标题前恰有一个直接 fence；源码内标题/分隔符
不参与结构。内联载荷取 closing fence 前真实 bytes，保留末尾换行；精确无末尾换行
使用 text/sidecar。非 UTF-8 oracle 支持 bytes，非 UTF-8 source 送 UTF-8 adapter 前 ERROR。

sidecar、suite roots、虚拟项目、output 为不同路径域，canonical 校验逃逸与 reparse。
导入用显式 fixture root 的相对引用，不猜 cwd。defaults 对象递归覆盖、数组整体替换，
最终必需字段仍校验；inspect 展示字段来源/suite hash。每次 suite 上下文绑定独立
execution key。空发现、读失败、未知/重复字段、未闭合 fence、坏 selector 均 ERROR。

### 5.2 独立观测和比较器

执行完成状态与断言判定分开。AVM 返回 legacy-display result/stdout、明确可识别
阶段的 diagnostics；a2r 返回 emit observations/actual 文件 manifest。强错误 oracle
必须 stage + code 或有限 message 辨识条件；缺阶段证据不能降成 any-error。
T-01 查错误类型/生产边界；必要时 T-04 增内部带阶段结果通道，旧 API 保持兼容。
panic、I/O、工具缺失、unsupported、timeout/resource 都是基础设施 ERROR，不满足语言错误。

默认 bytes；显式 lf 只 CRLF→LF；legacy-display 不 trim；未声明 stdout 与空 stdout
代表不同覆盖。有限 message 可 exact/显式 substring，本期无任意 regex/全局规范化。
legacy-any-error 仅导入已存在弱断言，携旧入口/债务并报告 weak，required 强验收
不把它算作阶段已证；至少新增一条独立审查的强诊断对照，不直接升级旧 error 文本。

多产物比较路径集合/每项 bytes，缺失/额外文件 FAIL。某 check 失败不影响独立 check；
emit 完成但 snapshot FAIL 仍保留 actual 供消费者，未产生可用产物才 BLOCKED。
用内部注册测试消费者验证交接，不能宣称 Rust 已编译；不得从 expected 取 actual。

### 5.3 只读、资源和报告

每 run/case/check 唯一 scratch，保留相对文件/依赖；不改全局 cwd/env。失败 actual/
diff/report 写指定输出，output 不得与输入/oracle root 重叠；不写旧目录 `.wrong.*`。
proposal 仅生成候选/diff、旧新 hash、共享消费者列表，本期无 accept；不批量更新 oracle。

执行支持限于可信、有限、无外部进程/网络的 pilot，不承诺通用恶意程序 containment。
未实现的 budget/fixture/env/resource 要求报 unsupported ERROR。协议 admission 有
显式文件/载荷 bytes 与合同展开数量限制，T-01 在依赖实现前冻结默认值、单位、边界
与两侧控制；超限为 resource ERROR，不能伪装语法错误或因 crash 临时削掉合法语法。
test harness 用既有 nextest 超时保护，不把它宣传为 standalone 用户程序硬截止；B
才交付通用 deadline/进程树隔离。无限执行/外部副作用不进入本期 pilot。

`autolang.test-report/1` 保存 case/check/execution key、declared/selected/required、
PASS/FAIL/ERROR/BLOCKED/SKIP/NOT_APPLICABLE/UNDECLARED/NOT_RUN。XFAIL/XPASS 可后续扩展，
本期合同显式 unsupported。总体 exit 0=所选 gate 完整满足，1=断言/required coverage
不足，2=合同/基础设施错误（并存优先 2）。selector 不能擦掉 required；个人局部
运行 report partial，不当原 gate pass。未选择项不伪装已执行。
report 保存原/提交 source、suite、oracle、parser/compiler/config 身份/hash、实际
产物和 diff；读取旧 report 对当前身份检查 stale，不能沿用旧 PASS。console/JSON
来自同一判定，不用聚合 suite PASS 遮蔽零动态检查。

### 5.4 引用式试点

目标 24 个现有逻辑用例，T-06 可有据调整至 20–30；新增协议负例不计迁移数。
新 `crates/auto-lang/test/cases/pilot/legacy-map.tsv` 记录旧 source/oracle/hash、
入口/selector/ignore、entry/module/source_dir、比较器/旧 gate、新 case/check、
helper/未挂接/weak/已发现分类。先 file refs，不移动源/金样、不删 wrapper。

已核实存在的候选（以下相对 `crates/auto-lang/test/`，不是已执行结果）：

| 类别 | 真实候选 | 要求 |
|---|---|---|
| VM 基础 | `vm/01_basics/{001_hello,002_arithmetic,003_str_upper,004_uint,005_byte,006_unary,007_group,008_comp,011_eq_false}` | result/stdout/布尔/空输出区别，保留 bytes |
| VM 短路/错误 | `vm/99_short_circuit/{001_and_or_short_circuit,003_negative_index_guard}`、`vm/15_nested_mutation/014_out_of_bounds_error` | weak 债务和独立强诊断对照 |
| `.as`/use | `vm/99_script_err/02_catch_value`、`vm/17_modules/001_use_fn` | T-06 核查依赖，只选无外部副作用/网络的有限语料 |
| a2r 基础 | `a2r/01_basics/{001_hello,003_func,004_doc_comments}`、`a2r/02_types/001_struct`、`a2r/03_control_flow/005_for_range`、`a2r/05_expressions/001_arithmetic` | 保持 module_name/原金样 |
| a2r manifest/项目 | `a2r/27_c_abi/{003_use_c_static,004_use_c_dynamic}` | 只发射不加载库，JSON 依赖登记；T-06 另选真实多模块项目验证多产物 |

至少 3 个 case 同一 source 同时登记 AVM/a2r。若需补另一后端 oracle，独立审查
产物/语义并记录新增断言；不从 VM result 推导其他后端结果。候选不可运行可换同类
现有语料并说明；多文件/多产物/weak 三类不得全替换掉。hash 相同只提示复用，
entry/module/deps 未确认不能合并 check 身份。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-test-case/project.md`、`profile-v1.md` | 无共享合同 → 本期支持表/schema/bytes/ref/suite/DAG/只读/report | 可复用 current-state | AC-01–05、AC-07 |
| SD-02 | add | `docs/specs/auto-test-case/architecture.md`、`plans.md` | 无裁判来源规范 → pinned 来源/raw locator/adapter 依赖方向与执行限制 | 避免 GUI/自举环 | AC-02/08 |
| SD-03 | modify | `docs/specs/auto-lang/trans/design/test-convention.md` | 仅旧目录 → 双轨只读 pilot、emit 与未来 compile 分开 | 不虚报编译覆盖 | AC-03–06 |
| SD-04 | modify | `docs/specs/auto-lang/vm/overview.md` | 仅旧文件语料 → pilot/阶段/weak/旧 gate 边界 | 对账覆盖 | AC-04/06 |

本次仅提议 delta；review 按最终提交冻结，merge 才写 canonical Spec/ADR/模块 plans、
全局 overview 新 crate 与 GOAL-016 进展，再生 spec 索引。

## 6. 测试设计

### 6.1 必须区分错误实现的控制（owner_stage: work）

下表为拟新增协议 tests、test_cases 模块测试和 pilot harness 的控制，T-02–07
使其可执行；草案阶段未运行。每项必须有 case ID、命令、观察值与当前提交证据。

| Case ID | AC/不变量 | 输入/变异 | 验证入口（拟） | 可观察 expected | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 | valid 多 case；duplicate ID/anchor/key、拼错字段、empty suite、缺 source/expected、读失败、坏平台/adapter、悬空 ref、DAG cycle | protocol lint + auto-case lint/list | valid 身份稳定；每种错误定位 ERROR/nonzero，零 check 不过 | 本期支持表 |
| V-02 | AC-02 | Unicode/CRLF/孤 CR/尾空格/空 fence/四反引号内三反引号/源内标题/未闭合 fence | raw locator + block tree + inspect | 与独立构造 bytes/hash 相等；坏闭合拒绝；inline/sidecar 同 bytes | 不用 serializer 作 oracle |
| V-03 | AC-03 | 分别只改 AVM/Rust expected，共享 source 不变 | 两个真实 adapters + CaseIR | 只相应断言 FAIL，独立 check 继续；actual/hash 来源准确 | 不代表 compile 正确 |
| V-04 | AC-03/05 | emit 成功而 snapshot 错、emit 失败、好 expected+坏 actual；消费者记录 bytes | a2r + registered test consumer | 前者消费 actual 且保留 FAIL，后者 BLOCKED，不消费 expected；manifest 含全部文件 | 真实 compile 留 B |
| V-05 | AC-04 | 正确语言错误换 stage/code/message，注入 panic/I/O/timeout/resource，一个旧 any-error | 强诊断 adapter + compare/report | 只有匹配强负例 PASS；错阶段 FAIL，基础设施 ERROR；legacy 标 weak | 阶段来自生产边界 |
| V-06 | AC-05 | success/failure/proposal，空流/无末尾 LF、只读 fixture、输出覆盖输入、path/reparse 逃逸 | materializer + run/proposal | 全部 source/oracle hash 不变；候选仅提议目录；逃逸/重叠 ERROR | 无外部副作用 fixture |
| V-07 | AC-06 | 打乱发现、漏旧输入/expected/ignore、深层 fixture、全量/局部 selector | importer/list/legacy-map | 顺序不改集合，未解释遗漏失败，旧新断言逐项对应 | 20–30，非全仓覆盖 |
| V-08 | AC-07 | required 缺失/被选掉/SKIP/ERROR/BLOCKED，suite defaults、改 source/oracle/compiler | gate reducer + report/inspect | required 不完整非 0；对象覆盖数组替换；身份变化 stale | XFAIL unsupported |
| V-09 | AC-02/08 | parser 来源变动、no-default build、两 run 同 basename、资源限额两侧 | manifest/tree/两次运行/protocol controls | 来源错拒绝，无 GUI，scratch 不撞；超限 resource ERROR | T-01 冻结资源表 |
| V-10 | AC-04/06 | use/manifest、多目录同名多模块、.as、entry/module 变化 | 真实 AVM/a2r 项目路径+旧新对照 | 目录/提交 bytes/产物对应；全局 cwd/env 不变，hash 区分上下文 | pilot 依赖闭包 |

### 6.2 门禁与角色

所有构建/测试在 lang-768 worktree。开发用 `cargo check -p auto-lang`、
`cargo test -p auto-test-case`、`cargo t test_cases`、`cargo tt test_cases`。
旧新对照用只读生产 API，避免旧失败 helper 写原目录。
最终 review：`cargo fmt --all -- --check`、`cargo check -p auto-lang`、
`cargo test -p auto-test-case`、裸 `cargo t`、`cargo tv`、`cargo tt`；另外明确调用
新增 AVM/a2r pilot 聚合入口、standalone no-default pilot 与 V-01–10。
`cargo tree -p auto-test-case` 验证无 auto-lang/图形依赖。

本期不触 ui_gen/book/文档生成器/schema/语法参考/aavm，不额外跑 tu/tb/taa/docs_gen；
裸 t 既有 docs_gen 集成保留。最终触面改变按 AGENTS 追加档。不跑 per-plan tf；merge
执行批量到期判定。草案阶段仅文档簿记，不运行 cargo t/docs_gen。

T-07 提供以下确定命令（**拟议接口**）：

```text
cargo run -p auto-lang --no-default-features --features test-cases --bin auto-case -- lint --suite crates/auto-lang/test/cases/pilot/pilot.suite.ad
cargo run -p auto-lang --no-default-features --features test-cases --bin auto-case -- list --suite crates/auto-lang/test/cases/pilot/pilot.suite.ad --format json
cargo run -p auto-lang --no-default-features --features test-cases --bin auto-case -- run --suite crates/auto-lang/test/cases/pilot/pilot.suite.ad --out <scratch-report-dir>
```

证据写 `docs/plans/attachments/768/`：可行性/来源/support/limits、迁移映射、真实
逐 check 报告、前后 hash、成本（构建/执行分开）、独立复审，绑定 source/dependency/
config/revision/实现 commit。user 未指定模型/代理；不另建 chat/agent。
work 完成实现与内部检查；review 独立上下文在同一树重新验收；merge 仅 reviewed 后
沉淀/合入/归档/清理。实现者自检不能代替独立复审。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 | 验证与 expected |
|---|---|---|---|
| AC-01 | work | 严格合同/完整发现/身份/ref/defaults/DAG，无静默忽略 | V-01，valid 稳定；坏合同逐项定位 ERROR |
| AC-02 | work | bytes/fence/映射可靠，裁判固定来源，无 serializer 取载荷 | V-02/09，独立 bytes/hash 相等，坏闭合/来源拒绝 |
| AC-03 | work | 共享 source 的 AVM/a2r 独立判定，actual/expected 分离 | ≥3 双 check pilot，V-03/04，actual 交接不受金样 FAIL 阻断 |
| AC-04 | work | 强阶段/诊断正确，基础设施不满足 expected error | V-05/10，weak 单列，错阶段 FAIL/基础设施 ERROR |
| AC-05 | work | run/proposal 只读，scratch/路径/产物隔离 | V-06，source/oracle 前后全 hash 一致，无越域写入 |
| AC-06 | work | 20–30 现有 case 完整对账 entry/断言/比较器/门禁 | V-07/10，无未解释遗漏，项目/多产物/weak 必含，强化具名 |
| AC-07 | work | required/selector/status/report 可信，partial 不当 gate pass | V-08，required 缺口非 0，改输入/预期显示 stale |
| AC-08 | work | 轻核心/无 GUI/裁判再生/逐 case 重编，旧 filter 可触达 pilot | V-09，no-default pilot、tv/tt listing、成本和依赖报告 |
| AC-09 | review | 独立 AC-01–08/SD-01–04 核查，遗漏/workaround/警告/门禁 | 全部当前证据，提交/revision 绑定 pass；合同红项不能债务化掩盖 |
| AC-10 | merge | 合入/Spec/ADR/账本/索引/归档/guarded cleanup 实际完成 | clean guard 后移树/分支/空组，终态/链接/计数收据 |

Design 35 §14 actual compile/run 条目归 B；本期 V-04 仅验证产物接口，不声称
真实编译验收，能力表须清楚显示缺口。

## 8. 执行步骤

| ID | owner_stage | 依赖 | 文件/符号与成果 | 验证 / AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户确认 r1 | 在指定树调查 parser 许可/来源/模块闭包、AVM 真阶段/entry/a2r 项目 API；新 attachments/768/feasibility.md 冻结来源、支持/资源表、试点约束 | 有限 parse→bytes→真 adapter whole-path 探针、V-09 边界定案；AC-02/04/08。关键不可行先修订合同 | [x] 2026-10-12 work：探针 [probe-t01.rs](attachments/768/probe-t01.rs)/[output](attachments/768/probe-t01-output.txt)（22 pass；fence 树↔raw 一一对应、AVM/a2r 与旧金样逐字节一致、program entry result="14"）；[feasibility.md](attachments/768/feasibility.md) 冻结 vendor 闭包/许可缺口/阶段分类/资源表/支持表/试点裁定。附带发现并修复：基线 `--no-default-features` 编译失败（frame_bench 4×E0433）→ 提交 2291cd78e；worktree 跨仓 path 解析需组内 auto-down 兄弟（pinned fba6563ed 分离 worktree，预存问题亦影响 lang-767） |
| T-02 | work | T-01 | 新 vendor/autodown-test-parser/ pinned snapshot；新 crates/auto-test-case/{Cargo.toml,src/lib.rs,src/document.rs,src/raw_ranges.rs}；根 Cargo 成员；tree/range、path/resource admission | cargo test -p auto-test-case，V-02/06/09、cargo tree；AC-02/05/08 | [x] 2026-10-12 work：commit 8feeb9678（vendor 三文件指纹=feasibility §1 + 守卫测试防漂移；CaseDocument 树↔raw 逐 fence 对应/未闭合 ERROR/anchor 小节恰一 fence；raw locator 复刻上游 fence 规则；admission 冻结默认/硬顶+虚拟路径/reparse 拒绝）。`cargo test -p auto-test-case` 16 pass；`cargo tree`：运行时依赖仅 vendor 解析器（auto-atom/auto-val 仅 dev） |
| T-03 | work | T-02 | 新协议 src/{profile,ir,discovery,suite,compare,report}.rs（可等价合并职责）；严格 schema/ref/defaults/DAG/comparator/report | protocol V-01/04/05/07/08；AC-01/03/04/07 | [x] 2026-10-12 work：commit 上接 8feeb9678（atom/contract/ir/suite/discovery/compare/report 七模块 + parity/decode/discovery 集成测试）。`cargo test -p auto-test-case` 57 pass 0 warn：未知字段定位错误、avm/run+a2r/emit 封闭矩阵、弱 oracle（裸 stage）拒绝、product_from 引用+环校验、defaults 深合并（对象递归/数组整体替换）、逐文件发现清单含错误保留、零发现硬错、退出码 2>1>0 与 NOT_RUN 破绿 |
| T-04 | work | T-03 | 新 src/test_cases/{mod,avm}.rs，lib.rs 挂接/Cargo 依赖与空 feature；AVM stage observations、明确 entry、.as lower、result/stdout | check + scoped t test_cases，V-03/05/10；AC-03/04/08 | [x] 2026-10-12 work：commit 1b13aa323（runner 编排 + avm 适配器走生产 path API；变体法阶段分类/MultipleErrors 递归/miette code；panic=基础设施 ERROR；.as lowered hash 证据；skip/coverage-gap→exit 2）。`cargo test -p auto-lang --lib --features test-cases test_cases` 8 pass（正/负/独立/错阶段/.as/skip/gap）；`--no-default-features --features test-cases` 编译通过且依赖树无图形件。注：基线 examples 非 fmt-clean（预存，复审用档照实记录） |
| T-05 | work | T-04 | 新 test_cases/{a2r,materialize}.rs，source_dir/project 真产物、manifest/scratch/proposal-only | scoped adapters V-03/04/06/10、前后 hash；AC-03/05/08 | [x] 2026-10-12 work：commit 7df514aa2（128MB 大栈发射；use.c builtin/盘上双路预检否则合同 ERROR；逐项 bytes+多出产物拒绝；product registry→上游未过=BLOCKED；proposal 只写提议目录+PROPOSALS.md）。测试 13/13：legacy 金样逐字节对拍、project 多产物≡同布局直接调用（发现 Cargo.toml app 名派生自父目录名→feasibility §4）、BLOCKED 链、缺 manifest、proposal 前后 hash 不变 |
| T-06 | work | T-05 | 新 test/cases/pilot/{*.case.ad,pilot.suite.ad,legacy-map.tsv,README.md} 和 importer；20–30 引用导入、≥3 双 check，未发现/weak/ignore 对账 | 真实旧新报告、V-07/10，新协议负例不计导入；AC-01/03/04/06/07 | [x] 2026-10-12 work：commit 85710109d。21 case/24 check（fixture_root 原位引用，零搬动）；3 双 check 同源独立 oracle；OOB 强化（map 记录）；json_is_null 旧 oracle 陈值修正（wrapper 从未验证）；8 个新 oracle 经 proposal 流生成+逐项人工审查；legacy-map 12 列对账=报告集合精确相等（测试锁定）。旧新一致证据：a2r wrappers 418 pass（2 预存红与本试点无关）、短路 wrappers 3 pass、新 runner 24/24。裁定修正：vm 01_basics/15_nested wrappers 均 #[ignore]]，活跃 legacy 门禁=发现式 corpus runner（map legacy_state 如实） |
| T-07 | work | T-06 | 新 src/bin/auto-case.rs lint/list/inspect/run/proposal；在 src/tests/{vm_file_tests,a2r_tests}.rs 加 pilot 聚合入口并固定名称，报告 harness→动态 checks | 开发命令、tv/tt listing、no-default pilot、JSON/console 一致；AC-01/05/06/07/08 | [x] 2026-10-12 work：commit 8a9f305cb（CLI 五子命令；selector 未选项记 NOT_RUN 不冒充全量 gate；t/tv/tt 别名接入 test-cases——旧 filter 直接触达 pilot 聚合入口；no-default CLI 全链 24/24 exit 0=§6.2 命令原样）。附带修复预存 VM 缺陷：register_shim_by_name 对动态段 id（next_id 被 register_with_id 顶过 10000）走 dynamic 表——新鲜二进制首跑 VM 原必炸（探针 registry=239/next_id=10015 实证）。日常档 no-fail-fast：仅四在案红+KNOWN-DEBT 396（Plan 744 dir_names），零新增红；cargo tv cookbook 红经 stash 基线对照为预存 |
| T-08 | work | T-07 | attachments/768 完整 V-01–10/成本/只读 hash/拟议 Spec delta，内部合同预检 | scoped 控制 + §6门禁；AC-01–08。全部 work-owned 完成才 execution_done | [x] 2026-10-12 work：[evidence.md](attachments/768/evidence.md)（V-01–10 全映射到可执行测试/命令且全过；成本构建/执行分列；只读树哈希前后一致 4ce1f974…；t/tv/tt/协议/无 GUI 档结果与预存红归因；SD-01–04 拟议内容；AC-01–08 自检表）+ pilot-run-console.txt/pilot-run-report.json。work-owned 8/10 完成 → execution_done |
| T-09 | review | T-08 | auto-plan:review 独立上下文同树核查 diff/控制/AC/SD，扫描遗漏/延期/workaround/KNOWN-DEBT；固定 delta/revision/commit | §6最终门禁 + 独立反例，AC-09；needs_fix 回同树 | [ ] |
| T-10 | merge | T-09 pass | auto-plan:merge 主检出合入、SD/module/global/账本/spec-index；归档 docs/plans/archive；guard→移树/分支/空组，批量到期判定 | AC-10；先 bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-768/auto-lang，必须 clean；真实终态收据 | [ ] |

total_steps=10；work 完成 8/10、review 9/10、实际 merge/清理后 10/10。
current_step 只计真实完成，未来阶段不能先勾。

## 9. 复审记录

### 2026-10-12 — work 完成 / execution_done 交接

- stage: work | PLAN-768 | plan_revision: 1 | outcome: **pass（work-owned 全部完成）**
  | worktree `D:/autostack/.wt/lang-768/auto-lang` | branch `plan-768-dev`
  | base `540dc7d30` | code commits `2291cd78e → 8feeb9678 → cbe902715 →
  1b13aa323 → 7df514aa2 → 85710109d → 8a9f305cb`（T-04 经 amend 剔除越界
  fmt churn，哈希为 1b13aa323）| deps: auto-down pinned 分离 worktree
  `fba6563e`（组内只读兄弟）| task_ids: T-01..T-08 | evidence:
  [feasibility](attachments/768/feasibility.md)、[evidence](attachments/768/evidence.md)、
  [探针](attachments/768/probe-t01.rs)+[输出](attachments/768/probe-t01-output.txt)、
  [pilot 运行](attachments/768/pilot-run-console.txt)/[JSON](attachments/768/pilot-run-report.json)
  | blockers: 无。
- Work-owned AC 结果：AC-01–08 全过（V-01–10 可执行控制全绿；证据绑定上述
  提交）。review/merge-owned：AC-09（独立复审）、AC-10（沉淀/归档/清理）。
- 诚实披露（review 需核）：①预存门禁红未修（日常档 4 红围栏族+ash_leak、
  KNOWN-DEBT 396、tt 30 红含 plan707 flake 在案、tv cookbook cb_* 10 红——
  均经 stash 基线对照/在案记录归因为预存，本计划 test_cases 0 红）；
  ②附带两项预存缺陷修复（no-default frame_bench、VM 动态段注册断言——
  各附最小实证）；③json_is_null 旧 oracle 为未验证陈值（wrapper #[ignore]），
  独立审查后修正为 true 并在 legacy-map 记录；④examples 基线 fmt 不洁（预存）。
- next_responsible_role: review（/auto-plan:review 独立上下文同树核查）。

### 2026-10-12 — work 启动 / T-01 完成

- 用户确认 r1，status → executing；T-01 完成（证据见 §8 T-01 行与 attachments/768）。
- worktree `D:/autostack/.wt/lang-768/auto-lang`，branch `plan-768-dev`，
  base `540dc7d30839eb7df8e68b4d19472358e170fe5b`，code commit `2291cd78e`
  （no-default frame_bench 修复）。依赖：auto-down 组内只读 pinned 分离 worktree
  `.wt/lang-768/auto-down` @ `fba6563ed2148ce85e68208863159b4ccccac710`
  （跨仓 path 解析 + 裁判来源固定；无 junction/symlink）。
- 预存问题（非本计划引入，需 767 会话知晓）：auto-lang 自 Plan 019 起的
  `autodown-core` 相对 path 依赖使**任何** `.wt/lang-*/auto-lang` worktree 在无
  组内 auto-down 兄弟时 cargo 全档失败；lang-767 现无该兄弟（实测 check 失败）。
- next: T-02（vendor + auto-test-case 基础）。

### 2026-10-12 — 草案交接

- stage: new；PLAN-768；plan_revision: 1；status: drafting。
- outcome: pass（草案结构/范围预检可交接，**不代表实现验收或实施授权**）。
- next: 用户确认 r1 → work/T-01。实施、运行、review、merge 尚未发生。
- T-01–10 覆盖 AC-01–10/SD-01–04；来源/阶段/资源以有界 T-01 前置处理。
- 主检出 v0.6-dev；与 767、真实编译 B、编辑 C 分开；weak/unsupported 可见。

## 10. 待澄清事项

- 按 L1 规则等待用户确认本 revision，然后创建实施工作树。
- parser 许可/来源、阶段/entry 通道、资源参数、多模块试点由 T-01 有界调查，
  不是默认通过的假设。若需跨仓写入/扩支持/削弱验收，先以具体证据修订给用户决定；
  常规路径/等价实现沿已确认范围自行推进。
- 首期执行资源边界如 §5.3；B 交付通用 deadline/containment。报告/help 显示边界，
  不接受声明硬资源要求却无能力执行的检查。
