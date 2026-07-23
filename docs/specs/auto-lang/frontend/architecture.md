# frontend 架构

```mermaid
graph LR
  subgraph 输入
    SRC[".at 源码"]
  end

  subgraph frontend["frontend（crates/auto-lang/src）"]
    US["use_scanner.rs<br/>scan_use_statements<br/>（字符串级预扫描）"]
    LEX["lexer.rs:Lexer<br/>（内部模块）"]
    TOK["token.rs:TokenKind/Token/Pos"]
    P["parser.rs:Parser<br/>递归下降"]
    PH["parser_helpers.rs<br/>ModuleTracker / LambdaIdGenerator"]
    D["dialect.rs:Dialect trait"]
    UD["dialect/ui.rs:UiDialect"]
    AST["ast.rs + ast/<br/>Code / Stmt / Expr"]
    MP["ast/module_path.rs<br/>ModulePath / PathPrefix"]
    SER["ast.rs<br/>ToNode / ToAtom / AtomWriter"]
  end

  subgraph 共享状态
    TS["types.rs:TypeStore<br/>（parser/typeck/codegen 共用）"]
    IC["infer/:InferenceContext"]
  end

  subgraph 外部依赖方
    RES["resolver.rs<br/>ModuleResolver / FilesystemResolver"]
    BE["后端：evaluator / AutoVM / trans(c,rust)"]
    MAC["auto-lang-macros crate<br/>value!/atom!/node!（经 AtomReader 复用 parser）"]
  end

  SRC --> US
  US --> RES
  SRC --> LEX --> TOK --> P
  PH -.辅助.- P
  D <-.实现.- UD
  P -->|语句位置关键字派发| D
  P --> AST
  P <--> TS
  P <--> IC
  AST --> MP --> RES
  AST --> SER
  AST --> BE
  MAC -.复用.- P
```

说明：方言解析产物仍是基础 `Stmt` 的合法变体，下游消费者（typeck/trans/vm）类型签名不变
（docs/design/dialect-extension-diagnosis.md §6.1）。`TypeStore` 是 parser 与各后端共享的
类型注册表（docs/design/01 §Core Components）。

## ADR 日志

### ADR-01: ToNode 与 ToAtom 双 trait 分离

- 日期：未标注（plan 归档于 docs/plans/old/）
- 来源：plan-003、plan-004
- 决策：新增 `ToNode` trait 直接返回 `Node`；`ToAtom` 收窄为文本序列化并改返回 `AutoStr`。
  节点产出型 AST 的 `to_atom()` 委托为 `Value::Node(self.to_node())`。
- 备选：保持单 `ToAtom` 返回 `Value`（pros：API 少一个 trait；cons：35 个实现中 32 个返回
  `Value::Node`，调用点被迫写 42 处 `.to_node().unwrap()`，类型层面无法区分"产出节点"与"产出标量"）。
- 后果：正面——消除全部 42 处 unwrap，"节点 vs 标量"在类型系统显式化；负面——AST 类型需同时
  维护两个 trait 实现。
- 状态：active

### ADR-02: AtomWriter 流式 S 表达式输出

- 日期：未标注（plan 归档于 docs/plans/old/）
- 来源：plan-005、plan-006
- 决策：为全部 AST 类型实现 `AtomWriter::write_atom(&self, f: &mut impl io::Write)`，
  输出 Lisp 风格 S 表达式（如 `(fn name=add params=(params ...) return=int body=(body ...))`）。
- 备选：先构造 `Node` 树再整体 to_string（pros：复用树结构；cons：中间字符串分配多，
  且 Display 顺序受 Node 存储影响）。
- 后果：正面——流式写出无中间分配，配合 `ToAtomStr` blanket impl 获得缓存字符串；
  负面——plan-006 揭示了手写期望与实现的 7 类格式偏差，结构体构造器判定最终靠
  "首字母大写"启发式（TypeDecl 不在作用域内），属于已知脆弱点。
- 状态：active

### ADR-03: Node/Obj 存储从 BTreeMap+Vec 迁移到 IndexMap

- 日期：2025-01-10（plan-012 文件内标注 COMPLETED）
- 来源：plan-012
- 决策：`NodeBody`/`Obj` 改用 `IndexMap<ValueKey, NodeItem>`，删除冗余的
  `index: Vec<ValueKey>` 同步字段。
- 备选：保留 BTreeMap+Vec 双结构（pros：有序遍历稳定；cons：O(log n) 查找、双份存储需手工
  同步、显示顺序是排序序而非插入序）；选 IndexMap 因其 O(1) 平均查找且保插入序
  （rustc/tokio/serde 同款）。
- 后果：正面——>100 项结构查找提速 2–10x、内存降约 20–30%、349 个测试通过；负面——Display
  与序列化顺序从排序序变为插入序，依赖排序输出的代码需显式 `.sorted()`。
