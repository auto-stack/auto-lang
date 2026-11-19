# AV 语言能力与来源映射（AV language parity ledger）

> Status: **delivered（PLAN-1078 AVP-00 / PLAN-1079 AVP-01，2026-11-19）**。本文件由 AVP-00 首次建立，
> 后续 AVP 批次（1079–1096）串行增量维护；它是当前 AV 语言**能力/来源映射**，不是第二
> 注册器——suite/固定组/发现仍由既有 TestCase/auto-case 机制各负其责。
> 只登记实际交付与真实运行事实；未交行保 pending/owner，不写 implemented。

## 1. 普通 AV 语言入口（AVP-00 交付）

- **SourceProfile::Auto**，profile 串 `auto.source.language.av/v1`，能力键
  `ac.source.language.av/v1`。首批语义 = aggregates 面（同一共享
  Schema/CST → 绑定 → Checked HIR → MIR → ABC → AV 管道；不新开第二
  parser/解释器）。后续 AVP 批次在 Auto 面增量扩充；冻结窄 profile 身份不变。
- **recipe `ac-av/language-source-run-v1`**（父侧 `RecipeFamily::LanguageSource`）：
  int 入口值 + 精确 stdout/stderr 双流 + 编译/运行失败；不转旧 VM。worker 链：
  `compile_language_source → lower_checked_closures → verify_mir → encode_abc →
  verify_abc_bytes → execute(+ConsoleCapture)`，core 身份 `av.closures/1`。
- **双流省略补零（user ruling）**：console 族 recipe（console-run-v1 /
  scheduler-source-run-v1 / language-source-run-v1）加载时把省略的 stdout/stderr
  独立补成预期零字节；显式流按原预期比较；省略流一旦实际输出一个字节（含换行）
  即判失败。其余检查族省略语义不变。
- **检查级 `#ignore`（AV-ONLY-1078-1096）**：非 AV 检查（AC/A2R/A2C/A2PY、
  旧 AVM 对照）在检查小节加 `#ignore` → 映射 `CheckSpec.skip`（reason=AV阶段
  暂缓非AV检查、issue=AV-ONLY-1078-1096、owner=PLAN-1078）；不 prepare/
  dispatch，不阻 AV 门；后端任务逐项移除。`#ignore` 不标整 Case/Auto 源。

## 2. 公共语言语料（feature corpus）

- 唯一活跃作者根：`crates/auto-lang/test/cases/language/`；公共总选择
  `crates/auto-lang/test/cases/language.suite.ad`。同一 Case 一份 Auto 源，
  AV/AC/A2R 各自追加目标检查与独立实际结果，不复制源码。
- 固定组 `case-av-language-parity` = 公共语料 **AV 视图**（`--check av`，
  按已注册目标/动作选择；legacy-reference 等旧对照只经显式 avm 选择，不重建旧引擎）。
- 迁移表 `language/migration/legacy-map.tsv`（数据映射，不作注册器）；旧义务
  保原 ID/条件/oracle，合并不删义务。

## 3. 本包 case → owner 映射（AVP-00，真实运行）

| Case ID | 源 | AV oracle | 实际结果 | owner |
|---|---|---|---|---|
| `av.parity.1078.baseline-value` | `fn main() int { return 2 + 3 * 4 }` | exit 14, 双流空 | PASS（14） | PLAN-1078 |
| `av.parity.1078.short-circuit` | `if false && … / if true \|\| …`（fail 不运行） | exit 42, 双流空 | PASS（42） | PLAN-1078 |
| `av.parity.1078.syntax-neighbor` | `fn main() int { return (40 + 2 }` | parse / `source.syntax` | PASS（diag） | PLAN-1078 |

- 非 AV 检查（baseline-value 的 ac-native 与 legacy-reference）带 `#ignore`
  （AV-ONLY-1078-1096），本轮不执行；ac-native 曾真实执行 PASS exit 14 的实证
  记录于 plan-artifacts（AV 阶段前的最小同源多端对照），后续 AC 任务打开后重跑。
- 旧义务承接：`legacy.aavm2.001_smoke`/`002_hello_compile` → research-hold
  （aavm2 族保 research/手动条件；语言面 overlap = console print + void main，
  不充 PASS）；L-04（i128/f32 后缀）/L-46（spread/set/when/match/global）无此形、
  L-45（NavCall）parser-only —— 记录适用状态，不实现。

## 4. 支持/拒绝边界（AVP-00 实际）

- 支持（首批=aggregates 面）：scalar 运算、短路、`fail` 语言错误、console 双流、
  parse 阶段 `source.syntax` 诊断（stage 映射 `text→parse`）。
