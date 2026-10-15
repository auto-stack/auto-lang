# PLAN-786 T01 冻结记录：调用者盘点 / 旧输入输出 oracle / 测试 filter / V01-03 表

基线：worktree `D:/autostack/.wt/lang-786/auto-lang`，branch `plan-786-dev`，base `a1d5ed311`（= 主检出 master tip 2026-10-15；与会话起点 `d0a454422` 之间仅 docs/plans 簿记差异，无代码差异）。

## 1. 真实调用者盘点（vue.rs 内两个重复行循环的完整消费面）

| 旧函数 | 生产调用点 | 测试调用点 | 处置 |
|---|---|---|---|
| `parse_pac_scalar` (vue.rs:2543) | 经 `parse_pac_title`("title")、`parse_pac_display_title_in`("title_zh") 间接 | vue.rs 测试 10016-10040 | 循环体删除，算法冻结搬迁至 reader 兼容臂（唯一副本） |
| `parse_pac_name` (vue.rs:2560) | vue.rs:3338（generate 流程 name → "aura-app" 兜底） | 无直接（vscode.rs/rust_ui.rs 等是各自独立副本） | 同上 |
| `parse_pac_title` (2518) | 无直接生产调用（display 链内部） | `pac_title_overrides_document_title` | 保留为 reader 的薄适配（测试入口，断言不动） |
| `parse_pac_display_title` (2525) | vue.rs:3342（AUTO_LOCALE env 读取） | 无（env 不可并行测，现状即如此） | locale 判定保留在 vue.rs，字段值改从 reader 读 |
| `parse_pac_display_title_in` (2534) | 无 | `pac_display_title_locale_chain` | locale 链保留，入参从 content 改为 `&DisplayRead`（断言不动） |

**明确保留的非选文本副本**（不在本计划范围，V05 控制不动）：`crates/auto/src/cmd_vue.rs:542`、`crates/auto-man/src/ark.rs:19`、`jet.rs:21`、`rust_ui.rs:3006/3069`（含 parse_pac_exe_name）、`vscode.rs:938`、vue.rs 内 `parse_workspace_path`/`parse_shadcn`/`parse_default_classes` 等行扫描（front 路径/toggle 族）。

## 2. 旧输入输出 oracle（冻结；来源 = 基线代码逐字核对）

旧行扫描语义（两循环共有的值归一，下称「旧行归一」）：行 trim → `key:` 前缀门槛 → 首冒号后文本 `.trim().trim_end_matches(',').trim_matches('"').trim_matches('\'')`。

