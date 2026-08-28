# AAVM v2 文件级映射表（plan-431 Phase B）

基线 `b3bd64f5`；行数取基线快照（429-C1 表）。topo 顺序即移植顺序（B2）：

| # | Rust 文件（基线行数） | .at 文件（auto/lib/） | 优先级 | 备注 |
|---|---|---|---|---|
| 1 | src/token.rs (516) | token.at | P0 | 全量直译（~140 TokenKind） |
| 2 | src/error.rs + pos (1,299) | error.at | P0 | 真实行列追踪（v1 Pos.line 恒 1 的教训） |
| 3 | src/lexer.rs (2,029) | lexer.at | P0 | 极稳定（3 个月 5 commit），近乎照抄结构 |
| 4 | src/ast.rs (1,734) + ast/ 核心子集 | ast.at | P1 | 按 boundary §A2 裁剪（8 个 UI 文件剔除：cover/grid/on/route/store/tag/task/ui） |
| 5 | src/parser.rs 核心区（~14.7k/17.7k） | parser.at | P1 | Pratt 优先级表直译；函数清单见 data/parser_fns.csv |
| 6 | src/types.rs (722) + infer/ 核心 | typeinfo.at | P2 | 单一 TypeStore，不复刻历史 5 套 registry（429 调研） |
| 7 | src/vm/opcode.rs (925) | opcode.at | P2 | 全量 194 条声明（编号一致） |
| 8 | src/vm/codegen.rs 核心 | codegen.at | P2 | UI 分支 BOUNDARY-OUT 标记 |
| 9 | src/vm/engine.rs 核心 | engine.at | P2 | 栈机 + 调度最小核 |
| 10 | src/vm/native_catalog.rs 子集 (455/521) | natives.at | P2 | X-macro 模式保留 |

**ast/ 33 文件处置**：剔除 8（cover/grid/on/route/store/tag/task/ui.rs）+
node.rs 中对应节点；其余 25 文件（alias/atom_helpers/body/branch/call/comptime/
dep_/enums/ext/for_/fstr/fun/hold/if_/is/module_path/parsers/range/spec/try_/
type_alias/types/union/use_）随 ast.at P1 移植。

**infer/ 10 文件处置**：剔除 task_types.rs；其余 9 随 typeinfo.at P2。

## topo 依赖（B2，P0/P1 细化）

```
token.at ──► lexer.at ──► ast.at ──► parser.at
                ▲            │
                └── error.at ◄┘（token/lexer/parser 都引用 Pos/Error;
                                  error.at 反向引用 TokenKind 做错误上下文——
                                  v2 以解耦的前向声明处理,移植时记录 divergence）
```

- token.at 无依赖（叶子，第一个移植）；
- error.at 依赖 token（TokenKind 显示）;Pos 结构独立可先行;
- lexer.at 依赖 token + error;
- ast.at 依赖 token（字面量/token 引用）;
- parser.at 依赖前三者全部 + ast;
- P2 链 opcode ← typeinfo ← parser;codegen ← opcode+typeinfo;engine ← 全部。

**登记规则**：每完成一个文件,把它加进 `crates/auto-lang/src/lib.rs` 的
`AUTO_LIB_FILES_V2`(单一事实源,plan-431 E2)——runner 会按序前置拼接。
