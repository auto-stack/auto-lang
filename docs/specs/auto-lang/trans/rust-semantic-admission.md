# 旧 A2R 静态语义准入

> 状态：implemented（PLAN-916）。实现位于
> `crates/auto-lang-core/src/trans/rust/semantics.rs`，不是 ACE worker、M1 case 特判或新 HIR-A2R。

## 管线位置与公共入口

旧 Rust 生产入口在 parser 产出原始 AST 后、CTE 与去死代码/发射前执行共享静态准入。未通过时直接返回 `AutoError`，不创建 accepted Rust artifact，也不启动 rustc 或目标程序。以下入口执行同一义务：

- `transpile_rust`、`transpile_rust_with_source_dir`、`transpile_rust_with_siblings`、`transpile_rust_full` 与 `transpile_rust_full_with_options`；
- `transpile_rust_project` 与 `transpile_rust_project_merged`，先形成跨模块共享 `TypeStore`，再逐模块检查；
- TestCase `a2r/emit` 只调用上述生产 API，并沿用 `classify_staged` 严格父判。CLI/SDK 同样经过这些公共入口，没有 adapter 专用过滤器。

项目 parser 仍可为完整声明解析使用宽松模式，但宽松解析不等于准入：后续共享检查不可绕过。项目语法/语义错误保持原始 typed 错误，不再压成普通 `Msg` 冒充基础设施失败。

## 七族义务

| 语义族 | 准入规则 | 当前 typed face |
|---|---|---|
| resolve | 不可达分支中的普通直接调用也解析；声明前向引用、合法 import、内建及动态构造按真实可见域处理 | `NameError::UndefinedFunction` / `auto_name_E0204` |
| bool/int | 算术运算不把 bool 与数值类型统一 | `TypeError::Mismatch` / `auto_type_E0101` |
| assign-immutable | `let`/`const` 绑定不可经 `=` 或复合赋值重写；`var` 合法 | `NameError::ImmutableAssignment` / `auto_name_E0203` |
| missing-return | 非 void 函数必须有保证返回路径、合法尾值或 generator/result-unit 形态；`if false { return ... }` 不覆盖其余路径 | `TypeError::Mismatch` / `auto_type_E0101` |
| borrow-escape | 局部 owned `Str` 转为 `str` 或局部 view 不得从函数逃逸；合法 owned 返回不受影响 | `TypeError::Mismatch` / `auto_type_E0101` |
| nominal-type | 同布局不同名的用户类型不可互作函数实参；构造 `Node`/`Call` 均恢复声明名义类型 | `TypeError::Mismatch` / `auto_type_E0101` |
| use-after-move | 显式 `.move`/`.take` 后不得读取旧 owner；新 owner 合法，分支 move 保守合并 | `TypeError::InvalidOperation` / `auto_type_E0102` |

这些检查是高置信、可区分的静态义务，不是“完整 Auto 类型检查器已完成”。旧 `infer` 对闭包、Option、生成器等仍有未知/不完全推导，准入不得把其所有诊断整包提升为拒绝。不得按 case ID、文件名、源片段或 `missing` 字面量特判。

## 可见域与兼容边界

- 同文件函数、类型、enum variant、union/tag、参数、局部绑定及项目共享声明进入可见域；普通未声明调用即使位于 `if false` 也拒绝。
- `use.rust`、`use.c`、模块 import、task 消息及动态构造保持旧 A2R 已授权边界；这些边界存在时，不把未知外部自由函数泛拒为语言错误。
- 内建输出、断言、字符串 helper 和运行时构造不算未声明用户函数。
- 尾值可以是表达式、`is`、`Node` 或全分支 `if`；`!void`、`Future`/`Stream` generator 不套用普通 int 返回规则。
- 修复后重新调用真实入口会重新解析并生成新产物；没有按源文本复用旧 PASS 或旧 artifact 的语义缓存。

## 诊断与失败原子性

`NameError::UndefinedFunction` 的 Display 必须包含实际符号名（例如
`undefined function: type_of`）；API 生成包装器同时保留 `library.at` 等模块名。
不能只打印族名后把定位责任交给使用方。code/span/help/suggestion 的 typed 身份保持。

### Vue 后端选择与 VM 专有能力（PLAN-955）

Vue 是前端选择，不意味着后端自动使用 VM。CLI `run` 缺省 server 为 Rust；
`--no-merge` 只选择分离模式。含 VM 专有 native 的应用应显式
`auto run --server vm`，生成 TS client 后由 VM HTTP 服务执行后端。
`type_of(x)` 当前只有 VM native 注册，裸名 A2R 无映射；不能因 VM 能执行
就将其列为 A2R 已支持能力，也不能把它的失败推断为所有跨模块 import 失效。
auto-reader 的 `use pathx: json_escape` 在共享 API 转译入口正常准入；
其 `library.at` 首个被拒调用是 `type_of`，而非 `json_escape`。

`#[test]` 是函数的测试发现标记，不是生产模块链接的条件排除指令。
Parser 保留函数及 `is_test`，VM codegen/loader 会处理其函数体与引用，
因此未执行的测试函数仍须符合模块可见性。跨模块测试 helper 须显式
`use helper: expect` 或使用限定名，不能依赖其他模块的裸名导出；
需要从产品装配排除的测试应放在独立测试源/入口。该规则来自现有模块
链接规则，PLAN-955 不引入新的 `#[test]` 排除或导入语义。

诊断以 `AutoError::{Name,Type,Syntax,MultipleErrors}` 传播真实 stage/code/span；工具 panic、I/O、资源超限和普通 `Msg` 不得满足语言负例。adapter 只在成功时持久化 accepted actual；失败时 artifact ref 为空，已有外部产物不由 API 覆盖。Rust 编译/运行拒绝不替代 Auto 前端拒绝。

## 验证与边界

- 原 B16/B17/B18/B19/T06/R09/R10 源不变，single/project 均拒绝；E03 复用 B16 与 B04/B20 对照，不伪造独立源。
- `case-ace-m1-a2r-semantics-916` 含 14 个 typed 负例检查和 NB17/NB18/NB19 三个独立 emit reference；NB 的 2/2/0 行为值由既有 908/911 独立套件认证，emit reference 本身不冒充运行时认证。
- 完整 `a2r` 组保护现有合法单文件、项目、泛型、import、FFI、闭包、Option 和 generator。`case-ace-m1-base`、`case-ace-m1-nb-diags-908` 与 `infra-ace-m1` 保持原分母和反控。
- R07/T08 发射、38 格复合认证、旧 AVM B18、ACE 共享前端和通用 checker 重写不在本规范承诺内。
