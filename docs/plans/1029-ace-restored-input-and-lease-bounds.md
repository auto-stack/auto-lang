---
plan_id: PLAN-1029
status: execution_done
feature_name: ACE恢复输入准入与语法租约索引有界性
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 6
current_step: 3
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-syntax/snapshot-cst.md, docs/specs/auto-syntax/grammar-contract.md, docs/specs/auto-semantics/source-scalar-instances.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1029] ACE恢复输入准入与语法租约索引有界性

## 0. 变更摘要

2026-11-17 r6（goal 会话实施）：用户 `/goal 完成计划1029` 授权全链交付。T-01 冻结重核（§4 漂移记录：五项缺陷全部仍在；PLAN-979 新增 rule_expects/ctx_names 使计数预留位变为 **14** 处），T-02 按 r4 DEC-ADM 合同逐片实现 ADM-01..05（每片带永久锚 audit1029_adm_01..05），T-03 固定组与反依赖验证完成（292/293，1 红=master 既有、非本 diff，见 §9）。代码提交 `8d09bd94c`（worktree `D:/autostack/.wt/lang-1029/auto-lang`，branch `plan-1029-dev`，base `19d1f8dc0`）。合同/验收/编号未变；本轮未改 canonical Specs（留 T-05）。

## 0. 变更摘要（历史）

2026-11-13 r4当前：针对r3独立目标复审发现两个合同缺陷：Pratt RuleId被混写为token引用、把解析预算256MiB当装载默认会缩旧codec接受域。已修订逐字段命名空间及由已验证wire推导的兼容解码资源封套，显式调用预算另行分账。 本轮只改计划；drafting 0/6及原Task/AC/Case编号保留，无实现/测试/配置/canonical Spec改动或ACE执行。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F16/F17/F21/F23/F31，优先级**P1（F17/F23为P2）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r5阶段裁定：F16/F17/F21/F23/F31全部A-required，先关闭原ADM-01..05与对应独立oracle再供985/A；恢复Grammar/FirstSets、InstancePage、paged目录与TreeLease均是共享A前门/恢复/owner边界，P2零租约/size-EOF也在本包闭合，不拆成等待B的半树。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

恢复数据先受计数/引用/跨度/格式预算约束再分配或遍历；最后租约释放后的索引可收敛且活租约仍可访问旧树。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。每个子片分别留原错误分类与合法输入；不可借cap缩原支持规模。F17当前仓内主要为测试consumer，不夸大产品泄漏；不做OOM/正式压力实验。

## 2. 架构方案

按四个reader和TreeSession分别修复，不建统一codec或改变合法语言深度。12处计数预留逐项审；Pratt和其他引用在消费前准入；实例页size/tag/EOF；paged目录与inline同等准入、checked跨度；零lease项回收。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。§5已冻结策略；work仅选择等价内部细节，不能临时改变限额/支持或加语言深度限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1029/auto-lang，建议分支codex/plan-1029-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1029/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2历史取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/syntax-runtime/src/program_codec.rs](../../experimental/ace/crates/syntax-runtime/src/program_codec.rs) | 936fdc535ebb5212ff21d718006ed922ee8485dd3059378f4480a11ec318e5a4 | 静态取材，非运行证据 |
| [experimental/ace/crates/syntax-runtime/src/parse/mod.rs](../../experimental/ace/crates/syntax-runtime/src/parse/mod.rs) | 9938851a21e287d01a0d60c3551f09b3c1b6667b84e895bf0f52dbbed7011cbb | 静态取材，非运行证据 |
| [experimental/ace/crates/syntax-runtime/src/handle.rs](../../experimental/ace/crates/syntax-runtime/src/handle.rs) | 484f539d3936ec4e6c78fff52b4b3de5d3d12815210cf29b2dc9ade6c8803907 | 静态取材，非运行证据 |
| [experimental/ace/crates/syntax-artifacts/src/lib.rs](../../experimental/ace/crates/syntax-artifacts/src/lib.rs) | b598874fd8758b99482de8d2b1c0f4ebd19a2e0c77aae82a4244ec0823fe5869 | 静态取材，非运行证据 |
| [experimental/ace/crates/source-query/src/instance_host.rs](../../experimental/ace/crates/source-query/src/instance_host.rs) | 425f873281b0db52b5ed61e47ad8f0c9fe2513cd156ebd92760e9f244952b0c5 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-syntax/snapshot-cst.md](../../docs/specs/auto-syntax/snapshot-cst.md) | 8b158de818761fa468e3b95e1efe84f48f8857fc30b55d8c34b9fcf819cf416c | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-syntax/grammar-contract.md](../../docs/specs/auto-syntax/grammar-contract.md) | 2203a19aa79239fa79b48b6bfb6d923caf119a5c7dd9dff9c0db94f7b6910afb | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-semantics/source-scalar-instances.md](../../docs/specs/auto-semantics/source-scalar-instances.md) | 09cbd9910fd423de7f1d9d6b7b0f52d034d47a1c4c5ce487ff206467fe233aff | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1实例页输入仍按935旧页的单ty字段描述；963已交付v4，合法类型码0..3且有args/consts两个计数。原159字节例不能冒当前codec。五项风险还缺逐入口预留/归属/回收断言。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/source-query/tests/instance_host_935.rs | 74f0082fb08906be11aa015a70020b48532e27fd9b48c1a3887de505ca0fc0c2 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/syntax-runtime/src/program_codec.rs | 936fdc535ebb5212ff21d718006ed922ee8485dd3059378f4480a11ec318e5a4 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/syntax-runtime/src/handle.rs | 484f539d3936ec4e6c78fff52b4b3de5d3d12815210cf29b2dc9ade6c8803907 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/syntax-artifacts/tests/conformance.rs | 6877a8a83ae9395400ea93a2f9ff64fb9d5bde59ac45f6cb9324f5400bd495de | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/syntax-runtime/src/testkit.rs | c1c4dcd74d4ab02f36ad527c225acfae3b60e77d9448cde99fe4b4ba0473deb2 | 现mini_program合法roundtrip控制，避免用自引用Pratt充合法解析 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。