- 未交/拒绝：task actor、脚本/Python 协议、泛型/闭包/生成器/async 等后续 AVP
  批次能力不因 Auto 面未覆盖而豁免；旧窄 profile 拒绝面不变。
- 平台：Windows 执行；Linux/WSL 腿按裁定归专用 Linux 环境，不判红、不充 PASS。

## 5. AVP-01（PLAN-1079）交付：标量、转换与文本

> 2026-11-19，普通 Auto 入口真机运行（`ac-av/language-source-run-v1`）。
> 只登记实际观察；未交行在 §5.3 具名并写 owner/关闭条件。

### 5.1 普通入口新增/修复的准入事实

- **affix-num 基数体切分**（`core/src/source/cst/project.rs::decode_typed_literal`）：
  `0x`/`0b` 前缀后的体按**目标基数**的数字位切分（16 进制 `a..f` 属体），
  十进制另收一个分点；"至少有一个数位"的门也按同一基数判定。因此
  `0xFF`、`0xF`、`0xFFu8`、`0b1010`、`1_000_000`、`40i64`、`42i32` 全部解码；
  此前 Auto 面报 `numeric literal ... carries no digits`（`source.literal-range`）
  或 `suffixed/radix/float numeric literal ... is not part of this source profile`。
- **i32 类型化常量渲染**（`adapter.rs`）：`EKind::TypedConst { vt: I32 }` 渲染
  `{value}i32`。缺这条臂时输出 `value: I32`，Atom bind 拒绝整个模块
  （`internal adapter error` / `constant value must be an integer value`），
  无后缀 affix-num（`1_000_000`、`0b1010`）因此不可用。
- **16 进制体与后缀的歧义裁定**：基数体取**最长**的该基数数字位串，因此
  `0x1Fb` 是十六进制数 507（不是 `0x1F` + `b` byte 后缀）—— 后缀在
  十六进制数字后不可用，十六进制 byte 用 `0xFFu8` 或 `0xFF.as(u8)` 表达；
  十进制/二进制下 //d 后缀不受影响。旧解码器对该形无裁定（直接报错），
  本批按此规则冻结并由 plan1079_language_semantics.rs 精确断言。
- 结论：Auto 面的字面量面（宽度/后缀/基数/分隔符）与 Numeric 面同一解码器，
  不是第二套解析。

### 5.2 本包 canonical Case 与真实结果

| Case ID | 作者文件 | 独立 oracle | 实际结果 |
|---|---|---|---|
| `av.parity.1079.wide-exact` | `language/numbers/literals-conversions.case.ad` | exit 42（i64 超 2^53 精确、`0xFFu8`、`0b1010u8`、`1_000_000`、i64::MIN 加减） | PASS |
| `av.parity.1079.mixed-width-explicit` | 同上 | exit 42（`to(int)`/`as(i64)`/`as(u64)`、`=` 复合算术、`& \| ^` 同层、`.shl/.shr`、整除取余） | PASS |
| `av.parity.1079.overflow-is-not-wrap` | 同上 | run / `av.trap.int-overflow`（i32 MAX+1 不 wrap） | PASS（trap） |
| `av.parity.1079.text-nul-and-format` | `language/text/interpolation-unicode.case.ad` | exit 42 + stdout `A\nB\n`（`len`=4/`len_bytes`=9/NUL/astral/FStr 源序恰一次/Str 相等/负数显示） | PASS |
| `av.parity.1079.text-literal-escapes` | 同上 | exit 42（`\n`、`\t\\\"`、`\u{4e2d}`、`to_codepoint` 十进制与 `0x4E2D` 双拼写） | PASS |

- 固定组 `case-av-language-parity` 复跑 8/8（1078 的 3 例 + 本包 5 例），
  单例 0.45–0.54s。
- 位型补充观察（父侧 Rust 类型化观测，`av/tests/cases/plan1079_language_semantics.rs`）：
  运行时 `x % ±0.0` 产生 NaN（自不等/无序/不 trap）、`-0.0` 经取负/相加/f64→f32
  收窄保持符号位、f64→f32 半程按 ties-to-even、`i64→int` 越界 trap
  `av.trap.int-overflow`、混宽 `source.type-mismatch`、超宽度字面量
  `source.literal-range`。

### 5.3 具名边界（未交付；owner + 关闭条件）