| # | 输入类别（示例形态） | 旧输出（name / title·title_zh） | 冻结路径分类（见 §3） |
|---|---|---|---|
| 1 | 单字段干净 `title: "X"` / `name: "n"` | Some("X") / Some("n") | Model(Value) |
| 2 | 三字段共存 | 各自独立同上 | Model(Value) |
| 3 | 仅 name、无 title | title→None | Model(Absent) |
| 4 | 重复 `title: "a"`,`title: "b"` | title→Some("a")（首匹配） | Compat(DuplicateKey) |
| 5 | name 重复，首个空 `name: ""`,`name: "x"` | name→Some("")（首匹配不跳空） | Compat(DuplicateKey) |
| 6 | 单独空串 `title: ""` / `title: ''` | title→None（跳空续扫无果） | Model(DeclaredEmpty) |
| 7 | 空串在前 `title: ""`,`title: "x"` | title→Some("x")（跳空续扫） | Compat(DuplicateKey) |
| 8 | 单独 `name: ""` | name→Some("") | Model(Value)（谓词相等） |
| 9 | 引号内边缘空白 `title: " X "` | title→Some(" X ")（quote 内空白不 trim） | Compat(LineFormMismatch)（模型 trim→Some("X")） |
| 10 | `name: " n "` | name→Some(" n ")（name 无 trim） | Model(Value) |
| 11 | 嵌套同名独立行（dep/未知节点/root 包裹体内 `title:`） | title→Some(嵌套值)（行扫描无视结构） | Compat(HiddenOrPhantomLine) |
| 12 | 嵌套同一行 `dep x { title: "i" }` | title→None（行首是 dep） | Model(Absent)（双方一致看不见） |
| 13 | 行内注释 `title: "x" // note` | title→Some(`x" // note`) | Compat(LineFormMismatch) |
| 14 | 单引号 `title: 'x'` | title→Some("x") | Compat(UnsupportedValue) |
| 15 | 转义 `title: "a\"b"` | title→Some(`a\"b`)（保留反斜杠字面） | Compat(LineFormMismatch)（模型反转义→`a"b`） |
| 16 | 插值 `title: "a${b}c"` | title→Some(`a${b}c`)（原文，不求值） | Compat(UnsupportedValue) |
| 17 | bare 值 `title: bare` / `name: bare` | Some("bare") | Compat(UnsupportedValue) |
| 18 | 729 最小对象格式多行（`{` 文档） | 行过 gate 即命中；模型 parse_object_document 同位 | 干净则 Model，否则同上各类 |
| 19 | root 包裹 `root { title: "x" }` 独立行 | title→Some("x") | Compat(HiddenOrPhantomLine) |
| 20 | 块注释藏行 `/*\ntitle: "c"\n*/` | title→Some("c") | Compat(HiddenOrPhantomLine) |
| 21 | 跨行字符串藏行（前 key 的 `"` 吞到后行） | 被吞行的 title 可命中 | Compat(HiddenOrPhantomLine) |
| 22 | 冒号前空格 `title : "x"` | title→None（门槛不过） | Compat(HiddenOrPhantomLine)（模型可见/行扫描不可见） |
| 23 | 行尾逗号 `title: "x",` | Some("x") | Model(Value)（谓词相等） |
| 24 | 引号内尾逗号 `title: "x,"` | Some("x,")（旧归一先剥逗号后剥引号，引号内逗号保留——oracle 修正：与模型一致，非分歧） | Model(Value) |
| 25 | 前缀相似 `titlex: "y"` | title→None（冒号在门槛内） | Model(Absent) |
| 26 | `name: nil` | name→Some("nil") | Compat(LineFormMismatch)（astr(Nil)=""） |
| 27 | `name: 123` | name→Some("123") | Model(Value)（astr(Int)="123"，谓词验证） |
| 28 | f-string / 调用 / on·off | Some(原文片段，如 `env("T")`、`` `x` ``、"on") | Compat(UnsupportedValue) |
| 29 | 赋值 `title = "x"` | title→None（无冒号门槛） | Model(Absent)（双方一致） |
| 30 | 坏静态输入（780 bad-* / 尾随内容 / 括号缺失） | 行扫描照常返回首匹配或 None | Compat(ParseError) |
| 31 | `title: "  "`（仅空白） | title→Some("  ")（非空判定在 quote 剥离后原文上） | Compat(LineFormMismatch)（模型 trim→None） |
| 32 | 无关字段超深嵌套（512 层 `[`/`{` 族，R1 F786-R1-01 反例） | 三字段照旧行扫描返回（`title: "ok"` 正常读出） | Compat(DeepNesting)（前置括号深度筛 >64，不做模型解析——512 层实测 parse_document 栈溢出 abort） |

title 家族与 name 的空值语义差异（冻结）：scalar 族（title/title_zh）空值跳过并**继续扫描**后续行；name 首个门槛行**立即返回**（可 Some("")）。

## 3. DTO 准入条件与具名兼容理由（T02 决策，源码依据）

一次 `auto_project_model::parse_document`（SourceInfo::inline）产出三字段共享的 descriptor；每字段独立判定：

- **Model(Value)**：解析 Ok ∧ gate 行集合==provenance 行集合 ∧ 无重复键 ∧ 模型值 Some(v) ∧ 旧行归一(该行)==Some(v)。
- **Model(Absent)**：解析 Ok ∧ gate 空 ∧ provenance 空（真实缺席；赋值/前缀相似/同行嵌套归此）。
- **Model(DeclaredEmpty)**：title 族专属——gate==prov 单行 ∧ 模型 None ∧ 旧行归一空（旧行为跳空续扫无果→None）。
- **Compat(ParseError)**：静态解析 Err（坏输入），旧行扫描照常。
- **Compat(HiddenOrPhantomLine)**：gate≠prov——模型看不见的行（块注释/跨行字符串/参数表/结构化值内部/嵌套节点体）或模型可见而行扫描不可见（冒号前空格）。
- **Compat(DuplicateKey)**：重复键（旧首匹配 vs 模型 last-wins）。
- **Compat(UnsupportedValue)**：值非干净静态字面量（单引号/bare/插值/f-string/调用/on-off），或 name 的 None+单行形态。
- **Compat(LineFormMismatch)**：值虽静态但行形态与旧行归一不等（转义、quote 内边缘空白、行内注释、非 Str 的 astr 差异如 nil）。
- **Compat(DeepNesting)**（R1 追加，R4 终形）：**开启括号字符总数**（`[`+`{`）>64（迭代单遍，零词法判定）。上界论证：解析器结构下钻按嵌套层递归（元素迭代、下钻才递归，每层必消费一个 opener 字符）⇒ 真实递归深度 ≤ 开启括号总数；闭括号（任意上下文）不参与计数不可能削减上界，注释/字符串上下文只会虚增计数（过计数=多走兼容臂，值不变）。四轮反例同根（按字符串/注释词法跳区域或让闭括号抵消计数）：R1 遇 LF 断、R2 跨 LF 吞 EOF、R3 skip_to_entry_end 行界重入、R4 字符串内注释标记藏括号+字符串 `]` 逐层抵消——零词法判定使该反例族结构性不存在。超限输入不做模型解析，三字段整体走兼容臂：512 层实测 STATUS_STACK_OVERFLOW abort（catch_unwind 不可捕获）。阈值 64=实测 128 通过/512 abort 之下的成对余量；进程隔离永久回归（子进程重放 R1..R4 全部形态，10s 外部截止=kill+reap 全路径守护）+ 反例原样字节 fixtures（`r3-*.at` 8 文件 + `r4-*.at` 10 文件）逐文件复验（见 §4）。

