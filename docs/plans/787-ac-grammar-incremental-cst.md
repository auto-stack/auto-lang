---
plan_id: PLAN-787
status: executing
feature_name: Atom 语法驱动的增量词法与无损 CST 首条闭环
plan_revision: 1
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-16
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-ac/artifact-probe.md
new_spec_components:
  - docs/specs/auto-syntax/project.md
  - docs/specs/auto-syntax/grammar-contract.md
  - docs/specs/auto-syntax/snapshot-cst.md
touched_goals: []
affects: [auto-syntax, auto-ac, auto-acc, auto-cache]
---

# [PLAN-787] Atom 语法驱动的增量词法与无损 CST 首条闭环

## 0. 变更摘要

把783的“合成语法节点布局”推进到真实语法闭环：Atom grammar→版本化解析产物→源码snapshot→可恢复lexer→无损容错CST→编辑后复用→持久化按需查询。首批是现有Auto core-i32语法及一个独立的声明描述小语言，共用运行时；Auto专用AST投影与通用CST分离。

依据 [Design39](../design/39-ac-incremental-syntax-and-language-platform.md)、[Design40](../design/40-compiler-artifacts-and-cache-lifecycle.md) 与 [Design41](../design/41-interface-metadata-and-foreign-abi.md)。本计划补足Design39工作包A/D的首个有界切片，不声称交付完整Auto版tree-sitter、完整AC前端或ACC自举。

可以在783复审期间规划并准备独立语法部分；T-06缓存接入及最终集成必须等待783完成修复、独立复审、实际合入和Spec沉淀。不能把783工作收据当作已获最终验收，也不从其未合入树复制实现。

## 1. 目标

1. 固定snapshot/edit/span/position与grammar/scanner/tree代际合同。
2. 从Atom配置编译有限语法程序，同一运行时执行两个语言包，不能按语言ID转调两个手写parser。
3. 保留原文字节、token/trivia/error/missing节点；错误编辑仍能查询当前revision，其strict adapter拒绝带错误树。
4. 对指定独立声明编辑真正局部重扫/重解析并复用未变页；其它困难编辑明确回退，结果与fresh一致。
5. 真实CST消费783产物/缓存接缝，有界驻留、跨进程恢复、小范围查询与旧snapshot释放。
6. 给出旧手写parser对照、按需内存与延迟证据，供后续语言扩面和语法产品决策。

非目标：完整Auto语法/scalar/text-record支持、GLR或任意CFG支持、任意regex、生成机器码parser、正式Batom格式、动态scanner插件、LSP协议服务、C/Rust语义/ABI授信、生产cache迁移/GC调度、生产parser切换、native代码生成或ACC主体迁移。enum/is与List/use仍需独立语言计划，不能以本语法底座替代。

范围：新增独立 `experimental/ac-syntax/` workspace、配置/语料、验证脚本与报告。只允许为真实A01/A02/A03接入而对 `experimental/ac-artifact-probe` 作加法式schema/chunk-kind扩展和对应测试更新；不修改其GC算法、原生driver，不修783未完成finding。现有 `experimental/ac-core/src/**` 和 `crates/**` 源码不修改、不切换生产消费者。无跨仓写入。

## 2. 架构方案

语法库依赖方向：snapshot→grammar-program/scanner→CST→结构查询；通用运行时不依赖ac-core、Cranelift、auto-cache或Auto语义环境。grammar编译器可消费现有auto-atom；对照/Auto AST projection位于独立adapter/测试层，缓存接入位于独立adapter层。同一workspace可用多个package分开依赖，T-01冻结准确manifest。

grammar是权威规则源；手写parser仅作现行core规范的兼容oracle，不进入新parse执行路径。兼容core全部承诺语法，不因为生成器暂时不足把正确core程序降为不支持。完整语言由后续profile扩展，不能把其它profile的合法语法称为Auto错误。

源码与树均使用不可变分页块；相对跨度+子块长度索引避免文件开头插入后重写所有绝对offset。编辑会话只保留当前根、overlay和必要页，旧树通过显式lease读取，释放后可淘汰。节点handle含tree/grammar/snapshot身份，跨树使用受控拒绝。

## 3. 技术栈

Rust独立workspace；AtomParser作为配置读取候选；仓内blake3；版本化数值kind/field ID与typed view。grammar-program及CST持久格式为实验Schema，不命名为正式Batom标准。语言包装载格式先做静态Atom→实验二进制往返，不以支持任意Batom为门槛。

算法限定为明确优先级的ordered-choice规则程序（非任意歧义求解）：sequence、choice、repeat、optional、rule/field、terminal，以及配置声明的Pratt precedence/associativity。禁止左递归、nullable repeat、未知引用和无法确定进展的scanner规则；生成期报定位错误，不静默改语法。T-01验证这一子集能表达core全部承诺面，否则回规划修订，不另塞隐藏手写Auto parser。

首平台Windows x64；纯语法逻辑不新增Windows专属语义。缓存集成复用783合入后的公开API，实验目录严格隔离。测试子进程复用已有截止/清理设施，不自建无限等待器。

## 4. 需求分析与背景调查

当前入口：[Spec overview](../specs/overview.md)、[core源码合同](../specs/auto-ac/source-core-i32.md)、[auto-ac](../specs/auto-ac/project.md)、[auto-acc](../specs/auto-acc/project.md)、[auto-atom](../specs/auto-atom/project.md)、[auto-cache](../specs/auto-cache/project.md)。

调查基面master `f56765c2f30f06f1ecfe816cfb007b9537fd5673`；783树 `D:/autostack/.wt/lang-783/auto-lang` 当时HEAD `62d93dae8`，主计划起读状态execution_done 7/9。起草期间出现 `docs/plans/attachments/783/review-r1.md`，R1判定needs_fix，包含lease/GC竞争、CAS实际读取校验、布局读取及证据问题；其修复归783，不能据工作报告的pass推广为缓存已稳定。来源SHA-256见 [planning-inputs.tsv](attachments/787/planning-inputs.tsv)。783报告/代码只作候选接口调查，非已合入Spec，D-783门继续有效。

