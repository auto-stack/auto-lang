# AAVM lib 模块化映射图（use 化重组调研）

> 2026-09-01 | Plan 514 W0 调研产物。**D5 翻案**：511 W0 的"聚合定案"
> （midlang-w0-archaeology.md §5）把"当下 99_unit 引用不可用"当成了终局
> 方案；用户裁定 lib 应当用上自身的中阶能力（`use` 模块化），本图为其
> 事实基础。分析脚本：`scripts/aavm_lib_xref.py`（顶层定义提取 + 跨文件
> 词边界引用扫描，剔除行注释；可重跑校验）。

## 1. 依赖树（实测，无环化后）

```text
token.at      （0 依赖；3 定义）
  ↑
lexer.at      → token（TokenKind, keyword_kind, kind_name；15 定义）
  ↑
parser.at     → token（TokenKind, kind_name）, lexer（Token, tokenize；73 定义）
  ↑                    ┌──────────────────────────────┐
typeinfo.at   → token（TokenKind）, lexer（tokenize）, parser（29 符号；19 定义）
  ↑                    │
codegen.at    → token, lexer, parser（25 符号）, typeinfo（t_is_type_prop；78 定义）
  ↑                    │
engine.at     → codegen（CG, OpCode, cg_compile, cg_compile_files, op_name；20 定义）
                       │  ※ 纯解释器层——不依赖 parser/typeinfo/lexer
a2r.at        → token, lexer, parser（25 符号）, typeinfo（t_is_type_prop；108 定义）
                       ※ 不依赖 codegen/engine
```

- **唯一环边**：`p_peek_text` 定义于 codegen.at:355，被 parser.at:1580
  引用（拼接模式下符号链接期全程序解析故可行；use 模块化后成环）。
  **处置：迁移至 parser.at**（P 游标取第 n token 文本，与 `p_peek` 同族，
  纯函数零风险）。
- 跨文件**重名定义：零**（拼接今日无遮蔽，pub 化无撞名风险）。
- 分层意外之喜：engine 是纯解释器（仅依赖 codegen 的指令载体与编译入口），
  a2r 完全独立于 codegen/engine——DAG 干净，无需更结构性的重组。

## 2. pub 导出面（被 ≥1 其他文件引用的符号，脚本实测）

| 文件 | pub 数 | 符号 |
|---|---|---|
| token.at | 3 | `TokenKind` `keyword_kind` `kind_name` |
| lexer.at | 2 | `Token` `tokenize` |
| parser.at | 38 | `E` `Op` `P` `builtin_type` `expect_eos` `expr_pratt` `infix_l` `infix_r` `int_val` `is_comment_kind` `is_name_kind` `is_op_kind` `is_pattern` `is_soft_ident_kind` `is_type_name` `is_unsupported_stmt_kind` `op_display` `p_bind` `p_decl_lookup` `p_err` `p_expect` `p_kind` `p_line` `p_lookup` `p_next` `p_op` `p_peek` `p_peek_text`(迁移后) `p_text` `parse_args` `parse_enum_decl` `parse_new` `parse_type` `parse_type_decl` `pop_scope` `prefix_power` `push_scope` `skip_empty_lines` `split_commas` |
| typeinfo.at | 1 | `t_is_type_prop` |
| codegen.at | 6 | `CG` `OpCode` `cg_compile` `cg_compile_files` `op_name`（`p_peek_text` 迁出后不再导出） |
| engine.at | 0+入口 | 跨文件零引用；**对外入口须 pub**：`ev_run` `ev_run_files` |
| a2r.at | 0+入口 | 跨文件零引用；**对外入口须 pub**：`ar_run`（+`aa2r_transpile_merge` 若入口需要） |

> engine/a2r 的跨文件引用为零是拼接模式的产物（没有别的文件需要它们）；
> 模块化后它们是**顶层入口模块**，入口 pub 是外部（aavm.at/闸门/矩阵）使
> 用的唯一通道。dump 族入口（`lex_dump`/`parse_dump`/`typecheck_dump`/
> `codegen_dump`）按需 pub（缺省一并 pub，语义上是模块 API）。

## 3. use 语句形态（点路径 stdlib 根解析）

宿主实证形态：`use auto.greet_mod: greet, add`（test/vm/17_modules/001_use_fn，
`auto/` 为 stdlib 根）。lib 文件位于 `auto/lib/`，故各文件头部形如：

```auto
// lexer.at
use auto.lib.token: TokenKind, keyword_kind, kind_name

// typeinfo.at
use auto.lib.token: TokenKind
use auto.lib.lexer: tokenize
use auto.lib.parser: Op, P, expr_pratt, ... (29 符号 → 定向或通配 use auto.lib.parser: *)
```

定向 vs 通配（`use auto.lib.parser: *`）：38 符号的定向清单冗长但显式；
通配简洁但削弱可读性。**缺省定向**（一对一风格延续），parser 这种大面可
在执行期裁定通配（待澄清）。

## 4. 前置与迁移约束（塔式）

1. **AA2R use 发射缺失是硬前置**（a2r.at:26 Missing 清单在案）：lib 文件
   一旦出现 use，`auto build`（AA2R 转译 lib→Rust）与矩阵⑤腿立断。
   → Plan 514 W2 增 use 发射项，**先于** lib 模块化（W3）。
2. **主 a2r 对 lib 内 use 语句的转译**（矩阵②腿）：W0 考古确认（推断可
   发 Rust use/限定名，未见反证；trans/rust.rs 有 use 符号提取位）。
3. **拼接式消费者双轨兼容**：M1–M5 harness（AUTO_LIB_FILES_V2 拼接）、
   `gen-aavm2-unit.py`（99_unit 聚合）、parity ②⑤腿转译输入——拼接产物
   中部出现 use 行会触发对被引文件的重复加载/重复定义。**缺省方案：双轨
   剥离**——拼接侧统一剥除 `use auto.lib.*` 行（行为与今日等价：符号仍在
   同一程序内），模块路径（`auto run` 入口/`auto build`）走真 use 解析。
   生成器加 `--check` 同步校验。
4. **auto test session 不播种源目录**（D5 发现②）：对 lib 内部互引无碍
   （stdlib 根 `auto.` 可达——lib 就在 auto/ 下）；对仓库根的测试目录引用
   lib 仍不可达 → 99_unit 维持聚合双轨（不强行 use 化测试件）。
5. 行为不变判据：模块化全程闸门（M1–M5/compile 腿/矩阵①–⑤/99_unit）+
   `auto run` 入口冒烟全绿；红即回退。

## 5. 目标形态

```text
auto/lib/            七模块 DAG（token ← lexer ← parser ← {typeinfo ← codegen ← engine}, a2r）
auto/aavm.at         生成的 CLI 入口：聚合 lib 前置剥离版 或
                     use auto.lib.engine 形态（W3 执行期按 4.3 实测定）；
                     fn main: auto.process.args → ev_run_files(path) → print
                     无参时内置 corpus 冒烟
```

lib 由此成为 aavm 自身 `use` 能力的第一个真实多模块程序（自举本味），
`auto run auto/aavm.at <目标.at>` 即上一问缺的"单文件启动入口"。
