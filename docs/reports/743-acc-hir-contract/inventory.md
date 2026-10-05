# PLAN-743 ACC 自举能力盘点报告（inventory）

> 计划：PLAN-743 r1 · 人类审定层（T-03）。机器观察见 [source-manifest.json](source-manifest.json)
> 与 [scan-summary.md](scan-summary.md)；结论绑定见 [manual-decisions.json](manual-decisions.json)。
> 执行身份/输入 hash/离线限制见 [run-context.md](run-context.md)。

## 0. 阅读说明：两个维度，不互相冒充

- **compiler-source-demand（编译器自身用到的语言/运行时）**：本报告从 auto/lib 七文件 +
  auto/aavm.at 的真实源码逐项取证。证据形态 = `file:line` + 扫描类别 + 计数。
- **compiled-language-support（AC 能编译的语言面）**：以 741 归档 Specs 为唯一事实基线
  （core-i32-draft：精确 i32/bool、add/mul/lt、Atom 文本单模块）。旧 AAVM/AA2R 语料绿
  **不**构成 AC implemented 证据；决定栏里凡涉 AC 一律按 741 边界表述。
- 工具词法观察 ≠ 语义解析：所有"已消解"的 unknown 均给出人工核对路径（§5）。
- r2 分类政策（QA-02）：变量接收者方法调用一律 unknown（不按方法名猜 native），
  native 仅限可证明宿主命名空间（List/IO/process/File）；本地类型接收者=type-qualified。
- 状态词汇：`implemented`（AC/741 已有实证）、`required`（ACC 需要且现缺）、
  `unknown`（需探针裁决，附 owner）、`not-required`（附理由/改写路径）。

## 1. 模块迁移决策表（AC-01 / AC-03）

八个源模块逐一审定。角色词汇：**adapt**=适配复用其代码；**adapt-core+rewrite-output**=核心复用、
产物层按新主体重写；**reference**=只作设计/测试参考，代码不搬迁；**non-subject**=不在 ACC 主体；
**retire-for-acc**=ACC 主线退役该机制。

| 模块 | 现有职责（真实入口/证据） | 决定 | 迁移理由与边界 | 前置能力 | 代表语料 |
|---|---|---|---|---|---|
| token.at | 词法判据面：139+Unknown TokenKind、keyword_kind、kind_name（token.at:9/154/219）；零运行时依赖 | **adapt** | 纯 enum+表函数，ACC 前端词法层直接候选；缺 Display/debug 格式化（文件头 Missing），适配点明确 | enum（MD-401）、str（MD-406） | corpus_m1 |
| lexer.at | 游标式扫描（D12 char_at 游标），tokenize→Token List（lexer.at:25,30,672） | **adapt** | 扫描核心自包含；依赖 str 容器操作与 char 比较。Missing：f-string/raw 串/LexerState 快照 → ACC 词法扩面清单 | str/char/List（MD-406/408/405） | corpus_m1 |
| parser.at | 游标 P（15 方法，parser.at:71）+ Pratt 表 + 语句/类型解析；产物为 parse_dump S-expr 直出（D20） | **adapt-core + rewrite-output** | P/Pratt/类型解析核心可复用；S-expr dump 判据层不是 ACC 的 AST——真实 AST/声明模型按新主体重写（MD-103） | record/method（MD-403/404）、mut 参数 | corpus_m2 |
| typeinfo.at | Display 串轻量型推断 typecheck_dump（typeinfo.at:39；D23/D27 自述非完整 typeck） | **reference** | 推断逻辑形状作 ACC typeck 设计参考；**不能作 Checked HIR 有效性凭证**（计划 §4.2），代码不搬迁 | —（参考件） | corpus_m3 |
| codegen.at | ABC 字节码发射（CG 34 方法、符号表/字符串池/占位回填）；cg_compile(_files) 被 engine 消费 | **reference**（+parity 测试依赖） | ACC native 路径=HIR→桥→CLIF，与 ABC 不同管道；工程模式（符号表/池/回填）作参考；cg_* 可作对拍 oracle。**不据此宣称 native 迁移完成** | —（参考件） | corpus_m4 |
| engine.at | ABC 栈机解释器 ev_run/ev_run_files（engine.at:51,53）；Val payload 枚举（engine.at:55） | **reference**（+语义 oracle） | 宿主 VM 行为对齐基线；Val/arena 引用模型是运行时语义参考。自举验收的 oracle 依赖，非主体 | —（参考件） | corpus_m4/m5、corpus_use |
| a2r.at | AA2R 转译驱动 ar_emit_program（a2r.at:4479；Ar 25 方法、static new a2r.at:165） | **non-subject**（AA2R 自举谱系） | 其自举闭环=Auto 写的转译器转译自身（Rust 文本产物），**不是 AC/ACC native 自举**；提供游标/发射器写法与塔式方法论参考 | —（参考件） | corpus_a2r g01–g17 |
| auto/aavm.at | AAVM CLI（process.args/IO.read_line/行数协议，aavm.at:14–45） | **non-subject** | ACC 驱动是新的多阶段主体（MD-205）；入口形态参考 | — | 001_smoke/002_hello_compile |
| lib.rs 注册表段 | AUTO_LIB_FILES_V2 拼接消费面 + use 行剥离（lib.rs:1893/1905） | **retire-for-acc** | ACC 走真模块系统（多编译单元+链接）；注册表保留为 AAVM 事实源并受本盘点监控 | use-module（MD-413） | — |

