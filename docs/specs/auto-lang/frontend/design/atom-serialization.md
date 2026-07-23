# AST 序列化体系（ToNode / ToAtom / AtomWriter）

## 范围

AST → Atom 文本/树结构的序列化层：trait 三件套、S 表达式格式、三层构造 API、
proc-macro DSL。对应 plan-report 01 的 plans 002–006、015、016。

## 原则

- **节点与标量在类型层面分离**：`ToNode` 返回 `Node`（树结构），`ToAtom` 返回 `AutoStr`
  （文本）。节点产出型类型的 `to_atom()` 委托为 `Value::Node(self.to_node())`（plan-003/004）。
- **流式输出优先**：`AtomWriter::write_atom(&self, f: &mut impl io::Write)` 直接写 io，
  不产生中间字符串；`ToAtomStr` 以 blanket impl 给所有 `AtomWriter` 类型提供
  `to_atom_str()`（plan-005）。
- **构造 API 分层**：链式 `with_*`（简单静态）→ Builder（条件/运行时）→ 宏 DSL
  （声明式），按场景选用，三层合计减少构造代码 60–70%（plan-015/016）。

## 细节

- 输出格式为 Lisp 风格 S 表达式：`(if (branch cond body) (else else-body))`、
  `(fn name=add params=(params ...) return=int body=(body ...))`。
- AST → Atom 映射惯例：字面量 `int(42)`、二元运算 `bina(op, left, right)`、
  复杂语句为带属性与子的节点（plan-002）。
- `ToAutoValue` trait（auto-val/src/to_value.rs，~102 行）把 Rust 原生类型转为
  AutoLang 值，支撑宏内 `#{var}` 插值；无插值时宏直接把原始字符串交给 `AtomReader`，
  跳过逐 token 处理（plan-016）。
- 宏实现在独立 `auto-lang-macros` crate，经主 crate re-export；用 AutoLang parser
  解析宏体，因此 parser 支持的语法在宏里自动可用（plan-016 的设计变更）。
- 已知脆弱点：结构体构造器的 `call` vs `node` 判定在方法解析期拿不到 TypeDecl，
  回退为"首字母大写"启发式（plan-006）。

## 显式非目标

- 不负责 AST 的语义分析或类型检查——序列化是纯结构映射。
- 宏 DSL 不追求编译期展开性能：proc-macro 运行时解析字符串，热路径性能未基准测试
  （plan-report 01 §Open Questions）。
- Node 存储的进一步统一（args/props 合一 plan-013、body/nodes/kids 合一 plan-014）
  仍是 planned，不在本体系已交付范围内。

> 来源: docs/plan-reports/01-ast-core.md（plans 002–006、015、016）；代码核对 ast.rs:95-137（ToAtom/AtomWriter/ToAtomStr/ToNode）
