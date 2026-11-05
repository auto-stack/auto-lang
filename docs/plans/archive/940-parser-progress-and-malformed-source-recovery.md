---
plan_id: PLAN-940
status: archived
feature_name: 解析推进性与坏源受控拒绝及恢复收口
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-06
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [auto-ui/PLAN-042, PLAN-900, PLAN-906]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-parser-api/progress-and-rejection.md, auto-ui:docs/specs/runtime/reload-readiness-and-lifecycle.md]
touched_goals: []
affects: [parser-core, parser-domain-api, auto-ui-dialect-consumer]
---

# PLAN-940：解析推进性与坏源受控拒绝及恢复收口

## 0. 变更摘要

接收UI042的截断widget约51GB病理分配历史观察，核当前Core/领域端口/UI grammar真实归因，修复EOF/恢复循环不推进或诊断无界累积，让真实坏源受控返回、下一份合法源码仍可消费。未复测前不宣称当前仍OOM或已修复。

## 1. 目标

下表AC与V为本包唯一验收合同。成功指实际producer→consumer闭环、失败受控且下一次合法操作可继续；不以研究归档、静态字符串或进程exit 0替代运行语义。保原合法行为/兼容/平台与资源条件；跨仓按实际owner参与，不重开已归档计划。

## 2. 架构方案

Core保通用游标、诊断与恢复协议；widget/view/store的语法循环由auto-ui拥有，跨仓只在实际根因路径修复。不得把UI词法/AST实现塞回语言仓。若归因完全在UI，可语言零生产差额、UI正式修复+语言控制验证关闭，不能为了标题强造Core改动。不是ACE/SG语法服务替换、任意输入无限资源保证或全面parser重构。

## 3. 技术栈

沿现Rust/Cargo/Auto领域解析、UI VM或Vue生成与已有测试工具。语言仅用固定组；UI沿t/tu（生成单进程）/必要ti按实际职责。T-01核真实--help/包target/过滤与输入，以下新fixture均标新路径，不伪造已存在命令。无依赖升级、工具链/优化/profile变更承诺。

## 4. 需求分析与背景调查

已读语言Spec overview与auto-parser-api/project（DomainParseContext/checkpoint/Err合同）、UI runtime/reload-readiness-and-lifecycle §7、UI042归档§11与真实测试。实物：crates/auto-lang-core/src/parser.rs::parse/parse_impl/parse_domain_impl、parser_bridge.rs、parser_helpers.rs；UI crates/auto-ui-dialect/src/grammar/{widget,view,store}.rs与regression/parser_ui_grammar.rs；runtime plan042_reload_readiness_tests.rs:309明确避开截断输入。widget.rs多处while仅判断RBrace，**这是待验证线索，非已确认根因**。现有资料只有示意坏源 `widget X { model {...} view {`，花括号内省略号不是已证逐字复现。T-01必须从真实合法fixture构造准确截断字节与hash，不把示意字符串当复现收据。

本轮用户授权“规划下一批并列状态/顺序”，只立约与更新规划文档，未授权本包实现。版本与关键内容hash见[本批设计](../../../auto-ui/docs/design/autoui/atomic-reload-and-live-isolation-stage-two.md)；T-01重核正式HEAD、实际源hash与他方WIP。旧历史观察≠当前实测；设计基线不等工具已构建。全部新运行证据为未执行。

**授权更新（2026-11-06）**：用户以 `/goal 实现计划940` 授予全链实施（work→review→有界修复→merge），授权记录见 §9 首条 goal 收据。原“未授权本包实现”状态由此废止；目标/验收/范围/仓库集未变。

## 5. 详细设计

支持与资源：原合法语法/深度/错误分类与检查点语义保持，不附加新的固定语法长度/深度上限。T-01使用小合法widget/view/store及Core块，逐token边界截断、缺关闭符、非法token与恢复后下一声明，固定最多20个短样本。危险复现必须先放受监督子进程：本轮规划的试验保护上限为每例10秒、512MiB进程内存，达到阈值停止并记resource-stop，绝不写PASS或解释为语法错误；该阈值只保护试验机器，不是语言准入限制。开工先证实际Job/进程树终止和内存计量有效；不可用则不运行危险腿，先静态定位或换等价安全监督方式，禁止裸跑历史51GB输入。