**新旧快照不拼接的声明**：本表按执行基点 c1ac219e7 的源码审定；与起草点无源码漂移
（run-context.md §1）。主机器恢复后的新输入按计划 §10.4 先做差异报告，不在本表内静默混入。

## 2. 新增 ACC 主体组件（全部"新建"，不从旧 lib 名册推导）

| 组件 | 决定 ID | 说明 | 不可替代性证据 |
|---|---|---|---|
| 名字解析 resolver | MD-201 | 模块级声明表/跨模块 DefRef；lib 内 decl_register/decl_lookup 仅 P 单游标作用域 | parser.at:1802/420 单游标域；741 仅单模块 |
| 类型检查 typeck | MD-202 | 产出 741 语义（精确宽度/初始化/块结构/求值位置）可验证凭证 | typeinfo.at Display 串推断自述非 typeck |
| 源码→HIR adapter | MD-203 | Auto 源码 → Checked HIR + 来源映射；741 只接 Atom 文本 | auto-hir spec:6；auto-ac spec:7 自述未完成 |
| 语义 lowering + 能力门 | MD-204 | while/for 脱糖、is 展开、命名参数落位、目标能力拒绝 | 战略 §3 pass 边界要求 |
| 驱动 + 来源诊断 | MD-205 | 读源→解析→verify→build 编排；file:line:col 源码域诊断 | aavm.at 单阶段形态；ac-probe 未接源码 |
| 后端桥客户端 | MD-206 | Auto 侧薄 C ABI/进程协议客户端（契约见 T-04 文档） | 战略 §4 桥接职责 |

## 3. 保留的非主体服务（不因此算自举完成）

| 服务 | 处置 | 依据 |
|---|---|---|
| Cranelift 0.126.2 桥（Rust 侧） | 保留，v0.6 主后端 | auto-ac spec 工具链节；战略 §1 |
| rust-lld / MSVC link.exe | 保留；发现逻辑留 Rust 侧 | auto-ac spec link.rs |
| kernel32/ExitProcess 启动包装 | 保留（ac_start 形态） | 741 收据 |
| 溢出 trap=ExitProcess(70) | implemented（测试 profile）；完整异常 ABI 另立 | auto-ac spec 溢出节（MD-304） |
| 宿主内建 File/IO/process/print/List | 短期经桥/宿主提供（非主体）；长期归运行时工作包；所有权语义 unknown（probe-rt-own） | engine.at:808/819 nat#1015/1016；aavm.at:19/24 |