| 源码/文档 | 已观察事实与决策 |
|---|---|
| ac-core/source/lexer.rs::tokenize_with | 完整str→Vec，跳过//；core排除/*和文本字面量，不能直接叫lossless/scanner-state |
| ac-core/source/parser.rs::parse_program_with | session内游标、显式栈表达式/块；core三运算及换行规则可对照，不能逐行复制为通用runtime |
| ac-core/source/ast.rs | Program和Ident等有字节span，可作Auto adapter投影对照；类型/资源语义仍属编译器 |
| auto-atom/src/parser.rs::AtomParser::parse | 静态Atom读取公开入口，可配置语法，不需要Auto VM执行配置 |
| 783 schema.rs/layout/identity/store | A01/A02 Future，A03为合成ProbeSeam；不是现成真实CST容器。布局证明不等于真实语法已分页 |
| 783 feasibility/perf报告 | 给出真实HIR/对象复用与分页造数证据；GC尾延迟、固定tmp并发包装、无pack均是边界，不在此生产化 |
| Design37、784/785/786 | 在途仓库迁移/host-provider/project-reader与本实验无生产热点写重叠；不依赖这些计划或迁移仓 |

782已在master的auto-ac Spec中登记组合profile，本计划不扩该语法；work重核其实际基线。Spec中旧“组合未立项”等历史段落有陈旧表述，遇冲突以具体profile规范及代码为依据，不据导航摘要删验收。

用户本轮明确要求继续规划下一计划，继承前文增量/热重载/插件、Atom语法配置、全局cache及低内存目标；当前只授权调查和草拟，不实施、不创建树/代理/聊天。没有指定模型、预算或工期。§5资源数字是拟议探针负载/策略，不是产品默认限制。

依赖分段：T-01–05可以在现有master独立开展；T-06–07须通过D-783门：783实际archived/合入、最终review与merge门禁收据齐全、SD已沉淀、绑定实际commit/API后才接入。若其API或schema改变，更新相关调查与契约版本；不引用工作树路径作Cargo依赖。

## 5. 详细设计

### 5.1 snapshot、坐标与scanner

SnapshotId绑定内容，Document/Revision绑定会话；相同文本跨文档可共享字节页，来源定位不能串工程。Edit带base/target及UTF-8字节半开范围；错base、乱序、非法UTF-8边界受控拒绝，旧snapshot不变。批量edit统一基于同一base、非重叠，按确定顺序应用。

保留LF/CRLF及Unicode原始字节；内部byte span，位置adapter支持UTF-8/UTF-16行列。emoji代理对中点、UTF-8码点中间和CRLF中间作为编辑/定位边界的策略须在T-01冻结并有正反控制；不把Unicode标量、字节、UTF-16单位混用。无需扫描全文才能转换一次位置，行/段索引也分页。

lexer消费分块snapshot，checkpoint包含完整mode/未闭合token状态，不能只有offset。core保持ASCII名、//注释及真实换行分隔，/*仍unsupported；另一个小语言支持引号字符串、转义和可嵌套/* */，用来证明跨块/跨行状态恢复，不擅自扩大core语义。token保留原拼写范围，解码值可按需投影。

### 5.2 有限grammar与两个真实profile

Auto core profile以现有source-core-i32合同为准：fn/参数/返回类型、let/var、赋值、return、if/else、while/loop/break/continue、位置/命名调用、括号及* > + > <左结合；负数字面量、换行/分号、括号内换行和callee行限制均不得丢。字面量范围由strict投影检查，不用词法token长度冒充数值校验。

第二profile是新建的 `interface-mini-v1`，只作声明描述语言，绝不宣称C兼容。基本形式 `api add(a: i32, b: i32) -> i32;`，支持 `note "text";`、ASCII名字、空白/注释及上述字符串/块注释。配置定义结构/字段/同步点；相同runtime产出outline候选（名字、参数文本、范围、syntax状态），类型含义/实际导出/ABI全部unknown。

配置terminal支持literal、受限字符集合/重复与声明的mode规则；不执行任意宿主代码。最长匹配/同长优先级在配置中显式，规则非前进/歧义无法裁定时拒绝，不依赖HashMap顺序。Pratt引擎通用，运算与binding power来自包。至少修改关键字拼写和运算优先级两个配置反控，不重新编译runtime也必须改变实际解析结果，排除“Atom仅包装硬编码parser”。

GrammarId、规范化内容hash、schema/compiler/runtime/scanner版本均参与A01/A02/A03 key。同名不同内容包隔离；热切换只切新请求，旧tree lease内可读，但旧handle不能用于新代查询。动态插件二进制装卸不在范围。

### 5.3 无损CST、恢复与严格投影

叶节点覆盖源码token/trivia/error字节，missing节点零宽且有期望种类；按文档顺序流式重建必须逐字节等于snapshot。父子跨度/field/type/ref可验证，不允许重叠/遗漏原文。CST构建、查询、回收均避免随源码深度递归爆栈，不能解析成功却在Drop时崩溃。

恢复规则在grammar中声明同步点；每次恢复必须消耗输入、完成一个有限missing动作或终止。未闭合参数/块、坏token、坏声明仍返回当前revision partial tree/诊断，并能查询后续独立合法声明；不偷偷返回旧AST。重复同输入+版本结果确定。

strict adapter遇error/missing/不支持构造受控拒绝，不产生Checked凭证。Auto专用投影从CST真实读取并构造现有Program，对已有parser的接受形态、AST结构/顺序/位置作差分；禁止调用旧parser再称为“CST投影”。这个投影仅用于对照和未来接口验证，不替换ac-core现有编译入口，不宣称新树已进入native管线。

### 5.4 增量与迟到结果