生产不做双臂逐次判等选路：Model 路径只运行模型 + 单行形态谓词（对**该声明行**的旧归一复算，非全文件双臂）；其余全部走兼容臂（旧行扫描唯一冻结副本，全文件首匹配）。旧行为对拍 oracle 仅存在于测试模块。

## 4. 测试 filter 登记（新 reader 族，`cargo test -p auto-man --lib display_reader`）

| filter | V | 内容 |
|---|---|---|
| `display_reader_v01_managed_corpus_matches_frozen_oracle` | V01 | git ls-files 全受管 pac.at（≥140）逐文件三字段 reader==旧 oracle；Model/Compat 计数收据 |
| `display_reader_v02_frozen_edge_matrix` | V02 | §2 矩阵全行内样例：冻结期望值 + 路径分类 + oracle 三方一致 |
| `display_reader_v03_locale_chain_explicit` | V03 | 显式 prefers_zh（zh/zh-CN/en/无 title_zh/title 缺席）；不触 env |
| `display_reader_v04_dynamic_bad_inputs_no_vm_no_writes` | V04 | 780 dynamic-*/bad-* fixtures + 插值/块注释控制：Compat 路径、不伪装静态、输入文件字节不变、10k 行输入有界完成 |
| `display_reader_v05_unified_entry_and_unselected_fields_intact` | V05 | vue.rs 两循环已删/新入口被调用（源码断言）；shadcn/default_classes/rust_ui/vscode 等非选路径原样 |
| `display_reader_v04_deep_nesting_process_isolated` | V04 | R1 起、R4 全集：自执行子进程重放 R1..R4 全部反例形态（平面/跨行/未闭合引号四形态/字符串内注释标记/闭括号抵消）；截止=ChildGuard 生命周期守护（全路径 kill+reap+诊断收集）+ 确定性挂起控制（30s vs 2s → TimedOut 非成功回收态） |
| `display_reader_v04_counterexample_files_replay` | V04 | R3 起原名 r3_、R4 扩展：复审反例**原样字节**（`r3-*.at` 8 + `r4-*.at` 10 持久 fixtures）逐文件复验——name=n/title=ok、32 控制 Model、>64 开启全 DeepNesting、oracle 三方一致 |
| `display_reader_v04_deep_nesting_boundary_and_awareness` | V04 | R1 起、R4 终形：开启括号 64(Model)/65(DeepNesting) 边界、{} 节点嵌套 512、零词法判定感知（字符串/注释内开括号计入但包络内不改变准入）；R2 跨行族、R3 未闭合引号四形态、R4 quoted-marker/cancel-depth 四形态进程内矩阵；未终止组→ParseError 安全路径 |

既有冻结测试（保留、断言不动）：`pac_title_overrides_document_title`、`pac_display_title_locale_chain`（入参形态随 reader 调整）。
新 fixtures：`test/fixtures/modularity/project-model/display-reader/`（三字段/缺标题/重复键/单引号/嵌套/块注释/插值/空标题各一）。**R1 改名**：扩展名 `.pac.at`→`.at`——脱离 780 受管语料 walker（`git ls-files *.at` + `ends_with("pac.at")`）的精确身份；display-reader 冻结面含旧 VM 拒绝的负例形态（单引号等），与 780 VM-语料合同是不同 oracle，负例集合隔离（R1 F786-R1-02）。

## 5. 冻结 V01/02/03 表（验收口径）

- **V01**：受管语料（≥140 文件）+ display-reader fixtures，三字段 `reader.{name,title,title_zh}` 与冻结旧 oracle 逐值相等；受管语料以 Model 路径为主导（计数留档），/fixtures 含 Compat 案例。
- **V02**：§2 全矩阵行内断言（期望值 + ReadPath 分类），oracle 三方一致；未支持不得假记 DTO 等价（路径分类即证据）。
- **V03**：`display_title(read, prefers_zh)` 链 zh→title_zh 优先、en→恒 title、无 title_zh 两态一致、title 全缺席→None（name/目录兜底留在调用者，由既有 `pac_title_overrides_document_title` 的 generate_index_html 断言继续覆盖）；显式入参，无 env 竞争。