## 4. 语言能力清单（AC-01 source-demand / AC-03 状态）

完整证据计数见 scan-summary.md；此处列能力级结论（决定 ID 见 manual-decisions.json MD-4xx）。

### 4.1 required（ACC 主体前置，现 AC 缺）

| 能力 | source-demand 证据（强度） | AC 现状（741） | 缺口/探针 |
|---|---|---|---|
| enum 无载荷 | TokenKind 140 臂、Op、OpCode；lib 全员（MD-401） | 无 | ACC 首批：enum 声明+match 判别 |
| enum 带载荷 | engine Val 六变体（engine.at:55）（MD-402） | 无 | 聚合工作包；先于泛型 |
| record 字段聚合 | lexer/parser/codegen/a2r 25+ 类型；位置式构造（MD-403） | 无 | 布局/传参归 ABI 探针 |
| 方法/static new/隐式 self | P15/CG34/Ar25；codegen.at:168、a2r.at:165（MD-404） | 无 | self 绑定语义进 typeck |
| List\<T\> 容器 | parser25/codegen72/a2r73 注记（MD-405） | 无 | 内置容器先行；用户泛型 unknown（probe-generics） |
| str 字符串 | annotation lexer12/parser76/codegen127/a2r141（MD-406） | 完全无 | **最高优先**：无字符串则诊断/符号表不成立 |
| 跨行字符串字面量 | engine.at:314/315/333/334（MD-407） | 无 | probe-str-ml：字面量语义定义 |
| char 字面量/比较 | lexer 81 例（MD-408） | 无 | probe-char-int：char↔int 映射 |
| int 全序运算 | add/sub/mul/div/mod/eq/ne/lt/gt/le/ge 全在用（MD-409） | 仅 add/mul/lt，溢出 trap | 扩面=补 8 个运算；probe-int-width（int↔i32/i64） |
| bool 逻辑运算 | &&/||/! lexer23/parser64/codegen179/a2r214（MD-410） | bool 类型有、运算无 | 短路求值顺序义务进 pass 契约 |
| while/for/break/continue | while 59/153/136、for 16/4/6（MD-411） | 块/循环脚手架有 | 脱糖 lowering 规则（T-04） |
| is 模式匹配 | token2/lexer5/parser14/typeinfo5/engine24/a2r13（MD-412） | 无 | 对 enum 展开 lowering |
| use 多模块 | 显式 use 边 18 条全量入 manifest（MD-413） | 单模块 | HIR bundle + 跨模块 DefRef |
| mut 参数 | parser35/a2r55/codegen26/typeinfo8（MD-414） | place 级 mutable | probe-own-param（所有权边界） |
| 全局变量 | c.is_global/gkey/var_gtor 链（MD-415） | 无 | 静态数据+初始化顺序 |
| f-string | a2r.at:3887 f"Some(${e})"；FStr token 族（MD-416） | 无 | 或改写少数使用点为拼接 |
| 来源映射诊断 | p.fail 单槽串 vs AC file:line:col（MD-421） | HIR 域 implemented | 源码域 span→HIR required |

### 4.2 not-required（附理由/改写路径）

| 能力 | 证据 | 理由 |
|---|---|---|
| Option/Result ADT 消费 | lib 0 例；a2r 仅发射 Rust 文本（MD-418） | 编译器自身用字符串哨兵 + AutoResult2 record（codegen.at:4303）；ADT 错误处理属完整语言面 |
| \|> 管道算子 | a2r3/lexer2 注释域为主（MD-417） | 迁移触及改写为方法调用 |
| 闭包/task/as 转型 | 源码 0 使用；engine VClo 是 VM 内部载体非源语法（MD-420） | ACC v1 无需求 |

### 4.3 unknown / split