推进不变量：循环每轮必须消费合法输入、完成有限结构转换或受控结束/返回错误；checkpoint合法回滚不等同于永久无进展，不能粗暴禁止所有回退。EOF、lexer错误、方言Err三类真实路径分别验；有限诊断含具体位置/上下文，不凭返回非0掩盖崩溃/超时。错误后同一公开消费流程下一次合法parse/build仍成功，注册/作用域不丢。安全回归保留在实际owner的tests，子进程保护不成为生产timeout补丁。

实施树拟定 `D:/autostack/.wt/lang-940/{auto-lang,auto-ui}`，语言分支 `codex/plan-940-parser-recovery`，UI参与分支同号。T-01核929/934–936及UI052/OS055实际共享面，源/接口单写，不借在途修复；不将这些整线变为前置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-parser-api/progress-and-rejection.md | 现具名差额→实证支持/受控拒绝/恢复及边界 | 唯一现行合同 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-parser-api/project.md | 保现owner规则，仅补真实新增行为与限制 | 防跨仓/实例责任漂移 | AC-01..04 |
| SD-03 | modify | auto-ui/docs/specs/runtime/reload-readiness-and-lifecycle.md | 与940/056/057正式交付逐项对账，未实现仍明列 | 不以研究或新包吞其他债 | AC-04/AC-06 |

SD-03跨仓项由该仓owner在同组参与，语言仅留指针；review最终确认真实差额，无差额须记理由而不造规范改动。

## 6. 测试设计

| Case ID | 不变量/输入 | 实际入口/支持状态 | 可观察oracle与关闭条件 | owner_stage |
|---|---|---|---|---|
| V-01 | AC-01；准确坏源与合法控制 | Core Parser::parse/parse_domain、UiDialect真实组合；最小源/hash+源码归因 | 坏源在监督阈值内受控诊断、合法对应源接受；超时/OOM/异常退出不得计通过 | work |
| V-02 | AC-02；EOF及恢复推进 | widget/view/store、Core块；逐token截断、缺闭合、异常token、两声明恢复 | 有限诊断/终止、游标合法、无诊断无限增殖；合法checkpoint回退及非UI同名语法保持 | work |
| V-03 | AC-03；真实UI消费和恢复 | build_dynamic_component与042原坏源保旧腿，使用正式UI组合 | 坏候选不替换旧count=11/handler；合法修复后再次parse/build/reload成功 | work |
| V-04 | AC-04；完整owner职责 | 语言cargo t parser，端口真改补cargo t dialect；三个modularity consumer独立manifest测试与UI原grammar回归 | 原oracle/样本/平台条件不减，零新增回归；源码owner归属守恒 | work |
| V-05 | AC-05；实际冻结源/原子义务/健康/SD | 同树独立上下文final review，首红/环境/条件排除分别记录 | 所有work AC实证满足，无未批准删项/降oracle；复审绑定源版本 | review |
| V-06 | AC-06/07；真实落地/知识/所有输出 | 冻结必要职责并集与canonical/guard/资源读回 | 被测==交付，规范已落地、归档与清理有摘要；持有owner/关闭条件齐 | merge |

语言现有正式命令 `cargo t parser`，端口真改补 `cargo t dialect`。外部三个consumer不是固定组：使用 `cargo test --manifest-path test/fixtures/modularity/<consumer>/Cargo.toml --target-dir <scope>/<consumer>-target`，consumer取dialect-api-consumer、ui-dialect-consumer、ui-grammar-consumer；真实端口受影响才跑对应完整consumer，依赖闭包脚本verify-deps.sh按现有入口保留。纯文档不跑项目门。UI参与仓运行其grammar/runtime职责。新增测试放真实模块namespace/target确保被职责门选中。原样本、金样、负控、ignored/平台/性能条件不减；准备/编译/发现/运行/清理分列，执行/原条件排除/首次红/重放绿分别记录。批量cargo t all到期由主检出单实例程序门承接，非本包重复全目录门。

所有新探针与测试均未运行。本轮授权仅调研、设计、取号、写合同与刷新路线；不建实施树、不改源码、不启动项目、不提交或推送。实施/自查/独立上下文final review使用同一权威树顺序进行，不另建同步review树；本轮不自动创建agent。最终review逐AC、遗漏/延后/workaround、告警/格式与SD核对；通过后merge冻结真实受影响职责并集，复用身份未变有效证据，不堆叠全仓轮次。