### r3补充依据与来源绑定

r3静态取材Git=6584e94d465dc1f8b2a1cd58344913de4d5ed18a；§4原r2 SHA表是历史快照。本轮重核原表及下列补充来源，当前字节身份见§9核验记录。以下是规则/代码证据，不是运行结果。

| 补充来源 | SHA256 | 作用 |
|---|---|---|
| experimental/ace/crates/syntax-runtime/src/program.rs | 971d0ae6882271ff86154b235bc5d56eee44cf51390232ffaef5c9fd9d646a31 | Budget默认与结构化错误 |
| experimental/ace/crates/artifacts/src/codec.rs | c8e6f98d5557dc97903cee75f321a18709f5e7b4004ad91fc8bf8556c4078556 | 64MiB块门与解码工作预算区分 |


### T-01开工授权与源漂移冻结（2026-11-17，goal会话）

授权：用户 `/goal 完成计划1029`（auto-plan-goal 范式）授权全链交付——本树实施、独立复审、有界修复与 merge；取代本计划 r2/r4/r5 时期"仅规划、不实施"的限制。实施树 `D:/autostack/.wt/lang-1029/auto-lang`（branch `plan-1029-dev`，base=19d1f8dc09eddda3f53e6528ae1072d938e37a4b）；材料 scope `D:/autostack/plan-artifacts/plan-1029/` 已登记。

源漂移重核（T-01，对照 §4 r2/r4 表）：handle.rs、instance_host.rs、identity.rs、instance_host_935.rs、conformance.rs、三个 Spec 与冻结值一致；program_codec.rs、parse/mod.rs、program.rs、testkit.rs、syntax-artifacts/lib.rs、syntax-grammar/lib.rs、artifacts/codec.rs 因 1042/979/1060/1017 等合入漂移。逐入口重读结论（五项缺陷全部仍在，无一项已被他 Plan 顺带修复）：

- ADM-01：现 codec 有 **14** 处计数预留（r4 表 12 + PLAN-979 新增 rule_expects、ctx_names 两表；最小wire字节 rule_expects=1、ctx_names=16）；仍无 count×min 预检，错误仍为 String。store 装载入口 get_grammar/get_grammar_by_id 把一切解码失败映射 Admissions。
- ADM-02：FirstSets::compute 惰性核 Tok/Rule/Pratt下标/CtxName，但 `first[prim]`（parse/mod.rs:227）按 `pratt[x].primary` 无界索引——坏 primary 直接 panic；Machine::new 以 `open_toks[o as usize]`/`token_name(op.op)`/`op_binding[op.op]` 无界索引（467-485）；call_args/postfix_tail（RuleId 命名空间）在 parse 消费点无准入；field/newline/recovery/scanner.mode 命名空间无准入。
- ADM-03：decode_instance_page 无 decoder 入口 512KiB 锚、无 pos==len 全消费检查；四 tag 0..3 拒 255 与 argc/constc≤64 已在（保留为回归）。
- ADM-04：`CstView::decl_entry` 分页路径返回 DeclIndexPage 条目**不经验证**（inline 路径经 validate_manifest）；decl_range `d.origin + nd.start`、`lo + nd.len`（1380-1381）与 decl_subtree `ps/pe_end`（1452-1453）均 unchecked u64 加法——origin=u64::MAX + start=1 即溢出。
- ADM-05：TreeLease::drop 仅 `saturating_sub`，零计数键不删除（handle.rs:145-152）。

### r4复审补充依据

基线Git=09119bd3cface39a8c794e54f9dacdd3f7dd4452，主检出D:/autostack/auto-lang，branch=master。本次按现Spec/代码重新构造反证；r3结论不作为证据。关键补充来源如下（SHA256为文件字节）：

| 来源 | SHA256 | 核查用途 |
|---|---|---|
| experimental/ace/crates/syntax-grammar/src/lib.rs | 087b300bf8f0fe2cdff946ecbd9798d3a5fe1828341cf4234f0a00f3ad09b4dd | 编译器call.args/postfix解析为rule_of |
| experimental/ace/crates/syntax-runtime/src/identity.rs | 87ef3bd2bf5d7d154d45f1e3b0d4733c44b82e7c399ced934cdfcc540cfa1196 | 重编码与identity临时内存来源 |
| experimental/ace/crates/syntax-runtime/src/program.rs | 971d0ae6882271ff86154b235bc5d56eee44cf51390232ffaef5c9fd9d646a31 | RuleId/TokenId字段与Budget本义 |

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F16/F17/F21/F23/F31。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01区分安全可运行入口与禁止裸跑的修前巨大预留坏包；ADM-01修前布局/分配顺序静态核证，修后真实decoder必须早拒绝。

数据/错误/责任：恢复数据先受计数/引用/跨度/格式预算约束再分配或遍历；最后租约释放后的索引可收敛且活租约仍可访问旧树。 按四个reader和TreeSession分别修复，不建统一codec或改变合法语言深度。12处计数预留逐项审；Pratt和其他引用在消费前准入；实例页size/tag/EOF；paged目录与inline同等准入、checked跨度；零lease项回收。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r4执行合同（继承r2输入，以下决定优先）

**输入到消费路径。** ADM-01/02沿program_codec.rs/parse模块现私有测试；ADM-03沿instance_host_935.rs的InstancePage::from(candidate.instances[0])→encode/decode；ADM-04在syntax-artifacts内部或现conformance admission根构造页/目录，保证到达paged分支；ADM-05在handle.rs内部单测可直接读取entries/lease_counts。测试用原encoder后定点变更，既验证变更发生，也避免伪造错magic证明错规则。

**状态、故障与责任。** 不得把v4已修的非法tag拒绝仍称开放缺陷；F23当前确认剩余为size/EOF，四tag与两计数是守恒回归。ADM-01修后实际调用decoder并核受控返回；修前22字节不裸跑。预留顺序以实际reader调用和必要的局部cfg(test)预留观察核实，不建全局分配器/压力套件。ADM-05同时断言活lease和零项收敛，不以entries=1替索引有界。