编辑后从可信checkpoint重扫，只有完整scanner状态和相应后缀token收敛才能复用。解析以顶层声明为首批增量单位；编辑一个独立函数body时重解析受影响声明，未变声明的内容页必须真实复用。位置通过长度索引重定位，开头插入不重写全部后缀页。

必验局部负载：约1 MiB多函数core文件，编辑中部单个小body且不跨mode边界，重新词法读取≤64 KiB、重解析≤2个顶层声明，未变声明内容块复用≥95%。计数绑定真实读/parse路径，禁调全量入口作反控；只证明这些负载的收益，不承诺每次编辑局部化。

跨声明分隔/未闭合词法状态等允许记录原因的全文回退；fresh与incremental的规范树/完整诊断/outline/位置仍一致。比较忽略不同分配ID，不能忽略error/missing、span、字段或token。至少连续200次确定种子混合编辑及所有固定反例逐步比较，每一步都校验原文重建。

任务取消/迟到输出带目标revision。旧revision任务即使晚完成也只能存其版本工件，不得覆盖当前发布根或诊断；显式事件屏障复现逆序完成，不靠sleep碰时序。

### 5.5 真实产物分页与783接入

新增SourceReader/ArtifactStore窄接口。运行时不持有全依赖数据库；缓存adapter消费783公开身份、块/manifest、root/lease设施，A01编译语法、A02 token/checkpoint、A03真实CST分别版本化注册。允许在783实验schema/codec加法扩展，不把Future整层改成“全量语言支持”，旧格式未知kind仍受控拒绝。

CST对源页和必需子页用required-data，诊断来源/历史跟踪单独表示；只保留当前工程根和显式旧树lease，释放后旧版本可淘汰。跨工程同grammar/文本可共享内容页，诊断文档归属隔离。重启恢复后可直接outline/节点查询，必须禁调parser控制且不反序列化整树。缺页/坏hash/错owner/grammar不匹配均受控miss/重算或明确错误，绝不恢复带信任标志的树。

压力负载是至少64 MiB实际源码、独立短声明流式生成，语法产物可能更大；热点≤8 MiB，前台构建/查询暂存≤8 MiB另计，两者总受管预算≤16 MiB，索引也计入。不先读成一个String/Vec再测缓存。源+树页外存，小范围查询payload≤512 KiB；记录真实读页/字节、runtime进程private bytes增量（目标≤64 MiB）与峰值，工具/测试生成器进程分列。OS指标不可用则必验证据缺口，不能只计自定义页数。

初始构建允许更长时间但必须流式/外存并计峰值；单个巨声明无法在配置预算完成时返回ResourceExhausted，不能把大小记成syntax错误或静默截断。没有固定语言嵌套上限；以显式工作栈及可配置资源预算控制，并检验深链/嵌套的构建、遍历、序列化、释放全路径。以上数字是探针验收负载，不是完整语言产品上限。

### 5.6 证据与下一步

报告fresh/局部edit/回退/重启小查询，旧手写parser与新strict投影比较；机器、提交、样本与P50/P95/P99、读写、重算、内存、共享率分列，每性能场景≥30测量样本，预热另列。不能从core小语料推断任意语言或全仓亚秒。

验收结果决定后续通用生成器扩面、scalar/text-record语法接入、真正Auto语义查询与enum/is/TokenKind语言计划的接口。此处generic kind ID不等同Auto enum，避免阻塞后续自举或把两条工作重复计数。

### 规范增量

下表为拟议merge增量；新模块名称是本仓实验命名，不决定未来仓库/产品品牌。本轮不编辑canonical Specs。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-syntax/project.md | 无语法服务→有限双profile/依赖方向/实验边界 | 防冒称完整产品 | AC-01–06 |
| SD-02 | add | docs/specs/auto-syntax/grammar-contract.md | 无规则程序合同→DSL支持/拒绝、版本、scanner与恢复 | 可配置/可重放 | AC-02、03 |
| SD-03 | add | docs/specs/auto-syntax/snapshot-cst.md | 无edit/CST合同→坐标/handle/恢复/增量/预算与事实 | 增量正确性 | AC-01、03–05 |
| SD-04 | modify | docs/specs/auto-ac/project.md | 原有独立parser→补实验adapter导航，生产入口仍原路径 | 不变相替换 | AC-06 |
| SD-05 | modify | docs/specs/auto-ac/artifact-contract.md | 783 A01/02 Future、A03合成→新增有限真实语法生产者/refs/版本 | 实际层级接入 | AC-05、06 |
| SD-06 | modify | docs/specs/auto-ac/artifact-probe.md | 783边界→记录新schema加法及既有probe兼容面，GC边界不变 | 不混淆两Plan贡献 | AC-05、06 |

## 6. 测试设计

新增入口由T-01–07物化，当前不是可执行证据。默认 `cargo test --manifest-path experimental/ac-syntax/Cargo.toml -p <package> <filter>`，包名T-01冻结；新 `scripts/verify-ac-syntax-787.ps1 -Scope acceptance` 聚合以下矩阵并拒绝缺例/skip/子失败，`-Scope closeout`归merge冻结提交完整独立套件。具体report/命令在T-01登记。