材料唯一 `D:/autostack/plan-artifacts/plan-940/<repo>/`，开工登记owner/用途/路径/在途。所有探针、显式外置target、安装/生成副本、日志/截图与隔离数据都在此；正常构建在专属树。merge逐项核实际输出（含仓外/其他盘）、进程/设置恢复，必要摘要折入Plan再删本scope/自有树产物；仅具名未闭hold可留且登记owner/理由/关闭条件。无junction/symlink；fresh wt-guard clean后才移树，不动他方WIP/用户进程，不写legacy specs.json。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 坏源在监督阈值内受控诊断、合法对应源接受；超时/OOM/异常退出不得计通过；由V-01验证 | work |
| AC-02 | 有限诊断/终止、游标合法、无诊断无限增殖；合法checkpoint回退及非UI同名语法保持；由V-02验证 | work |
| AC-03 | 坏候选不替换旧count=11/handler；合法修复后再次parse/build/reload成功；由V-03验证 | work |
| AC-04 | 原oracle/样本/平台条件不减，零新增回归；源码owner归属守恒；由V-04验证 | work |
| AC-05 | 独立final review逐work AC、健康/遗漏与SD pass，绑定冻结源，V-05 | review |
| AC-06 | 冻结门/依赖序合入/canonical/路线/归档真实读回，V-06 | merge |
| AC-07 | 全输出/资源/进程/设置对账、guard清树与scope删除，摘要折Plan/具名hold登记，V-06 | merge |

## 8. 执行步骤

T-01..T-04 已完成（work 阶段，证据见 §9 与各任务行）；T-05..T-07 属 review/merge。

| ID | 动作及关闭结果 | 依赖/AC | owner_stage |
|---|---|---|---|
| T-01 | [x] 冻结准确复现字节/版本、保护机制实证、Core/UI归因与最小关闭路径；不得危险裸跑（16 fixture 逐件 SHA256；监督自检 spin→RESOURCE-STOP@514MB 实证内存计量+进程树终止有效；修复前 11 形态 resource-stop、Core/对照 5 例受控 → 归因全在 auto-ui 文法收集循环，Core 零生产差额） | 既定已交付输入/相关前置；AC-01..04 | work |
| T-02 | [x] 修实际owner的推进/EOF/恢复错误路径，保checkpoint、注册及合法语法（auto-ui 11 处循环加 EOF 守卫，沿 parse_view_node/parse_control_body 既有 `if EOF { expect(closer)? }` 先例；语言仓零生产差额，控制验证见 T-03） | T-01；AC-01..04 | work |
| T-03 | [x] 永久合法/坏源/恢复反例与有界资源记录，补UI真实消费保旧→合法恢复（语言 parser::tests 4 例：截断/前缀扫/parse_domain 恢复/None 回滚；UI regression/parser_eof_progress 7 例：12 形态/前缀扫/语义读回/下一份合法/两声明/checkpoint/混合源；runtime plan042 t02b 截断保旧 count=11→合法恢复） | T-02；AC-01..04 | work |
| T-04 | [x] 跑完整受影响parser/领域consumer职责，健康/遗漏扫描与SD草案，冻结work源（`cargo t parser` 149/149（145存量+4新）；auto-ui-dialect 全套 102/102；plan042 t02 族 26/26；探针修复后 16/16 受控；新告警 0、新行 fmt 干净、无调试残留；端口未改 → 三个 modularity consumer 不触发；SD 草案=§5 表；冻结 commit：auto-ui d99a36b8 / auto-lang 928ca15db） | T-03；AC-01..04 | work |
| T-05 | [x] 独立上下文final review逐AC/反控/资源分类，finding返work（final pass，见 §9 R2 复审记录；finding 0） | T-04；AC-05 | review |
| T-06 | [x] 冻结必要职责并集，依赖序真实合入、canonical读回与归档（冻结门 parser 149/149+UI 102/102+26/26 绑定交付基线；rebase range-diff 全等价；两仓 ff-only 合入 lang ec4fb9368/ui eff9ece3；canonical 读回+main 冒烟 4/4；本文件 git mv 归档） | T-05；AC-06 | merge |
| T-07 | [x] 输出/进程/设置/登记对账，guard后删树与scope，摘要折Plan（四树 guard clean→移除，两分支删除于 tip=master tip，组目录删除；plan-940 scope 整删+登记表刷新，无具名持有；监督器临时文件逐例自删，无外置输出/进程/设置残留） | T-06；AC-07 | merge |

## 9. 复审记录