**决定状态。** DEC-ADM已在本节r4表修订闭合；T-01只核源漂移与使反例可运行，不再选择预算、错误映射或清零时点。
### r4准入决定 DEC-ADM（复审修订后设计闭合）

状态：2026-11-13 closed/design-static-r4；r3两个准入合同缺陷已在本表纠正，§9保原记录与复审 finding。T-01尚须在未来实施树建立实际反例/合法邻例并核源漂移，不能把本次静态决定当成实现或任务完成。

| 对象 / owner | 冻结的准入与资源规则 | 返回与相容结论 |
|---|---|---|
| v3 GrammarProgram / syntax-runtime reader | 按下面12项最小wire字节检查count，再检查分配预算，最后fallible reserve；字符串先checked取片、核UTF-8再分配；总输入必须全消费。所有pos+len、count*min、count*size_of及累计量用checked算术。 | 缺字节、溢出、错版本/tag/引用/UTF-8/尾随字节=InvalidProgram；预算/分配拒绝=ResourceExhausted，不能转成源码语法诊断。count坏包优先按剩余字节判malformed，不触发巨量reserve。 |
| grammar解码工作内存 / 同一装载上下文 | 旧decode_canonical及原get_grammar/get_grammar_by_id的默认选择为FromWire：先以借用byte预检结构/计数/字符串长度，推导本包有限B_wire，再受控分配。B_wire为全部合法count*size_of<T>请求、拥有字符串字节、规范重编码长度、身份临时串和引用校验工作表请求的checked总和；同包不逐表重置，不凭外来count直接reserve。 | 撤销r3无条件256MiB装载门。Budget::work_bytes当前约束解析工作，不证明旧loader有同一上限；新内部入口可显式选Bytes(B)，Bytes只在真实调用者明确提供装载预算或测试时使用，不能让所有旧调用暗中继承Budget::default。FromWire仍是随有限输入推导的有界封套，不是usize::MAX/无界分配。 |
| decoder接口 / syntax-artifacts | 增设结构化入口与明确FromWire/Bytes(B)选择（均拟实施）；现decode_canonical字符串Result包装走FromWire，原两个store装载入口也用同一结构化FromWire路径。InvalidProgram映射Admissions，预算/allocator拒绝保AdapterError::Rt(ResourceExhausted)。 | 不从String猜错误；预算统计受控请求字节，try_reserve_exact等防几何增长，分配前按请求计费，实际capacity与allocator额外驻留另观测，不冒“分配前已知实际容量/RSS硬界”。规范重编码与identity临时请求也计入封套，内容hash/字节算法不变。 |
| 引用 / syntax-runtime | 在授信/FirstSets/parse之前按字段语义核对应命名空间：TokenId=scanner.token、Pratt.op、group_open/close、call_paren.open/close、newline/recovery token；RuleId=start、Step.Rule、Pratt.primary/call_args/postfix_tail；另Step.Tok→tokens、Step.Pratt→pratt、Step.field→field_names；scanner.mode/next_mode→mode_names。 | 每项严格小于对应表长，optional None合法；call_args/postfix_tail虽经opt_tok编码函数读写，语义仍是RuleId。合法递归grammar沿原解析预算，不新增所有引用无环或布尔0/1限制。 |
| InstancePage / source-query | 原512KiB=524288字节锚在decoder入口先执行，等界可接受；保IN935P/v4、类型码0 I32/1 Bool/2 Text/3 Record(name)、argc/constc各≤64；每次取片checked，demands读完要求pos==len。 | >512KiB用现TooLarge；≤512KiB的旧v1..v3保BadVersion/MISS重算；超尺寸（包括旧版）优先TooLarge，错误优先级不冒旧BadVersion恒优先；非法tag、截断/UTF-8/尾随字节属坏页，不升级信任。沿现Truncated坏格式类处理suffix，不扩公共错误枚举/版本；合法encoder字节/identity保持。 |
| CST目录 / syntax-artifacts | paged DeclEntry在取出后、消费前复用inline目录相同验证：page存在、node在该页区间、node_lo及声明索引/计数归属有效、origin≤manifest total_len。decl_range用checked(origin+start+len)，hi≤total_len。 | 错目录/跨度=Admissions（上层按既有受控MISS/重算）；不能panic/wrap成合法跨度。声明长度与manifest先核相等，后续用该唯一total_len，不再引入第二个互相覆盖的权威。按需分页保持，不改成全树eager decode。 |
| 最后一份TreeLease / handle.rs | 选择最后Drop使计数从1到0时立即删除lease_counts键；仍为正的键保留。Drop只清租约索引；旧树entry仍由下次publish按原规则淘汰，当前树entry保持。 | 不是“立即或下次publish任选”。多lease仍能读旧树，最后释放后无零计数键；最后publish后无旧无租约树。物理Rc pin/OS内存回收各按自身owner，不能以清索引保证RSS归零。 |

12处预留的最小编码字节如下；它们是必要下界，后续reader逐字段仍须核截断/引用，不能将count*min当充分格式证明。规则/Alt/Step等嵌套表使用各自当前位置的剩余字节，不得按总输入反复授予预算。

| 预留表 | 每项最小wire字节 | 推导 |
|---|---|---|
| mode_names | 4 | 空字符串长度u32 |
| scanner.rules | 16 | mode4 + 最短StringLit matcher3 + token4 + trivia1 + next_mode4 |
| tokens | 4 | 空字符串长度u32 |
| grammar_rules | 9 | name长度4 + recover1 + alt_count4 |
| alts | 4 | step_count4 |
| steps | 9 | atom(tag1+ref4) + repeat1 + same_line1 + cut1 + field_presence1 |
| pratt | 13 | primary4 + op_count4 + 五个可选字段presence各1 |
| pratt.ops | 6 | token4 + binding1 + assoc1 |
| field_names | 4 | 字符串长度u32 |
| newline.stmt_end | 4 | token u32 |
| newline.sep_consume | 4 | token u32 |
| recovery.sync | 4 | token u32 |