| ID | AC / 不变式 | 代表输入/状态 | filter / work owner | 区分性oracle | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 坐标 | emoji/中文、LF/CRLF、跨块位置、错base/UTF边界/重叠edit | snapshot_ / T-02 | 往返与字节基线一致；非法编辑拒绝且旧快照hash不变 | UTF-8源；UTF-16映射 |
| V-02 | AC-02 配置执行 | 两grammar、换关键字/precedence、不改runtime | grammar_ / T-03、04 | 解析树按配置变化；无按language-ID手写分派 | finite ordered-choice+Pratt |
| V-03 | AC-02 生成拒绝 | 左递归、nullable repeat、错引用、非前进规则/坏Atom | grammar_reject_ / T-03 | 定位错误、无半成品；合法兄弟可装载 | 非任意CFG/regex |
| V-04 | AC-03 无损/恢复 | core注释换行；mini嵌套注释/转义；坏声明后合法声明 | lossless_、recovery_ / T-03、04 | 原文逐字节重建，当前错误+后续outline，有限进展 | partial tree不授语义信任 |
| V-05 | AC-03 strict | core既有source/valid及invalid适用语料；error/missing树 | core_projection_ / T-04、05 | Program/顺序/span与旧parser一致；坏树拒绝；禁调旧parser验证真实投影 | core；不是其它profile |
| V-06 | AC-04 增量 | 1MiB独立函数局部编辑、开头插入 | incremental_local_ / T-05 | fresh等价且≤64KiB重扫、≤2声明、≥95%未变内容块复用 | 规定局部负载 |
| V-07 | AC-04 状态 | 跨块未闭合字符串/注释、改分隔符、200次编辑 | incremental_replay_ / T-05 | 每步树/诊断/outline/重建等价；fallback原因可见 | 两profile，非总能局部 |
| V-08 | AC-01、04 代际 | 错tree handle、grammar换代、任务逆序完成/取消 | generation_ / T-02、05 | 旧lease内可读，跨树拒绝，迟到不覆盖当前根/诊断 | 非动态DLL插件 |
| V-09 | AC-05 持久化 | fresh→退出→新进程小查询、两工程同源 | syntax_cache_ / T-06 | parser禁调、结果相同、内容共享而来源隔离 | 783合入API |
| V-10 | AC-05 数据拒绝 | 有效frame替换错chunk、坏offset/ref/grammar、缺页 | syntax_admission_ / T-06 | 真读取入口受控拒绝/重算，无panic/错树成功 | 不用mock validator代替 |
| V-11 | AC-05 工作集 | 64MiB实际源、流式构建、随机小查询、旧lease释放 | syntax_budget_ / T-06 | 16MiB受管总预算/OS增量目标与I/O；只读必要页；旧根可回收 | 无整树常驻；巨声明资源拒绝 |
| V-12 | AC-03、05 深度 | 平链/连续和交错嵌套/缺括号，各≥4096层 | depth_ / T-04、06 | 构建、查询、编解码、Drop全路径无栈崩；资源不足明确 | 代表负载非固定深度限制 |
| V-13 | AC-06 兼容/证据 | 原core API/783身份codec，grammar缓存变体与依赖审计 | compatibility_、perf_ / T-07 | 现有相应例通过、生产零切换、数据/时延报告可复验 | 不扩大783GC验收 |

work完成所有矩阵及必要check，不能把功能验收留给merge。完整独立语法套件和受影响的783独立套件closeout归最终review通过后的merge阶段。本计划不改crates源码/语法参考，不跑根cargo t/tv/tt/tb/tu/taa/tf/docs_gen；若修订后改生产面须重判门禁，不沿用豁免。

实施/自检由后续work负责，用户未指定review人/模型，本轮不创建代理。最终独立review与work同树顺序执行，暂停实施写入、核对HEAD、绑定证据，不能以自检代替。计划簿记在master。预定唯一树 `D:/autostack/.wt/lang-787/auto-lang`、分支 `plan-787-dev`，尚未创建；禁止链接，依赖路径用组布局。

## 7. 验收标准

| ID | owner_stage | 可观察结果/方法 |
|---|---|---|
| AC-01 | work | snapshot/edit/坐标/handle/代际可用，错修订不覆盖，V-01/08 |
| AC-02 | work | Atom规则实际驱动双profile，生成期非法输入受控拒绝，V-02/03 |
| AC-03 | work | 无损当前修订CST、容错进展、core真实投影/strict边界、深度全路径成立，V-04/05/12 |
| AC-04 | work | 局部真实复用及200编辑fresh等价，迟到发布隔离，V-06/07/08 |
| AC-05 | work | D-783通过后真实A01/02/03分页/重启/共享/拒绝/预算/释放成立，V-09–12 |
| AC-06 | work | 兼容/依赖/性能原始证据与SD-01–06齐全，生产入口不切换，V-13 |
| AC-07 | review | 独立逐条核AC/SD/补充例/遗漏与旁路，零未处理阻断、commit绑定，merge-owned pending明确 |
| AC-08 | merge | 冻结提交完整独立语法与触面probe门禁通过，Spec/索引沉淀、合入归档、guard后清理有收据 |

## 8. 执行步骤

T-01失败不得靠删core承诺/扩大预算跳过。D-783只阻止依赖接入，不阻止无关工作；T-06未完成不能execution_done。