- 状态：active

### ADR-04: Atom 宏 DSL 用 proc-macro + AutoLang parser，而非 macro_rules

- 日期：未标注（plan-016 标注"设计变更说明"）
- 来源：plan-016
- 决策：`value!`/`atom!`/`node!` 实现为独立 `auto-lang-macros` crate 的过程宏：把 TokenStream
  转字符串后经 `AtomReader` 走 AutoLang parser 解析；`#{var}` 插值经 `ToAutoValue` trait 转换。
- 备选：`macro_rules!` 声明宏（pros：编译期展开、无运行时解析；cons：需为每种语法手写规则，
  无法自动覆盖 AutoLang 全语法）。
- 后果：正面——parser 支持的任何语法宏内直接可用，构造代码减少 60–70%（三层 API 合计）；
  负面——宏在运行时解析字符串，慢于直接构造（plan-report 01 §Open Questions 记录尚未基准测试）。
- 状态：active

### ADR-05: Parser 去除 Universe 依赖

- 日期：未标注（plan-090 标注 ✅ 完成）
- 来源：plan-090
- 决策：Parser 的符号定义/查找迁入 `TypeStore` + `InferenceContext`，模块路径追踪与 lambda
  ID 生成迁入新建的 `parser_helpers.rs`（`ModuleTracker`/`LambdaIdGenerator`）；`Parser.import()`
  删除，import 由 plan-085（AIE + AutoCache）接管。
- 备选：继续经 Universe 管符号（pros：不改调用面；cons：编译期数据与运行时解释器状态耦合，
  阻碍增量编译分离）。
- 后果：正面——frontend 与运行时解耦，为 AIE 增量编译铺路；负面——parser 构造需要
  `TypeStore`（故常用入口是 `Parser::new_with_type_store`）。
- 状态：active

### ADR-06: Dialect trait 正式化方言派发（轴 A）

- 日期：2026-07-04（诊断文档最后提交）
- 来源：docs/design/dialect-extension-diagnosis.md §6.1
- 决策：把"哪些关键字归我管 + 看到关键字怎么解析"抽象为 `Dialect` trait（`matches`/
  `keywords`/`try_parse_stmt`，另加 `try_parse_token_stmt` 接管真实 TokenKind），
  parser 构造时按 session 场景装配方言表；`UiDialect` 在 `Scenario::UI` 下接管
  `widget`/`msg`/`model`（Ident 路径）与 `view`/`on`（TokenKind 路径）。
- 备选：继续把 UI 关键字硬编码进核心 parser（pros：无间接层；cons：派发逻辑散落、基础
  `Stmt` 随方言膨胀，诊断文档 §1.1/§1.2）。
- 后果：正面——核心 parser 无需改动即可注册新方言，方言产物仍是基础 `Stmt` 变体，
  下游签名不变；负面——派发需 `mem::take` 移出方言表规避自引用借用（实现上的已知折衷）。
- 状态：active（PR-1 基建与 PR-2 UI 迁移均已落地，见 parser.rs:188 注释与 dialect/ui.rs）

### ADR-07: 模块路径语法 super/pac + ModuleResolver trait

- 日期：2025-03-18（plan-131 实现状态表标注 Completed）
- 来源：plan-131、plan-078
- 决策：`use` 路径支持 `super`（多级父目录）与 `pac`（包根搜索）前缀，AST 侧落为
  `ModulePath { prefix: PathPrefix, segments }`；解析策略抽象为 `ModuleResolver` trait，
  默认实现 `FilesystemResolver`，包管理器/远程注册表可实现同名 trait 接入。
- 备选：固定文件系统相对路径解析（pros：简单；cons：无法支持包根导入与未来的 registry
  解析）；`name.at` 与 `name/mod.at` 并存时报歧义错误（docs/design/10 §Code Organization）。
- 后果：正面——VM 可把模块解析委托给外部实现（plan-078 Stage 2 初衷）；负面——`super`
  到包根会报错并提示改用 `pac.`（resolver.rs 中的显式报错路径）。
- 状态：active

### ADR-08: Auto Flow 采用点链而非 pipe 运算符

- 日期：未标注
- 来源：docs/design/10-language-syntax.md §Auto Flow
- 决策：函数式链式调用走 `.map().filter()!` 点链（Rust/Java 风格），`!` 后缀做物化；
  不引入 `|>` pipe 运算符。
- 备选：pipe 运算符（pros：数据流向直观；cons：`.` 后 IDE 自动补全不自然、与结构体方法
  调用风格不一致、符号集更大）。
- 后果：正面——语法面收敛于"点"一种后缀机制，与统一点表示法（§Unified Dot Notation）一致；
  负面——Auto Flow 本体（`Iter<T>` spec、惰性适配器）至今未实现，该决策尚无落地代码检验。
- 状态：active（设计层；实现 planned）