限额之间不能混用：artifacts::codec的64MiB是单存储块payload，实例页512KiB是实例页格式锚，paged manifest512KiB是该manifest读取门，CST_HOT_BUDGET=8MiB是热缓存驻留策略；它们均不等于GrammarProgram解码堆上限。CST已有节点/边/页字段/父子包容检查继续有效，不在本包发明统一codec或语言深度限制。


**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。本轮§5 r4修订后的设计决定已闭合，新增策略是本Plan的拟交付合同；本轮不含实现授权，后续work按既定合同实施，不把关键边界再留到实现中任选。


#### r4相容性反证与资源边界

R-1029-01：tokens=2/rules=5、call_args=4是合法规则引用，错拿tokens比较会误拒；tokens=5/rules=2、postfix_tail=4则会错误放行。未来ADM-02需同时保这两方向和None，不能只测primary。

R-1029-02：在64位表示中，一个rule拥有N个Alt、每Alt一个Tok(0) Step，全部引用合法。N=5000000时wire可变部分13N=65000000B，加不足1024B的单mode/单token/单rule信封仍小于单块67108864B；Alt的Vec载荷至少24N、Step字段最低19N，加规范重编码13N，同驻留下界280000000B已超过268435456B。这里仅算式与布局反证，不创建/运行巨包，也不承诺此规模解析性能；足以反驳“256MiB装载默认对旧wire必然相容”。

FromWire预检必须在count驱动的对象图分配之前完成必要长度/算术检查，以固定层级wire遍历推导B_wire；对后置表的引用可借用重扫，不要求先信任身份。显式Bytes(B)的预算拒绝是资源态，不能重标坏包或静默MISS后重新分配；同一输入按FromWire与充分Bytes预算的程序/identity应相等。不存在新增的grammar规则数/语言深度门。store单块门、实例页512KiB及CST热缓存各维持原对象边界。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F16/F17/F21/F23/F31 → 原ADM-01..05 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：恢复Grammar/FirstSets、InstancePage、paged目录与TreeLease均是共享A前门/恢复/owner边界，P2零租约/size-EOF也在本包闭合，不拆成等待B的半树。关闭谓词：坏count/命名空间/size/EOF/页跨度在授信或分配前受控拒；FromWire/Bytes与原512KiB/当前格式按r4；多lease及最后Drop无零键、合法邻例/恢复后实际消费正确。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-syntax/snapshot-cst.md | 现规则/支持域保持；补充以下修复后的明确合同：恢复程序先核12计数下界/引用再预留/遍历；旧装载默认资源封套由已验证wire推导，显式装载预算另传；不把parse work_bytes的256MiB硬套loader，结构与资源错误分账。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-syntax/grammar-contract.md | 现规则/支持域保持；补充以下修复后的明确合同：inline/paged目录同等准入；origin/start/len先checked再比较；最后Drop立即删零键、publish淘汰旧无lease树，活句柄保持。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-03 | modify（拟议） | docs/specs/auto-semantics/source-scalar-instances.md | 现规则/支持域保持；补充以下修复后的明确合同：实例页现v4/四tag/args+consts；size/全消费准入及旧版失效，不重释旧页。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-syntax ace-semantics。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| ADM-01 | AC-02 / T-02 | 22字节v3头、mode count=u32::MAX；同族14计数位置小坏包 | decode_program(FromWire/Bytes)→decode_canonical→store装载 | 在巨大预留前受控malformed/resource；合法roundtrip与原身份重验保持 | ✅ audit1029_adm_01 已实施并执行（0.013-0.017s） |
| ADM-02 | AC-02 / T-02 | 单rule primary=1坏引用；双rule合法primary=0/坏2；错step/field/recovery引用 | FirstSets::compute及恢复parse入口 | 无panic；错误引用具名拒绝；合法邻例结果不变 | ✅ audit1029_adm_02 已实施并执行（0.009s） |
| ADM-03 | AC-02 / T-02 | 当前v4 args四tag0..3、consts、code255、截断、suffix、512KiB边界 | decode_instance_page→InstanceHost/restore消费者 | 保已拒255/旧版本；补当前size/EOF，args/consts和四tag逐项roundtrip | ✅ audit1029_adm_03 已实施并执行（0.009-0.029s） |
| ADM-04 | AC-02 / T-02 | paged decl origin=u64::MAX、start=1,len=1；正常inline/paged对子树 | CstView::decl_range/decl_subtree和manifest读取 | 越界在算术前拒绝；debug/release一致，无wrap成功；合法跨度不误拒 | ✅ audit1029_adm_04 已实施并执行（0.135-0.146s） |
| ADM-05 | AC-02 / T-02 | 1000轮publish/lease/drop，多lease及最后一次Drop | TreeSession::publish/lease与TreeLease::drop | entries与索引有界；活旧树不误清；最后释放后旧句柄按合同失效 | ✅ audit1029_adm_05 已实施并执行（0.009-0.013s） |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r4具体输入与状态断言（全部待实施）

ADM-01小坏包固定为LE布局u16(version=3),u32(schema=1),u32(name_len=0),u64(revision=0),u32(mode_count=0xffffffff)，22字节；12处同类计数分别给最小合法前缀。此反例修前有巨大预留风险，只保静态证据，不为重现申请巨量内存。

ADM-02除单rule坏primary=1静态反例外，真正合法parse控制采用当前testkit::mini_program或双rule构造：base只含Tok(0)，expr含Pratt(0)，表primary=base(0)，start=expr(1)，对单个token0成功；只把primary改2应拒绝。单rule primary=0可能自引用，不能把FirstSets范围内当全解析合法。

ADM-03以当前IN935P/v4生成正常页：args=[("T",I32)]，consts=[("N",2)]，其余信封由当前常量/真实InstanceRecord取得。0=I32、1=Bool、2=Text、3=Record("R")各有roundtrip；把实际args中code字节变255，不改别的字段。另测试argc/constc=64/65、v1..v3错版本、截断、单尾字节以及512KiB长度邻例。大页只验证decode长度合同，不冒真实巨大泛型源码支持。