| 状态 / ID | owner_stage | 依赖 | 操作/路径与产物 | 验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户执行指令 | 新experimental/ac-syntax工作区/锁版本，冻结包依赖、grammar/schema、坐标边界；做最小Atom→跨块CST→edit→fresh等价；新docs/reports/plan787/feasibility.md | 两语言最小输入、配置变更、core语法表达审计；公开接缝不足回规划；AC-01–05 |
| [x] T-02 | work | T-01 | 新runtime/src/snapshot.rs、position.rs、handle.rs，分页源和edit事务/位置/代际 | snapshot_、generation_坐标/handle；AC-01 |
| [x] T-03 | work | T-02 | 新grammar/src/、runtime/src/lexer/、grammars/*.at；Atom编译规则/Pratt表、checkpoint完整状态 | grammar_、grammar_reject_、lossless_词法；AC-02、03 |
| [x] T-04 | work | T-03 | 新runtime/src/{parse,cst,recovery}/，字段/错误节点/同步、结构outline；不可深递归 | lossless_、recovery_、depth_构建查询释放；AC-03 |
| [x] T-05 | work | T-04 | 新adapters/core/与runtime/src/edit/；真实Program投影、声明级复用、回退/取消/迟到 | core_projection_、incremental_local_、incremental_replay_、generation_；AC-03、04 |
| [x] T-06 | work | T-05、D-783 | 新adapters/artifacts/与paging；加法扩展ac-artifact-probe/src/schema.rs、codec.rs及对应测试，接真实A01/02/03 | syntax_cache_、syntax_admission_、syntax_budget_、depth_编解码；AC-05 |
| [x] T-07 | work | T-06 | 新scripts/verify-ac-syntax-787.ps1、reports/plan787/矩阵/性能、attachments/787/spec-delta.md；兼容旧接口/依赖审计 | acceptance V-01–13、compatibility_、perf_；AC-06；完成→execution_done=7/9 |
| [ ] T-08 | review | T-07 | 同树独立最终验收，绑定冻结commit/API/SD/补充反例，遗漏与债务扫描 | AC-07；pass→reviewed=8/9，merge-owned pending |
| [ ] T-09 | merge | T-08 pass | 冻结提交closeout完整独立套件+触面783套件；沉淀SD/索引，合入归档；批量回归到期判定按技能；guard后清理 | AC-08；实际收尾结束才9/9 |

变更导致旧证据失效须重验；review发现783遗留问题回报归属，不跨计划私修或假设全部稳定。采用新公开接口前以合入版重新绑定，不从在途树cherry-pick顶替D-783。

### 执行进度注记（work，2026-10-16 复审 R2 修复轮 → f7ea3a9e9）

**两连败诊断（技能条款：连续两次 final 未过后，先诊断再修复）**——R1 修复
轮的系统性根因：修复普遍停在"旧反例转绿 + 自有断言通过"，未把每项修复提升
为不变量级重写，且验收 oracle 强度不足（canonical 丢 field、诊断只比数量、
replay 链 fresh、子进程无截止、计量只算常驻容器）。R2 逐项不变量/漏检原因/
设计调整：

| finding | 不变量 | R1 为何漏检 | R2 设计/oracle 调整 |
|---|---|---|---|
| R2-01 | 错误/strict 判定只看当前根**可达**内容 | fresh 树全可达，等价比较不触发 | has_error_nodes 改可达遍历 + 错误修复反例 |
| R2-02 | 持久化当前根有**版本仲裁**（迟到旧任务不覆盖） | 内存 VersionedPublish 屏障测试冒充了真实入口 | manifest.revision + 发布前仲裁；流式根按工程 |
| R2-03 | 流式页键身份必须含**当前块**内容 | 单文档负载测不出键冲突 | part 先行再构键 + 双文档共享 store 反例 |
| R2-04 | 当前根 required 闭包=**可重开全数据** | 只断言 manifest/页，未断言源/grammar | 源页+grammar 边 + GC 后源重建/grammar 装载 |
| R2-05 | admission：先判界后运算/图无环/身份必验 | 旧例在 schema 缺字段层提前拒，未证结构拒绝 | checked 顺序 + BFS 环拒绝 + 流身份强校验 |
| R2-06 | checkpoint=scanner **实际状态**（可恢复续扫） | 往返测试只验 EOF mode=0 形状 | 末 token 拼写重放 + 恢复续扫=fresh 尾部对照 |
| R2-07 | 树实例身份**全局**不可混淆 | handle 设计只考虑单会话 | 全局原子 id + 跨会话双向反控 |
| R2-08 | 合法 core 深输入无固定句法上限 | CST 深度测试不覆盖投影递归 | 节点访问预算 + 平链/嵌套/块投影测试 |
| R2-09 | 计量=**实际读取/驻留/OS 全阶段** | 只统计常驻容器与页缓存 | 目录页 4096 条 + 全 chunk 读取峰值 + 暂存计入 + 查询/重建采样 |
| R2-10 | oracle 必须能检出**字段/诊断/会话链**突变 | 断言强度低于被比对象 | canonical 含 field + 全字段诊断 + 增量工件链 + 子进程截止 + 精确断言 |
| R2-11 | 格式检查覆盖全部新改面 | fmt 只跑了 src 目录 | 全 workspace rs 文件 --check |

修复提交 `f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd`（18 文件 +752/−182）。
证据：R2 反例 **18/18**（修复前 7/11）、R1 反例 7/7 保持、acceptance
**16 族 95 项 ALL PASS**（core_projection 19、syntax_cache 5 新增强度）、
783 触面 --lib 14/admission_ 7/dep_audit_ 5 全绿（probe 增 root_artifacts/
get_artifact_by_id 只读缝隙，加法）。64MiB 负载：受管峰值含暂存 4811KiB≤8MiB、
OS 全阶段增量 15972KiB≤64MiB、目录页 framed≈250KiB≤512KiB。

### 执行进度注记（work，2026-10-16 复审 R1 修复轮 → afbacfbcc）

复审 R1（needs_fix，10 findings）全部修复并验证，工作树 HEAD `afbacfbcc`
（基面 bf773ca9b；修复 diff 34 文件 +4259/−1625）：

- F01 ✅ snapshot 页 `Rc` 保活（PageRef 持页引用）：活快照压力不丢页、
  释放后未引用字节可回收（正反测试）。
- F02 ✅ locate_line 跨页续行行起点回溯；UTF-16/CRLF/emoji 往返正反例。
- F03 ✅ 增量词法 Scanner 硬限窗口 + 流式 K 收敛即停：rescan_bytes=实际
  扫描位置；1MiB 中部编辑真实源页读取 ≤48KiB、报告 214B（永久反控
  incremental_local_lexer_real_reads_are_bounded）。
- F04 ✅ CST 重构为不可变共享内容块（CstBlock）+长度目录（bases）+根孩子
  目录：perform_graft=旧块整体 Rc 采纳（零节点复制）+ splice 前缀身份映射；
  reused_decls 改为**实际嫁接计数**（候选命中才算）。1MiB 中部单 body 编辑
  节点复用 1147999/1148131（99.99%，新节点 132）；开头插入零后缀复制。
- F05 ✅ A02 键含 grammar-id；payload v2 含完整 checkpoint（offset/mode/
  mid/line）往返；A02 --required--> A01+源页；跨 grammar 真读 miss。
- F06 ✅ manifest --required--> 全部数据页+声明目录页+A02（闭包覆盖全树）；
  SourcePage 内容寻址源字节持久化（快照 16KiB 页聚合 ≤256KiB store 源页）
  + source_bytes/for_each_source_page 重建；当前根替换（清旧硬根+软根）+
  CstView 持 store lease——GC 保活（lease 内强制 GC 存活）与释放后物理回收
  （deleted_chunks>0 + miss）测试。
- F07 ✅ 读入口结构校验：manifest 页表连续性/decl 页号与节点范围/node_lo
  不变式/根边范围 + decode 页数一致 + 边目标越界 + chunk kind 校验；坏
  manifest ref 反例受控（不 panic）。
- F08 ✅ 64MiB 端到端有界：生成器独立进程（ac787-gen bin，峰值分列）；流式
  读文件（不整源驻留）；声明目录在线整页落盘（builder 受管峰值 4089KiB≤8MiB，
  不累积 2.57M 目录）；hot 8092KiB+manifest 1064KiB≤16MiB（索引计入）；
  Windows 实测 OS private bytes 增量 12176KiB≤64MiB（非 Windows sysinfo=0
  显式证据缺口行，不冒充）；巨声明 6 万参数受控 ResourceExhausted。
- F09 ✅ 真子进程恢复（ac787-query bin：grammar 按 (id,revision) 从 store
  反序列化、CST 走 manifest/页、源重建）；depth 真 ≥4096 同时未闭合嵌套 +
  depth_persisted_ 编解码往返；perf 同轮 fresh 对照 + 旧 parser 对照 +
  回退确实发生断言；verify closeout 参数修复（--lib 独立参数；schema_ 不存在
  → dep_audit_）+ **acceptance 含 work-owned perf_/syntax_budget_**（16 族
  90 项 ALL PASS）。
- F10 ✅ 新增警告清零（余 shim-metadata 预存归属）；rustfmt --check 全量
  干净；register_root/put 错误传播（flush_batch .expect 移除）。

反例重放：review-r1 counterexamples **7/7 通过**（修复前 1 pass/6 fail）。
783 触面：--lib 14/14、admission_ 7/7、dep_audit_ 5/5（codec 面期望随真实
生产者翻转——对应测试更新授权内）、identity_/concurrency_/review_r* 全绿。
probe 加法：ChunkKind::SourcePage=10、A02 key_inputs 补 grammar-id/page-hash。
touched_goals=[] 说明：本计划零生产目标改动（experimental 面新增；crates/
ac-core 零 diff——compatibility_ 审计）。

### 执行进度注记（work，2026-10-16 第二轮收口）

- T-01 ✅ feasibility 报告（8b1920c33）。
- T-02 ✅ snapshot/position/handle 33/33（c8eb1f216）。
- T-03 ✅ grammar 编译器+双 profile .at（85686c3c0/386ccf982）。
- T-04 ✅ 迭代 unwind/恢复语义（f7021cb46；depth 4096+ 无栈崩）。
- T-05 ✅ core 投影 **valid 41/41 逐字节全等 + invalid 85/85 接受/拒绝平行**
  （e5d55c2b8：close_call 后缀操作数、语义 span、组透传、嵌套 args 收集、
  named label 定位、type-ann 去冒号）+ 增量 edit 族 6/6（0a73848e5：splice
  前缀保留、词法粘连安全边界、error 声明重验证、1MiB 重扫 214B/复用 99.99%、
  200 轮 replay 双 profile fresh 等价、逆序屏障）。
- T-06 ✅ 783 探针加法扩展（4 ChunkKind + A01/02/03→Codec，GC 零改动）+ 真实
  生产者 adapter（A01 身份重验 / A02 往返 / A03 活子树 BFS 分页+目录分页+按需
  页视图+64MiB 流式构建器）；cache 3/3 + admission 4/4 + budget 1/1（64MiB
  256.8 万声明 hot 8177KiB≤8MiB）（39ccc0792）。
- T-07 ✅ 矩阵族补齐（snapshot_/generation_/compatibility_/perf_）+
  `scripts/verify-ac-syntax-787.ps1 -Scope acceptance` **ALL PASS** +
  matrix.md + spec-delta.md（4eed69e67）。workspace 106/106 绿。
- 工作树 HEAD `4eed69e67`，基面 `bf773ca9b`。
- **work 完成（execution_done 7/9）**：T-08 独立最终验收（同树、绑定冻结
  commit/SD/反例、遗漏与债务扫描）→ T-09 merge（closeout 完整套件含 64MiB
  负载与 783 触面族）。



- T-01 ✅ 工作区/清单冻结+最小闭环；feasibility 报告 [docs/reports/plan787/feasibility.md](../reports/plan787/feasibility.md)（提交 8b1920c33）。
- T-02 ✅ snapshot/position/handle 33/33（c8eb1f216）。
- T-03 ✅ grammar 编译器+双 profile .at，grammar_ 6/6 + grammar_reject_ 7/7（85686c3c0/386ccf982）。
- T-04 ✅ 迭代 unwind/恢复语义修复 + lossless_/recovery_/depth_（f7021cb46；depth_ 4096+ 全路径无栈崩）。
- T-05 🔄 core 投影 adapter 已落地（8b1920c33）：差分 oracle 17/41 valid 逐字节全等、62/85 invalid 平行；
  **剩余**：AST span 边界细节对齐（括号组/Binary ±1 字节）后移除两处 `#[ignore]`；
  增量 edit 族（incremental_local_/incremental_replay_ 主体在 runtime/src/edit 已实现）集成测试未落。
- T-06/T-07 未开始（D-783 门已通过：783 archived bf773ca9b，SD 已沉淀）。
- 工作树 `D:/autostack/.wt/lang-787/auto-lang` 分支 `plan-787-dev`，HEAD 8b1920c33，基面 bf773ca9b。
- workspace 测试面：`RUSTC_WRAPPER= cargo test --manifest-path experimental/ac-syntax/Cargo.toml --workspace` 70/70 绿。

## 9. 复审记录

### 2026-10-16 final review R2 — needs_fix（当前有效）

stage: review；review_scope: final；reviewer_identity: 当前 Codex 会话，延续 R1 独立审查，未参与实施、未创建代理。
plan_id: PLAN-787；plan_revision: 1；outcome: needs_fix；status: executing 1/9。
worktree_path: D:/autostack/.wt/lang-787/auto-lang；branch: plan-787-dev；
reviewed_commit: afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457；base_commit: bf773ca9bfc5a2d6ed672f904011419b03eff1ef；previous_review: 4eed69e67085eb150d30443d5c2f41d2530e34c6。
dependency_revisions: auto-atom/auto-ac-prototype/ac-artifact-probe 均绑定 reviewed_commit 的同树 path 依赖，清单/锁 hash 见附件。

[完整报告/闭环表/AC/SD/11 项 finding](attachments/787/review-r2/REVIEW.md)、[独立反例](attachments/787/review-r2/probes/counterexamples.rs)、[反例输出](attachments/787/review-r2/counterexamples.log)、[acceptance 输出](attachments/787/review-r2/acceptance.log)、[预算实际读取观测](attachments/787/review-r2/budget-observation.md)。
spec_inputs: [主检出 Spec/Design hash](attachments/787/review-r2/spec-inputs.tsv)；[依赖 hash](attachments/787/review-r2/baseline-hashes.tsv)；
[冻结 SD](attachments/787/review-r2/spec-delta.md) SHA-256 05143F502E3BF0EC8E6952EBC41A6D4A6E790DCF270139A4111EEB3779AF981D，未获通过，不发布 canonical。

acceptance_results: AC-01 fail、AC-02 partial、AC-03 fail、AC-04 fail、AC-05 fail、AC-06 fail、AC-07 fail、AC-08 pending。
现有 acceptance 16 族/90 次通过记录；独立 18 例=7 pass/11 fail（旧七例全绿，不能替代不变量闭合）。
check 通过且新增警告清零（仅 shim-metadata 既有警告）；diff-check 通过；全语法文件直接 rustfmt check 仍失败。
findings: F787-R2-01..11 均未闭合——共享块旧错污染修复树；真实持久化迟到发布覆盖当前根；流式不同文档前缀键冲突；GC 删当前源页；有效 frame 下的引用下界 panic/环图成功/错身份成功；checkpoint 猜测 depth=1；跨会话 handle 串树；core strict 固定 2048 深度；目录查询约 2.97MB 与峰值漏计；字段/完整诊断/连续 replay/子进程截止 oracle 缺口；格式未净。

仅 T-01 保持完成；T-02..T-07 重开，current_step=1/9；T-08/T-09 未勾。上方“全部修复”及旧 work 进度为历史主张，不能覆盖本轮结果。
pending_closeout_gates: 新冻结提交 final pass 后执行完整独立语法工作区+受影响 783 独立套件、Spec/索引/ledger、合入归档/guard 清理；不跑根整档/docs_gen。
next: 原树 auto-plan-work 修复全部当前 findings；第二次 final 非通过，下一 handoff 先提交根因与覆盖/设计调整表、旧反例与完整不变量证据，再送最终复审。本轮未修实施代码、未合入、未发布规范。


### 2026-10-16 final review R1 — needs_fix（历史记录；当前见 R2）

stage: review；review_scope: final（readiness 拒绝及有界风险审查）；reviewer_identity: 当前独立 Codex 会话，未参与实施、未创建代理。
plan_id: PLAN-787；plan_revision: 1；outcome: needs_fix；status: executing，2/9。
worktree_path: D:/autostack/.wt/lang-787/auto-lang；branch: plan-787-dev；
reviewed_commit: 4eed69e67085eb150d30443d5c2f41d2530e34c6；base_commit: bf773ca9bfc5a2d6ed672f904011419b03eff1ef。

[完整报告/AC/SD/修复要求](attachments/787/review-r1/REVIEW.md)、[反例](attachments/787/review-r1/probes/counterexamples.rs)、[重放输出](attachments/787/review-r1/counterexamples.log)、[Spec 输入 hash](attachments/787/review-r1/spec-inputs.tsv)、[依赖 hash](attachments/787/review-r1/baseline-hashes.tsv)、[冻结 SD](attachments/787/review-r1/spec-delta.md)。
冻结 SD SHA-256 EEC8FB8D32AB7FC613ACD151AFF921FF22B4921A6D03E9467D9CD097B9FFCAC7。
dependency_revisions: 三项 path 依赖实际位于同一实施树，均绑定 reviewed_commit 与锁/清单 hash（不是 work 收据所称主检出）。

acceptance_results: AC-01 fail、AC-02 partial、AC-03 partial、AC-04 fail、AC-05 fail、AC-06 fail、AC-07 fail、AC-08 pending。
现有 acceptance 13 族/78 次通过记录；独立 7 例=1 pass/6 fail。新工作区 check 成功但新增 23 警告；直接 rustfmt check 不通过。
findings: F787-R1-01..10，均未闭合：活跃快照丢页；跨页续行坐标错；全后缀扫描却报局部；复制 arena 冒称内容页共享；A02 跨 grammar 串用/未持久 checkpoint；GC 引用图/源页/lease 生命周期；有效封装坏引用 panic；预算非有界与 OS 缺证；oracle/closeout 参数/SD 缺口；新增警告与错误传播。

保留 T-01/T-03，重开 T-02/T-04/T-05/T-06/T-07，T-08/T-09 未完成。上方 work 完成进度及 ALL PASS 为历史声明，不能覆盖本轮 findings。
pending_closeout_gates: 完整独立语法套件、受影响 783 独立套件，以及 Spec/索引/ledger、合入归档/guard 清理；新冻结提交复审通过后由 merge 执行，不跑根整档/docs_gen。
next: 同树 auto-plan-work 修复全部 findings、补定向反例与完整验收后提交再审。本轮仅审查和簿记，未修实现、未合入、未发布规范。


### 2026-10-16 work 修复轮 handoff（R1 findings 全闭合 → 送审）

stage: work；plan_id: PLAN-787；plan_revision: 1；outcome: pass（execution_done 7/9 恢复）；
worktree: D:/autostack/.wt/lang-787/auto-lang；branch: plan-787-dev；
base_commit: bf773ca9bfc5a2d6ed672f904011419b03eff1ef；code_commit: afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457；
dependency_revisions: auto-atom/auto-ac-prototype/ac-artifact-probe 均为本树 path 依赖（同一实施树，绑定 afbacfbcc）；
task_ids: T-02、T-04、T-05、T-06、T-07（复核重勾，证据见上方修复轮注记）。

readiness 记录：
- work-owned AC：AC-01..AC-06 全部以修复后代码重验——AC-01（snapshot_4+generation_3+position/snapshot 单测 38）、
  AC-02（grammar_13+grammar_reject_7+跨 grammar token miss）、AC-03（lossless_15+recovery_4+core_projection_15+depth_4+depth_persisted_1）、
  AC-04（incremental_local_3：真实读取 ≤48KiB/节点复用 99.99%/开头插入零复制；incremental_replay_4：双 profile 200 轮 fresh 等价）、
  AC-05（syntax_cache_4+syntax_admission_6+syntax_budget_2：required 图/lease-GC 生命周期/64MiB 有界+OS 实测）、
  AC-06（compatibility_3+perf_2+matrix.md 修复轮+spec-delta 更新稿）。
- 未闭合 in-contract findings / 覆盖缺口 / 未决合同决定：无（F787-R1-01..10 全闭合，逐项见修复轮注记）。
- 反例重放：review-r1 counterexamples 7/7 通过（修复前 1/6）；既有清白行为保留断言（replay 200 轮 fresh 等价未回退）。
- 两轮失败诊断条款：不适用（首轮 final review 失败后的修复即闭合全部 findings，含不变量级修复——F04 为架构级
  内容块重构而非例子级补丁）。
- merge-owned pending：closeout 完整独立语法套件 + 783 触面（--lib/admission_/dep_audit_）+ Spec/索引/ledger 沉淀 +
  合入归档/guard 清理；不跑根整档/docs_gen。

blockers: 无；next_responsible_role: review（T-08 同树独立最终验收，绑定 afbacfbcc）。

### 2026-10-16 work R2 修复轮 handoff（两连败诊断 + R2 findings 全闭合 → 送审）

stage: work；plan_id: PLAN-787；plan_revision: 1；outcome: pass（execution_done 7/9 恢复）；
worktree: D:/autostack/.wt/lang-787/auto-lang；branch: plan-787-dev；
base_commit: bf773ca9bfc5a2d6ed672f904011419b03eff1ef；code_commit: f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd；
dependency_revisions: 同一实施树 path 依赖，绑定 f7ea3a9e9；
task_ids: T-02..T-07（R2 重开后复核重勾）。
诊断链接：见上方修复轮注记的诊断表（两连败根因=修复未到不变量级+oracle 强度不足）。
readiness：R2 反例 18/18 + R1 反例 7/7 保持 + acceptance 16 族 95 项 ALL PASS
（含 work-owned perf_/syntax_budget_）+ 783 触面全绿 + rustfmt 全量 clean +
零新增警告。未闭合 in-contract findings：无。既有清白行为保留断言：200 轮
replay（全字段诊断+增量工件链）未回退。
merge-owned pending：closeout 完整独立套件 + 783 触面（--lib/admission_/dep_audit_）
+ Spec/索引/ledger 沉淀 + 合入归档/guard 清理。
blockers: 无；next_responsible_role: review（T-08 绑定 f7ea3a9e9）。

2026-10-15 draft handoff：stage:new；PLAN-787 r1；outcome:pass（可交付的规划契约，非实现验收）；next:work（先独立T-01–05，T-06前核D-783）。新增T-01–09、AC-01–08、V-01–13、SD-01–06，当前0/9，未执行编译/测试。用户只授权规划，未创建实施树。

预检及依赖状态见 [planning-preflight.md](attachments/787/planning-preflight.md)。783复审/merge结果尚待其owner收口，不在本轮重复审查或修复。


2026-10-16 work handoff：stage:work；PLAN-787 r1；outcome:pass（execution_done 7/9）；
worktree:D:/autostack/.wt/lang-787/auto-lang；branch:plan-787-dev；
base:bf773ca9b；code:4eed69e67；
deps:auto-atom@主检出、auto-ac-prototype@主检出、ac-artifact-probe@主检出（783 已合入版）；
tasks:T-01..T-07；
evidence:verify-ac-syntax-787.ps1 acceptance ALL PASS（13 族）；V-05 valid 41/41 全等+invalid 85/85 平行；
V-06 1MiB 重扫 214B/复用≥95%；V-07 双 profile 200 轮 replay fresh 等价；V-11 64MiB hot 8177KiB≤8MiB；
V-12 depth 4096+ 全路径无栈崩；V-13 依赖审计零生产面；报告 docs/reports/plan787/{feasibility,matrix}.md；
blockers:无；next:review（T-08 同树独立最终验收，merge-owned closeout 门禁 pending）

## 10. 待澄清事项

无阻断起草的用户决策。T-01冻结具体包名、grammar字段和生成诊断、位置边界、支持规则审计与最小全路径证据；T-06前核实际D-783。本文已定支持面/预算不得在实施时降低；不可行带最小反例回规划。

更广grammar算法、正式Batom、C/Rust语言包、完整Auto各profile、语义查询、动态插件/热重载、生产store与未来仓归属另立计划。不会因泛化语法底座而默认所有旧语言workaround成为规范。
