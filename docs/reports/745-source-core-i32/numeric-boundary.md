# PLAN-745 T-01 数值边界核查报告（probe-int-width / probe-char-int）

- 计划：PLAN-745 r1 · T-01（work 阶段，AC-01/AC-07 证据）
- 工作树：`D:/autostack/.wt/lang-745/auto-lang` @ `plan-745-dev`
- 基线 HEAD：`eea7a80618d4d5e75749d666b7609a9a87925bbd`（= v0.6-dev / 745 专属树登记 HEAD；计划 inputs.json 23 项输入 hash 复核零漂移）
- 日期：2026-10-06
- 性质：有界事实核查（plan §10：不得用 unknown 扩大语言面；不以旧 VM 溢出行为作本 profile oracle）。
  本报告只做静态代码事实取证 + profile 决策收敛；数值 cases 的**运行证据**由 T-05
  `verify-ac-source-745.ps1` 物化执行后回填（见 §7）。

## 1. 输入版本（证据绑定）

| # | 证据文件 | SHA256（前 16） | 关键行 |
|---|---|---|---|
| E1 | `crates/auto-lang/src/vm/engine.rs` | 6fe1b2e152b04278 | 56–61（StackTag） |
| E2 | `crates/auto-val/src/value.rs` | fd809b1373c600f3 | 82–97（ValueData） |
| E3 | `crates/auto-lang/src/ast.rs` | 895cd6fdfdc8cdab | 339–348（Expr 字面量族） |
| E4 | `crates/auto-lang/src/parser.rs` | 7ed974fda56a8a8b | 3638–3682（parse_int） |
| E5 | `crates/auto-lang/src/trans/rust.rs` | 822a2cae792bbf6d | 1739–1749（rust_type_name） |
| E6 | `crates/auto-lang/src/libs/string.rs` | c4db1bf28ce3f347 | 863–891（str_char_at） |
| E7 | `crates/auto-lang/src/lexer.rs` | f4646df489edafb4 | 1406–1409（number() 入口）、282+（number()） |
| E8 | `auto/lib/lexer.at` | c92bcce8c6432ffd | 44–48（cur_char）、52（'0'..'9' 比较） |
| E9 | `experimental/ac-core/src/atom_text.rs` | 0d468f329a293078 | 295–321（IntLit::to_i32） |
| E10 | `experimental/ac-core/src/hir.rs` | 251d973f75a99167 | 216–219（Constant{value:i32}） |
| E11 | `experimental/ac-core/src/native.rs` | 69ef4577b3d2089b | 34–38（TRAP_EXIT_CODE=70）、627–669（checked_add/mul） |
| E12 | `experimental/ac-core/schema/core-i32.atom` | a23e38fb605d503d | 13–16、76–90（Constant I32 / BinOp / overflow） |

E9–E12 与计划 inputs.json 登记值一致（本次漂移复核 0/23）。E8 与 MD-408
`bound_input_hashes` 登记值一致（MD-408 证据未过期）。所有行号基于上表 hash 的文件内容。

## 2. 事实链 A：可见值（value repr）

- **VM 执行栈**（E1:56–61）：`StackTag { Int(i32), Str(u32) }`，注释明确
  “ints use i32 encoding”——VM 栈上整数为 32 位编码。
- **VM 值存储**（E2:82–97）：`ValueData { Byte, Int(i32), Uint(u32), USize(usize),
  I8, U8, I64(i64), Float, Double, Bool, Char(char), Nil, Str, …, Range(i32,i32) }`
  ——i32 与 i64 并存，另有 usize/byte 窄宽族；`Range` 端点也是 i32。
- **AC 基线**（E9/E10/E12）：741 core-i32 的 `Constant{value: i32}`、Schema
  `value: I32`、`IntLit::to_i32` 用 u128 幅值 + `i32::MIN/MAX` 域检查（负号单侧
  允许到 2147483648），越界在 bind 层 `bind.literal-range` 拒绝。

## 3. 事实链 B：parser（旧前端字面量分流）

- **旧 lexer**（E7:1406–1409）：数字 token 仅从数字字符起始（`c.is_digit(10)` →
  `number()`）；负号是独立 token，旧表面不存在“带符号整数字面量”这一 token。