ADM-04正常单页/一个声明作底，分别走inline与decl_index分页；只将分页条目origin改u64::MAX、节点start=1,len=1。独立checked-add溢出，必须Admissions拒绝。合法origin+start+len==total_len接受，超过1拒绝。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| ADM-01 | 短包count/剩余字节/总预算/12处预留 | 在不合理预留前受控失败；可失败reserve不代前置计数准入；不设任意语言深度限 |
| ADM-02 | 单rule坏primary1仅测范围；真实合法邻例用base Tok0/expr Pratt0两rule、primary0/start1，坏primary2；另错table/step/field/recovery | 双rule无环邻例可解析；单rule自指0只证明下标在界，不冒合法grammar；坏引用受控拒不panic |
| ADM-03 | v4四tag、args/consts边界、suffix/size；上层prepare/restore | 合法页逐字段相等，255已拒的事实保留；补size/EOF，旧版本按BadVersion/MISS规则失效 |
| ADM-04 | inline/paged、根/子树、末端等界/溢出/跨声明 | 两种目录准入一致；不能仅测inline后给paged记PASS |
| ADM-05 | 1000次publish/lease/drop，两个lease保旧树→drop一个→最后drop→publish | live租约始终可读；最后Drop立即删除零项，索引不随历史轮数线性增长；全drop后再publish还须淘汰旧无lease树，不能只核entries；不等同OS全内存证明 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| ADM-01 | audit1029_adm_01 | [experimental/ace/crates/syntax-runtime/src/program_codec.rs](../../experimental/ace/crates/syntax-runtime/src/program_codec.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| ADM-02 | audit1029_adm_02 | [experimental/ace/crates/syntax-runtime/src/parse/mod.rs](../../experimental/ace/crates/syntax-runtime/src/parse/mod.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| ADM-03 | audit1029_adm_03 | [experimental/ace/crates/source-query/tests/instance_host_935.rs](../../experimental/ace/crates/source-query/tests/instance_host_935.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| ADM-04 | audit1029_adm_04 | [experimental/ace/crates/syntax-artifacts/src/lib.rs](../../experimental/ace/crates/syntax-artifacts/src/lib.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| ADM-05 | audit1029_adm_05 | [experimental/ace/crates/syntax-runtime/src/handle.rs](../../experimental/ace/crates/syntax-runtime/src/handle.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。


### r4决定对应的补充断言（未来实施）

| 原Case | 固定边界与独立预期 |
|---|---|
| ADM-01 | 12个计数位分别验证count*min恰等剩余/多1项及checked溢出；完整小包在FromWire及充分Bytes(B)下结果/identity相同；用显式小B核请求计费等界/多1拒绝。重编码/identity临时计入同一封套；B_wire随合法输入请求单调推导。巨量反证只算式，不裸跑/OOM。 |
| ADM-02 | 所有数值引用表的末项/等于len/None及双rule合法parse；另tokens/rules数量故意不同，分别核call_args/postfix_tail合法高RuleId与非法低于token表但越rule表的值；保递归grammar合法控制，不能用“禁止递归”修复坏primary。 |
| ADM-03 | 构造格式完整且总长524287/524288/524289的v4页（通过合法字符串长度调节，不用追加padding冒合法页）；前两者不因size被拒，后一者在解析分配前TooLarge；单独合法页+1尾字节必须坏页。 |
| ADM-04 | inline/paged分别测origin边界、node页归属、node_lo、声明计数/索引、hi==total/多1/加法溢出；decl_range和decl_subtree都须到真实消费者。 |
| ADM-05 | 同一代2份lease→drop1仍保计数1与旧树可读→drop最后1立即无该键→publish后旧无lease entry淘汰；1000轮每次观察无零项，不能只在结尾清扫。 |


本轮A阶段交接：ADM-01..05的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结五个ADM的真实源、格式/资源边界和前置，合法邻例可在实际入口运行；巨大预留坏包修前仅核静态证据、修后要求实际decoder早拒，不为使反例可运行先触发OOM；落实r4修订DEC-ADM的命名空间/FromWire/Bytes预算与立即清零键断言，不改expected。 [✅ 2026-11-17 §4漂移冻结 + audit1029_adm_01（22字节头/14计数位/等界多1/FromWire≡Bytes 身份）全绿，无OOM裸跑]
- [x] AC-02 / work：r4具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。 [✅ 2026-11-17 五锚 audit1029_adm_01..05 5/5 PASS；§6 r4补充断言（双向命名空间/None、恰等剩余、524287/8/9 等界、立即删零、entries≤2）逐项落进测试体]
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。 [✅ 2026-11-17 cargo t ace-syntax ace-semantics 292/293（唯一红=master 既有 cache-roundtrip.document.case.ad 缺 executed 状态，非本 diff，证据见§9）；反依赖（av/comptime/native/abi/a2x-worker/case-worker/core/m2-build/conformance）check 全绿；decode_canonical 旧调用面不变；canonical Specs 留 T-05]
- [ ] AC-04 / review：逐项核r4独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、实施授权（/goal 2026-11-17）及当前源 | 按§5 r4修订DEC-ADM重核5个ADM真实入口/源漂移；建立不同token/rule表长、FromWire/Bytes小预算、v4合法等界页及即时删零观察；巨大预留修前只静态核证。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合。 [✅ 2026-11-17 §4 漂移冻结 + 五入口重读，五缺陷全在、无他Plan顺带修复；观察设置落进五锚测试] |
| T-02 / work | [x] T-01反例设置与r4既定决定 | 按reader/TreeSession逐片补准入与索引回收；每个ADM有合法邻例和永久断言，F23只补当前剩余size/EOF，保v4四tag/consts。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合。 [✅ 2026-11-17 commit 8d09bd94c：decode_program/LoadBudget/Charge、validate_references、512KiB入口锚+全消费、admit_decl_entry+checked跨度、Drop立即删零] |
| T-03 / work | [x] T-02 | 分别核query实例消费/恢复、inline/paged子树、FirstSets及多lease生命周期；完成syntax/semantics职责组，短包不做OOM实验。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合。 [✅ 2026-11-17 cargo t ace-syntax ace-semantics 293执行292过1既有红（§9具名）；五锚在固定组 nextest 被发现并执行] |
| T-04 / review | [ ] T-01..03完成、冻结提交 8d09bd94c | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1实例页输入仍按935旧页的单ty字段描述；963已交付v4，合法类型码0..3且有args/consts两个计数。原159字节例不能冒当前codec。五项风险还缺逐入口预留/归属/回收断言。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。 DEC-ADM：T-01逐reader列格式/剩余最小字节/现512KiB或既有预算/错误类，结果落§9；使用原限额，任何新增可接受输入上限先明确单位/相容性，不能顺手收紧源码。确认v4非法tag已修后，保该回归而不重复声称修复收益。 该具名决定在T-01关闭前不进入依赖T-02改动；不是把支持判断留给任意实现。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

| 路径 | r1字节SHA256 | r2字节SHA256 |
|---|---|---|
| experimental/ace/crates/source-query/src/instance_host.rs | 860b295538178504c68c3de1b05b0d10f29a2fe8937159015acd23bf376e790b | 425f873281b0db52b5ed61e47ad8f0c9fe2513cd156ebd92760e9f244952b0c5 |

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原5个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。 DEC-ADM为具名T-01决定，关闭前不修改其依赖部分；不是新增用户批准的限额/支持承诺。


### r3设计核清记录

2026-11-13 stage:new / revision:3 / decision:DEC-ADM closed/design-static。依据§4源码/Spec与12项编码布局完成静态核对；22字节头=2+4+4+8+4，scanner最短项=16、Step=9、Pratt=13。现decoder仍有12处with_capacity、现InstancePage仍缺前置size/EOF、现Drop仍保零键；未把决定冒作修复。r2“决定留到T-01”记录为历史，现合同以r3 §5/6/8为准。当前源HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a；完整来源hash由本轮终检核对。原5 Case、6 AC、6任务保留，drafting 0/6；T-01未整项完成。未创建worktree/fixture/探针文件，未运行ACE/构建/正式性能，未提交/合入/归档。

本轮静态来源复核：本Plan原§4来源均重算SHA256，原来源字节与r2相同。本轮三Plan共绑定39个去重来源（源码/Spec/组配置）；已于r3静态终检重核，无来源漂移，历史r2 hash不冒当前执行证据。


r3静态终检通过（2026-11-13）：三份文档0–10章节、r3/drafting 0/6元数据、原18个Task与18个AC及11个Case编号、表列/围栏/空白、本地链接和职责组实际名称核对通过；39个去重来源SHA256与本轮冻结快照一致。git diff --check通过，experimental/ace、crates、scripts无源码diff；未执行ACE测试/构建或把静态检查记作功能/资源/性能PASS。当前只关闭设计决定，未来实施与独立代码复审义务保留。


### r4目标复审与同轮修订记录

stage: review；review_scope: targeted-draft（r3执行前设计合同）；reviewer_identity: 本会话Codex；independence_limit: 与起草同一会话，未另起独立上下文/agent，本轮从现Spec、实际源码和独立构造反证重建裁定，不以r3闭合声明作证。outcome: r3 needs_replan；用户已授权直接修订，现改为r4并复核设计项，非final实现pass、不将状态改reviewed。

reviewed_plan_r3_sha256=b8dfb1c17e732471b2a431f38d99642d2c060994e606530ec8d940da14e1f0ef；source/base_commit=09119bd3cface39a8c794e54f9dacdd3f7dd4452；worktree_path=D:/autostack/auto-lang（计划主检出），branch=master；未创建实施树。reviewed_contract_r4_sha256=43b5543fdf3452c34f06a9ae884f76b4c73e001bd40a46e6171b59226910ad41（LF UTF-8，§0–8含frontmatter）；proposed_spec_delta_sha256=b3d80f15185a758d032af2c3ca3493eaf9756990ab2ca0fe4001a7db4e366438（§5“规范增量”至§6前，LF UTF-8）。原§9 r1–r3收据原样保留但其相容/截止/资源结论已被本次具名修订替代。

| finding / severity | r3合同问题与可复核证据 | 同轮修订 / 关闭oracle与范围 |
|---|---|---|
| R-1029-01 / P1 | PrattSpec.call_args/postfix_tail是RuleId；r3写成可选token检查，tokens=2/rules=5/ref4会误拒，反向tokens=5/rules=2/ref4会漏拒。依据program.rs字段、syntax-grammar rule_of和parse消费。 | §5逐字段命名空间表及§6 ADM-02双方向/None；AC-01/02/04、T-01/02/04。静态合同已纠正，运行证明待work。 |
| R-1029-02 / P1 | Budget.work_bytes现用于parse mem_estimate，旧codec装载无256MiB门；64位5M单Tok Alt的wire<64MiB而r3同驻留下界>256MiB，r3不能称兼容。另实际capacity不能在allocator返回前已知。 | 撤销暗设固定门；旧入口FromWire有限请求封套、显式Bytes(B)分开；请求计费/实际驻留分开。ADM-01小包等界、身份相同及算式反控；AC-01/02/03/04。禁止裸跑巨包。 |

验证方法：rg符号/逐入口读取/Spec对照；Python -B -仅内存算式和文档结构/hash检查。Pratt两方向分别得到rule_valid=true/token_valid=false与rule_valid=false/token_valid=true；N=5000000时wire上界65001024<67108864、受控请求同驻留下界280000000>268435456；ASCII源1048577B的最小信封1048595B<8388608。以上是设计反证，不是运行decoder/compiler/worker，未分配巨包，未运行ACE/构建/正式性能。

acceptance_results：AC-01/02/03及其T-01/02/03实现/动态oracle=pending，AC-04独立实施代码review=pending；本次仅其设计边界/Case/责任/Spec-delta逐项核清。AC-05/06 merge/archive/清理=pending。原6任务/6AC及本Plan Case分母保持，drafting 0/6，不把本轮核清勾为T-01全部完成。所有scope内复审 finding已落实到r4合同，未新增实现债务豁免或以1002迁移代正确性。

依赖快照：1024 r2 reviewed 4/5（SHA256=4834e56b888635bcb68f82bab88d61da39edbdb2c8897dee2f084960641825e5）；1002 r7 drafting 0（SHA256=751c438a33864cd333d19c6aa198c3cfa86aa88cb4974472fb5ac3b3d0358dfa，主检出未提交文档身份）。G1002仍只约束新作者格式/相关接线，不要求本包旧入口修复等待其全迁移。当前源码/Spec/配置以本轮54个去重源字节hash绑定；历史receipt不冒当前产品验证。next：未来获实施授权后在原计划专属树执行T-01反例设置与基线复核，按r4合同进入T-02；不得照r3已撤销限制施工。


补充来源绑定（其余见三Plan §4/既有来源差额表）：

| 来源 | SHA256 |
|---|---|
| docs/specs/auto-a2r/collections-hir-backend.md | 7cffe09c0ae08b86ba200dc742f30020633f01ddb094a70384ad46ef0385597f |
| scripts/test-groups.toml | e8b55f0eeecc01bb4eb2c5d672a11c2113ee00a39aae8842d81b3f650f6c7f2f |


复审期间并发合入1042 Python通道：首次来源冻结时HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a，现按正式合入HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452重核相关差额；不能将初始HEAD与后合入源混写成同一提交。contract只新增Py recipe注册/门，conformance新增Py模块及排序，ace-a2x组新增Python包，原数值cap与原R/C recipe保持。其余已绑定来源按hash逐项复核；本会话不修改/接管他Plan实现。

| 来源漂移 / 新消费者 | 当前SHA256 |
|---|---|
| crates/auto-test-case/src/contract.rs | de83e05a610ca64c14e138c76fd9a7beb18ff25243e5eeb0710d3c0dfbc0af0a |
| experimental/ace/crates/a2x-worker/tests/conformance.rs | 32fa9de0918496bf9d851c427936848602b8a7de0d2d180e307f3795604f35db |
| scripts/test-groups.toml | 1b432bb63abff7fef0222d18db33246b7ec25f15a4015c2393f0f4cb1a84b3ce |
| crates/auto-lang-core/src/test_cases/ac_a2py_hir.rs | c175327e5a4e92d98227b03140df2407e0effbabd326a0d614aa4b3085bfb3b6 |
| docs/specs/auto-a2x/python-stdlib-readiness.md | 930ab8b2fb6eb202f95afa2218aa4530836260d6bd83de411dbf97d2fcd0aa90 |
| experimental/ace/crates/a2x-worker/src/pyrun.rs | edad642fcc60ba21b337341ef29e7d3a9fa0f516d218dd463701bf4237276e18 |
| experimental/ace/crates/a2x-python/src/lower.rs | 37c47e24a989b9561abaf925f90fdfee2042cf252fc744a4f11f00abfec2f3c6 |


r4静态终检（2026-11-13）通过：54个相关源/Spec/配置及2个依赖文档与漂移重核后的绑定一致，结束HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452。三Plan的0–10章节、r4/drafting 0/6、原18任务/18AC/11Case、表列/围栏/空白、本地链接、实际固定组，以及§0–8合同SHA和拟议Spec delta SHA逐项核对通过；git diff --check通过，当前实现路径无WIP。本会话只写这三份计划，并发1042等正式合入差额已重核，不将它们说成本会话实现。设计finding的合同纠正已落实，运行oracle、T-04独立代码验收与merge门仍pending；未执行ACE/构建/性能，不签产品或最终实现pass。

2026-11-16 stage:new/revision | PLAN-1029 r5 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F16/F17/F21/F23/F31/原ADM-01..05明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

### r6 work 交付记录（2026-11-17，goal 会话）

stage: work | plan_id: PLAN-1029 | plan_revision: r6 | outcome: pass（work-owned T-01..03 完成） | worktree_path: D:/autostack/.wt/lang-1029/auto-lang | branch: plan-1029-dev | base_commit: 19d1f8dc09eddda3f53e6528ae1072d938e37a4b（=当时 master） | code_commit: 8d09bd94c | dependency_revisions: 无外部依赖树变更（仅语言仓） | task_ids: T-01, T-02, T-03 | blockers: 无 | next_responsible_role: review（T-04）

实施要点（对照 §5 r4 DEC-ADM，逐行）：

- ADM-01 `program_codec.rs`：新增 `pub fn decode_program(bytes, LoadBudget)`（`LoadBudget::{FromWire, Bytes(B)}`）与 `Charge` 计费（checked 累加；Bytes 逐笔对照上限）。14 处计数位（PLAN-979 后实况）在**任何预留前**按各表最小 wire 字节对借入剩余字节准入（`Reader::count`），全部 Vec 改 `try_reserve_exact`（ResourceExhausted），字符串先 checked 取片/核 UTF-8 再受控分配；规范重编码（max(1024,2×len)）与身份临时计入同封套；错误分账 InvalidProgram/ResourceExhausted。`decode_canonical` 保持签名走 FromWire。store 两个装载入口改 `decode_program`：InvalidProgram→Admissions、预算/allocator 拒→`AdapterError::Rt`（DEC-ADM 第3行）。r3 无条件 256MiB 装载门未复活。
- ADM-02 `program.rs::validate_references`：TokenId（scanner.token、Pratt.op、group_open/close、call_paren.open/close、newline stmt_end/sep_consume、recovery.sync）、RuleId（start、Step.Rule、Pratt.primary/**call_args/postfix_tail**）、Step.Pratt→pratt 表、Step.CtxName→ctx_names、Step.field→field_names、scanner.mode/next_mode→mode_names、rule_expects 平行、ctx_names 语义。`FirstSets::compute` 首行调用（授信前），修复 `first[prim]`（parse/mod.rs 原 :227）与 Machine::new token 表无界索引 panic；合法递归 grammar 不拒。
- ADM-03 `instance_host.rs`：`decode_instance_page` 入口先执行 512KiB 锚（超尺寸含旧版本→TooLarge，优先于 BadVersion）；demands 后 `pos==len` 全消费（+1 尾字节=Truncated）；`get_str` checked；四 tag/argc·constc≤64 原守卫保留。
- ADM-04 `syntax-artifacts/lib.rs`：`CstView::admit_decl_entry`（页存在/节点页区间[下界先判后算]/node_lo≤node/origin≤total_len），paged `DeclIndexPage` 条目取出后、返回前执行（inline 路径同样复用）；`decl_range`/`decl_subtree`（含子节点 cs/ce）origin+start+len 全部 `checked_add`。
- ADM-05 `handle.rs`：`TreeLease::drop` 计数 1→0 时**立即删除** lease_counts 键；正键保留；entries/publish 淘汰规则不变。

本会话测试（profile=dev/test、增量；占用检查：无并发 cargo/rustc/nextest）：

| 测试模块/实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| cargo t ace-syntax ace-semantics（固定组并集） | 293 执行：292 过、1 红、10 skip（含既有 ignored/平台条件） | 复用组缓存+增量 ≈50s（先前 check/test 编译已覆盖大部分） | 组发现（driver 内部） | 4.386s（nextest 汇总） | 无特殊清理 | 含准备约 60s |
| nextest 锚过滤（同 nextest-full.toml，5 bin） | 5 执行 5 过 | ≈7.5s（test profile 增量） | 即时 | 0.138-0.149s | 无 | ≈8s |
| 反依赖 check（av/comptime/native/abi/a2x-worker/case-worker/core/m2-build/conformance lib+tests） | 0 错 | 32.6s | — | — | — | 32.6s |

新增 case（全部 ≤10s 门，nextest 实测）：

| New case ID | 行为与 oracle | profile | 实测 | 结果 |
|---|---|---|---|---|
| audit1029_adm_01（syntax-runtime::program_codec） | 22字节头/u32::MAX 与 14 计数位准入拒；恰等剩余过门后受控截断；FromWire≡Bytes(B) 身份相等；计费等界（最小 B 成功、B-1 ResourceExhausted） | test | 0.013-0.017s | PASS |
| audit1029_adm_02（syntax-runtime::parse） | 坏 primary 具名拒不 panic；合法双 rule 单 token 真实 parse 成功；call_args/postfix_tail 双向（2tok/5rule 过、5tok/2rule 拒）；step/field/scanner/newline/sync/op/group 逐项拒；递归 grammar 过 | test | 0.009s | PASS |
| audit1029_adm_03（source-query::instance_host_935） | 四 tag+consts roundtrip；255 仍拒；argc/constc 64过65拒；v1..v3 BadVersion；截断/+1尾字节坏页；524287/524288 合法、524289 入口 TooLarge 且优先于 BadVersion（合法字符串长度调节，非 padding） | test | 0.009-0.029s | PASS |
| audit1029_adm_04（syntax-artifacts::conformance） | 真实 inline 流合法控制；手造 paged manifest 四类坏条目（origin=u64::MAX+start=1 溢出、页999、node_lo、节点越页）在 decl_range/decl_subtree 双入口受控拒；恰等 [10,16) 接受、多1 [10,17) 拒 | test | 0.135-0.146s | PASS |
| audit1029_adm_05（syntax-runtime::handle） | 1000 轮 publish/lease×2/drop：无零键、索引≤1、entries≤2；最后 Drop 立即删键；跨代活租约可读、释放后 publish 淘汰 | test | 0.009-0.013s | PASS |

具名既有问题（非本 diff，未修不冒 PASS）：

1. **master 既有红**：`ace-semantics-store::conformance sema_boundary_case_files_executed_status_honest` 断言 `crates/auto-lang/test/cases/ac-semantics/cache-roundtrip.document.case.ad` 含 `status: executed`；该文件由 PLAN-1062 恢复提交 `aeeecc8f3` 找回，master 与本树内容均无该字段（两侧 grep=0，worktree 对该路径 diff 为空）。owner=PLAN-1062/语义-store 台账，非本包范围。
2. **既有 warning**：`ace-source-query` lib/tests 各 1 处 `unused_mut`（src/tests.rs:1350、tests/instance_host_935.rs:168，Plan 963 线遗留），不在本 diff 触面。
3. **既有 fmt 漂移**：source-query crate 的 apply/consumer/frontend/lib/restore/tests/architecture_facts_941/instances_value_domain_963 等文件 `cargo fmt --check` 有漂移（master 既有）；本 diff 8 文件全部 fmt-clean，未顺带改写他人文件。

输出路径对账（T-06 正式对账的前置记录）：本轮唯一计划材料目录 `D:/autostack/plan-artifacts/plan-1029/`（已登记 .retention/policy.json），内含 adm04_test.rs 拼接草稿（已折入本计划描述，merge 时随 scope 删除）；测试/构建产物均在 worktree target/ 与组运行临时目录（driver 自管）。无第二证据目录、无外部盘符输出。

stage: goal | plan_id: PLAN-1029 | plan_revision: r6 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1029/auto-lang | commit: 8d09bd94c | next: review

## 10. 待澄清事项

DEC-ADM经r4复审修订后无关键设计未决：引用命名空间、FromWire/显式Bytes封套及512KiB页/目录/lease边界已定。r6 已按合同实施并通过固定组/锚验证；剩余义务：T-04 独立复审、T-05 规范沉淀与正式合入、T-06 收尾对账（含 §9 具名三条既有问题的归属确认：PLAN-1062 恢复文件缺 executed 状态红、source-query 两处 unused_mut、crate fmt 漂移——均不属本包，是否由债项计划承接由 review/merge 记录）。无等待用户的决定。