| 形态 | 实际观察 | owner | 关闭条件 |
|---|---|---|---|
| 复合赋值语句 `x += …` | `source.syntax`（`expected expression, found =`）；Auto 语句面只收 `=`，同一 checked 加法经 `x = x + 1` 已验 | PLAN-1080（调用与管道语句面） | 语句面接纳该形后按原 case oracle 重跑 |
| `char.to(Str)`（char→owned Str 装箱） | CST 前门拒绝（回退到 legacy 解析器报 `fn is not part of this source profile`） | PLAN-975 后续批次 | 该形可编译后按 `len`/`len_bytes` 断言 |
| 计算借用视图局部量 `let t = s.slice(1,3)` | `verify.local-type`（aggregates 面局部量只收 owned `Str`） | PLAN-1086 | 借用视图局部量准入后按 `slice` 原 oracle 重跑 |
| `str` 形参收 owned `Str` 实参 | `source.type-mismatch`（`expected str, found Str`） | PLAN-1086 | 形参视图/自有转换准入后重跑 |
| `~` 位非、raw string 字面量 | 不在冻结算符表 / 无此形（`text.invalid-escape`） | — | 语言不定义该构造，旧义务记 `no-such-shape` |

### 5.4 AVP-01 承接的 L 行验收状态（AC-04 映射）

路线设计 `docs/design/strategy/av-autovm-language-parity.md` §6 把下列 L 行指派给 AVP-01。
每行给 owner / 预期 / 当前状态 / actual 证据；子项缺失的行在状态列具名，不静默省略。
映射数据行见 `crates/auto-lang/test/cases/language/migration/legacy-map.tsv`（L-01/02/03/05/49）。

| L 行 | 预期（已定语义） | 状态 | actual 证据（本包真实运行） |
|---|---|---|---|
| L-01 整数字面量与后缀（i32/uint/i8/u8/i64/u64） | 六种宽度与后缀可解码，`int`=i32 | **mapped** | canonical `av.parity.1079.wide-exact`（`0xFFu8`/`0b1010u8`/`1_000_000`/`9007199254740993i64`/i64::MIN 折叠）+ `av.parity.1079.mixed-width-explicit`（`5b`/`7u`/`7u32`）；`plan1079_language_semantics::numeric_profile_suffix_vocabulary_decodes_exactly` 精确钉住 `0x1Fi8`→I8 31、`0b101b`→U8 5、`1_000_000u32`→U32、`0xFFu8`→U8、`0xFF`→I32。legacy `int=i64` 与隐式晋升是具名差异。 |
| L-02 float/double 字面量与 NaN/-0 | `float`=f32、`double`=f64，NaN 无字面量语法，-0 位型保留 | **mapped** | `av.parity.1079.mixed-width-explicit`：`1.5f32`/`1.5d` 各自宽度解码、`(1.5f32).as(f64)` 精确加宽、`-0.0d == 0.0d`；位型由 `negative_zero_bits_survive_arithmetic_and_narrowing`（-0 经取负/相加/f64→f32 收窄）与 `nan_from_zero_modulo_is_a_value_not_a_trap`（运行期 `x % ±0.0` 得 NaN）观测。 |
| L-03 byte/char/CStr 字面量 | char 为一标量；byte 为 u8；CStr 属旧拼写 | **mapped（partial）** | `av.parity.1079.text-literal-escapes`：`'中'`/`'\n'`/`'\u{4e2d}'` 与 `to_codepoint`；`av.parity.1079.mixed-width-explicit`：`5b` byte 字面量。**CStr 子项 no-such-shape**（ACE `TypeWord` 无 CStr 拼写），C 字符串互操作归 1093 foreign surface，不计 PASS。 |
| L-05 位运算/移位/符号扩展 | `& \| ^` 同层、`.shl/.shr` 有界、符号/零扩展 | **mapped** | `av.parity.1079.mixed-width-explicit`：`((0xF0&0x3C)\|0x01) = 0x31`（同层判别）、`.shl(5u32)`/`.shr(5u32)`、`(7u8).as(u64)` 零扩展；符号扩展 `(-1i8).as(i64)` 与宽位值由同执行端的 961 `av_numeric_semantics` v08 行覆盖。 |
| L-49 cast `as` / 转换 `to` | 窄化 checked、浮点↔整数 checked、混宽需显式 | **mapped** | `av.parity.1079.mixed-width-explicit`（`to(int)`/`as(i64)`/`as(u64)`）+ `av.parity.1079.overflow-is-not-wrap`（i32 MAX+1 → `av.trap.int-overflow`）；`conversions_are_checked_and_mixed_widths_are_source_rejections`：`i64→int` 越界 trap、混宽 `source.type-mismatch`、超宽度字面量 `source.literal-range`。 |

复跑入口：`cargo t case-av-language-parity`（AV 视图，选中 8 行）与
`cargo test -p ace-av --test conformance plan1079`（7 例类型化/位型 oracle）。