- **旧 parser**（E4:3638–3682）`parse_int`：十进制先按 `i64` 解析，
  `val > i32::MAX` → `Expr::I64(val)`，否则 `Expr::Int(val as i32)`；i64 也装不下
  → 回退 `u64` → `Expr::U64`。十六进制/二进制同构。即旧 AST 的“整数”是
  **按幅值分流的 i32|i64|u64 三态**，不是单一宽度。
- **旧 AST**（E3:339–348）：`Expr::{Int(i32), Uint(u32), I8, U8, I64(i64), U64(u64),
  Byte(u8), Float(f64,_), Double(f64,_), Bool, Char(char), Str, …}`——AST 名下
  整数宽度族远超 i32；`Expr::Char` 是宿主 `char`（Unicode scalar）。

## 4. 事实链 C：trans（转译目标宽度）

- **trans/rust.rs**（E5:1739–1749）`rust_type_name`：`Type::Int => "i64"`
  （注释：Plan 399 Phase 11.1，与 HTTP 后端 types.rs 统一为 i64）；
  `Type::Char => "char"`。即 Auto 源 `int` 在既有转译目标上固定映射 **i64**。

## 5. 事实链 D：char 路径（三宿主表示并存）

- **VM builtin**（E6:863–891）：`str_char_at(s, i)` 索引参数必须是
  `Value::Int(i32)`，**返回单字符字符串**（`Value::Str(chars[i].to_string())`），
  文档注释 `// returns "e"`。
- **auto-lib（aavm 表面）**（E8:44–48）：`fn cur_char(source str, p int) int {
  return source.char_at(p) }`——`char_at` 返回 **int 码点**；随后 `c >= '0' && c <= '9'`
  （E8:52）把 char 字面量当整数比较。
- **VM 值存储**（E2）：`ValueData::Char(char)`（Rust Unicode scalar）。

同一“取字符”操作在旧 VM builtin（单字符 str）、auto-lib（int 码点）、值存储
（Rust char）三种宿主表示间不一致；不存在单一 char↔int 映射可作为 oracle。

## 6. probe 决策报告

### 6.1 probe-int-width（MD-409 unknown → profile 裁决）

**决策**：在 `auto.source.core-i32.draft` revision 1 内，源码 `int`/`i32` 注解与
无后缀十进制整数字面量 = **精确有符号 32 位**，域 `[-2147483648, 2147483647]`：

- 越界字面量（宽临时 u128 幅值 + 双侧 range 检查）→ `text/source.literal-range`
  拒绝（N01 `2147483648`、N02 `-2147483649`）；禁截断/回绕。
- 负号仅允许直接后随数字：`-2147483648` 与 `- 2147483648`（P20）是**单个有符号
  literal**；`-x`、`-(expr)`、二元减法（N23/N36）拒绝。这是 profile 规则，与旧表面
  “负号独立 token + 幅值分流”不同，不宣称旧语言行为。
- `add_i32`/`mul_i32` 运行期 signed 溢出 → `ExitProcess(70)`（E11）；常量表达式
  不做编译期求值/拒绝（P12 未走分支的 `MAX+1` 必须可编译）。
- 其余宽度/形态一律显式拒绝：`i64/u32/u64/uint/usize` 注解（N22）、带后缀
  （N37 `1i32`）、进制前缀（N38 `0x10`）、浮点（N47 `1.5`）。

**决策依据**（输入版本见 §1）：现有表面 int 宽度**不一致且非单一**——VM 栈 i32
（E1）、VM 值存储 i32|i64 并存（E2）、旧 AST 按幅值 i32|i64|u64 三态（E3/E4）、
trans 目标 i64（E5）。因此“全语言 int=native 宽度”无法在任何单一映射下成立，
MD-409 的 unknown 不能收敛为全语言结论；本决策仅在 profile 边界内生效，
741 AC 基线（E9–E12）已是精确 i32 且与本规则同构，可行性**未被任何事实推翻**。

### 6.2 probe-char-int（MD-408 unknown → profile 裁决）

**决策**：`auto.source.core-i32.draft` **不引入 char**：char 字面量 `'a'`（N20）、
char 注解与任何 char↔int 转换均 `text/source.unsupported` 显式拒绝；char 不转整数。

**决策依据**（输入版本见 §1）：char 取值路径三宿主表示并存——旧 VM builtin
`str_char_at` 返回单字符字符串（E6）、auto-lib `char_at` 返回 int 码点且 char 字面量
按整数比较（E8）、值存储为 Rust `char`（E2）。任何单一映射都会与至少一个宿主表面
冲突，故 MD-408 的 char↔int 细则维持 unknown，仅收敛为“本 profile 无 char”；
完整 char 语义（含 81 例 lexer 语料，MD-408 证据）保留 open，owner 不变。

