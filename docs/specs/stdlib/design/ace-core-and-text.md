# ACE 基础数学与文本标准库切片（PLAN-945 交付稿）

> 状态：PLAN-945 交付稿（worktree 预备；merge 时正式沉淀）。四端 implemented
> 状态以 `experimental/ace/crates/av/tests/cases/stdlib_core_945.rs` 的真实
> 执行（AV 值 / mn 退出码 / rustc exe stdout / cl exe stdout，独立手算 oracle）
> 为证据；i64 公共族按"具名未支持"分账，不冒领、不窄化。

## 1. 冻结公共 std 包（唯一单源）

- **源与身份**：`stdlib/ace/std/math.at`、`stdlib/ace/std/text.at`（包 `std`、
  版本 `1.0.0`）；生产装载入口 `ace_source_query::modules::packages::stdlib_inputs()`
  （`include_str!` 嵌入同两文件，身份=仓库文件内容本身）。包名 `std` 由
  `ace_core::source::packages::stdlib` 保留：非逐字节同源（模块 id/版本/源文本
  任一不符）的 `std` 输入在绑定前 `ace.stdlib-reserved` 拒绝（冷/恢复双路径），
  防用户包遮蔽冒充公共绑定。
- **公开面**（全部经统一 packages 前门 → TextRecord profile → 共享 HIR → 四端，
  无平行语义、无 legacy fallback）：

| 公共拼写 | 冻结合同 | 语义（独立 oracle） |
|---|---|---|
| `std.math.square(x int) int` | math.private_square 公共包装（math.at 私有体镜像，checked i32） | `3→9`；`square(46341)` i32 溢出=trap（AV `IntOverflow`/mn exit 70/rust、c exe `trap:Overflow`），绝不回绕 |
| `std.math.cube(x int) int` | math.private_cube 同上 | `3→27`，同 checked 溢出语义 |
| `std.text.byte_len(s str) int` | text.byte_length 公共绑定（=`s.len_bytes()`） | UTF-8 字节：`A中😀`→8、`e\u{0301}`→3、`""`→0、`a\0b`→3 |
| `std.text.equal(a str, b str) bool` | text.equal 公共绑定（语言算子 `==`） | 逐字节精确相等；NUL 是内容非终结符；`a`≠`a\0` |
| `std.text.owned(s str) Str` | text.clone 公共绑定（视图接收者 `clone()`→独立拥有副本） | 副本独立：源覆写/释放后副本仍可读；分配失败保输入 |

- legacy 身份不变：`math.at` square/cube 仍私有、`abs/min/max` 仍 i64 公共声明、
  `str.rs.at::str_byte_len` 仍是 rust 层 native-only 绑定；本切片只新增公共包装，
  不改写旧层。
- **i64 具名未支持**：`std.math` 上 `abs/min/max` 引用（use/限定调用/限定值）
  在绑定层 `ace.stdlib-unwired` 具名拒绝（消息含 i64 与"无窄化适配"），先于任何
  调用/dispatch；普通拼写错误仍走 `source.module-symbol`。用户自有同名函数按其
  自有签名合法（packages 通道 961 数值族属用户域）。943 合同行 `math.abs/min/max`
  保持四端 `missing_capability`，gap 记该具名拒绝。

## 2. 四端分账（本切片）

| 端 | 状态 | 证据/边界 |
|---|---|---|
| AV | supported | `stdlib_core_945` P1/P2/P3 四程序独立 oracle（8/181703/3927）+ 溢出 trap + `make()` 返回拥有 `Value::Str` 观察；arena 故障扫描：每次注入=typed abort（Infra fault-injected）且零存活缓冲 |
| native (mn) | supported | 同程序真实 PE 退出码 oracle（8/181703/3927/溢出=70）；clone 分配失败=文本运行时 ABI 级真 PE 探针（`native/tests/cases/text_clone_fault_945.rs`：fail_nth 注入→`ac_rt_own_clone` NULL+源完好+账本收敛）。**程序级 clone 故障注入=命名边界**（owned wrapper 无故障旋钮，旋钮仅在 collections wrapper） |
| A2R (Rust host) | supported | rustc exe `value:*`/`trap:*` 协议；`A2X_RT_FAULT_ALLOC=1` → `trap:Resource`（fault 计数=exe 内真实分配序） |
| A2C (C host) | supported | cl exe 同协议同 oracle；`A2X_RT_FAULT_ALLOC` env 同语义 |

机器账：`stdlib/contracts/compatibility.toml` 中 `text.byte_length`/`text.equal`/
`text.clone`/`math.private_square`/`math.private_cube` 四端 `supported`（evidence 指
`stdlib/ace/std/*.at` 与双 conformance 套件），`stdlib_assembly::contracts` 钉扎
测试锁定；`math.abs/min/max` 保持 `missing_capability`。旧 a2r-std 适配只读核对：
`a2r_std::str_byte_len` 为 UTF-8 字节但 **i64** 宿主宽（旧 a2r/i64 宿主面），与本
切片 i32 域并列保留，不统一、不改 a2r-std。

## 3. 资源与恢复（本切片实测）

- clone 分配失败（V-04）：AV arena seam 全尝试扫描（typed abort、零泄漏、输入保
  存）；rust/c exe env 故障=typed `trap:Resource`；native ABI 探针（NULL+源完好+
  invariant 0）。失败均为类型化结果，无 panic 代 trap。
- 恢复增量（D5）：app body 编辑→仅 `main` 段 hash 变化、std 段逐字节稳定；重编译
  新值（8→9）；a2x restore `lower=0/print=0` 文件集全等；packages root 身份持久化
  +篡改收据拒绝（沿 960 机制复用）。
- manifest/reference proof（D7）：真实 stdlib 声明（六 Declaration）进 packages
  profile SemanticAssembly，四 producer freeze/validate/admit 全通过、篡改声明
  `assembly.stale-identity` 拒、错 producer `assembly.producer` 拒；AV+Rust 真执行
  （98 oracle）。packages 通道 manifest 耦合 cold assembly 仍为 960 命名后继，
  准入与执行分开断言、不混账。

## 4. 阶段与边界

- 本切片=M2-A（Design55 §16）：能力+永久测试均在 A 侧；B 消费由 985/A 接。
- M1-TAG 来源链：tag `v0.6-ACE-M1`（commit b923362f4，tag object acc3eb5b7）；
  依赖 943/944/947/960 archived；实施授权=/goal（见 PLAN-945 §5 T-01）。
- 不在本切片：i64 `abs/min/max`（948 交接）、float 族（`math.abs_f` 等宽域调查，
  972）、`len` 歧义族统一与 `parse_int` 差异（兼容账保留）、旧 VM/a2r 全文金样
  不充新侧 oracle。若条件以 ident 收尾再接 `{`，packages/text-record 面需括号
  （`if (a < b) {`）——record 构造歧义为该面既有语法特性，非本包新增。
