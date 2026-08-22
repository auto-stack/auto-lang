# LSP API Contract — auto-lsp

> **Plan 416 5-C**:固化 auto-lsp 实现的 LSP 方法与能力矩阵。本文件是
> 客户端(VSCode 扩展等)与服务端之间的契约快照——新增/变更能力须同步
> 更新此表。生成依据:`crates/auto-lsp/src/backend.rs`(initialize 能力
> 声明与 handler 实现)+ 各功能模块。

## 1. 能力矩阵(initialize 响应)

| LSP 能力 | 声明 | 实现位置 | 备注 |
|---|---|---|---|
| textDocument/completion | ✅(trigger: `.`, `:` 与隐式) | `completion.rs` | 上下文分派见 §2 |
| textDocument/hover | ✅ | `hover_info.rs` | 符号签名 + 文档 |
| textDocument/definition | ✅ | `goto_def.rs` | |
| textDocument/references | ✅ | `workspace.rs` | Plan 243 Phase 1 |
| textDocument/rename | ✅ | `workspace.rs` | Plan 243 Phase 2,跨文件 |
| textDocument/codeAction | ✅ | `backend.rs` | Plan 243 Phase 3 |
| textDocument/signatureHelp | ✅(trigger: `(`, `,`) | `signature_help.rs` | Plan 243 Phase 4 |
| textDocument/inlayHint | ✅ | `inlay_hints.rs` | Plan 243 Phase 4(类型/参数提示) |
| textDocument/documentSymbol | ✅ | `backend.rs` | 大纲视图 |
| workspace/symbol | ✅ | `backend.rs` | |
| **semanticTokens/full** | ✅ | `semantic_tokens.rs` | Plan 416 5-B(legend 见 §2;VSCode 着色核验待实机 F5) |

## 2. 补全数据源(Plan 416 5-C)

| 上下文 | 数据源 |
|---|---|
| 类型位置(`x: ` 之后) | 内置类型表 + 用户 `type` 声明(AST)+ **stdlib 模块名**(`stdlib_index`) |
| 成员访问(`obj.` 触发) | 变量类型推断 → 用户类型的字段/方法;**stdlib 模块成员**(`json.` → 该模块全部 pub fn,`stdlib_index`) |
| 函数名上下文(`fn ` 之后) | 内置函数模板 |
| 变量上下文 | 作用域内局部变量/参数 |
| 默认/关键字上下文 | 精选关键字(带 snippet)+ **lexer 权威关键字表补全**(`Token::all_keywords`,单一事实源) |

### 2.1 semantic tokens 图例(顺序即线上编码索引)

`keyword, type, function, variable, parameter, string, number, comment` ——
词法扫描(注释/字符串/数字/关键字)+ AST 符号分类(fn→function、参数→
parameter、局部→variable、type/enum/spec→type);未分类标识符回退启发式
(`ident(` → 调用、大写开头 → 类型)。modifiers v1 恒 0。

stdlib 索引(`stdlib_index.rs`)在首次使用时惰性解析 `stdlib/auto/<mod>.at`
(经 `auto_lang::util::find_std_lib()` 定位;只索引单扩展名的声明源文件,
`mod.c.at`/`mod.vm.at` 等后端副本不重复计入)。stdlib 缺失时全部降级为空
列表,不影响其余能力。

## 3. 传输与进程约定

- stdio 传输;服务端二进制经扩展 `bin/<platform>/auto-lsp[-wrapper]` 启动。
- 文档同步:incremental(位置换算见 `position.rs`)。
- 配置节:`autoLang`(`enableLSP` / `lspPath`)。

## 4. 已知边界

- 查找/重命名基于符号索引,宏生成代码内的符号不可见。
- completion 的 panic 被捕获并降级为空列表(见 `completion.rs::complete`)。
- 诊断为字符级 span(parser miette 输出换算)。