2026-11-05 / stage:new / PLAN-940 r1 / outcome:blocked（实施授权待给） / 已读实际Specs/源码/归档与在途合同，形成有限执行合同；本轮未实施、未运行探针/项目测试。next：授权后work/T-01，保持原子义务/owner边界，逐任务补证据；最终review与merge/清理均未完成。


静态预检（2026-11-05）：0–10章节、T/AC/V/SD唯一连续且引用有定义；owner_stage、当前本地引用、活动/归档唯一号与单调号源核对通过。未执行任何项目测试、危险复现、浏览器或独立final review，预检不抵实现验收。

2026-11-06 / stage:goal / PLAN-940 r1 / round:1 / from:new / to:work / status:executing / outcome:pass — 用户 `/goal 实现计划940` 授予全链授权（work→review→有界修复→merge）；目标/验收/范围/仓库集不变。worktree：`D:/autostack/.wt/lang-940/{auto-lang,auto-ui,auto-web,auto-down}`（后两者为解析依赖的 detached 兄弟树），分支 `codex/plan-940-parser-recovery`（两主仓同号）。next:work。

2026-11-06 / stage:work / PLAN-940 r1 / outcome:pass / worktree:`D:/autostack/.wt/lang-940/{auto-lang,auto-ui}` / branch:`codex/plan-940-parser-recovery` / base:lang 6fae9d3c8·ui 65aa99ae（各 master HEAD）/ code:lang 928ca15db·ui d99a36b8 / deps:auto-web 37d32c2(detached)·auto-down 58ea33f(detached，仅 manifest 解析) / tasks:T-01..T-04 / blockers:无 / next:final review（T-05）

**T-01 归因与保护实证**（原始输出在 `D:/autostack/plan-artifacts/plan-940/probes/`，merge 折叠后删）：监督脚本 supervise.ps1（Get-Process PrivateMemorySize64 50ms 轮询 + taskkill /T /F）自检 spin 腿 RESOURCE-STOP mem>512MB(saw 514MB)——内存计量与进程树终止实证有效。探针 trunc-probe（plan-artifacts scratch 工程，path 依赖两 worktree，生产入口 `Parser::parse_domain(&UiDialect)`+UI session）×16 fixture（逐件 SHA256，见 probes/fixtures/）：**修复前 11 形态 resource-stop**（c00 UI042 精确形态 1157MB、c01 view 1157、c02 msg 868、c03 for 1157、c04 memo 1157、c05 link 1157、c06 component-fn 参数 1157、c07 handler 形参 1157、c08 setup-refs 580、c09 use-refs 515、c10 use.web-refs 771），对照 5 例受控（c11 model parse_type Err、c12 store-on parse_body expect(LBrace) Err、c13 core 块 2 错有界、c14 合法 OK、c15 嵌套未闭合 view_node EOF 守卫 Err）。**归因：全部在 auto-ui 文法收集循环**（EOF 上 advance 不越过 EOF、parse_view_node/裸 token 臂 Ok 返回游标不动、循环只看闭合符→无限收集）；Core 主循环 EOF 安全、errors 受 error_limit 收口、parse_body/parse_expr/parse_type 在 EOF 受控报错——语言仓零生产差额（§2 明示路径）。

**T-02 修复**（auto-ui d99a36b8，+121/-1 生产）：11 处循环加 EOF 守卫，全部沿仓内既有先例（view.rs parse_view_node 体循环/parse_control_body 的 `if EOF { expect(closer)? }`）：view 根（parse_view_root_nodes）、for/memo/link 子节点循环、msg variants（declarations.rs）、component fn 与 view fn fragment 参数循环（外层+类型标注内层，两份同构拷贝各自加护）、parse_handler_params 外+内层、setup refs、widget use refs、use.web refs。EOF 一律经 expect 受控拒绝（错误带位置与期望闭合符）。checkpoint/注册/合法语法未动。

**T-03 永久反例**：语言 `crates/auto-lang-core/src/parser/tests.rs` +4（plan940_core_truncated_shapes/prefix_sweep/domain_truncation_bounded_and_recovers/domain_decline_rollback，含最小 gadget 测试方言驱动 parse_domain 循环）；UI `crates/auto-ui-dialect/src/regression/parser_eof_progress.rs` 7 例（12 截断形态、合法语料逐字节前缀扫、合法控制件语义读回 count=Int(11)+.Inc、坏源后下一次合法 parse、两声明恢复+诊断有界<10k chars、checkpoint/自动包裹保持、core fn+widget 混合源）；UI runtime `plan042_reload_readiness_tests.rs` t02b（截断候选泵不替换旧组件 count=11 → 修复后 COUNTER_V2 reload 成功 step=5），042 §11“截断输入不采用”具名缺口关闭（t02 注释同步改指 t02b）。