| 能力 | 现状 | 探针 | owner |
|---|---|---|---|
| float/double | 词法识别有、发射执行 0、AC 无（MD-419） | probe-float-need | 源码前端工作包 |
| int↔i32/i64 宽度映射 | 未冻结（MD-409 注） | probe-int-width | 计算子集 adapter（候选①） |
| char↔int | char_at 返 int、区间比较在用（MD-408 注） | probe-char-int | 同上 |
| 宿主容器/字符串所有权 | 释放/RC 语义未定（MD-303 注） | probe-rt-own | 运行时工作包 |
| 用户自定义泛型 | 内置 List 外 0 使用（MD-405 注） | probe-generics | 聚合工作包 |

**禁则重申**：上表 AC 现状列全部按 741 归档边界填写；不存在"旧 VM 绿 ⇒ AC implemented"的推理。
已知 implemented 项仅两个：i32 溢出 trap（测试 profile，MD-304）与 HIR 域阶段化诊断（MD-421 注）。

## 5. unknown 处置汇总（r2 家族制：146 个 (模块,类别,方法名) 组 → 7 族全闭环）

r2 QA-02 政策生效后，变量接收者的方法调用不再按方法名猜 native（113→687 个候选，
聚合为 146 组）；工具层保守保留 unknown，人工层按族裁定。覆盖账本=manual-decisions.json
的 `unknown_families`（7 族，严格门双向闭环），本表是人工裁定摘要：

| 族（模块 → 接收者类） | 裁定 | 决定 ID |
|---|---|---|
| parser.at → p./pp./self. 游标族 | 与 P 方法集逐名吻合（parser.at:71）；跨文件经 use 导入 | MD-501 |
| typeinfo.at → p. 游标族（导入 P 类型） | 同上 | MD-501 |
| codegen.at → c./self.→CG；p./pp.→P | CG static new（codegen.at:168）构造，方法集吻合 | MD-502（P 侧见 MD-501） |
| a2r.at → a./self.→Ar；p.→P | Ar static new（a2r.at:165），驱动 ar_emit_program | MD-503（P 侧见 MD-501） |
| engine.at → c.field_idx/c.pool → CG | 跨模块 CG 实例（engine.at:586）；其余变量接收者归宿主服务族 | MD-504/507 |
| lexer/typeinfo/aavm/engine 变量接收者 | 宿主容器/IO/转换服务族（与 MD-303 宿主内建清单一致）；实现期由 MD-201/202 名字解析精确化 | MD-507 |
| V* 裸构造 | Val payload 变体构造（enum-variant-construction 类别） | MD-505 |
| 跨行字符串（engine.at:314/333） | 合法字面量形态；r1"恢复正确"判定被 r2 QA-01 修正取代 | MD-506（保留为政策变更审定记录）+ MD-407 |

工具侧的保守原则保持：`p.kind()` 这类"局部变量接收者"扫描层永不被静默判定；上表是**人工**
核对结论，输入 hash 变化即失效重审（--check 强制）；unknown_families 与扫描候选双向
闭环由 `--require-decisions` 严格门强制。

## 6. 与旧 AAVM/AA2R 自举的关系（防混称声明）

- 已达成（GOAL-017 谱系，本计划不改写）：AAVM v2 七文件 lib 自举运行、AA2R 自转译闭环
  （Rust 文本产物、纯 cargo build 可编译）。
- **未达成**：Auto 源码 → Checked HIR → native 机器码的编译管线（AC 只有 Atom 文本入口）；
  ACC 主体自举（Auto 写的编译器编译自身）。本盘点全部结论以此为背景，两线不得互称。
- 比特固定点、字节级对拍不作为 ACC 验收门禁（战略 §4；可复现条件另行约定后才可入矩阵）。

## 7. 复核指引（供 /auto-plan:review）

1. 重跑 §5.1 三命令应全绿；--check 对本文件所在报告目录验证 manifest+decisions 新鲜度。
2. 抽查 MD-501..504：任取 unknown 首行 → 与 P/CG/Ar 方法集对照。
3. 抽查 MD-401..409 任一 evidence 行号 → 与源文件对照。
4. 确认 §4.1 每行 AC 现状都能在 docs/specs/auto-{hir,ac}/project.md 找到边界出处。