### 6.3 可行性结论

无事实推翻 r1 合同（plan §8 T-01 修复条件未触发）：741 Checked→native 管线
（i32 精确域、trap=70、bool=i32 0/1 ABI）与 §5.1 数值映射表完全相容，
无需修合同，T-02/T-03 按原合同继续。

## 7. 数值 cases（AC-01 运行证据计划，T-05 物化执行）

以下 case 由 `verify-ac-source-745.ps1` 真实构建+运行 PE 取证（60s 硬截止），
oracle 源自 [corpus.json](../../plans/attachments/745/corpus.json) 与 plan §6 补充矩阵：

| case | 源码要点 | oracle |
|---|---|---|
| P11 | `-2147483648` 与 `2147483647` 字面量、`x < y` | exit 0；HIR literal_values=[-2147483648, 2147483647] |
| P20 | 中文注释 + CRLF + `- 2147483648`（负号空格分隔） | exit 0；token/行列指原 .at |
| P13 | `return 2147483647 + 1` | exit 70（add_i32 trap，非编译期拒绝） |
| P14 | `return 2147483647 * 2` | exit 70（mul_i32 trap） |
| P12 | `if false { return 2147483647 + 1 }` | build 成功、exit 0（未走分支溢出不 hoist，X1） |
| P16 | `mark_a()` 后 `MAX+1` | exit 70、stderr 精确 `a`（可观察效果先于 trap，X9） |
| P02 | `add(-2, 3)` | exit 1（signed 语义） |
| N01/N02 | `2147483648` / `-2147483649` | text/source.literal-range，token=字面量 |
| N22/N37/N38/N47 | i64 注解 / 1i32 / 0x10 / 1.5 | text/source.unsupported |
| N20 | `'a'` | text/source.unsupported（char 拒绝，probe-char-int） |
| P18 | `below(2)` bool 返回 + if 分支 | exit 4（bool 与 i32 无隐式互转） |

退出码 70 与普通 `return 70` 同码的区分以 HIR overflow 结构 + 前驱 trace（P16）
作证据，不依赖 PowerShell signed 退出码展示（plan §5.1）。

## 8. MD-408 / MD-409 提案（proposed，T-06 落 proposed-spec-delta.md，merge 绑定）

对 `docs/reports/743-acc-hir-contract/manual-decisions.json` 两个条目的 note 追加
（不改变 conclusion/证据绑定，维持“unknown 不冒称 implemented”）：

- **MD-408**（cap:char）追加：
  `〔745 T-01/probe-char-int：source profile auto.source.core-i32.draft 拒绝 char（字面量/注解/转换均 source.unsupported，N20）；三宿主表示并存取证（VM builtin 单字符 str：string.rs:863-891；auto-lib char_at→int 码点：lexer.at:44-52；Value Char(char)：value.rs:82-97），char↔int unknown 维持，完整 char 语义保留 open。证据：docs/reports/745-source-core-i32/numeric-boundary.md §5/§6.2。〕`
- **MD-409**（cap:int-ops）追加：
  `〔745 T-01/probe-int-width：source profile 内 int/i32=精确有符号 32 位（[-2147483648,2147483647]，越界 source.literal-range；运行期溢出 trap=70；负号仅直接后随数字）。全语言宽度仍 unknown：VM 栈 i32、值存储 i32|i64、旧 AST 按幅值 i32|i64|u64、trans 目标 i64 并存。本 profile 仅 add/mul/lt（候选②未变）。证据：docs/reports/745-source-core-i32/numeric-boundary.md §2-§4/§6.1。〕`

## 9. 决策来源汇总

1. PLAN-745 r1 合同 §5.1 数值映射表、§4.2 约束依据（本次全部逐条对码核实）。
2. 743 归档计划 §10.2（最小探针 + unknown 不得当 implemented）与
   next-work-packages 候选①前置能力条目。
3. MD-408/409 现行 note 与 bound_input_hashes（E8 hash 复核一致）。
4. §1 证据文件 12 项（hash 绑定；ac-core 四项与计划 inputs.json 同源零漂移）。
5. corpus.json 数值相关 case 的待执行 oracle（§7，T-05 回填运行收据）。