**T-04 冻结门**：`cargo t parser`（lang worktree）PASS 149/149=145存量+4新，wall 86.9s，gate_passed=true；auto-ui-dialect 全套 102/102（86 迁移存量+7 新+codec 等）；plan042 t02 族 26/26；修复后探针监督复跑 16/16 受控（0 resource-stop，全部 0ms）。健康：新增行 rustfmt --check 干净（存量未格式化块不动）、改动文件新告警 0、无调试残留。方言端口未改 → 三个 modularity consumer 与 `cargo t dialect` 不触发（V-04 条件门）。统一回归（`cargo t all` 批量档）pending，owner=merge 后到期程序门。

---

2026-11-06 / stage:review / review_scope:final / reviewer:同会话实施上下文（局限性已声明；结论从工件重建：冻结 commit 实际 diff、独立新选反例、复跑验收，不采信执行方摘要）/ PLAN-940 r1 / outcome:**pass** / worktree:`D:/autostack/.wt/lang-940/{auto-lang,auto-ui}` / branch:`codex/plan-940-parser-recovery` / reviewed_commit:lang 928ca15db·ui d99a36b8（两树 clean 零漂移）/ base:lang 6fae9d3c8·ui 65aa99ae / deps:auto-web 37d32c2·auto-down 58ea33f（detached）/ spec_inputs:docs/specs/auto-parser-api/project.md（65行，841/842 现行）、auto-ui docs/specs/runtime/reload-readiness-and-lifecycle.md §7 / next:merge

**AC 逐项**（pass=实证）：AC-01 pass——修复前 11 形态 resource-stop（负控证明 oracle 可分辨错误实现）+ 修复后执行方 16/16 与复审方独立 12/12 全部 0ms 受控 Err，合法控制件 OK 且语义读回（count=Int(11)/.Inc）；AC-02 pass——双侧逐字节前缀扫完成即 oracle，诊断有界（实测 err_chars 16~97、断言 <10k），EOF 错误经 expect 带位置与期望闭合符，checkpoint 回滚（decline 腿）/命名 payload/自动包裹/非 UI 同名语法（core fn+widget 混合、store）保持；AC-03 pass——t02b 复审复跑 1/1：截断候选不产出 reload、旧 count=11 保值、修复后 reload 成功 step=5；AC-04 pass——parser 149=145存量+4新、dialect 102=95存量+7新、t02 族 26/26，生产 diff 纯增量（唯一删除行是 plan042 注释，断言零减弱），源码 owner 守恒（文法修复在 auto-ui、语言仓 tests-only）；AC-05 pass——本记录。

**复审方独立反例**（12 个执行方集合之外新形态，监督复跑 12/12 受控，原始输出 plan-artifacts/plan-940/probes/review-results.txt）：r01 component-fn 类型标注内层 `f(a: int`、r02 fragment 双参数类型标注、r03 handler 形参无逗号 `.Inc(p1 p2`、r04 for 体带内容截断、r08 use 块 fn-kind refs、r09 UI042 形态尾换行变体——均命中新增守卫族；r05 三层嵌套体/r06 bind/r07 if 体/r10 msg payload/r11 computed/r12 actions——命中既有受控路径，错误信息点名违法 token（如 "Expected string key in bind block, got ''"）。

**遗漏/延后/workaround 扫描**：无遗漏子项、无未经批准延后、无 workaround 补丁；§5"不附加新的固定语法长度/深度上限"满足（守卫仅 EOF 条件，无计数上限）；consumer/dialect 组条件不触发已记录理由；无新债务候选（KNOWN-DEBT-AND-RISKS 无关联条目）。健康复用 T-04 证据（commit 未变、树 clean）：新告警 0、新行 fmt 干净、无调试残留。

**SD 裁定**：SD-01（新增 docs/specs/auto-parser-api/progress-and-rejection.md）**有效**——内容为现行合同（Core EOF/推进/有界诊断保证 + 方言作者推进不变量义务：advance 不越过 EOF、cur_text()=""、收集循环必须 EOF 守卫、`if EOF { expect(closer)? }` 惯例、UI042 病理动机），merge 落盘；SD-02（modify project.md）**零差额记理由不造改动**（§5 明示路径：语言端口行为零变化，现文 841/842 合同仍准确，语言侧仅增控制测试）——frontmatter `supersedes_spec_components` 由 merge 按"零差额理由"处置；SD-03（modify auto-ui reload-readiness-and-lifecycle.md §7）**有效**——具名缺口"parser 病理分配"关闭 + owner 纠正（实为 auto-ui 方言文法循环，非 auto-lang parser）+ t02b 交付记录，auto-ui 仓 merge 时落盘。

**pending_closeout_gates**（merge-owned）：统一回归批量档（`cargo t all` 到期程序门，主检出单实例）；SD-01/SD-03 落盘与读回；归档/清树/scope 删除。findings：无。

---

2026-11-06 / stage:merge / PLAN-940:r1 / outcome:pass（分检查点记录；completion_kind: delivered）

- **prepared**：reviewed 基线 lang 928ca15db·ui d99a36b8（r1 final pass 绑定，两树 clean）；canonical Spec diff=SD-01 新增 `docs/specs/auto-parser-api/progress-and-rejection.md`（37行）+ SD-03 修改 auto-ui `docs/specs/runtime/reload-readiness-and-lifecycle.md` §7（+11/-5）；SD-02 零差额记理由（review 裁定，frontmatter supersedes 清空）；docs-only 后裔 commit lang 8630cc8f3·ui ce08bbde，实现/依赖零变化。冻结门绑定交付基线：`cargo t parser` PASS 149/149 wall 86.9s（run-18cnp0hx）+ auto-ui-dialect 102/102 + t02 族 26/26。
- **landed**：rebase 后 range-diff 全等价（lang 928ca15db=34e539500、8630cc8f3=ec4fb9368；ui d99a36b8=ad83bac3、ce08bbde=eff9ece3——ui 侧 base 前移为 master 新进的 059/064 四 commit，无冲突、本包文件无交集）；两仓 `git merge --ff-only` 成功零合并提交：**lang master tip=ec4fb9368**、**ui master tip=eff9ece3**（即交付 commit）。rebase 后基线复跑：ui dialect 102/102 + t02 26/26 于 eff9ece3；lang 门内容等价（range-diff，master 增量仅 docs/plans 簿记）。
- **canonical_verified**：master `git show` 读回——progress-and-rejection.md 在 master（blob 36b0d1ad，内容 sha256 dda0fcc5…）；auto-ui reload spec §7 含"已关闭，PLAN-940"且由 eff9ece3 落盘。main 冒烟：`cargo test -p auto-lang-core --lib parser::tests::plan940` 4/4。
- **archived**：本文件 git mv 至 `docs/plans/archive/`，status: archived，docs 提交入库（见下方 commit）。
- **观测项（产物时效）**：auto-lang/auto-ui 为库/工具链仓，本工作流不拥有常驻 backend 二进制或 web bundle 部署面（auto-musk 教训不适用）；下游应用下次构建即取修复。无进程/设置残留（探针均被监督器终止，无长驻进程）。
- **cleaned**：四树 `wt-guard.sh` clean（reparse point 扫描）后 `git worktree remove`：lang-940/{auto-lang,auto-ui,auto-web,auto-down}，两仓分支 `codex/plan-940-parser-recovery` 删除于 ec4fb9368/eff9ece3（=各 master tip，落地证明），组目录 `D:/autostack/.wt/lang-940` 删除；`D:/autostack/plan-artifacts/plan-940/` 整 scope 删除（探针源/fixture/结果/监督脚本/外置 target，证据已折 §9），`.retention/policy.json` 刷新（无具名持有）。收尾路径对账：本计划全部显式输出路径=plan-artifacts scope + worktree 内 target（随树亡）+ 监督器逐例临时文件（自删），无其他盘符/散落目录。

## 10. 待澄清事项

- 当前无需用户补充材料即可立约；T-01冻结内部机制、准确入口与实际来源。若改变公开语义、承诺支持或仓责任，给具体修订再进行受影响实施。
- 既有UI052/053、OS055及语法/ACE会话继续各自实施，本包不接管其WIP；文件/符号交集协调单写，实际正式差额才形成前置。
- 历史51GB输入未重放，不能预填当前FAIL/PASS；若当前已修，须以准确坏源、合法控制及真实UI恢复回归证明并记录零实现差额。
